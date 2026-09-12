# run_pre_1201.ps1 — 260912-01 Wave 2 V3 的 **pre-Wave-2 对照臂**（1.20.1，git worktree @ ce5286b）
# 与 post 臂（run_ab.ps1 -Tag w2-post-1.20.1 -Radius 160, CORESWAP_EXTRA_JVM=-Dcoreswap.mixlog=1）**同配方**：
#   同 seed / 同 chunky center(-48,-11) / 同 radius 160 / 同 dll（build.gradle 用绝对路径读主仓 target dll）/ mixlog=1
# 判据：两臂 [WG-CONTENT] 指纹多重集 diff = 0（内容等价）；boot/bridge/生成完成 = V2 run 回归
param(
    [string]$Tag = "w2-pre-1.20.1",
    [int]$Radius = 160,
    [int]$TimeoutMin = 30
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$wt = "$root\.tmp\w2\pre-wt"
$out = "$root\.tmp\shared-java-core-260912-01\w2"
$proj = "$wt\versions\1.20.1\java"
$rd = "$wt\runtime\1.20.1\java\run"
$seed = "417950215108767439"
$dll = "$root\target\release\worldgen.dll"
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
$log = "$out\1201-$Tag.log"; $err = "$log.err"
$rcon = "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py"
Write-Output "########## pre-arm $Tag start $(Get-Date -Format 'HH:mm:ss') radius=$Radius dll=$($dllSha.Substring(0,16)) ##########"

$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$env:JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=$root\.tmp\java-tmp -Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1"

Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $proj; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2
if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue

$p = Start-Process -FilePath "gradle" -ArgumentList @("runServer", "-PcppReplace=true",
    "-PcppWorldgenDir=$root\versions\1.20.1\data\worldgen", "--console=plain") `
    -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $err -PassThru -NoNewWindow
$t0 = Get-Date; $deadline = (Get-Date).AddMinutes(20); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) { Write-Output "[$Tag] FAIL no Done"; Get-Content $log -Tail 25; Get-Process java -EA SilentlyContinue | Stop-Process -Force; return }
$bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
Start-Sleep 6
$dllLine = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
$gotSha = if ($dllLine -match "sha256=([0-9a-f]+)") { $matches[1] } else { "-" }
$gate = if ($gotSha -ne "-" -and $dllSha -like "$gotSha*") { "OK" } else { "MISMATCH" }
Write-Output "[$Tag][dll] $gate executed=$($gotSha.Substring(0,[Math]::Min(16,$gotSha.Length))) want=$($dllSha.Substring(0,16))"
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
$exceptions = (Select-String -Path $log -Pattern "Exception|Error:|Cannot find target method|NoClassDefFoundError" -EA SilentlyContinue | Measure-Object).Count
$contentLines = (Select-String -Path $log -Pattern "\[WG-CONTENT\] chunk\(" | Measure-Object).Count
Write-Output "[$Tag][result] boot=${bootSec}s gen=${wallGen}s finish=$(if($finishLine){'FINISHED'}else{'TIMEOUT'}) exception-lines=$exceptions content-lines=$contentLines"
Write-Output "########## pre-arm $Tag DONE $(Get-Date -Format 'HH:mm:ss') ##########"
