# C 线（260911-05）全量验证臂：Tier 1/2（写回后指纹）+ Tier 2（派生字段）+ Tier 4（对称计时）+ 自检 + Tier 3 世界归档
#   臂1 = -Dcoreswap.bulkwb=0（旧逐块 setBlockState 路径）
#   臂2 = 默认（bulk 原地换容器）
# 两臂**对称开启**同一组诊断（口径对等纪律）：wbcontent + bulkwblog；bulkwbtest 仅 bulk 臂会实际触发。
# 每臂结束后把 run\world 归档，供 Tier 3（region 零差对拍）离线使用——驱动每臂开头会删 world，故必须先归档。
param([int]$Radius = 160)
$ErrorActionPreference = "Continue"
$drv = "E:\PYTHON\CoreSwap\.tmp\vivo-stutter-260911-02\ab-threads\run_ab.ps1"
$dll = "E:\PYTHON\CoreSwap\.tmp\a1-260911-05\worldgen-a1-new.dll"
$out = "E:\PYTHON\CoreSwap\.tmp\c-260911-05"
$world = "E:\PYTHON\CoreSwap\runtime\1.20.1\java\run\world"
New-Item -ItemType Directory -Force -Path $out | Out-Null

$want = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
Write-Output "===== C full-verify  radius=$Radius  dll=$($want.Substring(0,16))  start $(Get-Date -Format 'HH:mm:ss') ====="

$env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=0 -Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1 -Dcoreswap.bulkwbtest=1"
Write-Output "===== ARM c-old (bulkwb=0) ====="
& pwsh -NoProfile -File $drv -Dll $dll -Tag c-old -Form async -Radius $Radius
if (Test-Path $world) {
    Remove-Item "$out\world-old" -Recurse -Force -EA SilentlyContinue
    Copy-Item $world "$out\world-old" -Recurse -Force
    $n = (Get-ChildItem "$out\world-old\region" -File -EA SilentlyContinue | Measure-Object).Count
    Write-Output "[c-old] archived world-old region files = $n"
}

$env:CORESWAP_EXTRA_JVM = "-Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1 -Dcoreswap.bulkwbtest=1"
Write-Output "===== ARM c-bulk (default on) ====="
& pwsh -NoProfile -File $drv -Dll $dll -Tag c-bulk -Form async -Radius $Radius
if (Test-Path $world) {
    Remove-Item "$out\world-bulk" -Recurse -Force -EA SilentlyContinue
    Copy-Item $world "$out\world-bulk" -Recurse -Force
    $n = (Get-ChildItem "$out\world-bulk\region" -File -EA SilentlyContinue | Measure-Object).Count
    Write-Output "[c-bulk] archived world-bulk region files = $n"
}

# 驱动结束会把 target dll 还原为它自己的陈旧 bak（597e12ed）——把本次 dll 放回并核对
Copy-Item $dll "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Force
$now = (Get-FileHash "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Algorithm SHA256).Hash.ToLower()
Write-Output "restored target dll = $($now.Substring(0,16))  (want $($want.Substring(0,16)))  $(if($now -eq $want){'OK'}else{'MISMATCH'})"
Write-Output "===== done $(Get-Date -Format 'HH:mm:ss') ====="
