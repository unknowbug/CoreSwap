# minimal4 = minimal3 + eval_df + interp_0（8 角点 eval_df_base 调用）→ 定位 interp 调用链
import json, importlib.util, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)
nodes = g.df_nodes
N = len(nodes)

def flit(x):
    s = format(float(x), '.17g')
    if '.' not in s and 'e' not in s and 'E' not in s: s += '.0'
    return s + 'f'

types = ", ".join(str(n["type"]) for n in nodes)
a1s = ", ".join(str(n["a1"]) for n in nodes)
a2s = ", ".join(str(n["a2"]) for n in nodes)
a3s = ", ".join(str(n["a3"]) for n in nodes)
f0s = ", ".join(flit(n["f0"]) for n in nodes)
f1s = ", ".join(flit(n["f1"]) for n in nodes)
f2s = ", ".join(flit(n["f2"]) for n in nodes)
f3s = ", ".join(flit(n["f3"]) for n in nodes)

read_fields = {6: ('a1','a2'), 7: ('a1','a2'), 8: ('a1','a2'), 9: ('a1','a2'),
               10: ('a1',), 11: ('a1',), 12: ('a1',), 13: ('a1',), 14: ('a1',), 15: ('a1',), 16: ('a1',),
               17: ('a1','a2','a3'), 20: ('a1',), 21: ('a1',)}
max_parent = [-1]*N
for i, n in enumerate(nodes):
    if n["type"] in read_fields:
        for f in read_fields[n["type"]]:
            c = n[f]
            if 0 <= c < i: max_parent[c] = max(max_parent[c], i)
slot_of = [-1]*N; val_slots = 0
for i in range(N):
    used = set()
    for j in range(i):
        if slot_of[j] >= 0 and max_parent[j] >= i: used.add(slot_of[j])
    s = 0
    while s in used: s += 1
    slot_of[i] = s
    val_slots = max(val_slots, len(used)+1)
slot_src = ", ".join(str(s) for s in slot_of)

# interp_0 的 delegate_root（从 interp_funcs 解析）
m0 = re.match(r"eval_df_base\((\d+),", g.interp_funcs[0][1][0])
delegate_root = int(m0.group(1))
print(f"interp_0 delegate_root = {delegate_root}")

comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal3.comp', encoding='utf-8').read()
# 替换 main：调 eval_df（含 DF_INTERP → interp_0 → 8 次 eval_df_base）
main_old = """void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_df_base({root}, 0, int(idx), ix, iy, iz);
}}""".replace("{root}", str(N - 1))

main_new = f"""int floorDivP(int a, int b) {{ int q = a / b; return (a % b != 0 && (a < 0) != (b < 0)) ? q - 1 : q; }}
float interp_0(int sIdx, int ix, int iy, int iz) {{
    int chunkX = floorDivP(ix, 16); int chunkZ = floorDivP(iz, 16);
    int gx = ix - chunkX * 16; int gy = iy + 64; int gz = iz - chunkZ * 16;
    int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;
    float fx = float(gx % 4) / 4.0f; float fy = float(gy % 8) / 8.0f; float fz = float(gz % 4) / 4.0f;
    float d000 = eval_df_base({delegate_root}, 0, sIdx, (chunkX * 16 + (cx + 0) * 4), (-64 + (cy + 0) * 8), (chunkZ * 16 + (cz + 0) * 4));
    float d100 = eval_df_base({delegate_root}, 1, sIdx, (chunkX * 16 + (cx + 1) * 4), (-64 + (cy + 0) * 8), (chunkZ * 16 + (cz + 0) * 4));
    float d010 = eval_df_base({delegate_root}, 2, sIdx, (chunkX * 16 + (cx + 0) * 4), (-64 + (cy + 1) * 8), (chunkZ * 16 + (cz + 0) * 4));
    float d110 = eval_df_base({delegate_root}, 3, sIdx, (chunkX * 16 + (cx + 1) * 4), (-64 + (cy + 1) * 8), (chunkZ * 16 + (cz + 0) * 4));
    float d001 = eval_df_base({delegate_root}, 4, sIdx, (chunkX * 16 + (cx + 0) * 4), (-64 + (cy + 0) * 8), (chunkZ * 16 + (cz + 1) * 4));
    float d101 = eval_df_base({delegate_root}, 5, sIdx, (chunkX * 16 + (cx + 1) * 4), (-64 + (cy + 0) * 8), (chunkZ * 16 + (cz + 1) * 4));
    float d011 = eval_df_base({delegate_root}, 6, sIdx, (chunkX * 16 + (cx + 0) * 4), (-64 + (cy + 1) * 8), (chunkZ * 16 + (cz + 1) * 4));
    float d111 = eval_df_base({delegate_root}, 7, sIdx, (chunkX * 16 + (cx + 1) * 4), (-64 + (cy + 1) * 8), (chunkZ * 16 + (cz + 1) * 4));
    float d00 = d000 + (d100 - d000) * fx; float d10 = d010 + (d110 - d010) * fx;
    float d01 = d001 + (d101 - d001) * fx; float d11 = d011 + (d111 - d011) * fx;
    float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;
    return d0 + (d1 - d0) * fz;
}}
float eval_df(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {{
    for (int i = 0; i < DF_NODES; i++) {{
        int t = DF_TYPE[i];
        float r = 0.0;
        if (t == 5) {{ r = interp_0(sIdx, ix, iy, iz); valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[i]] = r; continue; }}
        if (t == 0) r = DF_F0[i];
        else if (t == 1) r = float(iy);
        else if (t == 2 || t == 19) r = normal_noise(DF_A1[i], sIdx);
        else if (t == 3) r = interp_noise(DF_A1[i], sIdx);
        else if (t == 4) r = spline_eval(DF_A1[i], corner, sIdx, ix, iy, iz);
        else if (t == 18) r = ycg(iy, DF_F0[i], DF_F1[i], DF_F2[i], DF_F3[i]);
        else if (t == 10) r = abs(valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]);
        else if (t == 11) {{ float v = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = v * v; }}
        else if (t == 12) {{ float v = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = v * v * v; }}
        else if (t == 13) {{ float v = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = (v > 0.0 ? v : v * 0.5); }}
        else if (t == 14) {{ float v = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = (v > 0.0 ? v : v * 0.25); }}
        else if (t == 15) {{ float v = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; float c = clamp(v, -1.0, 1.0); r = c / 2.0 - c * c * c / 24.0; }}
        else if (t == 16) r = clamp(valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], DF_F0[i], DF_F1[i]);
        else if (t == 17) {{
            float inp = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]];
            r = (inp >= DF_F0[i] && inp < DF_F1[i]) ? valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A2[i]]] : valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A3[i]]];
        }}
        else if (t == 20 || t == 21) r = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]];
        else if (t == 6) r = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]] + valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A2[i]]];
        else if (t == 7) r = valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]] * valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A2[i]]];
        else if (t == 8) r = min(valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A2[i]]]);
        else if (t == 9) r = max(valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[DF_A2[i]]]);
        valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[i]] = r;
    }}
    return valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[rootNode]];
}}
void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_df({N-1}, 0, int(idx), ix, iy, iz);
}}"""

comp = comp.replace(main_old, main_new)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal4.comp', 'w', encoding='utf-8').write(comp)
print("minimal4.comp 生成")
