import json, dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(f'{dfdir}\\overworld\\factor.json'))
open('factor.comp', 'w', encoding='utf-8').write(g.gen_shader(df))
print('factor shader:', len(g.gen_shader(df)))
