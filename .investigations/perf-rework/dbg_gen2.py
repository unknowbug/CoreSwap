import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
orig_gen = dfc_gen.DfcGen.gen
def h_gen(self, df):
    if isinstance(df, str) and ('sloped_cheese' in df or 'jagged' in df):
        print(f"gen(str) {df} interp_depth={self.interp_depth} suffix={self.noise_key_suffix}")
    if isinstance(df, dict) and df.get('type') == 'minecraft:interpolated':
        print(f"gen(interpolated) interp_depth={self.interp_depth} suffix={self.noise_key_suffix}")
    return orig_gen(self, df)
dfc_gen.DfcGen.gen = h_gen
g.gen(fd)
