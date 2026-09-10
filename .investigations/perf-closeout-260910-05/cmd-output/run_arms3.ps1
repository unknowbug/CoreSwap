# run_arms3.ps1 — 260910-05 驱动（C7 收口 + nether/end 异步化行为门）
# 基线 = .tmp/perf-reg-260910-04/run_arms2.ps1（R3 版，已验证姿势）
# 新增：① 维度臂（chunky world / DIM region 路径）② WG_CA_LOG 门（负/正对照）③ -Dcoreswap.rust.stages 经 JAVA_TOOL_OPTIONS
# ④ results 行加 args/stages/calog/world/inflight 字段（消 "开关无一手记录" 缺口，C5-④）
# 沙箱纪律：禁 WMI/CIM；server pid 用 WorkingSet 最大者；JVM attach 不用（Samples=0）
param([string[]]$Arms = @("r3"), [int]$RunIndex = 2, [int]$Samples = 0, [switch]$CollectRegion)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"; $out = "$root\.tmp\perf-reg-260910-05"; $inv = "$root\.investigations\perf-closeout-260910-05"
$seed = "417950215108767439"; $rcon = "$out\rcon_one.py"; $results = "$out\results.txt"
New-Item -ItemType Directory -Force -Path "$out\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$inv\cmd-output" | Out-Null

function Get-ArmSpec([string]$Arm) {
    $d1216 = "$root\target\release\worldgen1216.dll"; $w1216 = "$root\versions\1.21.6\data\worldgen"
    $base = @("-PcppReplace=true", "-PcppLib=$d1216", "-PcppWorldgenDir=$w1216", "-Pchunktime=1")
    $ow = "minecraft:overworld"; $nn = "minecraft:the_nether"; $en = "minecraft:the_end"
    switch ($Arm) {
        # --- C7 收口（overworld）---
        "r3"     { return @{ ver="1.21.6"; mode="coreswap"; x=$base; world=$ow; reg="region"; calog=$false; stages=$null; note="async baseline pair" } }
        "r3log"  { return @{ ver="1.21.6"; mode="coreswap"; x=$base; world=$ow; reg="region"; calog=$true;  stages=$null; note="neg control: mask=3 -> expect 0 [CA]" } }
        "r3feat" { return @{ ver="1.21.6"; mode="coreswap"; x=$base; world=$ow; reg="region"; calog=$true;  stages="5";    note="pos control: mask=5 features ON -> expect [CA]" } }
        "r3sync" { return @{ ver="1.21.6"; mode="coreswap"; x=($base + "-Psyncfill=1"); world=$ow; reg="region"; calog=$false; stages=$null; note="sync baseline pair" } }
        # --- nether（P4）---
        "nv" { return @{ ver="1.21.6"; mode="vanilla";  x=@("-PcppVanilla=true"); world=$nn; reg="DIM-1\region"; calog=$false; stages=$null; note="nether vanilla" } }
        "ns" { return @{ ver="1.21.6"; mode="coreswap"; x=($base + "-Psyncfill=1"); world=$nn; reg="DIM-1\region"; calog=$false; stages=$null; note="nether sync" } }
        "na" { return @{ ver="1.21.6"; mode="coreswap"; x=$base; world=$nn; reg="DIM-1\region"; calog=$false; stages=$null; note="nether async" } }
        "naS"{ return @{ ver="1.21.6"; mode="coreswap"; x=($base + "-Pmixlog=1"); world=$nn; reg="DIM-1\region"; calog=$false; stages=$null; note="nether async SANITY (mixlog on: 接管+写回证据)" } }
        # --- end（P4）---
        "ev" { return @{ ver="1.21.6"; mode="vanilla";  x=@("-PcppVanilla=true"); world=$en; reg="DIM1\region"; calog=$false; stages=$null; note="end vanilla" } }
        "es" { return @{ ver="1.21.6"; mode="coreswap"; x=($base + "-Psyncfill=1"); world=$en; reg="DIM1\region"; calog=$false; stages=$null; note="end sync" } }
        "ea" { return @{ ver="1.21.6"; mode="coreswap"; x=$base; world=$en; reg="DIM1\region"; calog=$false; stages=$null; note="end async" } }
        "eaS"{ return @{ ver="1.21.6"; mode="coreswap"; x=($base + "-Pmixlog=1"); world=$en; reg="DIM1\region"; calog=$false; stages=$null; note="end async SANITY (mixlog on)" } }
        default { throw "unknown arm $Arm" }
    }
}

function Invoke-Arm([string]$Arm, [int]$RunIndex) {
    $sp = Get-ArmSpec $Arm; $ver = $sp.ver; $mode = $sp.mode; $extra = $sp.x
    $run = "$root\runtime\$ver\java"; $tag = "$Arm-r$RunIndex"
    $log = "$out\logs\$tag.log"; $errf = "$log.err"
    Write-Output "`n########## $tag start $(Get-Date -Format 'HH:mm:ss') ver=$ver mode=$mode world=$($sp.world) [$($sp.note)] ##########"
    # --- env（每臂显式设置/清除，防跨臂串味）---
    $env:GRADLE_USER_HOME = "$root\.gradle-home"; $env:PYTHONIOENCODING = "utf-8"
    $jto = "-Djava.io.tmpdir=$root\.tmp\java-tmp"
    if ($sp.stages) { $jto = "$jto -Dcoreswap.rust.stages=$($sp.stages)" }
    $env:JAVA_TOOL_OPTIONS = $jto
    if ($sp.calog) { $env:WG_CA_LOG = "1" } else { Remove-Item Env:WG_CA_LOG -ErrorAction SilentlyContinue }
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
    while ((Get-Date) -lt $deadline2) {
        Start-Sleep 10
        if ($p.HasExited) { break }
        $hit = Select-String -Path $log -Pattern "Task finished" -EA SilentlyContinue | Select-Object -Last 1
        if ($hit) { $finishLine = $hit.Line; break }
    }
    $wallGen = [math]::Round(((Get-Date) - $tStart).TotalSeconds, 1)
    # CPU 必须在 stop/杀进程之前采（E2【推断·未核】：疑为停服后重复采样覆盖 ⇒ serverCpu=-1）
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
    $confLine = (Select-String -Path $errf -Pattern "\[WG-CONF\]" -EA SilentlyContinue | Select-Object -First 1).Line
    $confShort = if ($confLine -match "\[WG-CONF\]\s*(.*)$") { $matches[1] } else { "-" }
    $inflight = (Select-String -Path $log -Pattern "inflight max=(\d+)" -EA SilentlyContinue | Select-Object -Last 1)
    $inflightMax = if ($inflight -and $inflight.Line -match "inflight max=(\d+)") { $matches[1] } else { "-" }
    $caLines = (Select-String -Path $errf -Pattern "\[CA\]" -EA SilentlyContinue | Measure-Object).Count
    $mixinLines = (Select-String -Path $log -Pattern "\[Mixin\] populateNoise intercepted|\[Mixin\] buildSurface skipped" -EA SilentlyContinue | Measure-Object).Count
    $interceptN = (Select-String -Path $log -Pattern "populateNoise\(nether\) intercepted|populateNoise\(end\) intercepted" -EA SilentlyContinue | Measure-Object).Count
    # 注：$cpuSec/$srvEnd 已在停服前采样（上方）；此处**不得**重复采样（否则覆盖成 -1；E2 根因【推断·未核】）
    $cores = if ($wallGen -gt 0 -and $cpuSec -gt 0) { [math]::Round($cpuSec / $wallGen, 2) } else { -1 }
    Write-Output "[$tag][chunky] $finishLine"
    Write-Output "[$tag][evidence] mixinLines=$mixinLines interceptDim=$interceptN genCpuSec=$cpuSec avgCores=$cores inflightMax=$inflightMax CALines=$caLines"
    Write-Output "[$tag][conf] $confShort"

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

    "$tag`t$mode`tver=$ver`tworld=$($sp.world)`ttotal=$totalTime`tsec=$totalSec`tchunks=$chunks`twallgen=$wallGen`tboot=$bootSec`tmixinLines=$mixinLines`tinterceptDim=$interceptN`tserverCpu=$cpuSec`tcores=$cores`tinflightMax=$inflightMax`tCALines=$caLines`tdllsha=$sha`tstages=$(if($sp.stages){$sp.stages}else{"default"})`tcalog=$($sp.calog)`targs=$($extra -join ' ')`tconf=$confShort" | Add-Content $results
    Write-Output "########## $tag DONE total=$totalTime (${totalSec}s) chunks=$chunks inflight=$inflightMax ##########"
}

foreach ($a in $Arms) { Invoke-Arm $a $RunIndex }
Write-Output "`n===== results.txt ====="; Get-Content $results
