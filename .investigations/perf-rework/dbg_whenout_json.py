import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings['noise_router']['final_density']
# 找到 range_choice(sloped_cheese, 1.5625) 的 when_out_of_range
def find_rc(x, path='fd'):
    if isinstance(x, dict):
        if x.get('type') == 'minecraft:range_choice' and x.get('max_exclusive') == 1.5625:
            print('FOUND at', path)
            print(json.dumps(x, indent=1)[:100])
            return x
        for k, v in x.items():
            r = find_rc(v, path + '.' + k)
            if r: return r
    elif isinstance(x, list):
        for i, v in enumerate(x):
            r = find_rc(v, path + '[%d]' % i)
            if r: return r
    return None
rc = find_rc(fd)
# 展开 when_out 的引用（entrances/spaghetti 等）
def expand(x, ind=0, depth=0):
    if depth > 7: return
    if isinstance(x, dict):
        t = x.get('type', '?')
        args = {k: v for k, v in x.items() if k != 'type' and not isinstance(v, dict) and not isinstance(v, list)}
        print('  ' * ind + t + ' ' + str(args))
        for k in ('argument', 'argument1', 'argument2', 'input', 'when_in_range', 'when_out_of_range', 'spline'):
            if k in x:
                v = x[k]
                if isinstance(v, str):
                    print('  ' * (ind + 1) + 'REF ' + v)
                    # 展开引用文件
                    import os
                    if v.startswith('minecraft:overworld/'):
                        fn = v[len('minecraft:overworld/'):] + '.json'
                        p = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\%s' % fn
                        if os.path.exists(p):
                            sub = json.load(open(p, encoding='utf-8'))
                            expand(sub, ind + 1, depth + 1)
                else:
                    expand(v, ind + 1, depth + 1)
print('===== when_out_of_range 完整展开 =====')
expand(rc['when_out_of_range'])
