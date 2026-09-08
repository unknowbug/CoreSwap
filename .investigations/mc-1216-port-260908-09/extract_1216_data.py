# 260908-09 P2: extract data/minecraft/worldgen/** + tags/blocks/** from 1.21.6 loom merged jar
# into versions/1.21.6/data/worldgen/data/minecraft/ (mirror 1.20.1 layout, jar-style data root).
import sys, zipfile, shutil, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

JAR = r"E:\PYTHON\CoreSwap\runtime\1.21.6\java\.gradle\loom-cache\minecraftMaven\net\minecraft\minecraft-merged-b07cf08c30\1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2\minecraft-merged-b07cf08c30-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar"
DEST = r"E:\PYTHON\CoreSwap\versions\1.21.6\data\worldgen\data\minecraft"
PREFIXES = ["data/minecraft/worldgen/", "data/minecraft/tags/"]

n = 0
with zipfile.ZipFile(JAR) as zf:
    names = [x for x in zf.namelist()
             if any(x.startswith(p) for p in PREFIXES) and x.endswith(".json")]
    for name in names:
        rel = name[len("data/minecraft/"):]  # worldgen/... or tags/...
        dst = os.path.join(DEST, rel.replace("/", os.sep))
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        with zf.open(name) as src, open(dst, "wb") as out:
            shutil.copyfileobj(src, out)
        n += 1
print(f"[OK] extracted {n} json files -> {DEST}")

# sanity: key dirs present
# NOTE: 1.21.x jar uses tags/block (singular); Rust block_tags.rs loads <wg_dir>/data/<ns>/tags/blocks/
# (plural, 1.20.1 layout) -> we rename tags/block -> tags/blocks after extraction (data-level fix, zero engine change).
TAGSRC = os.path.join(DEST, "tags", "block")
TAGDST = os.path.join(DEST, "tags", "blocks")
if os.path.isdir(TAGSRC) and not os.path.isdir(TAGDST):
    os.rename(TAGSRC, TAGDST)
    print("[OK] renamed tags/block -> tags/blocks (Rust loader path contract)")
for sub in ("worldgen/noise_settings", "worldgen/density_function", "worldgen/noise",
            "worldgen/biome", "worldgen/configured_carver", "worldgen/configured_feature",
            "worldgen/placed_feature", "worldgen/multi_noise_biome_source_parameter_list",
            "worldgen/structure_set", "tags/blocks"):
    p = os.path.join(DEST, sub.replace("/", os.sep))
    cnt = sum(len(f) for _, _, f in os.walk(p)) if os.path.isdir(p) else -1
    print(f"  {sub}: {cnt} files")
