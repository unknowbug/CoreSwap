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
        # ---- spline SSBO 数据驱动（B1a：56 函数 → 1 个 spline_eval）----
        self.spline_ssbo_nodes = []      # [dict(coordType, n, locBegin, derBegin, valBegin)]
        self.spline_ssbo_locs = []       # 全部 nodes 的 locations 连续拼
        self.spline_ssbo_ders = []       # 全部 nodes 的 derivatives 连续拼
        self.spline_ssbo_val_kind = []   # value kind: 0=const, 1=nested spline node idx
        self.spline_ssbo_val_f = []      # const value（kind=0 用）
        self.spline_ssbo_val_node = []   # nested node idx（kind=1 用）
        self.spline_coords = []          # [coordExpr per coordType]（去重，4 种）
        self.spline_coord_map = {}       # coord expr json -> coordType
        # ---- normal_noise 数据驱动（C2：139 函数 → 1 个 normal_noise + 参数表）----
        self.normal_meta = []            # 每实例 {idx, n, octBase, splitBase, persistence, amplitude, amps}
        # ---- old_blended 数据驱动（D2：8 函数 → 1 个 interp_noise + 参数表）----
        self.old_meta = []               # 每实例 {idx, octBase, splitBase}
        # ---- DF 节点函数化（D1：镜像 C2ME newMethod/callDelegate，根治 68KB 展开）----
        self.node_func_cache = {}        # df json -> df_N 索引（结构去重）
        self.node_funcs = []             # [(idx, body_expr)]，body 用子节点 df_M(...) 调用
        self.node_depth = 0              # 节点函数化递归深度（调试/上限）
        self.node_mode = False           # True = gen_node 路径（registry 用形参坐标）
        # ---- D2：DF 树解释器（节点类型分派 + 数据 buffer）----
        self.spline_bind_base = 6        # P2-2: spline 6 表 SSBO binding 起始号（6-11），生成器统一产出
        self.df_nodes = []               # 每节点 {type, a1, a2, a3, f0, f1, f2, f3}（后序，子节点先）
        self.df_node_cache = {}          # 节点结构 key -> 索引（去重）
        # ---- D2 结构共享（方案1：噪声 slot 化 + 角点运行时实例查表）----
        self.noise_slots = []            # 每 slot {kind, key(去suffix), is_corner, base, stride}
        self.slot_index = {}             # (kind, noise_key, is_corner) -> slot id
        self.in_interp_corner = False    # 当前在 interp 角点上下文内（delegate 序列化时 True）
        self.corner_idx = 0              # 当前角点 c（0..7，eval 时查表用）
        self.slot_mode = False           # True = gen() 生成 slot 化噪声引用（spline coordinate 用）
        self.split_visited = set()       # gen_cpu 时已生成的 split 行 key（防 spline coordinate 重复引用爆炸）
        self.interp_roots = []           # 每个 interp 的 delegate_root（方案1e：每 interp 独立解释器副本）
        self.interp_root_pos = []        # 每个 interp 的 delegate_root 闭包位置（gen_shader 生成 interp_N 用）
        self._val_layout = None          # D19：val 布局缓存（eval_df_glsl + gen_cpu 共用 per_sample）
        self.noise_params = {}
        # D20 诊断：编译时间二分（DFC_DIAG=no_old/no_spline/no_normal，逗号分隔），仅用于测量编译瓶颈，非正确性输出
        self.diag = os.environ.get('DFC_DIAG', '')
        # 坐标链（CPU 预拆分）：主噪声的坐标链描述 + shift 噪声参数
        self.coord_chains = []        # 每个 normal 实例的坐标链（type/scale/shift/flat_cache）
        self.shift_noises = {}        # shift 噪声 noise_key -> {firstOctave, amplitudes}（CPU double 采样）
        self.flat_cache_depth = 0     # 当前在 flat_cache 内的嵌套深度
        # 坐标变量（gen_with_coords 可切换，用于 flat_cache 的 biome 对齐）
        self.cx, self.cy, self.cz = "ix", "iy", "iz"     # int 块坐标
        self.fx, self.fy, self.fz = "x", "y", "z"        # float 坐标
        self.sidx = "sIdx"                               # 拆分坐标采样点索引（interpolated 内切到角点索引）
        self.interp_instances = []                       # interpolated 实例（delegate DF），gen 时收集
        self.interp_funcs = []                           # [(interp_idx, samples[8])]，interp 包装函数
        self.noise_key_suffix = ""                       # interpolated 角点去重后缀（8 个独立角点实例）
        self.interp_depth = 0                            # interpolated 嵌套深度（>0 时 registry 引用展开不函数化）
        self.normal_chain_index = {}                     # normal 实例 key → coord_chains 索引
        self.normal_vec_index = {}                       # normal 实例 key → normals vector 索引
        self.old_vec_index = {}                          # old_blended 实例 key → oldBlendeds vector 索引
        self.normal_split_base = {}                      # normal 实例 key → splitBase
        self.old_split_base = {}                         # old_blended 实例 key → splitBase
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
        if self.interp_depth > 0:
            # interpolated 内：展开（每个角点独立注册 normal/old_blended，noise_key_suffix 含角点）
            if self.node_mode:
                # D1：节点函数化路径——registry 引用直接返回函数调用（形参坐标），不展开
                return self._gen_registry_call_node(ref)
            return self.gen(self.resolve_ref(ref))
        if ref in self.registry_funcs:
            return f"{self.registry_funcs[ref]}({self.sidx}, {self.cx}, {self.cy}, {self.cz})"   # 用当前坐标上下文（flat_cache 对齐后）
        fname = "df_" + ref.replace("minecraft:", "").replace("/", "_").replace(".", "_")
        self.registry_funcs[ref] = fname          # 先注册（防循环引用）
        df = self.resolve_ref(ref)
        if self.node_mode:
            expr = self.gen_node(df)             # D1：节点函数化递归（registry body 也节点化，不展开）
        else:
            expr = self.gen(df)
        self.registry_defs.append((fname, expr))
        if self.node_mode:
            return f"{fname}({self.sidx}, ix, iy, iz)"
        return f"{fname}({self.sidx}, {self.cx}, {self.cy}, {self.cz})"

    def _gen_registry_call_node(self, ref):
        """D2 版：registry 引用 → eval_df 节点调用（形参坐标，调用点传实际坐标）。
        registry body 用 gen_df 序列化 + eval_df 求值，不展开。"""
        if ref in self.registry_funcs:
            return f"{self.registry_funcs[ref]}({self.sidx}, ix, iy, iz)"
        fname = "df_" + ref.replace("minecraft:", "").replace("/", "_").replace(".", "_")
        self.registry_funcs[ref] = fname
        df = self.resolve_ref(ref)
        root = self.gen_df(df)                 # D2：registry body 序列化为节点数组
        self.registry_defs.append((fname, f"eval_df({root}, 0, sIdx, ix, iy, iz)"))
        return f"{fname}({self.sidx}, ix, iy, iz)"

    # ---- 噪声实例注册（运行时从 seed 生成参数，这里只收集 + 分配索引）----
    def _register_noise(self, kind, key, params):
        if key in self.noise_index:
            return self.noise_index[key]
        idx = len(self.noise_instances)
        params["_key"] = key
        self.noise_instances.append((kind, params))
        self.noise_index[key] = idx
        return idx

    def _noise_slot(self, kind, noise_key, params, chain, is_corner):
        """噪声 slot 分配（方案1 结构共享）：返回 slot id。
        is_corner=True：角点独立噪声（interp 角点内非 flat_cache），8 份连续实例
            （key=noise_key@c0..c7），stride=1，运行时实例索引 = base + corner。
        is_corner=False：共享噪声（flat_cache 内或 interp 外），1 份实例，stride=0。
        chain：normal 实例的坐标链描述（每实例一份，内容跨角点相同）。"""
        skey = (kind, noise_key, is_corner)
        if skey in self.slot_index:
            return self.slot_index[skey]
        slot = len(self.noise_slots)
        if is_corner:
            base = len(self.noise_instances)
            for c in range(8):
                key = noise_key + f"@c{c}"
                self._register_noise(kind, key, dict(params))
                if kind == "normal":
                    self.coord_chains.append(dict(chain))
                    self.normal_chain_index[key] = len(self.coord_chains) - 1
            self.noise_slots.append({"kind": kind, "key": noise_key, "is_corner": True, "base": base, "stride": 1})
        else:
            key = noise_key
            idx = self._register_noise(kind, key, params)
            if kind == "normal":
                self.coord_chains.append(dict(chain))
                self.normal_chain_index[key] = len(self.coord_chains) - 1
            self.noise_slots.append({"kind": kind, "key": noise_key, "is_corner": False, "base": idx, "stride": 0})
        self.slot_index[skey] = slot
        return slot

    # ---- D2：DF 树解释器（节点类型分派 + 数据）----
    # 节点类型枚举（与 GLSL eval_df 一致）
    DF_CONSTANT, DF_Y, DF_NOISE, DF_OLD_BLENDED, DF_SPLINE, DF_INTERP, \
    DF_ADD, DF_MUL, DF_MIN, DF_MAX, DF_ABS, DF_SQUARE, DF_CUBE, \
    DF_HALF_NEG, DF_QUARTER_NEG, DF_SQUEEZE, DF_CLAMP, \
    DF_RANGE_CHOICE, DF_Y_CLAMPED, DF_SHIFTED_NOISE, DF_BLEND_DENSITY, \
    DF_FLAT_CACHE, DF_WEIRD = range(23)

    def _df_node(self, t, a1=-1, a2=-1, a3=-1, f0=0.0, f1=0.0, f2=0.0, f3=0.0):
        """建节点（去重：相同 type+args+params 共享索引）。返回节点索引。"""
        key = (t, a1, a2, a3, f0, f1, f2, f3)
        if key in self.df_node_cache:
            return self.df_node_cache[key]
        idx = len(self.df_nodes)
        self.df_nodes.append({"type": t, "a1": a1, "a2": a2, "a3": a3,
                              "f0": f0, "f1": f1, "f2": f2, "f3": f3})
        self.df_node_cache[key] = idx
        return idx

    def gen_df(self, df):
        """DF 树 → 节点数组（后序，子节点先编号）。返回根节点索引。
        叶子（constant/y/noise/spline/interp/shift/y_clamped）直接建节点；
        算术（add/mul/...）先建子节点。flat_cache 用对齐坐标切换（子节点坐标在 eval_df 栈帧内切换）。"""
        if isinstance(df, (int, float)):
            return self._df_node(self.DF_CONSTANT, f0=float(df))
        if isinstance(df, str):
            if df == "minecraft:y":
                return self._df_node(self.DF_Y)
            if df == "minecraft:zero":
                return self._df_node(self.DF_CONSTANT, f0=0.0)
            if df == "minecraft:shift_x":
                return self.gen_df({"type": "minecraft:shift_a"})
            if df == "minecraft:shift_z":
                return self.gen_df({"type": "minecraft:shift_b"})
            return self.gen_df(self.resolve_ref(df))     # registry 引用 → 展开（节点数组去重）
        if isinstance(df, dict) and "points" in df and "coordinate" in df and "type" not in df:
            return self._df_spline_node(df)
        t = df.get("type", "")
        if t == "minecraft:y":
            # flat_cache 内 y 对齐到 0（flat_cache sample 语义：y=0）
            return self._df_node(self.DF_CONSTANT, f0=0.0) if self.flat_cache_depth > 0 else self._df_node(self.DF_Y)
        if t == "minecraft:constant":
            return self._df_node(self.DF_CONSTANT, f0=float(df.get("value", 0.0)))
        if t == "minecraft:old_blended_noise":
            obbase = (f"old_blended:{df.get('xz_scale',0.25)}:{df.get('y_scale',0.125)}:"
                      f"{df.get('xz_factor',80.0)}:{df.get('y_factor',160.0)}:"
                      f"{df.get('smear_scale_multiplier',8.0)}")
            is_corner = self.in_interp_corner
            slot = self._noise_slot("old_blended", obbase, {
                "xz_scale": df.get("xz_scale", 0.25), "y_scale": df.get("y_scale", 0.125),
                "xz_factor": df.get("xz_factor", 80.0), "y_factor": df.get("y_factor", 160.0),
                "smear": df.get("smear_scale_multiplier", 8.0),
            }, None, is_corner)
            return self._df_node(self.DF_OLD_BLENDED, a1=slot)
        if t == "minecraft:noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            nk = df.get("noise", "")
            is_corner = self.in_interp_corner
            chain = {
                "type": "noise", "noise_key": nk,
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
            }
            slot = self._noise_slot("normal", nk, {
                "noise": nk, "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            }, chain, is_corner)
            return self._df_node(self.DF_NOISE, a1=slot)
        if t == "minecraft:shifted_noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            nk = df.get("noise", "")
            is_corner = self.in_interp_corner
            chain = {
                "type": "shifted_noise", "noise_key": nk,
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
                "shift_x": self._resolve_shift(df.get("shift_x", 0.0)),
                "shift_y": self._resolve_shift(df.get("shift_y", 0.0)),
                "shift_z": self._resolve_shift(df.get("shift_z", 0.0)),
            }
            slot = self._noise_slot("normal", nk, {
                "noise": nk, "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            }, chain, is_corner)
            return self._df_node(self.DF_SHIFTED_NOISE, a1=slot)
        if t in ("minecraft:shift_a", "minecraft:shift_b", "minecraft:shift"):
            self._resolve_shift(df)
            return self._df_node(self.DF_CONSTANT, f0=0.0)
        if t == "minecraft:spline":
            return self._df_spline_node(df.get("spline", df))
        if t == "minecraft:add":
            return self._df_node(self.DF_ADD, a1=self.gen_df(df["argument1"]), a2=self.gen_df(df["argument2"]))
        if t == "minecraft:mul":
            return self._df_node(self.DF_MUL, a1=self.gen_df(df["argument1"]), a2=self.gen_df(df["argument2"]))
        if t == "minecraft:min":
            return self._df_node(self.DF_MIN, a1=self.gen_df(df["argument1"]), a2=self.gen_df(df["argument2"]))
        if t == "minecraft:max":
            return self._df_node(self.DF_MAX, a1=self.gen_df(df["argument1"]), a2=self.gen_df(df["argument2"]))
        if t == "minecraft:abs":
            return self._df_node(self.DF_ABS, a1=self.gen_df(df["argument"]))
        if t == "minecraft:square":
            return self._df_node(self.DF_SQUARE, a1=self.gen_df(df["argument"]))
        if t == "minecraft:cube":
            return self._df_node(self.DF_CUBE, a1=self.gen_df(df["argument"]))
        if t == "minecraft:half_negative":
            return self._df_node(self.DF_HALF_NEG, a1=self.gen_df(df["argument"]))
        if t == "minecraft:quarter_negative":
            return self._df_node(self.DF_QUARTER_NEG, a1=self.gen_df(df["argument"]))
        if t == "minecraft:squeeze":
            return self._df_node(self.DF_SQUEEZE, a1=self.gen_df(df["argument"]))
        if t == "minecraft:clamp":
            return self._df_node(self.DF_CLAMP, a1=self.gen_df(df["input"]), f0=float(df["min"]), f1=float(df["max"]))
        if t == "minecraft:range_choice":
            return self._df_node(self.DF_RANGE_CHOICE,
                                 a1=self.gen_df(df["input"]), a2=self.gen_df(df["when_in_range"]), a3=self.gen_df(df["when_out_of_range"]),
                                 f0=float(df["min_inclusive"]), f1=float(df["max_exclusive"]))
        if t == "minecraft:y_clamped_gradient":
            # flat_cache 内 y 对齐到 0（flat_cache sample 语义）
            if self.flat_cache_depth > 0:
                # y_clamped_gradient(y=0, ...) 直接计算常量
                fy, ty = float(df["from_y"]), float(df["to_y"])
                fv, tv = float(df["from_value"]), float(df["to_value"])
                tt = max(0.0, min(1.0, (0.0 - fy) / (ty - fy))) if ty != fy else 0.0
                return self._df_node(self.DF_CONSTANT, f0=fv + tt * (tv - fv))
            return self._df_node(self.DF_Y_CLAMPED, f0=float(df["from_y"]), f1=float(df["to_y"]), f2=float(df["from_value"]), f3=float(df["to_value"]))
        if t == "minecraft:weird_scaled_sampler":
            # D17: 实现 ws（原 stub 0.0f → entrances Y 分支错值，when_out 绑定错误）
            # 结构：d = scaleValue(rarity, input)；r = d * |noise(x/d, y/d, z/d)|
            # ws 噪声按 normal 实例注册（split 在 /d 坐标由 CPU split() 生成）；
            # rarity 输入为普通噪声槽（其 split 在正常坐标）。
            inp = self.gen_df(df.get("input", 0.0))
            nk_ws = df.get("noise", "")
            np_ws = self._resolve_noise_params(nk_ws)
            ws_chain = {
                "type": "noise", "noise_key": nk_ws,
                "xz_scale": 1.0, "y_scale": 1.0,
                "flat_cache": self.flat_cache_depth > 0,
                "ws": True,   # 标记：split 需 /d 坐标（_gen_split_lines 特判）
            }
            ws_slot = self._noise_slot("normal", nk_ws + ":ws", {
                "noise": nk_ws, "xz_scale": 1.0, "y_scale": 1.0,
                "firstOctave": np_ws["firstOctave"], "amplitudes": np_ws["amplitudes"],
                "ws": True,
            }, ws_chain, self.in_interp_corner)
            kind = 1.0 if df.get("rarity_value_mapper") == "type_2" else 0.0   # type_2=CAVES, type_1=TUNNELS
            return self._df_node(self.DF_WEIRD, a1=inp, a2=ws_slot, f0=kind)
        if t == "minecraft:flat_cache":
            # flat_cache：对齐坐标（x>>2<<2, 0, z>>2<<2）采样 delegate（eval_df 栈帧内切换坐标）
            # 注：flat_cache 内噪声仍角点独立（8 角点不同 4×4 列，density.h FlatCacheDF.k 判定）——
            #     共享是错的（HEAD 的 spline 结构去重曾把 coordinate 噪声错误共享成 @c0）。
            self.flat_cache_depth += 1
            inner = self.gen_df(df["argument"])
            self.flat_cache_depth -= 1
            return self._df_node(self.DF_FLAT_CACHE, a1=inner)
        if t in ("minecraft:cache_2d", "minecraft:cache_once", "minecraft:cache_all_in_cell"):
            return self.gen_df(df.get("argument", df.get("input", 0.0)))
        if t == "minecraft:interpolated":
            return self._df_interp_node(df)
        if t == "minecraft:blend_alpha":
            return self._df_node(self.DF_CONSTANT, f0=1.0)
        if t == "minecraft:blend_offset":
            return self._df_node(self.DF_CONSTANT, f0=0.0)
        if t == "minecraft:blend_density":
            return self._df_node(self.DF_BLEND_DENSITY, a1=self.gen_df(df.get("argument", 0.0)))
        raise ValueError(f"D2: 未处理的 DF 类型 {t}")

    def _df_spline_node(self, spline):
        """spline → spline_eval 节点（复用 B1a 数据驱动）。返回节点索引。
        a2 = 对齐标志（1=flat_cache 内，坐标用 (x>>2)<<2, 0, (z>>2)<<2）。"""
        call = self._gen_spline(spline)
        import re as _re
        m = _re.match(r"spline_eval\((\d+),", call)
        node_idx = int(m.group(1)) if m else 0
        aligned = 1 if self.flat_cache_depth > 0 else 0
        return self._df_node(self.DF_SPLINE, a1=node_idx, a2=aligned)

    def _df_interp_node(self, df):
        """interpolated → interp_N 节点（方案1：delegate 结构只序列化 1 份，8 角点共享）。
        角点差异（噪声实例索引）由 slot 表 + corner 参数运行时查表；角点坐标作为
        eval_df_base 的 ix/iy/iz 实参传入。"""
        arg = df.get("argument", df.get("input", 0.0))
        interp_idx = len(self.interp_instances)
        self.interp_instances.append(arg)
        samples = []
        self.interp_depth += 1
        old_in_corner = self.in_interp_corner
        self.in_interp_corner = True
        # 结构共享：delegate 序列化一次（噪声 slot 化 → 节点数组跨角点共享）
        delegate_root = self.gen_df(arg)
        self.interp_roots.append(delegate_root)
        for c in range(8):
            dx = c & 1; dy = (c >> 1) & 1; dz = (c >> 2) & 1
            self.corner_idx = c
            # 角点坐标用 interp_N 函数体内的局部变量表达（chunkX/cx/cy 等）
            ax = f"(chunkX * 16 + (cx + {dx}) * 4)"
            ay = f"(minY + (cy + {dy}) * 8)"
            az = f"(chunkZ * 16 + (cz + {dz}) * 4)"
            samples.append(f"eval_df_base_{interp_idx}(__ROOT__, {c}, sIdx, {ax}, {ay}, {az})")
        self.corner_idx = 0
        self.in_interp_corner = old_in_corner
        self.interp_depth -= 1
        self.interp_funcs.append((interp_idx, samples))
        return self._df_node(self.DF_INTERP, a1=interp_idx)

    # ---- D2：eval_df 解释器 GLSL 生成 ----
    def _compute_val_layout(self):
        """计算 val 布局（闭包/活跃分析/per_sample/BASE_N）——eval_df_glsl 与 gen_cpu 共用。
        D19：per_sample 必须随节点/槽数变化（ws 实现后 320→352），gen_cpu 侧用于生成
        CpuBackend.perSample（e2e 分配 valBuf 的大小依据），防硬编码陈旧越界。"""
        if self._val_layout is not None:
            return self._val_layout
        nodes = self.df_nodes
        n_nodes = len(nodes)
        read_fields = {6: ('a1', 'a2'), 7: ('a1', 'a2'), 8: ('a1', 'a2'), 9: ('a1', 'a2'),
                       10: ('a1',), 11: ('a1',), 12: ('a1',), 13: ('a1',), 14: ('a1',), 15: ('a1',), 16: ('a1',),
                       17: ('a1', 'a2', 'a3'), 20: ('a1',), 21: ('a1',), 22: ('a1',)}
        def closure_of(root):
            reach = set()
            def visit(i):
                if i < 0 or i >= n_nodes or i in reach:
                    return
                reach.add(i)
                n = nodes[i]
                if n["type"] == self.DF_WEIRD:
                    visit(n["a1"])   # a2 = ws 噪声 slot id（非节点索引，勿递归）
                    return
                visit(n["a1"]); visit(n["a2"]); visit(n["a3"])
            visit(root)
            return sorted(reach)
        def liveness(indices):
            idx_set = set(indices)
            pos = {i: p for p, i in enumerate(indices)}
            mp = {}
            for p, i in enumerate(indices):
                n = nodes[i]
                if n["type"] in read_fields:
                    for f in read_fields[n["type"]]:
                        c = n[f]
                        if c in idx_set:
                            mp[c] = max(mp.get(c, -1), i)
            slot = [-1] * len(indices)
            peak = 0
            for p, i in enumerate(indices):
                used = set()
                for q in range(p):
                    if slot[q] >= 0 and mp.get(indices[q], -1) >= i:
                        used.add(slot[q])
                s = 0
                while s in used:
                    s += 1
                slot[p] = s
                peak = max(peak, len(used) + 1)
            return pos, slot, peak
        closures = []
        for root in self.interp_roots:
            closure = closure_of(root)
            pos, slot, peak = liveness(closure)
            closures.append((closure, pos, slot, peak))
        top_closure = closure_of(n_nodes - 1)
        top_pos, top_slot, top_peak = liveness(top_closure)
        self.top_root_pos = top_pos[n_nodes - 1] if (n_nodes - 1) in top_pos else 0
        val_slots_top = top_peak
        per_sample = val_slots_top + 8 * sum(pk for _, _, _, pk in closures)
        bases = []
        acc = val_slots_top
        for _, _, _, pk in closures:
            bases.append(acc)
            acc += 8 * pk
        self.per_sample = per_sample
        self._val_layout = {
            "read_fields": read_fields, "closures": closures,
            "top_closure": top_closure, "top_pos": top_pos, "top_slot": top_slot, "top_peak": top_peak,
            "val_slots_top": val_slots_top, "per_sample": per_sample, "bases": bases,
        }
        return self._val_layout

    def eval_df_glsl(self):
        """生成 eval_df 显式栈解释器 + 节点 const 数组。
        显式栈后序求值：叶子出值，算术等子节点值齐后运算。flat_cache 在栈帧内切换对齐坐标。"""
        nodes = self.df_nodes
        if not nodes:
            return ""
        layout = self._compute_val_layout()
        read_fields = layout["read_fields"]
        closures = layout["closures"]
        top_closure = layout["top_closure"]
        top_pos = layout["top_pos"]
        top_slot = layout["top_slot"]
        val_slots_top = layout["val_slots_top"]
        per_sample = layout["per_sample"]
        bases = layout["bases"]
        def flit(x):
            s = format(float(x), '.17g')
            if '.' not in s and 'e' not in s and 'E' not in s:
                s += '.0'
            return s + 'f'
        types = ", ".join(str(n["type"]) for n in nodes)
        a1s = ", ".join(str(n["a1"]) for n in nodes)
        a2s = ", ".join(str(n["a2"]) for n in nodes)
        a3s = ", ".join(str(n["a3"]) for n in nodes)
        f0s = ", ".join(flit(n["f0"]) for n in nodes)
        f1s = ", ".join(flit(n["f1"]) for n in nodes)
        f2s = ", ".join(flit(n["f2"]) for n in nodes)
        f3s = ", ".join(flit(n["f3"]) for n in nodes)
        n_nodes = len(nodes)
        # 生成专属数组 + eval_df_base_N（每 interp）
        eval_bases = []
        for idx, (closure, pos, slot, peak) in enumerate(closures):
            ctype, ca1, ca2, ca3 = [], [], [], []
            cf0, cf1, cf2, cf3 = [], [], [], []
            for ci, i in enumerate(closure):
                n = nodes[i]
                t = n["type"]
                ctype.append(t)
                def map_a(v, f):
                    if v >= 0 and v in pos and f in read_fields.get(t, ()):
                        return pos[v]
                    return v
                ca1.append(map_a(n["a1"], "a1")); ca2.append(map_a(n["a2"], "a2")); ca3.append(map_a(n["a3"], "a3"))
                cf0.append(flit(n["f0"])); cf1.append(flit(n["f1"])); cf2.append(flit(n["f2"])); cf3.append(flit(n["f3"]))
            K = len(closure)
            B = f"{bases[idx]}"
            # D20 诊断：编译时间二分——分派简化（no_normal/no_old/no_spline 时对应节点返回 0）
            _noise_disp = (f"r = normal_noise(NOISE_SLOT_BASE[CA1_{idx}[ci]] + corner * NOISE_SLOT_STRIDE[CA1_{idx}[ci]], sIdx);"
                           if 'no_normal' not in self.diag else "r = 0.0;")
            _old_disp = (f"r = interp_noise(NOISE_SLOT_BASE[CA1_{idx}[ci]] + corner * NOISE_SLOT_STRIDE[CA1_{idx}[ci]], sIdx);"
                         if 'no_old' not in self.diag else "r = 0.0;")
            _spline_disp = (f"if (CA2_{idx}[ci] == 1) r = spline_eval(CA1_{idx}[ci], corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2);\n"
                            f"            else r = spline_eval(CA1_{idx}[ci], corner, sIdx, ix, iy, iz);"
                            if 'no_spline' not in self.diag else "r = 0.0;")
            body = f"""
// ---- interp_{idx} 独立解释器（D16：单调用者防驱动强制内联 TDR）----
const int CLOSURE_{idx}_LEN = {K};
const int VAL_SLOTS_{idx} = {peak};
const int CTYPE_{idx}[{K}] = int[]({", ".join(str(x) for x in ctype)});
const int CA1_{idx}[{K}] = int[]({", ".join(str(x) for x in ca1)});
const int CA2_{idx}[{K}] = int[]({", ".join(str(x) for x in ca2)});
const int CA3_{idx}[{K}] = int[]({", ".join(str(x) for x in ca3)});
const float CF0_{idx}[{K}] = float[]({", ".join(cf0)});
const float CF1_{idx}[{K}] = float[]({", ".join(cf1)});
const float CF2_{idx}[{K}] = float[]({", ".join(cf2)});
const float CF3_{idx}[{K}] = float[]({", ".join(cf3)});
const int SLOT_OF_{idx}[{K}] = int[]({", ".join(str(x) for x in slot)});
float eval_df_base_{idx}(int rootPos, int corner, int sIdx, int ix, int iy, int iz) {{
    for (int ci = 0; ci < CLOSURE_{idx}_LEN; ci++) {{
        int t = CTYPE_{idx}[ci];
        float r = 0.0;
        if (t == {self.DF_CONSTANT}) r = CF0_{idx}[ci];
        else if (t == {self.DF_Y}) r = float(iy);
        else if (t == {self.DF_NOISE} || t == {self.DF_SHIFTED_NOISE}) {_noise_disp}
        else if (t == {self.DF_OLD_BLENDED}) {_old_disp}
        else if (t == {self.DF_SPLINE}) {{
            {_spline_disp}
        }}
        else if (t == {self.DF_Y_CLAMPED}) r = y_clamped_gradient(iy, CF0_{idx}[ci], CF1_{idx}[ci], CF2_{idx}[ci], CF3_{idx}[ci]);
        else if (t == {self.DF_ABS}) r = abs(valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]);
        else if (t == {self.DF_SQUARE}) {{ float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]; r = v * v; }}
        else if (t == {self.DF_CUBE}) {{ float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]; r = v * v * v; }}
        else if (t == {self.DF_HALF_NEG}) {{ float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]; r = (v > 0.0 ? v : v * 0.5); }}
        else if (t == {self.DF_QUARTER_NEG}) {{ float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]; r = (v > 0.0 ? v : v * 0.25); }}
        else if (t == {self.DF_SQUEEZE}) {{ float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]]; float c = clamp(v, -1.0, 1.0); r = c / 2.0 - c * c * c / 24.0; }}
        else if (t == {self.DF_CLAMP}) r = clamp(valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]], CF0_{idx}[ci], CF1_{idx}[ci]);
        else if (t == {self.DF_RANGE_CHOICE}) {{
            float inp = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]];
            r = (inp >= CF0_{idx}[ci] && inp < CF1_{idx}[ci]) ? valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA2_{idx}[ci]]] : valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA3_{idx}[ci]]];
        }}
        else if (t == {self.DF_WEIRD}) {{
            float v = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]];
            float d = ws_scale(int(CF0_{idx}[ci]), v);
            r = d * abs(normal_noise(NOISE_SLOT_BASE[CA2_{idx}[ci]] + corner * NOISE_SLOT_STRIDE[CA2_{idx}[ci]], sIdx));
        }}
        else if (t == {self.DF_BLEND_DENSITY}) r = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]];
        else if (t == {self.DF_FLAT_CACHE}) r = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]];
        else if (t == {self.DF_ADD}) r = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]] + valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA2_{idx}[ci]]];
        else if (t == {self.DF_MUL}) r = valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]] * valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA2_{idx}[ci]]];
        else if (t == {self.DF_MIN}) r = min(valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]], valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA2_{idx}[ci]]]);
        else if (t == {self.DF_MAX}) r = max(valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA1_{idx}[ci]]], valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[CA2_{idx}[ci]]]);
        valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[ci]] = r;
    }}
    return valBuf[PER_SAMPLE * sIdx + {B} + corner * VAL_SLOTS_{idx} + SLOT_OF_{idx}[rootPos]];
}}
"""
            eval_bases.append(body)
            # 记录 interp_N 用的 root 闭包位置（gen_shader 生成 interp_N 时替换 __ROOT__）
            self.interp_root_pos.append(pos[self.interp_roots[idx]] if self.interp_roots[idx] in pos else 0)
        # 顶层 eval_df（闭包化：循环顶层闭包 ~21 节点，含 DF_INTERP 分支；D16 防驱动强制内联）
        top_ctype, top_ca1, top_ca2, top_ca3 = [], [], [], []
        top_cf0, top_cf1, top_cf2, top_cf3 = [], [], [], []
        for ci, i in enumerate(top_closure):
            n = nodes[i]
            t = n["type"]
            top_ctype.append(t)
            def map_a(v, f):
                if v >= 0 and v in top_pos and f in read_fields.get(t, ()):
                    return top_pos[v]
                return v
            top_ca1.append(map_a(n["a1"], "a1")); top_ca2.append(map_a(n["a2"], "a2")); top_ca3.append(map_a(n["a3"], "a3"))
            top_cf0.append(flit(n["f0"])); top_cf1.append(flit(n["f1"])); top_cf2.append(flit(n["f2"])); top_cf3.append(flit(n["f3"]))
        TK = len(top_closure)
        top_slot_src = ", ".join(str(x) for x in top_slot)
        # D20 诊断：顶层分派简化
        _t_noise_disp = ("r = normal_noise(NOISE_SLOT_BASE[CA1_T[ci]] + corner * NOISE_SLOT_STRIDE[CA1_T[ci]], sIdx);"
                         if 'no_normal' not in self.diag else "r = 0.0;")
        _t_old_disp = ("r = interp_noise(NOISE_SLOT_BASE[CA1_T[ci]] + corner * NOISE_SLOT_STRIDE[CA1_T[ci]], sIdx);"
                       if 'no_old' not in self.diag else "r = 0.0;")
        _t_spline_disp = ("""if (CA2_T[ci] == 1) r = spline_eval(CA1_T[ci], corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2);
            else r = spline_eval(CA1_T[ci], corner, sIdx, ix, iy, iz);"""
                          if 'no_spline' not in self.diag else "r = 0.0;")
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
        else if (t == {self.DF_NOISE} || t == {self.DF_SHIFTED_NOISE}) {_t_noise_disp}
        else if (t == {self.DF_OLD_BLENDED}) {_t_old_disp}
        else if (t == {self.DF_SPLINE}) {{
            {_t_spline_disp}
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
        else if (t == {self.DF_WEIRD}) {{
            float v = valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[CA1_T[ci]]];
            float d = ws_scale(int(CF0_T[ci]), v);
            r = d * abs(normal_noise(NOISE_SLOT_BASE[CA2_T[ci]] + corner * NOISE_SLOT_STRIDE[CA2_T[ci]], sIdx));
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
        glsl = f"""
// ===== D2e：DF 树解释器（方案1e：每 interp 独立解释器副本 + 闭包，防驱动强制内联 TDR）=====
// 节点数组后序（子节点先编号）→ 顺序求值；val 栈 SSBO（每采样点 PER_SAMPLE 槽）
// 方案1：噪声节点 a1 = slot id；实例索引 = NOISE_SLOT_BASE[slot] + corner * NOISE_SLOT_STRIDE[slot]
const int DF_NODES = {n_nodes};
const int DF_TYPE[{n_nodes}] = int[]({types});
const int DF_A1[{n_nodes}] = int[]({a1s});
const int DF_A2[{n_nodes}] = int[]({a2s});
const int DF_A3[{n_nodes}] = int[]({a3s});
const float DF_F0[{n_nodes}] = float[]({f0s});
const float DF_F1[{n_nodes}] = float[]({f1s});
const float DF_F2[{n_nodes}] = float[]({f2s});
const float DF_F3[{n_nodes}] = float[]({f3s});
const int VAL_SLOTS_TOP = {val_slots_top};
const int PER_SAMPLE = {per_sample};
{''.join(eval_bases)}
{eval_top}
"""
        return glsl

    # 思路：算术/组合节点 → 注册独立函数 df_N（body 里子节点用 df_M(...) 调用，不内联展开）；
    #       叶子节点（constant/y/noise/spline/interp/shift）→ 直接返回表达式（已是函数调用/字面量）。
    # 效果：interp 角点 delegate 树变成 df_N 函数链，不再 68KB 展开；每函数体小 → 编译秒级。
    def gen_node(self, df):
        """返回 df 的「节点调用」：叶子返回表达式，非叶子注册 df_N 函数并返回调用。"""
        old_mode = self.node_mode
        self.node_mode = True
        try:
            return self._gen_node_inner(df)
        finally:
            self.node_mode = old_mode

    def _gen_node_inner(self, df):
        """gen_node 核心（node_mode=True 时执行）。"""
        # 叶子判断：直接生成表达式（不再递归展开子节点）
        leaf = self._gen_leaf_expr(df)
        if leaf is not None:
            return leaf
        # 非叶子（算术/组合）：注册节点函数（key 含 noise_key_suffix —— interp 角点内 delegate 树独立注册，与 gen_cpu 拆分一致）
        key = json.dumps(df, sort_keys=True) + self.noise_key_suffix
        if key in self.node_func_cache:
            return f"df_{self.node_func_cache[key]}({self.sidx}, ix, iy, iz)"
        idx = len(self.node_funcs)
        self.node_func_cache[key] = idx      # 先注册（防循环引用）
        self.node_funcs.append((idx, None))  # 占位（防止递归子节点 idx 冲突）
        self.node_depth += 1
        body = self._gen_node_body(df)       # 子节点用 gen_node 调用（不展开）
        self.node_depth -= 1
        self.node_funcs[idx] = (idx, body)   # 回填 body
        return f"df_{idx}({self.sidx}, ix, iy, iz)"

    def _gen_leaf_expr(self, df):
        """叶子节点：返回可直接内联的表达式（常量/坐标/噪声/采样函数调用），非叶子返回 None。
        D1：叶子 = 不包含「递归子节点展开」的节点。"""
        if isinstance(df, (int, float)):
            return f"{float(df)}f"
        if isinstance(df, str):
            if df == "minecraft:y":
                return "iy"    # D1：节点函数化后用形参 iy（调用点传实际坐标）
            if df == "minecraft:zero":
                return "0.0f"
            if df == "minecraft:shift_x":
                return self.gen_node({"type": "minecraft:shift_a"})
            if df == "minecraft:shift_z":
                return self.gen_node({"type": "minecraft:shift_b"})
            return self._gen_registry_call(df)     # registry 引用 → 已函数化
        if isinstance(df, dict) and "points" in df and "coordinate" in df and "type" not in df:
            return self._gen_spline(df)            # spline → spline_eval 调用（已数据驱动）
        t = df.get("type", "")
        if t == "minecraft:y":
            # D1：节点函数化后 y 用形参 iy（节点函数签名统一 (sIdx, ix, iy, iz)，调用点传实际坐标）
            return "iy"
        if t == "minecraft:constant":
            return f"{float(df.get('value', 0.0))}f"
        if t == "minecraft:old_blended_noise":
            obkey = f"old_blended:{df.get('xz_scale',0.25)}:{df.get('y_scale',0.125)}:{df.get('xz_factor',80.0)}:{df.get('y_factor',160.0)}:{df.get('smear_scale_multiplier',8.0)}{self.noise_key_suffix}"
            idx = self._register_noise("old_blended", obkey, {
                "xz_scale": df.get("xz_scale", 0.25), "y_scale": df.get("y_scale", 0.125),
                "xz_factor": df.get("xz_factor", 80.0), "y_factor": df.get("y_factor", 160.0),
                "smear": df.get("smear_scale_multiplier", 8.0),
            })
            return f"interp_noise_{idx}({self.sidx})"
        if t == "minecraft:noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            idx = self._register_noise("normal", df.get("noise", "") + self.noise_key_suffix, {
                "noise": df.get("noise", ""), "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            })
            self.coord_chains.append({
                "type": "noise", "noise_key": df.get("noise", ""),
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
            })
            self.normal_chain_index[df.get("noise", "") + self.noise_key_suffix] = len(self.coord_chains) - 1
            return f"normal_noise({idx}, {self.sidx})"
        if t == "minecraft:shifted_noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            idx = self._register_noise("normal", df.get("noise", "") + self.noise_key_suffix, {
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
            self.normal_chain_index[df.get("noise", "") + self.noise_key_suffix] = len(self.coord_chains) - 1
            return f"normal_noise({idx}, {self.sidx})"
        if t in ("minecraft:shift_a", "minecraft:shift_b", "minecraft:shift"):
            self._resolve_shift(df)
            return "0.0f"
        if t == "minecraft:spline":
            return self._gen_spline(df.get("spline", df))
        if t == "minecraft:y_clamped_gradient":
            # D1：节点函数化后 y 用形参 iy（节点函数签名统一 (sIdx, ix, iy, iz)，调用点传实际坐标）
            return f"y_clamped_gradient(iy, {float(df['from_y'])}f, {float(df['to_y'])}f, {float(df['from_value'])}f, {float(df['to_value'])}f)"
        if t == "minecraft:weird_scaled_sampler":
            return "0.0f"
        if t == "minecraft:blend_alpha":
            return "1.0f"
        if t == "minecraft:blend_offset":
            return "0.0f"
        if t == "minecraft:blend_density":
            return self.gen_node(df.get("argument", 0.0))
        if t == "minecraft:flat_cache":
            # flat_cache：biome 对齐（x>>2<<2, 0, z>>2<<2）→ 注册对齐包装函数
            # delegate 根函数用对齐坐标调用（对齐 vanilla FlatCache.sample）
            key = json.dumps(df, sort_keys=True) + self.noise_key_suffix
            if key in self.node_func_cache:
                return f"df_{self.node_func_cache[key]}({self.sidx}, ix, iy, iz)"
            idx = len(self.node_funcs)
            self.node_func_cache[key] = idx
            self.node_funcs.append((idx, None))   # 占位（防递归子节点 idx 冲突）
            self.flat_cache_depth += 1
            inner_call = self.gen_node(df["argument"])      # delegate 根函数调用（形参 ix/iy/iz）
            self.flat_cache_depth -= 1
            # 对齐包装：df_fc(sIdx, ix, iy, iz) { return delegate(sIdx, (ix>>2)<<2, 0, (iz>>2)<<2); }
            body = self._align_call(inner_call)
            self.node_funcs[idx] = (idx, body)
            return f"df_{idx}({self.sidx}, ix, iy, iz)"
        if t in ("minecraft:cache_2d", "minecraft:cache_once", "minecraft:cache_all_in_cell"):
            return self.gen_node(df.get("argument", df.get("input", 0.0)))
        if t == "minecraft:interpolated":
            # interpolated：8 角点调 delegate 根函数（不展开），插值在主函数
            arg = df.get("argument", df.get("input", 0.0))
            interp_idx = len(self.interp_instances)
            self.interp_instances.append(arg)
            samples = []
            self.interp_depth += 1
            for c in range(8):
                dx = c & 1; dy = (c >> 1) & 1; dz = (c >> 2) & 1
                ax = f"(chunkX * 16 + (cx + {dx}) * 4)"
                ay = f"(minY + (cy + {dy}) * 8)"
                az = f"(chunkZ * 16 + (cz + {dz}) * 4)"
                old_suffix = self.noise_key_suffix
                self.noise_key_suffix = f"@c{c}"
                samples.append(self.gen_with_coords_call(arg, ax, ay, az))
                self.noise_key_suffix = old_suffix
            self.interp_depth -= 1
            self.interp_funcs.append((interp_idx, samples))
            return f"interp_{interp_idx}({self.sidx}, {self.cx}, {self.cy}, {self.cz})"
        return None   # 非叶子（算术/组合），由 _gen_node_body 处理

    def gen_with_coords_call(self, df, cx, cy, cz):
        """interp 角点：临时切坐标后 gen_node（delegate 根函数调用，传角点坐标）。"""
        old = (self.cx, self.cy, self.cz, self.fx, self.fy, self.fz)
        self.cx, self.cy, self.cz = cx, cy, cz
        self.fx, self.fy, self.fz = cx, cy, cz
        try:
            return self.gen_node(df)
        finally:
            (self.cx, self.cy, self.cz, self.fx, self.fy, self.fz) = old

    def _align_call(self, call):
        """flat_cache 对齐：把 delegate 根调用的坐标参数替换为对齐坐标 (x>>2)<<2, 0, (z>>2)<<2。
        call 形如 'df_5(sIdx, ix, iy, iz)' → 'df_5(sIdx, (ix>>2)<<2, 0, (iz>>2)<<2)'。
        仅处理「根节点调用」（首层函数调用），子节点坐标由各自函数形参处理。"""
        import re as _re
        m = _re.match(r"^(df_\d+)\(([^)]*)\)$", call.strip())
        if not m:
            # 非纯函数调用（如常量/表达式）→ 原样（flat_cache 包常量无对齐意义）
            return call
        fname, args = m.group(1), m.group(2)
        parts = [a.strip() for a in args.split(",")]
        if len(parts) == 4:   # (sIdx, ix, iy, iz)
            return f"{fname}({parts[0]}, (ix >> 2) << 2, 0, (iz >> 2) << 2)"
        return call

    def _gen_node_body(self, df):
        """非叶子节点 body：子节点用 gen_node 调用（不展开）。"""
        t = df.get("type", "")
        if t == "minecraft:add":
            return f"({self.gen_node(df['argument1'])} + {self.gen_node(df['argument2'])})"
        if t == "minecraft:mul":
            return f"({self.gen_node(df['argument1'])} * {self.gen_node(df['argument2'])})"
        if t == "minecraft:min":
            return f"min({self.gen_node(df['argument1'])}, {self.gen_node(df['argument2'])})"
        if t == "minecraft:max":
            return f"max({self.gen_node(df['argument1'])}, {self.gen_node(df['argument2'])})"
        if t == "minecraft:abs":
            return f"abs({self.gen_node(df['argument'])})"
        if t == "minecraft:square":
            v = self.gen_node(df['argument']); return f"({v} * {v})"
        if t == "minecraft:cube":
            v = self.gen_node(df['argument']); return f"({v} * {v} * {v})"
        if t == "minecraft:half_negative":
            v = self.gen_node(df['argument']); return f"({v} > 0.0f ? {v} : {v} * 0.5f)"
        if t == "minecraft:quarter_negative":
            v = self.gen_node(df['argument']); return f"({v} > 0.0f ? {v} : {v} * 0.25f)"
        if t == "minecraft:squeeze":
            v = self.gen_node(df['argument'])
            return f"(clamp({v}, -1.0f, 1.0f) / 2.0f - clamp({v}, -1.0f, 1.0f) * clamp({v}, -1.0f, 1.0f) * clamp({v}, -1.0f, 1.0f) / 24.0f)"
        if t == "minecraft:clamp":
            return f"clamp({self.gen_node(df['input'])}, {float(df['min'])}f, {float(df['max'])}f)"
        if t == "minecraft:range_choice":
            inp = self.gen_node(df['input'])
            return f"(({inp} >= {float(df['min_inclusive'])}f && {inp} < {float(df['max_exclusive'])}f) ? {self.gen_node(df['when_in_range'])} : {self.gen_node(df['when_out_of_range'])})"
        raise ValueError(f"D1: 未处理的非叶子节点类型 {t}")

    def _reset_collect(self):
        """重置 D1 收集（gen_shader 幂等：清空 node/interp/spline/noise 收集，重新遍历）。"""
        self.node_func_cache = {}
        self.node_funcs = []
        self.interp_funcs = []
        self.interp_instances = []
        self.spline_funcs = []
        self.spline_cache = {}
        self.spline_ssbo_nodes = []
        self.spline_ssbo_locs = []
        self.spline_ssbo_ders = []
        self.spline_ssbo_val_kind = []
        self.spline_ssbo_val_f = []
        self.spline_ssbo_val_node = []
        self.spline_coords = []
        self.spline_coord_map = {}
        self.noise_instances = []
        self.noise_index = {}
        self.normal_meta = []
        self.registry_funcs = {}
        self.registry_defs = []
        self.coord_chains = []
        self.normal_chain_index = {}
        self.normal_vec_index = {}
        self.old_vec_index = {}
        self.normal_split_base = {}
        self.old_split_base = {}
        self.df_nodes = []
        self.df_node_cache = {}
        self.noise_slots = []
        self.slot_index = {}
        self.in_interp_corner = False
        self.corner_idx = 0
        self.slot_mode = False
        self.split_visited = set()
        self.interp_roots = []
        self.interp_root_pos = []
        self.old_meta = []
        self._val_layout = None   # D19：val 布局缓存（eval_df_glsl + gen_cpu 共用 per_sample）

    # ---- 主入口：生成 DF 节点的 GLSL 表达式（float 语义，old_blended_noise 内部 double 转 float）----
    def gen(self, df):
        if isinstance(df, (int, float)):
            return f"{float(df)}f"
        if isinstance(df, str):
            if df == "minecraft:y":
                return self.fy
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
            obbase = (f"old_blended:{df.get('xz_scale',0.25)}:{df.get('y_scale',0.125)}:"
                      f"{df.get('xz_factor',80.0)}:{df.get('y_factor',160.0)}:"
                      f"{df.get('smear_scale_multiplier',8.0)}")
            params = {
                "xz_scale": df.get("xz_scale", 0.25), "y_scale": df.get("y_scale", 0.125),
                "xz_factor": df.get("xz_factor", 80.0), "y_factor": df.get("y_factor", 160.0),
                "smear": df.get("smear_scale_multiplier", 8.0),
            }
            if self.slot_mode:
                slot = self._noise_slot("old_blended", obbase, params, None, self.in_interp_corner)
                return f"interp_noise(NOISE_SLOT_BASE[{slot}] + corner * NOISE_SLOT_STRIDE[{slot}], {self.sidx})"
            idx = self._register_noise("old_blended", obbase + self.noise_key_suffix, params)
            return f"interp_noise({idx}, {self.sidx})"
        if t == "minecraft:noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            nk = df.get("noise", "")
            params = {
                "noise": nk, "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            }
            if self.slot_mode:
                chain = {
                    "type": "noise", "noise_key": nk,
                    "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                    "flat_cache": self.flat_cache_depth > 0,
                }
                slot = self._noise_slot("normal", nk, params, chain, self.in_interp_corner)
                return f"normal_noise(NOISE_SLOT_BASE[{slot}] + corner * NOISE_SLOT_STRIDE[{slot}], {self.sidx})"
            idx = self._register_noise("normal", nk + self.noise_key_suffix, params)
            self.coord_chains.append({
                "type": "noise", "noise_key": nk,
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
            })
            self.normal_chain_index[nk + self.noise_key_suffix] = len(self.coord_chains) - 1
            return f"normal_noise({idx}, {self.sidx})"
        if t == "minecraft:shifted_noise":
            np = self._resolve_noise_params(df.get("noise", ""))
            nk = df.get("noise", "")
            params = {
                "noise": nk, "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "firstOctave": np["firstOctave"], "amplitudes": np["amplitudes"],
            }
            if self.slot_mode:
                chain = {
                    "type": "shifted_noise", "noise_key": nk,
                    "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                    "flat_cache": self.flat_cache_depth > 0,
                    "shift_x": self._resolve_shift(df.get("shift_x", 0.0)),
                    "shift_y": self._resolve_shift(df.get("shift_y", 0.0)),
                    "shift_z": self._resolve_shift(df.get("shift_z", 0.0)),
                }
                slot = self._noise_slot("normal", nk, params, chain, self.in_interp_corner)
                return f"normal_noise(NOISE_SLOT_BASE[{slot}] + corner * NOISE_SLOT_STRIDE[{slot}], {self.sidx})"
            idx = self._register_noise("normal", nk + self.noise_key_suffix, params)
            self.coord_chains.append({
                "type": "shifted_noise", "noise_key": nk,
                "xz_scale": df.get("xz_scale", 1.0), "y_scale": df.get("y_scale", 1.0),
                "flat_cache": self.flat_cache_depth > 0,
                "shift_x": self._resolve_shift(df.get("shift_x", 0.0)),
                "shift_y": self._resolve_shift(df.get("shift_y", 0.0)),
                "shift_z": self._resolve_shift(df.get("shift_z", 0.0)),
            })
            self.normal_chain_index[nk + self.noise_key_suffix] = len(self.coord_chains) - 1
            return f"normal_noise({idx}, {self.sidx})"
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
            # D17: 表达式路径（D1 遗留，gen_df 为主路径；此处保持一致性）
            inp = self.gen(df.get("input", 0.0))
            nk_ws = df.get("noise", "")
            np_ws = self._resolve_noise_params(nk_ws)
            chain_ws = {
                "type": "noise", "noise_key": nk_ws,
                "xz_scale": 1.0, "y_scale": 1.0,
                "flat_cache": self.flat_cache_depth > 0, "ws": True,
            }
            slot_ws = self._noise_slot("normal", nk_ws + ":ws", {
                "noise": nk_ws, "xz_scale": 1.0, "y_scale": 1.0,
                "firstOctave": np_ws["firstOctave"], "amplitudes": np_ws["amplitudes"], "ws": True,
            }, chain_ws, self.in_interp_corner)
            kind = 1 if df.get("rarity_value_mapper") == "type_2" else 0
            return (f"(ws_scale({kind}, {inp}) * abs(normal_noise("
                    f"NOISE_SLOT_BASE[{slot_ws}] + corner * NOISE_SLOT_STRIDE[{slot_ws}], {self.sidx})))")
        if t == "minecraft:flat_cache":
            # flat_cache：坐标对齐到 biome（x>>2<<2, 0, z>>2<<2），delegate 采样（对齐 vanilla FlatCache.sample）
            # 注：flat_cache 内噪声仍角点独立（8 角点不同 4×4 列，不能共享）
            self.flat_cache_depth += 1
            inner = self.gen_with_coords(df["argument"], "((ix >> 2) << 2)", "0", "((iz >> 2) << 2)",
                                         "float((ix >> 2) << 2)", "0.0f", "float((iz >> 2) << 2)")
            self.flat_cache_depth -= 1
            return f"({inner})"
        if t in ("minecraft:cache_2d", "minecraft:cache_once", "minecraft:cache_all_in_cell"):
            # 缓存包装：采样结果 = delegate（原始坐标），剥掉（对齐 vanilla Cache2D/CacheOnce）
            return self.gen(df.get("argument", df.get("input", 0.0)))
        if t == "minecraft:interpolated":
            # cell 三线性插值（4×4×8）：8 角点 delegate 采样 + 三线性插值
            # 角点坐标 = chunkX*16 + (cx+dx)*4, minY + (cy+dy)*8, chunkZ*16 + (cz+dz)*4
            arg = df.get("argument", df.get("input", 0.0))
            interp_idx = len(self.interp_instances)
            self.interp_instances.append(arg)
            samples = []
            self.interp_depth += 1
            for c in range(8):
                dx = c & 1; dy = (c >> 1) & 1; dz = (c >> 2) & 1
                ax = f"(chunkX * 16 + (cx + {dx}) * 4)"
                ay = f"(minY + (cy + {dy}) * 8)"
                az = f"(chunkZ * 16 + (cz + {dz}) * 4)"
                old_suffix = self.noise_key_suffix
                self.noise_key_suffix = f"@c{c}"     # 8 个独立角点实例（去重 key 含角点）
                samples.append(self.gen_with_coords(arg, ax, ay, az, f"float({ax})", f"float({ay})", f"float({az})"))
                self.noise_key_suffix = old_suffix
            self.interp_depth -= 1
            self.interp_funcs.append((interp_idx, samples))
            return f"interp_{interp_idx}({self.sidx}, {self.cx}, {self.cy}, {self.cz})"
        if t == "minecraft:blend_alpha":
            return "1.0f"
        if t == "minecraft:blend_offset":
            return "0.0f"
        if t == "minecraft:blend_density":
            return self.gen(df.get("argument", 0.0))
        raise ValueError(f"unsupported type: {t}")

    # ---- spline 生成（B1a 数据驱动：SSBO 收集 + 单函数 spline_eval，56 函数 → 1）----
    def _spline_coord_type(self, coord):
        """coordinate 表达式去重 → coordType（0..N-1）。返回 (coordType, 是否首次)。"""
        ck = json.dumps(coord, sort_keys=True)   # 方案1：coordinate 表达式跨角点共享（slot 化 + corner 运行时查表）
        if ck in self.spline_coord_map:
            return self.spline_coord_map[ck], False
        ct = len(self.spline_coords)
        if self.node_mode:
            # D1：节点函数化——coordinate 用 gen_node（形参 ix/iy/iz；调用点 df_N 的坐标即实际坐标）
            expr = self.gen_node(coord)
        else:
            old_slot_mode = self.slot_mode
            self.slot_mode = True               # 方案1：coordinate 噪声 slot 化（corner 查表）
            expr = self.gen(coord)              # 用当前坐标上下文生成表达式（flat_cache 对齐后）
            self.slot_mode = old_slot_mode
        self.spline_coords.append(expr)
        self.spline_coord_map[ck] = ct
        return ct, True

    def _gen_spline(self, spline):
        """收集 spline 到 SSBO 数据，返回 spline_eval(nodeIdx, sIdx, ix, iy, iz) 调用。
        value 仅 2 种（实测）：const / nested spline。嵌套 spline 的 coordinate
        在子节点内独立计算（子节点自带 coordType）。"""
        key = json.dumps(spline, sort_keys=True)   # 方案1：spline 结构跨角点共享（coordinate 已 slot 化，corner 运行时区分）
        if key in self.spline_cache:
            return self.spline_cache[key]
        points = spline["points"]
        n = len(points)
        locs = [float(p["location"]) for p in points]
        ders = [float(p["derivative"]) for p in points]
        coord_type, is_first = self._spline_coord_type(spline["coordinate"])
        loc_begin = len(self.spline_ssbo_locs)
        der_begin = len(self.spline_ssbo_ders)
        self.spline_ssbo_locs.extend(locs)
        self.spline_ssbo_ders.extend(ders)
        # D17 修复：父 spline 的 loc/der 在循环前先 append（loc_begin/der_begin 正确），
        # 但 val 条目与子 spline 的 val 在循环内交错 append → val_begin 必须循环后统一 append 再捕获。
        # 同时 node_idx 也必须在子样条递归收集之后捕获（否则父样条记录陈旧索引 = 第一个新子样条的槽位）。
        my_kind, my_f, my_node = [], [], []
        for p in points:
            v = p["value"]
            if isinstance(v, dict) and "points" in v and "coordinate" in v and "type" not in v:
                # 嵌套 spline → SSBO 引用（递归收集子节点）
                sub_call = self._gen_spline(v)
                import re as _re
                m = _re.match(r"spline_eval\((\d+),", sub_call)
                assert m, f"unexpected spline call: {sub_call}"
                sub_node = int(m.group(1))
                my_kind.append(1)
                my_f.append(0.0)
                my_node.append(sub_node)
            else:
                my_kind.append(0)
                my_f.append(float(v) if not isinstance(v, dict) else self._const_value(v))
                my_node.append(-1)
        val_begin = len(self.spline_ssbo_val_kind)
        self.spline_ssbo_val_kind.extend(my_kind)
        self.spline_ssbo_val_f.extend(my_f)
        self.spline_ssbo_val_node.extend(my_node)
        node_idx = len(self.spline_ssbo_nodes)
        self.spline_ssbo_nodes.append({
            "coordType": coord_type, "n": n,
            "locBegin": loc_begin, "derBegin": der_begin, "valBegin": val_begin,
        })
        if self.node_mode:
            # D1：节点函数化——spline_eval 传形参（调用点 df_N 的 ix/iy/iz = 实际坐标，角点坐标由调用链传）
            call = f"spline_eval({node_idx}, {self.sidx}, ix, iy, iz)"
        else:
            call = f"spline_eval({node_idx}, {self.sidx}, {self.cx}, {self.cy}, {self.cz})"
        self.spline_cache[key] = call
        return call

    def _const_value(self, v):
        """value 为非嵌套 dict（理论无，防御）→ 仅支持 constant。"""
        t = v.get("type", "")
        if t == "minecraft:constant":
            return float(v.get("value", 0.0))
        raise ValueError(f"B1a: spline value 出现非 const/nested 类型 {t} —— 需扩展 SSBO 布局")

    def _spline_ssbo_glsl(self):
        """生成 spline 数据驱动代码：SSBO 数据表 + spline_coord + spline_eval（单函数，显式栈后序求值）。
        A 方案（2026-08-15，D21 根因修复）：6 张数据表从 const 数组 → SSBO（binding 6-11），
        动态 node 索引（SPLINE_NODE_PACK[node*5]）变运行时 buffer 读 → 驱动不再为 56 个可能
        node 各自展开数据流（原 903.4s 主因）。求值逻辑 = b1a 设计的 while 栈显式栈后序求值。"""
        nodes = self.spline_ssbo_nodes
        if not nodes:
            return ""
        node_pack = []
        for nd in nodes:
            node_pack += [nd["coordType"], nd["n"], nd["locBegin"], nd["derBegin"], nd["valBegin"]]

        coord_cases = []
        for ct, expr in enumerate(self.spline_coords):
            coord_cases.append(f"    case {ct}: return ({expr});")
        if 'coord_const' in self.diag:
            # D21 二分：spline_coord switch 固定返回（量化 coord 表达式分派成本）
            coord_cases = [f"    case {ct}: return 0.0f;" for ct in range(len(self.spline_coords))]
        elif 'coord_slot0' in self.diag:
            # 二分：4 case 全用 slot0 同实例（量化「不同实例展开」成本 vs「调用本身」）
            base0 = "NOISE_SLOT_BASE[0] + corner * NOISE_SLOT_STRIDE[0]"
            coord_cases = [f"    case {ct}: return normal_noise({base0}, sIdx);" for ct in range(len(self.spline_coords))]
        elif 'coord_case0' in self.diag:
            # 二分：仅 case 0 调 normal_noise，其余 0（量化单次调用 vs case 数）
            base0 = "NOISE_SLOT_BASE[0] + corner * NOISE_SLOT_STRIDE[0]"
            coord_cases = [f"    case {ct}: return normal_noise({base0}, sIdx);" if ct == 0 else f"    case {ct}: return 0.0f;" for ct in range(len(self.spline_coords))]
        coord_switch = "\n".join(coord_cases) if coord_cases else "    return 0.0f;"

        # ---- A5（D21 二分后续）：spline_coord 根因修复——switch 内 case 体常量传播展开。
        # 二分证据：coord_case0（仅 1 case 调 normal_noise）=74.8s vs coord_const=37.2s → 1 次调用 +37s；
        # 而 eval_df 里同样调 normal_noise（CA1_T[ci] 动态索引）不慢（no_spline=17.2s）。
        # 差异：switch(coordType) 让每个 case 内 slot 下标（NOISE_SLOT_BASE[0]）成为编译期常量
        #   → 驱动常量传播进 normal_noise → NORMAL_PACK 读取静态化 → 循环展开。
        # 修复：spline_coord 改「coordType 运行时查表」（COORD_SLOT_TABLE），去掉 switch 的
        #   case 常量化；fold 包装（如 ridges_folded 的 abs 链）提取为 if(coordType==k) 特例。
        import re as _re
        _coord_slots = []
        _coord_folds = []          # [(ct, fold_expr_with_v)]，v = normal_noise 结果
        _coord_ok = True
        for ct, expr in enumerate(self.spline_coords):
            m = _re.search(r'normal_noise\(NOISE_SLOT_BASE\[(\d+)\] \+ corner \* NOISE_SLOT_STRIDE\[\d+\], sIdx\)', expr)
            if not m:
                _coord_ok = False   # 非标准形态（无纯 normal_noise 调用）→ fallback switch
                break
            _coord_slots.append(int(m.group(1)))
            _coord_folds.append(expr.replace(m.group(0), 'v'))
        if _coord_ok and len(self.spline_coords) > 0:
            coord_slot_src = ", ".join(str(s) for s in _coord_slots)
            coord_fold_lines = "\n".join(
                f"    if (coordType == {ct}) v = {fold};" for ct, fold in enumerate(_coord_folds))
            coord_glsl = f"""const int COORD_SLOT_TABLE[{len(_coord_slots)}] = int[]({coord_slot_src});
float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz) {{
    int slot = COORD_SLOT_TABLE[coordType];
    float v = normal_noise(NOISE_SLOT_BASE[slot] + corner * NOISE_SLOT_STRIDE[slot], sIdx);
{coord_fold_lines}
    return v;
}}"""
        else:
            coord_glsl = f"""float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz) {{
    switch (coordType) {{
{coord_switch}
    }}
}}"""

        # A1b：布局数据导出（D19 铁律：宿主上传用，禁止硬编码）——gen_cpu 输出到 CpuBackend
        # P2-2：splineBindBase 一并导出（宿主 e2e wb 数组不再硬编码 binding 号）
        self.spline_layout = {
            "nNodes": len(nodes),
            "nodePack": node_pack,
            "locs": self.spline_ssbo_locs,
            "ders": self.spline_ssbo_ders,
            "valF": self.spline_ssbo_val_f,
            "valKind": self.spline_ssbo_val_kind,
            "valNode": self.spline_ssbo_val_node,
            "bindBase": self.spline_bind_base,   # spline 6 表 binding 起始号（6-11）
        }

        return f"""
// ===== spline 数据驱动（A 方案：SSBO 表 + 显式栈 spline_eval，56 函数 → 1）=====
// 数据表 SSBO（binding 6-11）：动态 node 索引变运行时读，驱动不展开（D21 根因修复）
layout(set = 0, binding = 6, std430) buffer SplineNodePackBuf {{ int splineNodePack[]; }};
layout(set = 0, binding = 7, std430) buffer SplineLocsBuf {{ float splineLocs[]; }};
layout(set = 0, binding = 8, std430) buffer SplineDersBuf {{ float splineDers[]; }};
layout(set = 0, binding = 9, std430) buffer SplineValFBuf {{ float splineValF[]; }};
layout(set = 0, binding = 10, std430) buffer SplineValKindBuf {{ int splineValKind[]; }};
layout(set = 0, binding = 11, std430) buffer SplineValNodeBuf {{ int splineValNode[]; }};
const int SPLINE_NODES = {len(nodes)};

{coord_glsl}

// vanilla MathHelper.binarySearch 精确复刻
int spline_find_range(float x, int locBegin, int n) {{
    int min = 0;
    int i = n;
    while (i > 0) {{
        int j = i / 2;
        int k = min + j;
        if (x < splineLocs[locBegin + k]) {{ i = j; }}
        else {{ min = k + 1; i -= j + 1; }}
    }}
    return min - 1;
}}

float spline_hermite(float coord, float lo, float span, float nv, float ov, float d0, float d1) {{
    float kd = (coord - lo) / span;
    float p = d0 * span - (ov - nv);
    float q = -d1 * span + (ov - nv);
    return (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p));
}}

// 显式栈后序求值（GLSL 禁递归 D4）：帧 = {{node, i, coord, stage, v0, v1}}
// stage: 0=init(coord+二分+边界) / 1=等 v0 子帧回填 / 3=等 v1 子帧回填（stage 2 为瞬态不挂起）
float spline_eval(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {{
    int st_node[32]; int st_i[32]; int st_stage[32];
    float st_coord[32]; float st_v0[32]; float st_v1[32];
    int sp = 0;
    st_node[0] = rootNode; st_stage[0] = 0; sp = 1;
    float outVal = 0.0;
    while (sp > 0) {{
        int f = sp - 1;
        int node = {'0' if 'fixed_node' in self.diag else 'st_node[f]'};
        int p = node * 5;
        int ct = splineNodePack[p + 0];
        int n = splineNodePack[p + 1];
        int locB = splineNodePack[p + 2];
        int derB = splineNodePack[p + 3];
        int valB = splineNodePack[p + 4];
        if (st_stage[f] == 0) {{
            float coord = spline_coord(ct, corner, sIdx, ix, iy, iz);
            int i = spline_find_range(coord, locB, n);
            st_coord[f] = coord; st_i[f] = i;
            if (i < 0) {{
                // D23 修复：边界外推遇嵌套 value 必须递归求值（vanilla Spline.apply L259：
                // sampleOutsideRange(f, ..., values.get(0).apply(x), ...)——不是 0.0）
                if (splineValKind[valB] == 0) {{
                    outVal = splineValF[valB] + splineDers[derB] * (coord - splineLocs[locB]);
                    sp--;
                }} else {{
                    st_stage[f] = 4;   // 等边界 v0（嵌套 spline）子帧回填
                    st_node[sp] = splineValNode[valB]; st_stage[sp] = 0; sp++;
                }}
            }} else if (i >= n - 1) {{
                if (splineValKind[valB + n - 1] == 0) {{
                    outVal = splineValF[valB + n - 1] + splineDers[derB + n - 1] * (coord - splineLocs[locB + n - 1]);
                    sp--;
                }} else {{
                    st_stage[f] = 5;   // 等边界 vn（嵌套 spline）子帧回填
                    st_node[sp] = splineValNode[valB + n - 1]; st_stage[sp] = 0; sp++;
                }}
            }} else {{
                st_stage[f] = 1;
                if (splineValKind[valB + i] == 0) {{
                    st_v0[f] = splineValF[valB + i];
                    st_stage[f] = 2;
                    if (splineValKind[valB + i + 1] == 0) {{
                        st_v1[f] = splineValF[valB + i + 1];
                        float lo = splineLocs[locB + i];
                        outVal = spline_hermite(coord, lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
                        sp--;
                    }} else {{
                        st_stage[f] = 3;
                        st_node[sp] = splineValNode[valB + i + 1]; st_stage[sp] = 0; sp++;
                    }}
                }} else {{
                    st_node[sp] = splineValNode[valB + i]; st_stage[sp] = 0; sp++;
                }}
            }}
        }} else if (st_stage[f] == 4) {{
            // D23：边界 v0 子帧回填 → 外推
            float coord = st_coord[f];
            outVal = outVal + splineDers[derB] * (coord - splineLocs[locB]);
            sp--;
        }} else if (st_stage[f] == 5) {{
            // D23：边界 vn 子帧回填 → 外推
            float coord = st_coord[f];
            outVal = outVal + splineDers[derB + n - 1] * (coord - splineLocs[locB + n - 1]);
            sp--;
        }} else if (st_stage[f] == 1) {{
            // 等 v0 子帧回填
            st_v0[f] = outVal;
            st_stage[f] = 2;
            int i = st_i[f];
            if (splineValKind[valB + i + 1] == 0) {{
                st_v1[f] = splineValF[valB + i + 1];
                float coord = st_coord[f];
                float lo = splineLocs[locB + i];
                outVal = spline_hermite(coord, lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
                sp--;
            }} else {{
                st_stage[f] = 3;
                st_node[sp] = splineValNode[valB + i + 1]; st_stage[sp] = 0; sp++;
            }}
        }} else if (st_stage[f] == 3) {{
            // 等 v1 子帧回填 → Hermite 完成
            st_v1[f] = outVal;
            int i = st_i[f];
            float coord = st_coord[f];
            float lo = splineLocs[locB + i];
            outVal = spline_hermite(coord, lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
            sp--;
        }}
    }}
    return outVal;
}}
"""

    # ---- 生成完整 shader 源码 ----
    def gen_shader(self, root_df):
        # D2：重置收集（幂等），DF 树 → 节点数组 → eval_df 解释器
        self._reset_collect()
        root_node = self.gen_df(root_df)
        expr = "eval_df(__TOP_ROOT__, 0, sIdx, ix, iy, iz)"
        funcs = []
        # interp 函数前向声明（registry/spline 可能调用 interp，GLSL 需先声明）
        for interp_idx, _ in self.interp_funcs:
            funcs.append(f"float interp_{interp_idx}(int sIdx, int ix, int iy, int iz);")
        # D2：eval_df 前向声明（噪声/spline 函数可能调用 eval_df）
        funcs.append("float eval_df(int rootNode, int corner, int sIdx, int ix, int iy, int iz);")
        # 噪声函数（old_blended double + normal float）先定义（registry 函数会调用）
        # 分配 octBase（perm/origin buffer 的 octave 偏移）+ splitBase（拆分坐标 buffer 的偏移，单位 6 值/octave）
        octBase = 0
        splitBase = 0
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                funcs.append(self._old_blended_func(idx, params, octBase, splitBase))
                self.old_split_base[params["_key"]] = splitBase
                self.old_vec_index[params["_key"]] = len(self.old_vec_index)
                octBase += 40
                splitBase += 7 * 40   # 5 参数 sample：7 值/octave [ix,iy,iz,gx,gy,gz,fadeY] × 40 octave
            elif kind == "normal":
                n = len(params.get("amplitudes", [1.0]))
                funcs.append(self._normal_func(idx, params, octBase, splitBase))   # C2：纯数据收集（返回空串）
                self.normal_split_base[params["_key"]] = splitBase
                self.normal_vec_index[params["_key"]] = len(self.normal_vec_index)
                octBase += 2 * n
                splitBase += 6 * 2 * n   # 6 值 [ix,iy,iz,gx,gy,gz] × 2n octave
        # normal_noise 数据驱动单函数（C2：139 函数 → 1）
        nn_glsl = self._normal_noise_glsl() if 'no_normal' not in self.diag else ""
        if nn_glsl:
            funcs.append(nn_glsl)
        # old_blended 数据驱动单函数（D2：8 函数 → 1）
        ob_glsl = self._old_blended_glsl() if 'no_old' not in self.diag else ""
        if ob_glsl:
            funcs.append(ob_glsl)
        self.split_total = splitBase      # 每采样点的拆分坐标总数
        # registry 函数定义（依赖序已保证），传 int 块坐标，内部转 float
        for fname, fexpr in self.registry_defs:
            funcs.append(f"float {fname}(int sIdx, int ix, int iy, int iz) {{\n    float x = float(ix), y = float(iy), z = float(iz);\n    return {fexpr};\n}}\n")
        # 方案1：噪声 slot 表（spline_coord/eval_df 之前定义）
        slot_tbl = self._noise_slot_table_glsl()
        if slot_tbl:
            funcs.append(slot_tbl)
        # spline 数据驱动（B1a：SSBO + 单函数 spline_eval，替代 56 个 spline_N 函数）
        ssbo_glsl = self._spline_ssbo_glsl() if 'no_spline' not in self.diag else ""
        if ssbo_glsl:
            funcs.append(ssbo_glsl)
        # D17：weird_scaled_sampler 的 scaleValue（type_2=CAVES, type_1=TUNNELS）
        funcs.append("""float ws_scale(int kind, float v) {
    if (kind == 1) {
        if (v < -0.75) return 0.5;
        if (v < -0.5) return 0.75;
        if (v < 0.5) return 1.0;
        return v < 0.75 ? 2.0 : 3.0;
    }
    if (v < -0.5) return 0.75;
    if (v < 0.0) return 1.0;
    return v < 0.5 ? 1.5 : 2.0;
}
""")
        # D2：eval_df 解释器 + 节点数据（替代 D1 的 300 个 df_N 函数）
        eval_glsl = self.eval_df_glsl()
        expr = expr.replace("__TOP_ROOT__", str(self.top_root_pos))
        if eval_glsl:
            funcs.append(eval_glsl)
        # interpolated 函数（cell 三线性插值：8 角点 delegate 采样 + 插值）
        for interp_idx, samples in self.interp_funcs:
            lines = [f"float interp_{interp_idx}(int sIdx, int ix, int iy, int iz) {{"]
            lines.append("    int chunkX = floorDivP(ix, 16); int chunkZ = floorDivP(iz, 16);")
            lines.append("    int gx = ix - chunkX * 16; int gy = iy - minY; int gz = iz - chunkZ * 16;")
            lines.append("    int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;")
            lines.append("    float fx = float(gx % 4) / 4.0f; float fy = float(gy % 8) / 8.0f; float fz = float(gz % 4) / 4.0f;")
            root_pos = self.interp_root_pos[interp_idx] if interp_idx < len(self.interp_root_pos) else 0
            for c in range(8):
                dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
                lines.append(f"    float d{dx}{dy}{dz} = {samples[c].replace('__ROOT__', str(root_pos))};")
            lines.append("    float d00 = d000 + (d100 - d000) * fx; float d10 = d010 + (d110 - d010) * fx;")
            lines.append("    float d01 = d001 + (d101 - d001) * fx; float d11 = d011 + (d111 - d011) * fx;")
            lines.append("    float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;")
            lines.append("    return d0 + (d1 - d0) * fz;")
            lines.append("}")
            funcs.append("\n".join(lines))
        return self._shader_template(expr, funcs)

    def _noise_slot_table_glsl(self):
        """方案1：噪声 slot 表（NOISE_SLOT_BASE/STRIDE）——必须定义在 spline_coord/eval_df 之前。"""
        if not self.noise_slots:
            return ""
        slot_bases = ", ".join(str(s["base"]) for s in self.noise_slots)
        slot_strides = ", ".join(str(s["stride"]) for s in self.noise_slots)
        n_slots = len(self.noise_slots)
        return f"""// 方案1：噪声 slot 表（结构共享 + 角点运行时实例查表）
const int NOISE_SLOT_COUNT = {n_slots};
const int NOISE_SLOT_BASE[{n_slots}] = int[]({slot_bases});
const int NOISE_SLOT_STRIDE[{n_slots}] = int[]({slot_strides});
"""

    def gen_noise_manifest(self):
        """输出噪声清单（JSON dict）：normal 实例的坐标链 + octBase/splitBase + shift 噪声参数，供 CPU 侧重放"""
        normal_instances = []
        octBase = 0
        splitBase = 0
        ci = 0   # coord_chains 索引（只对 normal 实例）
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                octBase += 40
                splitBase += 7 * 40
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

    # ---- CPU 后端（坐标链重放 + 拆分代码生成）----
    def _shift_cpp(self, s, ax, ay, az):
        """shift 描述 → C++ 表达式（坐标偏移，double）"""
        if s["type"] == "constant":
            return f"{s['value']:.17g}"
        key = s["noise_key"]
        if s["type"] == "shift_a":
            return f'shiftNoises.at("{key}").sample(({ax}) * 0.25, 0.0, ({az}) * 0.25) * 4.0'
        if s["type"] == "shift_b":
            return f'shiftNoises.at("{key}").sample(({az}) * 0.25, ({ax}) * 0.25, 0.0) * 4.0'
        return f'shiftNoises.at("{key}").sample(({ax}) * 0.25, ({ay}) * 0.25, ({az}) * 0.25) * 4.0'   # shift

    def _gen_split_lines(self, df, cx, cy, cz):
        """递归生成拆分代码行（在 cx/cy/cz int 坐标上下文重放 noise 坐标链）"""
        lines = []
        if isinstance(df, str):
            if df in ("minecraft:y", "minecraft:zero", "minecraft:shift_x", "minecraft:shift_z"):
                return lines
            return self._gen_split_lines(self.resolve_ref(df), cx, cy, cz)
        if isinstance(df, (int, float)):
            return lines
        t = df.get("type", "")
        if t == "minecraft:spline":
            # type=spline 节点：spline 字段才是 {coordinate, points}
            df = df.get("spline", df)
        if "coordinate" in df and "points" in df:
            # spline（含 nested {coordinate,points}）：遍历 coordinate（含坐标噪声）
            # 修复 D14：spline coordinate 噪声（continentalness/erosion/ridge）必须生成 split 行
            lines += self._gen_split_lines(df["coordinate"], cx, cy, cz)
            for p in df.get("points", []):
                v = p.get("value")
                if isinstance(v, dict) and "coordinate" in v and "points" in v:
                    lines += self._gen_split_lines(v, cx, cy, cz)   # nested spline 递归
            return lines
        if t in ("minecraft:noise", "minecraft:shifted_noise"):
            key = df.get("noise", "") + self.noise_key_suffix
            if os.environ.get('DFC_SPLIT_TRACE'):
                print(f'[SPLIT-TRACE] noise {df.get("noise","")} suffix={self.noise_key_suffix} visited={key in self.split_visited}')
            if key in self.split_visited:
                return lines    # 防 spline coordinate 重复引用（同实例同角点只 split 一次）
            self.split_visited.add(key)
            chain = self.coord_chains[self.normal_chain_index[key]]
            vi = self.normal_vec_index[key]
            sb = self.normal_split_base[key]
            n = len(self._resolve_noise_params(df.get("noise", ""))["amplitudes"])
            if chain.get("flat_cache"):
                ax = f"(({cx}) >> 2) << 2"; ay = "0"; az = f"(({cz}) >> 2) << 2"
            else:
                ax = cx; ay = cy; az = cz
            xs, ys = f"{chain['xz_scale']:.17g}", f"{chain['y_scale']:.17g}"
            if chain["type"] == "noise":
                dx, dy, dz = f"({ax}) * {xs}", f"({ay}) * {ys}", f"({az}) * {xs}"
            else:
                sx = self._shift_cpp(chain["shift_x"], ax, ay, az)
                sy = self._shift_cpp(chain["shift_y"], ax, ay, az)
                sz = self._shift_cpp(chain["shift_z"], ax, ay, az)
                dx = f"({ax}) * {xs} + ({sx})"
                dy = f"({ay}) * {ys} + ({sy})"
                dz = f"({az}) * {xs} + ({sz})"
            lines.append(f'    {{ splitDouble(normals[{vi}], {dx}, {dy}, {dz}, out, {sb}, {n}); }}')
        elif t == "minecraft:old_blended_noise":
            key = (f"old_blended:{df.get('xz_scale',0.25)}:{df.get('y_scale',0.125)}:"
                   f"{df.get('xz_factor',80.0)}:{df.get('y_factor',160.0)}:"
                   f"{df.get('smear_scale_multiplier',8.0)}{self.noise_key_suffix}")
            if key in self.split_visited:
                return lines
            self.split_visited.add(key)
            vi = self.old_vec_index[key]
            sb = self.old_split_base[key]
            lines.append(f'    {{ splitOldBlended(*oldBlendeds[{vi}], {cx}, {cy}, {cz}, out, {sb}); }}')
        elif t == "minecraft:interpolated":
            lines.append("    {")
            lines.append(f"        int _chunkX = floorDiv({cx}, 16); int _chunkZ = floorDiv({cz}, 16);")
            lines.append(f"        int _gx = ({cx}) - _chunkX * 16; int _gy = ({cy}) - minY; int _gz = ({cz}) - _chunkZ * 16;")
            lines.append("        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;")
            for c in range(8):
                dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
                ax = f"(_chunkX * 16 + (_cx + {dx}) * 4)"
                ay = f"(minY + (_cy + {dy}) * 8)"
                az = f"(_chunkZ * 16 + (_cz + {dz}) * 4)"
                old_suffix = self.noise_key_suffix
                self.noise_key_suffix = f"@c{c}"
                lines += self._gen_split_lines(df.get("argument", df.get("input", 0.0)), ax, ay, az)
                self.noise_key_suffix = old_suffix
            lines.append("    }")
        elif t == "minecraft:weird_scaled_sampler":
            # D17: rarity 输入 split（正常坐标）+ ws 噪声 split（/d 坐标，d = ws_scale(rarity, 输入值)）
            lines += self._gen_split_lines(df.get("input", 0.0), cx, cy, cz)
            ws_key = df.get("noise", "") + ":ws" + self.noise_key_suffix
            if ws_key not in self.split_visited:
                self.split_visited.add(ws_key)
                vi = self.normal_vec_index[ws_key]
                sb = self.normal_split_base[ws_key]
                n = len(self._resolve_noise_params(df.get("noise", ""))["amplitudes"])
                rc = self._ws_rarity_cpp(df.get("input", 0.0), cx, cy, cz)
                kind = 1 if df.get("rarity_value_mapper") == "type_2" else 0
                lines.append(f'    {{ double _d = ws_scale({kind}, {rc}); splitDouble(normals[{vi}], ({cx})/_d, ({cy})/_d, ({cz})/_d, out, {sb}, {n}); }}')
        else:
            for key in ("argument", "argument1", "argument2", "input", "when_in_range", "when_out_of_range"):
                if key in df:
                    lines += self._gen_split_lines(df[key], cx, cy, cz)
        return lines

    def _ws_rarity_cpp(self, df, cx, cy, cz):
        """ws rarity 输入在 CPU split() 侧的求值表达式（当前支持 plain noise / const）。
        cache 包装剥掉；噪声用 normals[vi].sample（double，与 split 数据同源，
        与 shader 侧 slot 采样值一致到 float 精度，piecewise scaleValue 使 d 逐位一致除非恰在阈值）。"""
        while isinstance(df, dict) and df.get("type") in ("minecraft:cache_once", "minecraft:cache_2d", "minecraft:cache_all_in_cell"):
            df = df.get("argument", df.get("input", 0.0))
        if isinstance(df, (int, float)):
            return f"{float(df):.17g}"
        t = df.get("type", "")
        if t == "minecraft:noise":
            key = df.get("noise", "") + self.noise_key_suffix
            chain = self.coord_chains[self.normal_chain_index[key]]
            vi = self.normal_vec_index[key]
            xs, ys = float(chain["xz_scale"]), float(chain["y_scale"])
            return f"normals[{vi}].sample(({cx}) * {xs:.17g}, ({cy}) * {ys:.17g}, ({cz}) * {xs:.17g})"
        raise ValueError(f"D17: ws rarity 输入暂只支持 plain noise/const，遇到 {t}: {json.dumps(df)[:150]}")

    def gen_cpu(self, root_df):
        """生成 CPU 后端 C++ 头文件（噪声生成 + 坐标链重放 + 拆分 + perm 收集）"""
        manifest = self.gen_noise_manifest()
        normals = manifest["normal_instances"]
        shift_noises = manifest["shift_noises"]

        # old_blended 实例（收集 + 分配 octBase/splitBase，与 gen_shader 一致）
        old_blendeds = []
        octBase = 0
        splitBase = 0
        for kind, params in self.noise_instances:
            if kind == "old_blended":
                old_blendeds.append({"params": params, "octBase": octBase, "splitBase": splitBase})
                self.old_split_base[params["_key"]] = splitBase
                self.old_vec_index[params["_key"]] = len(old_blendeds) - 1
                octBase += 40
                splitBase += 7 * 40
            elif kind == "normal":
                n = len(params.get("amplitudes", [1.0]))
                self.normal_split_base[params["_key"]] = splitBase
                self.normal_vec_index[params["_key"]] = len(self.normal_vec_index)
                octBase += 2 * n
                splitBase += 6 * 2 * n

        init_lines = []
        for key, np in shift_noises.items():
            amps = ", ".join(f"{a:.17g}" for a in np["amplitudes"])
            init_lines.append(
                f'    {{ auto r = rd.split("{key}"); shiftNoises.emplace("{key}", '
                f'wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{{{np["firstOctave"]}, {{{amps}}}}})); }}')
        for i, ni in enumerate(normals):
            amps = ", ".join(f"{a:.17g}" for a in ni["amplitudes"])
            init_lines.append(
                f'    {{ auto r = rd.split("{ni["noise_key"]}"); normals.emplace_back('
                f'wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{{{ni["firstOctave"]}, {{{amps}}}}})); '
                f'n.push_back({ni["n"]}); octBase.push_back({ni["octBase"]}); splitBase.push_back({ni["splitBase"]}); }}')
        for i, ob in enumerate(old_blendeds):
            p = ob["params"]
            init_lines.append(
                f'    {{ wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back('
                f'std::make_shared<wg::InterpolatedNoiseDF>(r, {p["xz_scale"]:.17g}, {p["y_scale"]:.17g}, {p["xz_factor"]:.17g}, {p["y_factor"]:.17g}, {p["smear"]:.17g})); '
                f'oldBase.push_back({ob["octBase"]}); oldSplitBase.push_back({ob["splitBase"]}); }}')

        self.split_visited.clear()
        split_lines = self._gen_split_lines(root_df, "x", "y", "z")

        # D19: perSample（valBuf 每采样点槽数）——e2e 分配 valBuf 的依据，防硬编码陈旧越界
        layout = self._compute_val_layout()
        per_sample = layout["per_sample"]

        # permSize = 总 octave 数（old_blended 40 + 所有 normal 2n）× 256
        total_octave = 0
        for kind, p in self.noise_instances:
            total_octave += 40 if kind == "old_blended" else 2 * len(p.get("amplitudes", [1.0]))
        perm_size = total_octave * 256

        # A1b：spline SSBO 数据导出（生成器产出，宿主上传——D19 铁律：禁止硬编码）
        def _flit(x):
            s = format(float(x), '.17g')
            if '.' not in s and 'e' not in s and 'E' not in s:
                s += '.0'
            return s + 'f'
        _node_pack = []
        for nd in self.spline_ssbo_nodes:
            _node_pack += [nd["coordType"], nd["n"], nd["locBegin"], nd["derBegin"], nd["valBegin"]]
        spline_members = f"""    // A1b：spline SSBO 数据（生成器导出，宿主上传——D19 铁律）
    int splineBindBase = {self.spline_bind_base};   // P2-2: spline 6 表 binding 起始号（6-11）
    int splineNodes = {len(self.spline_ssbo_nodes)};
    std::vector<int> splineNodePack = {{{{{", ".join(str(x) for x in _node_pack)}}}}};
    std::vector<float> splineLocs = {{{{{", ".join(_flit(x) for x in self.spline_ssbo_locs)}}}}};
    std::vector<float> splineDers = {{{{{", ".join(_flit(x) for x in self.spline_ssbo_ders)}}}}};
    std::vector<float> splineValF = {{{{{", ".join(_flit(x) for x in self.spline_ssbo_val_f)}}}}};
    std::vector<int> splineValKind = {{{{{", ".join(str(x) for x in self.spline_ssbo_val_kind)}}}}};
    std::vector<int> splineValNode = {{{{{", ".join(str(x) for x in self.spline_ssbo_val_node)}}}}};
"""

        return f"""// 自动生成（DFC CPU 后端），勿手改
#pragma once
#include <vector>
#include <map>
#include <string>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "density.h"

struct CpuBackend {{
    std::map<std::string, wg::DoublePerlinNoiseSampler> shiftNoises;
    std::vector<wg::DoublePerlinNoiseSampler> normals;
    std::vector<int> n, octBase, splitBase;
    std::vector<std::shared_ptr<wg::InterpolatedNoiseDF>> oldBlendeds;
    std::vector<int> oldBase, oldSplitBase;
    int splitTotal = {manifest["split_total"]};
    int permSize = {perm_size};
    int perSample = {per_sample};   // D19: valBuf 每采样点槽数（与 shader PER_SAMPLE 一致）
{spline_members}

    static int floorDiv(int a, int b) {{ int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }}
    static const int minY = -64;   // overworld 维度 minY（interpolated cell 网格）
    static double maintainPrecision(double v) {{ return v - (long)(v / 3.3554432E7 + 0.5) * 3.3554432E7; }}

    void init(uint64_t worldSeed) {{
        wg::XoroshiroRandom base(worldSeed);
        auto rd = base.nextSplitter();
{chr(10).join(init_lines)}
    }}

    static void splitOctave(const wg::PerlinNoiseSampler* pn, double cx, double cy, double cz, float* out) {{
        double ox = pn ? pn->originX : 0.0, oy = pn ? pn->originY : 0.0, oz = pn ? pn->originZ : 0.0;
        int ix = (int)std::floor(cx + ox), iy = (int)std::floor(cy + oy), iz = (int)std::floor(cz + oz);
        out[0] = (float)ix; out[1] = (float)iy; out[2] = (float)iz;
        out[3] = (float)(cx + ox - ix); out[4] = (float)(cy + oy - iy); out[5] = (float)(cz + oz - iz);
    }}

    static void splitDouble(const wg::DoublePerlinNoiseSampler& noise, double dx, double dy, double dz, float* out, int base, int nn) {{
        double lacunarity = std::pow(2.0, noise.firstSampler.firstOctave);
        double e = lacunarity;
        for (int i = 0; i < nn; i++) {{
            splitOctave(noise.firstSampler.octaveSamplers[i].get(),
                        maintainPrecision(dx*e), maintainPrecision(dy*e), maintainPrecision(dz*e),
                        &out[base + i * 6]);
            splitOctave(noise.secondSampler.octaveSamplers[i].get(),
                        maintainPrecision(dx*1.0181268882175227*e), maintainPrecision(dy*1.0181268882175227*e), maintainPrecision(dz*1.0181268882175227*e),
                        &out[base + 6 * nn + i * 6]);
            e *= 2.0;
        }}
    }}

    // 5 参数 sample 拆分：out = [ix,iy,iz,gx,gy(=h-n),gz,fadeY(=h)]
    static void split7(const wg::PerlinNoiseSampler* pn, double x, double y, double z, double yScale, double yMax, float* out) {{
        double sx = x + pn->originX, sy = y + pn->originY, sz = z + pn->originZ;
        int ix = wg::floorD(sx), iy = wg::floorD(sy), iz = wg::floorD(sz);
        double gx = sx - ix, gy_raw = sy - iy, gz = sz - iz;
        double n;
        if (yScale != 0.0) {{
            double m = (yMax >= 0.0 && yMax < gy_raw) ? yMax : gy_raw;
            n = wg::floorD(m / yScale + 1.0E-7F) * yScale;
        }} else n = 0.0;
        out[0] = (float)ix; out[1] = (float)iy; out[2] = (float)iz;
        out[3] = (float)gx; out[4] = (float)(gy_raw - n); out[5] = (float)gz; out[6] = (float)gy_raw;
    }}

    // D17: weird_scaled_sampler scaleValue（kind 1=CAVES, 0=TUNNELS）
    static double ws_scale(int kind, double v) {{
        if (kind == 1) {{
            if (v < -0.75) return 0.5;
            if (v < -0.5) return 0.75;
            if (v < 0.5) return 1.0;
            return v < 0.75 ? 2.0 : 3.0;
        }}
        if (v < -0.5) return 0.75;
        if (v < 0.0) return 1.0;
        return v < 0.5 ? 1.5 : 2.0;
    }}

    static void splitOldBlended(const wg::InterpolatedNoiseDF& ob, int x, int y, int z, float* out, int base) {{
        double d = x * ob.scaledXzScale;
        double e = y * ob.scaledYScale;
        double f = z * ob.scaledXzScale;
        double g = d / ob.xzFactor;
        double h = e / ob.yFactor;
        double i = f / ob.xzFactor;
        double j = ob.scaledYScale * ob.smearScaleMultiplier;
        double k = j / ob.yFactor;
        double o = 1.0;
        for (int q = 0; q < 8; q++) {{
            split7(ob.interpolation.getOctave(q), maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o, &out[base + (32+q)*7]);
            o /= 2.0;
        }}
        o = 1.0;
        for (int r = 0; r < 16; r++) {{
            double s2 = maintainPrecision(d*o), t2 = maintainPrecision(e*o), u2 = maintainPrecision(f*o);
            split7(ob.lower.getOctave(r), s2, t2, u2, j*o, e*o, &out[base + r*7]);
            split7(ob.upper.getOctave(r), s2, t2, u2, j*o, e*o, &out[base + (16+r)*7]);
            o /= 2.0;
        }}
    }}

    void split(int x, int y, int z, float* out) {{
{chr(10).join(split_lines)}
    }}

    void collectPerm(std::vector<uint32_t>& perm) {{
        perm.assign((size_t)permSize, 0);
        for (int i = 0; i < (int)oldBlendeds.size(); i++) {{
            for (int r = 0; r < 16; r++) {{
                const wg::PerlinNoiseSampler* pn = oldBlendeds[i]->lower.getOctave(r);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + r) * 256 + j] = (uint32_t)pn->permutation[j];
                pn = oldBlendeds[i]->upper.getOctave(r);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + 16 + r) * 256 + j] = (uint32_t)pn->permutation[j];
            }}
            for (int q = 0; q < 8; q++) {{
                const wg::PerlinNoiseSampler* pn = oldBlendeds[i]->interpolation.getOctave(q);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + 32 + q) * 256 + j] = (uint32_t)pn->permutation[j];
            }}
        }}
        for (int i = 0; i < (int)normals.size(); i++) {{
            for (int k = 0; k < n[i]; k++) {{
                const wg::PerlinNoiseSampler* pn = normals[i].firstSampler.octaveSamplers[k].get();
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(octBase[i] + k) * 256 + j] = (uint32_t)pn->permutation[j];
                pn = normals[i].secondSampler.octaveSamplers[k].get();
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(octBase[i] + n[i] + k) * 256 + j] = (uint32_t)pn->permutation[j];
            }}
        }}
    }}
}};
"""

    def _old_blended_func(self, idx, p, octBase, splitBase):
        # CPU 预拆分 5 参数 sample（7 值/octave），GPU 纯 float 采样 + float 累加（无 fp64）
        # D2 数据驱动化：不再每实例一个函数，统一 interp_noise(idx, sIdx) + 参数表
        self.old_meta.append({"idx": idx, "octBase": octBase, "splitBase": splitBase})
        return ""   # 不生成独立函数

    def _old_blended_glsl(self):
        """生成 interp_noise(idx, sIdx) 单函数 + 参数表（8 函数 → 1）。"""
        meta = self.old_meta
        if not meta:
            return ""
        # 修复（D16 续）：OLD_PACK 按 noise_instances 索引对齐（interp_noise 的 idx = slot base + corner
        #   = noise_instances 索引；old_blended 实例在 [32..39]，非 old 位置占位 0）
        meta_by_idx = {m["idx"]: m for m in meta}
        total = len(self.noise_instances)
        pack = []
        for idx in range(total):
            m = meta_by_idx.get(idx)
            if m:
                pack += [m["octBase"], m["splitBase"]]
            else:
                pack += [0, 0]
        n_inst = total
        pack_src = ", ".join(str(x) for x in pack)
        return f"""
// ===== old_blended_noise 数据驱动（D2：8 函数 → 1 个 interp_noise）=====
const int OLD_INSTANCES = {n_inst};
const int OLD_PACK[{len(pack)}] = int[]({pack_src});   // [octBase, splitBase] × 实例

float interp_noise(int idx, int sIdx) {{
    int octBase = OLD_PACK[idx * 2 + 0];
    int splitBase = OLD_PACK[idx * 2 + 1];
    // interpolation 8 octave（octBase+32..39）
    float n = 0.0f; float o = 1.0f;
    for (int q = 0; q < 8; q++) {{
        n += pn_section_f32(octBase + 32 + q, sIdx, splitBase + (32 + q) * 7) / o;
        o /= 2.0f;
    }}
    float qq = (n / 10.0f + 1.0f) / 2.0f;
    bool bl = qq >= 1.0f; bool bl2 = qq <= 0.0f;
    float l = 0.0f; float m = 0.0f; o = 1.0f;
    for (int r = 0; r < 16; r++) {{
        if (!bl) l += pn_section_f32(octBase + r, sIdx, splitBase + r * 7) / o;
        if (!bl2) m += pn_section_f32(octBase + 16 + r, sIdx, splitBase + (16 + r) * 7) / o;
        o /= 2.0f;
    }}
    float w = clamp(qq, 0.0f, 1.0f);
    return (l / 512.0f + w * (m / 512.0f - l / 512.0f)) / 128.0f;
}}
"""

    def _normal_func(self, idx, p, octBase, splitBase):
        # NormalNoise：CPU 预拆分坐标（int32 格点 + float 小数），GPU 纯 float 采样（无 fp64）
        # C2 数据驱动化：不再每实例一个函数，统一 normal_noise(noiseIdx, sIdx) + 参数表
        amps = p.get("amplitudes", [1.0])
        n = len(amps)
        persistence = (2.0 ** (n - 1)) / (2.0 ** n - 1.0)
        nonz = [i for i, a in enumerate(amps) if a != 0.0]
        j = min(nonz) if nonz else 0
        k = max(nonz) if nonz else 0
        create_amp = 0.1 * (1.0 + 1.0 / (k - j + 1))
        amplitude = 0.16666666666666666 / create_amp
        self.normal_meta.append({
            "idx": idx, "n": n, "octBase": octBase, "splitBase": splitBase,
            "persistence": persistence, "amplitude": amplitude, "amps": amps,
        })
        return ""   # 不生成独立函数（数据驱动单函数 normal_noise 统一生成）

    def _normal_noise_glsl(self):
        """生成 normal_noise(noiseIdx, sIdx) 单函数 + 参数表（139 函数 → 1）。"""
        meta = self.normal_meta
        if not meta:
            return ""
        def flit(x):
            s = format(float(x), '.17g')
            if '.' not in s and 'e' not in s and 'E' not in s:
                s += '.0'
            return s + 'f'
        # 每实例 5 参数打包：[n, octBase, splitBase, persistence, amplitude]
        # 修复（D16 续）：参数表按 noise_instances 全量索引对齐（normal_noise 的 noiseIdx = slot base + corner
        #   = noise_instances 索引，含 old_blended；old 位置占位 0，永不读）
        meta_by_idx = {m["idx"]: m for m in meta}
        total = len(self.noise_instances)
        pack = []
        pack_f = []
        amps_all = []
        amp_off = [0] * total
        for idx in range(total):
            m = meta_by_idx.get(idx)
            if m:
                pack += [m["n"], m["octBase"], m["splitBase"]]
                pack_f += [m["persistence"], m["amplitude"]]
                amp_off[idx] = len(amps_all)
                amps_all.extend(m["amps"])
            else:
                pack += [0, 0, 0]
                pack_f += [0.0, 0.0]
                amp_off[idx] = len(amps_all)
        n_inst = total
        pack_src = ", ".join(str(x) for x in pack)
        pack_f_src = ", ".join(flit(x) for x in pack_f)
        amps_src = ", ".join(flit(x) for x in amps_all)
        amp_off_src = ", ".join(str(x) for x in amp_off)
        return f"""
// ===== NormalNoise 数据驱动（C2：139 函数 → 1 个 normal_noise）=====
const int NORMAL_INSTANCES = {n_inst};
// 每实例 int 参数：n, octBase, splitBase（3 int/实例）
const int NORMAL_PACK[{len(pack)}] = int[]({pack_src});
// 每实例 float 参数：persistence, amplitude（2 float/实例）
const float NORMAL_PACK_F[{len(pack_f)}] = float[]({pack_f_src});
// 全部实例的 amps 连续表
const float NORMAL_AMPS[{len(amps_all)}] = float[]({amps_src});
// 每实例 amps 起始偏移
const int NORMAL_AMP_OFF[{len(amp_off)}] = int[]({amp_off_src});

float normal_noise(int noiseIdx, int sIdx) {{
    int base = noiseIdx * 3;
    int n = NORMAL_PACK[base + 0];
    int octBase = NORMAL_PACK[base + 1];
    int splitBase = NORMAL_PACK[base + 2];
    float persistence = NORMAL_PACK_F[noiseIdx * 2 + 0];
    float amplitude = NORMAL_PACK_F[noiseIdx * 2 + 1];
    int ampOff = NORMAL_AMP_OFF[noiseIdx];
    // first sampler（拆分坐标在 splitCoord，CPU 预计算 int32 格点 + float 小数）
    float d = 0.0f;
    float f = persistence;
    for (int i = 0; i < n; i++) {{
        int b = sIdx * SPLIT_TOTAL + splitBase + i * 6;
        int ix = int(splitBuf.splitCoord[b + 0]); int iy = int(splitBuf.splitCoord[b + 1]); int iz = int(splitBuf.splitCoord[b + 2]);
        float gx = splitBuf.splitCoord[b + 3]; float gy = splitBuf.splitCoord[b + 4]; float gz = splitBuf.splitCoord[b + 5];
        float ns = pn_sample3_f32(octBase + i, ix, iy, iz, gx, gy, gz);
        d += NORMAL_AMPS[ampOff + i] * ns * f;
        f /= 2.0f;
    }}
    // second sampler（拆分坐标偏移 + 6n）
    float d2 = 0.0f;
    f = persistence;
    for (int i = 0; i < n; i++) {{
        int b = sIdx * SPLIT_TOTAL + splitBase + 6 * n + i * 6;
        int ix = int(splitBuf.splitCoord[b + 0]); int iy = int(splitBuf.splitCoord[b + 1]); int iz = int(splitBuf.splitCoord[b + 2]);
        float gx = splitBuf.splitCoord[b + 3]; float gy = splitBuf.splitCoord[b + 4]; float gz = splitBuf.splitCoord[b + 5];
        float ns = pn_sample3_f32(octBase + n + i, ix, iy, iz, gx, gy, gz);
        d2 += NORMAL_AMPS[ampOff + i] * ns * f;
        f /= 2.0f;
    }}
    return (d + d2) * amplitude;
}}
"""

    def _shader_template(self, expr, funcs):
        funcs_src = "\n".join(funcs)
        no_old = 'no_old' in self.diag
        # D20 诊断：no_old 时去掉 fp64 计算链（保留 GRADIENTS 转 float + mapPermD，float normal 仍需）
        if no_old:
            fp64_ext = ""
            gradients = """// ===== float 工具（D20 诊断：GRADIENTS 转 float，去 fp64）=====
const float GRADIENTS[16][3] = {
    { 1,  1,  0}, {-1,  1,  0}, { 1, -1,  0}, {-1, -1,  0},
    { 1,  0,  1}, {-1,  0,  1}, { 1,  0, -1}, {-1,  0, -1},
    { 0,  1,  1}, { 0, -1,  1}, { 0,  1, -1}, { 0, -1, -1},
    { 1,  1,  0}, { 0, -1,  1}, {-1,  1,  0}, { 0, -1, -1}
};"""
            fp64_funcs = ""
            octave_func = ""
        else:
            fp64_ext = "#extension GL_ARB_gpu_shader_fp64 : require\n"
            gradients = """// ===== double 工具（old_blended_noise 用）=====
const double GRADIENTS[16][3] = {
    { 1,  1,  0}, {-1,  1,  0}, { 1, -1,  0}, {-1, -1,  0},
    { 1,  0,  1}, {-1,  0,  1}, { 1,  0, -1}, {-1,  0, -1},
    { 0,  1,  1}, { 0, -1,  1}, { 0,  1, -1}, { 0, -1, -1},
    { 1,  1,  0}, { 0, -1,  1}, {-1,  1,  0}, { 0, -1, -1}
};"""
            # P2-3：fp64 死代码清理——pn_sectionD/pn_sample5/octave_noise_f32 无调用者
            # （old_blended 实际走 interp_noise→pn_section_f32，float + splitCoord 数据驱动）；
            # GRADIENTS double 版保留（gradDotF 活代码用 float() 读它），fp64 扩展保留（double 数组需它）
            fp64_funcs = ""
            octave_func = ""
        return f"""#version 450
{fp64_ext}#extension GL_EXT_control_flow_attributes : require

layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// 坐标输入（int 块坐标，x,y,z 三元组）
layout(set = 0, binding = 0, std430) buffer CoordBuf {{ int coords[]; }} coord;
// perm 表（每 octave 256 uint，连续）
layout(set = 0, binding = 1, std430) buffer PermBuf {{ uint perm[]; }} permBuf;
// 输出 density
layout(set = 0, binding = 3, std430) buffer OutBuf {{ float density[]; }} outBuf;
// 拆分坐标（CPU 预计算：每采样点 SPLIT_TOTAL 个 float，[ix,iy,iz,gx,gy,gz] × 每 octave）
layout(set = 0, binding = 4, std430) buffer SplitBuf {{ float splitCoord[]; }} splitBuf;
// val 栈（方案1c：解释器中间值，每采样点 9 区段 = 8 角点 + 1 顶层；无实例名 → valBuf 直接可访问）
layout(set = 0, binding = 5, std430) buffer ValBuf {{ float valBuf[]; }};
const int SPLIT_TOTAL = {self.split_total};

{gradients}
int mapPermD(int octBase, int v) {{ return int(permBuf.perm[octBase * 256 + uint(v & 255)]); }}
{fp64_funcs}
// ===== float 工具（NormalNoise/spline/算术 用）=====
const int minY = -64;   // overworld 维度 minY（interpolated cell 网格用）
int floorDivP(int a, int b) {{ int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }}
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
// 5 参数 sample（old_blended_noise 用）：读 7 值拆分坐标 [ix,iy,iz,gx,gy(h-n),gz,fadeY(h)]，float 采样
float pn_section_f32(int octBase, int sIdx, int splitOffset) {{
    int b = sIdx * SPLIT_TOTAL + splitOffset;
    int sx = int(splitBuf.splitCoord[b + 0]);
    int sy = int(splitBuf.splitCoord[b + 1]);
    int sz = int(splitBuf.splitCoord[b + 2]);
    float lx = splitBuf.splitCoord[b + 3];
    float ly = splitBuf.splitCoord[b + 4];
    float lz = splitBuf.splitCoord[b + 5];
    float fadeY = splitBuf.splitCoord[b + 6];
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
    float r = perlinFadeF(lx); float s = perlinFadeF(fadeY); float t = perlinFadeF(lz);
    float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
    float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
    float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
    return lerpF(t, y0, y1);
}}
{octave_func}{funcs_src}

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

    # ---- C 方案：角点级拆 shader（8 corner + interp + noodle + merge）----
    # 实测：noodle 45 函数 1.88s ✅；factor 122 函数 >10min ❌ → 阈值在 45~122 间陡峭爆炸。
    # 角点数据：interp_5 的 8 角点噪声完全不相交（8×13），每 corner shader ≤13 噪声 → 必秒级。
    # 方案：corner_k.comp（interp_5 第 k 角点采样 → midCk）；interp.comp（读 8 角点 → 三线性 → squeeze → midA）；
    #       noodle.comp（interp_1..4 + 32 噪声 → midB）；merge.comp（min(a,b) → out）
    # 【P2-4 标注：本路径已弃用】C 方案（角点级拆 shader）被 G1-G4 系列实测否定（拆 shader 不是编译
    # 时间的正解——函数体复杂度才是主因，G2），且 A 方案（SSBO 化）改了 _spline_ssbo_glsl 输出形态
    # （新增 binding 6-11 spline SSBO + COORD_SLOT_TABLE），corner 宿主未适配、未验证（corner_*.comp
    # 是 B1a 时代旧产物）。如需启用 MUST 先同步宿主绑定 + 重新验证正确性。
    def gen_split_shaders(self, root_df):
        """生成角点级拆分 shader 集合。返回 dict：{name: (src, noise_ids)}。"""
        expr = self.gen(root_df)
        noise_func_srcs = {}
        octBase = 0
        splitBase = 0
        for idx, (kind, params) in enumerate(self.noise_instances):
            if kind == "old_blended":
                noise_func_srcs[idx] = self._old_blended_func(idx, params, octBase, splitBase)
                octBase += 40
                splitBase += 7 * 40
            elif kind == "normal":
                n = len(params.get("amplitudes", [1.0]))
                noise_func_srcs[idx] = self._normal_func(idx, params, octBase, splitBase)
                octBase += 2 * n
                splitBase += 6 * 2 * n
        self.split_total = splitBase
        ssbo_glsl = self._spline_ssbo_glsl()

        import re as _re
        def noise_ids_in(expr_str):
            return set(int(m.group(1)) for m in _re.finditer(r'(?:normal_noise|interp_noise)_(\d+)', expr_str))

        interp5_samples = None
        interp1_4_samples = {}
        for idx, samples in self.interp_funcs:
            if idx == 5:
                interp5_samples = samples
            elif idx in (1, 2, 3, 4):
                interp1_4_samples[idx] = samples
        assert interp5_samples is not None, "interp_5 缺失"

        # 主表达式：min(squeeze(0.64*interp_5(...)), df_overworld_caves_noodle(...))
        m = _re.match(r"min\((.*), (df_overworld_caves_noodle\([^)]*\))\)$", expr)
        if not m:
            raise ValueError(f"C: 无法解析主表达式 min 结构: {expr[:120]}")
        noodle_call = m.group(2)

        # 8 个 corner shader（含 spline：角点表达式用 spline_eval，需 ssbo_glsl + spline coordinate 噪声）
        import re as _re2
        ssbo_noise_ids = set()
        if ssbo_glsl:
            for mm in _re2.finditer(r'(?:normal_noise|interp_noise)_(\d+)', ssbo_glsl):
                ssbo_noise_ids.add(int(mm.group(1)))
        corner_srcs = {}
        corner_noise_ids = []
        corner_letters = ['000','100','010','110','001','101','011','111']
        for k, sample in enumerate(interp5_samples):
            ids = sorted(set(noise_ids_in(sample)) | ssbo_noise_ids)
            corner_noise_ids.append(ids)
            corner_main = f"""    uint idx = gl_GlobalInvocationID.x;
    if (idx >= midC.mid.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    midC.mid[idx] = corner_{k}(int(idx), ix, iy, iz);"""
            corner_buf = f"""layout(set = 0, binding = 5, std430) buffer MidCBuf {{ float mid[]; }} midC;"""
            eval_fn = (f"float corner_{k}(int sIdx, int ix, int iy, int iz) {{\n"
                       f"    int cy = (iy - minY) / 8;\n"
                       f"    return {sample};\n}}")
            corner_srcs[f"corner_{k}"] = build_shader_alt(
                self, eval_fn, ids, [], [], corner_buf, corner_main, ssbo_glsl, noise_func_srcs, with_interp=False)

        # interp shader：读 8 角点 → 三线性 → squeeze(0.64*x)
        interp_main = """    uint idx = gl_GlobalInvocationID.x;
    if (idx >= midA.mid.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    int chunkX = floorDivP(ix, 16); int chunkZ = floorDivP(iz, 16);
    int gx = ix - chunkX * 16; int gy = iy - minY; int gz = iz - chunkZ * 16;
    int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;
    float fx = float(gx % 4) / 4.0f; float fy = float(gy % 8) / 8.0f; float fz = float(gz % 4) / 4.0f;
    float d000 = midC0.mid[idx], d100 = midC1.mid[idx], d010 = midC2.mid[idx], d110 = midC3.mid[idx];
    float d001 = midC4.mid[idx], d101 = midC5.mid[idx], d011 = midC6.mid[idx], d111 = midC7.mid[idx];
    float d00 = d000 + (d100 - d000) * fx; float d10 = d010 + (d110 - d010) * fx;
    float d01 = d001 + (d101 - d001) * fx; float d11 = d011 + (d111 - d011) * fx;
    float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;
    float interp = d0 + (d1 - d0) * fz;
    float x = 0.64f * interp;
    float clamped = clamp(x, -1.0f, 1.0f);
    midA.mid[idx] = clamped / 2.0f - clamped * clamped * clamped / 24.0f;"""
        interp_buf = """layout(set = 0, binding = 5, std430) buffer MidC0Buf { float mid[]; } midC0;
layout(set = 0, binding = 6, std430) buffer MidC1Buf { float mid[]; } midC1;
layout(set = 0, binding = 7, std430) buffer MidC2Buf { float mid[]; } midC2;
layout(set = 0, binding = 8, std430) buffer MidC3Buf { float mid[]; } midC3;
layout(set = 0, binding = 9, std430) buffer MidC4Buf { float mid[]; } midC4;
layout(set = 0, binding = 10, std430) buffer MidC5Buf { float mid[]; } midC5;
layout(set = 0, binding = 11, std430) buffer MidC6Buf { float mid[]; } midC6;
layout(set = 0, binding = 12, std430) buffer MidC7Buf { float mid[]; } midC7;
layout(set = 0, binding = 13, std430) buffer MidABuf { float mid[]; } midA;"""
        interp_src = build_shader_alt(
            self, "float eval_i(int a) { return 0.0; }", [], [], [], interp_buf, interp_main,
            None, noise_func_srcs, with_interp=False)

        # noodle shader：interp_1..4 完整函数 + 32 噪声
        noodle_ids = set()
        for idx, samples in interp1_4_samples.items():
            for s in samples:
                noodle_ids |= noise_ids_in(s)
        noodle_main = """    uint idx = gl_GlobalInvocationID.x;
    if (idx >= midB.mid.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    midB.mid[idx] = df_overworld_caves_noodle(int(idx), ix, iy, iz);"""
        noodle_buf = """layout(set = 0, binding = 5, std430) buffer MidBBuf { float mid[]; } midB;"""
        noodle_src = build_shader_alt(
            self,
            f"float eval_b(int sIdx, int ix, int iy, int iz) {{\n    float x = float(ix), y = float(iy), z = float(iz);\n    return {noodle_call};\n}}",
            sorted(noodle_ids), [1, 2, 3, 4], ["df_overworld_caves_noodle"],
            noodle_buf, noodle_main, None, noise_func_srcs, with_interp=True)

        # merge shader
        merge_main = """    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    outBuf.density[idx] = min(midA.mid[idx], midB.mid[idx]);"""
        merge_buf = """layout(set = 0, binding = 5, std430) buffer MidABuf2 { float mid[]; } midA;
layout(set = 0, binding = 6, std430) buffer MidBBuf2 { float mid[]; } midB;"""
        merge_src = build_shader_alt(
            self, "float eval_m(int a, int b) { return min(a, b); }", [], [], [],
            merge_buf, merge_main, None, noise_func_srcs, with_interp=False)

        result = {}
        for k in range(8):
            result[f"corner_{k}"] = (corner_srcs[f"corner_{k}"], corner_noise_ids[k])
        result["interp"] = (interp_src, [])
        result["noodle"] = (noodle_src, sorted(noodle_ids))
        result["merge"] = (merge_src, [])
        return result

    def _shader_template_alt(self, eval_func, funcs, extra_bufs, main_body, with_ssbo=False):
        """拆分版 shader 模板：多中间 buffer（binding 5+），输出可指向 mid 或 out。"""
        funcs_src = "\n".join(f for f in funcs if f)
        return f"""#version 450
#extension GL_ARB_gpu_shader_fp64 : require
#extension GL_EXT_control_flow_attributes : require

layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0, std430) buffer CoordBuf {{ int coords[]; }} coord;
layout(set = 0, binding = 1, std430) buffer PermBuf {{ uint perm[]; }} permBuf;
layout(set = 0, binding = 3, std430) buffer OutBuf {{ float density[]; }} outBuf;
layout(set = 0, binding = 4, std430) buffer SplitBuf {{ float splitCoord[]; }} splitBuf;
{extra_bufs}
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
double interp_noise_f64(int octBase, int sIdx, int splitOffset) {{
    int b = sIdx * SPLIT_TOTAL + splitOffset;
    int ix = int(splitBuf.splitCoord[b + 0]); int iy = int(splitBuf.splitCoord[b + 1]); int iz = int(splitBuf.splitCoord[b + 2]);
    double gx = double(splitBuf.splitCoord[b + 3]); double gy = double(splitBuf.splitCoord[b + 4]); double gz = double(splitBuf.splitCoord[b + 5]);
    double fy = double(splitBuf.splitCoord[b + 6]);
    return pn_sectionD(octBase, ix, iy, iz, gx, gy, gz, fy);
}}

// ===== float 工具（NormalNoise/spline/算术 用）=====
const int minY = -64;
int floorDivP(int a, int b) {{ int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }}
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
float gradDotF(int hash, float x, float y, float z) {{
    vec3 g = vec3(float(GRADIENTS[hash & 15][0]), float(GRADIENTS[hash & 15][1]), float(GRADIENTS[hash & 15][2]));
    return g.x * x + g.y * y + g.z * z;
}}
float pn_sample3_f32(int octBase, int sx, int sy, int sz, float lx, float ly, float lz) {{
    int i = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
    int k = mapPermD(octBase, i + sy); int l = mapPermD(octBase, i + sy + 1);
    int m = mapPermD(octBase, j + sy); int n = mapPermD(octBase, j + sy + 1);
    float d = gradDotF(mapPermD(octBase, k + sz),     lx,     ly,     lz);
    float e = gradDotF(mapPermD(octBase, m + sz),     lx - 1.0, ly,     lz);
    float f = gradDotF(mapPermD(octBase, l + sz),     lx,     ly - 1.0, lz);
    float g = gradDotF(mapPermD(octBase, n + sz),     lx - 1.0, ly - 1.0, lz);
    float h = gradDotF(mapPermD(octBase, k + sz + 1), lx,     ly,     lz - 1.0);
    float o = gradDotF(mapPermD(octBase, m + sz + 1), lx - 1.0, ly,     lz - 1.0);
    float p = gradDotF(mapPermD(octBase, l + sz + 1), lx,     ly - 1.0, lz - 1.0);
    float q = gradDotF(mapPermD(octBase, n + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
    float r = perlinFadeF(lx); float s = perlinFadeF(ly); float t = perlinFadeF(lz);
    float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
    float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
    float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
    return lerpF(t, y0, y1);
}}
// 5 参数 sample（old_blended_noise 用）：读 7 值拆分坐标，float 采样
float pn_section_f32(int octBase, int sIdx, int splitOffset) {{
    int b = sIdx * SPLIT_TOTAL + splitOffset;
    int sx = int(splitBuf.splitCoord[b + 0]);
    int sy = int(splitBuf.splitCoord[b + 1]);
    int sz = int(splitBuf.splitCoord[b + 2]);
    float lx = splitBuf.splitCoord[b + 3];
    float ly = splitBuf.splitCoord[b + 4];
    float lz = splitBuf.splitCoord[b + 5];
    float fadeY = splitBuf.splitCoord[b + 6];
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
    float r = perlinFadeF(lx); float s = perlinFadeF(fadeY); float t = perlinFadeF(lz);
    float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
    float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
    float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
    return lerpF(t, y0, y1);
}}

{funcs_src}

{eval_func}

void main() {{
{main_body}
}}
"""


def build_shader_alt(gen, eval_func, noise_ids, extra_interps, extra_regs, extra_bufs, main_body, ssbo_glsl, noise_func_srcs, with_interp=False):
    """模块级辅助：构造拆分 shader（复用 gen 的 _shader_template_alt）。"""
    funcs = []
    for iidx in extra_interps:
        funcs.append(f"float interp_{iidx}(int sIdx, int ix, int iy, int iz);")
    for idx in sorted(noise_ids):
        funcs.append(noise_func_srcs[idx])
    for fname, fexpr in gen.registry_defs:
        if fname in extra_regs:
            funcs.append(f"float {fname}(int sIdx, int ix, int iy, int iz) {{\n    float x = float(ix), y = float(iy), z = float(iz);\n    return {fexpr};\n}}\n")
    if ssbo_glsl:
        funcs.append(ssbo_glsl)
    if with_interp:
        interp_map = dict(gen.interp_funcs)
        for iidx in extra_interps:
            samples = interp_map[iidx]
            lines = [f"float interp_{iidx}(int sIdx, int ix, int iy, int iz) {{"]
            lines.append("    int chunkX = floorDivP(ix, 16); int chunkZ = floorDivP(iz, 16);")
            lines.append("    int gx = ix - chunkX * 16; int gy = iy - minY; int gz = iz - chunkZ * 16;")
            lines.append("    int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;")
            lines.append("    float fx = float(gx % 4) / 4.0f; float fy = float(gy % 8) / 8.0f; float fz = float(gz % 4) / 4.0f;")
            for c in range(8):
                dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
                lines.append(f"    float d{dx}{dy}{dz} = {samples[c]};")
            lines.append("    float d00 = d000 + (d100 - d000) * fx; float d10 = d010 + (d110 - d010) * fx;")
            lines.append("    float d01 = d001 + (d101 - d001) * fx; float d11 = d011 + (d111 - d011) * fx;")
            lines.append("    float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;")
            lines.append("    return d0 + (d1 - d0) * fz;")
            lines.append("}")
            funcs.append("\n".join(lines))
    return gen._shader_template_alt(eval_func, funcs, extra_bufs, main_body, with_ssbo=bool(ssbo_glsl))


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


