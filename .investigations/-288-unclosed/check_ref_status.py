# -*- coding: utf-8 -*-
"""检查 8576/3200 参照是否含 FEATURE 产物（岩石替换/ore/草），判定 SURFACE vs FULL 状态。
"""
import struct, json, sys, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
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
        chunks[(cx, cz)] = d
    f.close()
    return magic, seed, size, ox, oz, miny, h, chunks

FEATURE_MARKERS = {
    "andesite", "granite", "diorite", "tuff",           # 岩石替换
    "coal_ore", "iron_ore", "copper_ore", "gold_ore", "diamond_ore", "redstone_ore", "lapis_ore",  # 矿石
    "oak_log", "oak_leaves", "birch_log", "grass", "tall_grass", "poppy", "dandelion",  # 树草花
    "cave_air",                                            # 洞穴
    "smooth_basalt", "calcite", "amethyst_block",          # 紫晶洞
    "dirt_path", "kelp", "seagrass", "mossy_cobblestone",  # 结构/植被
    "deepslate_coal_ore", "deepslate_iron_ore", "deepslate_copper_ore", "deepslate_gold_ore",
    "deepslate_diamond_ore", "deepslate_redstone_ore", "deepslate_lapis_ore",
}

for p in [DATA + r"\vanilla_8576294172403134396_6_720_-432.blocks",
          DATA + r"\vanilla_-8248318472910187742_4_3200_3208.blocks",
          DATA + r"\vanilla_-8248318472910187742_4_-288_-256.blocks"]:
    try:
        magic, seed, size, ox, oz, miny, h, chunks = read_blocks(p)
    except FileNotFoundError:
        print(f"{p}: MISSING")
        continue
    cnt = collections.Counter()
    for (cx, cz), d in chunks.items():
        for v in d:
            nm = id2name.get(v, "")
            short = nm.split(":")[1] if ":" in nm else nm
            cnt[short] += 1
    feats = {k: cnt[k] for k in FEATURE_MARKERS if cnt.get(k, 0) > 0}
    total_feat = sum(feats.values())
    total = sum(cnt.values())
    print(f"== {p.split(chr(92))[-1]} seed={seed} size={size} origin=({ox},{oz}) ==")
    print(f"  FEATURE 产物合计: {total_feat} ({100.0*total_feat/total:.2f}%)")
    for k in sorted(feats, key=lambda x: -feats[x])[:10]:
        print(f"    {k}: {feats[k]}")
    print()
