import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
orig_grc = dfc_gen.DfcGen._gen_registry_call
def h_grc(self, ref):
    if 'jagged' in ref:
        print(f"_gen_registry_call {ref} interp_depth={self.interp_depth} suffix={self.noise_key_suffix} node_mode={self.node_mode}")
    return orig_grc(self, ref)
dfc_gen.DfcGen._gen_registry_call = h_grc
g.gen(fd)
