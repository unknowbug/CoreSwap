# check_split_lines.py —— 对比 trace 遍历到的噪声 vs split() 实际生成的行
import json, sys, os, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
src = g.gen_cpu(fd)
# 提取 split() 函数体
m = re.search(r'void split\(int x, int y, int z, float\* out\) \{(.*?)\n    \}', src, re.S)
body = m.group(1)
lines = [l for l in body.split('\n') if 'splitDouble' in l or 'splitOldBlended' in l]
print(f'split() 拆分行数: {len(lines)}')
# 每行提取 normals[k]
idxs = [int(mm.group(1)) for l in lines for mm in [re.search(r'normals\[(\d+)\]', l)] if mm]
print(f'normals 索引范围: {min(idxs)}..{max(idxs)}  共 {len(idxs)} 个')
# 缺哪些索引（0..199）
missing = [i for i in range(200) if i not in idxs]
print(f'缺失 normals 索引: {missing}')
# 每行 base
bases = [int(mm.group(1)) for l in lines for mm in [re.search(r', out, (\d+),', l)] if mm]
print(f'base 范围: {min(bases)}..{max(bases)}')
print('最后 3 行:')
for l in lines[-3:]:
    print(' ', l[:120])
