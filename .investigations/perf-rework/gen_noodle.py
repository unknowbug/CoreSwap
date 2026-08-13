import json, dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(dfdir + r'\overworld\caves\noodle.json'))
g.gen(df)
open('cpu_backend.h', 'w', encoding='utf-8').write(g.gen_cpu(df))
open('noodle.comp', 'w', encoding='utf-8').write(g.gen_shader(df))
print('noodle: noise_instances =', len(g.noise_instances), 'split_total =', g.split_total, 'interp =', len(g.interp_funcs))
