import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
nr = settings["noise_router"]
print("noise_router keys:", list(nr.keys()))
for k, v in nr.items():
    if isinstance(v, (str, int, float)):
        print(f"  {k} = {v}")
    elif isinstance(v, dict):
        print(f"  {k} type={v.get('type')} keys={list(v.keys())[:5]}")
# where is factor referenced?
raw = open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8').read()
idx = raw.find('overworld/factor')
print("\nfactor ref context:", raw[max(0,idx-120):idx+40].replace("\n"," "))
