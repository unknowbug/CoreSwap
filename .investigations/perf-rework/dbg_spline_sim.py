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

nodes = g.spline_ssbo_nodes
locs = g.spline_ssbo_locs
ders = g.spline_ssbo_ders
val_kind = g.spline_ssbo_val_kind
val_f = g.spline_ssbo_val_f
val_node = g.spline_ssbo_val_node
print(f'spline nodes: {len(nodes)}')

MAX_STEPS = 200000

def spline_eval_sim(root, coord_fn):
    """模拟 spline_eval 的 GLSL while 循环。coord_fn(coordType) 返回 coord 值。返回 (result, steps)。"""
    nodeStack = [0] * 24; stageStack = [0] * 24; iStack = [0] * 24; outSlot = [0] * 24
    v0Stack = [0.0] * 24; v1Stack = [0.0] * 24; coordStack = [0.0] * 24
    sp = 0
    nodeStack[0] = root; stageStack[0] = 0; iStack[0] = 0; outSlot[0] = -1
    result = 0.0
    steps = 0
    while sp >= 0:
        steps += 1
        if steps > MAX_STEPS:
            return None, steps
        node = nodeStack[sp]
        if node < 0 or node >= len(nodes):
            return ('OOB_NODE', steps)
        nd = nodes[node]
        base = node * 5
        coordType, n, locBegin, derBegin, valBegin = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
        if stageStack[sp] == 0:
            coord = coord_fn(coordType)
            coordStack[sp] = coord
            # spline_find_range 模拟
            i = find_range(coord, locBegin, n)
            if i < 0:
                vk = val_kind[valBegin + 0]
                v0 = val_f[valBegin + 0] if vk == 0 else 0.0
                result = v0 + ders[derBegin + 0] * (coord - locs[locBegin + 0])
                ps = outSlot[sp]; sp -= 1
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            if i >= n - 1:
                vk = val_kind[valBegin + n - 1]
                vn = val_f[valBegin + n - 1] if vk == 0 else 0.0
                result = vn + ders[derBegin + n - 1] * (coord - locs[locBegin + n - 1])
                ps = outSlot[sp]; sp -= 1
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            iStack[sp] = i
            vk0 = val_kind[valBegin + i]
            if vk0 == 0:
                v0Stack[sp] = val_f[valBegin + i]
                stageStack[sp] = 1
            else:
                stageStack[sp] = 1
                sp += 1
                if sp >= 24: return ('STACK_OVERFLOW', steps)
                nodeStack[sp] = val_node[valBegin + i]
                stageStack[sp] = 0; iStack[sp] = 0
                outSlot[sp] = (sp - 1) * 2
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                continue
        if stageStack[sp] == 1:
            i = iStack[sp]
            vk1 = val_kind[valBegin + i + 1]
            if vk1 == 0:
                v1Stack[sp] = val_f[valBegin + i + 1]
                stageStack[sp] = 2
            else:
                stageStack[sp] = 2
                sp += 1
                if sp >= 24: return ('STACK_OVERFLOW', steps)
                nodeStack[sp] = val_node[valBegin + i + 1]
                stageStack[sp] = 0; iStack[sp] = 0
                outSlot[sp] = (sp - 1) * 2 + 1
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                continue
        if stageStack[sp] == 2:
            i = iStack[sp]
            coord = coordStack[sp]
            nv = v0Stack[sp]; ov = v1Stack[sp]
            span = locs[locBegin + i + 1] - locs[locBegin + i]
            if span == 0: return ('ZERO_SPAN', steps)
            result = nv + (coord - locs[locBegin + i]) / span * (ov - nv) + (coord - locs[locBegin + i]) / span * (1.0 - (coord - locs[locBegin + i]) / span) * (ders[derBegin + i] * span - (ov - nv) + (coord - locs[locBegin + i]) / span * (-ders[derBegin + i + 1] * span + (ov - nv) - (ders[derBegin + i] * span - (ov - nv))))
            ps = outSlot[sp]; sp -= 1
            if ps >= 0:
                if (ps & 1) == 0: v0Stack[ps >> 1] = result
                else: v1Stack[ps >> 1] = result
                stageStack[ps >> 1] = 2
            continue
        return ('BAD_STAGE', steps)
    return result, steps

def find_range(x, locBegin, n):
    lo = 0; hi = n
    while hi > 0:
        half = hi // 2
        mid = lo + half
        if x < locs[locBegin + mid]:
            hi = half
        else:
            lo = mid + 1
            hi -= half + 1
    return lo - 1

# 用多种固定 coord 值测试所有 spline 根节点
bad = []
import random
random.seed(42)
vals = [-10.0, -1.0, 0.0, 0.5, 1.0, 3.0, 10.0, float('nan'), float('inf'), float('-inf')]
vals += [random.uniform(-200, 200) for _ in range(5000)]
for root in range(len(nodes)):
    for cv in vals:
        res, steps = spline_eval_sim(root, lambda ct, cv=cv: cv)
        if res is None or isinstance(res, str):
            bad.append((root, cv, res, steps))
            break
    if bad:
        break
print(f'死循环/异常: {len(bad)}')
for b in bad[:15]:
    print(' ', b)
if not bad:
    print('OK: 随机 coord 值下所有 spline 节点有限步完成')
