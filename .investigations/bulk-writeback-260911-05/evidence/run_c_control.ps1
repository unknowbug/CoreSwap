# C 线（260911-05）Tier 3 **同实现对照**：old 臂跑两次（唯一变量 = run 本身）
# 目的：量化 live-server region 层的跨 run 噪声底——用于把「region 差 0 不可达」从论证升级为量化。
# 判据：若 old vs old2 的差分 ≈ old vs bulk（0.0277%）⇒ 写回在存档层不可分辨（噪声底主导）。
$ErrorActionPreference = "Continue"
$drv = "E:\PYTHON\CoreSwap\.tmp\vivo-stutter-260911-02\ab-threads\run_ab.ps1"
$dll = "E:\PYTHON\CoreSwap\.tmp\a1-260911-05\worldgen-a1-new.dll"
$out = "E:\PYTHON\CoreSwap\.tmp\c-260911-05"
$world = "E:\PYTHON\CoreSwap\runtime\1.20.1\java\run\world"
$want = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
Write-Output "===== C Tier3 control (old x2)  dll=$($want.Substring(0,16))  start $(Get-Date -Format 'HH:mm:ss') ====="

foreach ($tag in @("c-old2", "c-old3")) {
    $env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=0 -Dcoreswap.wbcontent=1"
    Write-Output "===== ARM $tag (bulkwb=0, run 2/3) ====="
    & pwsh -NoProfile -File $drv -Dll $dll -Tag $tag -Form async -Radius 160
    if (Test-Path $world) {
        $dst = "$out\world-$tag"
        Remove-Item $dst -Recurse -Force -EA SilentlyContinue
        Copy-Item $world $dst -Recurse -Force
        $n = (Get-ChildItem "$dst\region" -File -EA SilentlyContinue | Measure-Object).Count
        Write-Output "[$tag] archived world-$tag region files = $n"
    }
}

Copy-Item $dll "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Force
$now = (Get-FileHash "E:\PYTHON\CoreSwap\target\release\worldgen.dll" -Algorithm SHA256).Hash.ToLower()
Write-Output "restored target dll = $($now.Substring(0,16)) (want $($want.Substring(0,16))) $(if($now -eq $want){'OK'}else{'MISMATCH'})"
Write-Output "===== done $(Get-Date -Format 'HH:mm:ss') ====="
