# 宏观采样的「顶层性」确认（macro-layer topness）

> recode-scout 勘探产物（只读摸底，不解读/不写结论性 docs）。
> 目标：确认 SteelMC/Java 宏观采样（NoiseChunk cell grid multi-channel）是否真的是采样机制**真正顶层**——
> 之上是否还有影响「采样点数 / 采样方式」的层级，避免主会话挖到一半发现上面还有影响。
> 状态：scout 摸底（draft）。置信度：**reader 级确认**（源码直读）标注为「reader」；跨文件机制归并联立为「reader+推断」。无命令执行，纯只读源码。
> 依据文件：SteelMC `steel-core/src/{worldgen/{stages/noise.rs,generator/{mod,vanilla,context,generation_chunk}.rs},chunk/{chunk_pyramid.rs,chunk_status_tasks.rs}}` + `steel-worldgen/src/{noise/{noise_chunk.rs,aquifer.rs,beardifier.rs},density/{traits.rs,mod.rs}}` + `steel-worldgen/build/density/transpiler/*.rs` + 既有勘探图 `macro-layer-map.md`。

---

## 0. 一句话结论（供主会话后续收敛参考，非结论性定论）

- **宏观采样的最终控制点 = `NoiseChunk::fill` 的 cell grid**（slices SoA + corners 采样 + 块级 trilerp + combine）。其上**没有**任何「采样机制包装」。
- 从 chunk 生成调度到宏观采样，**全部上层都是「调度层」**（Chunk pyramid 状态机 / WorldGenContext 装配 / fill_from_noise 入口），**没有任何一层改变采样点数或采样方式**。
- 六个「疑似上层」（blending、inter-chunk StaticCache2D、Beardifier、aquifer、dimension settings、ColumnCache）**全部确认不构成「宏观采样之上的采样机制层」**：
  - **blending** → 在树的**内部**（transpiler 编译成常量 1.0/0.0 + BlendedNoise 叶在 corners 预采样），非外层包装。
  - **StaticCache2D/ChunkHolder（inter-chunk）** → 调度层，只用于 Beardifier 结构引用解析，不进宏观采样。
  - **Beardifier** → **块级**、在 `combine_interpolated` 之后加（noise_chunk.rs L436-438），非 corners 采样，非上层包装。
  - **aquifer** → **宏观采样之后**的逐 block 消费层（下游），用自己的独立 aquifer cell grid，不重采样地形 density。
  - **dimension noise settings（CELL_WIDTH/HEIGHT）** → 正确来自 N::Settings，是 cell 尺寸的**参数**（不是独立层）。
  - **ColumnCache（5x5 grid）** → 采样**机制内部**的性能缓存（O(1) 缓存 flat 值），同 chunk 内、非跨 chunk，不是采样机制层本身。
- **明确指认：`NoiseChunk::fill`（`.investigations/reference/SteelMC/steel-worldgen/src/noise/noise_chunk.rs` L252 起，配 `fill_slice_into` L148 起的 corners 采样）是宏观采样的最终控制点。** Rust 重构对齐目标应以此层的 cell grid 语义为准，其上无需再找。

---

## 1. 完整调用链（调度层 vs 采样机制层标注）

```
[调度层]  Chunk pyramid 状态机 (GENERATION_PYRAMID, chunk_pyramid.rs L369-421)
            Noise 状态: requirements=[(StructureStarts,8),(Biomes,1)], bswr=0, task=generate_noise
            ↓
[调度层]  ChunkStatusTasks::generate_noise (chunk_status_tasks.rs L62-69)
            → stages::noise::generate(context, step, cache, holder)
            ↓
[调度层]  stages::noise::generate (noise.rs L18-31)
            collect_structure_references(...)          // 从 holder 读 StructureReferences
            build_beardifier(cache, references, x, z)  // 用 StaticCache2D 解析引用 chunk 的 StructureStarts → Beardifier
            ↓
[调度层]  context.generator.fill_from_noise(GenerationChunk<NoisePhase>::acquire(&holder), beardifier)
            (GenerationChunk<NoisePhase> 是类型级 phase marker = 容器，generation_chunk.rs L12/L119-120，无采样)
            ↓
[入口]    VanillaGenerator::fill_from_noise (vanilla.rs L399-503)
            NoiseChunk::<N>::new(chunk_min_x, chunk_min_z)        // 建 cell grid 对象
            N::ColumnCache::default() + column_cache.init_grid(...) // 建 per-chunk flat 缓存
            Aquifer::<N>::new(..., column_cache.clone())           // 建 aquifer（带独立缓存）
            noise_chunk.fill(noises, &mut column_cache, beardifier, place_block闭包)
            ↓
[采样机制层·顶层]  NoiseChunk::fill (noise_chunk.rs L252-457)
            ├─ 预填 5 个 slice（每个 cell-X 边界平面）:
            │     fill_slice_into (L148-236):
            │       for cz in 0..=4:
            │          cache.ensure(block_x, block_z, noises)      // ColumnCache O(1)
            │          noises.compute_noise_column(...)            // BlendedNoise 整列 SIMD 批
            │          noises.fill_cell_corner_densities[_4x]     // 树的内层函数在 cell corners 采样 → slices[]
            │
            └─ 块级循环: for 每 block:
                   trilerp 每 channel (8 corners, SoA f64x4)      // L343-410
                   noises.combine_interpolated(cache, &interpolated, ...) // 外层 op (L416-422)
                   beardifier.compute(world_x,y,z) 加进 density  // L436-438 (块级, 非 corners)
                   place_block(local_x,y,z, density, &interpolated, cache)  // → 闭包内 aquifer 消费
```

**调度层 vs 采样机制层**：
- **调度层**（改变不了采样点数/方式）：Chunk pyramid → ChunkStatusTasks → stages::noise → fill_from_noise 装配。这些只决定「在哪个状态、哪个 chunk、给什么 beardifier/缓存句柄」。
- **采样机制层**：**仅有** `NoiseChunk::fill`（含 `fill_slice_into` 的 corners 采样 + 块级 trilerp + combine）。`fill_cell_corner_densities`/`combine_interpolated`/`compute_noise_column` 是被 fill 调用的**机制内部函数**（树求值），不构成独立的更高层。

---

## 2. 宏观采样的最终控制点（明确指认）

**最终控制点 = `NoiseChunk::fill` 的 cell grid**。理由：

- **cell 尺寸 / 每 chunk cell 数 / 角点数 / channel 数全部固化于 `NoiseChunk::new` + `fill`**：
  - `cell_width/height = N::Settings::CELL_WIDTH/HEIGHT`（L85-86，来自维度 settings → 见 §3.5）。
  - `cell_count_xz = 16/cell_width`，`cell_count_y = height/cell_height`，`corners_y = cell_count_y+1`（L94-96）。
  - `interp_count = N::interpolated_count()`（L100，来自密度树中 Interpolated marker 数，overworld=8）。
  - `slices[cx][corner*MAX_INTERP + ch]` SoA 布局（L39-50）。
- **corner 采样点**：`fill_slice_into` 内 `fill_cell_corner_densities[_4x]`（L197/L220）是机制的最深采样动作——对**每个 Interpolated marker 的内层函数**在 `(x, y, z)` corner 求值一次，写入 channel。
- **块级采样点**：L343-410 的 trilerp + L416 combine（外层 op），每 block 用 8 个已插值 corner 得密度（O(1)，不求值内层树）。这是「采样点数」= 1225 corners（overworld）由哪一层决定的唯一控制层。
- 其上没有再包一层「interpolated / 宏观二次采样 / per-chunk 预计算缓存链」对整树做二次 cell 采样（contrast：Rust 现 `MacroGrid` 正是错在「再包一层对整树采样」→ 52× 雪崩，见 macro-layer-map §3.2）。

---

## 3. 每个「疑似上层」的确认（是否影响采样点数/方式，在哪层）

### 3.1 Blending（旧世界过渡平滑）— 确认：**不是上层包装，在树的内部** ✅

- 证据（reader）：
  - density tree 中有 `BlendAlpha` / `BlendOffset` / `BlendedNoise`(`old_blended_noise`) 三种「blend」。
  - **`BlendAlpha → 常量 1.0`、`BlendOffset → 常量 0.0`**（transpiler `codegen_expr.rs` L370-371 fill 与 L676-677 SIMD 均如此）——即 Steel 把这两个 blend marker **编译成编译期常量，不采样**（旧世界过渡 blend 未实现为真实采样）。
  - **`BlendedNoise`**：`codegen_expr.rs` L346-352 —— fill_mode 时读预计算 `blended_noise_value`（来自 `compute_noise_column` 整列 SIMD），非 fill_mode（combine）时 `noises.blended_noise.compute(x,y,z)`。它是树的**叶函数**，在 corners 预采样（compute_noise_column），非外层包装。
  - `combine_interpolated` 头注释（noise_chunk.rs L413-415）明确把 `blend_alpha`/`blend_offset` 归为块级「外层操作」之一（x/z 无关）。
- 结论：blending 完全在 `NoiseChunk::fill` 的树求值**内部**，作为 combine 外层 op / corners 叶采样。**不是宏观采样之上的层级**，不改变 cell grid 的采样点数或方式。

### 3.2 column_cache / StaticCache2D / ChunkHolder（inter-chunk 缓存共享）— 确认：**调度层，不进宏观采样** ✅

- 证据（reader）：
  - `StaticCache2D<Arc<ChunkHolder>>` 在 NOISE stage 仅用于 `build_beardifier`：读取引用 chunk 的 `chunk.structure_starts()`（noise.rs L72-104），解析出 structure start → Beardifier。**不把邻居 chunk 的密度/噪声喂给宏观采样。**
  - `fill_from_noise` 里宏观采样用的 `column_cache` 是**新建的 per-chunk 实例**（`N::ColumnCache::default()` + `init_grid(chunk_min_x, chunk_min_z, noises)`，vanilla.rs L414-415），不跨 chunk 共享采样结果。
- 结论：inter-chunk 的 ChunkHolder/StaticCache2D 层不改变宏观采样机制；宏观采样**不跨 chunk 读缓存做地形采样**（仅 Beardifier 读结构引用）。

### 3.3 Beardifier（结构密度修正）— 确认：**块级、combine 之后加，非 sampled at corners** ✅

- 证据（reader + macro-layer-map §1.4）：
  - `NoiseChunk::fill` 在 `combine_interpolated`（外层 op）之后、调 `place_block` 之前：`density += beard.compute(world_x, world_y, world_z)`（noise_chunk.rs L436-438）。
  - 注释明确（L424-429）：vanilla 以 `add(final_density, Beardifier)` 包在 `cacheAllInCell` 里、块级求值；若在 corners 采样会进 squeeze 并线性插值失真。
- 结论：Beardifier 是**块级每点密度修正**（消费 density），既不在 corners 采样，也不构成宏观采样层。它改变的是**每个 block 的最终密度值**，但不改变「宏观采样点数/方式」。

### 3.4 aquifer — 确认：**宏观采样之后的逐 block 消费层（下游），有自己的独立 cell grid** ✅

- 证据（reader）：
  - `fill_from_noise` 的 place_block 闭包内：`aquifer.compute_substance(noises, world_x, world_y, world_z, density)`（vanilla.rs L459）。`density` 是 `NoiseChunk::fill` 已插值+combine 的输出。
  - `compute_substance`（aquifer.rs L552-779）：**非 solid 时才进 aquifer 逻辑**；用「2×3×2 邻居的 aquifer cell centers + 最近 4 个」做**独立 cell 最近邻采样**，sampling 独立于 NoiseChunk cell grid。
  - 它用 `self.col_cache`（克隆 ColumnCache）+ `status_cache`/`location_cache`（aquifer.rs L150-180, L611-616）+ 采样 router_barrier/fluid_level/lava（`calculate_pressure`/`compute_fluid`）。
  - 注释（vanilla.rs L426-428）：「Aquifer samples at arbitrary (x,z) outside the chunk, so it needs its own cache」——它有自己的独立采样域和缓存。
- 结论：aquifer 是**宏观采样地形 density 之后**的一个**下游独立采样子系统**（决定是水/空气/固体），**不重采样地形的 cell grid**，不构成宏观采样之上的包装。它的 cell grid 是 aquifer 专属（液面），与正在重构的「宏观密度采样」不是一个机制。

### 3.5 dimension noise settings（CELL_WIDTH/HEIGHT）— 确认：**参数，不是独立层，来源正确** ✅

- 证据（reader）：`NoiseChunk::new` 从 `N::Settings::CELL_WIDTH / CELL_HEIGHT / MIN_Y / HEIGHT` 取 cell 尺寸（noise_chunk.rs L85-88），来自维度 settings 生成的编译期常量（`NoiseSettings` trait，density/traits.rs L19-42）。cell 尺寸正确来自维度，是机制**参数**而非外层包装。

### 3.6 ColumnCache（5x5 grid）— 确认：**采样机制内部的性能缓存，非采样机制层** ✅

- 证据（reader）：`ColumnCache` trait 定义（density/traits.rs L47-64）：`ensure` 对列 O(1)（in-bounds）/ 越界 raw 坐标 on-the-fly；`init_grid` 预计算 `(quart_size+1)²` 网格，对齐 vanilla `NoiseChunk.FlatCache`。它在 `fill_slice_into` 中被 `cache.ensure` 喂 corners 采样的 flat 值（noise_chunk.rs L175），也被 `compute_noise_column`/`fill_cell_corner_densities`/`combine_interpolated` 读。**这是性能缓存**（缓存 xz-only 的 Y 无关值），是采样机制内部件，不是决定「采样点数/方式」的采样层。macro-layer-map §5.1 已对齐 Java ColumnCache 5x5 网格（init_grid 5²=25）。

---

## 4. 是否还有比 NoiseChunk cell grid **更高**的「采样机制包装」？— **无** ✅

- **无再包一层 interpolated**：`Interpolated` marker 在 fill 时被**透明穿透**（`codegen_expr.rs` Marker 分支 L381-389 fill_mode → `gen_expr(&wrapped)`），不产生 channel 引用；只在 `interpolated_param_mode`(combine) 读 `interpolated[idx]`。interp 是「竖切 channel」，非「网格套网格」。
- **无二次采样链**：`combine_*_fn` 直接用已插值 channel 数组，不再对整树重采样（macro-layer-map §1.5）。
- **无 per-chunk 预计算采样缓存链**：slices 一次性物化，每 block 走 trilerp，无「先采样一版粗 → 再采样一版细」的链（对比 Rust 现 `MacroGrid` 错误路径，§1 末尾）。
- **`FillTopSurface`**：`codegen_expr.rs` L391-417，是 FindTopSurface 的上边界/下边界逻辑，在 combine 里求值（块级对 density 树），**不是宏观采样包装**（它在 fill_slice 内层函数里也可能出现，但它是树内 DF，不是 cell 网格之上的层）。
- `BlendDensity`（L380）：inlines 其 input（缓存标记），非层。

---

## 5. 结论汇总表

| 层级 | 在调用链中的位置 | 类别 | 是否影响「采样点数/方式」 |
|---|---|---|---|
| Chunk pyramid 状态机 (Noise status) | 最上 | 调度层 | 否 |
| ChunkStatusTasks::generate_noise | 上 | 调度层 | 否 |
| stages::noise::generate (+Beardifier 构建) | 上 | 调度层（+块级修正数据装配） | 否（Beardifier 数据本身块级加） |
| WorldGenContext / VanillaGenerator | 上 | 装配/配置 | 否 |
| GenerationChunk::<NoisePhase> | 上 | 类型级 phase marker | 否 |
| **VanillaGenerator::fill_from_noise** | 入口 | 装配（建 NoiseChunk/ColumnCache/Aquifer 并调用 fill） | 否（只是入口，机制在 fill） |
| **NoiseChunk::fill (cell grid)** | **采样机制·顶层** | **采样机制层** | **是（最终控制点）** |
| ├ column_cache (5x5) | fill 内部 | 性能缓存 | 否（缓存已插值/flat 值） |
| ├ fill_cell_corner_densities[_4x] | fill 内部 | 树求值@corners | 机制内部 |
| ├ trilerp + combine_interpolated | fill 内部 | 块级插值+外层 op | 机制内部 |
| └ Beardifier.compute | fill 内部(块级) | 块级修正 | 否（改密度值，不改采样点/方式） |
| Aquifer.compute_substance | **fill 之后**(下游) | 下游独立采样子系统 | 否（自己的液面 cell grid，不重采样地形） |
| blending (BlendAlpha/Offset/BlendedNoise) | 树内部 | 树 op/叶 | 否（编译期常量 + corners 叶） |
| dimension settings (CELL_W/H) | NoiseChunk::new 参数 | 参数 | 否（cell 尺寸来源，非层） |
| StaticCache2D / ChunkHolder (inter-chunk) | 调度层 | 调度 | 否（仅 Beardifier 引用解析） |

**最终控制点：`NoiseChunk::fill` 的 cell grid**（= macro-layer-map §1.2-1.4 描述的 slices SoA + corners 采样 + 块级 trilerp + combine）。Rust 重构对齐应以此为「宏观采样顶层语义」，其上均为调度/装配。

---

## 6. 置信度标注

- **reader 级确认（源码直读）**：调度链（pyramid → tasks → noise stage → fill_from_noise）、`NoiseChunk::new/fill/fill_slice_into` cell 尺寸与结构、`BlendAlpha→1.0/BlendOffset→0.0`、`Beardifier` 块级位置、aquifer 下游消费 + 独立 cell、ColumnCache per-chunk、GenerationChunk phase marker、WorldGenContext 纯装配。
- **reader+推断（跨文件机制归并联立，但均有源码落点）**：interp「竖切无二次包装」语义（macro-layer-map §2.1-2.3 已实证互不嵌套）、aquifer cell grid 与噪声管理的同层性判断（aquifer 采 router_*，机制归属清晰）。
- 未执行命令，纯只读源码；「steel 未实现真实旧世界过渡 blend（BlendAlpha/Offset 编译为常量）」为 reader 观察到的事实，但**是否影响对齐**（Rust 是否需补）不在本勘探范围，交主会话裁决。

---

## 7. 关键源码引用（勘探依据）

- 调度链：
  - `steel-core/src/chunk/chunk_pyramid.rs`（GENERATION_PYRAMID Noise 状态 L385-389）
  - `steel-core/src/chunk/chunk_status_tasks.rs`（generate_noise L62-69）
  - `steel-core/src/worldgen/stages/noise.rs`（generate / build_beardifier L18-113）
  - `steel-core/src/worldgen/generator/context.rs`（WorldGenContext 纯装配 L42-135）
  - `steel-core/src/worldgen/generator/generation_chunk.rs`（NoisePhase marker L12/L119-120）
- 采样机制：
  - `steel-worldgen/src/noise/noise_chunk.rs`（new/fill/fill_slice_into，cell grid 顶层）
  - `steel-worldgen/src/density/traits.rs`（NoiseSettings/ColumnCache/DimensionNoises/fill_cell_corner_densities/combine_interpolated）
  - `steel-worldgen/build/density/transpiler/codegen_expr.rs`（Marker/Reference/BlendAlpha/BlendOffset/BlendedNoise 分支）
- 疑似上层：
  - `steel-worldgen/src/noise/beardifier.rs`（compute L221）
  - `steel-worldgen/src/noise/aquifer.rs`（compute_substance L552+, 独立 cell 采样）
  - `steel-worldgen/src/noise/blended_noise.rs`（compute_column / compute 叶）
  - `steel-worldgen/src/noise/ore_veinifier.rs`（compute_interpolated L100，用 interpolated channel）
- 既有勘探图：`.investigations/macro-layer-scout/macro-layer-map.md`
