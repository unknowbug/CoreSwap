# check_interp_bandwidth2.py —— 方案 C 优化带宽：interp 内容树用到的实例 split 行精确累加
# slot -> NOISE_SLOT_BASE(slot) = 实例序号？看 dbg_full_sim 的 noise_slots 与 NORMAL/OLD 索引关系
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)
g = sim.g

# 每个 interp delegate_root 用到的噪声 slot
def collect_slots(root):
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = nodes[i]
        if n['type'] == sim.DF_WEIRD:
            visit(n['a1']); return
        visit(n.get('a1',-1)); visit(n.get('a2',-1)); visit(n.get('a3',-1))
    visit(root)
    slots = set()
    for i in reach:
        n = nodes[i]
        if n['type'] in (sim.DF_NOISE, sim.DF_SHIFTED_NOISE, sim.DF_OLD_BLENDED):
            slots.add(n['a1'])
    return slots

# slot -> 实例序号：NOISE_SLOT_BASE(slot) 是 split 基址（0,8,16,...）——不是实例序号！
# 实际：dbg_full_sim eval_df_base 用 normal_noise(NOISE_SLOT_BASE(a1) + corner*STRIDE, sIdx)
# NOISE_SLOT_BASE(slot) = slot 的 base 值（0,8,16,...192）——是 split 行内的偏移！
# 但 normal_noise(noiseIdx,...) 的 noiseIdx = NORMAL dict 的 key（实例序号）
# 所以 slot.base 可能就是实例序号？slot[0].base=0, slot[8].base=64... 看 noise_slots 定义
# 查 dbg_full_sim 怎么构建 noise_slots
print('noise_slots[0]:', g.noise_slots[0])
print('noise_slots[8]:', g.noise_slots[8])
# 查 NORMAL key 范围
nkeys = sorted(sim.NORMAL.keys())
print('NORMAL keys: min=%d max=%d count=%d' % (nkeys[0], nkeys[-1], len(nkeys)))
print('OLD keys:', sorted(sim.OLD.keys()))

# 精确：interp 用到的 slot -> 实例行宽
def instance_row(inst_idx):
    if inst_idx in sim.NORMAL:
        return sim.NORMAL[inst_idx]['n'] * 12
    if inst_idx in sim.OLD:
        # OLD 行宽 = 280（splitBase 间隔）
        return 280
    return 0

for idx, root in enumerate(g.interp_roots):
    slots = collect_slots(root)
    total = sum(instance_row(s) for s in slots)
    print(f'interp[{idx}]: {len(slots)} slots, {total} split floats/点')
    # 若 slot.base 不是实例序号，打印映射说明
    for s in sorted(slots):
        print(f'    slot[{s}] base={g.noise_slots[s]["base"]} row={instance_row(s)}')
