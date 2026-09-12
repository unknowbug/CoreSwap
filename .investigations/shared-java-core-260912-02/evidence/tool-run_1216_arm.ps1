# run_1216_arm.ps1 — 260912-02 1.21.6 单臂运行台（参数化；不预设结论）
# 基于 260912-01 的 w2/run_1216_w2.ps1，补三件事：
#   ① 逐臂 **in-log dll 自证**（E1 教训：读日志里的 [CppBridge] dll= sha256=，不信磁盘/环境推断）
#   ② 异常级联 **早退**（修掉 260912-01「等 Task finished 卡 40 分钟」的坑）
#   ③ 指纹落盘（fp-1.21.6-<Tag>.txt，供多重集/序列比对；判据只用多重集）
# 纪律：两版共用端口 ⇒ 臂必须串行；每臂 MUST 读自证行后才可跨臂比对。
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [string]$Props = "",                  # 额外 -D 开关，如 "-Dcoreswap.bulkwb=1"
    [int]$Radius = 160,
    [int]$WaitFinishSec = 420,            # 等 "Task finished" 的上限（秒）
    [string]$ExpectDll = "abd7d8893d22e030",  # 1.21.6 权威 dll 前 16 位（票/target/jar 三处一致）
    [int]$ThrowAbortAt = 3,               # DIAG write threw 达到该数即判「异常级联」提前结束等待
    [switch]$KeepWorld                   # 默认每臂删 world（保证同起点）
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$proj = "$root\versions\1.21.6\java"
$out  = "$root\.tmp\shared-java-core-260912-02"
New-Item -ItemType Directory -Force -Path $out, "$root\.tmp\java-tmp" | Out-Null
$seed = "417950215108767439"
$dll  = "$root\target\release\worldgen1216.dll"
$log  = "$out\1216-$Tag.log"; $err = "$log.err"

# ---- 前置硬门：磁盘 dll 血统（不通过即拒绝开跑）----
if (-not (Test-Path $dll)) { Write-Output "[$Tag] FAIL worldgen1216.dll missing"; return }
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
if ($dllSha.Substring(0,16) -ne $ExpectDll) {
    Write-Output "[$Tag] FAIL pre-flight dll mismatch: disk=$($dllSha.Substring(0,16)) expect=$ExpectDll"
    return
}
$rcon = "$root\.tmp\feature-parity-260905-06\rcon_one.py"
if (-not (Test-Path $rcon)) { $rcon = "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py" }
Write-Output "########## 1216 $Tag start $(Get-Date -Format 'HH:mm:ss') radius=$Radius props='$Props' diskDll=$($dllSha.Substring(0,16)) ##########"

$bak = "$out\worldgen1216-canonical.dll"
if (-not (Test-Path $bak)) { Copy-Item $dll $bak -Force }

$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp -Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1"
if ($Props) { $jto = "$jto $Props" }
$env:JAVA_TOOL_OPTIONS = $jto

Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $proj; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2

$rd = "$root\runtime\1.21.6\java\run"
if (-not $KeepWorld) {
    if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
    if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
}
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue

$p = Start-Process -FilePath "gradle" -ArgumentList @("runServer", "-PcppReplace=true", "-PcppLib=$dll",
    "-PcppWorldgenDir=$root\versions\1.21.6\data\worldgen", "--console=plain") `
    -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $err -PassThru -NoNewWindow

# ---- 等 Done（boot）----
$t0 = Get-Date; $deadline = (Get-Date).AddSeconds(300); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) {
    Write-Output "[$Tag] FAIL no Done"
    Get-Content $log -Tail 20
    Get-Process java -EA SilentlyContinue | Stop-Process -Force
    return
}
$bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)

# ---- in-log dll 自证 + bridge init ----
$deadlineB = (Get-Date).AddSeconds(120); $initLine = $null; $dllLine = $null
while ((Get-Date) -lt $deadlineB) {
    $initLine = (Select-String -Path $log -Pattern "\[CppBridge\] init " -EA SilentlyContinue | Select-Object -First 1).Line
    $dllLine  = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
    if ($initLine -and $dllLine) { break }; Start-Sleep 2
}
$logDll = if ($dllLine -and $dllLine -match "sha256=([0-9a-fA-F]{16,})") { $matches[1].ToLower().Substring(0,16) } else { "NONE" }
$dllOk  = ($logDll -eq $ExpectDll)
Write-Output "[$Tag][bridge] $initLine"
Write-Output "[$Tag][attest] logDll=$logDll expect=$ExpectDll match=$dllOk"
if (-not ($initLine -match "seed=$seed")) { Write-Output "[$Tag] FAIL bridge seed check" }

Start-Sleep 3
foreach ($c in @("chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11", "chunky radius $Radius", "chunky start")) {
    & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$Tag][rcon] $_" }
}

# ---- 等生成完成；异常级联则早退（本次缺陷臂的预期形态）----
$tStart = Get-Date; $deadline2 = (Get-Date).AddSeconds($WaitFinishSec); $finishLine = $null; $aborted = $false; $throws = 0
while ((Get-Date) -lt $deadline2) {
    Start-Sleep 10
    if ($p.HasExited) { break }
    $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
    if ($hit) { $finishLine = $hit.Line; break }
    $throws = (Select-String -Path $log -Pattern "DIAG write threw" -EA SilentlyContinue | Measure-Object).Count
    if ($throws -ge $ThrowAbortAt) { $aborted = $true; break }
}
$wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
& python $rcon "chunky pause" | Out-Null
& python $rcon "stop" | Out-Null
$p.WaitForExit(180000) | Out-Null
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Start-Sleep 2

# ---- 指标 + 指纹落盘 ----
$finishTxt = if ($finishLine) { "FINISHED" } elseif ($aborted) { "EARLY-ABORT(throw-cascade)" } else { "TIMEOUT/no-finish" }
function CountPat($pat) { return (Select-String -Path $log -Pattern $pat -EA SilentlyContinue | Measure-Object).Count }
$cContent = CountPat "\[WG-CONTENT\] chunk\("
$cWb      = CountPat "\[WG-CONTENT-WB\] chunk\("
$cEntry   = CountPat "EntryMissingException"
$cThrow   = CountPat "DIAG write threw"
$cExc     = CountPat "Exception|Error:|Cannot find target method|Mixin apply failed|NoClassDefFoundError"
$bulkSum  = (Select-String -Path $log -Pattern "\[WG-BULKWB\] calls=" -EA SilentlyContinue | Select-Object -Last 1).Line
$pbSum    = (Select-String -Path $log -Pattern "\[WG-PERBLOCK\] calls=" -EA SilentlyContinue | Select-Object -Last 1).Line
# 指纹：合并文件（口径与 260912-01 既有 fp-*.txt 逐字段一致）+ **分层文件**（§9.7 可比性：
#   [WG-CONTENT] = Rust **buf** 层、[WG-CONTENT-WB] = Java **读回**层——两层 hash 域不同，不得跨层比对）。
#   scoute D5：dll 自证不符（ARM-VOID）的臂**不写指纹**，避免污染台账。
$fp  = "$out\fp-1.21.6-$Tag.txt"
$fpc = "$out\fp-1.21.6-$Tag.content.txt"
$fpw = "$out\fp-1.21.6-$Tag.wb.txt"
if ($dllOk) {
    $fl = Select-String -Path $log -Pattern "\[WG-CONTENT\] chunk\(|\[WG-CONTENT-WB\] chunk\(" -EA SilentlyContinue | ForEach-Object { ($_.Line -replace '^.*\[(WG-CONTENT(-WB)?)\]', '[$1]').Trim() }
    $fl | Sort-Object | Set-Content $fp -Encoding utf8
    $fl | Where-Object { $_ -match '^\[WG-CONTENT\] chunk\(' } | Sort-Object | Set-Content $fpc -Encoding utf8
    $fl | Where-Object { $_ -match '^\[WG-CONTENT-WB\] chunk\(' } | Sort-Object | Set-Content $fpw -Encoding utf8
}
$fpSha  = if (Test-Path $fp)  { (Get-FileHash $fp  -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }
$fpcSha = if (Test-Path $fpc) { (Get-FileHash $fpc -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }
$fpwSha = if (Test-Path $fpw) { (Get-FileHash $fpw -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }

Write-Output "[$Tag][result] boot=${bootSec}s gen=${wallGen}s status=$finishTxt throws=$cThrow entryMissing=$cEntry exceptions=$cExc content=$cContent wb=$cWb fpSha=$fpSha contentSha=$fpcSha wbSha=$fpwSha"
Write-Output "[$Tag][bulk] $bulkSum"
Write-Output "[$Tag][perblock] $pbSum"
Write-Output "[$Tag][verdict] dllAttest=$dllOk $(if(-not $dllOk){'ARM-VOID'}else{'attested'})"
Write-Output "########## 1216 $Tag DONE $(Get-Date -Format 'HH:mm:ss') ##########"
