# diff_dims_260910-05.ps1 — 维度 A/B 逐块对拍（无 bench 运行，避免 CPU 争用）
# 工具：cmd-output/diff_arms.py（与 overworld 同修订副本）；其内嵌 15 个 overworld 定点段对维度无意义（输出里会显示 NOT in both arms，已声明）。
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$co   = "$root\.investigations\perf-closeout-260910-05\cmd-output"
$py   = "$co\diff_arms.py"

$pairs = @(
  @{ tag = "N_sync_vs_vanilla";  a = "$co\region-ns-r1"; b = "$co\region-nv-r1" },
  @{ tag = "N_async_vs_sync";    a = "$co\region-na-r1"; b = "$co\region-ns-r1" },
  @{ tag = "N_async_vs_async2";  a = "$co\region-na-r1"; b = "$co\region-na-r2" },
  @{ tag = "E_sync_vs_vanilla";  a = "$co\region-es-r1"; b = "$co\region-ev-r1" },
  @{ tag = "E_async_vs_sync";    a = "$co\region-ea-r1"; b = "$co\region-es-r1" },
  @{ tag = "E_async_vs_async2";  a = "$co\region-ea-r1"; b = "$co\region-ea-r2" }
)
foreach ($p in $pairs) {
  $f = "$co\diff-$($p.tag).txt"
  Write-Output "===== $($p.tag) @ $(Get-Date -Format 'HH:mm:ss') ====="
  if (-not (Test-Path $p.a)) { Write-Output "MISSING $($p.a)"; continue }
  if (-not (Test-Path $p.b)) { Write-Output "MISSING $($p.b)"; continue }
  python $py $p.a $p.b *> $f
  Get-Content $f | Select-String -Pattern "^vanilla chunks|^coreswap chunks|^common=|^blocks=" | ForEach-Object { $_.Line }
}
Write-Output "ALL DONE @ $(Get-Date -Format 'HH:mm:ss')"
