# check_interp_final.py —— 最终语义验证：
# A) interp_N 等价（8 角点 delegate + 插值）——GPU 的实际形式
# B) 完整树 eval_df（正确值，GPU fill 已验证一致）
# 若 A 的 5 个 interp 组合 + 外层非线性 == B → interp_N 是唯一正确形式，方案 C（网格）不成立
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

def eval_base(interp_idx, corner, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    r = sim.eval_df_base(root, corner, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

def interp_N_equiv(interp_idx, sIdx, ix, iy, iz):
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX*16; gy = iy + 64; gz = iz - chunkZ*16
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
    pts = []
    for c in range(8):
        dx, dy, dz = c&1, (c>>1)&1, (c>>2)&1
        ax = chunkX*16 + (cx+dx)*4; ay = -64 + (cy+dy)*8; az = chunkZ*16 + (cz+dz)*4
        v = eval_base(interp_idx, c, sIdx, ax, ay, az)
        pts.append(v)
    d00=pts[0]+(pts[1]-pts[0])*fx; d10=pts[2]+(pts[3]-pts[2])*fx
    d01=pts[4]+(pts[5]-pts[4])*fx; d11=pts[6]+(pts[7]-pts[6])*fx
    d0=d00+(d10-d00)*fy; d1=d01+(d11-d01)*fy
    return d0+(d1-d0)*fz

# 用 A 重建完整树（DF_INTERP 节点 → interp_N_equiv），对比 B（eval_df 完整树）
def eval_df_with_interpN(sIdx, ix, iy, iz):
    val = [0.0] * N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == sim.DF_INTERP:
            r = interp_N_equiv(a1, sIdx, ix, iy, iz)
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

pts = [(0,-64,0),(10,-60,0),(20,-50,2),(44,-49,4),(63,-49,2),(5,-55,1),(30,-40,0),(44,-49,3)]
maxd = 0; worst = None; mism = 0
for (x,y,z) in pts:
    correct = sim.eval_df(N-1, 0, x, y, z)
    a = eval_df_with_interpN(0, x, y, z)
    d = abs(correct - a)
    if d > maxd: maxd = d; worst = (x,y,z,correct,a)
    if d > 1e-5: mism += 1
    print(f'  ({x},{y},{z}) correct={correct:.6f} interpN={a:.6f} diff={d:.2e}')
print(f'总: maxDiff={maxd:.3e} mism={mism}/{len(pts)}')
if worst: print(f'worst: {worst}')
