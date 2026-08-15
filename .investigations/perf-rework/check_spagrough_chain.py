# check_spagrough_chain.py —— spagrough 实例 56 的 chain
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(s['noise_router']['final_density'])
for i in [56, 0, 32, 152, 160, 168]:
    kind, p = g.noise_instances[i]
    key = p.get('_key', '')
    ci = g.normal_chain_index.get(key)
    c = g.coord_chains[ci] if ci is not None and ci < len(g.coord_chains) else None
    if c:
        print(f'[{i}] {key[:45]} chain={c.get("type")} xz={c.get("xz_scale")} y={c.get("y_scale")} flat={c.get("flat_cache")} shift_x={c.get("shift_x",{}).get("type","")}')
    else:
        print(f'[{i}] {key[:45]} chain=MISSING ci={ci}')
