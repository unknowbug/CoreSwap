# hand_calc_factor2.py —— 详细中间值，对比手算 vs GPU
import json

def spline_apply(spline, coord_fn, pos, depth=0, label=''):
    locs = [p['location'] for p in spline['points']]
    ders = [p['derivative'] for p in spline['points']]
    f = coord_fn(spline['coordinate'], pos)
    lo, hi = 0, len(locs)
    while lo < hi:
        mid = (lo + hi) // 2
        if f < locs[mid]: hi = mid
        else: lo = mid + 1
    i = lo - 1
    ind = '  ' * depth
    print(f"{ind}[{label}] coord={spline['coordinate']} f={f:.9f} 区间 i={i} locs[{i}]={locs[i] if 0<=i<len(locs) else 'N/A'}")
    def value_at(idx):
        v = spline['points'][idx]['value']
        if isinstance(v, (int, float)): return v
        return spline_apply(v, coord_fn, pos, depth+1, f'value[{idx}]')
    if i < 0:
        return value_at(0) + ders[0] * (f - locs[0])
    if i == len(locs) - 1:
        return value_at(len(locs)-1) + ders[-1] * (f - locs[-1])
    nv = value_at(i); ov = value_at(i+1)
    kd = (f - locs[i]) / (locs[i+1] - locs[i])
    p = ders[i] * (locs[i+1]-locs[i]) - (ov - nv)
    q = -ders[i+1] * (locs[i+1]-locs[i]) + (ov - nv)
    r = nv + kd*(ov-nv) + kd*(1-kd)*(p + kd*(q-p))
    print(f"{ind}   nv={nv:.9f} ov={ov:.9f} kd={kd:.9f} result={r:.9f}")
    return r

cpu_vals = {'minecraft:overworld/continents': -0.034252338,
            'minecraft:overworld/erosion': -0.372102956,
            'minecraft:overworld/ridges': 0.227816648}
def coord_fn(coord, pos):
    return cpu_vals[coord]

d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json'))
node = d
for k in ['argument', 'argument', 'argument2', 'argument2', 'argument2']:
    node = node[k]
s = node['spline']
r = spline_apply(s, coord_fn, None, 0, 'factor')
print(f"\n手算 factor = {r:.9f}")
