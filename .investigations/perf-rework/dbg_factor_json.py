import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
fac = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json', encoding='utf-8'))
def walk(x, ind=0, depth=0):
    if isinstance(x, dict):
        t = x.get('type', '?')
        args = {k: v for k, v in x.items() if k != 'type' and not isinstance(v, dict) and not isinstance(v, list)}
        print('  ' * ind + t + ' ' + str(args))
        for k in ('argument', 'argument1', 'argument2', 'input', 'when_in_range', 'when_out_of_range', 'spline'):
            if k in x:
                if k == 'spline':
                    print('  ' * (ind + 1) + 'spline coord=%s' % x[k].get('coordinate'))
                    for p in x[k].get('points', []):
                        v = p.get('value')
                        vs = v.get('type') if isinstance(v, dict) else ('const %.3f' % v)
                        print('  ' * (ind + 2) + 'loc=%s val=%s der=%s' % (p.get('location'), vs, p.get('derivative')))
                        if isinstance(v, dict):
                            walk(v, ind + 2, depth + 1)
                else:
                    walk(x[k], ind + 1, depth + 1)
walk(fac)
