# run_arms_1201.ps1 — 260910-06 驱动：1.20.1 R3 异步化 同构建态 A/B
# 基线 = 260910-05 run_arms3.ps1（1.21.6 版，已验证姿势）；本版差异：
#   ① 路径/参数换 1.20.1（-PcppWorldgenDir=versions\1.20.1\data\worldgen）
#   ② **不传 -PcppLib**：1.20.1 CppBridge 不读 cpp.worldgen.lib（build.gradle 有映射行但消费端 0 命中 ⇒ 死参数）
#   ③ dll 走 Fabric resources/native（processResources 同步 target\release\worldgen.dll）
#   ④ 证据列改 1.20.1 口径：[CHUNKTIME] 1.20.1 版只有 mixin/gap/inflight（见 ChunkTiming.java 注释）
# 沙箱纪律：禁 WMI/CIM（用 .NET Process 属性）；bench 严格串行；不用 JVM attach
param([string[]]$Arms = @("oa"), [int]$RunIndex = 1, [switch]$CollectRegion)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"; $out = "$root\.tmp\perf-reg-260910-06"; $inv = "$root\.investigations\perf-reg-260910-06"
$seed = "417950215108767439"; $rcon = "$out\rcon_one.py"; $results = "$out\results.txt"
New-Item -ItemType Directory -Force -Path "$out\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$inv\cmd-output" | Out-Null

function Get-ArmSpec([string]$Arm) {
    $w1201 = "$root\versions\1.20.1\data\worldgen"
    $base = @("-PcppReplace=true", "-PcppWorldgenDir=$w1201", "-Pchunktime=1")
    $ow = "minecraft:overworld"; $nn = "minecraft:the_nether"; $en = "minecraft:the_end"
    switch ($Arm) {
        "oa"  { return @{ mode="coreswap"; x=$base; world=$ow; reg="region"; watch=$false; note="overworld async (default)" } }
        "os"  { return @{ mode="coreswap"; x=($base + @("-Psyncfill=1","-Pstallwatch=30")); world=$ow; reg="region"; watch=$true; note="overworld SYNC (control + stallwatch)" } }
        "oaW" { return @{ mode="coreswap"; x=($base + "-Pstallwatch=30"); world=$ow; reg="region"; watch=$true; note="overworld async + stallwatch (diagnose hang)" } }
        "oaS" { return @{ mode="coreswap"; x=($base + "-Pmixlog=1"); world=$ow; reg="region"; watch=$false; note="overworld async SANITY (mixlog on)" } }
        "osS" { return @{ mode="coreswap"; x=($base + @("-Psyncfill=1","-Pmixlog=1")); world=$ow; reg="region"; watch=$false; note="overworld SYNC + mixlog (content-hash arm)" } }
        "ovan"{ return @{ mode="vanilla";  x=@("-PcppVanilla=true"); world=$ow; reg="region"; note="overworld VANILLA (diff-tool positive control)" } }
        "onS" { return @{ mode="coreswap"; x=($base + @("-Pmixlog=1","-Pstallwatch=30")); world=$nn; reg="DIM-1\region"; watch=$true; note="nether async SANITY" } }
        "oeS" { return @{ mode="coreswap"; x=($base + @("-Pmixlog=1","-Pstallwatch=30")); world=$en; reg="DIM1\region"; watch=$true; note="end async SANITY" } }
        default { throw "unknown arm $Arm" }
    }
}

function Invoke-Arm([string]$Arm, [int]$RunIndex) {
    $sp = Get-ArmSpec $Arm; $mode = $sp.mode; $extra = $sp.x
    $run = "$root\runtime\1.20.1\java"; $tag = "$Arm-r$RunIndex"
    $log = "$out\logs\$tag.log"; $errf = "$log.err"
    Write-Output "`n########## $tag start $(Get-Date -Format 'HH:mm:ss') mode=$mode world=$($sp.world) [$($sp.note)] ##########"
    $env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
    $env:JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=$root\.tmp\java-tmp"
    New-Item -ItemType Directory -Force -Path "$root\.tmp\java-tmp" | Out-Null
    Get-Process java -ErrorAction SilentlyContinue | Stop-Process -Force
    Push-Location $run; gradle --stop --console=plain 2>$null | Out-Null; Pop-Location
    Start-Sleep 2
    if (Test-Path "$run\run\world") { Remove-Item "$run\run\world" -Recurse -Force }
    # ⚠️ Chunky 1.3.146 把任务状态持久化在 config\chunky\tasks\<ns>\<dim>.properties（离场不自动清）：
    #    残留任务会让下一次 `chunky start` 回「A task was already started…」而**根本不开始**（实测 20:45 那臂）。
    if (Test-Path "$run\run\config\chunky\tasks") { Remove-Item "$run\run\config\chunky\tasks" -Recurse -Force }
    $spf = "$run\run\server.properties"
    if (-not (Test-Path "$spf.bak-perf-260910-06")) { Copy-Item $spf "$spf.bak-perf-260910-06" -Force }
    (Get-Content $spf) -replace "^level-seed=.*", "level-seed=$seed" | Set-Content $spf

    # ⚠️ 必须用 `:runServer`（根项目限定名）：裸 `runServer` 在 1.20.1 多项目构建里会**级联**触发
    #    `:content-test:runServer`（第二个服务器，同一 run 目录、同一 world），实测导致本臂
    #    Chunky 任务 Processed 恒 0（见 record）。1.21.6 侧只跑 `:runServer`。
    $p = Start-Process -FilePath "gradle" -ArgumentList (@(":runServer") + $extra + @("--console=plain")) -WorkingDirectory $run -RedirectStandardOutput $log -RedirectStandardError $errf -PassThru -NoNewWindow
    $t0 = Get-Date; $deadline = (Get-Date).AddMinutes(15); $done = $false
    while ((Get-Date) -lt $deadline) {
        Start-Sleep 3
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern "Done \(" -ErrorAction SilentlyContinue | Select-Object -First 1)) { $done = $true; break }
        if ($p.HasExited) { break }
    }
    if (-not $done) { Write-Output "[$tag] FAIL no Done"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; "$tag`tFAIL_no_done" | Add-Content $results; return }
    $bootSec = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
    Start-Sleep 6
    $bridgeInit = (Select-String -Path $log -Pattern "\[CppBridge\] init " -EA SilentlyContinue | Select-Object -First 1).Line
    $dllLine = (Select-String -Path $log -Pattern "\[CppBridge\] dll=" -EA SilentlyContinue | Select-Object -First 1).Line
    if ($mode -eq "coreswap" -and -not ($bridgeInit -match "seed=$seed")) {
        Write-Output "[$tag] FAIL bridge [$bridgeInit]"; Get-Process java -EA SilentlyContinue | Stop-Process -Force; "$tag`tFAIL_bridge" | Add-Content $results; return
    }
    Write-Output "[$tag][bridge] $bridgeInit"
    Write-Output "[$tag][dll] $dllLine"

    foreach ($c in @("chunky quiet 10", "chunky world $($sp.world)", "chunky center -48 -11", "chunky start")) {
        & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$tag][rcon] $_" }
    }
    $srv = Get-Process java -EA SilentlyContinue | Sort-Object WorkingSet64 -Descending | Select-Object -First 1
    $cpu0 = if ($srv) { $srv.CPU } else { 0 }
    $tStart = Get-Date
    $deadline2 = (Get-Date).AddMinutes(40); $finishLine = $null
    # 早停保护（260910-06 新增）：环境/形态停滞时不再空等 40min——每 20s 查 chunky progress，
    # Processed 连续 ~180s 恒 0 且服务器近空转 ⇒ 判 stall 落盘诊断后退出。
    $stallSince = $null; $lastProgress = $null
    while ((Get-Date) -lt $deadline2) {
        Start-Sleep 10
        if ($p.HasExited) { break }
        $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
        if ($hit) { $finishLine = $hit.Line; break }
        if ((((Get-Date) - $tStart).TotalSeconds % 20) -lt 11) {
            $prog = (& python $rcon "chunky progress" 2>&1 | Out-String).Trim()
            if ($prog -match "Processed: (\d+) chunks") {
                $pn = [int]$matches[1]
                $lastProgress = $prog
                if ($pn -eq 0) { if (-not $stallSince) { $stallSince = Get-Date } }
                else { $stallSince = $null }
                if ($stallSince -and ((Get-Date) - $stallSince).TotalSeconds -gt 180) {
                    Write-Output "[$tag][STALL] Processed 恒 0 超 180s -> 早停。progress=[$prog]"
                    "STALL`t$tag`t$prog" | Add-Content $results
                    break
                }
            }
        }
    }
    $wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
    $srvEnd = Get-Process -Id $srv.Id -EA SilentlyContinue
    $cpuSec = if ($srvEnd) { [math]::Round($srvEnd.CPU - $cpu0, 0) } else { -1 }
    Start-Sleep 4
    & python $rcon "chunky pause" | Out-Null
    & python $rcon "stop" | Out-Null
    $p.WaitForExit(180000) | Out-Null
    Get-Process java -EA SilentlyContinue | Stop-Process -Force
    Start-Sleep 3

    # --- 证据采集（全部在停服后）---
    $totalTime = if ($finishLine -match "Total time: ([\d:]+)") { $matches[1] } else { "?" }
    $totalSec = -1
    if ($totalTime -match "^(\d+):(\d+):(\d+)$") { $totalSec = [int]$matches[1] * 3600 + [int]$matches[2] * 60 + [int]$matches[3] }
    $chunks = if ($finishLine -match "Processed: (\d+) chunks") { $matches[1] } else { "?" }
    $sha = if ($dllLine -match "sha256=([0-9a-f]+)") { $matches[1] } else { "-" }
    $bridgeShort = if ($bridgeInit -match "\[CppBridge\] init (.*)$") { $matches[1] } else { "-" }
    # [CHUNKTIME] 最后一行（1.20.1 版字段：n / mixin / gap / sumMixin / sumGap / inflight max,now）
    $ctLine = (Select-String -Path $log -Pattern "\[CHUNKTIME\]" -EA SilentlyContinue | Select-Object -Last 1).Line
    $ctN = "-"; $ctMixin = "-"; $ctGap = "-"; $ctSumMixin = "-"; $ctSumGap = "-"; $inflightMax = "-"; $inflightNow = "-"
    if ($ctLine) {
        if ($ctLine -match "n=(\d+)") { $ctN = $matches[1] }
        if ($ctLine -match "mixin=([\d.]+)") { $ctMixin = $matches[1] }
        if ($ctLine -match "gap=([\d.]+)") { $ctGap = $matches[1] }
        if ($ctLine -match "sumMixin=([\d.]+)") { $ctSumMixin = $matches[1] }
        if ($ctLine -match "sumGap=([\d.]+)") { $ctSumGap = $matches[1] }
        if ($ctLine -match "inflight max=(\d+)") { $inflightMax = $matches[1] }
        if ($ctLine -match "now=(\d+)") { $inflightNow = $matches[1] }
    }
    $mixinLines = (Select-String -Path $log -Pattern "\[Mixin\] populateNoise intercepted|\[Mixin\] buildSurface skipped" -EA SilentlyContinue | Measure-Object).Count
    $interceptDim = (Select-String -Path $log -Pattern "populateNoise\(nether\) intercepted|populateNoise\(end\) intercepted" -EA SilentlyContinue | Measure-Object).Count
    $fillLines = (Select-String -Path $log -Pattern "\[WG-FILL\]" -EA SilentlyContinue | Measure-Object).Count
    $cores = if ($wallGen -gt 0 -and $cpuSec -gt 0) { [math]::Round($cpuSec / $wallGen, 2) } else { -1 }
    Write-Output "[$tag][chunky] $finishLine"
    Write-Output "[$tag][chunktime] $ctLine"
    Write-Output "[$tag][evidence] mixinLines=$mixinLines interceptDim=$interceptDim fillLines=$fillLines genCpuSec=$cpuSec avgCores=$cores inflightMax=$inflightMax"

    if ($CollectRegion) {
        $src = "$run\run\world\$($sp.reg)"
        $dst = "$inv\cmd-output\region-$tag"
        if (Test-Path $src) {
            New-Item -ItemType Directory -Force -Path $dst | Out-Null
            Copy-Item "$src\*.mca" $dst -Force
            $n = (Get-ChildItem $dst -Filter *.mca | Measure-Object).Count
            Write-Output "[$tag][region] copied $n mca -> $dst"
        } else { Write-Output "[$tag][region] MISSING $src" }
    }

    "$tag`tmode=$mode`tworld=$($sp.world)`ttotal=$totalTime`tsec=$totalSec`tchunks=$chunks`twallgen=$wallGen`tboot=$bootSec`tctN=$ctN`tmixin=$ctMixin`tgap=$ctGap`tsumMixin=$ctSumMixin`tsumGap=$ctSumGap`tinflightMax=$inflightMax`tinflightNow=$inflightNow`tmixinLines=$mixinLines`tfillLines=$fillLines`tinterceptDim=$interceptDim`tserverCpu=$cpuSec`tcores=$cores`tdllsha=$sha`tbridge=$bridgeShort`targs=$($extra -join ' ')" | Add-Content $results
    Write-Output "########## $tag DONE total=$totalTime (${totalSec}s) chunks=$chunks inflight=$inflightMax ##########"
}

foreach ($a in $Arms) { Invoke-Arm $a $RunIndex }
Write-Output "`n===== results.txt ====="; Get-Content $results
