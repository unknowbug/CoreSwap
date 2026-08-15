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

def eval_df_base_trace(root, corner, sIdx, ix, iy, iz):
    val = [0.0] * N
    for i in range(N):
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
        elif t == DF_INTERP: r = 0.0
        else: r = 0.0
        val[i] = r
    return val[root], val

root = g.interp_roots[0]
print(f'interp_0 delegate_root = {root}')
for ay in (-64, -56, -48):
    r, val = eval_df_base_trace(root, 0, 0, 0, ay, 0)
    print(f'--- 角点 y={ay}: delegate={r:.6f}')
    # 打印 ycg 相关链（node 2, 127 等）
    for i in (2, 127, 128):
        if i < N:
            print(f'  node[{i}] type={nodes[i]["type"]} val={val[i]:.6f}')
