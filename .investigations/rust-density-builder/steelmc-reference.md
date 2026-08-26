# SteelMC 参考研究：Rust worldgen 密度函数性能架构（Rust mc server，区块生成 18.8× 原版）

> 2026-08-24 | 主会话 | 来源：`github.com/4lve/SteelMC`（本仓库 clone 到 `.investigations/reference/SteelMC/`，只读参考，不改动）。
> 目的：为 CoreSwap Rust worldgen 的 **Interpolated grid 构建瓶颈（~433ms/chunk，5 嵌套 interpolated，6125 次 arg 采样）** 找方向。
> 结论：**SteelMC 用「codegen 转译密度树 → 原生 Rust + 通道化 cell-corner 批量 + SIMD」绕开了我们的树解释瓶颈，这是已验证的 Rust worldgen 性能路径。**

## 1. 核心架构（steel-worldgen）

### 1.1 转译器（build/density/transpiler/）—— 关键！
- **密度 JSON 树在 build 时转译成本地 Rust 代码**（`compute_*` 函数，`#[inline]`，flat native），**运行时不是解释树**。
- `codegen_functions.rs`：按拓扑序生成 `compute_<name>`（scalar）+ `compute_<name>_4x`（SIMD `f64x4`，4-Y 批量）；`router_*` 入口；`fill_cell_corner_densities`。
- `graph.rs`：`collect_interpolated_inners`（收集所有 `Interpolated` 标记的内层函数）、`is_flat_cached`（Y-independent 识别）、`unwrap_markers`。
- `bounds.rs`：**codegen 时静态求表达式 min/max 边界**（对应我们的 mn/mx，但 build 时分析，非运行时递归）。
- 生成结构体（`codegen_structs.rs`）+ `NoiseChunk`（cell 边界 SoA 角点缓冲，8 通道）。

### 1.2 通道化 interpolated（NoiseChunk / noise_chunk.rs）
- **所有 `Interpolated` 标记各自一个通道**（overworld 8 = 1 地形 + 4 noodle 洞穴 + 3 vein），共享**单一连续通道数组**。
- `fill_cell_corner_densities(noises, cache, x, y, z, blended_noise, &mut out[])` —— **一次调用求出所有通道的内层值**（每个 `out[idx] = <转译 node 表达式>`）。
- `fill_cell_corner_densities_4x(...)` —— **f64x4 SIMD 一次填 4 个 Y 角点**（lane-major SoA）。
- `compute_noise_column(...)` —— SIMD 批整个 Y 列的 blended noise。
- `fill()` 逐 block：**每个 block 对 8 通道做 f64x4 SIMD 三线性插值** → `combine_interpolated(...)` 用转译的外层操作（squeeze/min/add/blend_alpha 等）对插值结果逐 block 加工 → 加 Beardifier → place_block。

### 1.3 ColumnCache（density/traits.rs）
- 每 (x,z) 列缓存 Y-independent 路由器值（erosion/continentalness/temperature/vegetation/ridges/preliminary_surface 等）。
- `ensure()`：列已缓存则 no-op；`init_grid()`：**预填 flat-cache 的 2D 网格**（match vanilla `NoiseChunk.FlatCache`，`(quart_size+1)²`），边界内 O(1) 查，边界外 fallback 现场算。
- `DimensionNoises` 泛型 trait：`router_*` / `fill_cell_corner_densities` / `combine_interpolated` / surface rules。

### 1.4 其它
- **Beardifier 已实现**（`noise/beardifier.rs`，`fill` 里 per-block `density += beard.compute(x,y,z)`，且在 outer ops 之后 —— 对齐 vanilla 语义）。
- `std::simd::f64x4` 便携 SIMD。

## 2. 对我们（grid 构建瓶颈）的启示

| 我们的现状 | SteelMC 做法 | 对我们的价值 |
|---|---|---|
| 每个 Interpolated 节点**各自建 5×49×5 grid**（5 个 = 6125 次 arg 采样/ chunk，且嵌套递归）；树解释（~68μs/arg 采样深递归） | **转译成本地 Rust**（flat，`#[inline]`，无树递归/无虚调用/无指针追逐） | 消除深递归树解释开销 |
| 每次采样推进整棵树（range choice / spline / min/max） | **通道化 cell-corner 批量**：一次调用求所有通道内层值 + **f64x4 SIMD 4-Y 批量** | 已批量、向量化，大幅减调用次数 |
| 5 个独立 grid（嵌套） | **单一共享通道数组**（所有 Interpolated 内层一起求） | 消除「每节点独立 grid + 嵌套」 |
| 每 block 读 grid 做三线性 | **SoA 角点 + f64x4 跨通道三线性 SIMD** | 插值向量化 |
| min/max 运行时递归（已缓存 O(1)，但只是表） | `bounds.rs` **build 时静态分析** min/max | min/max 静态化 |
| Beardifier 缺失 | 已实现 + 语义对齐 | 后续 vanilla 对齐要补 |

**结论**：**codegen 转译（密度 JSON → 原生 Rust）+ 通道化 cell-corner 批量 + SIMD** 是 Rust worldgen 性能的**已验证路径**（SteelMC 18.8×）。C++ 侧 DFC 之所以 CPU 绕圈，是因为它用 **split 预拆分（GPU 设计）**；**codegen 直接生成求值表达式**，不重算 split，是正确的 CPU 版直排。

## 3. 对我们的建议方向（可选）
- 1. **codegen 转译**（重，借鉴 SteelMC）：build 脚本把 worldgen JSON → 原生 Rust `compute_*` 函数 + `fill_cell_corner_densities`/`combine_interpolated`。这是追平 SteelMC 性能的正路，但大工程。
- 2. **轻量改造**（中）：把我们的 enum 树改为「**数据驱动扁平表 + 显式栈直排**」（无递归/无 Box 解引用），并把 5 个 Interpolated 改为**共享通道数组 + 一次 cell-corner 求值**（对齐 SteelMC 的 channel batching），再做 SIMD。收益中等，风险可控。
- 3. **最小改动**（小）：仅把 arg 采样的深递归改显式栈 + 缓存 min/max + 复用，先压一压 433ms（预期有限，治标）。

> SteelMC 的 AGENTS.md 要求「ASK，DON'T GUESS」等；本参考不修改 SteelMC，只读。
