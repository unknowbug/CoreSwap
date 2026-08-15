# check_schemeB_error.py —— 评估方案 B 语义误差：完整 finalDensity 网格插值 vs 正确（interp 插值 + 非线性）
# 用 sim 对比两种算法在若干 block 的值差，判断方案 B 是否可用（误差若 < 1e-4 则可接受）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

# 找到 DF_INTERP 节点和顶层 DF_MIN/DF_SQUEEZE 等（外层非线性）
# eval_df 顶层 = 完整树（val[N-1]）
# 方案A（正确）：eval_df(完整树) per block
# 方案B（错误但快）：interp 网格角点完整值 → 三线性插值
#   即：对 8 个 cell 角点算「完整 finalDensity」，然后插值——但完整值含非线性！

# 直接对比：正确 = eval_df 完整树 per block；方案B = 先算 8 角点完整树值再插值
def eval_full(sIdx, ix, iy, iz):
    r = sim.eval_df(N - 1, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

def schemeB(sIdx, ix, iy, iz):
    # 简化：假设已知 cell 结构，对 8 角点算完整树，三线性插值
    # 角点坐标 = 对齐 cell 网格（x/z 每 4、y 每 8）
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX * 16; gy = iy + 64; gz = iz - chunkZ * 16  # minY=-64
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    fx = (gx % 4) / 4.0; fy = (gy % 8) / 8.0; fz = (gz % 4) / 4.0
    pts = []
    for c in range(8):
        dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
        ax = chunkX * 16 + (cx + dx) * 4
        ay = -64 + (cy + dy) * 8
        az = chunkZ * 16 + (cz + dz) * 4
        v = eval_full(sIdx, ax, ay, az)
        if v is None: return None
        pts.append(v)
    d00 = pts[0] + (pts[1] - pts[0]) * fx; d10 = pts[2] + (pts[3] - pts[2]) * fx
    d01 = pts[4] + (pts[5] - pts[4]) * fx; d11 = pts[6] + (pts[7] - pts[6]) * fx
    d0 = d00 + (d10 - d00) * fy; d1 = d01 + (d11 - d01) * fy
    return d0 + (d1 - d0) * fz

# 采样点：e2e 域 + 大坐标域 + 边界附近（密度翻转敏感区）
pts = [(0,-64,0),(10,-60,0),(20,-50,2),(44,-49,4),(63,-49,2),
       (784,160,-408),(720,160,-432),(832,160,-416),(800,64,-384),
       (732,72,-408),(768,80,-420),(808,64,-384)]
maxd = 0; worst = None; over1e4 = 0; over1e3 = 0; total = 0
for (x,y,z) in pts:
    for sIdx in range(0, 33, 8):
        a = eval_full(sIdx, x, y, z)
        b = schemeB(sIdx, x, y, z)
        if a is None or b is None: continue
        d = abs(a - b); total += 1
        if d > maxd: maxd = d; worst = (x,y,z,sIdx,a,b)
        if d > 1e-4: over1e4 += 1
        if d > 1e-3: over1e3 += 1
print(f'方案B vs 正确: total={total} maxDiff={maxd:.3e} >1e-4:{over1e4} >1e-3:{over1e3}')
if worst:
    print(f'  worst: pos={worst[:3]} sIdx={worst[3]} correct={worst[4]:.6f} B={worst[5]:.6f}')
