import json, dfc_gen, sys
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
name = sys.argv[1]
g = dfc_gen.DfcGen(dfdir, ndir)
df = json.load(open(f'{dfdir}\\overworld\\{name}.json'))
g.gen(df)
octBase = 0
for i, (kind, p) in enumerate(g.noise_instances):
    n = 40 if kind == 'old_blended' else 2 * len(p.get('amplitudes', [1.0]))
    print(f'[{i}] kind={kind} octBase={octBase} nOct={n} noise={p.get("noise","")} fo={p.get("firstOctave","")} amps={p.get("amplitudes","")}')
    octBase += n
