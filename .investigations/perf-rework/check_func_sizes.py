"""检查 D1 后 shader 的函数体大小。"""
import re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
src = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()
lines = src.split('\n')
print(f"shader lines: {len(lines)}")
# 找所有函数定义（interp_/df_/normal_noise/spline_eval）
defs = []
for i, l in enumerate(lines):
    m = re.match(r'^float ([\w_]+)\(', l)
    if m:
        name = m.group(1)
        size = 0
        j = i
        depth = 0
        while j < len(lines):
            size += len(lines[j])
            depth += lines[j].count('{') - lines[j].count('}')
            if depth <= 0 and j > i:
                break
            j += 1
        defs.append((name, size, i + 1))
# 汇总
from collections import defaultdict
agg = defaultdict(lambda: [0, 0, 0])  # name -> [count, max_size, total]
for name, size, ln in defs:
    base = re.match(r'(interp_\d+|df_\d+|normal_noise\w*|spline\w*|interp_noise_\d+)', name)
    base = base.group(1) if base else name
    agg[base][0] += 1
    agg[base][1] = max(agg[base][1], size)
    agg[base][2] += size
print("=== 函数族汇总 ===")
for base in sorted(agg, key=lambda k: -agg[k][1]):
    cnt, mx, tot = agg[base]
    print(f"  {base}: {cnt} 个, 最大 body {mx} chars, 总计 {tot} chars")
print("=== 最大 10 个函数体 ===")
for name, size, ln in sorted(defs, key=lambda x: -x[1])[:10]:
    print(f"  L{ln} {name}: {size} chars")
