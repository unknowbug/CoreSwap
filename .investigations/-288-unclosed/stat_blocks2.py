# -*- coding: utf-8 -*-
"""正确统计参照 blocks（块 id 用 blocks.json 映射，biome 用 256 名列表）。
关键判定：参照是否含 FEATURE 产物（tuff/diorite/granite 岩石替换、cave_air 洞穴、树）→ SURFACE vs FULL 状态。
"""
import struct, json, sys, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
# blocks.json: name -> id
id2name = {v: k for k, v in blocks.items()}

def read_blocks(path):
    f = open(path, "rb")
    magic, seed = struct.unpack(">iq", f.read(12))
    size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
    chunks = {}
    for c in range(size * size):
        cx, cz = struct.unpack(">ii", f.read(8))
        d = struct.unpack(f">{16*16*h}H", f.read(16 * 16 * h * 2))
        biome = []
        for _ in range(256):
            blen = struct.unpack(">H", f.read(2))[0]
            biome.append(f.read(blen).decode("utf-8", errors="replace"))
        chunks[(cx, cz)] = (d, biome)
    f.close()
    return size, ox, oz, miny, h, chunks

def stat(path, label):
    size, ox, oz, miny, h, chunks = read_blocks(path)
    cnt = collections.Counter()
    biome_cnt = collections.Counter()
    col_air_under_0 = 0  # y<0 有 air 的列数（洞穴特征）
    for (cx, cz), (d, biome) in chunks.items():
        for v in d:
            nm = id2name.get(v, f"id{v}")
            cnt[nm.split(":")[1] if ":" in nm else nm] += 1
        for b in biome:
            biome_cnt[b.split(":")[1] if ":" in b else b] += 1
        # 洞穴检测：y<0 段每列 air 数
        for lx in range(16):
            for lz in range(16):
                airs = 0
                for ly in range(0, 100):  # y = miny+ly, miny=-64 → y=-64..36
                    idx = (ly * 16 + lz) * 16 + lx
                    nm = id2name.get(d[idx], "?")
                    if nm.endswith("air"):
                        airs += 1
                if airs >= 3:
                    col_air_under_0 += 1
    total = sum(cnt.values())
    print(f"== {label}: size={size} origin=({ox},{oz}) miny={miny} h={h} ==")
    print(f"  y<0 洞穴 air 列数（≥3 层 air）: {col_air_under_0} / {size*size*256}")
    print("  块类型 top15:")
    for nm, n in cnt.most_common(15):
        print(f"    {nm:<22} {n:>9} ({100.0*n/total:.2f}%)")
    print("  biome top8:")
    for nm, n in biome_cnt.most_common(8):
        print(f"    {nm:<20} {n:>6}")

stat(DATA + r"\vanilla_3005152118058349760_4_-1320400_-198064.blocks", "新 seed 300515 用户目标区")
stat(DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks", "-288 参照")
