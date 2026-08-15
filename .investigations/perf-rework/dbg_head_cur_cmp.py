import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

def load_module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m

head = load_module(r'E:\PYTHON\CoreSwap\_head_dfc.py', "m_head")
g = head.DfcGen(dfdir, ndir)
g.gen(fd)
head_keys = [p["_key"] for _, p in g.noise_instances]
head_chains = [c for c in g.coord_chains]

cur = load_module(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py', "m_cur")
g2 = cur.DfcGen(dfdir, ndir)
g2._reset_collect()
g2.gen_df(fd)
cur_keys = [p["_key"] for _, p in g2.noise_instances]
cur_chains = [c for c in g2.coord_chains]

print(f"HEAD: {len(head_keys)} instances, {len(head_chains)} chains")
print(f"CUR : {len(cur_keys)} instances, {len(cur_chains)} chains")
print(f"key 列表一致: {head_keys == cur_keys}")
if head_keys != cur_keys:
    hs, cs = set(head_keys), set(cur_keys)
    print("HEAD 独有:", sorted(hs - cs))
    print("CUR 独有:", sorted(cs - hs))
    # 顺序差异
    for i, (a, b) in enumerate(zip(head_keys, cur_keys)):
        if a != b:
            print(f"  首个顺序差异 [{i}]: HEAD={a} CUR={b}")
            break
print(f"chain 列表一致: {head_chains == cur_chains}")
if head_chains != cur_chains:
    for i, (a, b) in enumerate(zip(head_chains, cur_chains)):
        if a != b:
            print(f"  chain 差异 [{i}]: HEAD={a}")
            print(f"                    CUR ={b}")
            break
