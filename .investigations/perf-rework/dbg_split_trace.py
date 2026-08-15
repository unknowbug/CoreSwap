import json, importlib.util, sys, io
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

spec = importlib.util.spec_from_file_location("m", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect()
g.gen_df(fd)

# 检查 normal_chain_index 里 continentalness 的 key
ck = sorted(k for k in g.normal_chain_index if 'continentalness' in k)
print(f"normal_chain_index continentalness keys: {ck}")

# 手动调 _gen_split_lines（顶层）
lines = g._gen_split_lines(fd, "x", "y", "z")
print(f"_gen_split_lines 总行数: {len(lines)}")
cont = [l for l in lines if 'continentalness' in l or 'normals[0]' in l]
print(f"continentalness 相关行: {len(cont)}")
for l in cont[:3]:
    print("  ", l.strip()[:150])

# 检查 noise_instances 前几个
for i, (kind, p) in enumerate(g.noise_instances[:6]):
    print(f"instance[{i}] kind={kind} key={p.get('_key')}")

# slot 表
for i, s in enumerate(g.noise_slots[:6]):
    print(f"slot[{i}] key={s['key']} is_corner={s['is_corner']} base={s['base']} stride={s['stride']}")
