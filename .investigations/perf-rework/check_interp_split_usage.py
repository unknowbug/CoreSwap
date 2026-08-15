# check_interp_split_usage.py —— 每个 interp 内容树实际用到哪些噪声 slot（split 行的子集）
# 关键：若 interp 内容树只用到少量噪声实例，GPU 可只上传这些 split 行 → 带宽大减
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)
g = sim.g

# eval_df_base 用 NOISE_SLOT_BASE(slot) = noise_slots[slot]['base']（split 行基址）
# 每个 interp delegate_root 闭包里的 DF_NOISE/DF_SHIFTED_NOISE 节点 a1 = noise slot id
def collect_noise_slots(root):
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = nodes[i]
        if n['type'] == sim.DF_WEIRD:
            visit(n['a1'])  # a1 = 树节点，a2 = ws 噪声 slot（不递归）
            return
        visit(n.get('a1', -1)); visit(n.get('a2', -1)); visit(n.get('a3', -1))
    visit(root)
    slots = set()
    for i in reach:
        n = nodes[i]
        if n['type'] in (sim.DF_NOISE, sim.DF_SHIFTED_NOISE):
            slots.add(n['a1'])
        if n['type'] == sim.DF_OLD_BLENDED:
            slots.add(n['a1'])
    return slots

print('split_total =', sim.SPLIT_TOTAL)
print('noise_slots =', len(g.noise_slots))
all_slots = set(range(len(g.noise_slots)))

total_split_floats = 0
for idx, root in enumerate(g.interp_roots):
    slots = collect_noise_slots(root)
    # 每 slot 的 split 行大小（stride 从 noise_slots 查）
    n_floats = 0
    for s in slots:
        ns = g.noise_slots[s]
        # split 行 = base..base+stride（大致；实际 per-slot 行宽需看 split 布局）
        n_floats += max(ns['stride'], 1)
    print(f'interp[{idx}] noise_slots={len(slots)} split_floats≈{n_floats}')
    total_split_floats = max(total_split_floats, n_floats)

print(f'最大 interp split_floats ≈ {total_split_floats}（vs 全局 {sim.SPLIT_TOTAL}）')
print(f'方案C优化: 1225 点 × {total_split_floats} × 4B = {1225*total_split_floats*4/1e6:.1f} MB/chunk（vs 212MB 全量）')
