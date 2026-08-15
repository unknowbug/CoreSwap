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

base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
splitCoord = struct.unpack('f'*8388608, open(base + r'\split_dump.bin','rb').read())
perm = struct.unpack('I'*335872, open(base + r'\perm_dump.bin','rb').read())

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
def pn_section_f32(octBase, sIdx, splitOffset):
    b = sIdx * 8192 + splitOffset
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

# interp_noise(32) 逐 octave
octBase = 576; splitBase = 3456
n = 0.0; o = 1.0
for q in range(8):
    v = pn_section_f32(octBase+32+q, 0, splitBase + (32+q)*7)
    print(f'  q={q} octave={32+q} pn={v:.4f} /o={1/o:.1f} contrib={v/o:.4f}')
    n += v / o
    o /= 2.0
print(f'  n={n:.4f}, qq={(n/10+1)/2:.4f}')
qq = (n/10+1)/2
bl = qq >= 1.0; bl2 = qq <= 0.0
print(f'  bl={bl} bl2={bl2}')
if not bl:
    l = 0.0; o = 1.0
    for r in range(16):
        v = pn_section_f32(octBase+r, 0, splitBase + r*7)
        print(f'  r={r} octave={r} pn={v:.4f} /o={1/o:.1f} contrib={v/o:.4f}')
        l += v / o
        o /= 2.0
    print(f'  l={l:.4f} -> {(l/5+1)/2:.4f}')
