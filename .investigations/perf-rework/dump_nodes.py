# dump_nodes.py —— 打印指定节点字段
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(s['noise_router']['final_density'])
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD_BLENDED',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQUARE',12:'CUBE',13:'HALF_NEG',14:'QUARTER_NEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE_CHOICE',18:'Y_CLAMPED',19:'SHIFTED_NOISE',20:'BLEND_DENSITY',21:'FLAT_CACHE',22:'WEIRD'}
for i in [138, 154, 155, 156, 157, 158]:
    n = g.df_nodes[i]
    print(f'node[{i}] type={n["type"]}({names.get(n["type"],"?")}) a1={n["a1"]} a2={n["a2"]} a3={n["a3"]} f0={n["f0"]} f1={n["f1"]} f2={n["f2"]} f3={n["f3"]}')
