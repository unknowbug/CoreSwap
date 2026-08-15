import json, sys, os, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)
g.gen_shader(fd)
# 每个 interp 的 8 角点各自引用哪些噪声（按角点切）
for idx, samples in g.interp_funcs:
    if idx not in (0, 5): continue
    corner_ids = []
    for c, s in enumerate(samples):
        ids = set(int(m.group(1)) for m in re.finditer(r'(?:normal_noise|interp_noise)_(\d+)', s))
        corner_ids.append(sorted(ids))
    all_ids = set()
    for ids in corner_ids: all_ids |= set(ids)
    print(f"interp_{idx}: {len(all_ids)} unique noises across 8 corners")
    for c, ids in enumerate(corner_ids):
        print(f"  corner d{['000','100','010','110','001','101','011','111'][c]}: {len(ids)} noises, min={min(ids) if ids else '-'} max={max(ids) if ids else '-'}")
    # 角点间重叠
    if corner_ids:
        common = set(corner_ids[0])
        for ids in corner_ids[1:]: common &= set(ids)
        print(f"  all-corners common: {len(common)}")
