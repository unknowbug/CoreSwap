# collect_evidence.ps1 — 260912-01 C5：把 .tmp 判据证据复制进 tracked evidence/（含 MANIFEST.txt）
$ErrorActionPreference = "Continue"
$repo = "E:\PYTHON\CoreSwap"
$t = "$repo\.tmp\shared-java-core-260912-01"
$ev = "$repo\.investigations\shared-java-core-260912-01\evidence"
New-Item -ItemType Directory -Force -Path $ev | Out-Null
$env:PYTHONIOENCODING = "utf-8"

# 1) 清单（V1b 判据）
Copy-Item "$t\pre\manifest-1.20.1-pre.tsv" "$ev\manifest-pre-1.20.1.tsv" -Force
Copy-Item "$t\pre\manifest-1.21.6-pre.tsv" "$ev\manifest-pre-1.21.6.tsv" -Force
foreach ($p in @(@('w2\manifest2-1.20.1.tsv', 'manifest-post2-1.20.1.tsv'), @('w2\manifest2-1.21.6.tsv', 'manifest-post2-1.21.6.tsv'),
        @('w2\manifest3-1.20.1.tsv', 'manifest-post3-1.20.1.tsv'), @('w2\manifest3-1.21.6.tsv', 'manifest-post3-1.21.6.tsv'),
        @('w2\manifest4-1.20.1.tsv', 'manifest-post4-1.20.1.tsv'), @('w2\manifest4-1.21.6.tsv', 'manifest-post4-1.21.6.tsv'),
        @('w2\manifest-prewt-1.20.1.tsv', 'manifest-prewt-1.20.1.tsv'))) {
    Copy-Item "$t\$($p[0])" "$ev\$($p[1])" -Force
}

# 2) 差异输出（可复现命令见 MANIFEST 注释；退出码 1 = 工具对差异集判 FAIL，属预期）
python "$t\jar_manifest_diff.py" "$ev\manifest-pre-1.20.1.tsv" "$ev\manifest-post2-1.20.1.tsv" *> "$ev\v1b-1.20.1.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-pre-1.21.6.tsv" "$ev\manifest-post2-1.21.6.tsv" *> "$ev\v1b-1.21.6.txt"
python "$t\jar_manifest_diff.py" "$t\w2\manifest-1.20.1.tsv" "$ev\manifest-post2-1.20.1.tsv" *> "$ev\post1-vs-post2-1.20.1.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-post2-1.20.1.tsv" "$ev\manifest-post3-1.20.1.tsv" *> "$ev\post2-vs-post3-1.20.1.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-post2-1.21.6.tsv" "$ev\manifest-post3-1.21.6.tsv" *> "$ev\post2-vs-post3-1.21.6.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-post3-1.20.1.tsv" "$ev\manifest-post4-1.20.1.tsv" *> "$ev\post3-vs-post4-1.20.1.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-post3-1.21.6.tsv" "$ev\manifest-post4-1.21.6.tsv" *> "$ev\post3-vs-post4-1.21.6.txt"
python "$t\jar_manifest_diff.py" "$ev\manifest-prewt-1.20.1.tsv" "$ev\manifest-post3-1.20.1.tsv" *> "$ev\prewt-pseudodiff.txt"

# 3) 运行期门控行（两族指纹）+ 逐臂摘要 + 完整日志
$arms = @(
    @{ n = '1.20.1-post';  log = "$repo\.tmp\vivo-stutter-260911-02\ab-threads\logs\w2c-post-1.20.1.log" },
    @{ n = '1.20.1-pre-r1'; log = "$t\w2\1201-w2c-pre-1.20.1.log" },
    @{ n = '1.20.1-pre-r2'; log = "$t\w2\1201-w2d-pre-1.20.1.log" },
    @{ n = '1.21.6-default'; log = "$t\w2\1216-w2-post-1.21.6.log" },
    @{ n = '1.21.6-bulk';   log = "$t\w2\1216-w2-bulk-1.21.6.log" }
)
foreach ($a in $arms) {
    Select-String -Path $a.log -Pattern '\[WG-CONTENT\] chunk\(|\[WG-CONTENT-WB\] chunk\(' |
        ForEach-Object { ($_.Line -replace '^.*\[(WG-CONTENT(-WB)?)\]', '[$1]').Trim() } |
        Sort-Object | Set-Content "$ev\fp-$($a.n).txt" -Encoding utf8
    Copy-Item $a.log "$ev\log-$($a.n).txt" -Force
}
$sum = @()
foreach ($a in $arms) {
    $sum += "===== $($a.n)  ($($a.log)) ====="
    $sum += (Select-String -Path $a.log -Pattern '\[CppBridge\] dll=|\[CppBridge\] init |Task finished|\[result\]|exception-lines|EntryMissingException|BUILD SUCCESSFUL|BUILD FAILED' |
        ForEach-Object { ($_.Line -replace '^.*\[STDOUT\]:\s*', '').Trim() } | Select-Object -First 12)
    $sum += ("[counts] WG-CONTENT=" + (Select-String -Path $a.log -Pattern '\[WG-CONTENT\] chunk\(' -AllMatches | Measure-Object).Count +
        " WG-CONTENT-WB=" + (Select-String -Path $a.log -Pattern '\[WG-CONTENT-WB\] chunk\(' -AllMatches | Measure-Object).Count +
        " EntryMissing=" + (Select-String -Path $a.log -Pattern 'EntryMissingException' -AllMatches | Measure-Object).Count +
        " WMI=" + (Select-String -Path $a.log -Pattern 'WmiQueryHandler' -AllMatches | Measure-Object).Count +
        " testcontent=" + (Select-String -Path $a.log -Pattern 'testcontent' -AllMatches | Measure-Object).Count)
    $sum += ""
}
$sum | Set-Content "$ev\arm-summary.txt" -Encoding utf8

# 4) 复现工具与源码对照基线
foreach ($f in @('fp_compare.py', 'extract_entry.py', 'javap_method_diff.py', 'jar_manifest.py', 'jar_manifest_diff.py', 'class_home_check.py')) {
    if (Test-Path "$t\$f") { Copy-Item "$t\$f" "$ev\tool-$f" -Force }
}
Copy-Item "$t\w2\run_pre_1201.ps1" "$ev\tool-run_pre_1201.ps1" -Force
Copy-Item "$t\w2\run_1216_w2.ps1" "$ev\tool-run_1216_w2.ps1" -Force
Copy-Item "$t\w2\pre-1.20.1-CppBridge.java" "$ev\src-pre-1.20.1-CppBridge.java" -Force
Copy-Item "$t\w2\pre-1.21.6-CppBridge.java" "$ev\src-pre-1.21.6-CppBridge.java" -Force
Copy-Item "$t\w2\collect_evidence.ps1" "$ev\tool-collect_evidence.ps1" -Force
Copy-Item "$t\w2\build4-1.20.1.log" "$ev\log-build4-1.20.1.txt" -Force
Copy-Item "$t\w2\build4-1.21.6.log" "$ev\log-build4-1.21.6.txt" -Force

# 5) MANIFEST（排除自身；含 jar sha 与复现命令注释）
$lines = @()
$lines += "# 260912-01 Wave 2 证据清单（sha256  +  size  +  name  +  source）"
$lines += "# jar sha（权威 dll 态，post3 ≡ post4）：1.20.1 461baedcb729dc9b… 1.21.6 772d7a6ed0bbdbfc…"
$lines += "# 冻结 pre jar：1.20.1 1027f4f6403aca26…  1.21.6 16d5e5e7adf0780c…"
$lines += "# post2（V1b 判定基线）：1.20.1 0681ec03…  1.21.6 772d7a6e…"
$lines += "# dll：1.20.1 dd3b645f2c79d2cb…  1.21.6 abd7d8893d22e030…（票/target/jar 三处一致）"
$lines += "# 复现：python tool-jar_manifest_diff.py manifest-pre-1.20.1.tsv manifest-post2-1.20.1.tsv"
$lines += "#       python tool-fp_compare.py log-1.20.1-pre-r2.txt log-1.20.1-post.txt '\[WG-CONTENT-WB\] chunk'"
$lines += "#       pwsh tool-run_pre_1201.ps1 -Tag <tag> -Radius 160   （pre 臂，需 .tmp/w2/pre-wt worktree @ ce5286b）"
$lines += ""
Get-ChildItem $ev -File | Where-Object { $_.Name -ne 'MANIFEST.txt' } | Sort-Object Name | ForEach-Object {
    $lines += ("{0}  {1,9}  {2}" -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Length, $_.Name)
}
$lines | Set-Content "$ev\MANIFEST.txt" -Encoding utf8
Write-Output "[collect] files=$( (Get-ChildItem $ev -File).Count ) ev=$ev"
