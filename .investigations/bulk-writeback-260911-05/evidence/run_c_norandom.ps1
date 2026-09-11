# C 线 Tier 3 判决实验（260911-05）：关闭 live-server 随机刻源后重跑，判定「切片 A 超量」是否时序中介
# 与 run_dim.ps1 的差别：chunky start 之前先关随机刻/天气/刷怪/火焰蔓延（消除生成期间对已加载 chunk 的随机改写），
#   其余（dll 硬门禁 / 清 world / 清 chunky tasks / 还原 target dll）保持同款。
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [string]$JvmExtra = ""
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$out = "$root\.tmp\c-260911-05"
$rcon = "$root\.tmp\vivo-stutter-260911-02\ab-threads\rcon_one.py"
$proj = "$root\versions\1.20.1\java"; $rd = "$root\runtime\1.20.1\java\run"
$targetDll = "$root\target\release\worldgen.dll"
$bak = "$out\worldgen-target.bak"
$dll = "$root\.tmp\a1-260911-05\worldgen-a1-new.dll"
if (-not (Test-Path $bak)) { Copy-Item $targetDll $bak -Force }
$wantSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
$log = "$out\logs\$Tag.log"; $errf = "$log.err"
New-Item -ItemType Directory -Force -Path "$out\logs" | Out-Null
$seed = "417950215108767439"

Write-Output "########## $Tag start $(Get-Date -Format 'HH:mm:ss') dll=$($wantSha.Substring(0,16)) ##########"
$env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
$jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp"
if ($JvmExtra) { $jto = "$jto $JvmExtra" }
$env:JAVA_TOOL_OPTIONS = $jto
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Push-Location $proj; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
Start-Sleep 2
if (Test-Path "$rd\world") { Remove-Item "$rd\world" -Recurse -Force }
if (Test-Path "$rd\config\chunky\tasks") { Remove-Item "$rd\config\chunky\tasks" -Recurse -Force }
(Get-Content "$rd\server.properties") -replace "^level-seed=.*", "level-seed=$seed" | Set-Content "$rd\server.properties"
Remove-Item "$root\.tmp\java-tmp\coreswap-native" -Recurse -Force -EA SilentlyContinue
Copy-Item $dll $targetDll -Force

$extra = @("-PcppReplace=true", "-PcppWorldgenDir=$root\versions\1.20.1\data\worldgen")
$p = Start-Process -FilePath "gradle" -ArgumentList (@(":runServer") + $extra + @("--console=plain")) -WorkingDirectory $proj -RedirectStandardOutput $log -RedirectStandardError $errf -PassThru -NoNewWindow
$t0 = Get-Date; $deadline = (Get-Date).AddMinutes(15); $done = $false
while ((Get-Date) -lt $deadline) {
    Start-Sleep 3
    if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -EA SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
    if ($p.HasExited) { break }
}
if (-not $done) { Write-Output "[$Tag] FAIL no boot"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; Copy-Item $bak $targetDll -Force; return }
Start-Sleep 6
$dllLine = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
$gotSha = if ($dllLine -match "sha256=([0-9a-f]+)") { $matches[1] } else { "-" }
if ($gotSha -eq "-" -or $wantSha -notlike "$gotSha*") {
    Write-Output "[$Tag] VOID: executed=$gotSha != want=$($wantSha.Substring(0,16))"
    & python $rcon "stop" | Out-Null; $p.WaitForExit(180000) | Out-Null
    Get-Process java -EA SilentlyContinue | Stop-Process -Force; Copy-Item $bak $targetDll -Force; return
}
Write-Output "[$Tag][dll] OK executed=$($gotSha.Substring(0,16))"

# ★ 本实验的核心：先把 live-server 对已加载 chunk 的随机改写源全部关掉，再开始生成
foreach ($c in @("gamerule randomTickSpeed 0", "gamerule doWeatherCycle false", "gamerule doMobSpawning false",
                 "gamerule doFireTick false", "gamerule doTraderSpawning false", "weather clear",
                 "chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11",
                 "chunky radius 160", "chunky start")) {
    & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$Tag][rcon] $_" }
}
$deadline2 = (Get-Date).AddMinutes(45); $finishLine = $null
while ((Get-Date) -lt $deadline2) {
    Start-Sleep 5
    if ($p.HasExited) { break }
    $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
    if ($hit) { $finishLine = $hit.Line; break }
}
Start-Sleep 4
& python $rcon "chunky pause" | Out-Null
& python $rcon "stop" | Out-Null
$p.WaitForExit(240000) | Out-Null
Get-Process java -EA SilentlyContinue | Stop-Process -Force
Start-Sleep 3
Copy-Item $bak $targetDll -Force
$chunks = if ($finishLine -match "Processed: (\d+) chunks") { [int]$matches[1] } else { -1 }
$wb = (Select-String -Path $log -Pattern "WG-CONTENT-WB" -SimpleMatch -EA SilentlyContinue).Count
Write-Output "[$Tag][result] chunks=$chunks wbLines=$wb"
# 归档世界
$dst = "$out\world-nr-$Tag"
if (Test-Path "$rd\world") {
    Remove-Item $dst -Recurse -Force -EA SilentlyContinue
    Copy-Item "$rd\world" $dst -Recurse -Force
    Write-Output "[$Tag] archived $dst region files = $((Get-ChildItem "$dst\region" -File -EA SilentlyContinue | Measure-Object).Count)"
}
Write-Output "########## $Tag DONE chunks=$chunks ##########"
