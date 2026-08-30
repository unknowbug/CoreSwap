# run_rust_client.ps1 — 用纯 Rust worldgen 作为 mod 运行 Minecraft 客户端/服务端
# 纯 Rust 单 dll 时代（2026-08-30 起）：WorldgenRust.dll 自身导出 Java_wg_CppWorldgen_*（jni_bridge.rs）
# + wg_* C ABI（api.rs），不再需要 C++ JNI 桥 / rust-dll 双文件 / CPP_RUST_LIB。
# 用法：
#   pwsh run_rust_client.ps1                # 客户端（默认）
#   pwsh run_rust_client.ps1 -Server        # 服务端
#   pwsh run_rust_client.ps1 -Rebuild       # 先 cargo build --release 再跑
# 注意：需图形环境（客户端窗口）；gradle home 在 CoreSwap（免提权）；
#       cargo 下载/编译依赖需提权（沙箱 TLS 限制），-Rebuild 首次或依赖变更时在提权终端跑。

param(
    [switch]$Server,
    [switch]$Rebuild
)

$ErrorActionPreference = "Stop"

# 纯 Rust 单 dll（Java_wg_CppWorldgen_* + wg_* 同体导出）
$rustDll = "E:\PYTHON\CoreSwap\WorldgenRust\target\release\WorldgenRust.dll"
$worldgenDir = "E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen"  # worldgen JSON 数据目录
$runJava = "E:\PYTHON\CoreSwap\runtime\1.20.1\java"  # gradle mod 工程

# JDK17（loom 要求；PATH 默认 java 可能是 24）
$env:JAVA_HOME = "E:\PYTHON\MC\tools\jdk17\jdk-17.0.20+8"
$env:Path = "$env:JAVA_HOME\bin;" + $env:Path

# 可选：先重编 Rust dll
if ($Rebuild) {
    Write-Host "=== cargo build --release ===" -ForegroundColor Cyan
    Push-Location "E:\PYTHON\CoreSwap\WorldgenRust"
    try { cargo build --release; if ($LASTEXITCODE -ne 0) { Write-Error "cargo build 失败"; return } }
    finally { Pop-Location }
}

# 校验 dll 存在
if (!(Test-Path $rustDll)) { Write-Error "缺少 dll: $rustDll（先 cargo build --release，或用 -Rebuild）"; return }

# gradle home 指向 CoreSwap（native-platform/依赖缓存在内）
$env:GRADLE_USER_HOME = "E:\PYTHON\CoreSwap\.gradle"

# 切到 mod 工程（runtime）
Push-Location $runJava
try {
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
