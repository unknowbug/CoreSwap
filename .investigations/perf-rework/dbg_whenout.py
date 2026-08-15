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

nodes = g.df_nodes
# node[122] = RANGE(a1=42, a2=69, a3=?)
n = nodes[122]
print(f'node[122] a1={n["a1"]} a2={n["a2"]} a3={n["a3"]} f0={n["f0"]} f1={n["f1"]}')
# a3 = when_out 子树
a3 = n['a3']
# 递归显示 when_out 子树（找 y 相关节点：YCLAMP/Y/INTERP）
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQ',12:'CUBE',13:'HNEG',14:'QNEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE',18:'YCLAMP',19:'SNOISE',20:'BLEND',21:'FLAT'}
seen = set()
def scan(i, d=0):
    if i < 0 or i in seen or d > 6:
        return
    seen.add(i)
    n = nodes[i]
    extra = ''
    if n['type'] in (18, 1):
        extra = ' <== Y相关!'
    if n['type'] in (2, 19):
        slot = n['a1']
        if slot < len(g.noise_slots):
            extra = ' slot=' + str(g.noise_slots[slot]['key'])[:35]
    if n['type'] == 5:
        extra = ' interp_idx=' + str(n['a1']) + ' <== INTERP!'
    if n['type'] == 4:
        extra = ' spline=' + str(n['a1'])
    if n['type'] == 18:
        extra = ' ycg(' + str(n['f0']) + '..' + str(n['f1']) + ')'
    print('  ' * d + f'node[{i}] {names.get(n["type"],"?")} a1={n["a1"]} a2={n["a2"]} a3={n["a3"]}{extra}')
    for f in ('a1', 'a2', 'a3'):
        if n[f] >= 0:
            scan(n[f], d + 1)
print('--- when_out_of_range 子树（node[122].a3）---')
scan(a3)
