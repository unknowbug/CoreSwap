import json, dfc_gen, sys
name = sys.argv[1]
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(f'{dfdir}\\overworld\\{name}.json'))
g.gen(df)   # 收集噪声实例 + 坐标链
manifest = g.gen_noise_manifest()
print(json.dumps(manifest, indent=1))
