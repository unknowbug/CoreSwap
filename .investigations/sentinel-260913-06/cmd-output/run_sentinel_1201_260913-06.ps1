# 260913-06 副本变更登记：① out -> .tmp\sentinel-260913-06；② +RTag 参数入标签（同 seed 跨 region 防同路径覆盖，#144 判据1）
# run_sentinel_1201.ps1 — 260913-05 跨 seed 动态压测（承 260913-03 版，最小改造：+Seed 参数、新区输出）
# ⚠️ 绝不 Stop-Process java（用户游戏进程可能在跑）；用 `:runServer`（#51 裸任务名双命中未修）；
#   无 Chunky / 无指纹 / 无计时结论——只取门控行为化证据（非性能测试）。
param(
    [Parameter(Mandatory = $true)][ValidateSet("off", "on")][string]$Gate,
    [string]$Seed = "417950215108767439",
    [string]$FRegion = "forceload add -128 -128 127 127",
    [string]$RTag = "r1"
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$proj = "$root\versions\1.20.1\java"
$rd   = "$root\runtime\1.20.1\java\run"
$out  = "$root\.tmp\sentinel-260913-06"
New-Item -ItemType Directory -Force -Path $out, "$root\.tmp\java-tmp" | Out-Null
$tag  = "$Gate-s$Seed-$RTag"
$log  = "$out\sentinel1201-$tag.log"; $err = "$log.err"

Write-Output "########## sentinel1201 gate=$Gate seed=$Seed start $(Get-Date -Format 'HH:mm:ss') ##########"
$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp -Dcoreswap.wbcontent=1"
if ($Gate -eq "on") { $jto = "$jto -Dcoreswap.bulkwbsentinel=1" }
$env:JAVA_TOOL_OPTIONS = $jto

if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$Seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue

$p = Start-Process -FilePath "gradle" -ArgumentList @(":runServer", "-PcppReplace=true", "-Psyncfill=1",
    "-PcppWorldgenDir=$root\versions\1.20.1\data\worldgen", "--console=plain") `
    -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $err -PassThru -NoNewWindow

$t0 = Get-Date; $deadline = (Get-Date).AddMinutes(12); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) { Write-Output "[gate=$Gate seed=$Seed] FAIL no Done"; Get-Content $log -Tail 25; & python "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py" "stop" 2>&1 | Out-Null; return }
$bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
Start-Sleep 6
# #80：bridge init 晚于 Done ⇒ 必须 post-Done forceload 触发新区块走接管管线
& python "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py" $FRegion 2>&1 | ForEach-Object { Write-Output "[rcon] $_" }
$dl2 = (Get-Date).AddSeconds(240); $wbHit = 0
while ((Get-Date) -lt $dl2) {
    Start-Sleep 10
    $wbHit = (Select-String -Path $log -Pattern "\[WG-CONTENT-WB\] chunk\(" -EA SilentlyContinue | Measure-Object).Count
    if ($wbHit -ge 100 -or $p.HasExited) { break }
}
Start-Sleep 4

function CountPat($pat) { return (Select-String -Path $log -Pattern $pat -EA SilentlyContinue | Measure-Object).Count }
$cArmed = CountPat "\[WG-BULKWB-SENTINEL\] armed"
$cCrash = CountPat "Accessing chunk sections from multiple threads"
$cWb    = CountPat "\[WG-CONTENT-WB\] chunk\("
$cExc   = CountPat "Exception|Mixin apply failed|NoClassDefFoundError"
Write-Output "[gate=$Gate seed=$Seed][result] boot=${bootSec}s armedLines=$cArmed sentinelCrash=$cCrash wbLines=$cWb suspExc=$cExc"

& python "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py" "stop" 2>&1 | ForEach-Object { Write-Output "[rcon] $_" }
$p.WaitForExit(240000) | Out-Null
Start-Sleep 2
Write-Output "########## sentinel1201 gate=$Gate seed=$Seed DONE $(Get-Date -Format 'HH:mm:ss') ##########"


