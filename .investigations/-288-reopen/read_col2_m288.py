# -*- coding: utf-8 -*-
# 读任意参照 .blocks 文件指定列（参数化版 read_col2）
# 用法: python read_col2_m288.py <blocks文件> <wx> <wz> [y0] [y1]
import struct, json, sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
blocks = json.load(open(r"E:\PYTHON\CoreSwap\versions\1.20.1\data\blocks.json", encoding="utf-8"))
id2 = {v: k.split(":")[1] for k, v in blocks.items()}

path = sys.argv[1]
wx, wz = int(sys.argv[2]), int(sys.argv[3])
y0 = int(sys.argv[4]) if len(sys.argv) > 4 else 0
y1 = int(sys.argv[5]) if len(sys.argv) > 5 else 127

f = open(path, "rb")
magic, seed = struct.unpack(">iq", f.read(12))
size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
print(f"# file={path} seed={seed} size={size} origin=({ox},{oz}) minY={miny} height={h}")

col = {}
for c in range(size * size):
    cx, cz = struct.unpack(">ii", f.read(8))
    d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
    for i, v in enumerate(d):
        lx = i % 16; ly = i // 256; lz = (i // 16) % 16
        if cx * 16 + lx == wx and cz * 16 + lz == wz:
            col[miny + ly] = v
    for _ in range(256):
        blen = struct.unpack(">H", f.read(2))[0]; f.read(blen)
f.close()

for y in range(y0, y1 + 1):
    v = col.get(y, 0)
    print("y=%d %d %s" % (y, v, id2.get(v, "?")))
