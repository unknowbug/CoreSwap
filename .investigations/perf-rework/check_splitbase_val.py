# check_splitbase_val.py —— normal_split_base 关键值 vs split 写入 vs NORMAL_PACK
import json, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
g.gen_cpu(fd)
for k in ['minecraft:noodle@c0', 'minecraft:noodle_thickness@c0', 'minecraft:noodle_ridge_a@c0', 'minecraft:noodle_ridge_b@c0',
          'minecraft:continentalness@c0', 'minecraft:pillar_thickness@c0']:
    sb = g.normal_split_base.get(k)
    vi = g.normal_vec_index.get(k)
    print(f'{k}: vi={vi} splitBase={sb}')
