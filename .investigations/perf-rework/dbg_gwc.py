import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
# hook gen 的 interp 分支（打印 interp_idx + suffix 变化）
orig_gen = dfc_gen.DfcGen.gen
def h_gen(self, df):
    if isinstance(df, dict) and df.get('type') == 'minecraft:interpolated':
        pass
    return orig_gen(self, df)
dfc_gen.DfcGen.gen = h_gen
# 直接 hook _gen_spline 打印 suffix
orig_gs = dfc_gen.DfcGen._gen_spline
def h_gs(self, spline):
    r = orig_gs(self, spline)
    # 只打印第一次 + suffix 非 @c0 的
    return r
dfc_gen.DfcGen._gen_spline = h_gs
# hook gen_with_coords 打印 suffix
orig_gwc = dfc_gen.DfcGen.gen_with_coords
def h_gwc(self, df, cx, cy, cz, fx=None, fy=None, fz=None):
    if self.noise_key_suffix != '':
        print(f"gen_with_coords suffix={self.noise_key_suffix}")
    return orig_gwc(self, df, cx, cy, cz, fx, fy, fz)
dfc_gen.DfcGen.gen_with_coords = h_gwc
g.gen(fd)
