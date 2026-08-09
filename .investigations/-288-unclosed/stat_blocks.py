# -*- coding: utf-8 -*-
"""统计参照 blocks 块类型分布：检查是否含洞穴（air 夹层）、岩石替换（diorite/granite/andesite/tuff）、水/岩浆。
判定参照导出状态：SURFACE（无 FEATURE）vs FULL（含 carvers/岩石替换）。
"""
import struct, json, sys, collections
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
    return size, ox, oz, miny, h, chunks

def stat(path, label):
    size, ox, oz, miny, h, chunks = read_blocks(path)
    cnt = collections.Counter()
    cave_air_ys = []
    for (cx, cz), (d, pal) in chunks.items():
        col_air = [0] * h
        for i, v in enumerate(d):
            ly = i // 256
            name = pal[v] if v < len(pal) else id2.get(v, str(v))
            short = name.split(":")[1] if ":" in name else name
            cnt[short] += 1
            if short in ("air",):
                col_air[ly] += 1
        # 每列统计：y 深处（miny+40 以下）是否有 air（洞穴特征）
        for ly in range(h):
            if ly < 100 and col_air[ly] >= 3 and miny + ly < 0:
                cave_air_ys.append(miny + ly)
    total = sum(cnt.values())
    print(f"== {label} ==")
    print(f"  size={size} origin=({ox},{oz}) miny={miny} h={h} total_blocks={total}")
    print("  top15 块类型:")
    for name, n in cnt.most_common(15):
        print(f"    {name:<20} {n:>8} ({100.0*n/total:.2f}%)")
    # 深部洞穴 air 列数
    deep_air_cols = collections.Counter(cave_air_ys)
    print(f"  深部(y<0)洞穴 air 特征: 共 {len(cave_air_ys)} 列点，y 分布 top10:")
    for y, n in deep_air_cols.most_common(10):
        print(f"    y={y}: {n} 列")

stat(DATA + r"\vanilla_3005152118058349760_4_-1320400_-198064.blocks", "新 seed 300515 (用户目标区)")
stat(DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks", "-288 参照（对比）")
