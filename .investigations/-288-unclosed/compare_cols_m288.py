# -*- coding: utf-8 -*-
"""对比 -288 区域样本列的 C++ vs vanilla 完整列形态（海底边界性质判定）。
数据: cpp_blocks_-288_-256.bin（C++ 输出） vs vanilla_-8248318472910187742_4_-288_-256.blocks（参照）
"""
import struct, json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2 = {v: k.split(":")[1] for k, v in blocks.items()}
id2[0] = "air"

def read_blocks(path):
    f = open(path, "rb")
    magic, seed = struct.unpack(">iq", f.read(12))
    size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
    chunks = {}
    for c in range(size * size):
        cx, cz = struct.unpack(">ii", f.read(8))
        d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
        pal = []
        for _ in range(256):
            blen = struct.unpack(">H", f.read(2))[0]
            pal.append(f.read(blen).decode("utf-8", errors="replace"))
        chunks[(cx, cz)] = (d, pal)
    f.close()
    return magic, seed, size, ox, oz, miny, h, chunks

def get_col(chunks, ox, oz, miny, h, wx, wz):
    cx, cz = wx // 16, wz // 16
    if (cx, cz) not in chunks:
        return None
    d, pal = chunks[(cx, cz)]
    col = {}
    for i, v in enumerate(d):
        lx = i % 16; ly = i // 256; lz = (i // 16) % 16
        if cx * 16 + lx == wx and cz * 16 + lz == wz:
            col[miny + ly] = v
    return col, pal

cpp = read_blocks(DATA + r"\cpp_blocks_-288_-256.bin")
van = read_blocks(DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks")

print(f"cpp: magic={cpp[0]:#x} seed={cpp[1]} size={cpp[2]} origin=({cpp[3]},{cpp[4]}) miny={cpp[5]} h={cpp[6]}")
print(f"van: magic={van[0]:#x} seed={van[1]} size={van[2]} origin=({van[3]},{van[4]}) miny={van[5]} h={van[6]}")

def col_name(c, pal, y):
    v = c.get(y, 0)
    return id2.get(v, str(v)) + (f"[{pal[v]}]" if False else "")

SAMPLES = [
    ("seabed 样本", -264, -215, 20, 64),
    ("seabed 样本2", -263, -216, 20, 64),
    ("gravel 样本", -228, -212, -50, 56),
    ("surface 样本", -248, -215, 18, 60),
    ("surface 样本2", -247, -216, 18, 60),
    ("深层 gravel", -288, -211, -25, 0),
]

for label, wx, wz, y0, y1 in SAMPLES:
    cc, cp = get_col(cpp[7], cpp[3], cpp[4], cpp[5], cpp[6], wx, wz)
    vc, vp = get_col(van[7], van[3], van[4], van[5], van[6], wx, wz)
    if cc is None or vc is None:
        print(f"== {label} ({wx},{wz}): MISSING chunk ==")
        continue
    print(f"\n== {label} ({wx},{wz}) ==")
    print(f"{'y':>5} {'C++':<12} {'vanilla':<12} {'=' if cc.get(y,0)==vc.get(y,0) else '!'}")
    for y in range(y0, y1 + 1):
        cn = id2.get(cc.get(y, 0), str(cc.get(y, 0)))
        vn = id2.get(vc.get(y, 0), str(vc.get(y, 0)))
        mark = "=" if cc.get(y, 0) == vc.get(y, 0) else ("!" if cn != vn else "?")
        if mark == "!" or y in (0, 15, 25, 35, 45, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64):
            print(f"{y:>5} {cn:<12} {vn:<12} {mark}")
