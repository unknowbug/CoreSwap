# check_split_row_size.py —— 每个噪声 slot 的 split 行实际大小（float 数）
# split_total=8672 对应 25 个 slot 的总和？还是别的分配？看 noise_slots 布局
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

g = sim.g
print(f'noise_slots count = {len(g.noise_slots)}, split_total = {sim.SPLIT_TOTAL}')
for s in range(len(g.noise_slots)):
    ns = g.noise_slots[s]
    print(f'  slot[{s}]: base={ns.get("base")} stride={ns.get("stride")}')
print('---')
# 汇总：每个 slot 行宽 = stride（或推断）
total = 0
for s in range(len(g.noise_slots)):
    ns = g.noise_slots[s]
    stride = ns.get('stride', 0)
    total += stride
    print(f'  slot[{s}] stride={stride} (base={ns.get("base")})')
print(f'sum(stride) = {total} vs split_total = {sim.SPLIT_TOTAL}')
