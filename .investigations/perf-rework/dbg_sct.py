import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
# hook _spline_coord_type 和 _gen_spline，打印 suffix
orig_sct = dfc_gen.DfcGen._spline_coord_type
def h_sct(self, coord):
    r = orig_sct(self, coord)
    print(f"_spline_coord_type coord={json.dumps(coord)[:40]} suffix={self.noise_key_suffix} -> {r}")
    return r
dfc_gen.DfcGen._spline_coord_type = h_sct
g.gen(fd)
