# dfc_gen.py —— DF 树 → GLSL compute shader 生成器（CoreSwap GPU 加速 DFC）
# 精度分层：minecraft:old_blended_noise（InterpolatedNoiseSampler）→ fp64（double），
#           其余（NormalNoise/spline/算术/插值）→ fp32（float）。
# 输入：density_function JSON 树（递归 dict），输出：GLSL 源码字符串。
import json
import os

# 坐标变量名（块坐标，整数语义）
CX, CY, CZ = "ix", "iy", "iz"   # int 块坐标

class DfcGen:
    def __init__(self, df_dir=None):
        self.df_dir = df_dir          # density_function 目录（用于解析 registry 引用）
        self.df_cache = {}            # ref -> DF dict
        self.noise_instances = []     # [(kind, params_dict)]：old_blended / normal / shifted / shift
        self.noise_index = {}         # 去重 key -> index
        self.registry_funcs = {}      # ref -> 函数名
        self.registry_defs = []       # [(函数名, 表达式)]，按依赖序

    # ---- registry 引用解析 ----
    def resolve_ref(self, ref):
        if ref == "minecraft:y":
            return {"type": "minecraft:y"}
        if ref == "minecraft:zero":
            return {"type": "minecraft:constant", "value": 0.0}
        if ref == "minecraft:shift_x":
            return {"type": "minecraft:shift_a"}
        if ref == "minecraft:shift_z":
            return {"type": "minecraft:shift_b"}
        if ref in self.df_cache:
            return self.df_cache[ref]
        # registry: "minecraft:overworld/continents" -> <df_dir>/overworld/continents.json
        rel = ref.replace("minecraft:", "")
        fpath = os.path.join(self.df_dir, rel + ".json") if self.df_dir else None
        if fpath and os.path.exists(fpath):
            with open(fpath, 'r', encoding='utf-8') as f:
                df = json.load(f)
            self.df_cache[ref] = df
            return df
        raise ValueError(f"cannot resolve registry ref: {ref}")

    # ---- registry 引用 → 命名函数（去重，避免表达式爆炸）----
    def _gen_registry_call(self, ref):
        if ref in self.registry_funcs:
            return f"{self.registry_funcs[ref]}(x, y, z)"
        fname = "df_" + ref.replace("minecraft:", "").replace("/", "_").replace(".", "_")
        self.registry_funcs[ref] = fname          # 先注册（防循环引用）
        df = self.resolve_ref(ref)
        expr = self.gen(df)
        self.registry_defs.append((fname, expr))
        return f"{fname}(x, y, z)"

    # ---- 噪声实例注册（运行时从 seed 生成参数，这里只收集 + 分配索引）----
    def _register_noise(self, kind, key, params):
        if key in self.noise_index:
            return self.noise_index[key]
        idx = len(self.noise_instances)
        self.noise_instances.append((kind, params))
        self.noise_index[key] = idx
        return idx

    # ---- 主入口：生成 DF 节点的 GLSL 表达式（float 语义，old_blended_noise 内部 double 转 float）----
    def gen(self, df):
        if isinstance(df, (int, float)):
            return f"{float(df)}f"
        if isinstance(df, str):
            if df == "minecraft:y":
                return "y"
            if df == "minecraft:zero":
                return "0.0f"
            if df == "minecraft:shift_x":
                return self.gen({"type": "minecraft:shift_a"})
            if df == "minecraft:shift_z":
                return self.gen({"type": "minecraft:shift_b"})
            # registry 引用 → 命名函数调用（去重，避免表达式爆炸）
            return self._gen_registry_call(df)
        if isinstance(df, dict) and "points" in df and "coordinate" in df and "type" not in df:
            # 嵌套 spline（无 type 字段，直接 {coordinate, points}）
            return self._gen_spline(df)
        t = df.get("type", "")
        if t == "minecraft:y":
            return "y"
        if t == "minecraft:constant":
            return f"{float(df.get('value', 0.0))}f"
        if t == "minecraft:old_blended_noise":
            # fp64：调用 double 采样函数，结果转 float
            idx = self._register_noise("old_blended", f"ob{len(self.noise_instances)}", {
                "xz_scale": df.get("xz_scale", 0.25), "y_scale": df.get("y_scale", 0.125),
                "xz_factor": df.get("xz_factor", 80.0), "y_factor": df.get("y_factor", 160.0),
                "smear": df.get("smear_scale_multiplier", 8.0),
            })
            return f"(float(interp_noise_{idx}({CX}, {CY}, {CZ})))"
        if t == "minecraft:noise":
            idx = self._register_noise("normal", f"n{len(self.noise_instances)}", {
                "noise": df.get("noise", ""), "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
            })
            return f"normal_noise_{idx}({CX}, {CY}, {CZ})"
        if t == "minecraft:shifted_noise":
            idx = self._register_noise("normal", f"n{len(self.noise_instances)}", {
                "noise": df.get("noise", ""), "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
            })
            sx = self.gen(df.get("shift_x", 0.0)); sy = self.gen(df.get("shift_y", 0.0)); sz = self.gen(df.get("shift_z", 0.0))
            # 注意：shifted_noise 的坐标 = pos*scale + shift
            xz = df.get("xz_scale", 1.0); y = df.get("y_scale", 1.0)
            return f"shifted_noise_{idx}({CX}, {CY}, {CZ}, {xz}f, {y}f, {sx}, {sy}, {sz})"
        if t in ("minecraft:shift_a", "minecraft:shift_b", "minecraft:shift"):
            idx = self._register_noise("normal", f"n{len(self.noise_instances)}", {"noise": "minecraft:offset"})
            axis = {"minecraft:shift_a": "x", "minecraft:shift_b": "z", "minecraft:shift": "xz"}[t]
            return f"shift_noise_{idx}({CX}, {CY}, {CZ}, \"{axis}\")"
        if t == "minecraft:spline":
            return self._gen_spline(df.get("spline", df))
        if t == "minecraft:add":
            return f"({self.gen(df['argument1'])} + {self.gen(df['argument2'])})"
        if t == "minecraft:mul":
            return f"({self.gen(df['argument1'])} * {self.gen(df['argument2'])})"
        if t == "minecraft:min":
            return f"min({self.gen(df['argument1'])}, {self.gen(df['argument2'])})"
        if t == "minecraft:max":
            return f"max({self.gen(df['argument1'])}, {self.gen(df['argument2'])})"
        if t == "minecraft:abs":
            return f"abs({self.gen(df['argument'])})"
        if t == "minecraft:square":
            v = self.gen(df['argument']); return f"({v} * {v})"
        if t == "minecraft:cube":
            v = self.gen(df['argument']); return f"({v} * {v} * {v})"
        if t == "minecraft:half_negative":
            v = self.gen(df['argument']); return f"({v} > 0.0f ? {v} : {v} * 0.5f)"
        if t == "minecraft:quarter_negative":
            v = self.gen(df['argument']); return f"({v} > 0.0f ? {v} : {v} * 0.25f)"
        if t == "minecraft:squeeze":
            v = self.gen(df['argument'])
            return f"(clamp({v}, -1.0f, 1.0f) / 2.0f - clamp({v}, -1.0f, 1.0f) * clamp({v}, -1.0f, 1.0f) * clamp({v}, -1.0f, 1.0f) / 24.0f)"
        if t == "minecraft:clamp":
            return f"clamp({self.gen(df['input'])}, {float(df['min'])}f, {float(df['max'])}f)"
        if t == "minecraft:range_choice":
            inp = self.gen(df['input'])
            return f"(({inp} >= {float(df['min_inclusive'])}f && {inp} < {float(df['max_exclusive'])}f) ? {self.gen(df['when_in_range'])} : {self.gen(df['when_out_of_range'])})"
        if t == "minecraft:y_clamped_gradient":
            return f"y_clamped_gradient({CY}, {float(df['from_y'])}f, {float(df['to_y'])}f, {float(df['from_value'])}f, {float(df['to_value'])}f)"
        if t == "minecraft:weird_scaled_sampler":
            # 依赖 input + noise + rarity_value_mapper（暂简化为 0，后续完善）
            return f"0.0f"
        if t in ("minecraft:flat_cache", "minecraft:cache_2d", "minecraft:cache_once", "minecraft:cache_all_in_cell", "minecraft:interpolated"):
            # 缓存/插值包装：GPU 端剥掉包装（flat_cache 由 CPU 预填充，见 C2ME CacheElimination）
            return self.gen(df.get("argument", df.get("input", 0.0)))
        if t == "minecraft:blend_alpha":
            return "1.0f"
        if t == "minecraft:blend_offset":
            return "0.0f"
        if t == "minecraft:blend_density":
            return self.gen(df.get("argument", 0.0))
        raise ValueError(f"unsupported type: {t}")

    # ---- spline 生成：Hermite 插值（float），对齐 vanilla 三段式（外推 + Hermite）----
    def _gen_spline(self, spline):
        coord = self.gen(spline["coordinate"])
        points = spline["points"]
        n = len(points)
        locs = [float(p["location"]) for p in points]
        ders = [float(p["derivative"]) for p in points]
        vals = [self.gen(p["value"]) for p in points]
        # 边界外推（vanilla Spline.apply 的 i<0 / i==n-1 分支）
        lo_extrap = f"({vals[0]} + {ders[0]}f * ({coord} - {locs[0]}f))"
        hi_extrap = f"({vals[n-1]} + {ders[n-1]}f * ({coord} - {locs[n-1]}f))"
        # 中间区间 if-else 链（从高区间往下套）
        expr = hi_extrap
        for i in range(n - 2, -1, -1):
            span = locs[i+1] - locs[i]
            seg = f"spline_seg({coord}, {locs[i]}f, {span}f, {vals[i]}, {vals[i+1]}, {ders[i]}f, {ders[i+1]}f)"
            cond = f"({coord} < {locs[i+1]}f)"
            expr = f"({cond} ? {seg} : {expr})"
        expr = f"(({coord} < {locs[0]}f) ? {lo_extrap} : {expr})"
        return expr

    # ---- 生成完整 shader 源码 ----
    def gen_shader(self, root_df):
        expr = self.gen(root_df)
        funcs = []
        # registry 函数定义（依赖序已保证）
        for fname, fexpr in self.registry_defs:
            funcs.append(f"float {fname}(float x, float y, float z) {{\n    return {fexpr};\n}}\n")
        # 噪声函数（old_blended double + normal float）
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                funcs.append(self._old_blended_func(idx, params))
            elif kind == "normal":
                funcs.append(self._normal_func(idx, params))
        return self._shader_template(expr, funcs)

    def _old_blended_func(self, idx, p):
        # 参数内联（scale/factor/smear）；perm/origin 先用 identity/0（Phase 2 改 params 读取）
        return f"""
double interp_noise_{idx}(int px, int py, int pz) {{
    double d = double(px) * {684.412 * p['xz_scale']:.17g};
    double e = double(py) * {684.412 * p['y_scale']:.17g};
    double f = double(pz) * {684.412 * p['xz_scale']:.17g};
    double g = d / {p['xz_factor']:.17g};
    double h = e / {p['y_factor']:.17g};
    double i = f / {p['xz_factor']:.17g};
    double j = {684.412 * p['y_scale']:.17g} * {p['smear']:.17g};
    double k = j / {p['y_factor']:.17g};
    double n = 0.0; double o = 1.0;
    for (int q = 0; q < 8; q++) {{
        n += pn_sample5(32 + q, maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o;
        o /= 2.0;
    }}
    double qq = (n / 10.0 + 1.0) / 2.0;
    bool bl = qq >= 1.0; bool bl2 = qq <= 0.0;
    double l = 0.0; double m = 0.0; o = 1.0;
    for (int r = 0; r < 16; r++) {{
        double s = maintainPrecision(d*o); double t = maintainPrecision(e*o); double u = maintainPrecision(f*o);
        double v = j*o;
        if (!bl) l += pn_sample5(r, s, t, u, v, e*o) / o;
        if (!bl2) m += pn_sample5(16 + r, s, t, u, v, e*o) / o;
        o /= 2.0;
    }}
    double w = clamp(qq, 0.0, 1.0);
    return (l / 512.0 + w * (m / 512.0 - l / 512.0)) / 128.0;
}}"""

    def _normal_func(self, idx, p):
        # NormalNoise（DoublePerlinNoiseSampler）：first + second × 1.018，float 版（Phase 2 补全参数）
        return f"""
float normal_noise_{idx}(int px, int py, int pz) {{
    return 0.0f;  // Phase 2：OctavePerlinNoiseSampler float 叠加
}}"""

    def _shader_template(self, expr, funcs):
        funcs_src = "\n".join(funcs)
        return f"""#version 450
#extension GL_ARB_gpu_shader_fp64 : require

layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// 坐标输入（int 块坐标，x,y,z 三元组）
layout(set = 0, binding = 0, std430) buffer CoordBuf {{ int coords[]; }} coord;
// 噪声参数（perm/origin/amplitudes，运行时从 seed 生成上传）
layout(set = 0, binding = 1, std430) buffer Params {{ uint data[]; }} params;
// 输出 density
layout(set = 0, binding = 2, std430) buffer OutBuf {{ float density[]; }} outBuf;

// ===== double 工具（old_blended_noise 用）=====
const double GRADIENTS[16][3] = {{
    {{ 1,  1,  0}}, {{-1,  1,  0}}, {{ 1, -1,  0}}, {{-1, -1,  0}},
    {{ 1,  0,  1}}, {{-1,  0,  1}}, {{ 1,  0, -1}}, {{-1,  0, -1}},
    {{ 0,  1,  1}}, {{ 0, -1,  1}}, {{ 0,  1, -1}}, {{ 0, -1, -1}},
    {{ 1,  1,  0}}, {{ 0, -1,  1}}, {{-1,  1,  0}}, {{ 0, -1, -1}}
}};
double maintainPrecision(double v) {{ return v - floor(v / 3.3554432E7 + 0.5) * 3.3554432E7; }}
double perlinFadeD(double v) {{ return v * v * v * (v * (v * 6.0 - 15.0) + 10.0); }}
double lerpD(double d, double s, double e) {{ return s + d * (e - s); }}
int mapPermD(int octBase, int v) {{ return int(params.data[octBase * 256 + uint(v & 255)]); }}
double gradD(int octBase, int hash, double x, double y, double z) {{
    return GRADIENTS[hash & 15][0] * x + GRADIENTS[hash & 15][1] * y + GRADIENTS[hash & 15][2] * z;
}}
double pn_sectionD(int octBase, int sx, int sy, int sz, double lx, double ly, double lz, double fadeY) {{
    int i = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
    int k = mapPermD(octBase, i + sy); int l = mapPermD(octBase, i + sy + 1);
    int m = mapPermD(octBase, j + sy); int n = mapPermD(octBase, j + sy + 1);
    double d = gradD(octBase, mapPermD(octBase, k + sz),     lx,     ly,     lz);
    double e = gradD(octBase, mapPermD(octBase, m + sz),     lx - 1.0, ly,     lz);
    double f = gradD(octBase, mapPermD(octBase, l + sz),     lx,     ly - 1.0, lz);
    double g = gradD(octBase, mapPermD(octBase, n + sz),     lx - 1.0, ly - 1.0, lz);
    double h = gradD(octBase, mapPermD(octBase, k + sz + 1), lx,     ly,     lz - 1.0);
    double o = gradD(octBase, mapPermD(octBase, m + sz + 1), lx - 1.0, ly,     lz - 1.0);
    double p = gradD(octBase, mapPermD(octBase, l + sz + 1), lx,     ly - 1.0, lz - 1.0);
    double q = gradD(octBase, mapPermD(octBase, n + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
    double r = perlinFadeD(lx); double s = perlinFadeD(fadeY); double t = perlinFadeD(lz);
    double x0 = lerpD(r, d, e); double x1 = lerpD(r, f, g);
    double x2 = lerpD(r, h, o); double x3 = lerpD(r, p, q);
    double y0 = lerpD(s, x0, x1); double y1 = lerpD(s, x2, x3);
    return lerpD(t, y0, y1);
}}
// 5 参数 sample（double，含 y 轴 smear），origin 当前硬编码 0（Phase 2 改 params 读取）
double pn_sample5(int octBase, double x, double y, double z, double yScale, double yMax) {{
    double d = x; double e = y; double f = z;
    int i = int(floor(d)); int j = int(floor(e)); int k = int(floor(f));
    double g = d - i; double h = e - j; double l = f - k;
    double n;
    if (yScale != 0.0) {{
        double m = (yMax >= 0.0 && yMax < h) ? yMax : h;
        n = floor(m / yScale + double(1.0e-7f)) * yScale;
    }} else n = 0.0;
    return pn_sectionD(octBase, i, j, k, g, h - n, l, h);
}}

// ===== float 工具（NormalNoise/spline/算术 用）=====
float perlinFadeF(float v) {{ return v * v * v * (v * (v * 6.0 - 15.0) + 10.0); }}
float lerpF(float d, float s, float e) {{ return s + d * (e - s); }}
float spline_seg(float f, float lo, float span, float nv, float ov, float d0, float d1) {{
    float kd = (f - lo) / span;
    float p = d0 * span - (ov - nv);
    float q = -d1 * span + (ov - nv);
    return (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p));
}}
float y_clamped_gradient(int y, float fromY, float toY, float fromV, float toV) {{
    float t = clamp((float(y) - fromY) / (toY - fromY), 0.0, 1.0);
    return fromV + t * (toV - fromV);
}}

{funcs_src}

float eval_density(int ix, int iy, int iz) {{
    float x = float(ix), y = float(iy), z = float(iz);
    return {expr};
}}

void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_density(ix, iy, iz);
}}
"""


def main():
    import sys
    if len(sys.argv) < 2:
        print("usage: python dfc_gen.py <df.json>")
        sys.exit(1)
    with open(sys.argv[1], 'r', encoding='utf-8') as f:
        df = json.load(f)
    g = DfcGen()
    shader = g.gen_shader(df)
    print(shader)


if __name__ == '__main__':
    main()
