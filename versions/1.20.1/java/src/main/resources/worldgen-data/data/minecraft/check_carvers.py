import json, glob, os
d = r"E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\biome"
patterns = {}
for f in glob.glob(os.path.join(d, "*.json")):
    data = json.load(open(f))
    carvers = data.get("carvers", {}).get("air", [])
    key = tuple(sorted(carvers))
    patterns.setdefault(key, []).append(os.path.basename(f))
print(f"unique carver patterns: {len(patterns)}")
for k, v in patterns.items():
    print(f"  {list(k)} -> {len(v)} biomes (e.g. {v[:3]})")
