# C 线（260911-05）维度覆盖 A/B：nether + end（overworld 已验）
# 复用 A2 的多维度驱动（.tmp/a2-260911-05/run_dim.ps1，带 -World），双臂对称开诊断。
# 判据：Tier 1（方块状态）+ Tier 2（派生字段）逐 chunk 全等；Tier 4 同 run 对称计时。
$ErrorActionPreference = "Continue"
$drv = "E:\PYTHON\CoreSwap\.tmp\a2-260911-05\run_dim.ps1"
$dll = "E:\PYTHON\CoreSwap\.tmp\a1-260911-05\worldgen-a1-new.dll"
$want = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
Write-Output "===== C 维度覆盖 dll=$($want.Substring(0,16)) start $(Get-Date -Format 'HH:mm:ss') ====="

foreach ($w in @(@("minecraft:the_nether", "nether"), @("minecraft:the_end", "end"))) {
    $world = $w[0]; $tag = $w[1]
    $env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=0 -Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1"
    Write-Output "===== ARM c-$tag-old (bulkwb=0, $world) ====="
    & pwsh -NoProfile -File $drv -Dll $dll -Tag "c-$tag-old" -Form async -World $world -Radius 160

    $env:CORESWAP_EXTRA_JVM = "-Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1"
    Write-Output "===== ARM c-$tag-bulk ($world) ====="
    & pwsh -NoProfile -File $drv -Dll $dll -Tag "c-$tag-bulk" -Form async -World $world -Radius 160
}

Copy-Item $dll "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Force
$now = (Get-FileHash "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Algorithm SHA256).Hash.ToLower()
Write-Output "restored target dll = $($now.Substring(0,16)) (want $($want.Substring(0,16))) $(if($now -eq $want){'OK'}else{'MISMATCH'})"
Write-Output "===== done $(Get-Date -Format 'HH:mm:ss') ====="
