import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
# trace: argument1 path
a1 = fd["argument1"]
print("a1 type:", a1.get("type"))
a1a = a1["argument"]
print("a1.argument type:", a1a.get("type"))
a1aa = a1a["argument"]
print("a1.argument.argument type:", a1aa.get("type"))
a1aaa = a1aa["argument"]
print("a1.argument.argument.argument type:", a1aaa.get("type"))
a1aaaa = a1aaa["argument"]
print("a1.argument.argument.argument.argument type:", a1aaaa.get("type"), "keys:", list(a1aaaa.keys()))
a2 = a1aaaa["argument2"]
print("  .argument2 type:", a2.get("type"))
a2a = a2["argument2"]
print("  .argument2.argument2 type:", a2a.get("type"), "keys:", list(a2a.keys()))
