# 补跑 260910-04 既有三对（E5 复算补齐，使 260910-05 §2 表 8 行全为规范读法）
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$co   = "$root\.investigations\perf-closeout-260910-05\cmd-output"
$old  = "$root\.investigations\perf-regression-260910-04\cmd-output"
$dir  = "$root\.investigations\e5-recompute-260911-05"
$py   = "$dir\diff_arms_fixed.py"
$outd = "$dir\recompute"
$pairs = @(
  @{ tag="r3-r1_vs_r3sync-r1";   a="$old\region-r3";     b="$old\region-r3sync" },
  @{ tag="r3-r1_vs_vanilla";     a="$old\region-r3";     b="$old\region-vanilla-260910-04" },
  @{ tag="r3sync-r1_vs_vanilla"; a="$old\region-r3sync"; b="$old\region-vanilla-260910-04" }
)
foreach ($p in $pairs) {
  $f = "$outd\diff-$($p.tag).txt"
  Write-Output "===== $($p.tag) @ $(Get-Date -Format 'HH:mm:ss') ====="
  if (-not (Test-Path $p.a)) { Write-Output "MISSING $($p.a)"; continue }
  if (-not (Test-Path $p.b)) { Write-Output "MISSING $($p.b)"; continue }
  python $py $p.a $p.b *> $f
  Get-Content $f | Select-String -Pattern "^vanilla chunks|^coreswap chunks|^common=|^blocks=" | ForEach-Object { $_.Line }
}
Write-Output "DONE @ $(Get-Date -Format 'HH:mm:ss')"
