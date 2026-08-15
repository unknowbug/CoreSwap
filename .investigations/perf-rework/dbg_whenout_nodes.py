# dump when_out 子树节点完整定义（含 f 参数）+ JSON 对照
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
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQ',12:'CUBE',13:'HNEG',14:'QNEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE',18:'YCLAMP',19:'SNOISE',20:'BLEND',21:'FLAT'}
def slotname(a1):
    s = g.noise_slots[a1]
    return 'slot%d(%s)' % (a1, s['key'][:28])
for i in range(40, 72):
    n = nodes[i]
    extra = ''
    if n['type'] in (2, 19): extra = ' ' + slotname(n['a1'])
    if n['type'] == 18: extra = ' ycg(%s..%s f=%s..%s)' % (n['f0'], n['f1'], n['f2'], n['f3'])
    if n['type'] == 4: extra = ' spline=%d' % n['a1']
    print(f'node[{i:3d}] {names.get(n["type"],"?"):6s} a1={n["a1"]:3d} a2={n["a2"]:3d} a3={n["a3"]:3d} f0={n["f0"]} f1={n["f1"]}{extra}')
print('--- 95-121 ---')
for i in range(95, 122):
    n = nodes[i]
    extra = ''
    if n['type'] in (2, 19): extra = ' ' + slotname(n['a1'])
    if n['type'] == 18: extra = ' ycg(%s..%s f=%s..%s)' % (n['f0'], n['f1'], n['f2'], n['f3'])
    print(f'node[{i:3d}] {names.get(n["type"],"?"):6s} a1={n["a1"]:3d} a2={n["a2"]:3d} a3={n["a3"]:3d} f0={n["f0"]} f1={n["f1"]}{extra}')
