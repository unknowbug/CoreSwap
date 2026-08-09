# -*- coding: utf-8 -*-
"""新 seed 样本列叠加：vanilla FULL blocks + C++ 差异点。
目标：判定 dirt/gravel 差异是 surface 规则差（P1）还是 FEATURE（ore_gravel/洞穴）。
样本：stone->dirt 密集列、deepslate->dirt（y=-3..7）、stone->gravel 浅层列。
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

def colview(wx, wz, y0, y1, only_mis=False):
    cx, cz = wx // 16, wz // 16
    if (cx, cz) not in van_cols:
        print(f"col({wx},{wz}) chunk({cx},{cz}) 不在参照范围")
        return
    col, biome = van_cols[(cx, cz)]
    lx, lz = wx % 16, wz % 16
    bm_col = biome[lz * 16 + lx]
    print(f"\n== col({wx},{wz}) chunk({cx},{cz}) biome={bm_col.split(':')[1] if ':' in bm_col else bm_col} ==")
    print(f"{'y':>5} {'vanilla':<14} {'C++差':<14} {'biome':<22}")
    for y in range(y0, y1 + 1):
        vn = short(col.get((lx, lz, y), 0))
        diff = ""
        bm = ""
        for dy, gid, vid, b in mis.get((wx, wz), []):
            if dy == y:
                diff, bm = short(gid), b
                break
        if only_mis and not diff:
            continue
        mark = "!" if diff and diff != vn else ("?" if diff else "")
        print(f"{y:>5} {vn:<14} {diff:<14} {bm:<22} {mark}")

# 样本：deepslate->dirt 密集列（y=-3..7）需要先找列
# 从 mis 找 stone->dirt / deepslate->dirt 最多的列
from collections import Counter
dirt_cols = Counter()
for (x, z), lst in mis.items():
    n = sum(1 for y, gid, vid, b in lst if short(gid) in ("stone", "deepslate") and short(vid) == "dirt")
    if n:
        dirt_cols[(x, z)] = n
print("stone/deepslate->dirt 列 top8:", dirt_cols.most_common(8))

gravel_cols = Counter()
for (x, z), lst in mis.items():
    n = sum(1 for y, gid, vid, b in lst if short(gid) in ("stone", "deepslate") and short(vid) == "gravel")
    if n:
        gravel_cols[(x, z)] = n
print("stone/deepslate->gravel 列 top8:", gravel_cols.most_common(8))

air_cols = Counter()
for (x, z), lst in mis.items():
    n = sum(1 for y, gid, vid, b in lst if short(gid) in ("stone", "deepslate") and short(vid) in ("air", "cave_air"))
    if n:
        air_cols[(x, z)] = n
print("stone/deepslate->air 列 top5:", air_cols.most_common(5))

# 展示 top dirt 列
for (x, z), n in dirt_cols.most_common(3):
    colview(x, z, -10, 76, only_mis=True)
