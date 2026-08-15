# check_meta_vs_splitbase.py —— normal_meta[idx].splitBase vs normal_split_base[key]
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
g.gen_shader(fd)
# normal_meta 全量 idx -> splitBase
meta_by_idx = {m["idx"]: m for m in g.normal_meta}
for i in [0, 64, 80, 152, 160, 168, 176, 184, 192]:
    m = meta_by_idx.get(i)
    if m:
        # 找对应 key
        kind, p = g.noise_instances[i]
        key = p.get("_key", "?")
        print(f'实例{i} key={key}: meta.splitBase={m["splitBase"]} normal_split_base={g.normal_split_base.get(key)} 一致={"YES" if m["splitBase"]==g.normal_split_base.get(key) else "NO"}')
    else:
        kind, p = g.noise_instances[i]
        print(f'实例{i} key={p.get("_key","?")}: meta=MISSING（old_blended 占位?）')
