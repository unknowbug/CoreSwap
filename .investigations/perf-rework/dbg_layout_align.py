import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

spec = importlib.util.spec_from_file_location("m", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)

# gen() 收集（gen_cpu 用）
g.gen(fd)
gen_keys = [p["_key"] for _,p in g.noise_instances]
print(f"[gen()] noise_instances={len(g.noise_instances)} split_total(未算)")

# gen_shader 会 _reset_collect + gen_df，这里手动模拟
g._reset_collect()
g.gen_df(fd)
df_keys = [p["_key"] for _,p in g.noise_instances]
print(f"[gen_df()] noise_instances={len(g.noise_instances)}")

# 对比 key 序列
print("\n--- key 序列差异（前 30 个）---")
for i,(a,b) in enumerate(zip(gen_keys, df_keys)):
    if a != b:
        print(f"[{i}] gen={a}  df={b}")
    if i >= 30: break

# 统计 key 集合差异
gs, ds = set(gen_keys), set(df_keys)
print(f"\ngen keys={len(gs)}  df keys={len(ds)}")
print("gen 独有:", sorted(gs - ds)[:20])
print("df 独有:", sorted(ds - gs)[:20])
