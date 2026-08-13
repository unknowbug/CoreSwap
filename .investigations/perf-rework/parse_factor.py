import json
d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json'))
node = d
for k in ['argument', 'argument', 'argument2', 'argument2', 'argument2']:
    node = node[k]
s = node['spline']
# factor 顶层 spline 的 location=-0.1 的 value（erosion spline）的 location=-0.5/-0.35 的 value（ridges spline）
for ploc in [-0.1, 0.03]:
    for p in s['points']:
        if p['location'] == ploc:
            erosion_spline = p['value']
            print(f"\nfactor location={ploc} 的 erosion spline:")
            for q in erosion_spline['points']:
                v = q['value']
                if isinstance(v, dict) and 'points' in v:
                    locs = [r['location'] for r in v['points']]
                    vals = [r['value'] for r in v['points']]
                    print(f"  erosion loc={q['location']} -> ridges locs={locs} vals={vals}")
                else:
                    print(f"  erosion loc={q['location']} -> {v}")
