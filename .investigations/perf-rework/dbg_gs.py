import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
orig_gs = dfc_gen.DfcGen._gen_spline
cnt = [0]
def h_gs(self, spline):
    cnt[0] += 1
    coord = spline.get('coordinate', '?')
    print(f"_gen_spline #{cnt[0]} coord={json.dumps(coord)[:30]} suffix={self.noise_key_suffix}")
    return orig_gs(self, spline)
dfc_gen.DfcGen._gen_spline = h_gs
g.gen(fd)
print("total _gen_spline calls:", cnt[0])
