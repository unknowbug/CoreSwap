# hand_calc_factor.py —— 用 CPU 诊断值 + spline 定义，手算 factor 值（double），定位 GPU vs CPU 差异
import json

# vanilla Spline.apply（double）
def spline_apply(spline, coord_fn, pos):
    locs = [p['location'] for p in spline['points']]
    ders = [p['derivative'] for p in spline['points']]
    f = coord_fn(spline['coordinate'], pos)
    # 二分
    lo, hi = 0, len(locs)
    while lo < hi:
        mid = (lo + hi) // 2
        if f < locs[mid]: hi = mid
        else: lo = mid + 1
    i = lo - 1
    def value_at(idx):
        v = spline['points'][idx]['value']
        if isinstance(v, (int, float)): return v
        return spline_apply(v, coord_fn, pos)
    if i < 0:
        return value_at(0) + ders[0] * (f - locs[0])
    if i == len(locs) - 1:
        return value_at(len(locs)-1) + ders[-1] * (f - locs[-1])
    nv = value_at(i); ov = value_at(i+1)
    kd = (f - locs[i]) / (locs[i+1] - locs[i])
    p = ders[i] * (locs[i+1]-locs[i]) - (ov - nv)
    q = -ders[i+1] * (locs[i+1]-locs[i]) + (ov - nv)
    return nv + kd*(ov-nv) + kd*(1-kd)*(p + kd*(q-p))

# CPU 诊断值（DensityBuilder）
cpu_vals = {'minecraft:overworld/continents': -0.034252338,
            'minecraft:overworld/erosion': -0.372102956,
            'minecraft:overworld/ridges': 0.227816648}
def coord_fn(coord, pos):
    return cpu_vals[coord]

# factor 顶层 spline
d = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json'))
node = d
for k in ['argument', 'argument', 'argument2', 'argument2', 'argument2']:
    node = node[k]
s = node['spline']

r = spline_apply(s, coord_fn, None)
print(f"手算 factor(double) = {r:.9f}")
print(f"CPU DensityBuilder = 5.118815835")
print(f"GPU shader = 4.690000057")
