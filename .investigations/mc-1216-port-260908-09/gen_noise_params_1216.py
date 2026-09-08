# 260908-09 P2: synthesize noise_params.json for 1.21.6 from extracted worldgen/noise/*.json
# (1.20.1 was a runtime NoiseParamProbe dump; 1.21.6 noise is fully data-driven in jar -> static merge is exact).
import sys, os, json
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

SRC = r"E:\PYTHON\CoreSwap\versions\1.21.6\data\worldgen\data\minecraft\worldgen\noise"
OUT = r"E:\PYTHON\CoreSwap\versions\1.21.6\data\noise_params.json"

result = {}
for fn in sorted(os.listdir(SRC)):
    if not fn.endswith(".json"):
        continue
    name = "minecraft:" + fn[:-5]
    with open(os.path.join(SRC, fn), encoding="utf-8") as f:
        d = json.load(f)
    result[name] = {"firstOctave": d["firstOctave"], "amplitudes": d["amplitudes"]}

with open(OUT, "w", encoding="utf-8") as f:
    json.dump(result, f, indent=2, ensure_ascii=False)
print(f"[OK] noise_params.json: {len(result)} entries -> {OUT}")
# sanity spot checks vs 1.20.1 known keys
for k in ("minecraft:temperature", "minecraft:aquifer_barrier"):
    print(" ", k, "->", "present" if k in result else "MISSING")
