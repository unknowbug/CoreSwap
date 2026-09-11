# run_recompute.ps1 — B1（260911-05）：用 E5 修正版工具复算 260910-05（1.21.6）全部对拍口径
# 依据：build-tooling #53 四步自检（自比 0 差 + 已知不同臂正对照非零 + 解析数 ≈ 生成器自报 + 规范核对）
# 全部离线：region 归档在 .investigations/ 下，无需重生成世界。
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$co   = "$root\.investigations\perf-closeout-260910-05\cmd-output"
$old  = "$root\.investigations\perf-regression-260910-04\cmd-output"
$dir  = "$root\.investigations\e5-recompute-260911-05"
$py   = "$dir\diff_arms_fixed.py"
$outd = "$dir\recompute"
New-Item -ItemType Directory -Force -Path $outd | Out-Null

$pairs = @(
  @{ tag="self_N_na-r1";           a="$co\region-na-r1"; b="$co\region-na-r1" },
  @{ tag="r3-r1_vs_r3-r2";         a="$old\region-r3";   b="$co\region-r3-r2" },
  @{ tag="r3sync-r1_vs_r3sync-r2"; a="$old\region-r3sync"; b="$co\region-r3sync-r2" },
  @{ tag="r3-r2_vs_r3sync-r1";     a="$co\region-r3-r2"; b="$old\region-r3sync" },
  @{ tag="r3-r2_vs_vanilla";       a="$co\region-r3-r2"; b="$old\region-vanilla-260910-04" },
  @{ tag="r3sync-r2_vs_vanilla";   a="$co\region-r3sync-r2"; b="$old\region-vanilla-260910-04" },
  @{ tag="N_sync_vs_vanilla";      a="$co\region-ns-r1"; b="$co\region-nv-r1" },
  @{ tag="N_async_vs_sync";        a="$co\region-na-r1"; b="$co\region-ns-r1" },
  @{ tag="N_async_vs_async2";      a="$co\region-na-r1"; b="$co\region-na-r2" },
  @{ tag="E_sync_vs_vanilla";      a="$co\region-es-r1"; b="$co\region-ev-r1" },
  @{ tag="E_async_vs_sync";        a="$co\region-ea-r1"; b="$co\region-es-r1" },
  @{ tag="E_async_vs_async2";      a="$co\region-ea-r1"; b="$co\region-ea-r2" }
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
