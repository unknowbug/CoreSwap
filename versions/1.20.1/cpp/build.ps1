# build.ps1 —— CoreSwap worldgen 可靠构建（cl + lib 直链，替代 ninja）
#
# ⚠️ 为什么不用 ninja：本机沙箱下 ninja 的「spawn 子进程 + 捕获 /showIncludes 管道」挂起（120s 无输出）。
#   cl + lib 直链已验证可靠（单文件 3-10s，整链 ~12s）。ninja 版本污染另见 build-toolchain-diagnosis.md
#   （Python312\Scripts\ninja.exe ≥ pip 装的 1.11.1 遮蔽了 VS 官方 1.13.2）。
#
# ⚠️ 构建铁律（用户强制，见顶层 CMakeLists.txt）：
#   - 严格 MSVC（cl.exe）——禁止 MinGW！MinGW -static 下 thread_local 退化 → 跨线程共享缓存 → 堆损坏 0xC0000005
#   - 源码 UTF-8（含中文注释）；/utf-8 /DNOMINMAX /EHsc
#
# 用法：
#   pwsh build.ps1                      # 构建 worldgen_core.lib + block_probe + bench_chunks
#   pwsh build.ps1 -Target bench_chunks # 只构建指定 exe
#   pwsh build.ps1 -All                 # 构建全部 CMake 目标
#
# 需要的环境：JAVA_HOME（jni）、VULKAN_SDK（GPU）。未设置 VULKAN_SDK → CPU-only（无 CORESWAP_GPU_ENABLED）。

param(
    [string]$Target = "",     # 空 = 默认（block_probe + bench_chunks）
    [switch]$All,             # 构建全部
    [switch]$Clean            # 清理 .obj/.lib
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$src  = Join-Path $root "worldgen\src"
$inc  = Join-Path $root "worldgen\include"
$gpu  = Join-Path $root "worldgen\gpu-assets"
$buildDir = Join-Path $root "build-msvc"
$bin  = Join-Path $buildDir "bin"

# 工具链
$vcvars   = "D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
$vulkan   = $env:VULKAN_SDK
if (-not $vulkan) { $vulkan = "C:\VulkanSDK\1.4.357.0" }
$vulkanLib = Join-Path $vulkan "Lib\vulkan-1.lib"

# worldgen_core 的源文件（.obj 装入静态库）
$coreSrcs = @(
    "worldgen.cpp", "md5.cpp", "worldgen_api.cpp", "gpu_density_engine.cpp"
)

# 常见验证工具（exe = obj 名）
$exes = @("block_probe", "bench_chunks", "noise_probe", "density_probe", "router_probe",
          "ore_probe", "got_export", "tbands_dump", "chunkrandom_test", "conc_density_probe",
          "conc_sample_probe")

function Invoke-VcCl {
    param([string]$Extra)
    $env:VSCMD_SKIP_SENDTELEMETRY = "1"
    cmd /c "call `"$vcvars`" >nul 2>&1 && $Extra"
}

if (-not (Test-Path $bin)) { New-Item -ItemType Directory -Path $bin -Force | Out-Null }
if ($Clean) { Get-ChildItem $bin -Include "*.obj","*.lib","*.exe" -ErrorAction SilentlyContinue | Remove-Item -Force }

# 公共 include + 编译选项
$commonInc = "/I`"$inc`" /I`"$src`""
$commonDef = "/DNOMINMAX /DCORESWAP_GPU_ENABLED=1"
$commonOpt = "/nologo /c /EHsc /utf-8 /std:c++17 /MD /O2"
if ($vulkan) { $commonInc += " /I`"$vulkan\Include`" /I`"$gpu`"" }

Write-Host "[build] worldgen_core ..." -ForegroundColor Cyan
foreach ($s in $coreSrcs) {
    $base = [System.IO.Path]::GetFileNameWithoutExtension($s)
    $obj = Join-Path $bin "$base.obj"
    $srcPath = Join-Path $src $s
    $cmd = "cl $commonOpt $commonDef $commonInc /Fo`"$obj`" `"$srcPath`" > `"$bin\bld_$base.txt`" 2>&1"
    Invoke-VcCl $cmd | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] $s" -ForegroundColor Red; Get-Content "$bin\bld_$base.txt" | Select-Object -Last 3; exit 1 }
    Write-Host "  [OK] $s.obj" -ForegroundColor Green
}

# 打包静态库
$libOut = Join-Path $bin "worldgen_core.lib"
$objs = $coreSrcs | ForEach-Object { Join-Path $bin "$([System.IO.Path]::GetFileNameWithoutExtension($_)).obj" }
$libCmd = "lib /nologo /out:`"$libOut`" " + ($objs | ForEach-Object { "`"$_`"" }) -join " "
Invoke-VcCl "$libCmd > `"$bin\bld_lib.txt`" 2>&1" | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] lib link" -ForegroundColor Red; Get-Content "$bin\bld_lib.txt" | Select-Object -Last 3; exit 1 }
Write-Host "  [OK] worldgen_core.lib" -ForegroundColor Green

# 链接可执行（需要 Vulkan 库——GPU 集成都需要）
function Build-Exe {
    param([string]$Name)
    $exeObj = Join-Path $bin "$Name.obj"
    if (-not (Test-Path $exeObj)) { Write-Host "  [SKIP] $Name (no .obj)" -ForegroundColor DarkGray; return }
    $exeOut = Join-Path $bin "$Name.exe"
    $vLib = if ($vulkan -and (Test-Path $vulkanLib)) { "`"$vulkanLib`"" } else { "" }
    $cmd = "cl /nologo /EHsc /utf-8 /std:c++17 /DNOMINMAX /MD /O2 `"$exeObj`" `"$libOut`" $vLib /Fe:`"$exeOut`" > `"$bin\bld_$Name.txt`" 2>&1"
    Invoke-VcCl $cmd | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Host "  [FAIL] $Name" -ForegroundColor Red; Get-Content "$bin\bld_$Name.txt" | Select-Object -Last 3; exit 1 }
    Write-Host "  [OK] $Name.exe" -ForegroundColor Green
}

$targets = if ($All) { $exes } elseif ($Target) { @($Target) } else { @("block_probe", "bench_chunks") }
foreach ($t in $targets) { Build-Exe $t }

Write-Host "`n[build] done ✔  lib=$libOut" -ForegroundColor Green
