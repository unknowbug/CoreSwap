# -*- coding: utf-8 -*-
"""-288 差异重归因：FEATURE 块剔除后，量化真核心差异（e 翻转 + surface 规则差）。
数据: m288_run1.txt（-288 FULL 差异全量，含块 id）
分类: FEATURE 类（岩石替换/矿石/树草/洞穴/村庄 dirt_path/紫晶洞/沉船方块）vs 核心类（water<->terrain 判定差）
"""
import json, sys, re, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2name = {v: k for k, v in blocks.items()}

LINE = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=(.*)$")
rows = []
with open(r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\m288_run1.txt", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line.startswith("MISMATCH"):
            continue
        m = LINE.match(line)
        if not m:
            continue
        cx, cz, x, y, z, gid, vid, biome = m.groups()
        rows.append((int(x), int(y), int(z), int(gid), int(vid), biome))
print("total mismatch:", len(rows))

def shortname(bid):
    nm = id2name.get(bid, f"id{bid}")
    return nm.split(":")[1] if ":" in nm else nm

# FEATURE 类块（vanilla 侧出现这些 = FEATURE 产物差异）
FEATURE_BLOCKS = {
    # 岩石替换
    "andesite", "granite", "diorite", "tuff",
    # 矿石
    "coal_ore", "copper_ore", "iron_ore", "gold_ore", "diamond_ore", "redstone_ore",
    "lapis_ore", "emerald_ore", "deepslate_coal_ore", "deepslate_copper_ore", "deepslate_iron_ore",
    "deepslate_gold_ore", "deepslate_diamond_ore", "deepslate_redstone_ore", "deepslate_lapis_ore",
    # 树草（森林）
    "oak_log", "oak_leaves", "birch_log", "birch_leaves", "spruce_log", "spruce_leaves",
    "acacia_log", "acacia_leaves", "dark_oak_log", "dark_oak_leaves", "jungle_log", "jungle_leaves",
    "cherry_log", "cherry_leaves", "azalea", "flowering_azalea", "mangrove_log", "mangrove_leaves",
    "grass", "tall_grass", "short_grass", "poppy", "dandelion", "azure_bluet", "allium", "cornflower",
    "oxeye_daisy", "lilac", "rose_bush", "peony", "sunflower", "fern", "large_fern", "dead_bush",
    # 村庄/结构方块
    "dirt_path", "oak_planks", "oak_stairs", "oak_fence", "cobblestone", "mossy_cobblestone",
    "chest", "farmland", "hay_block", "white_wool", "brown_wool", "red_wool", "bed", "bell",
    "cobblestone_stairs", "cobblestone_wall", "glass", "glass_pane", "torch", "crafting_table",
    "furnace", "smooth_stone", "oak_door", "oak_slab", "cobbled_deepslate", "bookshelf",
    # 沉船/海洋结构
    "oak_planks_vertical", "spruce_planks", "spruce_stairs", "spruce_fence", "spruce_log_vertical",
    "air_bubble", "ladder", "iron_bars",
    # 紫晶洞
    "amethyst_block", "budding_amethyst", "calcite", "smooth_basalt", "amethyst_cluster",
    "small_amethyst_bud", "medium_amethyst_bud", "large_amethyst_bud",
    # 水下植被
    "kelp", "kelp_plant", "seagrass", "tall_seagrass",
    # 洞穴装饰
    "dripstone_block", "pointed_dripstone", "cave_vines", "cave_vines_plant", "moss_carpet",
    # 其他 FEATURE
    "clay", "mud", "rooted_dirt", "moss_block",
}

def is_feature(nm):
    return nm in FEATURE_BLOCKS

# 分类统计
core_pairs = collections.Counter()   # 核心差异（无 FEATURE 块）
feat_pairs = collections.Counter()   # 含 FEATURE 块
core_rows, feat_rows = [], []

for x, y, z, gid, vid, biome in rows:
    gn, vn = shortname(gid), shortname(vid)
    if is_feature(gn) or is_feature(vn):
        feat_pairs[(gn, vn)] += 1
        feat_rows.append((x, y, z, gn, vn, biome))
    else:
        core_pairs[(gn, vn)] += 1
        core_rows.append((x, y, z, gn, vn, biome))

print(f"\n== FEATURE 类差异（含 FEATURE 块）: {len(feat_rows)} 块 ==")
for (g, v), n in feat_pairs.most_common(20):
    print(f"  got={g:<16} vanilla={v:<16} {n:>6}")

print(f"\n== 核心差异（无 FEATURE 块）: {len(core_rows)} 块 ==")
for (g, v), n in core_pairs.most_common(20):
    print(f"  got={g:<16} vanilla={v:<16} {n:>6}")

# 核心差异细分：water<->terrain（e 翻转候选）vs terrain<->air（洞穴/carvers）vs terrain<->terrain（surface 规则）
water_terr = collections.Counter()
terr_air = collections.Counter()
terr_terr = collections.Counter()
for x, y, z, gn, vn, biome in core_rows:
    if "water" in (gn, vn) or "lava" in (gn, vn):
        water_terr[(gn, vn)] += 1
    elif "air" in (gn, vn):
        terr_air[(gn, vn)] += 1
    else:
        terr_terr[(gn, vn)] += 1

print(f"\n== 核心差异细分 ==")
print(f"  water<->terrain（e 翻转候选）: {sum(water_terr.values())} 块")
for (g, v), n in water_terr.most_common(12):
    print(f"    got={g:<14} vanilla={v:<14} {n:>6}")
print(f"  terrain<->air（洞穴/carvers）: {sum(terr_air.values())} 块")
for (g, v), n in terr_air.most_common(6):
    print(f"    got={g:<14} vanilla={v:<14} {n:>6}")
print(f"  terrain<->terrain（surface 规则/岩石替换）: {sum(terr_terr.values())} 块")
for (g, v), n in terr_terr.most_common(10):
    print(f"    got={g:<14} vanilla={v:<14} {n:>6}")
