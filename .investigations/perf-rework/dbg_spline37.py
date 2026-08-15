import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dbg_full_sim as S

# spline_node=37 的 SSBO 数据
nd = g.spline_ssbo_nodes[37]
print(f'spline_ssbo_nodes[37]: {nd}')
coordType = nd['coordType']
print(f'  coordType={coordType} expr={g.spline_coords[coordType][:80]}')
n = nd['n']
lb, db, vb = nd['locBegin'], nd['derBegin'], nd['valBegin']
print(f'  n={n} locs={[round(x,4) for x in g.spline_ssbo_locs[lb:lb+n]]}')
print(f'  ders={[round(x,4) for x in g.spline_ssbo_ders[db:db+n]]}')
print(f'  val_kind={g.spline_ssbo_val_kind[vb:vb+n]} val_f={[round(x,4) for x in g.spline_ssbo_val_f[vb:vb+n]]}')
# 检查 span
locs = g.spline_ssbo_locs[lb:lb+n]
spans = [locs[i+1]-locs[i] for i in range(n-1)]
print(f'  spans={[round(x,6) for x in spans]}  零 span: {any(x==0 for x in spans)}')
# 模拟 spline_eval(37) 值（角点 y=-64 对齐坐标）
sv = S.spline_eval_py(37, 0, 0, 0, -64, 0)
print(f'spline_eval_py(37, corner=0, (0,-64,0)) = {sv}')
# coord 值
coord = S.spline_coord_py(coordType, 0, 0, 0, -64, 0)
print(f'spline_coord_py(coordType={coordType}) = {coord}')
