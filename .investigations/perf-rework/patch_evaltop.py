# 替换 dfc_gen.py 的 eval_top 区域为闭包版（用锚点切片，避免 edit 空白不匹配）
path = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py'
src = open(path, encoding='utf-8').read()

start_anchor = '        # 顶层 eval_df（含 DF_INTERP 分支，区段 = PER_SAMPLE*sIdx + 0）'
end_anchor = '        glsl = f"""'
i0 = src.index(start_anchor)
i1 = src.index(end_anchor, i0)

new_code = '''        # 顶层 eval_df（闭包化：循环顶层闭包 ~21 节点，含 DF_INTERP 分支；D16 防驱动强制内联）
        top_ctype, top_ca1, top_ca2, top_ca3 = [], [], [], []
        top_cf0, top_cf1, top_cf2, top_cf3 = [], [], [], []
        for ci, i in enumerate(top_closure):
            n = nodes[i]
            t = n["type"]
            top_ctype.append(t)
            def map_a(v):
                if v >= 0 and v in top_pos and t in read_fields:
                    return top_pos[v]
                return v
            top_ca1.append(map_a(n["a1"])); top_ca2.append(map_a(n["a2"])); top_ca3.append(map_a(n["a3"]))
            top_cf0.append(flit(n["f0"])); top_cf1.append(flit(n["f1"])); top_cf2.append(flit(n["f2"])); top_cf3.append(flit(n["f3"]))
        TK = len(top_closure)
        top_slot_src = ", ".join(str(x) for x in top_slot)
        eval_top = f"""
// ---- 顶层解释器（闭包 {TK} 节点，单调用者 main，区段 0；D16 防驱动强制内联）----
const int CLOSURE_T_LEN = {TK};
const int CTYPE_T[{TK}] = int[]({", ".join(str(x) for x in top_ctype)});
const int CA1_T[{TK}] = int[]({", ".join(str(x) for x in top_ca1)});
const int CA2_T[{TK}] = int[]({", ".join(str(x) for x in top_ca2)});
const int CA3_T[{TK}] = int[]({", ".join(str(x) for x in top_ca3)});
const float CF0_T[{TK}] = float[]({", ".join(top_cf0)});
const float CF1_T[{TK}] = float[]({", ".join(top_cf1)});
const float CF2_T[{TK}] = float[]({", ".join(top_cf2)});
const float CF3_T[{TK}] = float[]({", ".join(top_cf3)});
const int SLOT_OF_T[{TK}] = int[]({top_slot_src});
float eval_df(int rootPos, int corner, int sIdx, int ix, int iy, int iz) {{
    for (int ci = 0; ci < CLOSURE_T_LEN; ci++) {{
        int t = CTYPE_T[ci];
        float r = 0.0;
        if (t == {self.DF_INTERP}) {{
            if (CA1_T[ci] == 0) r = interp_0(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 1) r = interp_1(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 2) r = interp_2(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 3) r = interp_3(sIdx, ix, iy, iz);
            else r = interp_4(sIdx, ix, iy, iz);
            valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[ci]] = r;
            continue;
        }}
        if (t == {self.DF_CONSTANT}) r = CF0_T[ci];
        else if (t == {self.DF_Y}) r = float(iy);
        else if (t == {self.DF_NOISE} || t == {self.DF_SHIFTED_NOISE}) r = normal_noise(NOISE_SLOT_BASE[CA1_T[ci]] + corner * NOISE_SLOT_STRIDE[CA1_T[ci]], sIdx);
        else if (t == {self.DF_OLD_BLENDED}) r = interp_noise(NOISE_SLOT_BASE[CA1_T[ci]] + corner * NOISE_SLOT_STRIDE[CA1_T[ci]], sIdx);
        else if (t == {self.DF_SPLINE}) {{
            if (CA2_T[ci] == 1) r = spline_eval(CA1_T[ci], corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2);
            else r = spline_eval(CA1_T[ci], corner, sIdx, ix, iy, iz);
        }}
        else if (t == {self.DF_Y_CLAMPED}) r = y_clamped_gradient(iy, CF0_T[ci], CF1_T[ci], CF2_T[ci], CF3_T[ci]);
        else if (t == {self.DF_ABS}) r = abs(valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]);
        else if (t == {self.DF_SQUARE}) {{ float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]; r = v * v; }}
        else if (t == {self.DF_CUBE}) {{ float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]; r = v * v * v; }}
        else if (t == {self.DF_HALF_NEG}) {{ float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]; r = (v > 0.0 ? v : v * 0.5); }}
        else if (t == {self.DF_QUARTER_NEG}) {{ float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]; r = (v > 0.0 ? v : v * 0.25); }}
        else if (t == {self.DF_SQUEEZE}) {{ float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]]; float c = clamp(v, -1.0, 1.0); r = c / 2.0 - c * c * c / 24.0; }}
        else if (t == {self.DF_CLAMP}) r = clamp(valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]], CF0_T[ci], CF1_T[ci]);
        else if (t == {self.DF_RANGE_CHOICE}) {{
            float inp = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]];
            r = (inp >= CF0_T[ci] && inp < CF1_T[ci]) ? valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA2_T[ci]]] : valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA3_T[ci]]];
        }}
        else if (t == {self.DF_BLEND_DENSITY}) r = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]];
        else if (t == {self.DF_FLAT_CACHE}) r = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]];
        else if (t == {self.DF_ADD}) r = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]] + valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA2_T[ci]]];
        else if (t == {self.DF_MUL}) r = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]] * valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA2_T[ci]]];
        else if (t == {self.DF_MIN}) r = min(valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]], valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA2_T[ci]]]);
        else if (t == {self.DF_MAX}) r = max(valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]], valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA2_T[ci]]]);
        valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[ci]] = r;
    }}
    return valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[rootPos]];
}}
"""
'''
src = src[:i0] + new_code + src[i1:]
open(path, 'w', encoding='utf-8').write(src)
print(f'替换 eval_top 完成（{i0}..{i1}）')
