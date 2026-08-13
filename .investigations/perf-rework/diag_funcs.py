import json, dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen(fd)
g.gen_shader(fd)
print('=== 函数分布 ===')
print('noise_instances:', len(g.noise_instances), '(normal', sum(1 for k,_ in g.noise_instances if k=='normal'), '+ old_blended', sum(1 for k,_ in g.noise_instances if k=='old_blended'), ')')
print('registry_defs:', len(g.registry_defs))
for fname, fexpr in g.registry_defs:
    print('  ', fname, 'expr len', len(fexpr))
print('spline_funcs:', len(g.spline_funcs))
print('interp_funcs:', len(g.interp_funcs))
# registry 函数里，哪些是"顶层独立"（不依赖其他 registry）
print('registry 依赖（expr 里的 df_ 调用）:')
import re
for fname, fexpr in g.registry_defs:
    deps = sorted(set(re.findall(r'df_(\w+)\(', fexpr)))
    print('  ', fname, '-> deps:', deps[:6] if deps else '无')
