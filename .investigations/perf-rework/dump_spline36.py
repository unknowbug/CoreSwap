# dump_spline36.py —— spline 36/55 的完整数据 + coord 计算
import sys, os, json, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.SPLIT_TOTAL = 8672
g = sim.g
for i in [36, 55]:
    nd = g.spline_ssbo_nodes[i]
    ct, n, lb, db, vb = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
    print(f'spline[{i}]: coordType={ct} n={n} locs={g.spline_ssbo_locs[lb:lb+n]} ders={g.spline_ssbo_ders[db:db+n]}')
    print(f'  valBegin={vb} kind={g.spline_ssbo_val_kind[vb:vb+n]} f={g.spline_ssbo_val_f[vb:vb+n]} node={g.spline_ssbo_val_node[vb:vb+n]}')
# coord（corner0）
for i in [36, 55]:
    nd = g.spline_ssbo_nodes[i]
    ct = nd['coordType']
    try:
        coord = sim.spline_coord_py(ct, 0, 0, 784, 160, -408)
        print(f'spline[{i}] coordType={ct} coord(corner0, 784,160,-408) = {coord}')
    except Exception as e:
        print(f'spline[{i}] coord err: {e}')
