import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
fac = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json', encoding='utf-8'))
sc = fac['argument']['argument']['argument2']['argument2']['argument2']['spline']
print('continents spline:')
for p in sc['points']:
    v = p['value']
    if isinstance(v, dict):
        print('  loc=%s -> %s' % (p['location'], v.get('coordinate')))
    else:
        print('  loc=%s -> const %s' % (p['location'], v))
    if isinstance(v, dict):
        print('    points:')
        for q in v['points']:
            qv = q['value']
            if isinstance(qv, dict):
                print('      loc=%s -> nested %s' % (q['location'], qv.get('coordinate')))
            else:
                print('      loc=%s -> const %s' % (q['location'], qv))
