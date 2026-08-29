# verif_grid_trilinear.py — 验证「grid 缓存」的三线性逻辑 == 现有 interp_N（=production）
# 前提：grid 节点值唯一（verif_grid_cache_correctness.md 已证）。本验证证明：
#   「预先对 5×49×5 网格节点各算 eval_df_base 存 grid，再 sampleInterpGrid 三线性」
#   == 「interp_N 每点重算 8 角点 + 三线性」（=production InterpolatedDF）
# 若相等 → grid 缓存的「三线性逻辑」正确（无需 split 翻转即可先验证 grid 结构）。
import struct, json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_df(fd)

DFAY = range(23)  # placeholder
nodes = g.df_nodes
interp_roots = g.interp_roots
minY = -64

# 引用 dbg_full_sim 的 eval_df_base / NOISE_SLOT_BASE etc.
base_path = r'E:\PYTHON\CoreSwap\.investigations\perf-rework'
simmod = importlib.util.spec_from_file_location('sim', base_path + r'\dbg_full_sim.py')
sim = importlib.util.module_from_spec(simmod); simmod.loader.exec_module(sim)
eval_df_base = sim.eval_df_base
NOISE_SLOT_BASE = sim.NOISE_SLOT_BASE
NOISE_SLOT_STRIDE = sim.NOISE_SLOT_STRIDE

print("interp_roots =", interp_roots)
print("df_nodes =", len(nodes))

# ---- grid 缓存逻辑验证 ----
# 对 chunk (0,0)，grid 节点 = chunkX*16+gx*4, minY+gy*8, chunkZ*16+gz*4，gx/gz∈[0,5), gy∈[0,49)
# 先把网格节点值预存 grid[gy][gz][gx]，再 sampleInterpGrid 三线性，对比 interp_N 每点重算。
def build_grid(interpIdx, chunkX, chunkZ):
    root = interp_roots[interpIdx]
    GX, GY, GZ = 5, 49, 5
    grid = [[[None]*GX for _ in range(GZ)] for _ in range(GY)]
    for gy in range(GY):
        for gz in range(GZ):
            for gx in range(GX):
                nx = chunkX*16 + gx*4
                ny = minY + gy*8
                nz = chunkZ*16 + gz*4
                # grid 节点作为「该 cell 的角点」求值：cell=(chunkX, gx//4, gy//8, chunkZ, gz//4)
                cx, cy, cz = gx//4, gy//8, gz//4
                # 该节点的 corner 索引 = 相对 cell 的 (gx%4, gy%8, gz%4) → dx,dy,dz
                dx, dy, dz = gx - cx*4, gy - cy*8, gz - cz*4
                # 映射到 corner bit：dx∈{0,1}, dy∈{0,1}, dz∈{0,1}（cell 4x8x4 内，grid 节点恰在角点）
                corner = (dx & 1) | ((dy & 1) << 1) | ((dz & 1) << 2)
                sIdx = 0  # 单点场景；splitCoord 用 sIdx=0（此处 sim 读 sIdx*SPLIT_TOTAL）
                # 注意：sIdx 需要是该 cell 的一个「采样点」的 sIdx，但 sim 全局 splitCoord 是对每个采样点。
                # 这里用 sIdx=0 + 节点真实坐标（nx,ny,nz）—— sim eval_df_base 用 sIdx 读 split + iy 实参。
                grid[gy][gz][gx] = eval_df_base(root, corner, sIdx, nx, ny, nz)
    return grid

def sample_grid(interpIdx, grid, ix, iy, iz, chunkX, chunkZ):
    gx = ix - chunkX*16; gy = iy - minY; gz = iz - chunkZ*16
    cx, cy, cz = gx//4, gy//8, gz//4
    fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
    g = lambda dx,dy,dz: grid[cy+dy][cz+dz][cx+dx]
    d000=g(0,0,0); d100=g(1,0,0); d010=g(0,1,0); d110=g(1,1,0)
    d001=g(0,0,1); d101=g(1,0,1); d011=g(0,1,1); d111=g(1,1,1)
    d00=d000+(d100-d000)*fx; d10=d010+(d110-d010)*fx
    d01=d001+(d101-d001)*fx; d11=d011+(d111-d011)*fx
    d0=d00+(d10-d00)*fy; d1=d01+(d11-d01)*fy
    return d0+(d1-d0)*fz

print("\n=== grid 缓存三线性 vs interp_N 每点重算 ===")
for interpIdx in range(1):  # 先测 interp_0
    root = interp_roots[interpIdx]
    print(f"\n-- interp_{interpIdx} (root={root}) chunk(0,0) --")
    grid = build_grid(interpIdx, 0, 0)
    # 取 chunk 内 8 个代表性 block（不同 cell / 不同 frac），对比 grid-三线性 vs interp_N 每点重算
    maxdiff = 0.0
    for (bx, by, bz) in [(0,-64,0),(2,-60,2),(4,-56,0),(7,-52,3),(12,-48,2),(15,-44,1)]:
        v_grid = sample_grid(interpIdx, grid, bx, by, bz, 0, 0)
        v_interp = sim.interp_N(interpIdx, 0, bx, by, bz)  # sIdx=0
        d = abs(v_grid - v_interp)
        if d > maxdiff: maxdiff = d
        print(f"  block({bx},{by},{bz}) grid={v_grid:.9f} interpN={v_interp:.9f} diff={d:.3e}")
    print(f"  maxdiff = {maxdiff:.3e}")
