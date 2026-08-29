import sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SIM_DIR = r'E:\PYTHON\CoreSwap\.investigations\perf-rework'
sys.path.insert(0, SIM_DIR)
import dbg_full_sim as sim
g = sim.g; eval_df_base = sim.eval_df_base
rep = {}
for sIdx,(x,y,z) in enumerate(sim.coords):
    c = (x//16, z//16, (x - (x//16)*16)//4, (y+64)//8, (z - (z//16)*16)//4)
    if c not in rep: rep[c] = (sIdx, x, y, z)
def corners(chunkX, chunkZ, cx, cy, cz):
    out=[]
    for c in range(8):
        dx,dy,dz = c&1,(c>>1)&1,(c>>2)&1
        out.append((chunkX*16+(cx+dx)*4, -64+(cy+dy)*8, chunkZ*16+(cz+dz)*4))
    return out
# scan: find a grid node whose value varies across its covering cells (position-variant)
for interp_idx, root in enumerate(g.interp_roots):
    found=False
    for cell,(sIdx,x,y,z) in rep.items():
        chX, chZ, cx, cy, cz = cell
        for c, (ax,ay,az) in enumerate(corners(chX,chZ,cx,cy,cz)):
            v = eval_df_base(root, c, sIdx, ax, ay, az)
            # wrong cell: cx+1 (same chunk) if exists, its corner0 split at x+4
            wcell = (chX, chZ, cx+1, cy, cz)
            if wcell in rep:
                wsIdx = rep[wcell][0]
                vw = eval_df_base(root, 0, wsIdx, ax, ay, az)
                if not isinstance(v, tuple) and not isinstance(vw, tuple) and abs(v-vw) > 1e-9:
                    print(f"interp_{interp_idx} root={root} N=({ax},{ay},{az}) good={v:.6f} "
                          f"wrongcell_split@{wcell} vw={vw:.6f} diff={abs(v-vw):.3e}")
                    found=True
                    break
        if found: break
    if not found:
        print(f"interp_{interp_idx} root={root}: 未找到位置相关节点（域内函数似位置不变）")
