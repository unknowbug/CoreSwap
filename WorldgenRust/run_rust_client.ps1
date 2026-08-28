# run_rust_client.ps1 — 用 Rust worldgen 作为 mod 运行 Minecraft 客户端
# 多线程版 Rust dll（wg_fill_blocks_multi 并行，14.26ms/chunk）。
# 用法：pwsh run_rust_client.ps1
# 注意：需图形环境（Minecraft 客户端窗口）；gradle 需 danger-full-access（native-platform.dll）。

$ErrorActionPreference = "Stop"

# Rust dll 路径
$rustDir = "E:\PYTHON\CoreSwap\WorldgenRust\rust-dll"
$rustJni = Join-Path $rustDir "worldgen.dll"       # C++ JNI 桥（导出 Java_wg_CppWorldgen_*）
$rustCore = Join-Path $rustDir "WorldgenRust.dll"  # Rust cdylib（导出 wg_* C ABI）
$worldgenDir = "E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen"  # worldgen JSON 数据目录

# 校验 dll 存在
foreach ($d in @($rustJni, $rustCore)) {
    if (!(Test-Path $d)) { Write-Error "缺少 dll: $d（先 cargo build --release 并 Copy-Item 到 rust-dll/）"; return }
}

# 设置 Rust dll 环境变量（JNI 桥 LoadLibrary 找不到 WorldgenRust.dll 时回退）
$env:CPP_RUST_LIB = $rustCore

# 切到 mod 工程
Push-Location "E:\PYTHON\MC\versions\1.20.1\java"
try {
    Write-Host "=== 运行 Rust worldgen Minecraft 客户端 ===" -ForegroundColor Cyan
    Write-Host "JNI 桥: $rustJni"
    Write-Host "Rust 核心: $rustCore"
    Write-Host "数据目录: $worldgenDir"
    Write-Host ""
    # -PcppReplace=true → -Dcpp.replace=1（启用 CppBridge）
    # -PcppLib → -Dcpp.worldgen.lib（加载 Rust JNI 桥 worldgen.dll）
    # -PcppWorldgenDir → -Dcpp.worldgen.dir（worldgen 数据目录）
    gradle runClient `
        -PcppReplace=true `
        -PcppLib=$rustJni `
        -PcppWorldgenDir=$worldgenDir
}
finally {
    Pop-Location
}
