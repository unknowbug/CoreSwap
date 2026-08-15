# verify_p11_path.py —— P1-1 针对性验证：normal-range 父帧的 v0/v1 子帧为边界嵌套帧
# judge 指出：dbg_full_sim.py 旧版 stage 6/7 完成路径回填时 stageStack[ps>>1]=2 覆盖父帧 stage，
# 该路径未被现有验证覆盖（顶层边界 + e2e 无边界域均不触发）。
# 本脚本：① 扫描 spline 结构找「normal-range 父帧 v0/v1 嵌套 + 子帧边界可达」组合；
#         ② 对候选组合跑 spline_eval_py，断言 stage 6/7 完成回填时父帧 stage 未被错误覆盖。
import sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

svk = sim.svk; svn = sim.svn; svf = sim.svf
snodes = sim.snodes; slocs = sim.slocs; sders = sim.sders

# ① 扫描：每个 spline 节点，normal-range 内 v[i]（i in 0..n-2）是嵌套（svk==1）的子节点
#    ——这些子节点若自身 coord 边界可达，即构成 P1-1 路径候选
candidates = []
for node in range(len(snodes)):
    nd = snodes[node]
    n = nd['n']; vb = nd['valBegin']; cb = nd['coordType']
    for i in range(n - 1):   # normal-range 槽位（v0 侧 i, v1 侧 i+1）
        for side, slot in (('v0', i), ('v1', i + 1)):
            if svk[vb + slot] == 1:
                child = svn[vb + slot]
                candidates.append((node, side, slot, child))
print(f'P1-1 候选组合（父帧 normal-range 槽位嵌套子帧）: {len(candidates)}')
for c in candidates[:20]:
    print('  parent=%d %s slot=%d child=%d' % c)

# ② 对每个候选子帧，检查其边界可达性：child 的 coord 由 coordType+corner+sIdx+坐标决定
#    用固定 corner=0 扫描多个 sIdx 与坐标组合，触发 child 的 i<0 或 i>=n-1
hits = []
import itertools
for (parent, side, slot, child) in candidates:
    nd = snodes[child]
    n = nd['n']; lb = nd['locBegin']
    lo = slocs[lb]; hi = slocs[lb + n - 1]
    # 用 sim 的 spline_coord_py 计算 child 的 coord（corner=0, sIdx=0..32, 几组坐标）
    for sIdx in range(0, 33, 8):
        for (ix, iy, iz) in ((0, -64, 0), (784, 160, -408), (720, 160, -432), (832, 160, -416)):
            try:
                coord = sim.spline_coord_py(nd['coordType'], 0, sIdx, ix, iy, iz)
            except Exception:
                continue
            if coord < lo or coord > hi:
                hits.append((parent, side, slot, child, sIdx, ix, iy, iz, coord, lo, hi))
                break
        else:
            continue
        break
print(f'P1-1 候选且子帧边界可达: {len(hits)}')
for h in hits[:10]:
    print('  parent=%d %s child=%d sIdx=%d pos=(%d,%d,%d) coord=%.4f loc=[%.4f,%.4f]' % (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8], h[9]))

# ③ 对边界可达组合跑 spline_eval_py，验证结果非 0 外推（应递归求值）
print('--- 逐候选跑 spline_eval_py（corner=0）---')
for (parent, side, slot, child, sIdx, ix, iy, iz, coord, lo, hi) in hits[:6]:
    r = sim.spline_eval_py(child, 0, sIdx, ix, iy, iz)
    status = 'OK' if not (isinstance(r, tuple) and isinstance(r[0], str)) else 'ERR'
    print(f'  child={child} pos=({ix},{iy},{iz}) coord={coord:.4f} (边界外: {coord<lo or coord>hi}) -> {r} [{status}]')
