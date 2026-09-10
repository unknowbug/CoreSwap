# scout 勘探笔记 — vanilla 跨 chunk 写语义 vs Rust `pending_cross_writes` 语义面比对

> 角色：recode-scout（只读勘探，隔离执行）。日期标签沿用父任务目录 `perf-closeout-260910-05`
> （实际日期锚 `record.md:3`：Get-Date 2026-09-10 18:45；scout 被禁跑命令，未自行取时间）。
> 一手源：`versions/1.21.6/data/mc_src_extract/`（1.21.6 Java 源码 extract）+ `worldgen-core/src/`（Rust）
> + `runtime/1.21.6/java/`（mod 侧）+ `knowledge/discovered/`（既有结论）。
> 置信度标注只用三档：**confirmed-by-source**（源码可逐行核到）/ **推断**（源码级推理，运行时后果未实测）/ **未核到**。
> 本笔记只写 `.investigations/`，不改任何代码/知识库；不含需要跑命令才能定的数据（见末节待验清单）。

---

## §1 vanilla 特性放置写到邻 chunk 时是否真持久化？

### 1.1 「读邻 chunk」路径：不触发同步生成，缺状态=崩 —— confirmed-by-source

- `ChunkRegion.getChunk(x, z)`（`ChunkRegion.java:105-107`）默认降级到 4 参版；4 参版
  `getChunk(chunkX, chunkZ, leastStatus, create)`（`:109-142`）**完全忽略 `create` 参数**（函数体内无任何引用），
  因此不存在「读时按需生成/加载」通道。
- 邻 chunk 来源 = `this.chunks.get(chunkX, chunkZ)`（`:116`），即 `BoundedRegionArray<AbstractChunkHolder>`；
  该容器在**构造时一次性**用 getter 填满窗口（`BoundedRegionArray.java:20-32`），越界直接
  `IllegalArgumentException`（`:40-46`）。
- 可用性判定（`ChunkRegion.java:112-122`）：`i = centerPos 到 (cx,cz) 的 Chebyshev 距离`；
  `chunkStatus = directDependencies.get(i)`（`i >= size → null`）；命中则 `getUncheckedOrNull(chunkStatus)`。
  **未命中/状态不足 → 组装 CrashReport 抛 `CrashException("Requested chunk unavailable during world generation")`**
  （`:127-141`）——是硬崩，不是静默降级、也不是同步补齐。
- 依赖状态由 FEATURES 步骤定义：`dependsOn(STRUCTURE_STARTS, 8).dependsOn(CARVERS, 1)`
  （`ChunkGenerationSteps.java:23-29`）⇒ 距离 0/1 的邻 chunk 需 ≥ CARVERS，2..8 需 ≥ STRUCTURE_STARTS。
  调度器在步骤启动前保证这些状态在位，读的时候直接拿到**共享的同一 chunk 对象**。

### 1.2 「写邻 chunk」路径：允许 + 直写 + 立即持久（带半径门）—— confirmed-by-source

三条写入口都落到目标 chunk 对象本身：

1. **半径门**：`ChunkRegion.isValidForSetBlock`（`:242-271`）用 `generationStep.blockStateWriteRadius()`。
   FEATURES 步骤的该值为 **1**（`ChunkGenerationSteps.java:27`；字段定义 `ChunkGenerationStep.java:15`）
   ⇒ |Δcx| ≤ 1 且 |Δcz| ≤ 1（3×3）允许。
   **超出半径**：`Util.logErrorOrPause(...)` + 返回 false（`:258-268`）——即 **「error 日志 + 丢弃」，既不抛异常也不静默**。
   `logErrorOrPause` 本体 = `LOGGER.error` + 仅 dev 下 `pause()`（`Util.java:792-797`），
   而 `pause()` 只是触发 missing-breakpoint handler（`Util.java:841-848`），**不抛异常**；无调试器时等价于「日志 + 继续」。
2. **`ChunkRegion.setBlockState`**（`:274-310`）：门通过后 `chunk = this.getChunk(pos)`（同一共享对象，`:278`）→
   `chunk.setBlockState(pos, state, flags)`（`:279`）→ **直接写进目标 chunk（跨 chunk 时就是邻 chunk 对象），立即可见、立即持久**，中间没有任何缓冲。
   注意：该方法**只要 `isValidForSetBlock` 通过就返回 true**（`:280-308` 只用返回的旧 state 做 `onBlockStateChanged`，
   不检查目标是否真的接受了写）——这为 1.3 的「静默吞写」埋了伏笔。
3. **OreFeature 的 section 级写**（`OreFeature.java:110/137-151`）：
   `world.isValidForSetBlock(mutable)`（`:137`）→ `chunkSectionCache.getSection(mutable)`（`:138`）→
   `chunkSection.setBlockState(ad, ae, af, target.state, false)`（`:147`，lock=false 走 `swapUnsafe`）。
   `ChunkSectionCache.getSection`（`ChunkSectionCache.java:26-44`）内部 `world.getChunk(sectionCoord(x), sectionCoord(z))`
   拿的是**同一共享 chunk 对象**，取 `chunk.getSection(i)` 后 `chunkSection.lock()`（`:34`），
   `close()` 时统一 `unlock()`（`:58-62`，try-with-resources 见 `OreFeature.java:110` / `:163`）。
   ⇒ Ore 的跨 chunk 写同样是**直写邻 chunk 的真实 section + 立即持久**，且额外有 **per-section 锁**（并发保护面）。
   读侧同源：`shouldPlace` → `isExposedToAir(posToState, pos)`（`OreFeature.java:174`）→
   `Feature.isExposedToAir`（`Feature.java:181-183`）→ `testAdjacentStates` 遍历 **`Direction.values()` 全 6 邻**
   （`Feature.java:168-179`）→ 经 `chunkSectionCache::getBlockState`（`OreFeature.java:146`，`ChunkSectionCache.java:46-56`）
   ⇒ **Ore 的越界读也走真实邻 chunk section**（不是快照）。

### 1.3 持久性的例外：vanilla 自己也有「写被吞」窗口（目标已 FULL）—— 主结论 confirmed-by-source，后效 推断

- chunk 转 FULL 时 `convertToFullChunk` 执行 `abstractChunkHolder.replaceWith(new WrapperProtoChunk(worldChunk, false))`
  （`ChunkGenerating.java:169-197`，替换点在 `:186`；`propagateToWrapped = false`）。
- `WrapperProtoChunk.setBlockState` 在 `propagateToWrapped == false` 时 **直接 `return null`，不写任何东西**
  （`WrapperProtoChunk.java:74-78`）；`getSection` 也回落到 `super.getSection(yIndex)`（`:69-72`），
  而 `super` 走 `ProtoChunk` 构造 → `Chunk` 构造 `sections = null` → `fillSectionArray` 填**全新的空 section**
  （`Chunk.java:92-118` + `:120-126`；`WrapperProtoChunk.java:41-51` 未传 sections）⇒ section 级读会返回空（air）。
- `AbstractChunkHolder.replaceWith` 把 **索引 0 .. length-2（即 EMPTY..LIGHT/SPAWN）的所有 future** 都换成这个只读 wrapper
  （`AbstractChunkHolder.java:95-110`）；`getUncheckedOrNull(CARVERS)`（`:267-272`）因此对**已 FULL 的邻 chunk**
  返回该 wrapper。
- ⇒ 结论：**vanilla 的写持久化条件 = 「写时刻目标 chunk 仍是可变 ProtoChunk」**。
  目标已 FULL 时：`ChunkRegion.setBlockState` 路径的写被静默吞掉（`:279` 返回值 null，但 `:308` 仍返回 true）；
  section 级路径（Ore 的 `isExposedToAir`/`setBlockState`）读到的是空 section（**推断**：运行时后果未实测，
  机制来自 `WrapperProtoChunk.java:69-78` + `Chunk.java:104/117-126` 的构造链）。
- 与 Rust 的关键差别不在「有没有丢失窗口」，而在**窗口位置**与**可见性**：
  vanilla 的窗口在 FULL 之后（FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL 之后，
  `ChunkGenerationSteps.java:30-33`），且在 ProtoChunk 阶段写入**立即可见**；
  Rust 的窗口在「目标已跑过 apply_features」即开始，且写入在此前**完全不可见**（见 §2）。

### 1.4 ChunkHolder 的角色 —— confirmed-by-source

`server/world/ChunkHolder.java` 不带 worldgen 写路径：它管 level / future / 待发送 block-update 集合
（`ChunkHolder.java:33-80`，含 `blockUpdatesBySection`、`levelIncreaseFuture` 等），
worldgen 期的读写语义全部由 `ChunkRegion` + chunk 对象（ProtoChunk / WrapperProtoChunk）承载。
即：**没有任何 worldgen 跨 chunk 写要经过 ChunkHolder**，「写邻 chunk 触发同步生成/加载」在本版本不存在。

---

## §2 vanilla 是否有与 Rust `pending_cross_writes` 对应的延迟写缓冲语义？

### 2.1 vanilla：没有块写缓冲 —— confirmed-by-source

`ChunkRegion` 是 worldgen 期唯一写入口，写即直写（§1.2）。仓库内能被误认为「延迟」的结构都不是块写缓冲：

| 结构 | 位置 | 延迟的是什么 |
|---|---|---|
| `MultiTickScheduler<Block>/<Fluid>` | `ChunkRegion.java:70-71` | 计划刻（tick），块本身已写 |
| `markBlockForPostProcessing` | `ChunkRegion.java:304-314` | 后处理**标记**，块本身已写 |
| `addPendingBlockEntityNbt`（DUMMY） | `ChunkRegion.java:293-299` | 方块实体的 **NBT**，块本身已写 |
| `ChunkSectionCache` | `ChunkSectionCache.java:16-44` | 只是 section 引用缓存 + 锁（`getSection` 返回**真实 section**），`OreFeature.java:147` 的写是直写 → 立刻对同一 cache 的后续 `getBlockState` 可见（`:46-56`） |
| `ChunkCache`（非 worldgen 用） | `world/chunk/ChunkCache.java` | 非本路径 |

⇒ **vanilla 语义 = write-through + 真实时间序的 last-write-wins**：写立即进目标 chunk、立即对后续读可见，
不存在「等目标进入某阶段才生效」的机制。

### 2.2 Rust `pending_cross_writes` = deferred + 可能永久丢失 —— confirmed-by-source（结构），后果 推断

Rust 侧全貌：

- 字段：`pending_cross_writes: Mutex<HashMap<(i32,i32), Vec<(usize, BlockId)>>>`（`worldgen_handle.rs:144`，初始化 `:506`）。
- 生产：越界写回调 `pending_cross_cb`（`:1086-1091`）→ `pc.entry((tcx,tcz)).or_default().push((idx, state))`；
  由 `OreFeatureContext::set_block`（`feature.rs:208-222`）在**任意**越界坐标触发（`:215-221`）。
- 消费：**只在目标 chunk 自己 `apply_features` 的开头**（`:1050-1063`，`if ca_min` 内）
  `pc.remove(&(cx,cz))` 后逐条覆写本地 `col`。
- 写入对读不可见：`apply_features` 内越界读走 3×3 预取快照 `ca_snapshot`（`:1120-1128`）→
  `neighbor_terrain`（`:999-1022`）= **无 feature 的地形列**，且不变量显式声明「本 chunk features 的写入不可见于越界读」
  （`:1115-1119`）。即：A 写进 B 的 buffered 方块，在 A 自己的后续越界读里看不到，在 B 的 overlay 之前也看不到。
- Rust 自认的残余差：`worldgen_handle.rs:142-144` 注释 `IDK-cA2：先于 center 生成的邻 chunk 收不到 overlay`。

### 2.3 判定：语义分歧（lossy），不是等价实现

**不等价的四个可观测面**（前三个 推断，第 4 个 confirmed-by-source 结构）：

1. **丢失（drop）**：目标 chunk 已跑过 apply_features ⇒ 条目永不被 remove、也永不生效（`:1053-1062` 是唯一消费点）；
   旧同步 inflight=1 形态下顺序确定、DAG 依赖保证目标后跑，故不暴露；异步 inflight=23 后顺序不定 ⇒ 变成内容敏感（父会话假说的机制面成立）。
   注意还有第二条 drop 通道：`ca_min=false` 时 `pending_cross` 回调是 `None`（`worldgen_handle.rs:1283`），越界写**直接丢弃**（`feature.rs:215-221` 无 else 分支）。
2. **可见性（visibility）**：vanilla 写入立即可见；Rust 在目标 overlay 前不可见（§2.2 末）⇒ 会改变 feature 的放置判定
   （如 `isAir` 类谓词、`would_survive`、树冠 `can_replace`）。
3. **顺序反转（order inversion）**：若真实时间序是「目标自己的 features 先写、源 chunk 后写」（vanilla 里源写覆盖目标写），
   Rust 在目标 features **之前** overlay，等于把源写提前 ⇒ last-write-wins 反了（Rust 注释 `:1052` 自称「Java 写持久语义」，实际只覆盖了「源先写」一支）。
4. **残留（leak / 陈旧注入）**：条目只在目标 apply_features 时 `remove`（`:1054`）⇒ 从不重跑的目标坐标条目**常驻**
   （内存增长），且在同一 handle 内**重新生成该坐标**（重复 region sweep / 同 JVM 二次 bench）时会被迟到 overlay
   ⇒ 注入陈旧方块（跨批污染）。`api.rs` 导出的全部 `wg_*`（`:59-225`）**没有任何跨写导出/查询接口**，即该缓冲无外部清空/观测面。

**等价条件（很窄）**：仅当 ① 目标 chunk 尚未跑 apply_features；② 该目标在本批或更晚完成；③ 批次内没有更晚的写翻转先后；
④ 缓冲不跨批次残留 —— 四者同时成立时行为才近似 vanilla。默认出货语义（mask=3）下该路径整体不执行（§3）。

**另有一处半径语义分歧**：vanilla 超半径（>1 chunk）**拒写 + logErrorOrPause**（`ChunkRegion.java:242-271` + `Util.java:792-797`）；
Rust 无半径判定，任意越界 chunk 都入 buffer（`feature.rs:208-222`）⇒ 若有越半径写，vanilla 拒 / Rust 收（双分歧）。

**对照参考（历史载体）**：C++ 侧当年的复刻走「两阶段 + 区域列存储 + 串行 phase2」，
`knowledge/discovered/algorithm-fingerprints.md:179-201`（发现 #8）+ `versions/1.20.1/cpp/worldgen/src/block_probe.cpp:243-246`
+ `versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp:1343-1349`（phase2 强制 `threads=1`）——
那条路径 pendingCross 是写进 **regionCols 区域列缓存**（即最终输出列），因此不丢；Rust 的 per-chunk 返回式 API（§3）不具备该结构。

---

## §3 `stageMask=3` 下 Rust fill 路径还会不会触碰邻 chunk 的写？

### 3.1 静态链复核：结论 = 结构不可达 —— confirmed-by-source（支持父会话结论）

- 默认 mask：`CppBridge.resolveStageMask()` 无 sysprop ⇒ `0b011`（`runtime/1.21.6/java/src/main/java/wg/bench/CppBridge.java:66-74`）；
  三处句柄（overworld/nether/end）都 `setFlags(handle, resolveStageMask())`（`:82` / `:128` / `:148`）。
  行为化就地证据（父会话已采）：`record.md:19` —— 两 R3 臂日志均 `flags=3 skip_features=true skip_carver=true skip_surface=false`。
- Rust 消费点：`FLAG_SKIP_FEATURES = 1 << 1`（`worldgen_handle.rs:148-150`）；
  `let skip_features = flags & FLAG_SKIP_FEATURES != 0 || env WG_SKIP_FEATURES; if !skip_features { self.apply_features(...) }`
  （`:669-671`）。mask=3 ⇒ 不进 `apply_features`。
- `pending_cross_writes` 全仓仅 4 处引用：`:144`（声明）/ `:506`（初始化）/ `:1053`（**消费**，在 `apply_features` 体内）/
  `:1088`（**生产**，`pending_cross_cb` 定义，同在 `apply_features` 体内，函数范围 `:1036-1375`）。
  生产闭包只被塞进本地 `octx.pending_cross`（`:1283`）；全仓 `pending_cross:` 赋值点只有两处：
  `worldgen_handle.rs:1283`（`ca_min` 门控）与 `bin-diag/b5b6_smoke.rs:126`（显式 `None`）。
- `apply_features` 唯一调用点 = `:671`（`fn apply_features` 非 pub，`:1036`，无其他调用者）；
  `fill_chunk_blocks` 的**生产**调用者唯一 = `api.rs:130-171 wg_fill_blocks_multi`（`:156`），
  其余调用者全是 `bin/` / `bin-diag/` 诊断程序（不参与 JNI 出货路径）。
- `neighbor_terrain`（`:999-1022`）只有两个调用点，都在 `apply_features` 内（`:1124`、`:1242`）⇒ mask=3 下同样不可达。

⇒ **默认出货语义下 `pending_cross_writes` 结构不可达（生产点、消费点、连邻域读快照预取都不执行）**，
同样不可达的还有 `ca_snapshot`/`neighbor_terrain` 整条 c-A-min 路径。

### 3.2 除 apply_features 外是否还有任何写邻 chunk 的出口？—— 没有；`neighbor_terrain` 是「读 + 缓存回填」

- `neighbor_terrain` 语义 = **读 + 本地 memo 回填**：命中返回 `e.col.clone()`（`:1008`，Arc clone）；
  未命中 `fill_terrain_column`（`:1012`）后把 `CaTerrainEntry{Arc<BlockColumn>}` 塞进 `terrain_cache`（`:1014-1020`）。
  **回填的是内部 memo，不是任何 chunk 输出列**；且 `fill_chunk_blocks` 命中缓存时是 `(*e.col).clone()`（`:624`）
  再在其上跑 features ⇒ 缓存条目永不被 feature 写改写（`Arc` 无可变访问，无别名写）。
- carver 路径（mask=3 时也不跑）只写本地列：`carve_region_impl` 把 r/t 用 `.max(0)` / `.min(15)` 夹到本 chunk
  （`carver.rs:374-381`），写的是 `col`（`:397`、`:426`）；邻 chunk 只用于起始条件/中心判定（`:368-371`）。
  这与 Java CARVERS 的 `blockStateWriteRadius(0)`（`ChunkGenerationSteps.java:22`）一致。
- mask=3 时 fill 主路径（`fill_chunk_blocks:613-678` → `fill_terrain_column:683-837`）内**唯一的共享可变状态写入**
  是内部 memo upsert：`terrain_cache`（`:631-635`）；`est_l2` 是 aquifer 阶段的精确值缓存（`aquifer.rs:250-288`，FIFO 淘汰）；
  `beardifiers` 只读当前 chunk 自己的条目（`:710`）。
- `build_surface` 只吃本地 `col` + biome + initial density（`:820-829`），无邻 chunk 读；`ore_vein.apply` 是按坐标的纯函数（`:742`）。
- JNI 出口面：`api.rs` 导出的全部函数（`:59-225`）里没有任何「跨写收集/应用」入口，也没有 `wg_fill_blocks_multi_phase`
  （C++ 曾有，Rust 无）⇒ 即使缓冲里有内容，**也没有第二条投递通道**。

⇒ **支持父会话静态结论**。唯一能把该路径激活的外部条件：`-Dcoreswap.rust.stages=1`（bit1 清 ⇒ mask=1，
`rustFeaturesTakeover()` 为 true，见 `CppBridge.java:167-170`）、`-Dcoreswap.rust.stages=all|0`（双跑对照臂，
`rustFeaturesTakeover()` 显式排除 mask=0 ⇒ Rust 跑 features 且 Java 也跑，语义第三态）、或 native bin-diag（flags=0，不设 flags）。

---

## §4 若未来重新开启 Rust features 接管（bit1 清零），还有哪些顺序敏感面？

### A. 内容敏感（改造前 MUST 设会计或保序）

| # | 面 | 依据 | 敏感类型 |
|---|---|---|---|
| A1 | **跨 chunk 写投递**（drop / leak / 顺序反转 / 陈旧注入） | `worldgen_handle.rs:142-144`（IDK-cA2 自认）、`:1050-1063`、`:1086-1091`；`api.rs:130-171` 是 per-chunk 返回式（数组交出后不可再改） | 顺序敏感（inflight>1 时非确定）；架构级 |
| A2 | **越界读可见性**：快照 = 无 feature 地形，永不接受任何跨 chunk 写 | `:1115-1119`（不变量）、`:1120-1128`、`:1240-1244` | 相对 vanilla 内容敏感，但 Rust 内确定性（顺序无关） |
| A3 | **越界高度图读哨兵**：越界返回 `min_y-1` / 直接放行 | `feature.rs:196`、`:204`；`placement.rs:419`、`:450`、`:478`（后两处已自标「邻域——保留（登记）」）；Java 对照点 `ChunkRegion.java:406-408`（读邻 chunk 真实 WG heightmap）。可达性：`OreFeature.java:44-50` / `feature.rs:247-253` 的 s/t 扫描必然跨出本 chunk | 顺序无关，内容敏感 |
| A4 | **目标 chunk 高度图被跨 chunk 写更新**：Java `ProtoChunk.setBlockState` 会刷新**目标 chunk** 的 heightmaps；Rust 快照 heightmap 只在 miss 时算一次、永不被写更新 | `ProtoChunk.java:143-155` vs `worldgen_handle.rs:1014-1020` | 顺序无关，内容敏感 |
| A5 | **写半径语义**：vanilla 仅 ≤1、超半径拒+logErrorOrPause；Rust 无半径判定 | `ChunkGenerationSteps.java:27` + `ChunkRegion.java:242-271` vs `feature.rs:215-221` | 顺序无关，内容敏感 |
| A6 | **两阶段/区域列存储缺位**：无区域共享列 ⇒「后写覆盖」只在同批+目标未跑 features 时近似；若改区域批次，批内 feature 迭代序（Java = chunk 序 + 每 chunk 内 step/p 升序）必须一并定义 | `ChunkGenerator.java:333-422`（尤其 `:359-413`）vs `worldgen_handle.rs:1175-1196`（step/p 序已对齐）；C++ 历史形态 `algorithm-fingerprints.md:179-201` | 顺序敏感；架构级 |
| A7 | **3×3 biome 并集近似**（已知 IDK-1）：Rust 用 y∈{0,64,128} 三切片采样，Java 读 3×3 chunk 的全部 biome container 值 | `worldgen_handle.rs:1144-1156` vs `ChunkGenerator.java:344-352` | 顺序无关，内容敏感（重启 features 前需另立课题） |

### B. 内容中性（只影响命中率/成本/内存，不改内容）

| # | 面 | 依据 |
|---|---|---|
| B1 | `terrain_cache` 容量/clear-all 淘汰 | `:1028-1030`（cap 2048）、`:1016-1019`、`:631-635`；命中值 = 重算值（`:618` 声明、`:157-158` 同源 Arc） |
| B2 | `est_l2` FIFO 淘汰 | `aquifer.rs:248-288`（`:248` 注释明示「淘汰只影响命中率不影响正确性，重算同值」；`:277-282` FIFO 环） |
| B3 | `pending_cross_writes` 的 HashMap 迭代序 | 从不迭代（只有 `remove`（`:1054`）与 `entry.push`（`:1089`））⇒ 无顺序面；但**清空时机**属 A1 |
| B4 | `beardifiers` per-chunk map | `:115-116`、`:710`（只读自身键；fill 前由 Java `feedBeardifier` 写入，`NoiseChunkGeneratorMixin.java:124-129`）；注意只增不删（仅 `wg_clear_beardifier` 全清，`api.rs:193-197`）⇒ 内存面 |
| B5 | flags 回显 / `CA_*` / `CA_NT_*` 诊断计数 / 一次性 CONF 打印 | `:642-661`、`:1076-1091`、`:166-172` |
| B6 | inflight 并发度本身 | 只影响 wall；**但它经由 A1 放大成内容敏感**（inflight=1 顺序确定 → async 后不定），即本课题的因果链 |

### C. 重启前需先固化的口径（非顺序敏感，但会影响验收判据）

- C1 接管判定 = `mask 非零 且 bit1 清`（`CppBridge.java:167-170`）⇒ `all`/`0` 是对照臂而非接管臂，`1` 才是接管臂（`knowledge/discovered/workflow-patterns.md` 发现 #66/#44 家族）。
- C2 Java 让位只 redirect `PlacedFeature.generate`（`ChunkGeneratorFeaturesMixin.java:129-148`），
  `generateFeatures` 内的**结构件放置段保留**（`:59-89` HEAD 只做门控不 cancel；理由见 `:20-39`）
  ⇒ 结构件的跨 chunk 写仍是 Java write-through 语义，「Rust features + Java structures」是混合语义，
  验收口径必须先声明（发现 #66/#91 家族）。

---

## 对主会话的待验清单（需跑命令/实测，scout 无 shell）

1. **行为化直证（父会话进行中）**：mask=3 + `WG_CA_LOG=1` ⇒ `[CA]` 行数应为 **0**；mask=5 + `WG_CA_LOG=1` ⇒ 应 **>0**
   （正向对照）。这是「apply_features 真不执行」的执行面证据，用于给 §3 的静态链补行为证据。
2. **A1 drop 计数（当前无会计点）**：mask=1 + `WG_CA_LOG=1` 下，需要**新增**计数（例如在 `apply_features` 开头
   `remove` 之外，统计「目标已跑过 features 的迟到写」）才能量化丢写。判据：drop>0 ⇒ A1 可观测。
   现无该计数，无法用现有 `CA_PENDING_WRITES`（`:1080-1081` 只统计入缓冲次数）区分「已生效」与「永丢」。
3. **`pending_cross_writes` 常驻条目实测**：mask=1 区域 sweep 后 dump map key 数（需新增诊断出口，`api.rs` 无对应导出）。
   判据：sweep 结束后 map 非空 ⇒ leak/陈旧注入面成立。
4. **A3 可达性实测**：mask=1 下给 `get_ocean_floor_top_y`/`get_world_surface_top_y` 越界分支（`feature.rs:196/204`）
   + `placement.rs:419/450/478` 越界分支加计数，确认生产 feature 集合是否真的走到；若为 0，A3 降级为「理论面」。
5. **1.3 后效（vanilla 侧）核实**：确认邻 chunk 为 FULL（`WrapperProtoChunk(..., false)`）时
   `ChunkSectionCache.getSection` 是否真返回空 section（我的**推断**）。做法：Java 探针 mixin 打点
   `WrapperProtoChunk.getSection` / `propagateToWrapped` / `ChunkRegion.getChunk` 返回类型，1.21.6 实机跑一次。
   （本仓库 extract 已是源码级，但「运行时窗口有多大」需实机。）
6. **定量对拍前置**：vanilla features 层本身 run 级非确定（`knowledge/discovered/workflow-patterns.md` 发现 #67），
   任何跨 chunk 语义差的定量对拍必须用「冻结顺序生成」载体或多次取交集判据，否则差异不可裁决。
7. **若采纳「区域共享列存储」替代 buffer**：需先确认 JNI 面扩展可行性——现 `api.rs` 只有 per-chunk 的
   `wg_fill_blocks_multi`（`:130-171`），Rust 侧**没有** C++ 当年的两阶段入口（`wg_fill_blocks_multi_phase`），需新增导出。

## 未核到清单（明确说明为什么）

- **Java→Rust 数组写回 ProtoChunk 的具体代码点**：`CppBridge.fillChunk` → `CppWorldgen`/JNI 的写回路径本轮未读
  （不影响本 4 问结论，但「Java 侧写回本身会不会丢/覆盖」这一面未核）。
- **mask=1 臂当前的实际数值残差**：本问只做语义面比对，不做数值对比（需要跑 mask=1 对照臂）。
- **「目标邻 chunk 在 FEATURES 写时刻处于哪个 status」的运行时分布**：决定 §1.3 窗口的实际大小，需实机探针（待验 5）。
- **`WG_FEATURESEQ` 序列在 mask=1 下的当前一致性**：属另一课题，本轮未核。
- **C2ME `readonly_protection` 是否另有 section 级只读处理**：仅核到 `MixinChunkRegion`（tick scheduler 面，
  `versions/1.20.1/data/C2ME-fabric/.../readonly_protection/MixinChunkRegion.java:42-73`）与
  `ServerAccessible.java:76/85` 的 `new WrapperProtoChunk(worldChunk, false)` 同构用法；
  section 级读为空的后果未在 C2ME 侧核到专门修复。
