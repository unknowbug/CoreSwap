# 历史代理基线（0.0150%）同口径复算：出处 = .tmp/hang-repro-260910/diff-result.txt:74（未归档，region 仍在）
$ErrorActionPreference = "Continue"
$root = "E:\PYTHON\CoreSwap"
$dir  = "$root\.investigations\e5-recompute-260911-05"
$py   = "$dir\diff_arms_fixed.py"
$outd = "$dir\recompute"
$a = "$root\.tmp\hang-repro-260910\arms\region-coreswap"
$b = "$root\.tmp\hang-repro-260910\arms\region-vanilla"
"=== 旧口径一手出处（diff-result.txt 第 70-80 行）==="
Get-Content "$root\.tmp\hang-repro-260910\diff-result.txt" | Select-Object -Skip 69 -First 12
"=== 规范读法复算 ==="
python $py $a $b *> "$outd\diff-hist-baseline.txt"
Get-Content "$outd\diff-hist-baseline.txt" | Select-String -Pattern "^vanilla chunks|^coreswap chunks|^common=|^blocks=" | ForEach-Object { $_.Line }
"=== 自比（负对照：region-coreswap 自比应 0 差）==="
python $py $a $a *> "$outd\diff-hist-self.txt"
Get-Content "$outd\diff-hist-self.txt" | Select-String -Pattern "^common=|^blocks=" | ForEach-Object { $_.Line }
