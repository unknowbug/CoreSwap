# check_gencpu_index.py —— gen_cpu 完整跑后 normal_vec_index 状态 + split 缺失原因
import json, sys, os, traceback
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
try:
    src = g.gen_cpu(fd)
    print('gen_cpu OK')
except Exception:
    print('gen_cpu EXCEPTION:')
    traceback.print_exc()
print('normal_vec_index 大小:', len(g.normal_vec_index))
# noodle_ridge_b@c0 是否在
for k in ['minecraft:noodle_ridge_b@c0', 'minecraft:continentalness@c0', 'minecraft:noodle@c0']:
    print(f'  {k}: {"IN" if k in g.normal_vec_index else "MISSING"} -> vi={g.normal_vec_index.get(k)}')
# noise_instances 的 _key 与 split key 一致性抽查
for i in [0, 168, 184, 192]:
    kind, p = g.noise_instances[i]
    print(f'  noise_instances[{i}] _key={p.get("_key","")}')
