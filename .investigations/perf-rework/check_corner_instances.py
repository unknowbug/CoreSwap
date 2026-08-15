# check_corner_instances.py —— 查看角点分组的实例分配：corner 偏移的实例是什么
# NOISE_SLOT_BASE[slot] + corner*STRIDE = 实例索引？实例是同一噪声的不同副本？
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

g = sim.g
# 看 slot 与实例的映射：NOISE_SLOT_BASE(slot) + corner*STRIDE(slot)
# slot 的 base/stride
for s in range(5):
    ns = g.noise_slots[s]
    print(f'slot[{s}]: base={ns["base"]} stride={ns["stride"]}')
    for c in range(4):
        inst = ns['base'] + c * ns['stride']
        print(f'   corner{c} -> 实例 {inst}: {g.noise_instances[inst] if inst < len(g.noise_instances) else "OOB"}')
