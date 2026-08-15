# sim_spline36.py —— sim 单独调 spline_eval(36) 对 (784,160,-408) corner0，看为什么 0
import sys, os, json, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.coords = [(784, 160, -408)]
sim.SPLIT_TOTAL = 8672
px, py, pz = 784, 160, -408
for root in [36, 55]:
    try:
        v = sim.spline_eval_py(root, 0, 0, px, py, pz)
        print(f'spline_eval({root}, corner0, {px},{py},{pz}) = {v}')
    except Exception as e:
        print(f'spline_eval({root}) err: {e}')
# spline 36/55 的数据（spline_ssbo_nodes）
g = sim.g
for i, nd in enumerate(g.spline_ssbo_nodes):
    if i in (36, 55) or i < 3:
        print(f'spline_node[{i}]: coordType={nd["coordType"]} n={nd["n"]} locBegin={nd["locBegin"]} derBegin={nd["derBegin"]} valBegin={nd["valBegin"]}')
# coordType -> coordinate 表达式
print('spline_coords:', g.spline_coords)
