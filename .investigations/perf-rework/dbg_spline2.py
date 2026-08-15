import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
# find all string refs in fd
refs = []
def find_refs(d, path="/"):
    if isinstance(d, str) and d.startswith("minecraft:"):
        refs.append((path, d))
    elif isinstance(d, dict):
        for k, v in d.items():
            find_refs(v, path + "/" + k)
    elif isinstance(d, list):
        for i, it in enumerate(d):
            find_refs(it, path + f"[{i}]")
find_refs(fd)
print("refs in final_density:", refs)
# factor.json full spline hunt
f = json.load(open(os.path.join(DFDIR, "overworld", "factor.json"), encoding='utf-8'))
def find_spline(d, path="/"):
    if isinstance(d, dict):
        if "spline" in d or ("points" in d and "coordinate" in d):
            print("SPLINE at", path, "| keys:", list(d.keys()))
        for k, v in d.items():
            find_spline(v, path + "/" + k)
    elif isinstance(d, list):
        for i, it in enumerate(d):
            find_spline(it, path + f"[{i}]")
find_spline(f)
