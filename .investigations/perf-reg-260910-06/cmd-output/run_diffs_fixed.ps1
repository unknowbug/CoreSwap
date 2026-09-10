$root = "E:\PYTHON\CoreSwap"
$inv = "$root\.investigations\perf-reg-260910-06\cmd-output"
$tool = "$root\.tmp\perf-reg-260910-06\diff_1201.py"
$outd = "$root\.tmp\perf-reg-260910-06\diffs-fixed"
New-Item -ItemType Directory -Force -Path $outd | Out-Null
$env:PYTHONIOENCODING = "utf-8"
$pairs = @(
    @{ n = "self_os"; a = "region-os-r1"; b = "region-os-r1" },
    @{ n = "ctrl_vanilla_x_async"; a = "region-ovan-r1"; b = "region-oa-r1" },
    @{ n = "anchor_sync"; a = "region-os-r1"; b = "region-os-r2" },
    @{ n = "anchor_async"; a = "region-oa-r1"; b = "region-oa-r2" },
    @{ n = "cross_os1_x_oa1"; a = "region-os-r1"; b = "region-oa-r1" },
    @{ n = "cross_os2_x_oa2"; a = "region-os-r2"; b = "region-oa-r2" },
    @{ n = "sanity_mixlog_oa1_x_oaS1"; a = "region-oa-r1"; b = "region-oaS-r1" }
)
foreach ($p in $pairs) {
    $o = "$outd\diff-$($p.n).txt"
    Write-Output "===== $($p.n) ====="
    python $tool "$inv\$($p.a)" "$inv\$($p.b)" *> $o
    Select-String -Path $o -Pattern '^A chunks=|^common=|^box |^blocks=' | ForEach-Object { $_.Line }
}
