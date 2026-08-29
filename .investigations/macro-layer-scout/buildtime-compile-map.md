# SteelMC build-time 编译 transpiler 完整机制地图

> 勘探对象：SteelMC `steel-worldgen` 的 build-time density transpiler（把 density 树 JSON 编译成 native Rust 代码）。
> 目的：为 CoreSwap Rust（WorldgenRust）的 build-time 编译 density 树打底，替代当前「运行时 enum match 解释」（3677 节点）。
> 角色：recode-scout（只读勘探，只摸底不解读）。产物只写 `.investigations/`。
> 置信度标注：`[reader 确认]` = 直接读源码确认；`[推断]` = 从源码结构合理推断，未逐行验证。

---

## 0. 一句话结论

SteelMC 的 transpiler 是 **build-time 的「specialized 函数生成器」**：build.rs 在编译期读 datapack JSON（density_function/*.json + noise_settings/*.json），用 `proc_macro2`/`quote` 把 density 树**逐节点内联展开成 Rust 源码**（每个节点类型 → 一段内联表达式，每个命名函数 → 一个 `compute_*` 函数，每个 router 入口 → 一个 `router_*` 函数），写到 `src/generated/`，运行时通过 `#[path]` 模块 + trait 直接调用。**不是数据驱动（节点数组 + 解释器），而是真正的代码生成（specialized 内联）**。这与 CoreSwap 当前「enum match 递归解释」是两种完全不同的运行时形态。

---

## 1. build-time 编译完整流程（build.rs → transpiler → 生成 → 接线）

### 1.1 入口：`build/build.rs` `[reader 确认]`

```
build.rs main()
├─ println!("cargo:rerun-if-changed=build/")          # 整个 build/ 目录变化触发重编
├─ out_dir = CARGO_MANIFEST_DIR/src/generated/        # 生成代码写到 src/generated/
├─ 生成 2 个静态数据文件（非 density）：
│   ├─ multi_noise::build() → vanilla_multi_noise.rs
│   └─ noise_parameters::build() → vanilla_noise_parameters.rs
├─ density::build() → DensityFunctionFiles { overworld, nether, end, index }
│   └─ 写到 src/generated/vanilla_density_functions/{overworld,nether,end}.rs + mod.rs
└─ 若 feature "fmt" 开启 → rustfmt 格式化生成文件
```

关键点 `[reader 确认]`：
- **生成代码写到 `src/generated/`（源码树内，非 OUT_DIR）**，通过 `#[path = "generated/..."]` 直接作为 crate 模块编译。
- **增量写入**：`if fs::read_to_string(&path).is_ok_and(|existing| existing == content) { continue; }` —— 内容不变就不重写（避免触发下游重编）。
- **rerun-if-changed**：`build/` 目录 + 每个 density JSON 文件（`functions.rs` 里 `println!("cargo:rerun-if-changed={}", file.display())`）。

### 1.2 数据读取：`build/density/functions.rs` `[reader 确认]`

```
read_density_function_registry()
├─ DATAPACK_BASE = "../steel-utils/build_assets/builtin_datapacks/minecraft/worldgen"
├─ 递归收集 density_function/*.json（collect_json_files）
├─ path_to_id: .../density_function/overworld/continents.json → "minecraft:overworld/continents"
└─ serde_json 解析成 DensityFunctionJson（untagged: Constant(f64) | Reference(String) | Data(typed)）

read_noise_settings(dimension) → NoiseSettingsJson（含 noise_router + surface_rule + noise config）
```

JSON → 内部 `DensityFunction` 树（`json_to_df` / `json_data_to_df`）：
- **noise 不 bake**（`noise: None`，运行时从 seed 建）
- **reference 不 resolve**（`resolved: None`，transpiler 通过 registry 处理）
- 每个 JSON `"type"` 字段 → 一个 `DensityFunction` enum variant（见 §2.1 映射表）

### 1.3 每维度 transpile：`transpile_dimension()` `[reader 确认]`

```
transpile_dimension(dimension, prefix, registry)
├─ settings = read_noise_settings(dimension)
├─ router_entries = router_to_entries(settings.noise_router)   # 14 个 router 入口
├─ cell_width = settings.noise.size_horizontal * 4
├─ TranspilerInput { registry, router_entries, prefix, cell_width, legacy_random_source }
└─ transpile(&input) → TokenStream
```

三个维度：`overworld`/`nether`/`end`，prefix 分别为 `Overworld`/`Nether`/`End`。

### 1.4 运行时接线：`src/lib.rs` `[reader 确认]`

```rust
#[expect(warnings)]
#[rustfmt::skip]
#[path = "generated/vanilla_density_functions/mod.rs"]
pub mod density_functions;
```

生成代码通过 `#[path]` 属性作为 `density_functions` 模块编译进 crate。生成代码内部 `use steel_worldgen::density::...`（`extern crate self as steel_worldgen` 自引用），所以生成代码能调用 crate 的运行时支持（`spline_eval`、`NormalNoise`、`RarityValueMapper` 等）。

### 1.5 完整数据流图 `[reader 确认]`

```
datapack JSON (density_function/*.json + noise_settings/*.json)
  │  build.rs (cargo build script)
  ▼
serde 解析 → DensityFunctionJson → DensityFunction 树（noise 未 bake / ref 未 resolve）
  │  transpile()（proc_macro2 TokenStream 生成）
  ▼
Rust 源码字符串（specialized 内联函数）
  │  写到 src/generated/vanilla_density_functions/*.rs
  ▼
cargo 编译（#[path] 模块）→ native 机器码
  │  运行时
  ▼
DimensionNoises trait 的 router_* 函数（直接调用，无树解释）
```

---

## 2. codegen 机制（density 树 → Rust 代码）

### 2.1 节点类型映射表 `[reader 确认]`

| JSON type | DensityFunction variant | 生成代码形态（gen_expr） |
|---|---|---|
| `minecraft:constant` | Constant | 内联 f64 字面量 |
| `minecraft:y_clamped_gradient` | YClampedGradient | `map_clamped(y, from_y, to_y, from_v, to_v)` |
| `minecraft:noise` | Noise | `noises.n_<id>.get_value(x*xz, y*ys, z*xz)`（flat 时 `get_value_xz`） |
| `minecraft:shifted_noise` | ShiftedNoise | 先算 dx/dy/dz 再 `get_value(x*xz+dx, ...)` |
| `minecraft:shift_a` | ShiftA | `get_value_xz(x*0.25, z*0.25) * 4.0` |
| `minecraft:shift_b` | ShiftB | `get_value_xy(z*0.25, x*0.25) * 4.0` |
| `minecraft:shift` | Shift | `get_value(x*0.25, y*0.25, z*0.25) * 4.0` |
| `minecraft:add/mul/min/max` | TwoArgumentSimple | `((a)+(b))` / `((a)*(b))` / min/max 带静态边界短路 |
| `minecraft:abs/square/cube/half_negative/quarter_negative/invert/squeeze` | Mapped | 内联一元表达式（如 `{let v=..; v*v}`） |
| `minecraft:clamp` | Clamp | `clamp(inner, min, max)` |
| `minecraft:range_choice` | RangeChoice | `let v = input; if cond { in } else { out }`（带死分支消除） |
| `minecraft:interval_select` | IntervalSelect | 嵌套 `if v < t { f } else { ... }` 链 |
| `minecraft:spline` | Spline | 内联 if/else 区间链（Hermite，见 §4） |
| `minecraft:old_blended_noise` | BlendedNoise | `noises.blended_noise.compute(x,y,z)`（fill 模式用参数） |
| `minecraft:weird_scaled_sampler` | WeirdScaledSampler | `scale * noise.get_value(x/scale,...).abs()` |
| `minecraft:end_islands` | EndIslands | `noises.end_islands.sample(x,y,z)` |
| `minecraft:blend_alpha/offset/density` | BlendAlpha/Offset/Density | `1.0` / `0.0` / 递归 input |
| `minecraft:interpolated` | Marker(Interpolated) | 见 §3 |
| `minecraft:flat_cache/cache_2d/cache_once/cache_all_in_cell` | Marker(FlatCache/Cache2D/...) | 见 §3 |
| `minecraft:find_top_surface` | FindTopSurface | 内联 while 循环（Y 扫描） |
| `minecraft:beardifier` | Constant(0.0) | 占位（结构未生成时正确） |
| 裸数字 | Constant | 内联字面量 |
| 裸字符串 | Reference | 见 §2.3 |

### 2.2 生成形态：specialized 内联函数（非数据驱动）`[reader 确认]`

**核心结论：生成的是「specialized 函数」，每个节点内联展开，不是「节点数组 + 解释器」。**

生成代码结构（`transpiler/mod.rs` 的 `transpile()`）：
```
use std::simd::f64x4; ...（imports）
{Prefix}Noises struct          # 每个用到的 noise 一个 NormalNoise 字段
{Prefix}Noises impl           # create(seed, splitter, params) 构造器
{Prefix}ColumnCache struct    # flat-cached 值 + grid 数组
compute_* 函数（拓扑序）       # 每个命名 density 函数一个 #[inline] fn
compute_*_4x 函数（SIMD）      # 每个非 flat 命名函数一个 4-Y 批量版
router_* 函数                  # 每个 router 入口一个 pub fn
fill_cell_corner_densities / combine_interpolated 等插值函数
```

关键机制 `[reader 确认]`：
- **每个命名函数 → 一个 `compute_<sanitized_name>` 函数**，函数体是 `gen_expr` 递归内联展开的表达式（无递归调用，无 enum match）。
- **Reference 节点 → 函数调用**：`compute_<name>(noises, cache, x, y, z)`（3D）或 `cache.df_<name>`（flat-cached 读缓存）。
- **拓扑排序**（`analyze.rs::topological_sort`）：依赖先定义，保证函数定义顺序正确。
- **CSE（公共子表达式消除）**：`fingerprint.rs` 对 `Reference`/`Noise`/`ShiftedNoise` 做结构哈希，相同子树 hoist 成 `let` 绑定复用（`cse_bindings`）。
- **静态边界分析**（`bounds.rs::compute_bounds`）：对 min/max/range_choice 做区间算术，消除死分支（如 `min(a,b)` 当 `a <= b_lo` 时直接返回 a，跳过 b 求值）。这是 C2ME `MaxShortNode`/`MinShortNode` 的移植。
- **SIMD 双路径**：`gen_expr`（标量）+ `gen_expr_simd`（`f64x4` 4-Y 批量）。SIMD 路径对 Constant/Noise/YClampedGradient/Mapped/Clamp/TwoArgumentSimple 等做真 SIMD，其余 fallback 到标量 4× 展开。

### 2.3 Reference 处理 `[reader 确认]`

- Reference 节点在 codegen 时**不内联**（除非特殊模式），而是生成对 `compute_<name>` 的调用。
- 三种情况（`gen_expr` 的 Reference arm）：
  1. `interpolated_param_mode` 且 ref 含 Interpolated → 内联（让 `interpolated[i]` 替换生效）
  2. `fill_mode` 且 ref 含 BlendedNoise → 内联（用预计算 `blended_noise_value`）
  3. `flat_cached` → 读 `cache.df_<name>`（列缓存）
  4. 否则 → `compute_<name>(noises, cache, x, y, z)` 函数调用

---

## 3. interpolated / channel 处理（竖切）

### 3.1 Interpolated marker 语义 `[reader 确认]`

`minecraft:interpolated` → `Marker(MarkerType::Interpolated)`，是**优化提示**：标记「这个子树的值在 cell corners 采样一次，块内三线性插值」，避免每点重算。

### 3.2 竖切（channel）机制 `[reader 确认]`

transpiler 把含 Interpolated 的 router 入口（`final_density`/`vein_toggle`/`vein_ridged`）**竖切成 channel 数组**：

```
gen_all_interpolation_functions()（codegen_functions.rs）
├─ Phase 1: collect_interpolated_inners() 收集所有 Interpolated 的 inner（DFS 序）
│   └─ 所有 entry 共享一个连续 channel 数组，索引按 final_density → vein_toggle → vein_ridged 顺序
├─ Phase 2: fill_cell_corner_densities() —— 在 cell corner 采样每个 channel inner
│   └─ out[i] = <inner_i 的内联表达式>（fill_mode=true，BlendedNoise 用参数）
├─ Phase 2b: fill_cell_corner_densities_4x() —— SIMD 4-Y 批量版
├─ Phase 3: combine_interpolated() —— 块级对已插值 channel 应用外层操作
│   └─ interpolated_param_mode=true，Interpolated marker → interpolated[i] 参数引用
└─ Phase 4: combine_vein_toggle() / combine_vein_ridged()
```

**channel 在生成代码里的表示** `[reader 确认]`：
- `INTERPOLATED_COUNT` 常量 = 总 channel 数。
- `fill_cell_corner_densities(noises, cache, x, y, z, blended_noise_value, out)` 填 `out[0..INTERPOLATED_COUNT]`。
- `combine_interpolated(noises, cache, interpolated, x, y, z)` 里，`interpolated_param_mode` 下 `Interpolated` marker 生成 `interpolated[idx]`（`idx` 是 `interpolated_param_counter` 递增分配的 channel 索引）。
- 含 Interpolated 的 Reference 在 param 模式下**内联**（`interpolated_refs` 集合），让 marker 的 `interpolated[i]` 替换穿透函数边界。

### 3.3 与 CoreSwap 的对应 `[reader 确认]`

CoreSwap `density.rs` 已有等价实现：`macrolize_channels()` 做同样的竖切（`Interpolated` → `ReadChannel{ch}`），`sample_combine()` 读 `interp[ch]`。**语义已对齐**，只是 CoreSwap 是运行时 enum match 解释，SteelMC 是 build-time 内联。

---

## 4. spline / noise 生成

### 4.1 Spline（Hermite）`[reader 确认]`

**生成形态：内联 if/else 区间链，非数据驱动查表。**

`gen_spline_expr()`（codegen_expr.rs）：
- 每个 spline 点 → 一个 `if __coord < L_i { ... } else if ...` 区间 arm。
- 区间内 Hermite 插值**内联展开**，操作顺序与 `spline_eval::hermite_interpolate` 逐位一致（保证 vanilla 确定性）：
  ```
  __t = (__coord - li) / (li1 - li)
  __h = li1 - li
  __a = di * __h - (y2 - y1)
  __b = -di1 * __h + (y2 - y1)
  __lerp_y = y1 + __t * (y2 - y1)
  __lerp_ab = __a + __t * (__b - __a)
  result = __lerp_y + __t * (1 - __t) * __lerp_ab
  ```
- **嵌套 spline**（`SplineValue::Spline`）→ 生成独立 `spline_helper_N` 函数（`gen_spline_helper`），区间 arm 里调用。
- 空 spline → `0.0`；单点 spline → 退化外推。
- 对比：运行时 `spline_eval.rs` 的 `evaluate_spline` 用**二分查找 + 闭包 value_at**（数据驱动），但 transpiler **不用它**，而是内联 if/else 链（C2ME `SplineAstNode` 风格），避免二分查找和闭包间接。

### 4.2 Noise（Perlin）`[reader 确认]`

**生成形态：数据驱动（运行时 NormalNoise 对象），非内联。**

- 每个用到的 noise → `{Prefix}Noises` struct 的一个 `NormalNoise` 字段（`n_<sanitized_id>`）。
- 生成代码调用 `noises.n_<id>.get_value(x, y, z)` / `get_value_xz(x, z)` / `get_value_y_simd(...)`。
- **noise 本身不内联**（Perlin 算法在 `NormalNoise` 运行时对象里），transpiler 只生成「调用哪个 noise 字段 + scale 参数」。
- noise 参数（first_octave + amplitudes）从 `noise_parameters` 生成文件（`vanilla_noise_parameters.rs`）来，运行时 `create()` 用 seed + splitter 构造。
- **legacy_random_source**（下界）：temperature/vegetation 用 `LegacyRandom(seed)` + `create_legacy_nether_biome(-7, [1.0,1.0])`，BlendedNoise 用 `LegacyRandom(seed)` 而非 positional splitter（`codegen_structs.rs` 的 `gen_noises_impl`）。

### 4.3 BlendedNoise（old_blended_noise）`[reader 确认]`

- 每维度最多一个 `blended_noise` 字段（`blended_noise_config`）。
- 生成代码 `noises.blended_noise.compute(x, y, z)`。
- **fill 模式**（`fill_cell_corner_densities`）下，BlendedNoise 生成 `blended_noise_value` 参数引用（预计算一次，避免每 corner 重算）。
- 含 BlendedNoise 的 Reference 在 fill 模式内联（`blended_noise_refs`）。

---

## 5. Rust 对应可行性（CoreSwap WorldgenRust）

### 5.1 现状 `[reader 确认]`

- CoreSwap `WorldgenRust` 是**运行时解释**：`density.rs` 的 `DensityFunction` enum + `sample_ctx()` 递归 match（3677 节点）。
- 数据驱动边界（`data-driven-boundary.md`）：density 树从 `density_function/*.json` **运行时加载**（`DensityBuilder::set_external_loader`），noise settings 从 `noise_settings/*.json` 运行时读。
- **无 build.rs**（`Cargo.toml` 只有 `[package]`/`[lib]`/`[dependencies]`，无 `[build-dependencies]`，无 build.rs 文件）。
- 依赖极简：`Cargo.toml` 的 `[dependencies]` 为空（自研 json 解析 `crate::json`，无 serde/proc_macro2/quote）。

### 5.2 可行性结论 `[推断]`

**可行，但需要引入 build-time 依赖 + 重构数据加载边界。** 具体：

1. **codegen 框架**：需引入 `proc-macro2` + `quote`（build-dependencies）+ `serde`/`serde_json`（或复用自研 `crate::json`）。SteelMC 用 `proc_macro2::TokenStream` 生成源码字符串；CoreSwap 可同样做，或更简单：**直接生成 Rust 源码字符串**（`format!` 拼接），不一定要 proc_macro2（proc_macro2 只是方便，本质是拼字符串）。

2. **生成代码形态**：可完全照搬 SteelMC 的「specialized 内联函数」形态——每个命名 density 函数 → `compute_*` 函数，每个 router 入口 → `router_*` 函数，Interpolated 竖切成 channel 数组。CoreSwap 的 `macrolize_channels` 已对齐竖切语义，可直接映射。

3. **编译集成**：加 `build.rs`，读 `density_function/*.json` + `noise_settings/*.json`，生成 `src/generated/*.rs`，`lib.rs` 用 `#[path]` 或 `include!` 接线。**关键冲突**：CoreSwap 当前是**运行时加载 JSON**（数据驱动边界），build-time 编译意味着**把 JSON 数据 bake 进二进制**，跨版本需重新编译（而非换 JSON 文件）。这与 `data-driven-boundary.md` 的「跨版本换数据文件即可」原则**冲突**，需用户裁决（见 §6.3）。

4. **noise 处理**：CoreSwap 的 `DoublePerlinNoiseSampler`/`OctavePerlinNoiseSampler` 已是运行时对象，可直接作为 `{Prefix}Noises` struct 字段（同 SteelMC 的 `NormalNoise`）。noise 参数表（`build_noise_params` 硬编码 + `noise_params.json`）可生成 `vanilla_noise_parameters.rs`。

5. **spline 处理**：CoreSwap `SplineData` 是扁平表（`nodes`/`locations`/`derivatives`/`sub_idx`），运行时 `sample_node` 二分查找。build-time 可改为内联 if/else 链（同 SteelMC），或保留扁平表数据驱动（生成静态数组 + 二分查找函数）。**内联 if/else 链更快**（无二分查找、无递归），但生成代码更大。

### 5.3 需要什么（清单）`[推断]`

| 项 | 说明 |
|---|---|
| build.rs | 读 JSON → 生成 Rust 源码 → 写 src/generated/ |
| build-dependencies | proc-macro2 + quote（或纯 format! 拼字符串）+ serde/serde_json（或复用 crate::json） |
| 生成代码形态 | specialized 内联函数（compute_* / router_* / fill/combine） |
| 运行时支持 | NormalNoise 等价物（已有 DoublePerlinNoiseSampler）、spline_eval 等价物（已有 SplineData） |
| 编译集成 | lib.rs `#[path]` 或 `include!` 接线 |
| 数据边界重构 | 从「运行时加载 JSON」改为「build-time bake JSON」（需用户裁决） |

---

## 6. 关键难点

### 6.1 codegen 复杂度 `[reader 确认]`

SteelMC transpiler 规模（build/density/ 目录）：
- `types.rs`（811 行）：DensityFunction enum + 22 个 variant + resolve 逻辑
- `functions.rs`（1144 行）：JSON 解析 + JSON→DF 转换 + 每维度 transpile + noise settings 生成
- `transpiler/` 8 个文件：analyze（图分析/拓扑排序/flatness 推断）、graph（树遍历）、fingerprint（结构哈希/CSE）、bounds（静态边界）、naming（标识符）、codegen_expr（标量+SIMD 表达式生成，1142+ 行）、codegen_functions（命名函数+router+插值函数）、codegen_structs（Noises/ColumnCache struct）
- `surface_rules.rs`（483 行）：surface rule 也 transpile

**总规模约 4000+ 行 build-time 代码**。这是完整移植的主要成本。但 CoreSwap 已有等价运行时逻辑（density.rs 702 行 + density_builder.rs 415 行 + spline.rs），可复用语义，只需重写「生成」层。

### 6.2 生成代码规模 + 编译时间 `[推断]`

- 生成代码是**逐节点内联展开**，3677 节点的 density 树展开成内联表达式，生成代码规模可能**数万到数十万行**（每个命名函数 + 每个 router + SIMD 双份 + 插值函数）。
- 编译时间：内联展开 + SIMD 双路径会显著增加编译时间（SteelMC 用 nightly + `portable_simd`）。CoreSwap 当前无 SIMD 依赖，若只做标量版可减半。
- **缓解**：SteelMC 用 CSE（结构哈希去重）+ 静态边界（死分支消除）控制展开规模；CoreSwap 可同样做。

### 6.3 与现有 density.rs（运行时解释）的关系 + 数据驱动边界冲突 `[reader 确认 + 推断]`

- **核心冲突**：CoreSwap 的架构铁律是「**数据驱动**」（`data-driven-boundary.md` + AGENTS.md「数据驱动架构铁律」）——density 树从 JSON **运行时加载**，跨版本换 JSON 文件即可。build-time 编译把 JSON **bake 进二进制**，跨版本需**重新编译**（换 JSON 后 cargo 重编）。
- 这是**方向性冲突**，需用户裁决：
  - 方案 A：build-time 编译（性能优先，牺牲「换 JSON 免重编」）
  - 方案 B：保留运行时加载 + 运行时 JIT/缓存优化（数据驱动优先）
  - 方案 C：混合——build-time 编译 vanilla 维度（overworld/nether/end），mod 维度运行时解释（SteelMC 实际就是 build-time 编译 vanilla，mod 维度走 registry 运行时）
- **注意**：SteelMC 的 AGENTS.md 明确「**Vanilla extracted registry/worldgen data should be compiled by build scripts into typed Rust data, not parsed from JSON at runtime**」——SteelMC 明确选择了 build-time 编译（放弃运行时 JSON 加载）。CoreSwap 的「数据驱动」铁律与之**相反**，这是两个项目的根本架构分歧，移植前必须明确。

### 6.4 其他难点 `[reader 确认 + 推断]`

1. **确定性对齐**：生成代码必须与运行时解释**逐位一致**（SteelMC 反复强调「bit-identical to spline_eval」「vanilla determinism preserved」）。CoreSwap 已有逐位对齐 C++ 的验证链（block_probe），build-time 版需重新验证逐位一致。
2. **SIMD**：SteelMC 用 nightly `portable_simd`（`f64x4`）。CoreSwap 若不做 SIMD，可只做标量版（性能提升仍显著，因消除 enum match 递归 + 虚调用）。
3. **noise 参数表**：CoreSwap `build_noise_params` 硬编码 + `noise_params.json` 双源，build-time 需统一。
4. **多世界**：CoreSwap 已参数化维度（`create_for_dim`），build-time 编译需为每个维度生成独立代码（SteelMC 就是 overworld/nether/end 三份），mod 维度需运行时 fallback。
5. **surface rule**：SteelMC 也 transpile surface rule（`surface_rules.rs`）。CoreSwap 的 surface rule 是代码规则（overworld）+ JSON 数据驱动（其他维度），build-time 化是独立课题。

---

## 7. 分层总结

| 层 | 机制 | 置信度 |
|---|---|---|
| build.rs 流程 | 读 JSON → transpile → 写 src/generated/ → #[path] 接线 | reader 确认 |
| codegen 形态 | specialized 内联函数（非数据驱动） | reader 确认 |
| 节点映射 | 22 个 variant → 内联表达式 | reader 确认 |
| Reference | 函数调用 / 缓存读 / 特殊模式内联 | reader 确认 |
| Interpolated/channel | 竖切成 channel 数组 + interpolated[i] | reader 确认 |
| Spline | 内联 if/else 区间链（Hermite） | reader 确认 |
| Noise | 数据驱动（运行时 NormalNoise 对象） | reader 确认 |
| CSE/边界分析 | 结构哈希去重 + 静态区间死分支消除 | reader 确认 |
| SIMD | f64x4 双路径（标量 + SIMD） | reader 确认 |
| Rust 对应可行性 | 可行，需 build.rs + 依赖 + 数据边界重构 | 推断 |
| 数据驱动冲突 | build-time bake vs 运行时加载，需用户裁决 | reader 确认（冲突存在）+ 推断（裁决方向） |

---

## 附：关键文件索引

| 文件 | 作用 |
|---|---|
| `build/build.rs` | 入口：调 transpiler，写 src/generated/ |
| `build/density/mod.rs` | 模块声明 + re-export |
| `build/density/types.rs` | DensityFunction enum + 22 variant + resolve |
| `build/density/functions.rs` | JSON 解析 + JSON→DF + 每维度 transpile + noise settings |
| `build/density/surface_rules.rs` | surface rule transpile |
| `build/density/transpiler/mod.rs` | transpile() 主流程 + TranspilerInput |
| `build/density/transpiler/analyze.rs` | 图分析 + 拓扑排序 + flatness 推断 |
| `build/density/transpiler/graph.rs` | 树遍历（uses_y / collect_references / interpolated 检测） |
| `build/density/transpiler/fingerprint.rs` | 结构哈希 + CSE 候选 |
| `build/density/transpiler/bounds.rs` | 静态边界分析（死分支消除） |
| `build/density/transpiler/naming.rs` | ID → Rust 标识符 |
| `build/density/transpiler/codegen_expr.rs` | 标量 + SIMD 表达式生成（核心） |
| `build/density/transpiler/codegen_functions.rs` | 命名函数 + router + 插值函数生成 |
| `build/density/transpiler/codegen_structs.rs` | Noises/ColumnCache struct 生成 |
| `build/density/transpiler/context.rs` | transpile 状态 |
| `src/density/traits.rs` | DimensionNoises/ColumnCache/NoiseSettings trait |
| `src/density/spline_eval.rs` | 运行时 spline 参考（生成代码对齐目标） |
| `src/lib.rs` | #[path] 接线生成代码 |
