# sim_single_point.py —— D23 决定性判别：Python 复刻 GPU 解释器对单点 (784,160,-408)
# 若模拟 = 0.045（GPU 值）→ 生成器/解释器共同 bug（H5，非 GPU）；若 = -0.458（CPU 参照）→ GPU kernel 特有。
# 复用 dbg_full_sim.py 的完整 eval 逻辑（import 它并跑单点）。
import sys, os, json, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
# dbg_full_sim 顶层直接读 dump 并跑 eval——改为 import 后手动单点（需先执行其模块级初始化）
spec.loader.exec_module(sim)

# dbg_full_sim 模块级已读 split_dump.bin（1024 点）。替换为单点数据：
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.coords = [(784, 160, -408)]
sim.SPLIT_TOTAL = 8672

px, py, pz = 784, 160, -408
# 顶层 eval_df（与 dbg_full_sim main 相同调用）
try:
    r = sim.eval_df(sim.N - 1, 0, px, py, pz)
    if isinstance(r, tuple):
        print('sim result (tuple):', r)
    else:
        print(f'sim eval_df({px},{py},{pz}) = {r:.9f}')
except Exception as e:
    print('sim error:', e)
# interp_N 直接（interp_4 = idx 4）
try:
    r4 = sim.interp_N(4, 0, px, py, pz)
    if isinstance(r4, tuple):
        print('sim interp_4 (tuple):', r4)
    else:
        print(f'sim interp_4({px},{py},{pz}) = {r4:.9f}')
except Exception as e:
    print('sim interp_4 error:', e)
print('CPU 参照 = -0.458333（finalDensity）；GPU = 0.045303289')
