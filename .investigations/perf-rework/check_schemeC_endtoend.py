# check_schemeC_endtoend.py —— 方案 C 端到端 sim 验证：
# GPU(≈sim eval_df_base) 算 5 interp 内容树网格角点 → CPU 插值 + 外层非线性 → 最终密度
# 对比：完整树 eval_df（= GPU fill 正确值 = CPU finalDensity->sample）
# 若逐 block 一致 → 方案 C 语义正确，可实施
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

# ===== 1. GPU 侧：算 5 interp 内容树在 chunk 网格角点(1225)的值 =====
# 网格 GX=5, GY=49, GZ=5（x/z 每 4、y 每 8）；角点坐标 chunkX*16+gx*4, minY+gy*8, chunkZ*16+gz*4
def gpu_interp_grids(chunkX, chunkZ):
    GX, GY, GZ = 5, 49, 5
    grids = []
    for idx in range(5):
        root = sim.g.interp_roots[idx]
        grid = []
        for gy in range(GY):
            for gz in range(GZ):
                for gx in range(GX):
                    ax = chunkX*16 + gx*4
                    ay = -64 + gy*8
                    az = chunkZ*16 + gz*4
                    v = sim.eval_df_base(root, 0, 0, ax, ay, az)
                    grid.append(v if not (isinstance(v, tuple) and isinstance(v[0], str)) else 0.0)
        grids.append(grid)
    return grids

# ===== 2. CPU 侧：逐 block 插值 + 外层非线性（重建完整树求值，interp 节点用网格） =====
def eval_df_with_grids(sIdx, ix, iy, iz, grids):
    # 复制 eval_df 逻辑，但 DF_INTERP 节点改为查网格插值（不调 interp_N 的 delegate）
    val = [0.0] * N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == sim.DF_INTERP:
            # 用网格插值替代 interp_N（interp_idx = a1）
            idx = a1
            GX, GY, GZ = 5, 49, 5
            chunkX = ix // 16; chunkZ = iz // 16
            gx = ix - chunkX*16; gy = iy + 64; gz = iz - chunkZ*16
            cx = min(gx//4, 3); cy = min(gy//8, 47); cz = min(gz//4, 3)
            fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
            grid = grids[idx]
            def at(dx, dy, dz):
                return grid[((cy+dy)*GZ + (cz+dz))*GX + (cx+dx)]
            d000=at(0,0,0); d100=at(1,0,0); d010=at(0,1,0); d110=at(1,1,0)
            d001=at(0,0,1); d101=at(1,0,1); d011=at(0,1,1); d111=at(1,1,1)
            d00=d000+(d100-d000)*fx; d10=d010+(d110-d010)*fx
            d01=d001+(d101-d001)*fx; d11=d011+(d111-d011)*fx
            d0=d00+(d10-d00)*fy; d1=d01+(d11-d01)*fy
            r = d0+(d1-d0)*fz
            val[i] = r
            continue
        if t == sim.DF_CONSTANT: r = f0
        elif t == sim.DF_Y: r = float(iy)
        elif t in (sim.DF_NOISE, sim.DF_SHIFTED_NOISE): r = sim.normal_noise(sim.NOISE_SLOT_BASE(a1), sIdx)
        elif t == sim.DF_OLD_BLENDED: r = sim.interp_noise(sim.NOISE_SLOT_BASE(a1), sIdx)
        elif t == sim.DF_SPLINE:
            sv = sim.spline_eval_py(a1, 0, sIdx, ix, iy, iz)
            if isinstance(sv, tuple) and isinstance(sv[0], str): return None
            r = sv[0]
        elif t == sim.DF_Y_CLAMPED:
            tt = max(0.0, min(1.0, (float(iy)-f0)/(f1-f0))) if f1 != f0 else 0.0
            r = f2 + tt*(f3-f2)
        elif t == sim.DF_ABS: r = abs(val[a1])
        elif t == sim.DF_SQUARE: r = val[a1]**2
        elif t == sim.DF_CUBE: r = val[a1]**3
        elif t == sim.DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v*0.5
        elif t == sim.DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v*0.25
        elif t == sim.DF_SQUEEZE:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c/2 - c*c*c/24
        elif t == sim.DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == sim.DF_RANGE_CHOICE:
            inp = val[a1]; r = val[a2] if (f0 <= inp < f1) else val[a3]
        elif t == sim.DF_WEIRD:
            d = sim.ws_scale_py(int(f0), val[a1])
            r = d * abs(sim.normal_noise(sim.NOISE_SLOT_BASE(a2), sIdx))
        elif t in (sim.DF_BLEND_DENSITY, sim.DF_FLAT_CACHE): r = val[a1]
        elif t == sim.DF_ADD: r = val[a1] + val[a2]
        elif t == sim.DF_MUL: r = val[a1] * val[a2]
        elif t == sim.DF_MIN: r = min(val[a1], val[a2])
        elif t == sim.DF_MAX: r = max(val[a1], val[a2])
        else: r = 0.0
        val[i] = r
    return val[N-1]

# ===== 3. 对比 =====
chunkX, chunkZ = 0, 0
grids = gpu_interp_grids(chunkX, chunkZ)
print('interp 网格角点值范围: interp[0] min=%.4f max=%.4f' % (min(grids[0]), max(grids[0])))

pts = [(0,-64,0),(10,-60,0),(20,-50,2),(44,-49,4),(63,-49,2),(44,-49,3),(5,-55,1),(30,-40,0)]
maxd = 0; worst = None; mism = 0
for (x,y,z) in pts:
    correct = sim.eval_df(N-1, 0, x, y, z)   # 完整树（正确）
    schemeC = eval_df_with_grids(0, x, y, z, grids)  # 方案 C
    if correct is None or schemeC is None:
        print(f'  ({x},{y},{z}) None'); continue
    d = abs(correct - schemeC)
    if d > maxd: maxd = d; worst = (x,y,z,correct,schemeC)
    if d > 1e-5: mism += 1
    print(f'  ({x},{y},{z}) correct={correct:.6f} schemeC={schemeC:.6f} diff={d:.2e}')
print(f'总: maxDiff={maxd:.3e} mism(>1e-5)={mism}/{len(pts)}')
if worst: print(f'worst: {worst}')
