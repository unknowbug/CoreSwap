# check_interp_equiv.py —— 关键等价性验证：
# A) sim 的 interp_N 手动重建（8 角点 eval_df_base + 三线性）——GPU interp_N 的等价实现
# B) 我的方案 C 网格（1225 点 eval_df_base corner=0）插值
# 两者都应 = 内容树插值。若不等 → 我对 interp 语义的理解有误
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

N = sim.N
nodes = sim.nodes

def eval_base(interp_idx, corner, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    r = sim.eval_df_base(root, corner, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

# A) interp_N 等价（8 角点 delegate，corner=0..7，坐标 = cell 角）
def interp_N_equiv(interp_idx, sIdx, ix, iy, iz):
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX*16; gy = iy + 64; gz = iz - chunkZ*16
    cx = gx // 4; cy = gy // 8; cz = gz // 4
    fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
    pts = []
    for c in range(8):
        dx, dy, dz = c&1, (c>>1)&1, (c>>2)&1
        ax = chunkX*16 + (cx+dx)*4; ay = -64 + (cy+dy)*8; az = chunkZ*16 + (cz+dz)*4
        v = eval_base(interp_idx, c, sIdx, ax, ay, az)
        pts.append(v)
    d00=pts[0]+(pts[1]-pts[0])*fx; d10=pts[2]+(pts[3]-pts[2])*fx
    d01=pts[4]+(pts[5]-pts[4])*fx; d11=pts[6]+(pts[7]-pts[6])*fx
    d0=d00+(d10-d00)*fy; d1=d01+(d11-d01)*fy
    return d0+(d1-d0)*fz

# B) 方案 C 网格（1225 点 corner=0）+ 插值
def schemeC_interp(interp_idx, sIdx, ix, iy, iz, grid=None):
    chunkX = ix // 16; chunkZ = iz // 16
    gx = ix - chunkX*16; gy = iy + 64; gz = iz - chunkZ*16
    cx = min(gx//4, 3); cy = min(gy//8, 47); cz = min(gz//4, 3)
    fx = (gx % 4)/4.0; fy = (gy % 8)/8.0; fz = (gz % 4)/4.0
    def at(dx, dy, dz):
        if grid is not None:
            return grid[((cy+dy)*5 + (cz+dz))*5 + (cx+dx)]
        # 无网格：直接算角点（corner=0）
        ax = chunkX*16 + (cx+dx)*4; ay = -64 + (cy+dy)*8; az = chunkZ*16 + (cz+dz)*4
        return eval_base(interp_idx, 0, sIdx, ax, ay, az)
    d000=at(0,0,0); d100=at(1,0,0); d010=at(0,1,0); d110=at(1,1,0)
    d001=at(0,0,1); d101=at(1,0,1); d011=at(0,1,1); d111=at(1,1,1)
    d00=d000+(d100-d000)*fx; d10=d010+(d110-d010)*fx
    d01=d001+(d101-d001)*fx; d11=d011+(d111-d011)*fx
    d0=d00+(d10-d00)*fy; d1=d01+(d11-d01)*fy
    return d0+(d1-d0)*fz

pts = [(0,-64,0),(10,-60,0),(20,-50,2),(44,-49,4),(63,-49,2),(5,-55,1),(30,-40,0)]
print('A vs B（interp 8角点delegate vs corner=0 角点）——应一致如果 corner 不影响内容树:')
for interp_idx in range(5):
    print(f'  interp[{interp_idx}]:')
    for (x,y,z) in pts:
        a = interp_N_equiv(interp_idx, 0, x, y, z)
        b = schemeC_interp(interp_idx, 0, x, y, z)
        d = abs(a - b) if a is not None and b is not None else float('nan')
        flag = ' ***' if d > 1e-5 else ''
        print(f'    ({x},{y},{z}) A={a:.6f} B={b:.6f} diff={d:.2e}{flag}')
