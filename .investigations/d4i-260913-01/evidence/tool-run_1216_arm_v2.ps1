# run_1216_arm_v2.ps1 — 260913-01 1.21.6 单臂运行台 v2（三维度参数化）
#
# 相对 260912-02 的 run_1216_arm.ps1 的变更（逐条登记，供 judge 核对）：
#   ① **显式带冒号 `:runServer`**（#139 ② 计数载体纪律；任务图直证见 evidence/arm-commands-260913-01.txt）
#      —— 注：1.21.6 的 content-test 目录为空（无 build.gradle）⇒ 本版裸 runServer 只命中 :runServer 一个任务；
#         带冒号是**防御性写法**，防止将来补 content-test 后静默退化。1.20.1 侧才是真正的双命中缺陷面。
#   ② **三维度参数化** `-Dim`（overworld / the_nether / the_end）
#   ③ 逐维 **载体存在性断言**（#130）：[WG-CONTENT] 与 [WG-CONTENT-WB] 行数为 0 时显式标 CARRIER-MISSING
#   ④ 逐臂 **生效自证**（#139 ①）：由调用方传 -Props "-Dcoreswap.bulkwblog=1" 触发 [WG-BULKWB]/[WG-PERBLOCK] 行
#   ⑤ 保留：磁盘 dll 前置硬门 + in-log dll 自证 + 异常级联早退 + 两层指纹分层落盘
#
# 纪律：两版共用端口 ⇒ 臂必须串行；每臂 MUST 读自证行后才可跨臂比对。
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [string]$Props = "",
    [ValidateSet("overworld", "the_nether", "the_end")][string]$Dim = "overworld",
    [int]$Radius = 160,
    [int]$WaitFinishSec = 420,
    [string]$ExpectDll = "abd7d8893d22e030",
    [int]$ThrowAbortAt = 3,
    [switch]$KeepWorld
)
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$proj = "$root\versions\1.21.6\java"
$out  = "$root\.tmp\d4i-260913-01"
New-Item -ItemType Directory -Force -Path $out, "$root\.tmp\java-tmp" | Out-Null
$seed = "417950215108767439"
$dll  = "$root\target\release\worldgen1216.dll"
$log  = "$out\1216-$Tag.log"; $err = "$log.err"
$mcDim = "minecraft:$Dim"

# ---- 前置硬门：磁盘 dll 血统（不通过即拒绝开跑）----
if (-not (Test-Path $dll)) { Write-Output "[$Tag] FAIL worldgen1216.dll missing"; return }
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLower()
if ($dllSha.Substring(0,16) -ne $ExpectDll) {
    Write-Output "[$Tag] FAIL pre-flight dll mismatch: disk=$($dllSha.Substring(0,16)) expect=$ExpectDll"
    return
}
$rcon = "$root\.investigations\perf-reg-260910-07\cmd-output\rcon_one.py"
Write-Output "########## 1216 $Tag dim=$Dim start $(Get-Date -Format 'HH:mm:ss') radius=$Radius props='$Props' diskDll=$($dllSha.Substring(0,16)) ##########"

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

# 显式带冒号 :runServer（①）
$p = Start-Process -FilePath "gradle" -ArgumentList @(":runServer", "-PcppReplace=true", "-PcppLib=$dll",
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
    Write-Output "[$Tag] FAIL no Done"; Get-Content $log -Tail 20
    Get-Process java -EA SilentlyContinue | Stop-Process -Force; return
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
foreach ($c in @("chunky quiet 10", "chunky world $mcDim", "chunky center -48 -11", "chunky radius $Radius", "chunky start")) {
    & python $rcon $c 2>&1 | ForEach-Object { Write-Output "[$Tag][rcon] $_" }
}

# ---- 等生成完成；异常级联则早退 ----
$tStart = Get-Date; $deadline2 = (Get-Date).AddSeconds($WaitFinishSec); $finishLine = $null; $aborted = $false
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
# 接管计数：**两种实形并存**（本块 M1 二次修正）——
#   overworld: `[Mixin] populateNoise intercepted chunk(x,z)`（无维度名）
#   nether/end: `[Mixin] populateNoise(nether|end) intercepted chunk(x,z)`（带维度名）
# 统一正则用 `populateNoise(\(.*\))? intercepted` 兼收两形；旧的单形正则任取其一都会造成**假零**（#13 家族）。
$intercept= CountPat "populateNoise(\(.*\))? intercepted"
# 非 WMI 真实异常（#139 ②：计数须绑单次 JVM 运行；WMI/COM 行是环境良性噪声）
$cNonWmi  = (Select-String -Path $log -Pattern "Exception|Error:" -EA SilentlyContinue | Where-Object { $_.Line -notmatch "WmiQueryHandler|Win32_Processor|Win32_PhysicalMemory" } | Measure-Object).Count

# 载体存在性断言（③，#130）
$carrier = if (($cContent -gt 0) -and ($cWb -gt 0)) { "OK" } else { "CARRIER-MISSING" }

# 指纹：合并 + 分层（§9.7：Rust buf 层 vs Java 读回层，不得跨层比对）
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

Write-Output "[$Tag][result] dim=$Dim boot=${bootSec}s gen=${wallGen}s status=$finishTxt throws=$cThrow entryMissing=$cEntry exceptions=$cExc nonWmi=$cNonWmi intercept=$intercept content=$cContent wb=$cWb carrier=$carrier fpSha=$fpSha contentSha=$fpcSha wbSha=$fpwSha"
Write-Output "[$Tag][bulk] $bulkSum"
Write-Output "[$Tag][perblock] $pbSum"
Write-Output "[$Tag][verdict] dllAttest=$dllOk $(if(-not $dllOk){'ARM-VOID'}else{'attested'})"
Write-Output "########## 1216 $Tag dim=$Dim DONE $(Get-Date -Format 'HH:mm:ss') ##########"
