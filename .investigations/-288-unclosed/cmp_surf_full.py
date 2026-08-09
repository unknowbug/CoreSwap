# -*- coding: utf-8 -*-
"""对比同一列 (-244,-256) 在 SURFACE 参照 vs FULL 参照 vs NOISE-BLK 的块形态。
判定：岛（y=58-61 stone）在哪个阶段出现——SURFACE（无 FEATURE）就 stone = aquifer 产物；只有 FULL 才有 = FEATURE。
"""
import struct, json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2name = {v: k for k, v in blocks.items()}

def short(bid):
    nm = id2name.get(bid, f"id{bid}")
    return nm.split(":")[1] if ":" in nm else nm

def read_cols(path):
    f = open(path, "rb")
    magic, seed = struct.unpack(">iq", f.read(12))
    size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
    cols = {}
    for c in range(size * size):
        cx, cz = struct.unpack(">ii", f.read(8))
        d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
        biome = []
        for _ in range(256):
            blen = struct.unpack(">H", f.read(2))[0]
            biome.append(f.read(blen).decode("utf-8", errors="replace"))
        col = {}
        for i, v in enumerate(d):
            lx = i % 16; ly = i // 256; lz = (i // 16) % 16
            col[(lx, lz, miny + ly)] = v
        cols[(cx, cz)] = (col, biome)
    f.close()
    return size, ox, oz, miny, h, cols

def show(path, label, wx, wz, y0, y1):
    size, ox, oz, miny, h, cols = read_cols(path)
    cx, cz = wx // 16, wz // 16
    if (cx, cz) not in cols:
        print(f"== {label}: chunk({cx},{cz}) 不在参照 ==")
        return
    col, biome = cols[(cx, cz)]
    lx, lz = wx % 16, wz % 16
    print(f"== {label}: col({wx},{wz}) chunk({cx},{cz}) ==")
    for y in range(y0, y1 + 1):
        print(f"  y={y:>3} {short(col.get((lx, lz, y), 0))}")

SURF = DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks"          # 新导出 SURFACE
FULL = DATA + r"\vanilla_-8248318472910187742_4_-288_-256_FULL.blocks"     # 备份 FULL

for path, label in [(SURF, "SURFACE"), (FULL, "FULL")]:
    show(path, label, -244, -256, 38, 76)
    print()
