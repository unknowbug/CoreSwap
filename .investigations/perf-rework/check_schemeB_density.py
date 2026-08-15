# check_schemeB_density.py —— 方案 B 误差 vs 网格密度：加密网格能否让完整树插值误差可接受
# 逐 cell 用不同密度的「完整 finalDensity 网格」插值，对比正确值
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

def eval_full(sIdx, ix, iy, iz):
    r = sim.eval_df(N - 1, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

# 网格密度参数：stepX/stepY/stepZ = 角点间隔
def schemeB_density(sIdx, ix, iy, iz, stepX=4, stepY=8, stepZ=4):
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX * 16; gy = iy + 64; gz = iz - chunkZ * 16
    # cell 基（对齐 step 网格）
    cx0 = (gx // stepX) * stepX; cy0 = (gy // stepY) * stepY; cz0 = (gz // stepZ) * stepZ
    fx = (gx - cx0) / stepX; fy = (gy - cy0) / stepY; fz = (gz - cz0) / stepZ
    pts = []
    for c in range(8):
        dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
        ax = chunkX * 16 + cx0 + dx * stepX
        ay = -64 + cy0 + dy * stepY
        az = chunkZ * 16 + cz0 + dz * stepZ
        v = eval_full(sIdx, ax, ay, az)
        if v is None: return None
        pts.append(v)
    d00 = pts[0] + (pts[1] - pts[0]) * fx; d10 = pts[2] + (pts[3] - pts[2]) * fx
    d01 = pts[4] + (pts[5] - pts[4]) * fx; d11 = pts[6] + (pts[7] - pts[6]) * fx
    d0 = d00 + (d10 - d00) * fy; d1 = d01 + (d11 - d01) * fy
    return d0 + (d1 - d0) * fz

# 采样点含边界附近（密度翻转敏感）
pts = [(0,-64,0),(10,-60,0),(20,-50,2),(44,-49,4),(63,-49,2),
       (784,160,-408),(720,160,-432),(832,160,-416),(800,64,-384),
       (732,72,-408),(768,80,-420),(808,64,-384),(44,-50,0),(50,-52,3)]
for (sx, sy, sz) in [(4,8,4), (2,4,2), (1,2,1), (2,8,4), (4,4,4)]:
    maxd = 0; worst = None; over1e4 = 0; over1e3 = 0; total = 0
    for (x,y,z) in pts:
        for sIdx in range(0, 33, 8):
            a = eval_full(sIdx, x, y, z)
            b = schemeB_density(sIdx, x, y, z, sx, sy, sz)
            if a is None or b is None: continue
            d = abs(a - b); total += 1
            if d > maxd: maxd = d; worst = (x,y,z,sIdx,a,b)
            if d > 1e-4: over1e4 += 1
            if d > 1e-3: over1e3 += 1
    print(f'step=({sx},{sy},{sz}) maxDiff={maxd:.3e} >1e-4:{over1e4}/{total} >1e-3:{over1e3}')
    if worst: print(f'    worst pos={worst[:3]} sIdx={worst[3]} correct={worst[4]:.6f} B={worst[5]:.6f}')
