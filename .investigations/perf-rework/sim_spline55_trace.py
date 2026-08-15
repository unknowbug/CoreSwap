# sim_spline55_trace.py —— trace spline_eval(55) 的边界分支
import sys, os, json, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.SPLIT_TOTAL = 8672
# 手动跑 spline 55 的 find_range
nd = sim.g.spline_ssbo_nodes[55]
ct, n, lb, db, vb = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
coord = sim.spline_coord_py(ct, 0, 0, 784, 160, -408)
print(f'spline55: coord={coord} locs={sim.g.spline_ssbo_locs[lb:lb+n]} kind={sim.g.spline_ssbo_val_kind[vb:vb+n]} node={sim.g.spline_ssbo_val_node[vb:vb+n]}')
i = sim.spline_find_range(coord, lb, n)
print(f'find_range -> i={i} (n-1={n-1})')
print(f'右边界端点 kind={sim.g.spline_ssbo_val_kind[vb+n-1]} node={sim.g.spline_ssbo_val_node[vb+n-1]}')
# 跑 spline_eval 看步骤
v = sim.spline_eval_py(55, 0, 0, 784, 160, -408)
print(f'spline_eval(55) = {v}')
