# check_interp_roots.py —— interp_roots 各 interp 的 delegate root + 闭包组成
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(s['noise_router']['final_density'])
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD_BLENDED',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQUARE',12:'CUBE',13:'HALF_NEG',14:'QUARTER_NEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE_CHOICE',18:'Y_CLAMPED',19:'SHIFTED_NOISE',20:'BLEND_DENSITY',21:'FLAT_CACHE',22:'WEIRD'}
for idx, root in enumerate(g.interp_roots):
    print(f'interp_{idx}: root={root} ({names.get(g.df_nodes[root]["type"], "?")})')
# interp_4 的闭包节点类型分布
closure = set()
def visit(i):
    if i < 0 or i >= len(g.df_nodes) or i in closure: return
    closure.add(i)
    n = g.df_nodes[i]
    if n['type'] == 22: visit(n['a1']); return
    visit(n['a1']); visit(n['a2']); visit(n['a3'])
if len(g.interp_roots) > 4:
    visit(g.interp_roots[4])
    print('interp_4 闭包节点:')
    for i in sorted(closure):
        n = g.df_nodes[i]
        print(f'  [{i}] {names.get(n["type"],"?")} a1={n["a1"]} a2={n["a2"]} f0={n["f0"]} f1={n["f1"]}')
