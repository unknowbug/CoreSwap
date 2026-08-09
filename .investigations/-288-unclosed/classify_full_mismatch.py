# -*- coding: utf-8 -*-
"""新 seed FULL 差异归类：FEATURE 类 vs 核心类（aquifer 判定差）。
数据: m300515_run1.txt（-mismatch 输出，got=<id> vanilla=<id>）
"""
import json, sys, re, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2name = {v: k for k, v in blocks.items()}

# 打印 970/909 映射（第一行示例）
for bid in (970, 909):
    print(f"id {bid} = {id2name.get(bid, '?')}")

LINE = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=(.*)$")
rows = []
with open(r"E:\PYTHON\CoreSwap\.investigations\-288-unclosed\m300515_run1.txt", encoding="utf-8") as f:
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

# 分类
def cat(nm):
    n = nm.split(":")[1] if ":" in nm else nm
    if n == "air": return "air"
    if n == "cave_air": return "cave_air"
    if n == "water": return "water"
    if n == "lava": return "lava"
    if n in ("stone", "deepslate", "bedrock", "dirt", "grass_block", "gravel", "sand", "sandstone", "mud", "clay"):
        return "core_terrain"
    if n in ("andesite", "granite", "diorite", "tuff"):
        return "rock_replace"
    if "ore" in n or n.endswith("_ore"):
        return "ore"
    if n in ("oak_log", "oak_leaves", "birch_log", "birch_leaves", "spruce_log", "spruce_leaves",
             "acacia_log", "acacia_leaves", "dark_oak_log", "dark_oak_leaves", "jungle_log", "jungle_leaves",
             "cherry_log", "cherry_leaves", "azalea", "flowering_azalea", "mangrove_log", "mangrove_leaves",
             "grass", "tall_grass", "short_grass", "poppy", "dandelion", "azure_bluet", "allium", "cornflower",
             "oxeye_daisy", "lilac", "rose_bush", "peony", "sunflower", "fern", "large_fern", "dead_bush",
             "dandelion", "torchflower", "pitcher_plant", "vine", "sweet_berry_bush", "brown_mushroom", "red_mushroom",
             "brown_mushroom_block", "red_mushroom_block", "mushroom_stem"):
        return "vegetation"
    if n in ("kelp", "kelp_plant", "seagrass", "tall_seagrass"):
        return "aquatic_veg"
    if n in ("cobblestone", "mossy_cobblestone", "diorite", "granite", "andesite"):
        return "cobble_feature"
    return "other"

pair_cat = collections.Counter()
pair_block = collections.Counter()
for x, y, z, gid, vid, biome in rows:
    gn, vn = shortname(gid), shortname(vid)
    pc = (cat(gn), cat(vn))
    pair_cat[pc] += 1
    pair_block[(gn, vn)] += 1

print("\n== 差异对（类别级别）top25 ==")
for (gc, vc), n in pair_cat.most_common(25):
    print(f"  got={gc:<16} vanilla={vc:<16} {n:>7}")

print("\n== 差异对（块级别）top30 ==")
for (g, v), n in pair_block.most_common(30):
    print(f"  got={g:<20} vanilla={v:<20} {n:>7}")

# 核心类统计：含 core_terrain/water 双向的差异
core_pairs = {k: v for k, v in pair_cat.items()
              if ("core_terrain" in k or "water" in k or "lava" in k) and "vegetation" not in k and "rock_replace" not in k and "ore" not in k and "aquatic_veg" not in k}
print("\n== 疑似核心差异（terrain/water 相关，排除 FEATURE 类）==")
for (gc, vc), n in sorted(core_pairs.items(), key=lambda kv: -kv[1]):
    print(f"  got={gc:<16} vanilla={vc:<16} {n:>7}")

# y 分布（核心类）
core_ys = collections.Counter()
for x, y, z, gid, vid, biome in rows:
    gn, vn = shortname(gid), shortname(vid)
    pc = (cat(gn), cat(vn))
    if pc in core_pairs and core_pairs[pc] > 100:
        core_ys[y] += 1
print("\n== 核心类 y 分布 top15 ==")
for y, n in sorted(core_ys.items(), key=lambda kv: -kv[1])[:15]:
    print(f"  y={y}: {n}")
