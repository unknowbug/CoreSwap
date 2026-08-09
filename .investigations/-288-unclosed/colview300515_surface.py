# -*- coding: utf-8 -*-
"""完整地表列：vanilla 地表（grass_block/dirt 层）vs C++。判定 surface 规则差。
"""
import struct, json, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2name = {v: k for k, v in blocks.items()}

f = open(DATA + r"\vanilla_3005152118058349760_4_-1320400_-198064.blocks", "rb")
magic, seed = struct.unpack(">iq", f.read(12))
size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
van_cols = {}
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
    van_cols[(cx, cz)] = (col, biome)
f.close()

LINE = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=(.*)$")
mis = {}
with open(r"E:\PYTHON\CoreSwap\.investigations\-288-unclosed\m300515_run1.txt", encoding="utf-8") as fh:
    for line in fh:
        line = line.strip()
        if not line.startswith("MISMATCH"):
            continue
        m = LINE.match(line)
        if not m:
            continue
        cx, cz, x, y, z, gid, vid, biome = m.groups()
        mis.setdefault((int(x), int(z)), []).append((int(y), int(gid), int(vid), biome))

def short(bid):
    nm = id2name.get(bid, f"id{bid}")
    return nm.split(":")[1] if ":" in nm else nm

def colview(wx, wz, y0, y1):
    cx, cz = wx // 16, wz // 16
    if (cx, cz) not in van_cols:
        print(f"col({wx},{wz}) chunk({cx},{cz}) 不在参照范围")
        return
    col, biome = van_cols[(cx, cz)]
    lx, lz = wx % 16, wz % 16
    bm_col = biome[lz * 16 + lx]
    print(f"\n== col({wx},{wz}) chunk({cx},{cz}) biome={bm_col.split(':')[1] if ':' in bm_col else bm_col} ==")
    print(f"{'y':>5} {'vanilla':<14} {'C++差':<14} {'biome':<24}")
    for y in range(y0, y1 + 1):
        vn = short(col.get((lx, lz, y), 0))
        diff = ""
        bm = ""
        for dy, gid, vid, b in mis.get((wx, wz), []):
            if dy == y:
                diff, bm = short(gid), b
                break
        mark = "!" if diff and diff != vn else ("?" if diff else "")
        print(f"{y:>5} {vn:<14} {diff:<14} {bm:<24} {mark}")

# 3 个 dirt 差异列的地表形态（y=55..85 完整）
for wx, wz in [(-1320358, -198033), (-1320358, -198032), (-1320359, -198033)]:
    colview(wx, wz, 55, 85)
