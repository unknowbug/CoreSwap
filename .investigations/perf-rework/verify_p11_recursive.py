# verify_p11_recursive.py —— P1-1 终极判别：显式栈 spline_eval_py vs 递归版 Spline.apply 参照
# 对 53 个「边界可达嵌套子帧」候选 + 全 spline 节点 × 多坐标，逐节点对拍。
# 递归版 = vanilla Spline.apply 语义的直译（边界外推 = value[0]+der[0]*(x-loc[0]) 递归求值）。
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

svk = sim.svk; svn = sim.svn; svf = sim.svf
snodes = sim.snodes; slocs = sim.slocs; sders = sim.sders
scoords = sim.scoords

def spline_coord(coordType, corner, sIdx, ix, iy, iz):
    expr = scoords[coordType]
    import re
    slots = [int(x) for x in re.findall(r'NOISE_SLOT_BASE\[(\d+)\]', expr)]
    if not slots:
        return 0.0
    s = sim.g.noise_slots[slots[0]]
    n = sim.normal_noise(s['base'] + corner * s['stride'], sIdx)
    if 'abs(' in expr:
        return -3.0 * (-1.0/3.0 + abs(-2.0/3.0 + abs(n)))
    return n

def spline_eval_recursive(node, corner, sIdx, ix, iy, iz, depth=0):
    """递归版 = vanilla Spline.apply 直译。返回 (value, depth)。"""
    if depth > 32:
        return (None, depth)
    nd = snodes[node]
    n, lb, db, vb = nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
    coord = spline_coord(nd['coordType'], corner, sIdx, ix, iy, iz)
    # 二分找区间
    mn = 0; i = n
    while i > 0:
        j = i // 2; k = mn + j
        if coord < slocs[lb + k]:
            i = j
        else:
            mn = k + 1; i -= j + 1
    i = mn - 1
    def val_at(slot):
        if svk[vb + slot] == 0:
            return svf[vb + slot]
        v, d = spline_eval_recursive(svn[vb + slot], corner, sIdx, ix, iy, iz, depth + 1)
        return v
    if i < 0:
        return (val_at(0) + sders[db] * (coord - slocs[lb]), depth)
    if i >= n - 1:
        return (val_at(n - 1) + sders[db + n - 1] * (coord - slocs[lb + n - 1]), depth)
    nv = val_at(i); ov = val_at(i + 1)
    span = slocs[lb + i + 1] - slocs[lb + i]
    kd = (coord - slocs[lb + i]) / span
    p = sders[db + i] * span - (ov - nv)
    q = -sders[db + i + 1] * span + (ov - nv)
    h = (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p))
    return (h, depth)

# 对拍：全部 spline 节点 × 多坐标（含边界触发域）
points = [(0, -64, 0), (784, 160, -408), (720, 160, -432), (832, 160, -416),
          (63, -49, 2), (44, -49, 4), (1, -63, 0), (816, 160, -336)]
mism = 0; total = 0; boundary_hit = 0
for node in range(len(snodes)):
    for (ix, iy, iz) in points:
        for corner in (0,):
            for sIdx in range(0, 33, 16):
                total += 1
                sv = sim.spline_eval_py(node, corner, sIdx, ix, iy, iz)
                rv, _ = spline_eval_recursive(node, corner, sIdx, ix, iy, iz)
                if isinstance(sv, tuple) and isinstance(sv[0], str):
                    print('STACK-ERR node=%d pos=(%d,%d,%d) sIdx=%d -> %s' % (node, ix, iy, iz, sIdx, sv))
                    mism += 1
                    continue
                if rv is None:
                    print('REC-DEPTH node=%d pos=(%d,%d,%d)' % (node, ix, iy, iz))
                    continue
                # 判定该点是否触发边界（栈内 stage 6/7 计数）
                d = abs(sv[0] - rv)
                if d > 1e-9:
                    mism += 1
                    if mism <= 15:
                        print('MISMATCH node=%d pos=(%d,%d,%d) sIdx=%d stack=%.9f rec=%.9f diff=%.2e' %
                              (node, ix, iy, iz, sIdx, sv[0], rv, d))
print(f'total={total} mismatch={mism}')
