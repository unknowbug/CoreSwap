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
def show(i, d=0):
    if i < 0 or d > 3:
        return
    n = nodes[i]
    extra = ''
    if n['type'] in (2, 19):
        slot = n['a1']
        if slot < len(g.noise_slots):
            extra = ' slot->' + str(g.noise_slots[slot]['key'])[:40]
    if n['type'] == 4:
        extra = ' spline_node=' + str(n['a1']) + ' aligned=' + str(n['a2'])
    print('  ' * d + f'node[{i}] type={n["type"]}({names.get(n["type"],"?")}) a1={n["a1"]} a2={n["a2"]}{extra}')
    for f in ('a1', 'a2', 'a3'):
        if n[f] >= 0 and n[f] < i:
            show(n[f], d + 1)
show(36)
