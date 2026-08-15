import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
# hook gen 的 noise 注册，打印 continentalness 的 suffix
orig_register = dfc_gen.DfcGen._register_noise
def hooked(self, kind, key, params):
    if 'continentalness' in key:
        print(f"register continentalness kind={kind} key={key}")
    return orig_register(self, kind, key, params)
dfc_gen.DfcGen._register_noise = hooked
g.gen(fd)
