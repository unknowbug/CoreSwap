# -*- coding: utf-8 -*-
"""dump 两个参照文件的 chunk pal 前 20 条 + 前 5 个块 id，确认格式一致"""
import struct, json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))

def read_chunk0(path):
    f = open(path, "rb")
    magic, seed = struct.unpack(">iq", f.read(12))
    size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
    cx, cz = struct.unpack(">ii", f.read(8))
    d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
    pal = []
    for _ in range(256):
        blen = struct.unpack(">H", f.read(2))[0]
        pal.append(f.read(blen).decode("utf-8", errors="replace"))
    f.close()
    return magic, seed, size, ox, oz, miny, h, cx, cz, d, pal

for p, label in [
    (DATA + r"\vanilla_3005152118058349760_4_-1320400_-198064.blocks", "新 seed 300515"),
    (DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks", "-288"),
]:
    magic, seed, size, ox, oz, miny, h, cx, cz, d, pal = read_chunk0(p)
    print(f"== {label}: magic={magic:#x} seed={seed} size={size} origin=({ox},{oz}) miny={miny} h={h} chunk=({cx},{cz}) ==")
    print("  pal 前 12:", pal[:12])
    print("  前 8 块 id:", d[:8], "->", [pal[v] if v < len(pal) and pal[v] else "?" for v in d[:8]])
    # 检查 blocks.json 映射
    print("  blocks.json 有 'minecraft:flower_forest'?", "minecraft:flower_forest" in blocks)
    print("  blocks.json 有 'minecraft:stone'?", "minecraft:stone" in blocks)
