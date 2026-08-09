# -*- coding: utf-8 -*-
"""验证用户坐标 y=-4 附近（-1320400,-198049）的具体差异：是否 ore_dirt/ore_gravel 团。
用户坐标 x=-1320400 z=-198049 y=-4 → chunk(-82525,-12379) 局部(0,15)。
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

# 用户坐标列
wx, wz = -1320400, -198049
cx, cz = wx // 16, wz // 16
lx, lz = wx % 16, wz % 16
col, biome = van_cols[(cx, cz)]
print(f"用户坐标列 ({wx},{wz}) chunk({cx},{cz}) 局部({lx},{lz}) biome={biome[lz*16+lx]}")
print("y=-15..15 全形态:")
for y in range(-15, 16):
    vn = short(col.get((lx, lz, y), 0))
    diff = ""
    for dy, gid, vid, b in mis.get((wx, wz), []):
        if dy == y:
            diff = short(gid)
            break
    mark = "!" if diff and diff != vn else ""
    print(f"  y={y:>3} vanilla={vn:<14} C++={'':<14} {mark}" if not diff else f"  y={y:>3} vanilla={vn:<14} C++={diff:<14} {mark}")

# 该 chunk 全局：y=-10..0 的所有差异块统计
print("\n== chunk(-82525,-12379) y=-12..4 差异块 ==")
from collections import Counter
c = Counter()
for (x, z), lst in mis.items():
    if x // 16 != cx or z // 16 != cz:
        continue
    for y, gid, vid, b in lst:
        if -12 <= y <= 4:
            c[(short(gid), short(vid), y)] += 1
for (g, v, y), n in sorted(c.items(), key=lambda kv: -kv[1])[:25]:
    print(f"  y={y:>3} C++={g:<14} vanilla={v:<14} {n}")
