# check_spagrough_json.py —— spaghetti_roughness_function.json 的 noise scale
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\caves\spaghetti_roughness_function.json'))
def walk(node, path='root'):
    if isinstance(node, dict):
        if node.get('type') == 'minecraft:noise':
            print(f'{path}: noise={node.get("noise")} xz_scale={node.get("xz_scale")} y_scale={node.get("y_scale")}')
        if node.get('type') == 'minecraft:shifted_noise':
            print(f'{path}: shifted_noise noise={node.get("noise")} xz_scale={node.get("xz_scale")} y_scale={node.get("y_scale")}')
        for k in ('argument','argument1','argument2','input','when_in_range','when_out_of_range','coordinate','value','spline'):
            if k in node:
                v = node[k]
                if isinstance(v, list):
                    for j, it in enumerate(v): walk(it, f'{path}.{k}[{j}]')
                else: walk(v, f'{path}.{k}')
walk(d)
