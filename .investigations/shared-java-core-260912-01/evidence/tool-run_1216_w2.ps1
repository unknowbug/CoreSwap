# run_1216_w2.ps1 — 260912-01 Wave 2 V2/V3 arm (1.21.6)
# 复用 hang1216_260910.ps1 的 1.21.6 配方（runServer -PcppReplace -PcppLib -PcppWorldgenDir）
# 目的：V2 run 回归（boot/mixin apply/bridge init/生成完成）+ V3 [WG-CONTENT] 指纹 sanity（-Dcoreswap.mixlog=1）
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [int]$Radius = 160,
    [int]$TimeoutMin = 40
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$proj = "$root\versions\1.21.6\java"
$out = "$root\.tmp\shared-java-core-260912-01\w2"
New-Item -ItemType Directory -Force -Path $out, "$root\.tmp\java-tmp" | Out-Null
$seed = "417950215108767439"
$dll = "$root\target\release\worldgen1216.dll"
$bak = "$out\worldgen1216-bak.dll"
if (-not (Test-Path $bak)) { Copy-Item $dll $bak -Force }
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
$log = "$out\1216-$Tag.log"; $err = "$log.err"

$rcon = "$root\.tmp\feature-parity-260905-06\rcon_one.py"
if (-not (Test-Path $rcon)) { $rcon = "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py" }
Write-Output "########## 1216 $Tag start $(Get-Date -Format 'HH:mm:ss') radius=$Radius dll=$($dllSha.Substring(0,16)) rcon=$rcon ##########"
Write-Output "[$Tag] dll1216 sha=$dllSha"

$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp -Dcoreswap.mixlog=1"
if ($env:CORESWAP_EXTRA_JVM) { $jto = "$jto $($env:CORESWAP_EXTRA_JVM)" }
$env:JAVA_TOOL_OPTIONS = $jto

Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $proj; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2
$rd = "$root\runtime\1.21.6\java\run"
if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue

$p = Start-Process -FilePath "gradle" -ArgumentList @("runServer", "-PcppReplace=true", "-PcppLib=$dll",
    "-PcppWorldgenDir=$root\versions\1.21.6\data\worldgen", "--console=plain") `
    -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $err -PassThru -NoNewWindow

$t0 = Get-Date; $deadline = (Get-Date).AddMinutes(20); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) {
    Write-Output "[$Tag] FAIL no Done"
    Get-Content $log -Tail 25
    Get-Process java -EA SilentlyContinue | Stop-Process -Force
    return
}
$bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
$deadlineB = (Get-Date).AddSeconds(90); $initLine = $null
while ((Get-Date) -lt $deadlineB) {
    $initLine = (Select-String -Path $log -Pattern "\[CppBridge\] init " -EA SilentlyContinue | Select-Object -First 1).Line
    if ($initLine) { break }; Start-Sleep 2
}
Write-Output "[$Tag][bridge] $initLine"
if (-not ($initLine -match "seed=$seed")) { Write-Output "[$Tag] FAIL bridge seed check" }

Start-Sleep 3
foreach ($c in @("chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11", "chunky radius $Radius", "chunky start")) {
    & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$Tag][rcon] $_" }
}
$tStart = Get-Date; $deadline2 = (Get-Date).AddMinutes($TimeoutMin); $finishLine = $null
while ((Get-Date) -lt $deadline2) {
    Start-Sleep 10
    if ($p.HasExited) { break }
    $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
    if ($hit) { $finishLine = $hit.Line; break }
}
$wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
& python $rcon "chunky pause" | Out-Null
& python $rcon "stop" | Out-Null
$p.WaitForExit(240000) | Out-Null
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Start-Sleep 2
$finishTxt = if ($finishLine) { "FINISHED" } else { "TIMEOUT/no-finish" }
$exceptions = (Select-String -Path $log -Pattern "Exception|Error:|Cannot find target method|Mixin apply failed|NoClassDefFoundError" -EA SilentlyContinue | Measure-Object).Count
$contentLines = (Select-String -Path $log -Pattern "\[WG-CONTENT\] chunk\(" -AllMatches -EA SilentlyContinue | Measure-Object).Count
$bulkwbLines = (Select-String -Path $log -Pattern "\[WG-BULKWB\]" -EA SilentlyContinue | Measure-Object).Count
Write-Output "[$Tag][result] boot=${bootSec}s gen=${wallGen}s status=$finishTxt exception-lines=$exceptions content-lines=$contentLines bulkwb-lines=$bulkwbLines"
Write-Output "########## 1216 $Tag DONE $(Get-Date -Format 'HH:mm:ss') ##########"
