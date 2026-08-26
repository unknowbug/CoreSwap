# Rust 密度树通道化 / 直排架构设计（借鉴 SteelMC，以逐位对齐为准绳）

> 2026-08-24 | 主会话 | 状态：待确认 | 模式档位：重量
> 背景：Rust 密度树已逐位对齐 C++ buildNode；性能瓶颈 = Interpolated grid 构建（~433ms/chunk，5 嵌套 interpolated = 6125 次 arg 采样，深递归树解释）。
> 参考：SteelMC（`.investigations/reference/SteelMC/steelmc-reference.md`）——codegen 转译 + 通道化 cell-corner + ColumnCache + SIMD。
> ⚠️ 原则：**只借 SteelMC 的「求值优化框架」，不借它的「对齐假设」**——对齐以我们自己的 buildNode 语义 + block_probe 逐位为准绳；SIMD 暂缓（浮点重排易破坏 Java 逐位）。

## 1. 目标 / 范围

**目标**：把「每 chunk 5 个独立 interpolated grid + 深递归树解释」改为「**共享通道数组 + 一次 cell-corner 求值 + 外层逐 block 加工**」，砍掉 grid 构建嵌套递归（~433ms → 期望 1 位数 ms/chunk），并**逐位保持 block_probe 对齐**。

**范围（进）**：finalDensity（overworld）的 interpolated 通道化 + 数据驱动直排。
**范围（明确不进）**：SIMD（暂缓）；nether/end；完整块管线（surface/aquifer/ore 另立项）；Beardifier（暂缓，列入后续 vanilla）。

## 2. 关键设计（对齐 vanilla / SteelMC）

### 2.1 通道化 cell-corner（核心）
- **vanilla 语义**：`Interpolated` 标记只包内层函数；内层在 **cell 角点**求值；外层操作（squeeze/min/add/...）在 **三线性插值后逐 block** 应用。
- **改造**：把 finalDensity 树里所有 `Interpolated` 标记的**内层函数**收集为「通道」；每 cell 角点**一次求值所有通道**（`fill_cell_corner`）；存**每 chunk 共享通道网格**；每个 `Interpolated` 标记在求值时被替换为「读通道数组对应索引」。
- **外层**：`combine_interpolated(通道值, x, y, z)` 逐 block 应用外层操作。
- 对照 SteelMC `fill_cell_corner_densities` / `combine_interpolated` / `NoiseChunk`。

### 2.2 数据驱动扁平表 / 显式栈直排（消除深递归）
- 把树摊平为**节点表**（索引引用代替 Box 递归），采样用**显式栈**迭代，非递归 → 消除深递归 + Box 解引用。
- 对照 SteelMC codegen 生成的 flat `compute_*`（我们不做真 codegen，用「数据驱动扁平表 + 显式栈」模拟，语义仍是 match+load，非生成代码）。

### 2.3 ColumnCache（Y-independent 路由器缓存）
- 每 (x,z) 缓存 erosion/continentalness/temperature/vegetation/ridges/preliminary_surface 等 Y-independent 值；`init_grid` 预填 flat-cache。对齐 vanilla `NoiseChunk.FlatCache`。

## 3. 分阶段（每阶段 block_probe 逐位验证）

| 阶段 | 内容 | 验证 |
|---|---|---|
| S0 | 通道分析：写工具识别 finalDensity 所有 `Interpolated` 标记 + 内层 + 所属条目 | 静态分析 + 与 C++ 逐位（当前树不变） |
| S1 | 通道化 cell-corner：共享通道网格 + `fill_cell_corner` + `combine_interpolated` 逐 block | **block_probe 逐位**（S1 前 vs 后，8576/3200 零退化 + chunkgrid 2560/2560） |
| S2 | 数据驱动扁平表 + 显式栈直排 | block_probe 逐位 + 性能对比（期望 grid 构建大降） |
| S3 | ColumnCache（Y-independent 路由） | block_probe 逐位 |
| S4（可选） | SIMD `f64x4`（仅在逐位验证通过才上） | **逐位 bit-exact 验证**（不过则不上） |

## 4. 对齐敏感点（每步 MUST 逐位验证，防 SteelMC 式对齐丢失）
- 三线性插值**浮点顺序**必须与 Java 一致（先插哪个轴、lerp 公式）——我们的 InterpData 已经是（对齐 C++），通道化时保持同一公式。
- cell 边界 / grid 尺寸（5×49×5, CELL_X/Y/Z=4/8/4）不变。
- ColumnCache 边界内/外 fallback 语义（跨 chunk 复用，edgeCol）。
- 外层 ops（squeeze/min/add/blend）**在插值后、且仅与 Y 相关**（vanilla outer ops x/z-independent）——逐 block 应用顺序。
- `interpolated` 通道索引映射（final_density / vein 等顺序）。

## 5. judge / fan-out / knowledge 预置
- **judge**：S1/S2 每个对齐结论 SHOULD 审；收尾交付 MUST（三源核对）。
- **fan-out**：暂无不互斥候选；若通道化与扁平表出现「顺序冲突」再评估。
- **knowledge**：每阶段结论 subagent 产出草稿（错误优先：对齐挖到的错误 → rust-errors.md；性能数据 → rust-rewrite-progress.md；通用模式 → discovered）。

## 6. 风险 & 回退
- 通道化改动了「interpolated 求值时机」（从每节点 lazy grid → 每 chunk 共享通道网格）。**必须逐位验证**：若某处非 vanilla（如 interpolated 与 min/max 交互的求值时机不同），回退到「保留节点原样 + 仅扁平表」（S2 独立）。
- 对齐是铁律；**任何阶段若 block_probe 逐位破坏，立即停**，回退到已对齐版本，不强行。
- 参考 SteelMC 只读；**不照抄其转译输出/对齐假设**。

## 7. 记录
- 本架构：`.investigations/rust-density-builder/rust-density-channel-arch.md`
- 参考：`.investigations/rust-density-builder/steelmc-reference.md`
