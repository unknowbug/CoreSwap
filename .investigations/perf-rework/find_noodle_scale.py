# find_noodle_scale.py —— noodle_ridge_b 在 vanilla JSON 的 scale
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\caves\noodle.json'))
def find_noise(node, path='root'):
    if isinstance(node, dict):
        if node.get('type') == 'minecraft:noise' and 'noodle' in str(node.get('noise','')):
            print(f'{path}: noise={node.get("noise")} xz_scale={node.get("xz_scale")} y_scale={node.get("y_scale")}')
        for k in ('argument','argument1','argument2','input','when_in_range','when_out_of_range','coordinate','value'):
            if k in node:
                v = node[k]
                if isinstance(v, list):
                    for j, it in enumerate(v): find_noise(it, f'{path}.{k}[{j}]')
                else: find_noise(v, f'{path}.{k}')
find_noise(d)
