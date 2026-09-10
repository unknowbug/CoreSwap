# diff_pairs_260910-05.ps1 — 逐块对拍（严格在无 bench 运行时跑，避免 CPU 争用污染）
# 工具：diff_arms.py（与 260910-04 同修订副本，保证口径可比）——注意其内嵌 15 个 overworld 定点坐标段对维度臂无意义（声明为 ovw 专用）。
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$inv  = "$root\.investigations\perf-closeout-260910-05"
$old  = "$root\.investigations\perf-regression-260910-04\cmd-output"
$co   = "$inv\cmd-output"
$py   = "$co\diff_arms.py"
$outd = "$co"
New-Item -ItemType Directory -Force -Path $outd | Out-Null

$pairs = @(
  @{ tag = "r3-r1_vs_r3-r2";       a = "$old\region-r3";                      b = "$co\region-r3-r2" },
  @{ tag = "r3sync-r1_vs_r3sync-r2"; a = "$old\region-r3sync";                b = "$co\region-r3sync-r2" },
  @{ tag = "r3-r2_vs_r3sync-r1";   a = "$co\region-r3-r2";                    b = "$old\region-r3sync" },
  @{ tag = "r3-r2_vs_vanilla";     a = "$co\region-r3-r2";                    b = "$old\region-vanilla-260910-04" },
  @{ tag = "r3sync-r2_vs_vanilla"; a = "$co\region-r3sync-r2";                b = "$old\region-vanilla-260910-04" }
)
foreach ($p in $pairs) {
  $f = "$outd\diff-$($p.tag).txt"
  Write-Output "===== $($p.tag) @ $(Get-Date -Format 'HH:mm:ss') ====="
  if (-not (Test-Path $p.a)) { Write-Output "MISSING $($p.a)"; continue }
  if (-not (Test-Path $p.b)) { Write-Output "MISSING $($p.b)"; continue }
  python $py $p.a $p.b *> $f
  Get-Content $f | Select-String -Pattern "^vanilla chunks|^coreswap chunks|^common=|^blocks=" | ForEach-Object { $_.Line }
}
Write-Output "ALL DONE @ $(Get-Date -Format 'HH:mm:ss')"
