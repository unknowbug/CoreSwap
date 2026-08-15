# find_noodle_ref.py —— 定位 noodle_ridge_b 在 DF 树的引用路径（gen_df 收集到但 _gen_split_lines 可能漏）
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)

# 找 noodle_ridge_b 实例的注册来源：noise_instances 192-199 的 key
for i in range(192, 200):
    kind, p = g.noise_instances[i]
    print(f'[{i}] {kind} key={p.get("_key","")}')

# 找 DF 树中引用 noodle_ridge_b 的位置：递归搜索 json
def find_refs(node, path="root", depth=0):
    if isinstance(node, dict):
        t = node.get("type", "")
        nz = node.get("noise", "")
        if "noodle" in str(nz):
            print(f'  {path} type={t} noise={nz}')
        for k in ("argument","argument1","argument2","input","when_in_range","when_out_of_range","coordinate","value","spline","points"):
            if k in node:
                v = node[k]
                if isinstance(v, list):
                    for j, item in enumerate(v):
                        find_refs(item, f"{path}.{k}[{j}]", depth+1)
                else:
                    find_refs(v, f"{path}.{k}", depth+1)
    elif isinstance(node, list):
        for j, item in enumerate(node):
            find_refs(item, f"{path}[{j}]", depth+1)

print("=== noodle 引用路径（final_density 树） ===")
find_refs(fd)
print("=== sloped_cheese.json 引用 noodle？ ===")
sc = json.load(open(rf'{dfdir}\overworld\sloped_cheese.json'))
find_refs(sc)
print("=== caves/noodle.json ===")
try:
    nn = json.load(open(rf'{dfdir}\overworld\caves\noodle.json'))
    find_refs(nn)
except Exception as e:
    print('err', e)
print("=== caves/entrances.json ===")
try:
    en = json.load(open(rf'{dfdir}\overworld\caves\entrances.json'))
    find_refs(en)
except Exception as e:
    print('err', e)
