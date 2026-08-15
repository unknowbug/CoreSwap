# check_split_base.py —— 对账 split() 写入 base vs NORMAL_PACK splitBase
import re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
src = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\cpu_backend.h', encoding='utf-8').read()
m = re.search(r'void split\(int x, int y, int z, float\* out\) \{(.*?)void collectPerm', src, re.S)
body = m.group(1)
for k in [160, 168, 176, 184, 192]:
    found = False
    for l in body.split('\n'):
        if re.search(rf'normals\[{k}\]', l):
            b = re.search(r', out, (\d+),', l)
            print(f'split normals[{k}]: base={b.group(1) if b else "?"}')
            found = True
            break
    if not found:
        print(f'split normals[{k}]: NOT FOUND')
