# check_interp_split_exact.py —— 精确计算 interp 内容树需要的 split floats/点
# 每个 slot → 噪声实例 → 实例的 split 行宽（normal_noise: n_octaves*6*2；old_blended: 7+）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)
g = sim.g

# NORMAL 元数据：每实例的 splitBase + 行宽（从 dbg_full_sim 的 NORMAL dict）
# NORMAL[noiseIdx] = {n, octBase, splitBase, ...}——n = octave 数，行宽 = n*6*2
print('NORMAL instances:', len(sim.NORMAL))
# 汇总每实例行宽
rows = {}
for idx, mm in sim.NORMAL.items():
    n = mm['n']; sb = mm['splitBase']
    rows[idx] = (sb, n * 6 * 2)
    print(f'  NORMAL[{idx}]: n={n} splitBase={sb} row={n*6*2} floats')
# OLD 实例（old_blended）
print('OLD instances:', len(sim.OLD))
for idx, mm in sim.OLD.items():
    sb = mm.get('splitBase', mm.get('base', '?'))
    print(f'  OLD[{idx}]: {mm.get("n","?")} oct splitBase={sb}')
