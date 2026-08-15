# check_schemeC_semantics.py —— 验证方案 C 正确性前提：
# GPU/sim 的 eval_df_base(interp_roots[idx]) = interp 内容树值 = CPU InterpolatedDF.arg 值
# 用 sim（与 GPU 同生成产物）算内容树值，对比 dbg_full_sim 的 eval_df（完整树）里 interp 的作用
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

# sim 的 eval_df_base(root, corner, sIdx, ix, iy, iz) = 内容树求值器（与 GPU eval_df_base_{idx} 同构）
# 验证：对 interp[0] 的 delegate_root，算 1225 网格角点的值（chunk 0）
def content_tree_value(interp_idx, corner, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    r = sim.eval_df_base(root, corner, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

# 与完整树 eval_df 里 DF_INTERP 节点对比：找 DF_INTERP 节点，它的值 = interp_0(...)
# eval_df 顶层闭包里 DF_INTERP 分支调 interp_N——验证 interp_N 与内容树一致性
# 直接验证：interp[0] 内容树在网格角点的值（应平滑——内容是 initialDensity 链）
import random
random.seed(42)
vals = []
for _ in range(20):
    gx = random.randint(0, 60)
    gy = random.randint(0, 300)
    gz = random.randint(0, 60)
    v = content_tree_value(0, 0, 0, gx, gy, gz)
    if v is not None:
        vals.append((gx, gy, gz, v))
print('interp[0] 内容树值采样（应为平滑密度链，非 0）：')
for (gx, gy, gz, v) in vals[:10]:
    print(f'  ({gx},{gy},{gz}) = {v:.6f}')

# 关键验证：方案 C 的完整链 = 外层非线性(interp 插值) —— 用 sim 手动重建
# CPU 正确路径: finalDensity->sample = 完整树（GPU eval_df 已验证一致）
# 方案 C: 对 interp 节点用「内容树网格角点 + CPU 插值」，其余节点照常
# 等价验证: sim.eval_df(完整树) vs 手动「内容树角点插值 + 外层」——检查能否逐位一致
print()
print('验证 sim 的 eval_df_base 与 eval_df 里 interp 用法的等价性：')
# eval_df 里 DF_INTERP 节点: interp_N(sIdx, ix, iy, iz) = 8 角点 eval_df_base + 插值
# 手动模拟 interp_N: 用 eval_df_base 算 8 角点 + 三线性
def interp_manual(interp_idx, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX*16; gy = iy + 64; gz = iz - chunkZ*16  # minY=-64
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
    pts = []
    for c in range(8):
        dx, dy, dz = c&1, (c>>1)&1, (c>>2)&1
        ax = chunkX*16 + (cx+dx)*4; ay = -64 + (cy+dy)*8; az = chunkZ*16 + (cz+dz)*4
        v = sim.eval_df_base(root, c, sIdx, ax, ay, az)
        if isinstance(v, tuple): return None
        pts.append(v)
    d00 = pts[0]+(pts[1]-pts[0])*fx; d10 = pts[2]+(pts[3]-pts[2])*fx
    d01 = pts[4]+(pts[5]-pts[4])*fx; d11 = pts[6]+(pts[7]-pts[6])*fx
    d0 = d00+(d10-d00)*fy; d1 = d01+(d11-d01)*fy
    return d0+(d1-d0)*fz

# 对比 eval_df 完整树结果与「手动 interp 重建」——先验证 eval_df 里的 interp 行为
# 找 eval_df 闭包里 DF_INTERP 节点位置
for i in range(N):
    if nodes[i]['type'] == sim.DF_INTERP:
        print(f'DF_INTERP 节点[{i}] a1={nodes[i]["a1"]}')
