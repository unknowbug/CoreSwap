# check_entrances_json.py —— entrances.json 结构（找 spline + coordinate）
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\caves\entrances.json'))
def walk(node, path='root', depth=0):
    if depth > 8: return
    if isinstance(node, dict):
        t = node.get('type', '')
        brief = ''
        if t == 'minecraft:spline' or ('points' in node and 'coordinate' in node):
            brief = f" points={len(node.get('points', []))} coord_type={node.get('coordinate',{}).get('type','?') if isinstance(node.get('coordinate'),dict) else '?'}"
        if t in ('minecraft:min','minecraft:max','minecraft:range_choice','minecraft:mul','minecraft:add'):
            brief = f" args={[k for k in ('argument','argument1','argument2','input','when_in_range','when_out_of_range') if k in node]}"
        print(f'{"  "*depth}{t}{brief}')
        for k in ('argument','argument1','argument2','input','when_in_range','when_out_of_range','coordinate','value'):
            if k in node:
                v = node[k]
                if isinstance(v, list):
                    for j, it in enumerate(v): walk(it, f'{path}.{k}[{j}]', depth+1)
                else: walk(v, f'{path}.{k}', depth+1)
walk(d)
