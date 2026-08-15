import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
print("fd keys:", list(fd.keys()))
print("fd type:", fd.get("type"))
print("fd argument:", str(fd.get("argument"))[:80])
# resolve factor
f = json.load(open(os.path.join(DFDIR, "overworld", "factor.json"), encoding='utf-8'))
print("factor type:", f.get("type"), "keys:", list(f.keys()))
print("factor argument type:", f["argument"].get("type"))
print("factor arg2 type:", f["argument"]["argument"]["argument2"].get("type"))
print("factor arg2 keys:", list(f["argument"]["argument"]["argument2"].keys()))
spline = f["argument"]["argument"]["argument2"].get("spline")
print("spline present:", spline is not None)
if spline:
    print("spline coord:", str(spline.get("coordinate"))[:80])
    print("spline points:", len(spline.get("points", [])))
