# check_vi56.py —— vi=56 归属哪个噪声 + chain 的 xz_scale
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(s['noise_router']['final_density'])
g.gen_cpu(s['noise_router']['final_density'])
# vi -> key
vi_to_key = {v: k for k, v in g.normal_vec_index.items()}
for vi in [56, 48, 40, 32, 152, 160]:
    key = vi_to_key.get(vi)
    if key:
        ci = g.normal_chain_index.get(key)
        c = g.coord_chains[ci] if ci is not None and ci < len(g.coord_chains) else None
        print(f'vi={vi} key={key[:45]} chain_xz={c.get("xz_scale") if c else "?"} chain_y={c.get("y_scale") if c else "?"}')
    else:
        print(f'vi={vi} key=?')
