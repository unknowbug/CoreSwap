# check_spline_locfns.py —— 统计 SplineDF locationFunctions 构成
# 每个 spline 节点的 locationFunction 类型（FlatCache / noise / 嵌套 spline）
# 推断 spline 92μs 的根源：locationFunction 慢（FlatCache miss → buildGrid 递归）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

# sim 的 spline 结构：snodes = spline_ssbo_nodes（GPU 布局）
# 但 locationFunction 是 CPU 侧的 DF——sim 只有 spline 的 coordType/loc/der/val
# 换思路：看 dfc_gen.py 生成 spline 时 locationFunction 怎么注册（CPU side）
print('sim snodes:', len(sim.snodes))
for i in range(min(6, len(sim.snodes))):
    nd = sim.snodes[i]
    print(f'  spline[{i}]: coordType={nd["coordType"]} n={nd["n"]}')
