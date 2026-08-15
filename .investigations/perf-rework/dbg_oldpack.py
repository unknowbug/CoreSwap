import json, importlib.util, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)

# old_blended 实例在 noise_instances 的位置
print('noise_instances:')
for i, (kind, p) in enumerate(g.noise_instances):
    if kind == 'old_blended':
        print(f'  [{i}] {kind} key={p["_key"][:60]}')
print(f'old_blended 实例数: {sum(1 for k, _ in g.noise_instances if k == "old_blended")}')
# old_blended slot
for i, s in enumerate(g.noise_slots):
    if s['kind'] == 'old_blended':
        print(f'  slot[{i}] base={s["base"]} stride={s["stride"]} key={s["key"][:50]}')
# OLD_PACK 生成（_old_blended_glsl 的输出）
ob = g._old_blended_glsl()
if ob:
    mm = re.search(r'const int OLD_PACK\[(\d+)\] = int\[\]\((.*?)\);', ob, re.S)
    if mm:
        vals = [int(x) for x in mm.group(2).split(',')]
        print(f'OLD_PACK 长度 {mm.group(1)}, 前 12 值: {vals[:12]}')
        print(f'OLD_PACK 组数: {len(vals)//2}')
