# CPU 完整模拟：读 CpuBackend 的 splitCoord/perm + gen_df 节点数组 → eval_df → 对比 GPU/CPU 参照
import json, importlib.util, sys, struct, math, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)   # 完整收集（normal_meta/old_meta 等）

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
nodes = g.df_nodes
N = len(nodes)

# ---- 读 dump ----
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
splitCoord = struct.unpack('f' * (int(8672 * 1024)), open(base + r'\split_dump.bin', 'rb').read())
perm = struct.unpack('I' * (int(356352)), open(base + r'\perm_dump.bin', 'rb').read())
coords = [tuple(map(int, l.split())) for l in open(base + r'\coords_dump.txt')]
SPLIT_TOTAL = 8672

# ---- NORMAL 参数表（从 shader 提取：实例 → (n, octBase, splitBase, persistence, amplitude, ampOff, amps)）----
def get_normal_params():
    # 从 g.normal_meta + noise_instances 对齐
    meta_by_idx = {m['idx']: m for m in g.normal_meta}
    out = {}
    for idx, (kind, p) in enumerate(g.noise_instances):
        mm = meta_by_idx.get(idx)
        if mm:
            out[idx] = mm
    return out
NORMAL = get_normal_params()
OLD = {}
old_by_idx = {m['idx']: m for m in g.old_meta}
for idx, (kind, p) in enumerate(g.noise_instances):
    mm = old_by_idx.get(idx)
    if mm:
        OLD[idx] = mm

# ---- 噪声函数（GLSL 移植，float 语义）----
def mapPermD(octBase, v):
    return perm[octBase * 256 + (v & 255)]
GRADS = [[1,1,0],[-1,1,0],[1,-1,0],[-1,-1,0],[1,0,1],[-1,0,1],[1,0,-1],[-1,0,-1],
         [0,1,1],[0,-1,1],[0,1,-1],[0,-1,-1],[1,1,0],[0,-1,1],[-1,1,0],[0,-1,-1]]
def gradDotF(hash, x, y, z):
    gv = GRADS[hash & 15]
    return gv[0]*x + gv[1]*y + gv[2]*z
def perlinFadeF(v):
    return v*v*v*(v*(v*6.0-15.0)+10.0)
def lerpF(d, s, e):
    return s + d*(e-s)
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
    if not mm:
        return 0.0
    n = mm['n']; octBase = mm['octBase']; splitBase = mm['splitBase']
    persistence = mm['persistence']; amplitude = mm['amplitude']
    amps = mm['amps']
    d = 0.0; f = persistence
    for i in range(n):
        b = sIdx * SPLIT_TOTAL + splitBase + i*6
        ix = int(splitCoord[b]); iy = int(splitCoord[b+1]); iz = int(splitCoord[b+2])
        gx = splitCoord[b+3]; gy = splitCoord[b+4]; gz = splitCoord[b+5]
        ns = pn_sample3_f32(octBase+i, ix, iy, iz, gx, gy, gz)
        d += amps[i] * ns * f
        f /= 2.0
    d2 = 0.0; f = persistence
    for i in range(n):
        b = sIdx * SPLIT_TOTAL + splitBase + 6*n + i*6
        ix = int(splitCoord[b]); iy = int(splitCoord[b+1]); iz = int(splitCoord[b+2])
        gx = splitCoord[b+3]; gy = splitCoord[b+4]; gz = splitCoord[b+5]
        ns = pn_sample3_f32(octBase+n+i, ix, iy, iz, gx, gy, gz)
        d2 += amps[i] * ns * f
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
    if not mm:
        return 0.0
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

# ---- spline_eval（Python 移植）----
snodes = g.spline_ssbo_nodes
slocs = g.spline_ssbo_locs; sders = g.spline_ssbo_ders
svk = g.spline_ssbo_val_kind; svf = g.spline_ssbo_val_f; svn = g.spline_ssbo_val_node
scoords = g.spline_coords  # expr 字符串（slot 化 normal_noise）

def spline_coord_py(coordType, corner, sIdx, ix, iy, iz):
    # 精确实现 4 个 spline_coords case（从 g.spline_coords 提取 slot 后按 GLSL 公式求值）
    expr = scoords[coordType]
    # 提取所有 NOISE_SLOT_BASE[s] 引用 → slot 列表
    import re
    slots = [int(x) for x in re.findall(r'NOISE_SLOT_BASE\[(\d+)\]', expr)]
    def noise_val(slot):
        s = g.noise_slots[slot]
        return normal_noise(s['base'] + corner * s['stride'], sIdx)
    if len(slots) == 0:
        return 0.0
    n = noise_val(slots[0])
    # case 0/1/3：直接 ((normal_noise(...))) → n
    # case 2：-3*(-1/3 + abs(-2/3 + abs(n)))
    if 'abs(' in expr:
        return -3.0 * (-1.0/3.0 + abs(-2.0/3.0 + abs(n)))
    return n

def spline_find_range(x, locBegin, n):
    mn = 0; i = n
    while i > 0:
        j = i // 2; k = mn + j
        if x < slocs[locBegin + k]:
            i = j
        else:
            mn = k + 1; i -= j + 1
    return mn - 1

def spline_eval_py(rootNode, corner, sIdx, ix, iy, iz):
    TRACE = os.environ.get('DFC_SIM_SPLINE_TRACE') == '1'
    nodeStack = [0]*24; stageStack = [0]*24; iStack = [0]*24; outSlot = [0]*24
    v0Stack = [0.0]*24; v1Stack = [0.0]*24; coordStack = [0.0]*24
    sp = 0; nodeStack[0] = rootNode; stageStack[0] = 0; iStack[0] = 0; outSlot[0] = -1
    result = 0.0
    steps = 0
    while sp >= 0:
        steps += 1
        if steps > 100000:
            return ('LOOP', steps)
        node = nodeStack[sp]
        if node < 0 or node >= len(snodes):
            return ('OOB', steps)
        nd = snodes[node]
        coordType, n, locBegin, derBegin, valBegin = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
        if TRACE:
            print(f'[SPLINE] step{steps} sp={sp} node={node} stage={stageStack[sp]} n={n} outSlot={outSlot[sp]} coord={coordStack[sp]:.6f} v0={v0Stack[sp]:.6f} v1={v1Stack[sp]:.6f}', flush=True)
        if stageStack[sp] == 0:
            coord = spline_coord_py(coordType, corner, sIdx, ix, iy, iz)
            coordStack[sp] = coord
            i = spline_find_range(coord, locBegin, n)
            if i < 0:
                vk = svk[valBegin]
                if vk == 0:
                    result = svf[valBegin] + sders[derBegin] * (coord - slocs[locBegin])
                    ps = outSlot[sp]; sp -= 1
                    if ps >= 0:
                        if (ps & 1) == 0: v0Stack[ps >> 1] = result
                        else: v1Stack[ps >> 1] = result
                        if stageStack[ps >> 1] in (6, 7):
                            continue   # 父帧边界模式：保持 stage，由父帧完成路径处理
                    continue
                # D23：边界嵌套 value 递归——父帧 stage=6（左边界），压子帧
                # 注意：不覆盖 outSlot[sp]（保留本帧自己的返回地址），仅改 stage
                stageStack[sp] = 6; sp += 1
                nodeStack[sp] = svn[valBegin]; stageStack[sp] = 0; iStack[sp] = 0
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                outSlot[sp] = (sp - 1) * 2
                continue
            if i >= n - 1:
                vk = svk[valBegin + n - 1]
                if vk == 0:
                    result = svf[valBegin + n - 1] + sders[derBegin + n - 1] * (coord - slocs[locBegin + n - 1])
                    ps = outSlot[sp]; sp -= 1
                    if ps >= 0:
                        if (ps & 1) == 0: v0Stack[ps >> 1] = result
                        else: v1Stack[ps >> 1] = result
                        if stageStack[ps >> 1] in (6, 7):
                            continue   # 父帧边界模式：保持 stage，由父帧完成路径处理
                    continue
                # D23：边界嵌套 value 递归——父帧 stage=7（右边界），压子帧
                # 注意：不覆盖 outSlot[sp]（保留本帧自己的返回地址），仅改 stage
                stageStack[sp] = 7; sp += 1
                nodeStack[sp] = svn[valBegin + n - 1]; stageStack[sp] = 0; iStack[sp] = 0
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
                outSlot[sp] = (sp - 1) * 2 + 1
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
                # 父帧 stage 在压入子帧前已设为恢复点（1=等v1 / 2=Hermite / 6,7=边界），不覆盖
            continue
        if stageStack[sp] == 6:
            # D23：边界左外推——v0 子帧已回填，v0Stack[sp] + der[derBegin]*(coord-loc[locBegin])
            pnd = snodes[node]
            c2 = coordStack[sp]
            result = v0Stack[sp] + sders[pnd['derBegin']] * (c2 - slocs[pnd['locBegin']])
            ps = outSlot[sp]; sp -= 1
            if ps >= 0:
                if (ps & 1) == 0: v0Stack[ps >> 1] = result
                else: v1Stack[ps >> 1] = result
                # 父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），不覆盖
            continue
        if stageStack[sp] == 7:
            # D23：边界右外推——v1 子帧已回填，v1Stack[sp] + der[derBegin+n-1]*(coord-loc[locBegin+n-1])
            pnd = snodes[node]
            c2 = coordStack[sp]
            result = v1Stack[sp] + sders[pnd['derBegin'] + pnd['n'] - 1] * (c2 - slocs[pnd['locBegin'] + pnd['n'] - 1])
            ps = outSlot[sp]; sp -= 1
            if ps >= 0:
                if (ps & 1) == 0: v0Stack[ps >> 1] = result
                else: v1Stack[ps >> 1] = result
                # 父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），不覆盖
            continue
    return result, steps

# ---- eval_df（完整）----
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
            if isinstance(sv, tuple) and isinstance(sv[0], str):
                return sv
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
            r = d * abs(normal_noise(NOISE_SLOT_BASE(a2) + corner * NOISE_SLOT_STRIDE(a2), sIdx))
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        elif t == DF_INTERP: r = 0.0
        else: r = 0.0
        val[i] = r
    return val[root]

def NOISE_SLOT_BASE(slot):
    return g.noise_slots[slot]['base']
def NOISE_SLOT_STRIDE(slot):
    return g.noise_slots[slot]['stride']

def interp_N(interp_idx, sIdx, ix, iy, iz):
    minY = -64
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX * 16; gy = iy - minY; gz = iz - chunkZ * 16
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    root = g.interp_roots[interp_idx]
    pts = []
    for c in range(8):
        dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
        ax = chunkX * 16 + (cx + dx) * 4
        ay = minY + (cy + dy) * 8
        az = chunkZ * 16 + (cz + dz) * 4
        v = eval_df_base(root, c, sIdx, ax, ay, az)
        if isinstance(v, tuple):
            return v
        pts.append(v)
    fx = (gx % 4) / 4.0; fy = (gy % 8) / 8.0; fz = (gz % 4) / 4.0
    d00 = pts[0] + (pts[1] - pts[0]) * fx; d10 = pts[2] + (pts[3] - pts[2]) * fx
    d01 = pts[4] + (pts[5] - pts[4]) * fx; d11 = pts[6] + (pts[7] - pts[6]) * fx
    d0 = d00 + (d10 - d00) * fy; d1 = d01 + (d11 - d01) * fy
    return d0 + (d1 - d0) * fz

def interp_0(sIdx, ix, iy, iz):
    return interp_N(0, sIdx, ix, iy, iz)

def eval_df(rootPos, sIdx, ix, iy, iz):
    # 顶层闭包（root = N-1）
    val = [0.0] * N
    for i in range(N):
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_INTERP:
            r = interp_N(a1, sIdx, ix, iy, iz)
            if isinstance(r, tuple):
                return r
            val[i] = r
            continue
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (DF_NOISE, DF_SHIFTED_NOISE): r = normal_noise(NOISE_SLOT_BASE(a1), sIdx)
        elif t == DF_OLD_BLENDED: r = interp_noise(NOISE_SLOT_BASE(a1), sIdx)
        elif t == DF_SPLINE:
            sv = spline_eval_py(a1, 0, sIdx, ix, iy, iz)
            if isinstance(sv, tuple) and isinstance(sv[0], str):
                return sv
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
            r = d * abs(normal_noise(NOISE_SLOT_BASE(a2), sIdx))
        elif t in (DF_BLEND_DENSITY, DF_FLAT_CACHE): r = val[a1]
        elif t == DF_ADD: r = val[a1] + val[a2]
        elif t == DF_MUL: r = val[a1] * val[a2]
        elif t == DF_MIN: r = min(val[a1], val[a2])
        elif t == DF_MAX: r = max(val[a1], val[a2])
        else: r = 0.0
        val[i] = r
    return val[N-1]

# 对比几个点（y=-64, -62, -54）；gpu 值取 cmd-output/e2e-A5-20260815-135509.txt（D23 修复后）
gpu_vals = {0: 0.037482418, 128: 0.036994793, 640: 0.040212158, 896: 0.049567353}
print(f'NORMAL 实例数: {len(NORMAL)}, OLD 实例数: {len(OLD)}')
print(f'noise_instances 数: {len(g.noise_instances)}, slots: {len(g.noise_slots)}')
# 测 normal_noise（实例 32 = old？实例 0 = continentalness）
for idx in (0, 32, 152):
    print(f'  normal_noise({idx}): {normal_noise(idx, 0):.6f}')
for idx in (32,):
    print(f'  interp_noise({idx}): {interp_noise(idx, 0):.6f}')
# 角点 delegate
r0 = eval_df_base(g.interp_roots[0], 0, 0, 0, -64, 0)
print(f'角点 delegate (y=-64): {r0}')
for idx in (0, 128, 640, 896):
    x, y, z = coords[idx]
    r = eval_df(N - 1, idx, x, y, z)
    print(f'idx={idx} pos=({x},{y},{z}) sim={r} gpu={gpu_vals.get(idx)}')
