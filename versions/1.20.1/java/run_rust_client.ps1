# run_rust_client.ps1 — 用纯 Rust worldgen 作为 mod 运行 Minecraft 客户端/服务端
# 纯 Rust 单 dll 时代（2026-08-30 起）：WorldgenRust.dll 自身导出 Java_wg_CppWorldgen_*（jni_bridge.rs）
# + wg_* C ABI（api.rs），不再需要 C++ JNI 桥 / rust-dll 双文件 / CPP_RUST_LIB。
# 用法：
#   pwsh run_rust_client.ps1                # 客户端（默认，带 Rust worldgen mod）
#   pwsh run_rust_client.ps1 -Server        # 服务端
#   pwsh run_rust_client.ps1 -Rebuild       # 先 cargo build --release 再跑
#   pwsh run_rust_client.ps1 -Vanilla       # 无 mod（纯 vanilla worldgen）——A/B 实机对比用
#   pwsh run_rust_client.ps1 -PureVanilla -Seed 8576294172403134396
#       # 真·纯 vanilla：绕开 gradle/loom/fabric-loader，直接跑官方 minecraft_server 1.20.1 jar
#       # （-PcppVanilla 仍走 loom dev 运行时 = fabric-loader+fabric-api 在场，不算纯 vanilla）
#       # 产物目录 runtime\1.20.1\vanilla-server\run\，用户可用官方启动器（无 mod 档）连入目测
# 注意：需图形环境（客户端窗口）；gradle home 在 CoreSwap（免提权）；
#       cargo 下载/编译依赖需提权（沙箱 TLS 限制），-Rebuild 首次或依赖变更时在提权终端跑。

param(
    [switch]$Server,
    [switch]$Rebuild,
    [switch]$Vanilla,       # 无 mod：不传 cppReplace/cppLib/cppWorldgenDir，走 vanilla worldgen（仍走 loom/fabric-loader）
    [switch]$PureVanilla,   # 真·纯 vanilla：官方 server jar，无 gradle/无 loader
    [string]$Seed = ""      # -PureVanilla 用：写入 server.properties level-seed
)

$ErrorActionPreference = "Stop"

# ── -PureVanilla：真·纯 vanilla 服务器（无 gradle/loom/fabric-loader）──────────
if ($PureVanilla) {
    $vsDir = "E:\PYTHON\CoreSwap\runtime\1.20.1\vanilla-server"
    $runDir = Join-Path $vsDir "run"
    $jar = Join-Path $vsDir "server.jar"
    New-Item -ItemType Directory -Force -Path $runDir | Out-Null

    # JDK17 与 loom 侧同源
    $env:JAVA_HOME = "D:\Program Files\Java\jdk-17.0.12"
    $env:Path = "$env:JAVA_HOME\bin;" + $env:Path

    # server.jar 不在则从 piston-meta 解析官方 1.20.1 下载地址拉取（sha1 校验）
    if (!(Test-Path $jar)) {
        Write-Host "=== 下载官方 minecraft_server 1.20.1（piston-meta）===" -ForegroundColor Cyan
        $manifest = Invoke-RestMethod "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
        $verMeta = ($manifest.versions | Where-Object { $_.id -eq "1.20.1" }).url
        $ver = Invoke-RestMethod $verMeta
        $dl = $ver.downloads.server
        Invoke-WebRequest -Uri $dl.url -OutFile $jar
        $sha = (Get-FileHash $jar -Algorithm SHA1).Hash.ToLower()
        if ($sha -ne $dl.sha1) { Write-Error "sha1 不匹配: $sha != $($dl.sha1)"; return }
        Write-Host "[OK] sha1 校验通过: $sha"
    }

    # eula + server.properties（幂等写入；level-seed 覆盖需删 run\world 才生效——seed 三查铁律）
    Set-Content (Join-Path $runDir "eula.txt") "eula=true" -Encoding Ascii
    $props = @()
    if (Test-Path (Join-Path $runDir "server.properties")) { $props = Get-Content (Join-Path $runDir "server.properties") }
    $props = @($props | Where-Object { $_ -notmatch "^(online-mode|level-seed)=" })
    if ($Seed -ne "") {
        $props += "level-seed=$Seed"
        Write-Host "level-seed = $Seed（若 run\world 已存在 MUST 先删除，否则旧 seed 的 level.dat 仍生效）" -ForegroundColor Yellow
    }
    $props += "online-mode=false"
    Set-Content (Join-Path $runDir "server.properties") $props -Encoding Ascii

    Write-Host "=== 运行纯 vanilla 1.20.1 服务器（无 loader）run=$runDir ===" -ForegroundColor Green
    Write-Host "目测核对：官方启动器选纯 1.20.1 profile（无 mods）→ 多人直连 127.0.0.1 → /forceload add <x1> <z1> <x2> <z2>"
    Push-Location $runDir
    try { & "$env:JAVA_HOME\bin\java.exe" -Xmx4G -jar $jar --nogui }
    finally { Pop-Location }
    return
}

# 纯 Rust 单 dll（Java_wg_CppWorldgen_* + wg_* 同体导出）
# 260905-01 workspace 拆分后：dll = 根 workspace target/release/worldgen.dll（包名 worldgen，cdylib）
$rustDll = "E:\PYTHON\CoreSwap\target\release\worldgen.dll"
$worldgenDir = "E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen"  # worldgen JSON 数据目录
$runJava = "E:\PYTHON\CoreSwap\versions\1.20.1\java"  # gradle mod 工程

# JDK17（loom 要求；PATH 默认 java 可能是 24）
$env:JAVA_HOME = "D:\Program Files\Java\jdk-17.0.12"
$env:Path = "$env:JAVA_HOME\bin;" + $env:Path

# 可选：先重编 Rust dll（260905-01 拆分后 = 根 workspace -p worldgen；沙箱/网络限制用 --offline）
if ($Rebuild) {
    Write-Host "=== cargo build --offline -p worldgen --release ===" -ForegroundColor Cyan
    Push-Location "E:\PYTHON\CoreSwap"
    try { cargo build --offline -p worldgen --release; if ($LASTEXITCODE -ne 0) { Write-Error "cargo build 失败"; return } }
    finally { Pop-Location }
}

# 校验 dll 存在
if (!(Test-Path $rustDll)) { Write-Error "缺少 dll: $rustDll（先 cargo build --release，或用 -Rebuild）"; return }

# gradle home 指向 CoreSwap（native-platform/依赖缓存在内）
$env:GRADLE_USER_HOME = "E:\PYTHON\CoreSwap\.gradle"

# 切到 mod 工程（260910-07 起 = versions/1.20.1/java；运行环境在 runtime/1.20.1/java/run）
Push-Location $runJava
try {
    if ($Vanilla) {
        Write-Host "=== 运行 vanilla（无 mod worldgen）Minecraft $(if ($Server) { '服务端' } else { '客户端' }) ===" -ForegroundColor Yellow
        if ($Server) { gradle runServer "-PcppVanilla=true" }
        else { gradle runClient "-PcppVanilla=true" }
        return
    }
    Write-Host "=== 运行 纯 Rust worldgen Minecraft $(if ($Server) { '服务端' } else { '客户端' }) ===" -ForegroundColor Cyan
    Write-Host "Rust dll: $rustDll ($((Get-Item $rustDll).Length) bytes)"
    Write-Host "数据目录: $worldgenDir"
    Write-Host ""
    # -PcppReplace=true → -Dcpp.replace=1（启用 CppBridge 替换 vanilla worldgen）
    # -PcppLib → -Dcpp.worldgen.lib（直接 System.load 纯 Rust dll，绕过 jar 解压/缓存，改 dll 后无需 gradle 重打包）
    # -PcppWorldgenDir → -Dcpp.worldgen.dir（worldgen 数据目录）
    # 注意：gradle.bat 传含反斜杠路径的 -P 参数必须加引号（防批处理吃掉路径）。
    $args2 = @(
        "-PcppReplace=true",
        "-PcppLib=$rustDll",
        "-PcppWorldgenDir=$worldgenDir"
    )
    if ($Server) { gradle runServer @args2 }
    else { gradle runClient @args2 }
}
finally {
    Pop-Location
}
