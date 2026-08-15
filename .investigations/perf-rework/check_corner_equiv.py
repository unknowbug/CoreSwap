# check_corner_equiv.py —— 验证 GPU 角点分组噪声 vs CPU 共享实例的等价性
# 若等价 → GPU 可改「共享实例」结构 → 方案 C 复活（1225 网格角点可独立算）
# 对比：GPU 的 eval_df_base_{idx}(corner=c, 角点坐标) vs CPU 共享实例 arg->sample(角点坐标)
# 用 sim：eval_df_base(root, corner=c, ...) 是 GPU 同构；对比不同 corner 在同一坐标的值
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

def eval_base(interp_idx, corner, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    r = sim.eval_df_base(root, corner, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

# 同一坐标、不同 corner 的值——若 corner 只影响「噪声实例索引」而实例对应不同坐标采样，
# 则同一坐标不同 corner 应不同（角点分组）；若等价则相同
pts = [(0,-64,0),(8,-64,0),(4,-56,0),(0,-48,0),(12,-64,0),(4,-64,4),(8,-56,8)]
print('同一坐标不同 corner 的 interp[0] 值（角点分组 vs 共享）：')
for (x,y,z) in pts:
    vals = []
    for c in range(8):
        v = eval_base(0, c, 0, x, y, z)
        vals.append(v)
    diff = max(vals) - min(vals)
    print(f'  ({x},{y},{z}) c0={vals[0]:.6f} c1={vals[1]:.6f} ... c7={vals[7]:.6f} range={diff:.2e}')
