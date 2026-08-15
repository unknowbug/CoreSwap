import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings['noise_router']['final_density']
print(json.dumps(fd, indent=1)[:12000])
