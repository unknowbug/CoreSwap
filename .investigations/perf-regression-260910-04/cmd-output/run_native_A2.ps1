# run_native_A2.ps1 — A2 组：in-game 语义对齐的 native bench（stageMask=3 等价：SKIP_FEATURES + SKIP_CARVER）
# 目的：bench 默认 flags=0 含 carver+features，而 in-game `stageMask=3` 让 Rust 只做 NOISE+SURFACE。
#       本组用 WG_SKIP_FEATURES=1 + WG_SKIP_CARVER=1 对齐 in-game，得到「in-game 真实 Rust 段」单 chunk 成本，
#       并做 1.21.6 vs 1.20.1 同语义对照。
param([string]$OutRoot = "E:\PYTHON\CoreSwap\.tmp\perf-reg-260910-04")
$ErrorActionPreference = "Stop"
$root = "E:\PYTHON\CoreSwap"
$seed = "-8248318472910187742"
$exe = "$OutRoot\camin_bench.exe"
$log = "$OutRoot\native-A2.txt"

$env:WG_SKIP_FEATURES = "1"
$env:WG_SKIP_CARVER = "1"
$arms = @(
    @{ n = "1216_terrain_only_camin_on";  wg = "$root\versions\1.21.6\data\worldgen"; env = @{} },
    @{ n = "1216_terrain_only_camin_off"; wg = "$root\versions\1.21.6\data\worldgen"; env = @{ WG_CA_MIN = "0" } },
    @{ n = "1201_terrain_only_camin_on";  wg = "$root\versions\1.20.1\data\worldgen"; env = @{} },
    @{ n = "1201_terrain_only_camin_off"; wg = "$root\versions\1.20.1\data\worldgen"; env = @{ WG_CA_MIN = "0" } }
)
foreach ($a in $arms) {
    foreach ($k in $a.env.Keys) { Set-Item -Path "env:$k" -Value $a.env[$k] }
    Write-Output "[A2][run] $($a.n) wg=$($a.wg) env=$($a.env | ConvertTo-Json -Compress) skip(feat+carver)=on"
    & $exe $seed $a.wg 16 0 0 2>&1 | Select-String -Pattern "^(handle|RESULT|\[WG-CONF\])" | Tee-Object -FilePath $log -Append
    foreach ($k in $a.env.Keys) { Remove-Item -Path "env:$k" -ErrorAction SilentlyContinue }
    Start-Sleep 2
}
Remove-Item env:WG_SKIP_FEATURES -ErrorAction SilentlyContinue
Remove-Item env:WG_SKIP_CARVER -ErrorAction SilentlyContinue
Write-Output "[A2] done -> $log"
