import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen_shader(fd)
# 检查 interp 的 delegate（interp_instances）是否含嵌套 interpolated
for i, arg in enumerate(g.interp_instances):
    def has_interp(d, depth=0):
        if isinstance(d, dict):
            if d.get('type') == 'minecraft:interpolated':
                return True
            for k,v in d.items():
                if isinstance(v, dict) and has_interp(v, depth+1): return True
                if isinstance(v, list):
                    for it in v:
                        if isinstance(it, dict) and has_interp(it, depth+1): return True
        return False
    print(f"interp_{i} delegate 含嵌套 interp: {has_interp(arg)}")
