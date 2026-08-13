import json, dfc_gen, sys
name = sys.argv[1] if len(sys.argv) > 1 else 'factor'
out = sys.argv[2] if len(sys.argv) > 2 else f'{name}.comp'
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(f'{dfdir}\\overworld\\{name}.json'))
open(out, 'w', encoding='utf-8').write(g.gen_shader(df))
print(f'{name} shader:', len(g.gen_shader(df)))
