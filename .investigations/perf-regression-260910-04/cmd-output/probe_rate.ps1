# probe_rate.ps1 — 池缩放判别探针（260910-04）：只跑到「速率稳定」即停（不跑完 region），
# 比较同一臂在 pool=23（实测已知）与 pool=4 下的 cps，区分「worker 受限」vs「驱动/串行资源受限」。
# 用法：probe_rate.ps1 -Arm coreswap|vanilla -Pool 4 -Seconds 100
param([Parameter(Mandatory = $true)][string]$Arm, [int]$Pool = 4, [int]$Seconds = 100)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"; $out = "$root\.tmp\perf-reg-260910-04"
$seed = "417950215108767439"; $rcon = "$out\rcon_one.py"
$run = "$root\runtime\1.21.6\java"
$tag = "rate-$Arm-pool$Pool"
$log = "$out\logs\$tag.log"
$extra = @("-PmaxBgThreads=$Pool")
if ($Arm -eq "coreswap") {
    $extra += @("-PcppReplace=true", "-PcppLib=$root\target\release\worldgen1216.dll", "-PcppWorldgenDir=$root\versions\1.21.6\data\worldgen")
} else {
    $extra += @("-PcppVanilla=true")
}
$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=$root\.tmp\java-tmp"; $env:PYTHONIOENCODING = "utf-8"
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $run; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2
if (Test-Path "$run\run\world") { Remove-Item "$run\run\world" -Recurse -Force }
$spf = "$run\run\server.properties"
(Get-Content $spf) -replace "^level-seed=.*", "level-seed=$seed" -replace "^enable-rcon=.*", "enable-rcon=true" -replace "^rcon.password=.*", "rcon.password=coreswap" | Set-Content $spf
$p = Start-Process -FilePath "gradle" -ArgumentList (@("runServer") + $extra + @("--console=plain")) -WorkingDirectory $run -RedirectStandardOutput $log -RedirectStandardError "$log.err" -PassThru -NoNewWindow
$deadline = (Get-Date).AddMinutes(10); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) { Write-Output "[$tag] FAIL no Done"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; exit 1 }
Start-Sleep 5
$srv = Get-Process java -EA SilentlyContinue | Sort-Object WorkingSet64 -Descending | Select-Object -First 1
$cpu0 = if ($srv) { $srv.CPU } else { 0 }
foreach ($c in @("chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11", "chunky start")) { & python $rcon $c 2>&1 | Out-Null }
$t0 = Get-Date
Start-Sleep $Seconds
$rates = Select-String -Path $log -Pattern "Task running" -EA SilentlyContinue | Select-Object -Last 3
$rates | ForEach-Object { Write-Output "[$tag] $($_.Line)" }
$wall = ((Get-Date) - $t0).TotalSeconds
$srvEnd = Get-Process -Id $srv.Id -EA SilentlyContinue
$cpu = if ($srvEnd) { $srvEnd.CPU - $cpu0 } else { -1 }
Write-Output "[$tag] wall=$([math]::Round($wall,0))s cpu=$([math]::Round($cpu,0))s avgCores=$([math]::Round($cpu/$wall,2))"
& python $rcon "chunky pause" | Out-Null
& python $rcon "stop" | Out-Null
$p.WaitForExit(90000) | Out-Null
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Write-Output "[$tag] done"
