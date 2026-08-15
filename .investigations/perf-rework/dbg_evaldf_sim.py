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

def normal_noise(inst, corner, sIdx):
    return 0.0  # 简化：噪声=0（聚焦解释器控制流死循环）

def spline_eval(node, corner, sIdx, ix, iy, iz):
    # 简化：返回固定值（聚焦 eval_df 控制流）
    return 0.5

def eval_df_base(root, corner, sIdx, ix, iy, iz, max_steps):
    val = [0.0] * N
    steps = 0
    for i in range(N):
        steps += 1
        if steps > max_steps: return None, steps
        t = nodes[i]['type']
        a1, a2, a3 = nodes[i]['a1'], nodes[i]['a2'], nodes[i]['a3']
        f0, f1, f2, f3 = nodes[i]['f0'], nodes[i]['f1'], nodes[i]['f2'], nodes[i]['f3']
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE): r = normal_noise(a1, corner, sIdx)
        elif t == DF_OLD_BLENDED: r = normal_noise(a1, corner, sIdx)
        elif t == DF_SPLINE:
            if a2 == 1: r = spline_eval(a1, corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2)
            else: r = spline_eval(a1, corner, sIdx, ix, iy, iz)
        elif t == DF_Y_CLAMPED: r = 0.0
        elif t == DF_ABS: r = abs(val[a1])
        elif t == DF_SQUARE: r = val[a1] * val[a1]
        elif t == DF_CUBE: r = val[a1] ** 3
        elif t == DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == DF_SQUEEZE:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c / 2.0 - c * c * c / 24.0
        elif t == DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == DF_RANGE_CHOICE:
            inp = val[a1]; r = val[a2] if (inp >= f0 and inp < f1) else val[a3]
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        else:
            return ('UNKNOWN_TYPE', steps)
        val[i] = r
    if root < 0 or root >= N:
        return ('OOB_ROOT', steps)
    return val[root], steps

def eval_df(root, corner, sIdx, ix, iy, iz, max_steps):
    val = [0.0] * N
    steps = 0
    for i in range(N):
        steps += 1
        if steps > max_steps: return None, steps
        t = nodes[i]['type']
        a1, a2, a3 = nodes[i]['a1'], nodes[i]['a2'], nodes[i]['a3']
        f0, f1, f2, f3 = nodes[i]['f0'], nodes[i]['f1'], nodes[i]['f2'], nodes[i]['f3']
        if t == DF_INTERP:
            # interp_N → eval_df_base 8 次（简化：直接调用 eval_df_base）
            val[i] = eval_df_base(a1 * 0 + 0, corner, sIdx, ix, iy, iz, max_steps)[0] if False else 0.5
            continue
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE): r = normal_noise(a1, corner, sIdx)
        elif t == DF_OLD_BLENDED: r = normal_noise(a1, corner, sIdx)
        elif t == DF_SPLINE:
            if a2 == 1: r = spline_eval(a1, corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2)
            else: r = spline_eval(a1, corner, sIdx, ix, iy, iz)
        elif t == DF_Y_CLAMPED: r = 0.0
        elif t == DF_ABS: r = abs(val[a1])
        elif t == DF_SQUARE: r = val[a1] * val[a1]
        elif t == DF_CUBE: r = val[a1] ** 3
        elif t == DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == DF_SQUEEZE:
            v = val[a1]; c = max(-1.0, min(1.0, v)); r = c / 2.0 - c * c * c / 24.0
        elif t == DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == DF_RANGE_CHOICE:
            inp = val[a1]; r = val[a2] if (inp >= f0 and inp < f1) else val[a3]
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        else:
            return ('UNKNOWN_TYPE', steps)
        val[i] = r
    return val[root], steps

MAXS = 100000
res, steps = eval_df(N - 1, 0, 0, 0, -60, 0, MAXS)
print(f'eval_df(顶层): result={res} steps={steps} {"死循环!" if res is None else ""}')
# 找 interp 节点并测 eval_df_base
interps = [(i, n['a1']) for i, n in enumerate(nodes) if n['type'] == DF_INTERP]
print(f'interp 节点: {interps}')
# 顶层 interp 的 delegate root：需要从 _df_interp_node 拿，简化跳过
# 直接测 eval_df_base 所有可能 root（每个非 interp 节点做 root）
for c in range(8):
    for root in range(N):
        res, steps = eval_df_base(root, c, 0, 0, -60, 0, MAXS)
        if res is None or isinstance(res, str):
            print(f'eval_df_base root={root} corner={c} → {res} @ {steps} {"死循环!" if res is None else ""}')
            break
    else:
        continue
    break
else:
    print('eval_df_base 全部 root 有限步完成')
