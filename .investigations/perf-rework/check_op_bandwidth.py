# check_op_bandwidth.py —— FP32 算子库带宽账：单算子 vs 完整树
# 单算子（如 continentalness 噪声）每点只需 1 个实例的 split 行
# 完整树每点 8672 floats。对比上传量。
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

# NORMAL 实例行宽：n*6*2 floats（n = octave 数）
print('=== 单算子 split 行宽（n*6*2 floats/点）===')
rows = {}
for idx, mm in sim.NORMAL.items():
    n = mm['n']
    rows[idx] = n * 12
    if idx < 12 or idx % 20 == 0:
        print(f'  NORMAL[{idx}]: n={n} row={n*12} floats')

# 一些关键噪声实例（interp 内容树用的）
print()
print('=== 关键算子带宽对比（每点 floats）===')
key_instances = {
    'continentalness(interp[0] 用)': None,
    'erosion': None,
    'jagged': None,
}
# 找 continentalness 实例
for idx, mm in sim.NORMAL.items():
    pass  # NORMAL 只存数值，实例名在 noise_instances

# 用 noise_instances 找关键噪声
for inst_idx, (kind, params) in enumerate(sim.g.noise_instances):
    key = params.get('noise', params.get('_key', ''))
    if inst_idx < 40 and ('continentalness' in key or 'erosion' in key or 'jagged' in key):
        if 'c0' in key or '@c' not in key:
            row = rows.get(inst_idx, '?')
            print(f'  实例[{inst_idx}] {kind} {key}: split行={row} floats')

print()
print('=== 带宽对比（N 点批量）===')
N = 100000  # 10 万点
full = N * 8672 * 4 / 1e6
op_small = N * 12 * 4 / 1e6  # 1 octave 实例
op_large = N * 192 * 4 / 1e6  # 16 octave 实例（jagged）
print(f'完整树: {N}点 × 8672 × 4B = {full:.0f} MB')
print(f'单算子(1 octave): {N}点 × 12 × 4B = {op_small:.1f} MB')
print(f'单算子(16 octave jagged): {N}点 × 192 × 4B = {op_large:.0f} MB')
print(f'带宽比: 完整树/单算子 = {full/op_small:.0f}x (1 octave)')
