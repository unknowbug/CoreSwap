# 生成 minimal2：minimal + spline_eval（spline_coord 简化 0）→ 定位 spline_eval 是否 TDR
import json, importlib.util, sys
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

# spline SSBO 数据
snodes = g.spline_ssbo_nodes
locs = ", ".join(flit(x) for x in g.spline_ssbo_locs)
ders = ", ".join(flit(x) for x in g.spline_ssbo_ders)
vf = ", ".join(flit(x) for x in g.spline_ssbo_val_f)
vk = ", ".join(str(x) for x in g.spline_ssbo_val_kind)
vn = ", ".join(str(x) for x in g.spline_ssbo_val_node)
pack = []
for nd in snodes:
    pack += [nd["coordType"], nd["n"], nd["locBegin"], nd["derBegin"], nd["valBegin"]]
pack_src = ", ".join(str(x) for x in pack)

comp = f"""#version 450
layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0, std430) buffer CoordBuf {{ int coords[]; }} coord;
layout(set = 0, binding = 1, std430) buffer PermBuf {{ uint perm[]; }} permBuf;
layout(set = 0, binding = 2, std430) buffer OriginBuf {{ double origin[]; }} originBuf;
layout(set = 0, binding = 3, std430) buffer OutBuf {{ float density[]; }} outBuf;
layout(set = 0, binding = 4, std430) buffer SplitBuf {{ float splitCoord[]; }} splitBuf;
layout(set = 0, binding = 5, std430) buffer ValBuf {{ float valBuf[]; }};
const int DF_NODES = {N};
const int DF_TYPE[{N}] = int[]({types});
const int DF_A1[{N}] = int[]({a1s});
const int DF_A2[{N}] = int[]({a2s});
const int DF_A3[{N}] = int[]({a3s});
const float DF_F0[{N}] = float[]({f0s});
const float DF_F1[{N}] = float[]({f1s});
const float DF_F2[{N}] = float[]({f2s});
const float DF_F3[{N}] = float[]({f3s});
const int VAL_SLOTS = {val_slots};
const int SLOT_OF[{N}] = int[]({slot_src});

float ycg(int iy, float fy, float ty, float fv, float tv) {{
    float t = clamp((float(iy) - fy) / (ty - fy), 0.0, 1.0);
    return fv + t * (tv - fv);
}}

// ===== spline 数据驱动（完整 SSBO 数据 + 显式栈）=====
const int SPLINE_NODES = {len(snodes)};
const float SPLINE_LOCS[{len(g.spline_ssbo_locs)}] = float[]({locs});
const float SPLINE_DERS[{len(g.spline_ssbo_ders)}] = float[]({ders});
const float SPLINE_VAL_F[{len(g.spline_ssbo_val_f)}] = float[]({vf});
const int SPLINE_VAL_KIND[{len(g.spline_ssbo_val_kind)}] = int[]({vk});
const int SPLINE_VAL_NODE[{len(g.spline_ssbo_val_node)}] = int[]({vn});
const int SPLINE_NODE_PACK[{len(pack)}] = int[]({pack_src});

float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz) {{
    return 0.5;   // 简化：coord 固定
}}
int spline_find_range(float x, int locBegin, int n) {{
    int min = 0; int i = n;
    while (i > 0) {{
        int j = i / 2; int k = min + j;
        if (x < SPLINE_LOCS[locBegin + k]) {{ i = j; }} else {{ min = k + 1; i -= j + 1; }}
    }}
    return min - 1;
}}
float spline_hermite(float coord, float lo, float span, float nv, float ov, float d0, float d1) {{
    float kd = (coord - lo) / span;
    float p = d0 * span - (ov - nv);
    float q = -d1 * span + (ov - nv);
    return (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p));
}}
float spline_eval(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {{
    int nodeStack[24]; int stageStack[24]; int iStack[24]; int outSlot[24];
    float v0Stack[24]; float v1Stack[24]; float coordStack[24];
    int sp = 0;
    nodeStack[0] = rootNode; stageStack[0] = 0; iStack[0] = 0; outSlot[0] = -1;
    v0Stack[0] = 0.0; v1Stack[0] = 0.0; coordStack[0] = 0.0;
    float result = 0.0;
    while (sp >= 0) {{
        int node = nodeStack[sp];
        int base = node * 5;
        int coordType = SPLINE_NODE_PACK[base + 0];
        int n = SPLINE_NODE_PACK[base + 1];
        int locBegin = SPLINE_NODE_PACK[base + 2];
        int derBegin = SPLINE_NODE_PACK[base + 3];
        int valBegin = SPLINE_NODE_PACK[base + 4];
        if (stageStack[sp] == 0) {{
            float coord = spline_coord(coordType, corner, sIdx, ix, iy, iz);
            coordStack[sp] = coord;
            int i = spline_find_range(coord, locBegin, n);
            if (i < 0) {{
                int vk = SPLINE_VAL_KIND[valBegin + 0];
                float v0 = (vk == 0) ? SPLINE_VAL_F[valBegin + 0] : 0.0;
                result = v0 + SPLINE_DERS[derBegin + 0] * (coord - SPLINE_LOCS[locBegin + 0]);
                int ps = outSlot[sp]; sp--;
                if (ps >= 0) {{
                    if ((ps & 1) == 0) {{ v0Stack[ps >> 1] = result; }} else {{ v1Stack[ps >> 1] = result; }}
                    stageStack[ps >> 1] = 2;
                }}
                continue;
            }}
            if (i >= n - 1) {{
                int vk = SPLINE_VAL_KIND[valBegin + n - 1];
                float vn = (vk == 0) ? SPLINE_VAL_F[valBegin + n - 1] : 0.0;
                result = vn + SPLINE_DERS[derBegin + n - 1] * (coord - SPLINE_LOCS[locBegin + n - 1]);
                int ps = outSlot[sp]; sp--;
                if (ps >= 0) {{
                    if ((ps & 1) == 0) {{ v0Stack[ps >> 1] = result; }} else {{ v1Stack[ps >> 1] = result; }}
                    stageStack[ps >> 1] = 2;
                }}
                continue;
            }}
            iStack[sp] = i;
            int vk0 = SPLINE_VAL_KIND[valBegin + i];
            if (vk0 == 0) {{ v0Stack[sp] = SPLINE_VAL_F[valBegin + i]; stageStack[sp] = 1; }}
            else {{
                stageStack[sp] = 1; sp++;
                nodeStack[sp] = SPLINE_VAL_NODE[valBegin + i];
                stageStack[sp] = 0; iStack[sp] = 0; outSlot[sp] = (sp - 1) * 2;
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0;
                continue;
            }}
        }}
        if (stageStack[sp] == 1) {{
            int i = iStack[sp];
            int vk1 = SPLINE_VAL_KIND[valBegin + i + 1];
            if (vk1 == 0) {{ v1Stack[sp] = SPLINE_VAL_F[valBegin + i + 1]; stageStack[sp] = 2; }}
            else {{
                stageStack[sp] = 2; sp++;
                nodeStack[sp] = SPLINE_VAL_NODE[valBegin + i + 1];
                stageStack[sp] = 0; iStack[sp] = 0; outSlot[sp] = (sp - 1) * 2 + 1;
                v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0;
                continue;
            }}
        }}
        if (stageStack[sp] == 2) {{
            int i = iStack[sp];
            float coord = coordStack[sp];
            float nv = v0Stack[sp]; float ov = v1Stack[sp];
            float span = SPLINE_LOCS[locBegin + i + 1] - SPLINE_LOCS[locBegin + i];
            result = spline_hermite(coord, SPLINE_LOCS[locBegin + i], span, nv, ov, SPLINE_DERS[derBegin + i], SPLINE_DERS[derBegin + i + 1]);
            int ps = outSlot[sp]; sp--;
            if (ps >= 0) {{
                if ((ps & 1) == 0) {{ v0Stack[ps >> 1] = result; }} else {{ v1Stack[ps >> 1] = result; }}
                stageStack[ps >> 1] = 2;
            }}
            continue;
        }}
    }}
    return result;
}}

float eval_df_base(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {{
    for (int i = 0; i < DF_NODES; i++) {{
        int t = DF_TYPE[i];
        float r = 0.0;
        if (t == 0) r = DF_F0[i];
        else if (t == 1) r = float(iy);
        else if (t == 2 || t == 19 || t == 3) r = 0.0;
        else if (t == 4) r = spline_eval(DF_A1[i], corner, sIdx, ix, iy, iz);
        else if (t == 18) r = ycg(iy, DF_F0[i], DF_F1[i], DF_F2[i], DF_F3[i]);
        else if (t == 10) r = abs(valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]);
        else if (t == 11) {{ float v = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = v * v; }}
        else if (t == 12) {{ float v = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = v * v * v; }}
        else if (t == 13) {{ float v = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = (v > 0.0 ? v : v * 0.5); }}
        else if (t == 14) {{ float v = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; r = (v > 0.0 ? v : v * 0.25); }}
        else if (t == 15) {{ float v = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]]; float c = clamp(v, -1.0, 1.0); r = c / 2.0 - c * c * c / 24.0; }}
        else if (t == 16) r = clamp(valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], DF_F0[i], DF_F1[i]);
        else if (t == 17) {{
            float inp = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]];
            r = (inp >= DF_F0[i] && inp < DF_F1[i]) ? valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A2[i]]] : valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A3[i]]];
        }}
        else if (t == 20 || t == 21) r = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]];
        else if (t == 6) r = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]] + valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A2[i]]];
        else if (t == 7) r = valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]] * valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A2[i]]];
        else if (t == 8) r = min(valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A2[i]]]);
        else if (t == 9) r = max(valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A1[i]]], valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[DF_A2[i]]]);
        valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[i]] = r;
    }}
    return valBuf[(sIdx * 9 + corner) * VAL_SLOTS + SLOT_OF[rootNode]];
}}
void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_df_base({N-1}, 0, int(idx), ix, iy, iz);
}}
"""
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal2.comp', 'w', encoding='utf-8').write(comp)
print(f"minimal2.comp 生成: {N} 节点, spline_nodes={len(snodes)}")
