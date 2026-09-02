# gen_tables_rs.py —— P2a：dfc_gen 数据表的 Rust 后端 emitter（lossless-accel 260903-03）
# 产物：
#   WorldgenRust/src/generated/dfc_cpu_tables.rs  —— const 表 + init 元数据 + spline_coord_fold（mod 级 include!）
#   WorldgenRust/src/generated/dfc_cpu_split.rs   —— fn split / fn split_top 生成体（impl DfcBackend 内 include!）
# 同源红线：与 gen_cpu（dfc_gen.py:1651）共用 gen_df 收集 + _compute_val_layout，禁止独立重算布局。
# 用法：Push-Location .investigations\perf-rework; python gen_tables_rs.py
import json, os, re, sys
import dfc_gen

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

OUT_DIR = r'E:\PYTHON\CoreSwap\WorldgenRust\src\generated'


def flit(x):
    """f32 字面量（Rust 无 f 后缀，带小数点即可推断；指数形合法）"""
    s = format(float(x), '.9g')
    if '.' not in s and 'e' not in s and 'E' not in s:
        s += '.0'
    return s


def dlit(x):
    s = format(float(x), '.17g')
    if '.' not in s and 'e' not in s and 'E' not in s:
        s += '.0'
    return s


def rs_expr(e):
    """C++/GLSL 风格表达式 → Rust：去 f 后缀、abs/min/max → f32::UFCS"""
    e = e.replace('std::fabs(', 'f32::abs(').replace('abs(', 'F32ABS(')
    e = re.sub(r'([0-9])[fF](?![0-9a-zA-Z_])', r'\1', e)
    e = e.replace('F32ABS(', 'f32::abs(')
    return e


class RustSplitEmitter:
    """_gen_split_lines 的 Rust 行模板直译（复制 dfc_gen.py:1541-1649 逻辑，仅换模板）。"""

    def __init__(self, g: dfc_gen.DfcGen):
        self.g = g
        # shift 噪声名 → shifts Vec 下标（顺序 = manifest shift_noises 插入序）
        self.shift_index = {k: i for i, k in enumerate(g.shift_noises.keys())}

    def _shift_rs(self, s, ax, ay, az):
        if s["type"] == "constant":
            return dlit(s['value'])
        key = s["noise_key"]
        i = self.shift_index[key]
        if s["type"] == "shift_a":
            return f'self.shifts[{i}].sample(({ax}) as f64 * 0.25, 0.0, ({az}) as f64 * 0.25) * 4.0'
        if s["type"] == "shift_b":
            return f'self.shifts[{i}].sample(({az}) as f64 * 0.25, ({ax}) as f64 * 0.25, 0.0) * 4.0'
        return f'self.shifts[{i}].sample(({ax}) as f64 * 0.25, ({ay}) as f64 * 0.25, ({az}) as f64 * 0.25) * 4.0'

    def _ws_rarity_rs(self, df, cx, cy, cz):
        g = self.g
        while isinstance(df, dict) and df.get("type") in ("minecraft:cache_once", "minecraft:cache_2d", "minecraft:cache_all_in_cell"):
            df = df.get("argument", df.get("input", 0.0))
        if isinstance(df, (int, float)):
            return dlit(df)
        t = df.get("type", "")
        if t == "minecraft:noise":
            key = df.get("noise", "") + g.noise_key_suffix
            chain = g.coord_chains[g.normal_chain_index[key]]
            vi = g.normal_vec_index[key]
            xs, ys = float(chain["xz_scale"]), float(chain["y_scale"])
            return f'self.normals[{vi}].sample(({cx}) as f64 * {dlit(xs)}, ({cy}) as f64 * {dlit(ys)}, ({cz}) as f64 * {dlit(xs)})'
        raise ValueError(f"D17: ws rarity 输入暂只支持 plain noise/const，遇到 {t}: {json.dumps(df)[:150]}")

    def gen(self, df, cx, cy, cz, corner0_only=False):
        g = self.g
        lines = []
        if isinstance(df, str):
            if df in ("minecraft:y", "minecraft:zero", "minecraft:shift_x", "minecraft:shift_z"):
                return lines
            return self.gen(g.resolve_ref(df), cx, cy, cz, corner0_only=corner0_only)
        if isinstance(df, (int, float)):
            return lines
        t = df.get("type", "")
        if t == "minecraft:spline":
            df = df.get("spline", df)
        if "coordinate" in df and "points" in df:
            lines += self.gen(df["coordinate"], cx, cy, cz, corner0_only=corner0_only)
            for p in df.get("points", []):
                v = p.get("value")
                if isinstance(v, dict) and "coordinate" in v and "points" in v:
                    lines += self.gen(v, cx, cy, cz, corner0_only=corner0_only)
            return lines
        if t in ("minecraft:noise", "minecraft:shifted_noise"):
            key = df.get("noise", "") + g.noise_key_suffix
            if key in g.split_visited:
                return lines
            g.split_visited.add(key)
            chain = g.coord_chains[g.normal_chain_index[key]]
            vi = g.normal_vec_index[key]
            sb = g.normal_split_base[key]
            n = len(g._resolve_noise_params(df.get("noise", ""))["amplitudes"])
            if chain.get("flat_cache"):
                ax = f"(({cx}) >> 2) << 2"; ay = "0"; az = f"(({cz}) >> 2) << 2"
            else:
                ax = cx; ay = cy; az = cz
            xs, ys = dlit(chain['xz_scale']), dlit(chain['y_scale'])
            if chain["type"] == "noise":
                dx, dy, dz = f"({ax}) as f64 * {xs}", f"({ay}) as f64 * {ys}", f"({az}) as f64 * {xs}"
            else:
                sx = self._shift_rs(chain["shift_x"], ax, ay, az)
                sy = self._shift_rs(chain["shift_y"], ax, ay, az)
                sz = self._shift_rs(chain["shift_z"], ax, ay, az)
                dx = f"({ax}) as f64 * {xs} + ({sx})"
                dy = f"({ay}) as f64 * {ys} + ({sy})"
                dz = f"({az}) as f64 * {xs} + ({sz})"
            lines.append(f'        Self::split_double(&self.normals[{vi}], {dx}, {dy}, {dz}, out, {sb}, {n});')
        elif t == "minecraft:old_blended_noise":
            key = (f"old_blended:{df.get('xz_scale',0.25)}:{df.get('y_scale',0.125)}:"
                   f"{df.get('xz_factor',80.0)}:{df.get('y_factor',160.0)}:"
                   f"{df.get('smear_scale_multiplier',8.0)}{g.noise_key_suffix}")
            if key in g.split_visited:
                return lines
            g.split_visited.add(key)
            vi = g.old_vec_index[key]
            sb = g.old_split_base[key]
            lines.append(f'        Self::split_old_blended(&self.olds[{vi}], {cx}, {cy}, {cz}, out, {sb});')
        elif t == "minecraft:interpolated":
            lines.append("    {")
            lines.append(f"        let _chunk_x = Self::floor_div({cx}, 16); let _chunk_z = Self::floor_div({cz}, 16);")
            lines.append(f"        let _gx = ({cx}) - _chunk_x * 16; let _gy = ({cy}) - MIN_Y; let _gz = ({cz}) - _chunk_z * 16;")
            lines.append("        let _cx = _gx / 4; let _cy = _gy / 8; let _cz = _gz / 4;")
            for c in (range(1) if corner0_only else range(8)):
                dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
                ax = f"(_chunk_x * 16 + (_cx + {dx}) * 4)"
                ay = f"(MIN_Y + (_cy + {dy}) * 8)"
                az = f"(_chunk_z * 16 + (_cz + {dz}) * 4)"
                old_suffix = g.noise_key_suffix
                g.noise_key_suffix = f"@c{c}"
                lines += self.gen(df.get("argument", df.get("input", 0.0)), ax, ay, az, corner0_only=corner0_only)
                g.noise_key_suffix = old_suffix
            lines.append("    }")
        elif t == "minecraft:weird_scaled_sampler":
            lines += self.gen(df.get("input", 0.0), cx, cy, cz, corner0_only=corner0_only)
            ws_key = df.get("noise", "") + ":ws" + g.noise_key_suffix
            if ws_key not in g.split_visited:
                g.split_visited.add(ws_key)
                vi = g.normal_vec_index[ws_key]
                sb = g.normal_split_base[ws_key]
                n = len(g._resolve_noise_params(df.get("noise", ""))["amplitudes"])
                rc = self._ws_rarity_rs(df.get("input", 0.0), cx, cy, cz)
                kind = 1 if df.get("rarity_value_mapper") == "type_2" else 0
                lines.append(f'        let _d = Self::ws_scale({kind}, {rc});')
                lines.append(f'        Self::split_double(&self.normals[{vi}], ({cx}) as f64 / _d, ({cy}) as f64 / _d, ({cz}) as f64 / _d, out, {sb}, {n});')
        else:
            for key in ("argument", "argument1", "argument2", "input", "when_in_range", "when_out_of_range"):
                if key in df:
                    lines += self.gen(df[key], cx, cy, cz, corner0_only=corner0_only)
        return lines


def main():
    g = dfc_gen.DfcGen(dfdir, ndir)
    root = g.gen_df(fd)
    manifest = g.gen_noise_manifest()
    normals = manifest["normal_instances"]
    shift_noises = manifest["shift_noises"]

    # ---- split 生成体（corner0_only 复用同一次遍历语义，与 gen_cpu 一致）----
    # 同源关键：gen_cpu 在 split walk 前先按 noise_instances 序重放 octBase/splitBase 并填
    # normal/old 的 split_base/vec_index（dfc_gen.py:1662-1678）——顺序不可颠倒。
    octBase = 0
    splitBase = 0
    for idx, (kind, params) in enumerate(g.noise_instances):
        if kind == "old_blended":
            g._old_blended_func(idx, params, octBase, splitBase)   # 填 old_meta（D2 数据驱动化）
            g.old_split_base[params["_key"]] = splitBase
            g.old_vec_index[params["_key"]] = len(g.old_vec_index)
            octBase += 40
            splitBase += 7 * 40
        elif kind == "normal":
            n = len(params.get("amplitudes", [1.0]))
            g._normal_func(idx, params, octBase, splitBase)        # 填 normal_meta（n/octBase/splitBase/persistence/amplitude/amps）
            g.normal_split_base[params["_key"]] = splitBase
            g.normal_vec_index[params["_key"]] = len(g.normal_vec_index)
            octBase += 2 * n
            splitBase += 6 * 2 * n
    em = RustSplitEmitter(g)
    g.split_visited.clear()
    split_lines = em.gen(fd, "x", "y", "z")
    g.split_visited.clear()
    split_top_lines = em.gen(fd, "x", "y", "z", corner0_only=True)

    # ---- init 元数据（对齐 gen_cpu init_lines 语义）----
    # （octBase/splitBase 重放填充已在 split walk 前完成，见上）

    shift_init = ",\n".join(
        f'    NoiseInit {{ key: {json.dumps(k)}, first_octave: {int(np["firstOctave"])}, amps: &[{", ".join(dlit(a) for a in np["amplitudes"])}] }}'
        for k, np in shift_noises.items())
    normal_init = ",\n".join(
        f'    NoiseInit {{ key: {json.dumps(ni["noise_key"])}, first_octave: {int(ni["firstOctave"])}, amps: &[{", ".join(dlit(a) for a in ni["amplitudes"])}] }}'
        for ni in normals)
    old_init = ",\n".join(
        f'    OldInit {{ xz_scale: {dlit(p["xz_scale"])}, y_scale: {dlit(p["y_scale"])}, xz_factor: {dlit(p["xz_factor"])}, y_factor: {dlit(p["y_factor"])}, smear: {dlit(p["smear"])} }}'
        for kind, p in g.noise_instances if kind == "old_blended")

    perm_size = sum(40 if k == "old_blended" else 2 * len(p.get("amplitudes", [1.0])) for k, p in g.noise_instances) * 256

    # ---- 闭包/节点表（镜像 gen_cpu_sampling 1906-2103，仅换语法）----
    nodes = g.df_nodes
    n_nodes = len(nodes)
    top_root = n_nodes - 1
    layout = g._compute_val_layout()
    read_fields = layout["read_fields"]
    closures = layout["closures"]
    top_closure = layout["top_closure"]
    top_pos = layout["top_pos"]
    top_slot = layout["top_slot"]
    top_peak = layout["top_peak"]

    def _map_a(cur_pos, t, v, f):
        if v >= 0 and v in cur_pos and f in read_fields.get(t, ()):
            return cur_pos[v]
        return v

    cls_off, cls_len, cls_peak, cls_root_pos = [], [], [], []
    cls_type, cls_a1, cls_a2, cls_a3 = [], [], [], []
    cls_f0, cls_f1, cls_f2, cls_f3, cls_slot = [], [], [], [], []
    acc = 0
    for k, (closure, pos, slot, peak) in enumerate(closures):
        cls_off.append(acc); cls_len.append(len(closure)); cls_peak.append(peak)
        root = g.interp_roots[k] if k < len(g.interp_roots) else (closure[0] if closure else -1)
        cls_root_pos.append(pos[root] if root in pos else 0)
        for ci, i in enumerate(closure):
            nd = nodes[i]; t = nd["type"]
            cls_type.append(t)
            cls_a1.append(_map_a(pos, t, nd["a1"], "a1"))
            cls_a2.append(_map_a(pos, t, nd["a2"], "a2"))
            cls_a3.append(_map_a(pos, t, nd["a3"], "a3"))
            cls_f0.append(flit(nd["f0"])); cls_f1.append(flit(nd["f1"]))
            cls_f2.append(flit(nd["f2"])); cls_f3.append(flit(nd["f3"]))
            cls_slot.append(slot[ci])
        acc += len(closure)
    if not closures:
        cls_off = [0]; cls_len = [0]; cls_peak = [1]; cls_root_pos = [0]
        cls_type = cls_a1 = cls_a2 = cls_a3 = cls_slot = [0]
        cls_f0 = cls_f1 = cls_f2 = cls_f3 = [flit(0.0)]
    cls_total = len(cls_type)

    top_type, top_a1, top_a2, top_a3 = [], [], [], []
    top_f0, top_f1, top_f2, top_f3 = [], [], [], []
    top_slot_flat = []
    for ci, i in enumerate(top_closure):
        nd = nodes[i]; t = nd["type"]
        top_type.append(t)
        top_a1.append(_map_a(top_pos, t, nd["a1"], "a1"))
        top_a2.append(_map_a(top_pos, t, nd["a2"], "a2"))
        top_a3.append(_map_a(top_pos, t, nd["a3"], "a3"))
        top_f0.append(flit(nd["f0"])); top_f1.append(flit(nd["f1"]))
        top_f2.append(flit(nd["f2"])); top_f3.append(flit(nd["f3"]))
        top_slot_flat.append(top_slot[ci])
    top_len = len(top_closure)
    if top_len == 0:
        top_type = top_a1 = top_a2 = top_a3 = top_slot_flat = [0]
        top_f0 = top_f1 = top_f2 = top_f3 = [flit(0.0)]

    # noise slot 表
    ns_bases = ", ".join(str(s["base"]) for s in g.noise_slots) if g.noise_slots else "0"
    ns_strides = ", ".join(str(s["stride"]) for s in g.noise_slots) if g.noise_slots else "0"
    n_slots = len(g.noise_slots)

    # coord slots + fold
    coord_slots, coord_folds, coord_ok = [], [], True
    for ct, expr in enumerate(g.spline_coords):
        m = re.search(r'normal_noise\(NOISE_SLOT_BASE\[(\d+)\] \+ corner \* NOISE_SLOT_STRIDE\[\d+\], sIdx\)', expr)
        if not m:
            coord_ok = False
            break
        coord_slots.append(int(m.group(1)))
        coord_folds.append(expr.replace(m.group(0), 'v'))
    coord_src = ", ".join(str(x) for x in coord_slots) if (coord_ok and coord_slots) else "0"
    n_coord = len(g.spline_coords)
    if coord_ok:
        fold_lines = "\n".join(
            f"        {ct} => {rs_expr(fold)}," for ct, fold in enumerate(coord_folds))
        fold_fn = (f"pub fn spline_coord_fold(coord_type: usize, v: f32) -> f32 {{\n"
                   f"    match coord_type {{\n{fold_lines}\n        _ => v,\n    }}\n}}\n")
    else:
        fold_fn = "pub fn spline_coord_fold(_coord_type: usize, v: f32) -> f32 { v }\n"

    # normal/old pack
    n_inst = len(g.noise_instances)
    meta_by_idx = {m["idx"]: m for m in g.normal_meta}
    pack, pack_f, amps_all, amp_off = [], [], [], [0] * n_inst
    for idx in range(n_inst):
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
    old_by_idx = {m["idx"]: m for m in g.old_meta}
    pack_old = []
    for idx in range(n_inst):
        m = old_by_idx.get(idx)
        pack_old += [m["octBase"], m["splitBase"]] if m else [0, 0]

    interp_roots_src = ", ".join(str(r) for r in g.interp_roots) if g.interp_roots else "0"
    n_interp = len(g.interp_roots) if g.interp_roots else 1

    def arr_i32(name, vals, const=True):
        c = "pub const" if const else "pub static"
        return f"{c} {name}: [i32; {len(vals)}] = [{', '.join(str(v) for v in vals)}];"

    def arr_f32(name, vals):
        return f"pub const {name}: [f32; {len(vals)}] = [{', '.join(vals)}];"

    # ---- 产物 1：tables ----
    t = []
    t.append("// 自动生成（gen_tables_rs.py，DFC Rust 后端数据表），勿手改。同源：dfc_gen.py gen_cpu/gen_df + _compute_val_layout")
    t.append(f"pub const SPLIT_TOTAL: usize = {manifest['split_total']};")
    t.append(f"pub const PERM_SIZE: usize = {perm_size};")
    t.append("pub const MIN_Y: i32 = -64; // overworld 维度 minY（interpolated cell 网格）")
    t.append(f"pub const N_INTERP: usize = {n_interp};")
    t.append(f"pub const N_SHIFTS: usize = {len(shift_noises)};")
    t.append(f"pub const N_NORMALS: usize = {len(normals)};")
    t.append(f"pub const N_OLDS: usize = {sum(1 for k, _ in g.noise_instances if k == 'old_blended')};")
    t.append(f"pub const SPLINE_BIND_BASE: usize = {g.spline_bind_base};")
    t.append(f"pub const SPLINE_NODES: usize = {len(g.spline_ssbo_nodes)};")
    t.append(f"pub const DF_NODES: usize = {n_nodes};")
    t.append(f"pub const TOP_ROOT: usize = {top_root};")
    t.append(f"pub const N_CLOSURE: usize = {n_interp};")
    t.append(f"pub const CLOSURE_MAX_SLOTS: usize = {max(cls_peak) if cls_peak else 1};")
    t.append(f"pub const TOP_CLOSURE_LEN: usize = {top_len};")
    t.append(f"pub const VAL_SLOTS_TOP: usize = {top_peak};")
    t.append(f"pub const TOP_ROOT_POS: usize = {g.top_root_pos};")
    t.append(f"pub const NORMAL_INSTANCES: usize = {n_inst};")
    t.append(f"pub const NOISE_SLOT_COUNT: usize = {n_slots if n_slots else 1};")
    t.append(f"pub const COORD_TYPES: usize = {n_coord if n_coord else 1};")
    t.append("")
    t.append("#[derive(Clone, Copy)] pub struct NoiseInit { pub key: &'static str, pub first_octave: i32, pub amps: &'static [f64] }")
    t.append("#[derive(Clone, Copy)] pub struct OldInit { pub xz_scale: f64, pub y_scale: f64, pub xz_factor: f64, pub y_factor: f64, pub smear: f64 }")
    t.append("")
    t.append(f"pub static SHIFT_INIT: [NoiseInit; N_SHIFTS] = [\n{shift_init},\n];")
    t.append(f"pub static NORMAL_INIT: [NoiseInit; N_NORMALS] = [\n{normal_init},\n];")
    t.append(f"pub static OLD_INIT: [OldInit; N_OLDS] = [\n{old_init},\n];")
    t.append("")
    t.append(arr_i32("DF_TYPE", [n["type"] for n in nodes]))
    t.append(arr_i32("DF_A1", [n["a1"] for n in nodes]))
    t.append(arr_i32("DF_A2", [n["a2"] for n in nodes]))
    t.append(arr_i32("DF_A3", [n["a3"] for n in nodes]))
    t.append(arr_f32("DF_F0", [flit(n["f0"]) for n in nodes]))
    t.append(arr_f32("DF_F1", [flit(n["f1"]) for n in nodes]))
    t.append(arr_f32("DF_F2", [flit(n["f2"]) for n in nodes]))
    t.append(arr_f32("DF_F3", [flit(n["f3"]) for n in nodes]))
    t.append(arr_i32("INTERP_ROOTS", g.interp_roots if g.interp_roots else [0]))
    t.append(arr_i32("CLOSURE_OFF", cls_off))
    t.append(arr_i32("CLOSURE_LEN", cls_len))
    t.append(arr_i32("CLOSURE_VAL_SLOTS", cls_peak))
    t.append(arr_i32("CLOSURE_ROOT_POS", cls_root_pos))
    t.append(arr_i32("CLOSURE_TYPE", cls_type))
    t.append(arr_i32("CLOSURE_A1", cls_a1))
    t.append(arr_i32("CLOSURE_A2", cls_a2))
    t.append(arr_i32("CLOSURE_A3", cls_a3))
    t.append(arr_f32("CLOSURE_F0", cls_f0))
    t.append(arr_f32("CLOSURE_F1", cls_f1))
    t.append(arr_f32("CLOSURE_F2", cls_f2))
    t.append(arr_f32("CLOSURE_F3", cls_f3))
    t.append(arr_i32("CLOSURE_SLOT", cls_slot))
    t.append(arr_i32("TOP_TYPE", top_type))
    t.append(arr_i32("TOP_A1", top_a1))
    t.append(arr_i32("TOP_A2", top_a2))
    t.append(arr_i32("TOP_A3", top_a3))
    t.append(arr_f32("TOP_F0", top_f0))
    t.append(arr_f32("TOP_F1", top_f1))
    t.append(arr_f32("TOP_F2", top_f2))
    t.append(arr_f32("TOP_F3", top_f3))
    t.append(arr_i32("TOP_SLOT", top_slot_flat))
    # spline（A1b 数据）
    node_pack = []
    for nd in g.spline_ssbo_nodes:
        node_pack += [nd["coordType"], nd["n"], nd["locBegin"], nd["derBegin"], nd["valBegin"]]
    t.append(arr_i32("SPLINE_NODE_PACK", node_pack))
    t.append(arr_f32("SPLINE_LOCS", [flit(x) for x in g.spline_ssbo_locs]))
    t.append(arr_f32("SPLINE_DERS", [flit(x) for x in g.spline_ssbo_ders]))
    t.append(arr_f32("SPLINE_VAL_F", [flit(x) for x in g.spline_ssbo_val_f]))
    t.append(arr_i32("SPLINE_VAL_KIND", g.spline_ssbo_val_kind))
    t.append(arr_i32("SPLINE_VAL_NODE", g.spline_ssbo_val_node))
    t.append(arr_i32("NOISE_SLOT_BASE", [s["base"] for s in g.noise_slots] if g.noise_slots else [0]))
    t.append(arr_i32("NOISE_SLOT_STRIDE", [s["stride"] for s in g.noise_slots] if g.noise_slots else [0]))
    t.append(arr_i32("COORD_SLOT_TABLE", coord_slots if (coord_ok and coord_slots) else [0]))
    t.append(arr_i32("NORMAL_PACK", pack))
    t.append(arr_f32("NORMAL_PACK_F", [flit(x) for x in pack_f]))
    t.append(arr_f32("NORMAL_AMPS", [flit(x) for x in amps_all]))
    t.append(arr_i32("NORMAL_AMP_OFF", amp_off))
    t.append(arr_i32("OLD_PACK", pack_old))
    t.append("")
    t.append(fold_fn)
    tables_src = "\n".join(t) + "\n"

    # ---- 产物 2：split 生成体（自带 impl 包裹，模块级 include!）----
    s = []
    s.append("// 自动生成（gen_tables_rs.py split 生成体），勿手改。模块级 include!（自带 impl DfcBackend 包裹）。")
    s.append("// split：全树拆分（grid 构建/buildInterpGrid 用）；split_top：角点 0 仅（sample 热路径，整树 1/8）。")
    s.append("impl DfcBackend {")
    s.append("#[allow(unused_variables, unused_assignments, unused_parens, non_snake_case)]")
    s.append("fn split(&self, x: i32, y: i32, z: i32, out: &mut [f32]) {")
    s += split_lines if split_lines else ["        // (空树)"]
    s.append("    }")
    s.append("#[allow(unused_variables, unused_assignments, unused_parens, non_snake_case)]")
    s.append("fn split_top(&self, x: i32, y: i32, z: i32, out: &mut [f32]) {")
    s += split_top_lines if split_top_lines else ["        // (空树)"]
    s.append("    }")
    s.append("}")
    split_src = "\n".join(s) + "\n"

    os.makedirs(OUT_DIR, exist_ok=True)
    open(os.path.join(OUT_DIR, 'dfc_cpu_tables.rs'), 'w', encoding='utf-8').write(tables_src)
    open(os.path.join(OUT_DIR, 'dfc_cpu_split.rs'), 'w', encoding='utf-8').write(split_src)
    print('[OK] dfc_cpu_tables.rs + dfc_cpu_split.rs written')
    print('stats: noise_instances =', len(g.noise_instances), 'split_total =', manifest['split_total'],
          'interp =', len(g.interp_funcs), 'df_nodes =', len(g.df_nodes), 'noise_slots =', len(g.noise_slots),
          'per_sample =', g.per_sample, 'shifts =', len(shift_noises), 'olds =', old_init.count('('))


if __name__ == '__main__':
    main()
