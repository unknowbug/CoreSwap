# run_arms2.ps1 — C 组判别矩阵（v2：无 WMI 调用；WG-CONF 事后读；含线程快照）
# 根因修正：Get-NetTCPConnection / Get-CimInstance 在本沙箱不可用会挂起 → 一律不用；
#           server pid 用「CPU 最高的 java 进程」判定。
# 口径备注：Chunky 1.4.40 无 `chunkradius` 子命令（本版忽略）→ 实际 region = 默认 radius 500 方块
#           ≈ 65×65 = 4225 chunks（与 260910-03 基线同口径，逐臂用日志 Total 行核对）。
param([string[]]$Arms = @("c2", "c3", "c0", "c4", "c5"), [int]$RunIndex = 1, [int]$Samples = 3)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"; $out = "$root\.tmp\perf-reg-260910-04"
$seed = "417950215108767439"; $rcon = "$out\rcon_one.py"; $results = "$out\results.txt"
$jcmd = "D:\Program Files\Java\jdk-24.0.1\bin\jcmd.exe"
New-Item -ItemType Directory -Force -Path "$out\logs" | Out-Null

function Get-ArmSpec([string]$Arm) {
    $d1216 = "$root\target\release\worldgen1216.dll"; $d1201 = "$root\target\release\worldgen.dll"
    $w1216 = "$root\versions\1.21.6\data\worldgen"; $w1201 = "$root\versions\1.20.1\data\worldgen"
    switch ($Arm) {
        "c0" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-Pmixlog=1") } }
        "c1" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216") } }
        "c2" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-PcoreswapThreads=1") } }
        "c3" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-PcaMin=0") } }
        "c4" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-PestL2=0") } }
        "c5" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-PcaMin=0", "-PestL2=0", "-PcoreswapThreads=1") } }
        "c6" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-PcaCap=65536") } }
        "ct1" { return @{ ver = "1.21.6"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-Pchunktime=1") } }
        "coreswap1201" { return @{ ver = "1.20.1"; mode = "coreswap"; x = @("-PcppReplace=true", "-PcppLib=$d1201", "-PcppWorldgenDir=$w1201") } }
        "vanilla1216" { return @{ ver = "1.21.6"; mode = "vanilla"; x = @("-PcppVanilla=true") } }
        "vt1" { return @{ ver = "1.21.6"; mode = "vanilla"; x = @("-PcppVanilla=true", "-Pchunktime=1") } }
        "vanilla1201" { return @{ ver = "1.20.1"; mode = "vanilla"; x = @("-PcppVanilla=1") } }
        default { throw "unknown arm $Arm" }
    }
}

function Invoke-Arm([string]$Arm, [int]$RunIndex) {
    $sp = Get-ArmSpec $Arm; $ver = $sp.ver; $mode = $sp.mode; $extra = $sp.x
    $run = "$root\runtime\$ver\java"; $tag = "$Arm-r$RunIndex"
    $log = "$out\logs\$tag.log"; $errf = "$log.err"
    Write-Output "`n########## $tag start $(Get-Date -Format 'HH:mm:ss') ver=$ver mode=$mode ##########"
    $env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=$root\.tmp\java-tmp"; $env:PYTHONIOENCODING = "utf-8"
    New-Item -ItemType Directory -Force -Path "$root\.tmp\java-tmp" | Out-Null
    Get-Process java -ErrorAction SilentlyContinue | Stop-Process -Force
    Push-Location $run; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
    Start-Sleep 2
    if (Test-Path "$run\run\world") { Remove-Item "$run\run\world" -Recurse -Force }
    $spf = "$run\run\server.properties"
    if (-not (Test-Path "$spf.bak-perf-260910-04")) { Copy-Item $spf "$spf.bak-perf-260910-04" -Force }
    (Get-Content $spf) -replace "^level-seed=.*", "level-seed=$seed" -replace "^enable-rcon=.*", "enable-rcon=true" -replace "^rcon.password=.*", "rcon.password=coreswap" | Set-Content $spf

    $p = Start-Process -FilePath "gradle" -ArgumentList (@("runServer") + $extra + @("--console=plain")) -WorkingDirectory $run -RedirectStandardOutput $log -RedirectStandardError $errf -PassThru -NoNewWindow
    $t0 = Get-Date; $deadline = (Get-Date).AddMinutes(12); $done = $false
    while ((Get-Date) -lt $deadline) {
        Start-Sleep 3
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -ErrorAction SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
        if ($p.HasExited) { break }
    }
    if (-not $done) { Write-Output "[$tag] FAIL no Done"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; "$tag`tFAIL_no_done" | Add-Content $results; return }
    $bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
    Start-Sleep 6   # 让 CppBridge 初始化行落盘
    $bridgeInit = (Select-String -Path $log -Pattern "\[CppBridge\] init " -EA SilentlyContinue | Select-Object -First 1).Line
    $dllLine = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
    if ($mode -eq "coreswap" -and -not ($bridgeInit -match "stageMask=3" -and $bridgeInit -match "seed=$seed")) {
        Write-Output "[$tag] FAIL bridge [$bridgeInit]"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; "$tag`tFAIL_bridge" | Add-Content $results; return
    }
    Write-Output "[$tag][bridge] $bridgeInit"
    Write-Output "[$tag][dll] $dllLine"

    foreach ($c in @("chunky quiet 10", "chunky world minecraft:overworld", "chunky center -48 -11", "chunky start")) {
        & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$tag][rcon] $_" }
    }
    # 服务器进程识别（无 WMI）：WorkingSet 最大者 = 服务器 JVM；记生成起止 CPU 以算平均并发度
    $srv = Get-Process java -EA SilentlyContinue | Sort-Object WorkingSet64 -Descending | Select-Object -First 1
    $cpu0 = if ($srv) { $srv.CPU } else { 0 }
    Write-Output "[$tag][srvpid] $($srv.Id) ws_mb=$([math]::Round($srv.WorkingSet64/1MB,0)) cpu0=$([math]::Round($cpu0,0))"
    $tStart = Get-Date
    $deadline2 = (Get-Date).AddMinutes(40); $finishLine = $null; $taken = 0; $lastSample = Get-Date
    while ((Get-Date) -lt $deadline2) {
        Start-Sleep 10
        if ($p.HasExited) { break }
        $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
        if ($hit) { $finishLine = $hit.Line; break }
        if ($Samples -gt 0 -and $taken -lt $Samples -and ((Get-Date) - $lastSample).TotalSeconds -gt 50) {
            $taken++
            $jp = Get-Process java -EA SilentlyContinue | Sort-Object CPU -Descending | Select-Object -First 1
            if ($jp) {
                $sf = "$out\tdump-$tag-$taken.txt"
                & $jcmd $jp.Id Thread.print 2>$null | Out-File -Encoding utf8 $sf
                $prog = (Select-String -Path $log -Pattern "Task running" -EA SilentlyContinue | Select-Object -Last 1).Line
                Write-Output "[$tag][sample$taken] cpu=$([math]::Round($jp.CPU,0))s size=$((Get-Item $sf -EA SilentlyContinue).Length) prog=$prog"
            }
            $lastSample = Get-Date
        }
    }
    $wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
    Write-Output "[$tag][chunky] $finishLine"
    Select-String -Path $log -Pattern "Task running" -EA SilentlyContinue | Select-Object -Last 2 | ForEach-Object { Write-Output "[$tag][rate] $($_.Line)" }
    $mixLines = (Select-String -Path $log -Pattern "\[Mixin\] populateNoise intercepted|\[Mixin\] buildSurface skipped" -EA SilentlyContinue | Measure-Object).Count
    $confLine = (Select-String -Path $errf -Pattern "\[WG-CONF\]" -EA SilentlyContinue | Select-Object -First 1).Line
    $srvEnd = Get-Process -Id $srv.Id -EA SilentlyContinue
    $cpuSec = if ($srvEnd) { [math]::Round($srvEnd.CPU - $cpu0, 0) } else { -1 }
    $cores = if ($wallGen -gt 0 -and $cpuSec -gt 0) { [math]::Round($cpuSec / $wallGen, 2) } else { -1 }
    Write-Output "[$tag][evidence] mixinLines=$mixLines genCpuSec=$cpuSec avgCores=$cores"
    Write-Output "[$tag][conf] $confLine"

    Start-Sleep 4
    & python $rcon "chunky pause" | Out-Null
    & python $rcon "stop" | Out-Null
    $p.WaitForExit(120000) | Out-Null
    Get-Process java -EA SilentlyContinue | Stop-Process -Force
    Start-Sleep 3

    $totalTime = if ($finishLine -match "Total time: ([\d:]+)") { $matches[1] } else { "?" }
    $totalSec = -1
    if ($totalTime -match "^(\d+):(\d+):(\d+)$") { $totalSec = [int]$matches[1] * 3600 + [int]$matches[2] * 60 + [int]$matches[3] }
    $chunks = if ($finishLine -match "Processed: (\d+) chunks") { $matches[1] } else { "?" }
    $sha = if ($dllLine -match "sha256=([0-9a-f]+)") { $matches[1] } else { "-" }
    $confShort = if ($confLine -match "\[WG-CONF\]\s*(.*)$") { $matches[1] } else { "-" }
    "$tag`t$mode`tver=$ver`ttotal=$totalTime`tsec=$totalSec`tchunks=$chunks`twallgen=$wallGen`tboot=$bootSec`tmixinLines=$mixLines`tserverCpu=$cpuSec`tdllsha=$sha`tconf=$confShort" | Add-Content $results
    Write-Output "########## $tag DONE total=$totalTime (${totalSec}s) chunks=$chunks serverCpu=$cpuSec ##########"
}

foreach ($a in $Arms) { Invoke-Arm $a $RunIndex }
Write-Output "`n===== results.txt ====="; Get-Content $results
