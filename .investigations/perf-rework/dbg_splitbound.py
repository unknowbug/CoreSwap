import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)
sb = 0; mx = 0; mx_key = ''
over = []
for idx, (kind, p) in enumerate(g.noise_instances):
    if kind == 'old_blended':
        end = sb + 7 * 40
    else:
        n = len(p.get('amplitudes', [1.0]))
        end = sb + 6 * 2 * n
    if end > 8192:
        over.append((p.get('_key'), sb, end))
    if end > mx:
        mx, mx_key = end, p.get('_key')
    sb = end
print(f'最后实例结束偏移: {sb} (SPLIT_TOTAL=8192) {"OK" if sb == 8192 else "MISMATCH"}')
print(f'越界实例: {over if over else "无"}')
print(f'max end={mx} key={mx_key}')
