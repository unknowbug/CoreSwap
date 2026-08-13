import json, dfc_gen, sys
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen(fd)
open('cpu_backend.h', 'w', encoding='utf-8').write(g.gen_cpu(fd))
open('final_density.comp', 'w', encoding='utf-8').write(g.gen_shader(fd))
print('final_density: noise_instances =', len(g.noise_instances), 'split_total =', g.split_total,
      'interp =', len(g.interp_funcs), 'spline =', len(g.spline_funcs))
