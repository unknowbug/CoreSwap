# 宏观采样组织勘探图（Java/SteelMC「上层」宏观采样 + Interpolated channel 语义）

> recode-scout 勘探产物（只读摸底，不解读/不写结论性 docs）。
> 目标：为「Rust grid 优化 + vulkan 无递归编译」打底，搞清 cell 网格 / multi-channel / Interpolated 语义。
> 状态：scout 摸底（draft）。置信度：代码事实 = reader 级确认；推断处已标注。
> 依据文件：SteelMC `steel-worldgen/src/noise/noise_chunk.rs`、`density/{traits,mod}.rs`、`build/density/`（transpiler）；Rust `WorldgenRust/src/{density,terrain,density_builder}.rs`；`.investigations/perf-rework/`（vulkan DFC 历史）。

---

## 0. 一句话结论（供主会话后续收敛参考，非结论性定论）

- **Java = 单层宏观网格，multi-channel SoA**：把整棵 density 树「竖切」成「被 Interpolated 标的内层函数」（在 cell corners 采样一次）+「外层操作树」（块级对已插值 channel 应用）。每个 Interpolated marker 一个独立 channel，一块一 f64x4 同时插 4 个 channel。**只有一层 cell 网格，无嵌套独立缓存**。
- **Rust 现状 = 每个 InterpolatedDF 独立自持 chunk 网格 + 块级逐点遍历整树**——两者结构不同，且「外层再包宏观网格」与「内部自持缓存」冲突 → 52× 雪崩。
- **雪崩/「无限递归」本质**：都不是真正的无限递归，而是**缓存重建级联**（C++/Rust 的 FlatCache/Interpolated 懒建 chunk 网格，采样点越出当前 chunk 时反复重建邻居网格 → 递归蔓延）与 **GLSL 静态递归检测 / 驱动内联爆炸**（vulkan DFC）。搞清「Interpolated = 独立 channel 竖切，不嵌套」是消除两者的关键。

---

## 1. Java/SteelMC 宏观采样完整流程

### 1.1 总架构（NoiseChunk + build transpiler）

- 数据源：density function 树由 `noise_settings/<dim>.json` 在 **build time** 通过 `build/density/` transpiler 编译成 **native Rust 代码**（`src/generated/vanilla_density_functions/{overworld,nether,end}.rs`），运行时**不解释树**，直接执行生成的 `compute_*` 函数。见 `density/mod.rs` 头注释 + `build.rs`.
- 核心数据结构 `NoiseChunk`（`noise_chunk.rs`）：
  - **cell 尺寸**：XZ = `CELL_WIDTH`（overworld 4），Y = `CELL_HEIGHT`（overworld 8），来自各维 `NoiseSettings::CELL_WIDTH/HEIGHT`。
  - **每 chunk cell 数**：XZ `cell_count_xz = 16/cell_width = 4`；Y `cell_count_y = height/cell_height = 384/8 = 48`；**corners_y = 49**；**z_corners = 5**。
  - **slice 布局**：`slices[cx][corner_idx * MAX_INTERP + ch]`，`cx∈[0..5]`（cell_count_xz+1 = 5 个 X 平面），`corner_idx = z_corner*corners_y + y_corner`，`z_corner∈[0..5]`。每 slice 大小 `MAX_INTERP*MAX_SLICE_LEN = 16*256 = 4096 f64`。**全部 5 个 slice 物化**（不 swap），使 slice-fill 阶段可并行（每 cx 边界独立）。
  - **block_ys**：Y 角点块坐标 `(cy + cell_min_y) * cell_height`，预计算一次，所有 slice 共用。

### 1.2 填 slice（`fill_slice_into`，cell corner 采样）

```
对每个 X 平面 cx (0..=4)：
  block_x = cell_x * cell_width
  对每个 cz (0..=4)：
    cell_z = first_cell_z + cz
    cache.ensure(block_x, block_z)         # 列缓存就绪（flat-cached 值）
    compute_noise_column(...)              # 整 Y 列 SIMD 批 blended noise（overworld BlendedNoise）
    4-Y SIMD 分批 (while cy+4 <= corners_y，即 cy 一次算 4 个 Y 角点)：
      fill_cell_corner_densities_4x(cache, x, ys_v(f64x4), z, blended_v(f64x4), values_4x[..4*interp_count])
      每 lane 写入 slice[corner_idx*MAX_INTERP .. +interp_count]
    标量尾部扫残余 corner
```

### 1.3 cell corners 采样（`fill_cell_corner_densities` / `_4x`）

- 由 transpiler 生成（`codegen_functions.rs` `gen_all_interpolation_functions`）：
  - 对**所有**含 Interpolated 的 router entry（`final_density`, `vein_toggle`, `vein_ridged`）收集**全部 Interpolated marker 的内层函数**（DFS order，`collect_interpolated_inners`，**resolve references through registry**）。
  - 每个内层函数在 `fill_cell_corner_densities` 里占一个 `out[idx]`（= 一个 channel）。
  - `fill_mode = true` 时，Interpolated marker **被透明穿透**（`Marker => gen_expr(&wrapped)`，即直接内联 inner 表达式），不产生 channel 引用——因为当前就是在算 inner。
  - `_4x` 形式：out 布局 lane-major SoA，`out[lane*interp_count + ch]`，per-lane 位等同 4 次标量调用。
  - 关键：**采样的是「内层函数」在 cell corners 的值，不采样整个 final_density 树**。外层操作（squeeze/min/…）不在此处求值。

### 1.4 块级 fill（`NoiseChunk::fill`，三线性插值 + 外层操作）

- 对每个 block `(cell_x_idx, cell_z_idx, x_in_cell, z_in_cell, cell_y_idx, y_in_cell)`：
  - factor_x/z = x_in_cell/cell_width；factor_y = y_in_cell/cell_height。
  - **三线性插值每个 channel**（SoA 布局单 `f64x4` load 4 channel，位等同标量）：
    - 8 角点：`s0=slices[cx]`, `s1=slices[cx+1]`，index 组合 `(z0/z1)`×`(y/y+1)`：
      `n000..n111` → lerp_y → `d00/d10/d01/d11` → lerp_x → `d0/d1` → lerp_z → `result[ch]`。
  - **应用外层操作**（`combine_interpolated`）：传入 `&interpolated[..interp_count]`，x/z=0（外层操作 x/z 无关，仅 YClampedGradient 用 y）。
  - **`interpolated_param_mode = true`** 时，Interpolated marker 变成 `interpolated[idx]`——即**读取已插值 channel 值**。
  - **Beardifier 块级加在 combine 之后**（`add(final_density, Beardifier)` CellCache 语义），不在 corners 采样（避免被 squeeze/线性插值失真）。

### 1.5 key：外层操作树在 combine 里「复用已插值 channel」

- `codegen_expr.rs` Marker 分支（L381-389）：
  - `interpolated_param_mode && kind==Interpolated` → `interpolated[counter++]`（读 channel）
  - 否则 → `gen_expr(&wrapped)`（fill 时内联 inner）
- `codegen_expr.rs` Reference 分支（L419-448）：interpolated_param_mode 遇到含 Interpolated 的 **命名函数**（`interpolated_refs`）→ **inline** 该引用（使外层树复用 channel 而非重算整棵子树）。

---

## 2. Interpolated 的 cell marker 语义（为什么避免递归/雪崩）

### 2.1 vanilla 语义（noise_chunk.rs 头注释 + static-audit-c2me-steel 印证）

> "Vanilla wraps density functions with Interpolated markers. Only the inner functions (arguments to Interpolated) are evaluated at cell corners; the rest (= outer) use trilinear interpolation. Each Interpolated marker gets its own independent channel." + static-audit-c2me-steel: `Wrapping.INTERPOLATED` 节点被替换为 `DensityInterpolator`。

- **不是**「包一层宏观网格对整树采样」，而是**把树竖切**：
  - interp markers 的 **inner** → 在 cell corners 精确采样（高频分量，低采样率但每次全树算）
  - **outer**（squeeze/min/add/…，在 markers 之上）→ 对已插值的 channel 逐 block 应用（低频，便宜）
- **互不嵌套**：`dfc_grid_cache_design.md` §2.2 实证「interp 节点只出现在 finalDensity 树的最外层（顶层闭包），5 个互不嵌套」；interp 的 delegate 不会再是 interp。**这是「只有一层宏观网格」的结构前提**。

### 2.2 channel 分配（`gen_all_interpolation_functions`）

- `entry_names = ["final_density", "vein_toggle", "vein_ridged"]`（顺序固定）。
- 所有 entry 的 Interpolated inner 按 DFS 顺序**连续分配**进单一 `interp_count`（`INTERPOLATED_COUNT` const）与单一 channel 数组。
- 各 entry 的 `combine_*_fn` 各自从自己的 `start` offset 读 channel。
- `const INTERPOLATED_COUNT` = 所有 entry 的 inner 总数。**overworld 8 的来源：final_density 5（1 blend_density 顶层 + 4 cave/noodle）+ vein 3（vein_toggle/vein_ridged 各独立 inner）** = 8。⚠️ vein 3 的具体 inner 数为推断（`MAX_INTERP=16`、`NoiseChunk::interp_count = interpolated_count()`；final_density 5 已由 dfc_grid_cache_design 实证，vein 数推断）。
- `INTERPOLATED_COUNT` 决定 `MAX_INTERP` 需求（≤16）；`interp_count` 用于 slice 布局与 4-channel SIMD 批。

### 2.3 为什么避免雪崩/递归

- **同一 chunk 内**：每个 interp inner 只在**自己的 cell corners**（1225 点/chunk/instance）采样一次，块级全部走三线性插值（O(1) 查网格）。块级**不求值 inner 树** → 不触发嵌套缓存重建。
- **只有一层网格，无嵌套独立 chunk 缓存** → 块级采样永远落在自己 chunk 的 corner 网格内（或 clamp），不会因采样点越界触发内部缓存重建 → **无递归蔓延**。

---

## 3. Rust 现状 vs SteelMC 差异（雪崩根因）

### 3.1 Rust `InterpolatedData`（`WorldgenRust/src/density.rs` L220-315）

- 每个 Interpolated 节点**独立** `build_grid` 自持 chunk 网格缓存（`gx=5, gy=height/8+1=49, gz=5` = 1225 节点），懒建（`sample` miss → `build_grid`）。
- CELL = 4×8×4（overworld），与 Java/SteelMC 相同。
- `sample(pos)`：按 `floor_div(pos.x/z, 16)` 算 chunk key，miss 则 `build_grid(clamped chunk)`，再三线性插值。**越界 clamp 到 gx-2 等**（L294-298）。
- 另有 `Cache2DData`（16 槽 LRU，y 无关）与 `FlatCacheData`（25 格 grid）。

### 3.2 Rust `fill_chunk`（`terrain.rs`）两路径

- **默认逐点**：`for lz/lx/ly` 对每个 block 调 `dense.sample(...)` = `self.df.sample(pos)`，即**逐点遍历整棵 final_density 树**（98304 次）。每次遍历会触达树内 5 个 Interpolated，每个 `sample` 查自己的 chunk 网格（命中则 O(1)）。
  - **慢因 1**：98304 次 × 整树遍历（含 5 interp 的 sample 调用 + spline + noise 多级）——比 Java 的「1225 corners × inner + 98304 次 cheap trilerp」多出大量 inner 树求值路径上的分支/cache 访问。
- **`WG_MACROGRID` 实验路径**（`MacroGrid`，L20-75）：对 chunk cell corners（~1225 点）采样**整个 `final_density` 树**（`dense.sample`），块级三线性插值。
  - **这是错误做法 → 52× 雪崩**（macro_grid_conclusion.txt）：采样 corners 时，坐标覆盖 chunk 边界（x=cx*16+16 即下一 chunk 首列、同理 z/y），内部 5 个 `InterpolatedData::sample` 各自检测 chunk key 变化 → **反复清空重建各自网格**（每次 build_grid = 1225 arg 采样）→ 级联 → 52× 慢。
  - 与「外层再包 Interpolated 2359ms」同因（macro_grid_rootcause.txt）。

### 3.3 Rust vs SteelMC 核心差异总结

| 维度 | SteelMC (Java 对齐) | Rust 现状 |
|---|---|---|
| 宏观网格 | 单层统一，multi-channel SoA（`slices[cx][corner*16+ch]`） | `MacroGrid` 对**整树**采样（实验，默认关） |
| Interpolated 缓存 | **无**自持独立 chunk 网格；靠外层统一网格，块级 trilerp | **每个 interp 独立** chunk 网格懒建缓存 |
| 块级采样 | 对已插值 channel 应用外层 op（`combine_interpolated`） | 逐点遍历整树 `df.sample`（98304 次） |
| 嵌套结构 | 竖切：inner corners + outer combine，无嵌套 | 树内嵌 5 个自持缓存 interp |

**雪崩根因**（candidate，代码 + macro_grid 记录支撑）：Rust 的「自持 chunk 网格缓存」与「外层再采样整树（含这些缓存）」结构叠加：外层 MacroGrid 的 corners 采样点覆盖相邻 chunk → 内部 interp 缓存反复失效重建 → 级联。**正确方向 = 对齐 SteelMC multi-channel：统一一层宏观网格，去掉内部 interp 独立 chunk 缓存的「采样整树」路径（把 interp 当 channel 竖切）。**

---

## 4. vulkan DFC「无限递归」可能根因（perf-rework 历史）

### 4.1 不是「真正的无限递归崩溃」，而是两类不同现象

**A. CPU/C++ 侧：缓存重建级联（无堆栈溢出，是性能雪崩）**
- H2（已定论 / confirmed，`draft-07-block-pipeline-rootcause-confirmed.md` + `fix-design.md`）：
  - **FlatCacheDF 单槽 thread_local 缓存** + `buildGrid` 角点 i=4/j=4 越界（`p.x=(chunkX*4+4)*4=(chunkX+1)*16` = **下一 chunk 首列**）→ 嵌套 spline 的 FlatCache 收到**邻居 chunk key** → 单槽污染 → 重建邻居网格 → **递归蔓延 112 chunk**（rebuild 36,252 = **168×** → spline 20×）。
  - **修复**：当前生成 chunk 上下文绑定（`g_curChunkX/Z`）+ 越界直算不重建 ≠ Java per-chunk 实例语义。**关键教训：per-chunk LRU 不足，必须消除「越界→重建」语义**。
- Rust `InterpolatedData::sample` 雪崩（52×）正是同族机制：**采样点越界 → 懒建缓存反复重建**。

**B. vulkan DFC 编译侧：GLSL/驱动约束（非运行时无限递归）**
- **D4**：GLSL/SPIR-V 链接**静态拒绝递归**（含相互递归）——`spline_eval ↔ spline_val_at` 递归查表被拒。→ 用「显式栈」。**这不是「无限递归」，是「编译拒递归」**。
- **D11**：interp 角点 delegate 用 `eval_df`（含 DF_INTERP 分支，会调 `interp_N`）→ 形成 `eval_df → interp_N → eval_df` **符号环**，GLSL 静态递归检测报 Recursion detected。**修复：拆 `eval_df`（顶层含 interp）/ `eval_df_base`（interp 角点 delegate 用，delegate 无 interp）**。
- **D4/D21/D22**：spline 动态 node 索引 + const 大表 → 驱动内联展开 SPIR-V 17×（D1）/ >10min（D21，spline ~885s）/ SSBO 化后 350s（D22 常量传播）。→ 正确形态 = **数据驱动（每类型一函数 + 数据 buffer）+ 运行时查表破常量传播**，函数数 ≤ ~50。
- **D23**：spline 边界外推遇嵌套 value 直接 0.0（未递归求值 vanilla Spline.apply）→ 大坐标域系统性错值。修复 = 边界嵌套 value 递归求值。

### 4.2 搞清「上层」能否避免

- **能**，两条直接关联：
  1. **Interpolated 是竖切 channel（inner 在 corners 采样，不嵌套独立网格）** → DFC 编译时 interp 就是「8 角点 delegate 采样 + trilerp」，delegate 树无 interp（`dfc_grid_cache_design` §2.2 实证无嵌套）→ **天然规避 D11 递归符号环**（interp 角点用 `eval_df_base` 即可）。把 interp 当独立 channel 竖切，是让 DFC 的 `eval_df_base`（无 DF_INTERP）与顶层 `eval_df`（含）能拆开的正确语义基础。
  2. **单层网格 + corner 去重 + edgeCol 跨 chunk 复用**（production InterpolatedDF 已有）→ **避免「越界→重建」级联**（H2 / Rust 52× 同族）。`dfc_split_optimize_design.md / dfc_grid_cache_design.md` 的「按网格节点生成 split（不按 8 角点每 block 展开）」正是要消除「每点 8 角点 × split-precompute 冗余」。**搞清 channel 语义后，Rust grid 优化应「统一一层网格 + 每 interp 竖切成 channel + 块级 trilerp + 去重角点」。**

---

## 5. grid 构建高效方法（SteelMC SIMD 对标）

### 5.1 SteelMC 高效采样 cell corners

- **SIMD 4-Y 批处理**：`fill_slice_into` 里 `while cy+4 <= corners_y`，用 `f64x4` 打包 4 个 Y 角点 → `fill_cell_corner_densities_4x` 一次算 4 个角点的全部 channel（`values_4x[4*interp_count]` lane-major）。
  - `compute_noise_column`（BlendedNoise，overworld）整体 SIMD 批整列 → `blended_column[cy..cy+4]` 喂给 `_4x`。
  - transpiler 生成 `compute_*_4x`（`gen_expr_simd`）：Noise `get_value_y_simd`、Constant/BlendAlpha/BlendOffset splat、Y-独立 splat；其它 variant 标量 4× 降级（`gen_simd_scalar_fallback`）。per-lane 位等同 4 次标量。
- **slices SoA**（`corner_idx*MAX_INTERP + ch`）让 4 channel contiguous → 块级 trilerp `f64x4::from_slice` 一次 load 4 channel，4 channel SIMD 批插值。
- **ColumnCache grid**：flat-cached xz-only 值预计算 `(16/cell_width+1)² = 5²` 网格（`init_grid`），`ensure` in-bounds O(1) 查表，out-of-bounds 落在重算。

### 5.2 Rust 对标建议（方向性，交主会话）

- 对齐 multi-channel：单层 `slices[cx][corner*16 + ch]`，`interp_count` 从 density 树收集（final_density + vein），4-Y SIMD `fill_cell_corner_densities_4x` + 4-channel `f64x4` trilerp。
- **去掉「采样整树」的 Interpolated 独立 chunk 缓存路径**（`density.rs InterpolatedData::sample` 改为 channel 竖切），消除 52× 雪崩与 H2 类「越界→重建」。
- corner 去重 + edgeCol 跨 chunk 复用（production InterpolatedDF 已有，`dfc_grid_cache_design` 已量化 36% 冗余）。
- 逐位对齐注意：越界 clamp 策略需与 Java `DensityInterpolator` 对齐（SteelMC noise_chunk 保留 clamp；dfc_grid_cache_design §5 记录「是否保留 clamp」为开放项）。

---

## 6. 待深入点清单（主会话/后续 subagent）

1. **vein channel 具体 inner 数与分配**（overworld interp_count=8 精确构成）：需从 overworld.json 的 `vein_toggle`/`vein_ridged` 收集 Interpolated inner 数，确认「5+3=8」。
2. **`compute_noise_column` / BlendedNoise SIMD 批**在 Rust 的实现对标。
3. **Rust 移除 Interpolated 自持缓存 → multi-channel 重构**对现有 逐点 fill、C++ 对齐、BK-001 约束的影响评估（架构变更，交主会话裁决）。
4. **vulkan DFC**：确认 `dfc_gen.py` 现行是否已按「interp 竖切 + eval_df_base 无 DF_INTERP」；若按「每 block 8 角点展开整树」仍是 D11/D1/D21 根源。
5. **越界 clamp 语义**：SteelMC vs Java vs Rust 三者对齐（open item）。

---

## 7. 关键源码引用（勘探依据）

- SteelMC：
  - `steel-worldgen/src/noise/noise_chunk.rs`（slices SoA / fill_slice_into / fill 4-Y SIMD）
  - `steel-worldgen/src/density/traits.rs`（NoiseSettings / ColumnCache / DimensionNoises / fill_cell_corner_densities(_4x) / combine_interpolated / interpolated_count）
  - `steel-worldgen/build/density/transpiler/{codegen_functions,codegen_expr,codegen_structs,analyze,graph}.rs`（channel 分配 / fill vs interpolated_param_mode / ColumnCache grid）
  - `steel-worldgen/build/build.rs`（build-time 编译 → native code）
- Rust 现状：
  - `WorldgenRust/src/density.rs`（InterpolatedData 自持网格 / Cache2D / FlatCache / sample）
  - `WorldgenRust/src/terrain.rs`（fill_chunk 逐点 + MacroGrid 实验 + WG_MACROGRID）
  - `WorldgenRust/src/density_builder.rs`（minecraft:interpolated/node 构建）
- perf-rework（vulkan DFC 历史）：
  - `perf-rework/gpu-accel-errors.md`（D1/D4/D11/D21/D22/D23/H2 递归/雪崩）
  - `perf-rework/dfc_grid_cache_design.md`（interp 5 个无嵌套 / channel 语义 / grid 缓存设计）
  - `perf-rework/dfc_split_optimize_design.md`、`dfc-design.md`、`dfc_cpu_mapping.md`
  - `perf-rework/fix-design.md`、`review-rootcause.md`、`knowledge-drafts/draft-07-block-pipeline-rootcause-confirmed.md`（H2 定论）
  - `perf-rework/static-audit-c2me-steel.md`（Interpolated marker 语义）
- Rust 52× 雪崩直接记录：`.investigations/rust-mod-load/cmd-output/macro_grid_rootcause.txt` + `macro_grid_conclusion.txt`
