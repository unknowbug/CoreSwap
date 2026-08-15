import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
print("fd type:", fd.get("type"), "| argument2:", str(fd.get("argument2"))[:60])
# resolve noodle
noodle_path = os.path.join(DFDIR, "overworld", "caves", "noodle.json")
print("noodle exists:", os.path.exists(noodle_path))
noodle = json.load(open(noodle_path, encoding='utf-8'))
print("noodle type:", noodle.get("type"), "keys:", list(noodle.keys())[:6])
def find_spline(d, path="/", depth=0):
    if isinstance(d, dict):
        t = d.get("type", "")
        if t == "minecraft:spline" and "spline" in d:
            print(f"FOUND spline at {path}")
            return True
        for k, v in d.items():
            if isinstance(v, dict) and find_spline(v, path + "/" + k, depth+1):
                return True
    return False
print("noodle has spline:", find_spline(noodle))
# sloped_cheese
sc = json.load(open(os.path.join(DFDIR, "overworld", "sloped_cheese.json"), encoding='utf-8'))
print("sloped_cheese type:", sc.get("type"), "keys:", list(sc.keys())[:6])
print("sloped_cheese has spline:", find_spline(sc))
