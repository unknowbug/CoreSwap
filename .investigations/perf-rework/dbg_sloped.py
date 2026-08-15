import json, importlib.util, sys, struct, math
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dbg_full_sim as S

# 在 eval_df_base（角点）里打印 sloped_cheese 链（node 122 = range_choice input）
# 修改 S.eval_df_base 返回 val，然后打印关键节点
import types

orig = S.eval_df_base
def traced(root, corner, sIdx, ix, iy, iz):
    val = [0.0] * S.N
    nodes = S.nodes
    N = S.N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == 0: r = f0
        elif t == 1: r = float(iy)
        elif t in (2, 19): r = S.normal_noise(S.NOISE_SLOT_BASE(a1) + corner * S.NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == 3: r = S.interp_noise(S.NOISE_SLOT_BASE(a1) + corner * S.NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == 4:
            sv = S.spline_eval_py(a1, corner, sIdx, ix, iy, iz)
            r = sv[0] if not (isinstance(sv, tuple) and isinstance(sv[0], str)) else 0.0
        elif t == 18:
            tt = max(0.0, min(1.0, (float(iy) - f0) / (f1 - f0))) if f1 != f0 else 0.0
            r = f2 + tt * (f3 - f2)
        elif t == 10: r = abs(val[a1])
        elif t == 11: r = val[a1]**2
        elif t == 12: r = val[a1]**3
        elif t == 13: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == 14: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == 15:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c/2 - c*c*c/24
        elif t == 16: r = max(f0, min(f1, val[a1]))
        elif t == 17:
            inp = val[a1]; r = val[a2] if (f0 <= inp < f1) else val[a3]
        elif t in (20, 21): r = val[a1]
        elif t == 6: r = val[a1] + val[a2]
        elif t == 7: r = val[a1] * val[a2]
        elif t == 8: r = min(val[a1], val[a2])
        elif t == 9: r = max(val[a1], val[a2])
        elif t == 5: r = 0.0
        else: r = 0.0
        val[i] = r
    return val

# 角点 (0, -64, 0) 的 delegate 各节点值
root0 = g.interp_roots[0]
for ay in (-64, -56):
    val = traced(root0, 0, 0, 0, ay, 0)
    print(f'--- 角点 y={ay} ---')
    for i in (30, 37, 38, 39, 40, 41, 42):
        if i < len(val):
            n = S.nodes[i]
            info = f'node[{i}] type={n["type"]} a1={n["a1"]} a2={n["a2"]}'
            if n["type"] in (2, 19):
                info += f' slot={n["a1"]}'
            print(f'  {info} val={val[i]:.6f}')
