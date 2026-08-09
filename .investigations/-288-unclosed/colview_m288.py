# -*- coding: utf-8 -*-
"""叠加重建样本列：vanilla blocks（参照列）+ m288_natural_rows（C++ 差异点）。
判定海底边界性质：C++ 判水 vs vanilla 判石的分界位置差。
"""
import struct, json, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2 = {v: k.split(":")[1] for k, v in blocks.items()}
id2[0] = "air"

# ---- 读 vanilla blocks ----
f = open(DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks", "rb")
magic, seed = struct.unpack(">iq", f.read(12))
size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
van_cols = {}
for c in range(size * size):
    cx, cz = struct.unpack(">ii", f.read(8))
    d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
    pal = []
    for _ in range(256):
        blen = struct.unpack(">H", f.read(2))[0]
        pal.append(f.read(blen).decode("utf-8", errors="replace"))
    col = {}
    for i, v in enumerate(d):
        lx = i % 16; ly = i // 256; lz = (i // 16) % 16
        if (cx, cz) not in van_cols:
            pass
        col[miny + ly] = v
    van_cols[(cx, cz)] = (col, pal)
f.close()

# ---- 读 MISMATCH 行（run1 格式: got=37 vanilla=1，无名称）----
LINE = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=(.*)$")
mis = {}
with open(r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\m288_run1.txt", encoding="utf-8") as fh:
    for line in fh:
        line = line.strip()
        if not line or not line.startswith("MISMATCH"):
            continue
        m = LINE.match(line)
        if not m:
            print("PARSE FAIL:", line)
            continue
        cx, cz, x, y, z, gid, vid, biome = m.groups()
        gname = id2.get(int(gid), str(gid))
        vname = id2.get(int(vid), str(vid))
        mis.setdefault((int(x), int(z)), []).append((int(y), gname, vname, biome))

def colview(wx, wz, y0, y1, only_mis=False):
    cx, cz = wx // 16, wz // 16
    col, pal = van_cols[(cx, cz)]
    print(f"\n== col({wx},{wz}) chunk({cx},{cz})  [vanilla 列 + C++ MISMATCH 叠加] ==")
    print(f"{'y':>5} {'vanilla':<12} {'C++差':<14} {'biome':<20}")
    for y in range(y0, y1 + 1):
        vn = id2.get(col.get(y, 0), str(col.get(y, 0)))
        diff = ""
        biome = ""
        for dy, gn, vn_, bm in mis.get((wx, wz), []):
            if dy == y:
                diff, biome = gn, bm
                break
        if only_mis and not diff:
            continue
        mark = "!" if diff else ""
        print(f"{y:>5} {vn:<12} {diff:<14} {biome:<20} {mark}")

S = [
    ("海底边界样本", -264, -215, 20, 64),
    ("海底边界样本2", -263, -216, 20, 64),
    ("海底边界样本3", -241, -256, 40, 64),
    ("gravel 海底样本", -228, -212, -50, 56),
    ("surface beach 样本", -248, -215, 18, 60),
    ("surface beach 样本2", -247, -216, 18, 60),
    ("深层 gravel", -288, -211, -25, 0),
]
for label, wx, wz, y0, y1 in S:
    colview(wx, wz, y0, y1)
