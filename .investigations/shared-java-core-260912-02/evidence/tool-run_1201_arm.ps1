# run_1201_arm.ps1 — 260912-02 1.20.1（出货线）单臂运行台
# 口径对齐 260912-01 的 V2/V3 臂：同 seed / chunky center(-48,-11) / radius 160 / mixlog=1 + wbcontent=1 /
#   `-PcppReplace=true -Psyncfill=1`（= run_ab.ps1 默认 Form='sync' 的口径）。
# 与 run_ab.ps1 的关键差异：**本台绝不写 dll**（run_ab.ps1 末尾会用历史 bak-597e12ed 覆盖 target dll = E1 污染陷阱）；
#   本台只做「磁盘 dll 前置硬门 + 运行期 [CppBridge] dll= 自证」，任何不一致即判 ARM-VOID。
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [string]$Props = "",
    [int]$Radius = 160,
    [int]$WaitFinishSec = 900,
    [string]$ExpectDll = "dd3b645f2c79d2cb",
    [switch]$KeepWorld
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$proj = "$root\versions\1.20.1\java"
$rd   = "$root\runtime\1.20.1\java\run"
$out  = "$root\.tmp\shared-java-core-260912-02"
New-Item -ItemType Directory -Force -Path $out, "$root\.tmp\java-tmp" | Out-Null
$seed = "417950215108767439"
$dll  = "$root\target\release\worldgen.dll"
$log  = "$out\1201-$Tag.log"; $err = "$log.err"

if (-not (Test-Path $dll)) { Write-Output "[$Tag] FAIL worldgen.dll missing"; return }
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
if ($dllSha.Substring(0,16) -ne $ExpectDll) {
    Write-Output "[$Tag] FAIL pre-flight dll mismatch: disk=$($dllSha.Substring(0,16)) expect=$ExpectDll"
    return
}
$rcon = "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py"
Write-Output "########## 1201 $Tag start $(Get-Date -Format 'HH:mm:ss') radius=$Radius props='$Props' diskDll=$($dllSha.Substring(0,16)) ##########"

$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp -Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1"
if ($Props) { $jto = "$jto $Props" }
$env:JAVA_TOOL_OPTIONS = $jto

Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $proj; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2
if (-not $KeepWorld) {
    if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
    if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
}
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue

$p = Start-Process -FilePath "gradle" -ArgumentList @("runServer", "-PcppReplace=true", "-Psyncfill=1",
    "-PcppWorldgenDir=$root\versions\1.20.1\data\worldgen", "--console=plain") `
    -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $err -PassThru -NoNewWindow

$t0 = Get-Date; $deadline = (Get-Date).AddMinutes(15); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) {
    Write-Output "[$Tag] FAIL no Done"; Get-Content $log -Tail 20
    Get-Process java -EA SilentlyContinue | Stop-Process -Force; return
}
$bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
Start-Sleep 6
$dllLine = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
$logDll = if ($dllLine -and $dllLine -match "sha256=([0-9a-fA-F]{16,})") { $matches[1].ToLower().Substring(0,16) } else { "NONE" }
$dllOk = ($logDll -eq $ExpectDll)
Write-Output "[$Tag][dll] executed=$logDll want=$ExpectDll match=$dllOk"
if (-not $dllOk) {
    Write-Output "[$Tag] VOID: dll 自证不符（本臂作废，不当证据）"
    & python $rcon "stop" | Out-Null
    $p.WaitForExit(180000) | Out-Null
    Get-Process java -EA SilentlyContinue | Stop-Process -Force
    return
}

foreach ($c in @("chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11", "chunky radius $Radius", "chunky start")) {
    & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$Tag][rcon] $_" }
}
$tStart = Get-Date; $deadline2 = (Get-Date).AddSeconds($WaitFinishSec); $finishLine = $null
while ((Get-Date) -lt $deadline2) {
    Start-Sleep 10
    if ($p.HasExited) { break }
    $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
    if ($hit) { $finishLine = $hit.Line; break }
}
$wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
Start-Sleep 4
& python $rcon "chunky pause" | Out-Null
& python $rcon "stop" | Out-Null
$p.WaitForExit(240000) | Out-Null
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Start-Sleep 2

function CountPat($pat) { return (Select-String -Path $log -Pattern $pat -EA SilentlyContinue | Measure-Object).Count }
$finishTxt = if ($finishLine) { "FINISHED" } else { "TIMEOUT/no-finish" }
$cContent = CountPat "\[WG-CONTENT\] chunk\("
$cWb      = CountPat "\[WG-CONTENT-WB\] chunk\("
$cExc     = CountPat "Exception|Error:|Cannot find target method|Mixin apply failed|NoClassDefFoundError"
# 指纹：合并文件（口径与 260912-01 既有 fp-*.txt 逐字段一致；本台已在 dll 自证不符时提前 return ⇒ 不写指纹）
#   + 分层文件（§9.7 可比性：[WG-CONTENT] = Rust buf 层、[WG-CONTENT-WB] = Java 读回层，不得跨层比对）。
$fp  = "$out\fp-1.20.1-$Tag.txt"
$fpc = "$out\fp-1.20.1-$Tag.content.txt"
$fpw = "$out\fp-1.20.1-$Tag.wb.txt"
$fl = Select-String -Path $log -Pattern "\[WG-CONTENT\] chunk\(|\[WG-CONTENT-WB\] chunk\(" -EA SilentlyContinue | ForEach-Object { ($_.Line -replace '^.*\[(WG-CONTENT(-WB)?)\]', '[$1]').Trim() }
$fl | Sort-Object | Set-Content $fp -Encoding utf8
$fl | Where-Object { $_ -match '^\[WG-CONTENT\] chunk\(' } | Sort-Object | Set-Content $fpc -Encoding utf8
$fl | Where-Object { $_ -match '^\[WG-CONTENT-WB\] chunk\(' } | Sort-Object | Set-Content $fpw -Encoding utf8
$fpSha  = if (Test-Path $fp)  { (Get-FileHash $fp  -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }
$fpcSha = if (Test-Path $fpc) { (Get-FileHash $fpc -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }
$fpwSha = if (Test-Path $fpw) { (Get-FileHash $fpw -Algorithm SHA256).Hash.ToLower().Substring(0,16) } else { "NONE" }
Write-Output "[$Tag][result] boot=${bootSec}s gen=${wallGen}s status=$finishTxt exceptions=$cExc content=$cContent wb=$cWb fpSha=$fpSha contentSha=$fpcSha wbSha=$fpwSha dllAttest=$dllOk"
Write-Output "########## 1201 $Tag DONE $(Get-Date -Format 'HH:mm:ss') ##########"
