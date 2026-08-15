# check_interp_real_split.py —— 精确算 interp 内容树需要的 split floats（按实例）
# 每个 slot 对应一个 noise_instances 条目，其 split 行大小 = 实例的维度数（x+y+z 拆分）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)
g = sim.g

# noise_slots[slot] 有 base/stride——但 stride 是 slot 内实例数？看 dbg_full_sim 怎么用 NOISE_SLOT_BASE
# eval_df_base: normal_noise(NOISE_SLOT_BASE(a1) + corner*NOISE_SLOT_STRIDE(a1), sIdx)
# normal_noise(idx, sIdx) 读 splitCoord[sIdx*SPLIT_TOTAL + ?]——实例 idx 的 split 行
# 所以 slot->实例 idx（NOISE_SLOT_BASE(slot)=实例基址），实例的 split 行宽需查 noise_instances
# 看 noise_instances 结构
print('noise_instances:', len(g.noise_instances))
if hasattr(g, 'noise_instances') and g.noise_instances:
    print('sample keys:', list(g.noise_instances[0].keys()) if isinstance(g.noise_instances[0], dict) else type(g.noise_instances[0]))
    print('first:', g.noise_instances[0])
