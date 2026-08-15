import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)

# GPU 侧 OLD 参数（gen_shader 分配）
print('GPU OLD (old_meta):')
for mm in g.old_meta:
    print(f'  idx={mm["idx"]} octBase={mm["octBase"]} splitBase={mm["splitBase"]}')

# CPU 侧（gen_cpu 的 split 布局）——模拟 gen_cpu 的分配
# 先 _reset_collect + gen_df（gen_cpu 用 gen_df 收集）
g2 = m.DfcGen(dfdir, ndir)
g2.gen_df(fd)
manifest = g2.gen_noise_manifest()
print('CPU manifest split 布局:')
# gen_cpu 遍历 noise_instances 分配（同 gen_shader）
ob = 0; sb = 0
for idx, (kind, p) in enumerate(g2.noise_instances):
    if kind == 'old_blended':
        print(f'  instance[{idx}] old: octBase={ob} splitBase={sb}')
        ob += 40; sb += 7*40
    else:
        n = len(p.get('amplitudes', [1.0]))
        ob += 2*n; sb += 6*2*n
print(f'CPU split_total = {sb} (GPU split_total = {g.split_total})')
