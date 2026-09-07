# 260907-04: extract data/minecraft/tags/blocks/*.json from loom minecraft-merged.jar (1.20.1)
# into both repo working data dir and mod resources (mirror jar layout data/minecraft/tags/blocks).
import sys, zipfile, shutil, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

JAR = r"C:\Users\NDark\.gradle\caches\fabric-loom\1.20.1\minecraft-merged.jar"
DESTS = [
    r"E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\tags",
    r"E:\PYTHON\CoreSwap\runtime\1.20.1\java\src\main\resources\worldgen-data\data\minecraft\tags",
]
PREFIX = "data/minecraft/tags/blocks/"

n_total, n_copied, total_bytes = 0, 0, 0
with zipfile.ZipFile(JAR) as zf:
    names = [x for x in zf.namelist() if x.startswith(PREFIX) and x.endswith(".json")]
    n_total = len(names)
    for dest in DESTS:
        os.makedirs(os.path.join(dest, "blocks"), exist_ok=True)
        for name in names:
            rel = name[len("data/minecraft/tags/"):]  # blocks/<tag>.json
            dst = os.path.join(dest, rel.replace("/", os.sep))
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            with zf.open(name) as src, open(dst, "wb") as out:
                shutil.copyfileobj(src, out)
            n_copied += 1
            total_bytes += os.path.getsize(dst)

print(f"[OK] jar tag files: {n_total}, copied (both dests): {n_copied}, total bytes written: {total_bytes}")
# sanity: dump the two key tags
with zipfile.ZipFile(JAR) as zf:
    for t in ("overworld_carver_replaceables", "base_stone_overworld", "dirt", "sand", "stone_ore_replaceables", "deepslate_ore_replaceables", "netherrack", "base_stone_nether"):
        try:
            print(f"--- {t}:", zf.read(PREFIX + t + ".json").decode("utf-8"))
        except KeyError:
            print(f"--- {t}: MISSING")
