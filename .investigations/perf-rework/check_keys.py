# check_keys.py —— 对比 normal_vec_index keys vs split 遍历 keys（noodle 家族）
import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
print('=== normal_vec_index 中 noodle 相关 key ===')
for k, v in g.normal_vec_index.items():
    if 'noodle' in k:
        print(f'  normal_vec_index[{k}] = {v}')
print('=== noise_instances 中 noodle 实例 ===')
for i, (kind, p) in enumerate(g.noise_instances):
    if 'noodle' in str(p.get('_key','')):
        print(f'  noise_instances[{i}] key={p.get("_key","")}')
print('=== coord_chains 长度 ===', len(g.coord_chains))
print('=== noise_instances 155..168 ===')
for i in range(155, 169):
    kind, p = g.noise_instances[i]
    print(f'  [{i}] {kind} key={p.get("_key","")[:60]}')
for k, v in g.normal_chain_index.items():
    if 'noodle' in k:
        print(f'  normal_chain_index[{k}] = {v}')
print('=== normal_split_base 中 noodle key ===')
for k, v in g.normal_split_base.items():
    if 'noodle' in k:
        print(f'  normal_split_base[{k}] = {v}')
