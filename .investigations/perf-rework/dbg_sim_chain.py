import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)

DF_CONSTANT, DF_Y, DF_NOISE, DF_OLD_BLENDED, DF_SPLINE, DF_INTERP, \
DF_ADD, DF_MUL, DF_MIN, DF_MAX, DF_ABS, DF_SQUARE, DF_CUBE, \
DF_HALF_NEG, DF_QUARTER_NEG, DF_SQUEEZE, DF_CLAMP, \
DF_RANGE_CHOICE, DF_Y_CLAMPED, DF_SHIFTED_NOISE, DF_BLEND_DENSITY, \
DF_FLAT_CACHE = range(22)

nodes = g.df_nodes
N = len(nodes)
read_fields = {6: ('a1','a2'), 7: ('a1','a2'), 8: ('a1','a2'), 9: ('a1','a2'),
               10: ('a1',), 11: ('a1',), 12: ('a1',), 13: ('a1',), 14: ('a1',), 15: ('a1',), 16: ('a1',),
               17: ('a1','a2','a3'), 20: ('a1',), 21: ('a1',)}

def eval_df_base(root, corner, sIdx, ix, iy, iz, maxsteps):
    val = [0.0] * N
    steps = 0
    for i in range(N):
        steps += 1
        if steps > maxsteps: return None, steps, f'loop@i={i}'
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE, DF_OLD_BLENDED): r = 0.0
        elif t == DF_SPLINE: r = 0.5
        elif t == DF_Y_CLAMPED:
            tt = max(0.0, min(1.0, (float(iy) - f0) / (f1 - f0))) if f1 != f0 else 0.0
            r = f2 + tt * (f3 - f2)
        elif t == DF_ABS: r = abs(val[a1])
        elif t == DF_SQUARE: r = val[a1]**2
        elif t == DF_CUBE: r = val[a1]**3
        elif t == DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == DF_SQUEEZE:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c/2 - c*c*c/24
        elif t == DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == DF_RANGE_CHOICE:
            inp = val[a1]; r = val[a2] if (f0 <= inp < f1) else val[a3]
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        elif t == DF_INTERP: r = 0.0   # eval_df_base 里无 interp 分支（GLSL 同）
        else: return None, steps, f'unknown type {t} @ {i}'
        val[i] = r
    if not (0 <= root < N): return None, steps, f'OOB root {root}'
    return val[root], steps, None

interp_root = g.interp_roots[0]
print(f'interp_0 delegate_root = {interp_root}')

def interp_0(sIdx, ix, iy, iz, maxsteps):
    # 8 角点 eval_df_base + 三线性（真实 cell 网格：minY=-64，cell 4×8×4）
    minY = -64
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX * 16; gy = iy - minY; gz = iz - chunkZ * 16
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    pts = {}
    for c in range(8):
        dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
        ax = chunkX * 16 + (cx + dx) * 4
        ay = minY + (cy + dy) * 8
        az = chunkZ * 16 + (cz + dz) * 4
        v, st, err = eval_df_base(interp_root, c, sIdx, ax, ay, az, maxsteps)
        if v is None: return None, st, f'interp corner {c}: {err}'
        pts[c] = v
    fx = (gx % 4) / 4.0; fy = (gy % 8) / 8.0; fz = (gz % 4) / 4.0
    def g(c): return pts[c]
    d00 = g(0) + (g(1) - g(0)) * fx; d10 = g(2) + (g(3) - g(2)) * fx
    d01 = g(4) + (g(5) - g(4)) * fx; d11 = g(6) + (g(7) - g(6)) * fx
    d0 = d00 + (d10 - d00) * fy; d1 = d01 + (d11 - d01) * fy
    return d0 + (d1 - d0) * fz, 40, None

def eval_df(root, sIdx, ix, iy, iz, maxsteps):
    val = [0.0] * N
    steps = 0
    for i in range(N):
        steps += 1
        if steps > maxsteps: return None, steps, f'loop@i={i}'
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_INTERP:
            v, st, err = interp_0(sIdx, ix, iy, iz, maxsteps - steps)
            if v is None: return None, steps + st, f'interp@{i}: {err}'
            val[i] = v
            continue
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE, DF_OLD_BLENDED): r = 0.0
        elif t == DF_SPLINE: r = 0.5
        elif t == DF_Y_CLAMPED:
            tt = max(0.0, min(1.0, (float(iy) - f0) / (f1 - f0))) if f1 != f0 else 0.0
            r = f2 + tt * (f3 - f2)
        elif t == DF_ABS: r = abs(val[a1])
        elif t == DF_SQUARE: r = val[a1]**2
        elif t == DF_CUBE: r = val[a1]**3
        elif t == DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == DF_SQUEEZE:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c/2 - c*c*c/24
        elif t == DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == DF_RANGE_CHOICE:
            inp = val[a1]; r = val[a2] if (f0 <= inp < f1) else val[a3]
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        else: return None, steps, f'unknown {t}@{i}'
        val[i] = r
    if not (0 <= root < N): return None, steps, f'OOB root {root}'
    return val[root], steps, None

MAXS = 200000
res, steps, err = eval_df(N - 1, 0, 0, -60, 0, MAXS)
print(f'eval_df 模拟: result={res} steps={steps} err={err}')
# 对比 GPU 输出：y=-64..-50（x=0, z=0）
for y in (-64, -62, -60, -58, -56, -54, -52, -50):
    r2, s2, e2 = eval_df(N - 1, 0, 0, y, 0, MAXS)
    print(f'  y={y}: sim={r2:.9f}')
# 角点 delegate 值（interp_0 的角点 y=-64 和 y=-56，x=0, z=0）
for ay in (-64, -56, -48):
    v, s, e = eval_df_base(interp_root, 0, 0, 0, ay, 0, MAXS)
    print(f'  corner y={ay}: delegate={v:.9f}')
# ycg 节点直接算（final_density 里 from_y=-64 to_y=-40）
for y in (-64, -62, -56, -54, -50):
    tt = max(0.0, min(1.0, (y - (-64)) / (40)))  # (-40 - -64) = 24
    print(f'  ycg(-64..-40) y={y}: t={tt:.4f}')
