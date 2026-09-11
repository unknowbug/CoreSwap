# C 线（260911-05）判后补跑：judge M1 指出的**唯一缺失对照** = bulk × bulk（同实现，同 dll，唯一变量 = run）
#   + 一个 wbcheck 臂：独立实测「section 入口全空气」前提（judge M3 指该前提六臂均未实测）
# 判据（judge 给的最小上呈条件）：若 bulk×bulk 切片 A ≈ 0.024-0.028% ⇒ M1 降为「噪声底内」；
#   若 ≈ 0.0348% ⇒ 须把「写回在存档层有可测影响」正式登记并触发 fan-out。
$ErrorActionPreference = "Continue"
$drv = "E:\PYTHON\CoreSwap\.tmp\vivo-stutter-260911-02\ab-threads\run_ab.ps1"
$dll = "E:\PYTHON\CoreSwap\.tmp\a1-260911-05\worldgen-a1-new.dll"
$out = "E:\PYTHON\CoreSwap\.tmp\c-260911-05"
$world = "E:\PYTHON\CoreSwap\runtime\1.20.1\java\run\world"
$want = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
Write-Output "===== C post-judge: bulk x2 + wbcheck  dll=$($want.Substring(0,16))  start $(Get-Date -Format 'HH:mm:ss') ====="

# ① bulk ×2：与 c-bulk 完全同参数（bulkwb 默认开 + wbcontent），唯一变量 = run
foreach ($tag in @("c-bulk2", "c-bulk3")) {
    $env:CORESWAP_EXTRA_JVM = "-Dcoreswap.wbcontent=1"
    Write-Output "===== ARM $tag (bulkwb=1, bulk 同实现 run) ====="
    & pwsh -NoProfile -File $drv -Dll $dll -Tag $tag -Form async -Radius 160
    if (Test-Path $world) {
        $dst = "$out\world-$tag"
        Remove-Item $dst -Recurse -Force -EA SilentlyContinue
        Copy-Item $world $dst -Recurse -Force
        Write-Output "[$tag] archived world-$tag region files = $((Get-ChildItem "$dst\region" -File -EA SilentlyContinue | Measure-Object).Count)"
    }
}

# ② wbcheck 臂（老路径 + 自检门）：独立实测「跳过的空气写 = 当前格恰为 Blocks.AIR」
$env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=0 -Dcoreswap.wbcheck=1 -Dcoreswap.wbcontent=1"
Write-Output "===== ARM c-wbcheck (old + wbcheck=1) ====="
& pwsh -NoProfile -File $drv -Dll $dll -Tag "c-wbcheck" -Form async -Radius 160

Copy-Item $dll "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Force
$now = (Get-FileHash "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Algorithm SHA256).Hash.ToLower()
Write-Output "restored target dll = $($now.Substring(0,16)) (want $($want.Substring(0,16))) $(if($now -eq $want){'OK'}else{'MISMATCH'})"
Write-Output "===== done $(Get-Date -Format 'HH:mm:ss') ====="
