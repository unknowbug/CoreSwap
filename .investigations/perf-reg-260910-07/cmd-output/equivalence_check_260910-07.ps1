# equivalence_check_260910-07.ps1 — 迁移等价性门（260910-07，可原位复跑）
# 用途：对「迁移前(旧路径) / 迁移后(新路径)」同一版本的 jar 做**条目级 + 整文件级**比对，
#       把「只搬位置、不改语义」变成二值强判据（→ knowledge/discovered/workflow-patterns #116）。
# 用法：pwsh equivalence_check_260910-07.ps1 -Version 1.20.1
#       旧 jar 默认取 runtime/<ver>/java/build/libs/*.jar（迁移前原地保留，**勿删**）；
#       新 jar 默认取 versions/<ver>/java/build/libs/*.jar（迁移后重建；若不存在先 `gradle :build`）。
# 产出：追加/覆盖式打印到 stdout（本文件由 .investigations/perf-reg-260910-07/cmd-output/equivalence-260910-07.txt 收录）。
param(
    [Parameter(Mandatory = $true)][ValidateSet('1.20.1', '1.21.6')][string]$Version,
    [string]$OldJar,
    [string]$NewJar,
    [string]$TargetDll
)
$ErrorActionPreference = 'Continue'
$root = 'E:\PYTHON\CoreSwap'
if (-not $OldJar) {
    $OldJar = (Get-ChildItem "$root\runtime\$Version\java\build\libs\*.jar" -EA SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}
if (-not $NewJar) {
    $NewJar = (Get-ChildItem "$root\versions\$Version\java\build\libs\*.jar" -EA SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}
if (-not $TargetDll) {
    $dllName = if ($Version -eq '1.21.6') { 'worldgen1216.dll' } else { 'worldgen.dll' }
    $TargetDll = "$root\target\release\$dllName"
}
foreach ($p in @($OldJar, $NewJar, $TargetDll)) {
    if (-not (Test-Path $p)) { Write-Output "[FAIL] missing: $p"; exit 1 }
}
# judge C14：解包依赖 jar.exe，未设/失效必须**响亮失败**——否则解包为空会让下面「0/0/0」假绿通过（#23/#25/#40 家族）
$jarExe = Join-Path $env:JAVA_HOME 'bin\jar.exe'
if (-not (Test-Path $jarExe)) { Write-Output "[FAIL] jar.exe not found (JAVA_HOME='$env:JAVA_HOME')"; exit 1 }
Write-Output "# equivalence_check_260910-07  version=$Version  $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Output "old jar    = $OldJar"
Write-Output "new jar    = $NewJar"
Write-Output "target dll = $TargetDll"

$work = Join-Path $env:TEMP ("eq-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path "$work\old", "$work\new" | Out-Null
Copy-Item $OldJar "$work\old.jar" -Force; Copy-Item $NewJar "$work\new.jar" -Force
Push-Location "$work\old"; & $jarExe xf ../old.jar 2>$null; Pop-Location
Push-Location "$work\new"; & $jarExe xf ../new.jar 2>$null; Pop-Location

function Entries([string]$dir) {
    Get-ChildItem $dir -Recurse -File | Where-Object { $_.FullName -notmatch '\\META-INF\\' } |
        ForEach-Object { @{ Rel = $_.FullName.Replace("$dir\", '').Replace('\', '/'); Sha = (Get-FileHash $_.FullName -Algorithm SHA256).Hash } }
}
$oh = @{}; Entries "$work\old" | ForEach-Object { $oh[$_.Rel] = $_.Sha }
$nh = @{}; Entries "$work\new" | ForEach-Object { $nh[$_.Rel] = $_.Sha }
# judge C14：空解包守卫——entries=0 时下面「0/0/0」会假绿（#23/#25/#40 家族）
if ($oh.Count -eq 0 -or $nh.Count -eq 0) { Write-Output "[FAIL] empty extraction (entries old=$($oh.Count) new=$($nh.Count)) - JAVA_HOME/jar.exe?"; exit 2 }
if (-not (Test-Path "$work\new\native\worldgen.dll")) { Write-Output "[FAIL] jar-inner native/worldgen.dll missing (jar layout changed?)"; exit 2 }
$onlyOld = @($oh.Keys | Where-Object { -not $nh.ContainsKey($_) })
$onlyNew = @($nh.Keys | Where-Object { -not $oh.ContainsKey($_) })
$contentDiff = @($oh.Keys | Where-Object { $nh.ContainsKey($_) -and $nh[$_] -ne $oh[$_] })

Write-Output "entries old=$($oh.Count) new=$($nh.Count) | only-old=$($onlyOld.Count) only-new=$($onlyNew.Count) content-diff=$($contentDiff.Count)"
if ($onlyOld.Count) { Write-Output ("  only-old: " + ($onlyOld -join ', ')) }
if ($onlyNew.Count) { Write-Output ("  only-new: " + ($onlyNew -join ', ')) }
if ($contentDiff.Count) { Write-Output ("  content-diff: " + (($contentDiff | Select-Object -First 10) -join ', ')) }
$oldSha = (Get-FileHash $OldJar -Algorithm SHA256).Hash.ToLower()
$newSha = (Get-FileHash $NewJar -Algorithm SHA256).Hash.ToLower()
Write-Output "old jar sha = $oldSha"
Write-Output "new jar sha = $newSha"
Write-Output ("whole-file identical = " + ($oldSha -eq $newSha))
$tSha = (Get-FileHash $TargetDll -Algorithm SHA256).Hash.ToLower()
$iSha = (Get-FileHash "$work\new\native\worldgen.dll" -Algorithm SHA256).Hash.ToLower()
Write-Output "target dll    = $tSha"
Write-Output "jar-inner dll = $iSha"
Write-Output ("TRIPLE MATCH = " + ($tSha -eq $iSha))
Remove-Item $work -Recurse -Force -EA SilentlyContinue
if ($onlyOld.Count -or $onlyNew.Count -or $contentDiff.Count -or ($oldSha -ne $newSha) -or ($tSha -ne $iSha)) { exit 2 } else { Write-Output "[OK] migration equivalence gate PASSED"; exit 0 }
