# -*- coding: utf-8 -*-
# 解析 block_probe -mismatch 输出 m288_run1.txt，统计差异构成（原始数据采集，解读交分析角色）
# 用法: python analyze_m288.py <mismatch.txt> <blocks.json> <outdir>
import sys, re, os, json, collections

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

def load_blocks_json(path):
    """blocks.json: {minecraft:name: id} -> {id: name}"""
    with open(path, "r", encoding="utf-8") as f:
        mapping = json.load(f)
    rev = {}
    for name, bid in mapping.items():
        rev[bid] = name
    return rev

# 块类别：natural = 自然地形/含水层/表面规则产物（C++ 应生成）；structure = 结构/FEATURE 产物（C++ 不做）
NATURAL_KEYWORDS = ("stone", "deepslate", "dirt", "grass_block", "sand", "gravel",
                    "terracotta", "water", "air", "cave_air", "void_air", "bedrock",
                    "clay", "mud", "snow_block", "snow", "ice", "packed_ice",
                    "sandstone", "andesite", "granite", "diorite", "tuff",
                    "calcite", "dripstone", "magma", "basalt", "blackstone",
                    "powder_snow", "moss_block", "mossy_cobblestone")
STRUCTURE_KEYWORDS = ("_ore", "chest", "log", "_planks", "_wood", "leaves",
                      "cobblestone", "tnt", "rail", "lantern", "torch", "fence",
                      "sapling", "flower", "tall_grass", "grass", "vine", "lily",
                      "kelp", "seagrass", "coral", "sugar_cane", "bamboo", "pumpkin",
                      "melon", "dead_", "bone", "pot", "button", "door", "sign",
                      "slab", "stairs", "wall", "trapdoor", "pressure_plate", "barrel",
                      "sponge", "sea_pickle", "cactus", "wheat", "carrot", "potato",
                      "beetroot", "turtle", "shulker", "conduit", "lodestone",
                      "spawner", "chest")

def classify(name):
    if name == "minecraft:air":
        return "air"
    if any(k in name for k in STRUCTURE_KEYWORDS):
        return "structure_feature"
    if any(k in name for k in NATURAL_KEYWORDS):
        return "natural"
    return "unknown"

def main():
    mismatch_path, blocks_path, outdir = sys.argv[1], sys.argv[2], sys.argv[3]
    os.makedirs(outdir, exist_ok=True)
    rev = load_blocks_json(blocks_path)

    # MISMATCH chunk(cx,cz) pos(x,y,z) got=N vanilla=M biome=...
    pat = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=([\w:]+)")
    pairs = collections.Counter()       # (got, vanilla) -> count
    chunk_stat = collections.Counter()  # chunk -> count
    rows = []
    total_mismatch = 0
    with open(mismatch_path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            m = pat.search(line)
            if not m:
                continue
            cx, cz, x, y, z, got, van, biome = m.groups()
            cx, cz, x, y, z, got, van = int(cx), int(cz), int(x), int(y), int(z), int(got), int(van)
            pairs[(got, van)] += 1
            chunk_stat[(cx, cz)] += 1
            rows.append((cx, cz, x, y, z, got, van, biome))
            total_mismatch += 1

    with open(os.path.join(outdir, "m288_pair_counts.txt"), "w", encoding="utf-8") as f:
        f.write(f"# (got, vanilla) pair counts, total={total_mismatch}\n")
        for (got, van), cnt in pairs.most_common():
            gname = rev.get(got, f"?{got}")
            vname = rev.get(van, f"?{van}")
            f.write(f"got={got}({gname}) vanilla={van}({vname}) count={cnt}\n")

    with open(os.path.join(outdir, "m288_chunk_counts.txt"), "w", encoding="utf-8") as f:
        f.write("# chunk (cx,cz) -> mismatch count\n")
        for (cx, cz), cnt in sorted(chunk_stat.items()):
            f.write(f"chunk({cx},{cz}) {cnt}\n")

    # 按 vanilla 类别汇总（natural / structure_feature / air / unknown）
    van_cat = collections.Counter()
    van_name_cnt = collections.Counter()
    natural_rows = []
    for cx, cz, x, y, z, got, van, biome in rows:
        vname = rev.get(van, f"?{van}")
        cat = classify(vname)
        van_cat[cat] += 1
        van_name_cnt[(van, vname, cat)] += 1
        if cat == "natural" or cat == "unknown":
            natural_rows.append((cx, cz, x, y, z, got, van, vname, biome))

    with open(os.path.join(outdir, "m288_vanilla_cat.txt"), "w", encoding="utf-8") as f:
        f.write("# vanilla 侧块类别汇总（natural=C++ 应生成、structure_feature=C++ 不做的结构/FEATURE）\n")
        for cat, cnt in van_cat.most_common():
            f.write(f"{cat}: {cnt} ({cnt*100.0/total_mismatch:.2f}%)\n")
        f.write("\n# 明细（vanilla 名称 -> 计数）\n")
        for (van, vname, cat), cnt in van_name_cnt.most_common():
            f.write(f"{cat} vanilla={van}({vname}) {cnt}\n")

    # 非结构 mismatch 行（natural/unknown 类别）全量落盘，供分析定位
    with open(os.path.join(outdir, "m288_natural_rows.txt"), "w", encoding="utf-8") as f:
        f.write("# non-structure mismatch rows (vanilla 侧 natural/unknown 类别)\n")
        for cx, cz, x, y, z, got, van, vname, biome in natural_rows:
            gname = rev.get(got, f"?{got}")
            f.write(f"chunk({cx},{cz}) pos({x},{y},{z}) got={got}({gname}) vanilla={van}({vname}) biome={biome}\n")

    print(f"[OK] total_mismatch={total_mismatch}")
    for cat, cnt in van_cat.most_common():
        print(f"[CAT] {cat}: {cnt}")
    print(f"[OK] non-structure rows written: {len(natural_rows)}")
    print(f"[OK] outputs in {outdir}")

if __name__ == "__main__":
    main()
