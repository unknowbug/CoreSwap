# 逐角点评估 when_out 子树节点：ay=-64 vs ay=-56（同 x,z 列），找 y 相关差异源
import json, importlib.util, sys, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)

nodes = g.df_nodes
N = len(nodes)
print(f'N={N}, interp_roots={g.interp_roots}')

# 顶层闭包节点
print('--- 顶层闭包（root=N-1 可达）---')
top = set()
def scan_t(i, d=0):
    if i < 0 or i in top: return
    top.add(i)
    n = nodes[i]
    for f in ('a1','a2','a3'):
        if n[f] >= 0: scan_t(n[f], d+1)
scan_t(N-1)
for i in sorted(top):
    n = nodes[i]
    print(f'  node[{i}] t={n["type"]} a1={n["a1"]} a2={n["a2"]} a3={n["a3"]} f0={n["f0"]} f1={n["f1"]}')

# 读 dump（噪声 split 数据）
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
splitCoord = struct.unpack('f' * (int(8672 * 1024)), open(base + r'\split_dump.bin', 'rb').read())
perm = struct.unpack('I' * (int(356352)), open(base + r'\perm_dump.bin', 'rb').read())
SPLIT_TOTAL = 8672

# ---- 移植 dbg_full_sim 的噪声/spline 基础设施（精简：只算 NOISE/OLD/SPLINE/YCLAMP）----
meta_by_idx = {m['idx']: m for m in g.normal_meta}
NORMAL = {}
for idx, (kind, p) in enumerate(g.noise_instances):
    if idx in meta_by_idx: NORMAL[idx] = meta_by_idx[idx]
OLD = {}
old_by_idx = {m['idx']: m for m in g.old_meta}
for idx, (kind, p) in enumerate(g.noise_instances):
    if idx in old_by_idx: OLD[idx] = old_by_idx[idx]

GRADS = [[1,1,0],[-1,1,0],[1,-1,0],[-1,-1,0],[1,0,1],[-1,0,1],[1,0,-1],[-1,0,-1],
         [0,1,1],[0,-1,1],[0,1,-1],[0,-1,-1],[1,1,0],[0,-1,1],[-1,1,0],[0,-1,-1]]
def mapPermD(octBase, v): return perm[octBase * 256 + (v & 255)]
def gradDotF(h, x, y, z):
    gv = GRADS[h & 15]; return gv[0]*x + gv[1]*y + gv[2]*z
def perlinFadeF(v): return v*v*v*(v*(v*6.0-15.0)+10.0)
def lerpF(d, s, e): return s + d*(e-s)
def pn_sample3_f32(octBase, sx, sy, sz, lx, ly, lz):
    i = mapPermD(octBase, sx); j = mapPermD(octBase, sx+1)
    k = mapPermD(octBase, i+sy); l = mapPermD(octBase, i+sy+1)
    mm = mapPermD(octBase, j+sy); nn = mapPermD(octBase, j+sy+1)
    d = gradDotF(mapPermD(octBase, k+sz), lx, ly, lz)
    e = gradDotF(mapPermD(octBase, mm+sz), lx-1, ly, lz)
    f = gradDotF(mapPermD(octBase, l+sz), lx, ly-1, lz)
    gg = gradDotF(mapPermD(octBase, nn+sz), lx-1, ly-1, lz)
    h = gradDotF(mapPermD(octBase, k+sz+1), lx, ly, lz-1)
    o = gradDotF(mapPermD(octBase, mm+sz+1), lx-1, ly, lz-1)
    p = gradDotF(mapPermD(octBase, l+sz+1), lx, ly-1, lz-1)
    q = gradDotF(mapPermD(octBase, nn+sz+1), lx-1, ly-1, lz-1)
    r = perlinFadeF(lx); s = perlinFadeF(ly); t = perlinFadeF(lz)
    x0 = lerpF(r, d, e); x1 = lerpF(r, f, gg)
    x2 = lerpF(r, h, o); x3 = lerpF(r, p, q)
    y0 = lerpF(s, x0, x1); y1 = lerpF(s, x2, x3)
    return lerpF(t, y0, y1)
def normal_noise(noiseIdx, sIdx):
    mm = NORMAL.get(noiseIdx)
    if not mm: return 0.0
    n = mm['n']; octBase = mm['octBase']; splitBase = mm['splitBase']
    persistence = mm['persistence']; amplitude = mm['amplitude']; amps = mm['amps']
    d = 0.0; f = persistence
    for i in range(n):
        b = sIdx * SPLIT_TOTAL + splitBase + i*6
        ix = int(splitCoord[b]); iy = int(splitCoord[b+1]); iz = int(splitCoord[b+2])
        gx = splitCoord[b+3]; gy = splitCoord[b+4]; gz = splitCoord[b+5]
        d += amps[i] * pn_sample3_f32(octBase+i, ix, iy, iz, gx, gy, gz) * f
        f /= 2.0
    d2 = 0.0; f = persistence
    for i in range(n):
        b = sIdx * SPLIT_TOTAL + splitBase + 6*n + i*6
        ix = int(splitCoord[b]); iy = int(splitCoord[b+1]); iz = int(splitCoord[b+2])
        gx = splitCoord[b+3]; gy = splitCoord[b+4]; gz = splitCoord[b+5]
        d2 += amps[i] * pn_sample3_f32(octBase+n+i, ix, iy, iz, gx, gy, gz) * f
        f /= 2.0
    return (d + d2) * amplitude
def pn_section_f32(octBase, sIdx, splitOffset):
    b = sIdx * SPLIT_TOTAL + splitOffset
    sx = int(splitCoord[b]); sy = int(splitCoord[b+1]); sz = int(splitCoord[b+2])
    lx = splitCoord[b+3]; ly = splitCoord[b+4]; lz = splitCoord[b+5]
    fadeY = splitCoord[b+6]
    i = mapPermD(octBase, sx); j = mapPermD(octBase, sx+1)
    k = mapPermD(octBase, i+sy); l = mapPermD(octBase, i+sy+1)
    mm = mapPermD(octBase, j+sy); nn = mapPermD(octBase, j+sy+1)
    d = gradDotF(mapPermD(octBase, k+sz), lx, ly, lz)
    e = gradDotF(mapPermD(octBase, mm+sz), lx-1, ly, lz)
    f = gradDotF(mapPermD(octBase, l+sz), lx, ly-1, lz)
    gg = gradDotF(mapPermD(octBase, nn+sz), lx-1, ly-1, lz)
    h = gradDotF(mapPermD(octBase, k+sz+1), lx, ly, lz-1)
    o = gradDotF(mapPermD(octBase, mm+sz+1), lx-1, ly, lz-1)
    p = gradDotF(mapPermD(octBase, l+sz+1), lx, ly-1, lz-1)
    q = gradDotF(mapPermD(octBase, nn+sz+1), lx-1, ly-1, lz-1)
    r = perlinFadeF(lx); s = perlinFadeF(fadeY); t = perlinFadeF(lz)
    x0 = lerpF(r, d, e); x1 = lerpF(r, f, gg)
    x2 = lerpF(r, h, o); x3 = lerpF(r, p, q)
    y0 = lerpF(s, x0, x1); y1 = lerpF(s, x2, x3)
    return lerpF(t, y0, y1)
def interp_noise(idx, sIdx):
    mm = OLD.get(idx)
    if not mm: return 0.0
    octBase = mm['octBase']; splitBase = mm['splitBase']
    n = 0.0; o = 1.0
    for q in range(8):
        n += pn_section_f32(octBase+32+q, sIdx, splitBase + (32+q)*7) / o
        o /= 2.0
    qq = (n / 10.0 + 1.0) / 2.0
    bl = qq >= 1.0; bl2 = qq <= 0.0
    l = 0.0; mm2 = 0.0; o = 1.0
    for r in range(16):
        if not bl:
            l += pn_section_f32(octBase+r, sIdx, splitBase + r*7) / o
        if not bl2:
            mm2 += pn_section_f32(octBase+16+r, sIdx, splitBase + (16+r)*7) / o
        o /= 2.0
    w = max(0.0, min(1.0, qq))
    return (l / 512.0 + w * (mm2 / 512.0 - l / 512.0)) / 128.0

# spline
snodes = g.spline_ssbo_nodes; slocs = g.spline_ssbo_locs; sders = g.spline_ssbo_ders
svk = g.spline_ssbo_val_kind; svf = g.spline_ssbo_val_f; svn = g.spline_ssbo_val_node
scoords = g.spline_coords
import re
def spline_coord_py(coordType, corner, sIdx, ix, iy, iz):
    expr = scoords[coordType]
    slots = [int(x) for x in re.findall(r'NOISE_SLOT_BASE\[(\d+)\]', expr)]
    def noise_val(slot):
        s = g.noise_slots[slot]
        return normal_noise(s['base'] + corner * s['stride'], sIdx)
    if len(slots) == 0: return 0.0
    n = noise_val(slots[0])
    if 'abs(' in expr:
        return -3.0 * (-1.0/3.0 + abs(-2.0/3.0 + abs(n)))
    return n
def spline_find_range(x, locBegin, n):
    mn = 0; i = n
    while i > 0:
        j = i // 2; k = mn + j
        if x < slocs[locBegin + k]: i = j
        else: mn = k + 1; i -= j + 1
    return mn - 1
def spline_eval_py(rootNode, corner, sIdx, ix, iy, iz):
    nodeStack = [0]*24; stageStack = [0]*24; iStack = [0]*24; outSlot = [0]*24
    v0Stack = [0.0]*24; v1Stack = [0.0]*24; coordStack = [0.0]*24
    sp = 0; nodeStack[0] = rootNode; stageStack[0] = 0; iStack[0] = 0; outSlot[0] = -1
    result = 0.0; steps = 0
    while sp >= 0:
        steps += 1
        if steps > 100000: return ('LOOP', steps)
        node = nodeStack[sp]
        if node < 0 or node >= len(snodes): return ('OOB', steps)
        nd = snodes[node]
        coordType, n, locBegin, derBegin, valBegin = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
        if stageStack[sp] == 0:
            coord = spline_coord_py(coordType, corner, sIdx, ix, iy, iz)
            coordStack[sp] = coord
            i = spline_find_range(coord, locBegin, n)
            if i < 0:
                vk = svk[valBegin]; v0 = svf[valBegin] if vk == 0 else 0.0
                result = v0 + sders[derBegin] * (coord - slocs[locBegin])
                ps = outSlot[sp]; sp -= 1
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            if i >= n - 1:
                vk = svk[valBegin + n - 1]; vn = svf[valBegin + n - 1] if vk == 0 else 0.0
                result = vn + sders[derBegin + n - 1] * (coord - slocs[locBegin + n - 1])
                ps = outSlot[sp]; sp -= 1
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            iStack[sp] = i
            vk0 = svk[valBegin + i]
            if vk0 == 0:
                v0Stack[sp] = svf[valBegin + i]; stageStack[sp] = 1
            else:
                stageStack[sp] = 1; sp += 1
                nodeStack[sp] = svn[valBegin + i]; stageStack[sp] = 0; iStack[sp] = 0
                outSlot[sp] = (sp - 1) * 2
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                continue
        if stageStack[sp] == 1:
            i = iStack[sp]
            vk1 = svk[valBegin + i + 1]
            if vk1 == 0:
                v1Stack[sp] = svf[valBegin + i + 1]; stageStack[sp] = 2
            else:
                stageStack[sp] = 2; sp += 1
                nodeStack[sp] = svn[valBegin + i + 1]; stageStack[sp] = 0; iStack[sp] = 0
                outSlot[sp] = (sp - 1) * 2 + 1
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                continue
        if stageStack[sp] == 2:
            i = iStack[sp]
            coord = coordStack[sp]; nv = v0Stack[sp]; ov = v1Stack[sp]
            span = slocs[locBegin + i + 1] - slocs[locBegin + i]
            kd = (coord - slocs[locBegin + i]) / span
            p = sders[derBegin + i] * span - (ov - nv)
            q = -sders[derBegin + i + 1] * span + (ov - nv)
            result = (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p))
            ps = outSlot[sp]; sp -= 1
            if ps >= 0:
                if (ps & 1) == 0: v0Stack[ps >> 1] = result
                else: v1Stack[ps >> 1] = result
                stageStack[ps >> 1] = 2
            continue
    return result, steps

def NOISE_SLOT_BASE(s): return g.noise_slots[s]['base']
def NOISE_SLOT_STRIDE(s): return g.noise_slots[s]['stride']

DF_CONSTANT, DF_Y, DF_NOISE, DF_OLD_BLENDED, DF_SPLINE, DF_INTERP, \
DF_ADD, DF_MUL, DF_MIN, DF_MAX, DF_ABS, DF_SQUARE, DF_CUBE, \
DF_HALF_NEG, DF_QUARTER_NEG, DF_SQUEEZE, DF_CLAMP, \
DF_RANGE_CHOICE, DF_Y_CLAMPED, DF_SHIFTED_NOISE, DF_BLEND_DENSITY, \
DF_FLAT_CACHE, DF_WEIRD = range(23)

def ws_scale_py(kind, v):
    if kind == 1:
        if v < -0.75: return 0.5
        if v < -0.5: return 0.75
        if v < 0.5: return 1.0
        return 2.0 if v < 0.75 else 3.0
    if v < -0.5: return 0.75
    if v < 0.0: return 1.0
    return 1.5 if v < 0.5 else 2.0

def eval_df_base(root, corner, sIdx, ix, iy, iz):
    val = [0.0] * N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE): r = normal_noise(NOISE_SLOT_BASE(a1) + corner * NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == DF_OLD_BLENDED: r = interp_noise(NOISE_SLOT_BASE(a1) + corner * NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == DF_SPLINE:
            sv = spline_eval_py(a1, corner, sIdx, ix, iy, iz)
            if isinstance(sv, tuple) and isinstance(sv[0], str): return sv
            r = sv[0]
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
        elif t == DF_WEIRD:
            d = ws_scale_py(int(f0), val[a1])
            r = d * abs(normal_noise(NOISE_SLOT_BASE(a2) + corner * NOISE_SLOT_STRIDE(a2), 0))
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        elif t == DF_INTERP: r = 0.0
        else: r = 0.0
        val[i] = r
    return val[root]

# 关注的节点
watch = [2, 3, 4, 5, 6, 8, 10, 12, 13, 14, 15, 40, 41, 42, 45, 47, 48, 66, 67, 70, 71, 72, 73, 74, 75, 76, 77, 79, 80, 81, 82, 83, 84,
         90, 91, 92, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
         116, 117, 118, 119, 120, 121, 122, 125, 126, 127, 128, 129, 133, 134, 135, 138, 139, 140, 141, 142, 143, 145, 146, 149, 150]
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQ',12:'CUBE',13:'HNEG',14:'QNEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE',18:'YCLAMP',19:'SNOISE',20:'BLEND',21:'FLAT'}
for corner, (ax, ay, az) in [(0, (0, -64, 0)), (2, (0, -56, 0)), (4, (0, -64, 4)), (6, (0, -56, 4))]:
    print(f'\n===== corner={corner} pos=({ax},{ay},{az}) =====')
    r = eval_df_base(N - 1, corner, 0, ax, ay, az)
    print(f'root result = {r}')
    # 重算一次拿全部值
    val = [0.0] * N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(ay)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE): r = normal_noise(NOISE_SLOT_BASE(a1) + corner * NOISE_SLOT_STRIDE(a1), 0)
        elif t == DF_OLD_BLENDED: r = interp_noise(NOISE_SLOT_BASE(a1) + corner * NOISE_SLOT_STRIDE(a1), 0)
        elif t == DF_SPLINE:
            sv = spline_eval_py(a1, corner, 0, ax, ay, az)
            r = sv[0] if not (isinstance(sv, tuple) and isinstance(sv[0], str)) else 0.0
        elif t == DF_Y_CLAMPED:
            tt = max(0.0, min(1.0, (float(ay) - f0) / (f1 - f0))) if f1 != f0 else 0.0
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
        elif t == DF_WEIRD:
            d = ws_scale_py(int(f0), val[a1])
            r = d * abs(normal_noise(NOISE_SLOT_BASE(a2) + corner * NOISE_SLOT_STRIDE(a2), 0))
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        elif t == DF_INTERP: r = 0.0
        else: r = 0.0
        val[i] = r
    for i in watch:
        n = nodes[i]
        print(f'  node[{i:3d}] {names.get(n["type"],"?"):7s} = {val[i]:.9f}')
    # interp 各根 + 分支
    def rootv(root, corner, ax, ay, az):
        r = eval_df_base(root, corner, 0, ax, ay, az)
        return r[0] if (isinstance(r, tuple) and isinstance(r[0], str)) else r
    i0 = rootv(129, corner, ax, ay, az)
    i1 = rootv(135, corner, ax, ay, az)
    i2 = rootv(143, corner, ax, ay, az)
    i3 = rootv(146, corner, ax, ay, az)
    i4 = rootv(150, corner, ax, ay, az)
    b1 = 0.64 * (lambda c: c/2 - c*c*c/24)(max(-1.0, min(1.0, i0)))
    b2 = i2 + 1.5 * max(abs(i3), abs(i4))
    print(f'  interp0={i0:.9f} interp1={i1:.9f} interp2={i2:.9f} interp3={i3:.9f} interp4={i4:.9f}')
    print(f'  branch1(squeeze*0.64)={b1:.9f} branch2(offset+1.5*max|e|)={b2:.9f} min={min(b1,b2):.9f}')
