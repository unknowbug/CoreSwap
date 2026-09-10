# run_native_A.ps1 — A 组 native bench（串行、无 JNI、#28 串行铁律 + #30 rlib 纪律）
# 载体：worldgen-core/src/bin-diag/camin_bench.rs（直接调 fill_chunk_blocks，无 JNI/日志/线程 spawn）
# 用途：① 引擎固有成本与 CA_MIN/EST_L2 的**无争用**成本；② 1.20.1 vs 1.21.6 数据/引擎自回归；
#       ③ 行为等价 hash 哨兵（#47：改动后 on/off hash 必须逐位不变 = 6908dbfc9a79c40a / 115641b86711d9cd）
param([string]$OutRoot = "E:\PYTHON\CoreSwap\.tmp\perf-reg-260910-04")
$ErrorActionPreference = "Stop"
$root = "E:\PYTHON\CoreSwap"
$seed = "-8248318472910187742"   # 与 260909-04/06 camin-perf 同 seed/同参数（可直接比对）
$exe = "$OutRoot\camin_bench.exe"
$log = "$OutRoot\native-A.txt"

# 1) rlib 选择（#30：必须链 deps 下最新 hash rlib，根 rlib 是陈旧缓存）
$rlib = Get-ChildItem "$root\target\release\deps\libWorldgenRust-*.rlib" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$src = Get-Item "$root\worldgen-core\src\worldgen_handle.rs"
Write-Output "[A][rlib] $($rlib.Name) mtime=$($rlib.LastWriteTime) vs worldgen_handle.rs=$($src.LastWriteTime)"
if ($rlib.LastWriteTime -lt $src.LastWriteTime) { Write-Output "[A] FAIL rlib 陈旧于源码（#30）"; exit 1 }

# 2) 编译（rustc 单编，不进 cargo 默认构建）
Push-Location $root
rustc -O --edition 2021 "worldgen-core\src\bin-diag\camin_bench.rs" `
    --extern "WorldgenRust=$($rlib.FullName)" -L "$root\target\release\deps" -o $exe 2>&1 |
    Select-String -Pattern "^error" | Select-Object -First 10
Pop-Location
if (-not (Test-Path $exe)) { Write-Output "[A] FAIL compile"; exit 1 }
Write-Output "[A][exe] $exe mtime=$((Get-Item $exe).LastWriteTime)"

# 3) 四臂串行（每臂 256 chunks，12×12 之外保持与本课题历史同参）
$arms = @(
    @{ n = "1216_camin_on";  wg = "$root\versions\1.21.6\data\worldgen"; env = @{} },
    @{ n = "1216_camin_off"; wg = "$root\versions\1.21.6\data\worldgen"; env = @{ WG_CA_MIN = "0" } },
    @{ n = "1216_estl2_off"; wg = "$root\versions\1.21.6\data\worldgen"; env = @{ WG_EST_L2 = "0" } },
    @{ n = "1201_camin_on";  wg = "$root\versions\1.20.1\data\worldgen"; env = @{} },
    @{ n = "1201_camin_off"; wg = "$root\versions\1.20.1\data\worldgen"; env = @{ WG_CA_MIN = "0" } }
)
foreach ($a in $arms) {
    foreach ($k in $a.env.Keys) { Set-Item -Path "env:$k" -Value $a.env[$k] }
    Write-Output "[A][run] $($a.n) wg=$($a.wg) env=$($a.env | ConvertTo-Json -Compress)"
    & $exe $seed $a.wg 16 0 0 2>&1 | Tee-Object -FilePath $log -Append
    foreach ($k in $a.env.Keys) { Remove-Item -Path "env:$k" -ErrorAction SilentlyContinue }
    Start-Sleep 2
}
Write-Output "[A] done -> $log"
