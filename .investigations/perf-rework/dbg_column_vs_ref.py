# 全列模拟 vs 参照（e2e 参照列已知值）
import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ref = {  # e2e 参照列 (x=0,z=0)，N=1024 覆盖 y=-64..-49（-49 未采集，跳过）
    -64: 0.037482422, -62: 0.036994793, -60: 0.036507146, -58: 0.036019482,
    -56: 0.035531801, -54: 0.040212155, -52: 0.044890742, -50: 0.049567355,
}
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)
# 复用 dbg_full_sim 的实现
import runpy
sim_mod = runpy.run_path(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
eval_df = sim_mod['eval_df']
nodes = g.df_nodes
# coords_dump: sIdx → (x=i%64, y=-64+(i/64%16), z=0)。(0,y,0) 列 = sIdx = (y+64)*64
maxdiff = 0.0
for y in sorted(ref):
    sIdx = (y + 64) * 64
    r = eval_df(len(nodes) - 1, sIdx, 0, y, 0)
    d = abs(r - ref[y])
    maxdiff = max(maxdiff, d)
    print('y=%d sim=%.9f ref=%.9f diff=%.3e' % (y, r, ref[y], d))
print('maxDiff(全列): %.3e' % maxdiff)
