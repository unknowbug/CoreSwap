import json, dfc_gen, sys
name = sys.argv[1]
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(f'{dfdir}\\overworld\\{name}.json'))
g.gen(df)
open('cpu_backend.h', 'w', encoding='utf-8').write(g.gen_cpu())
open(f'{name}.comp', 'w', encoding='utf-8').write(g.gen_shader(df))
print(f'{name}: shader {len(g.gen_shader(df))} bytes, splitTotal={g.split_total}, normals={len(g.noise_instances)}')
