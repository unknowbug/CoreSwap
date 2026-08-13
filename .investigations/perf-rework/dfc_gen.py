# dfc_gen.py —— DF 树 → GLSL compute shader 生成器（CoreSwap GPU 加速 DFC）
# 精度分层：minecraft:old_blended_noise（InterpolatedNoiseSampler）→ fp64（double），
#           其余（NormalNoise/spline/算术/插值）→ fp32（float）。
# 输入：density_function JSON 树（递归 dict），输出：GLSL 源码字符串。
import json
import os

# 坐标变量名（块坐标，整数语义）
CX, CY, CZ = "ix", "iy", "iz"   # int 块坐标（默认）

class DfcGen:
    def __init__(self, df_dir=None, noise_dir=None):
        self.df_dir = df_dir
        self.df_cache = {}
        self.noise_instances = []
        self.noise_index = {}
        self.registry_funcs = {}
        self.registry_defs = []
        self.spline_funcs = []        # [(函数名, coord表达式, n, locs, ders, vals)]，嵌套 spline 函数化
        self.spline_cache = {}        # spline 结构 JSON -> 函数调用（去重，ridges 被多处引用）
        self.noise_params = {}
        # 坐标链（CPU 预拆分）：主噪声的坐标链描述 + shift 噪声参数
        self.coord_chains = []        # 每个 normal 实例的坐标链（type/scale/shift/flat_cache）
        self.shift_noises = {}        # shift 噪声 noise_key -> {firstOctave, amplitudes}（CPU double 采样）
        self.flat_cache_depth = 0     # 当前在 flat_cache 内的嵌套深度
        # 坐标变量（gen_with_coords 可切换，用于 flat_cache 的 biome 对齐）
        self.cx, self.cy, self.cz = "ix", "iy", "iz"     # int 块坐标
        self.fx, self.fy, self.fz = "x", "y", "z"        # float 坐标
        if noise_dir:
            for f in os.listdir(noise_dir):
                if f.endswith(".json"):
                    with open(os.path.join(noise_dir, f), 'r', encoding='utf-8') as fh:
                        np = json.load(fh)
                    self.noise_params[f[:-5]] = {"firstOctave": np.get("firstOctave", 0), "amplitudes": np.get("amplitudes", [1.0])}

    def gen_with_coords(self, df, cx, cy, cz, fx=None, fy=None, fz=None):
        """临时切换坐标变量生成表达式（flat_cache biome 对齐用）"""
        old = (self.cx, self.cy, self.cz, self.fx, self.fy, self.fz)
        self.cx, self.cy, self.cz = cx, cy, cz
        self.fx, self.fy, self.fz = fx or cx, fy or cy, fz or cz
        try:
            return self.gen(df)
        finally:
            (self.cx, self.cy, self.cz, self.fx, self.fy, self.fz) = old

    def _resolve_noise_params(self, noise_key):
        """noise key（如 minecraft:continentalness）→ {firstOctave, amplitudes}"""
        name = noise_key.replace("minecraft:", "")
        return self.noise_params.get(name, {"firstOctave": 0, "amplitudes": [1.0]})

    def _resolve_shift(self, shift_df):
        """解析 shift 节点，返回 {type, noise_key}，并记录 shift 噪声参数（CPU double 采样）"""
        if isinstance(shift_df, str):
            if shift_df == "minecraft:shift_x":
                shift_df = {"type": "minecraft:shift_a"}
            elif shift_df == "minecraft:shift_z":
                shift_df = {"type": "minecraft:shift_b"}
            else:
                return {"type": "constant", "value": 0.0}
        if isinstance(shift_df, (int, float)):
            return {"type": "constant", "value": float(shift_df)}
        if isinstance(shift_df, dict):
            t = shift_df.get("type", "")
            if t in ("minecraft:shift_a", "minecraft:shift_b", "minecraft:shift"):
                np = self._resolve_noise_params("minecraft:offset")
                self.shift_noises["minecraft:offset"] = {"firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"]}
                return {"type": t.replace("minecraft:", ""), "noise_key": "minecraft:offset"}
        return {"type": "constant", "value": 0.0}

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
            return f"{self.registry_funcs[ref]}({self.cx}, {self.cy}, {self.cz})"   # 用当前坐标上下文（flat_cache 对齐后）
        fname = "df_" + ref.replace("minecraft:", "").replace("/", "_").replace(".", "_")
        self.registry_funcs[ref] = fname          # 先注册（防循环引用）
        df = self.resolve_ref(ref)
        expr = self.gen(df)
        self.registry_defs.append((fname, expr))
        return f"{fname}(sIdx, {self.cx}, {self.cy}, {self.cz})"

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
            return self.fy
        if t == "minecraft:constant":
            return f"{float(df.get('value', 0.0))}f"
        if t == "minecraft:old_blended_noise":
            # fp64：调用 double 采样函数，结果转 float
            idx = self._register_noise("old_blended", f"ob{len(self.noise_instances)}", {
                "xz_scale": df.get("xz_scale", 0.25), "y_scale": df.get("y_scale", 0.125),
                "xz_factor": df.get("xz_factor", 80.0), "y_factor": df.get("y_factor", 160.0),
                "smear": df.get("smear_scale_multiplier", 8.0),
            })
            return f"(float(interp_noise_{idx}({self.cx}, {self.cy}, {self.cz})))"
        if t == "minecraft:noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            idx = self._register_noise("normal", df.get("noise", ""), {
                "noise": df.get("noise", ""), "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            })
            self.coord_chains.append({
                "type": "noise", "noise_key": df.get("noise", ""),
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
            })
            return f"normal_noise_{idx}(sIdx)"
        if t == "minecraft:shifted_noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            idx = self._register_noise("normal", df.get("noise", ""), {
                "noise": df.get("noise", ""), "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            })
            self.coord_chains.append({
                "type": "shifted_noise", "noise_key": df.get("noise", ""),
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
                "shift_x": self._resolve_shift(df.get("shift_x", 0.0)),
                "shift_y": self._resolve_shift(df.get("shift_y", 0.0)),
                "shift_z": self._resolve_shift(df.get("shift_z", 0.0)),
            })
            return f"normal_noise_{idx}(sIdx)"
        if t in ("minecraft:shift_a", "minecraft:shift_b", "minecraft:shift"):
            # shift 噪声（offset）是坐标链的一部分，CPU 侧 double 采样，GPU 侧不采样
            self._resolve_shift(df)
            return "0.0f"
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
            return f"y_clamped_gradient({self.cy}, {float(df['from_y'])}f, {float(df['to_y'])}f, {float(df['from_value'])}f, {float(df['to_value'])}f)"
        if t == "minecraft:weird_scaled_sampler":
            # 依赖 input + noise + rarity_value_mapper（暂简化为 0，后续完善）
            return f"0.0f"
        if t == "minecraft:flat_cache":
            # flat_cache：坐标对齐到 biome（x>>2<<2, 0, z>>2<<2），delegate 采样（对齐 vanilla FlatCache.sample）
            self.flat_cache_depth += 1
            inner = self.gen_with_coords(df["argument"], "((ix >> 2) << 2)", "0", "((iz >> 2) << 2)",
                                         "float((ix >> 2) << 2)", "0.0f", "float((iz >> 2) << 2)")
            self.flat_cache_depth -= 1
            return f"({inner})"
        if t in ("minecraft:cache_2d", "minecraft:cache_once", "minecraft:cache_all_in_cell"):
            # 缓存包装：采样结果 = delegate（原始坐标），剥掉（对齐 vanilla Cache2D/CacheOnce）
            return self.gen(df.get("argument", df.get("input", 0.0)))
        if t == "minecraft:interpolated":
            # cell 三线性插值（4×4×8，高频噪声防 alias）——Phase 6 后续实现，暂剥掉
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
        # 嵌套 spline 用函数调用（避免 if-else 链指数膨胀），二分查找 + 中间区间 if-else 链
        # spline 函数接受 int 块坐标（coordinate 表达式在函数体内计算）
        key = json.dumps(spline, sort_keys=True)   # 结构去重（ridges 被多处引用）
        if key in self.spline_cache:
            return self.spline_cache[key]
        coord = self.gen(spline["coordinate"])
        points = spline["points"]
        n = len(points)
        locs = [float(p["location"]) for p in points]
        ders = [float(p["derivative"]) for p in points]
        vals = []
        for p in points:
            v = p["value"]
            if isinstance(v, dict) and "points" in v and "coordinate" in v and "type" not in v:
                vals.append(self._gen_spline(v))   # 嵌套 spline → 函数调用（spline_M(ix,iy,iz)）
            else:
                vals.append(self.gen(v))           # 其他 → 内联表达式
        idx = len(self.spline_funcs)
        fname = f"spline_{idx}"
        self.spline_funcs.append((fname, coord, n, locs, ders, vals))
        call = f"{fname}(sIdx, {self.cx}, {self.cy}, {self.cz})"   # 用当前坐标上下文（flat_cache 对齐后的）
        self.spline_cache[key] = call
        return call

    def _spline_body(self, fname, coord, n, locs, ders, vals):
        def flit(x):
            s = format(x, '.17g')
            if '.' not in s and 'e' not in s and 'E' not in s:
                s += '.0'
            return s + 'f'
        lines = []
        lines.append(f"float {fname}(int sIdx, int ix, int iy, int iz) {{")
        lines.append(f"    float x = float(ix), y = float(iy), z = float(iz);")
        lines.append(f"    float coord = {coord};")
        # 边界外推 + 中间 Hermite（if-else 链，无数组无循环，NVIDIA 编译快）
        lines.append(f"    if (coord < {flit(locs[0])}) {{ return ({vals[0]}) + {flit(ders[0])} * (coord - {flit(locs[0])}); }}")
        for i in range(n - 1):
            span = locs[i+1] - locs[i]
            lines.append(f"    if (coord < {flit(locs[i+1])}) {{")
            lines.append(f"        float nv = ({vals[i]}); float ov = ({vals[i+1]});")
            lines.append(f"        float kd = (coord - {flit(locs[i])}) / {flit(span)};")
            lines.append(f"        float p = {flit(ders[i])} * {flit(span)} - (ov - nv);")
            lines.append(f"        float q = -{flit(ders[i+1])} * {flit(span)} + (ov - nv);")
            lines.append(f"        return lerpF(kd, nv, ov) + kd * (1.0 - kd) * lerpF(kd, p, q);")
            lines.append(f"    }}")
        lines.append(f"    return ({vals[n-1]}) + {flit(ders[n-1])} * (coord - {flit(locs[n-1])});")
        lines.append(f"}}")
        return "\n".join(lines)

    # ---- 生成完整 shader 源码 ----
    def gen_shader(self, root_df):
        expr = self.gen(root_df)
        funcs = []
        # 噪声函数（old_blended double + normal float）先定义（registry 函数会调用）
        # 分配 octBase（perm/origin buffer 的 octave 偏移）+ splitBase（拆分坐标 buffer 的偏移，单位 6 值/octave）
        octBase = 0
        splitBase = 0
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                funcs.append(self._old_blended_func(idx, params, octBase))
                octBase += 40
            elif kind == "normal":
                n = len(params.get("amplitudes", [1.0]))
                funcs.append(self._normal_func(idx, params, octBase, splitBase))
                octBase += 2 * n
                splitBase += 6 * 2 * n   # 6 值 [ix,iy,iz,gx,gy,gz] × 2n octave
        self.split_total = splitBase      # 每采样点的拆分坐标总数
        # registry 函数定义（依赖序已保证），传 int 块坐标，内部转 float
        for fname, fexpr in self.registry_defs:
            funcs.append(f"float {fname}(int sIdx, int ix, int iy, int iz) {{\n    float x = float(ix), y = float(iy), z = float(iz);\n    return {fexpr};\n}}\n")
        # spline 函数定义（依赖序：嵌套 spline 先定义）
        for fname, coord, n, locs, ders, vals in self.spline_funcs:
            funcs.append(self._spline_body(fname, coord, n, locs, ders, vals))
        return self._shader_template(expr, funcs)

    def gen_noise_manifest(self):
        """输出噪声清单（JSON dict）：normal 实例的坐标链 + octBase/splitBase + shift 噪声参数，供 CPU 侧重放"""
        normal_instances = []
        octBase = 0
        splitBase = 0
        ci = 0   # coord_chains 索引（只对 normal 实例）
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                octBase += 40
            elif kind == "normal":
                n = len(params.get("amplitudes", [1.0]))
                chain = self.coord_chains[ci] if ci < len(self.coord_chains) else {"type": "noise", "noise_key": "", "xz_scale": 1.0, "y_scale": 1.0, "flat_cache": False}
                normal_instances.append({
                    "noise_key": params.get("noise", ""),
                    "firstOctave": params.get("firstOctave", 0),
                    "amplitudes": params.get("amplitudes", [1.0]),
                    "octBase": octBase, "splitBase": splitBase, "n": n,
                    "coord_chain": chain,
                })
                ci += 1
                octBase += 2 * n
                splitBase += 6 * 2 * n
        return {"normal_instances": normal_instances, "shift_noises": self.shift_noises,
                "split_total": splitBase}

    def _old_blended_func(self, idx, p, octBase):
        # 参数内联（scale/factor/smear）；perm/origin 从 PermBuf/OriginBuf 读（octBase 偏移）
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
        n += pn_sample5({octBase} + 32 + q, maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o;
        o /= 2.0;
    }}
    double qq = (n / 10.0 + 1.0) / 2.0;
    bool bl = qq >= 1.0; bool bl2 = qq <= 0.0;
    double l = 0.0; double m = 0.0; o = 1.0;
    for (int r = 0; r < 16; r++) {{
        double s = maintainPrecision(d*o); double t = maintainPrecision(e*o); double u = maintainPrecision(f*o);
        double v = j*o;
        if (!bl) l += pn_sample5({octBase} + r, s, t, u, v, e*o) / o;
        if (!bl2) m += pn_sample5({octBase} + 16 + r, s, t, u, v, e*o) / o;
        o /= 2.0;
    }}
    double w = clamp(qq, 0.0, 1.0);
    return (l / 512.0 + w * (m / 512.0 - l / 512.0)) / 128.0;
}}"""

    def _normal_func(self, idx, p, octBase, splitBase):
        # NormalNoise：CPU 预拆分坐标（int32 格点 + float 小数），GPU 纯 float 采样（无 fp64）
        amps = p.get("amplitudes", [1.0])
        n = len(amps)
        persistence = (2.0 ** (n - 1)) / (2.0 ** n - 1.0)
        nonz = [i for i, a in enumerate(amps) if a != 0.0]
        j = min(nonz) if nonz else 0
        k = max(nonz) if nonz else 0
        create_amp = 0.1 * (1.0 + 1.0 / (k - j + 1))
        amplitude = 0.16666666666666666 / create_amp
        amps_str = ", ".join(f"{a:.17g}" for a in amps)
        return f"""
float normal_noise_{idx}(int sIdx) {{
    const double amps[{n}] = double[]({amps_str});
    // first sampler（拆分坐标在 splitCoord，CPU 预计算 int32 格点 + float 小数）
    double d = 0.0;
    double f = {persistence:.17g};
    for (int i = 0; i < {n}; i++) {{
        int b = sIdx * SPLIT_TOTAL + {splitBase} + i * 6;
        int ix = int(splitBuf.splitCoord[b + 0]); int iy = int(splitBuf.splitCoord[b + 1]); int iz = int(splitBuf.splitCoord[b + 2]);
        float gx = splitBuf.splitCoord[b + 3]; float gy = splitBuf.splitCoord[b + 4]; float gz = splitBuf.splitCoord[b + 5];
        float ns = pn_sample3_f32({octBase} + i, ix, iy, iz, gx, gy, gz);
        d += amps[i] * double(ns) * f;
        f /= 2.0;
    }}
    // second sampler（拆分坐标偏移 + 6n）
    double d2 = 0.0;
    f = {persistence:.17g};
    for (int i = 0; i < {n}; i++) {{
        int b = sIdx * SPLIT_TOTAL + {splitBase} + 6 * {n} + i * 6;
        int ix = int(splitBuf.splitCoord[b + 0]); int iy = int(splitBuf.splitCoord[b + 1]); int iz = int(splitBuf.splitCoord[b + 2]);
        float gx = splitBuf.splitCoord[b + 3]; float gy = splitBuf.splitCoord[b + 4]; float gz = splitBuf.splitCoord[b + 5];
        float ns = pn_sample3_f32({octBase} + {n} + i, ix, iy, iz, gx, gy, gz);
        d2 += amps[i] * double(ns) * f;
        f /= 2.0;
    }}
    return float((d + d2) * {amplitude:.17g});
}}"""

    def _shader_template(self, expr, funcs):
        funcs_src = "\n".join(funcs)
        return f"""#version 450
#extension GL_ARB_gpu_shader_fp64 : require
#extension GL_EXT_control_flow_attributes : require

layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// 坐标输入（int 块坐标，x,y,z 三元组）
layout(set = 0, binding = 0, std430) buffer CoordBuf {{ int coords[]; }} coord;
// perm 表（每 octave 256 uint，连续）
layout(set = 0, binding = 1, std430) buffer PermBuf {{ uint perm[]; }} permBuf;
// origin（每 octave 3 double，连续）
layout(set = 0, binding = 2, std430) buffer OriginBuf {{ double origin[]; }} originBuf;
// 输出 density
layout(set = 0, binding = 3, std430) buffer OutBuf {{ float density[]; }} outBuf;
// 拆分坐标（CPU 预计算：每采样点 SPLIT_TOTAL 个 float，[ix,iy,iz,gx,gy,gz] × 每 octave）
layout(set = 0, binding = 4, std430) buffer SplitBuf {{ float splitCoord[]; }} splitBuf;
const int SPLIT_TOTAL = {self.split_total};

// ===== double 工具（old_blended_noise 用）=====
const double GRADIENTS[16][3] = {{
    {{ 1,  1,  0}}, {{-1,  1,  0}}, {{ 1, -1,  0}}, {{-1, -1,  0}},
    {{ 1,  0,  1}}, {{-1,  0,  1}}, {{ 1,  0, -1}}, {{-1,  0, -1}},
    {{ 0,  1,  1}}, {{ 0, -1,  1}}, {{ 0,  1, -1}}, {{ 0, -1, -1}},
    {{ 1,  1,  0}}, {{ 0, -1,  1}}, {{-1,  1,  0}}, {{ 0, -1, -1}}
}};
double maintainPrecision(double v) {{ return v - trunc(v / 3.3554432E7 + 0.5) * 3.3554432E7; }}
double perlinFadeD(double v) {{ return v * v * v * (v * (v * 6.0 - 15.0) + 10.0); }}
double lerpD(double d, double s, double e) {{ return s + d * (e - s); }}
int mapPermD(int octBase, int v) {{ return int(permBuf.perm[octBase * 256 + uint(v & 255)]); }}
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
// 5 参数 sample（double，含 y 轴 smear），origin 从 OriginBuf 读
double pn_sample5(int octBase, double x, double y, double z, double yScale, double yMax) {{
    double d = x + originBuf.origin[octBase * 3 + 0];
    double e = y + originBuf.origin[octBase * 3 + 1];
    double f = z + originBuf.origin[octBase * 3 + 2];
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

// ===== float Perlin（NormalNoise 用）=====
// 单 octave float 采样：hash 用 int32（精确），grad/fade/lerp 用 float（~1e-7）
float gradDotF(int hash, float x, float y, float z) {{
    vec3 g = vec3(float(GRADIENTS[hash & 15][0]), float(GRADIENTS[hash & 15][1]), float(GRADIENTS[hash & 15][2]));
    return g.x * x + g.y * y + g.z * z;
}}
float pn_sample3_f32(int octBase, int sx, int sy, int sz, float lx, float ly, float lz) {{
    int i = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
    int k = mapPermD(octBase, i + sy); int l = mapPermD(octBase, i + sy + 1);
    int m = mapPermD(octBase, j + sy); int n = mapPermD(octBase, j + sy + 1);
    float d = gradDotF(mapPermD(octBase, k + sz),     lx,     ly,     lz);
    float e = gradDotF(mapPermD(octBase, m + sz),     lx - 1.0f, ly,     lz);
    float f = gradDotF(mapPermD(octBase, l + sz),     lx,     ly - 1.0f, lz);
    float g = gradDotF(mapPermD(octBase, n + sz),     lx - 1.0f, ly - 1.0f, lz);
    float h = gradDotF(mapPermD(octBase, k + sz + 1), lx,     ly,     lz - 1.0f);
    float o = gradDotF(mapPermD(octBase, m + sz + 1), lx - 1.0f, ly,     lz - 1.0f);
    float p = gradDotF(mapPermD(octBase, l + sz + 1), lx,     ly - 1.0f, lz - 1.0f);
    float q = gradDotF(mapPermD(octBase, n + sz + 1), lx - 1.0f, ly - 1.0f, lz - 1.0f);
    float r = perlinFadeF(lx); float s = perlinFadeF(ly); float t = perlinFadeF(lz);
    float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
    float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
    float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
    return lerpF(t, y0, y1);
}}
// OctavePerlinNoiseSampler（float 采样 + double 坐标拆分）：从 params 读 origin，返回叠加值
double octave_noise_f32(int octBase, int nOct, double dx, double dy, double dz,
                        double lacunarity, double persistence) {{
    double d = 0.0; double e = lacunarity; double f = persistence;
    for (int i = 0; i < nOct; i++) {{
        double cx = maintainPrecision(dx * e);
        double cy = maintainPrecision(dy * e);
        double cz = maintainPrecision(dz * e);
        int ix = int(floor(cx)); int iy = int(floor(cy)); int iz = int(floor(cz));
        float gx = float(cx - double(ix)); float gy = float(cy - double(iy)); float gz = float(cz - double(iz));
        float n = pn_sample3_f32(octBase + i, ix, iy, iz, gx, gy, gz);
        d += double(n) * f;   // amplitude 系数运行时上传（Phase 2），此处先简化
        e *= 2.0; f /= 2.0;
    }}
    return d;
}}

{funcs_src}

float eval_density(int sIdx, int ix, int iy, int iz) {{
    float x = float(ix), y = float(iy), z = float(iz);
    return {expr};
}}

void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_density(int(idx), ix, iy, iz);
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
