# check_two_alloc.py —— 对比 gen_shader 与 gen_cpu 的 splitBase 分配（noodle@c0）
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
# gen_shader 分配
g.gen_shader(fd)
sb_shader = dict(g.normal_split_base)
vi_shader = dict(g.normal_vec_index)
# gen_cpu 重新分配（reset 后）
g2 = dfc_gen.DfcGen(dfdir, ndir)
g2.gen_df(fd)
g2.gen_cpu(fd)
sb_cpu = dict(g2.normal_split_base)
vi_cpu = dict(g2.normal_vec_index)
for k in ['minecraft:noodle@c0', 'minecraft:continentalness@c0', 'minecraft:pillar_thickness@c0']:
    print(f'{k}: shader vi={vi_shader.get(k)} sb={sb_shader.get(k)} | cpu vi={vi_cpu.get(k)} sb={sb_cpu.get(k)}')
# 全部对比
diff = {k: (sb_shader.get(k), sb_cpu.get(k)) for k in sb_shader if sb_shader.get(k) != sb_cpu.get(k)}
print(f'splitBase 不一致数: {len(diff)}')
for k, (a, b) in list(diff.items())[:5]:
    print(f'  {k}: shader={a} cpu={b}')
