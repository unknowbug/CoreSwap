# b2 候选：写回语义副作用 = 948ms gap 的主体

> **结论（前置）：REFUTED（作为「主体」主张）** —— 写回差异在**直接成本**上是 3.7ms/chunk（占 999ms 周期的 0.37%），在**后续阶段**上是 ≈0（光照/后处理/序列化/高度图逐条核对后均不产生增量，方向甚至有负）。因此它不可能构成 948ms 的主体。幸存的是两个**量级极小**的真实差异（全 air 写、6 张高度图冗余），量级合计 ≤3.7ms/chunk。
>
> 口径：静态推导（一手 yarn 源码 `path:line`）+ 任务书已核实实测事实。**未跑任何命令/探针/构建**。验证分层 = **Degraded（静态审查，无运行证据）**。置信度 = **draft**。凡未实测的次数/占比一律进 §6 @idk，不猜数。
> 本文件是本候选的唯一产物；只读分析，未修改任何源码，未写 `knowledge/`、`docs/`、其他 `.artifacts/`。

---

## 候选主张

**原文复述（待判）**：「与 vanilla 相比，我们的写回路径有若干语义差异，这些差异可能在**后续 stage**（尤其光照/高度图/区块状态/序列化/后处理）上产生 vanilla 没有的额外开销或额外工作量，是那 948ms `gap` 的主体。」

**判定**：分两层，结论相反。

| 层 | 判定 | 依据摘要 |
|---|---|---|
| 直接成本（NOISE 内的写回本身） | 差异**存在但极小** | 已测 `write=3.5ms`（98304 次 `setBlockState`，**含 air**）＋ `hmap=0.22ms`（6 张全量）＝ 3.7ms/chunk，占 999ms 周期 0.37%（`ChunkTiming.java:60-67` 口径） |
| 后续 stage 增量（本候选的真正主张） | **REFUTED（≈0，且方向为负）** | 光照不读 Heightmap；高度图 4 张必被 FEATURES 重建（冗余而非增量）；post-processing 我们标记更少；序列化只由内容决定；section 空标志/计数写在 air 上无副作用（逐条见 §2） |

**上界论证（不依赖任何新测量）**：coreswap 臂整机 CPU = 1.7 核 × 234s ≈ 398 核·秒 / 4225 chunks ≈ **94ms CPU/chunk**；写回占其中 3.7ms → **即使假设 948ms gap 100% 是 CPU 争用，写回的归因上界也只有 3.9%**。而实测 1.7/24 核 ⇒ 系统 93% 空闲、gap 主要是**非 CPU 的排队/等待** ⇒ 以「多加 CPU 工作」为形式的候选（写回正是此类）在结构上不能主导 gap。

---

## 差异清单（file:line + 是否可能产生后续阶段成本）

一手源码根：`versions/1.21.6/data/mc_src_extract/`（yarn 命名）；我方代码：`runtime/1.21.6/java/src/main/java/wg/bench/`。

### D1 写全部格（含 air） vs vanilla 只写非 air —— 直接成本小，后续成本 **否**

| 项 | vanilla | coreswap |
|---|---|---|
| 写入门槛 | `NoiseChunkGenerator.java:411` `if (blockState != AIR && !SharedConstants.isOutsideGenerationArea(...))` | 无门槛：`CppBridge.java:504-531` 逐 y/z/x 写满 `height × 256` 格 |
| 写入调用 | `NoiseChunkGenerator.java:412` `chunkSection.setBlockState(y,u,ab,blockState,false)` | `CppBridge.java:528` `sec.setBlockState(x, sy, z, st)`（3 参 → `lock=true`） |
| 每格副作用 | `ChunkSection.java:64-101`：`PalettedContainer.swap` → 3 个计数（`nonEmptyBlockCount` / `randomTickableBlockCount` / `nonEmptyFluidCount`）增减 | 同源（同一个方法） |

- **air→air 是计数空操作**（`ChunkSection.java:78-98`：`!old.isAir()` 假、`!new.isAir()` 假、两个 fluid 均 `isEmpty()` ⇒ 四个分支全不进）⇒ 写 air **不改变** `isEmpty()`（`:103-105`）、`getHighestNonEmptySection()`（`Chunk.java:144-155`）、`getHighestNonEmptySectionYOffset()`（`:160-163`）。
- **非线性副作用（palette 升级）不构成两臂差异**：`PalettedContainer.swap` → `palette.index(value)`（`PalettedContainer.java:164-167`）；`ArrayPalette.index` 是**线性扫描**（`ArrayPalette.java:48-53`，size ≤ 16），溢出才 `listener.onResize`（`:61`）→ `PalettedContainer.onResize`（`:139-145`）→ `importFrom` **整段 4096 格拷贝**（`:374-379`）。但**升级次数只由 section 内不同 state 数决定**（内容量），两臂内容等价 ⇒ 升级次数相同。air 还是 palette **首项**（构造时 `this.data.palette.index(object)`：`PalettedContainer.java:116-121`，默认值 = `Blocks.AIR`：`ChunkSection.java:42`）⇒ 写 air 命中最便宜的扫描分支（1 次 `==` 比较）。
- **后续 stage 成本：否。** 因为下游全部以「内容 / 空标志 / 计数」为输入，而这三者与 vanilla 等价：
  - `initializeLight` 的 per-section `setSectionStatus` 只为 `!isEmpty()` 的 section 触发（`ServerLightingProvider.java:148-156`）→ 触发数由内容决定；
  - `ChunkSkyLight.refreshSurfaceY` 以 `getHighestNonEmptySection()` + 方块内容为输入（`ChunkSkyLight.java:33-45,47-75`）→ 等价；
  - `Heightmap.populateHeightmaps` 以 `getHighestNonEmptySectionYOffset()` + 内容为输入（`Heightmap.java:43-79`）→ 等价；
  - 序列化字节（palette 位宽 / 打包数组长度）由内容决定（`ChunkSection.toPacket` `:172-180`；`SerializedChunk.java:380,464`）→ 等价。
- 成本上界 = **已测 3.5ms/chunk**（该计时含 98304 次 `STATE_BY_ID.get`（`CppBridge.java:513`）+ 上界检查（`:511`）+ 98304 次 swap + LockHelper 每格 2 次原子操作，见 D2）。vanilla 省下的「air 那部分」未拆测 → @idk-6。

### D2 每格加锁（lock=true）vs vanilla lock=false —— 仅 NOISE 内常数差，后续 **否**

- vanilla：`setBlockState(..., false)`（`NoiseChunkGenerator.java:412`）→ `ChunkSection.java:72-74` `swapUnsafe`（不加锁）；
- coreswap：3 参版（`CppBridge.java:528`）→ `ChunkSection.java:70-71` `swap` → `PalettedContainer.swap`（`:147-158`）→ `LockHelper.lock()/unlock()`；
- `LockHelper`（`LockHelper.java:17-71`）：`ReentrantLock.lock()` + `Semaphore.tryAcquire()` + `semaphore.release()` + `ReentrantLock.unlock()` ⇒ 每格 ≥2 次原子操作 + 1 次 AQS tryAcquire。
- **锁是每 section 私有**（每个 `PalettedContainer` 自建 `LockHelper`：`PalettedContainer.java:38,44-53`）⇒ **不触及任何全局/跨线程共享串行资源**，23 线程之间零争用。反观 vanilla 会在整段 fill 期间**持满 24 个 section 锁**（`NoiseChunkGenerator.java:336-340` lock / `:346-348` finally unlock）——我们反而是「竞争面更小」的一侧。
- 副产物（非成本）：vanilla 的整段持锁使 `LockHelper` 成为**并发访问检测器**（`LockHelper.unlock` `:56-71` 会抛 `CrashException`）；我们的路径不做该检测（正确性面，见 @idk-10，本候选不判定）。
- 后续 stage 成本：**否**（锁开销只发生在 NOISE 循环内）。

### D3 高度图：6 张全量 vs vanilla 增量 2 张 + FEATURES 再全量 4 张 —— 冗余，**不是增量**（否）

- vanilla：NOISE 只增量维护 2 张 WG 图（`NoiseChunkGenerator.java:359-360` 取图 + `:413-414` `trackUpdate`，**仅非 air 格**）；
- coreswap：NOISE 结束一次性全量 6 张（`CppBridge.java:534-539+`，`WORLD_SURFACE_WG / WORLD_SURFACE / OCEAN_FLOOR_WG / OCEAN_FLOOR / …`）；
- **关键（决定性）**：`ChunkGenerating.generateFeatures` 在 FEATURES 前**无条件**全量重建 4 张 NORMAL 高度图 —— `ChunkGenerating.java:135-137`（`MOTION_BLOCKING, MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, WORLD_SURFACE`）；类型集定义见 `ChunkStatus.java:18-20`，`FEATURES` 的高度图类型即 `NORMAL_HEIGHTMAP_TYPES`（`:28`）。⇒ 我们在 NOISE 预置的那 4 张**必然被丢弃重建**，是**纯冗余**，而非「后续 stage 的额外工作量」。
- **光照不依赖高度图**（推翻本候选在 item 2/3 的核心假设）：对 `world/chunk/light/` 全包 grep `Heightmap|getHeightmap` **零命中**；`initializeLight` 只读 `chunk.getSectionArray()` + `isEmpty()`（`ServerLightingProvider.java:148-156`），`refreshSurfaceY` 走 `ChunkSkyLight`（`ChunkSkyLight.java:33-45`）。⇒ 预置 6 张高度图**不改变**光照成本或行为。
- 成本上界 = **已测 0.22ms/chunk**（6 张全量的总耗时；其中 4 张冗余部分未拆测 → @idk-8）。

### D4 不标记 post-processing —— 方向为 **负**（我们更轻），否

- vanilla 在 NOISE 标记：`NoiseChunkGenerator.java:415-418`（`aquiferSampler.needsFluidTick() && !block.getFluidState().isEmpty()` → `chunk.markBlockForPostProcessing(mutable)`）；coreswap 无对应动作。
- 消费侧唯一执行点：`WorldChunk.runPostProcessing`（`WorldChunk.java:561-584`）逐条 `fluidState.onScheduledTick(...)` / `Block.postProcessState(...)`；条目累积于 `ProtoChunk.markBlockForPostProcessing`（`ProtoChunk.java:259-264`）。
- 静态上 coreswap 的条目**更少**：vanilla 侧还有 `SurfaceBuilder.java:98`（我们 cancel 了 buildSurface：`NoiseChunkGeneratorMixin.java:140-158`）、`Carver.java:145,154`、features（`Feature.java:185-194` 等）、`Blender.tickLeavesAndFluids`（`Blender.java:302-312`）。⇒ coreswap 的 `runPostProcessing` 负载**≤ vanilla**。
- 「不标记 → 触发别的路径 / 状态异常」无静态依据：空列表仅使循环体跳过（`WorldChunk.java:564-584`，无 else 分支）。

### D5 绕过 `ProtoChunk.setBlockState` 直写 `ChunkSection` —— **两臂同源**，不构成缺失（否）

- coreswap 直写 `ChunkSection.setBlockState`（`CppBridge.java:528`）；**vanilla `populateNoise` 也是直写 `chunkSection.setBlockState`**（`NoiseChunkGenerator.java:412`）⇒ 这一层两臂同源，不存在「我们少了记账」。
- 逐条核对 `ProtoChunk.setBlockState`（`ProtoChunk.java:112-168`）里在 NOISE 状态会做的事，确认三者**在 vanilla 侧同样不发生**：
  1. 光照记账 `if (this.status.isAtLeast(ChunkStatus.INITIALIZE_LIGHT))`（`:131-141`）→ NOISE 时状态为 NOISE（`ChunkStatus.java:25`，早于 `INITIALIZE_LIGHT`：`:29`）⇒ 假，两臂都不触发；
  2. 高度图：按 `getStatus().getHeightmapTypes()` 增量（`:143-163`）⇒ NOISE 只有 2 张 WG（`ChunkStatus.java:17,25`），而 vanilla 的 `populateNoise` 手工 `trackUpdate` 同样只有这 2 张（`NoiseChunkGenerator.java:413-414`）⇒ 等价；
  3. `markNeedsSaving` 不在此方法内（在 `setStatus`：`ProtoChunk.java:224-231`，每状态转换 1 次，两臂相同）。
- 唯一真实差异是**编排**而非写回语义：vanilla 的 fill 在 `Util.getMainWorkerExecutor()`（**同一个 Worker-Main ForkJoinPool**：`Util.java:103,204-235,262-264`；`NoiseChunkGenerator.java:352` `.named("wgen_fill_noise")`）上异步执行且期间持 24 把 section 锁；coreswap 在 mixin HEAD **同步**跑完（`NoiseChunkGeneratorMixin.java:99-113`）。→ 属调度结构候选（scout 的 S1/S3），**不属本候选**。

### D6 区块状态 / 序列化 —— 否

- `SerializedChunk` 保存内容由 palette 决定（`SerializedChunk.java:380`）；`PostProcessing` 段同理（`:464`）⇒ coreswap 条目更少（D4）⇒ ≤ vanilla。
- 区块状态推进 `ProtoChunk.setStatus`（`ProtoChunk.java:224-231`）与「怎么写入方块」无关。
- 「NOISE 前置状态未完成即写方块」在本候选语境下不成立：`populateNoise` 由 `ChunkGenerating.populateNoise`（`ChunkGenerating.java:78-100`）在 NOISE 步骤内调用，写方块正是该步骤的定义；两臂处于同一状态。

### D7 顺带排除（非写回语义，列出以免混入）

- 每 chunk **无条件全量扫 98304 项**的诊断计数（`CppBridge.java:393-396`）→ 已计为 `scan=0.08ms`（0.008%）。
- `wgSettingsId()` 每 chunk 的 `Identifier.toString()` 分配（`NoiseChunkGeneratorMixin.java:54-59, 98`）→ Java 固定开销，与写回无关。
- 写回路径读的 `STATE_BY_ID`（`CppBridge.java:513`）是共享 `AtomicReferenceArray`，但**稳态只读**（仅首见 id 才 `set`：`:524`）⇒ 不构成共享串行点（volatile 读不制造写争用）。

---

## 机制推演

### 1) 严格的量级预算（用已核实事实，不引新数）

任务书已核实的 chunk 级实测（n=1024，`ChunkTiming.java`）：

```
mixin = 51.0ms  = jni 47.2 + write 3.5 + hmap 0.22 + scan 0.08 + beard 0.04     （恒等式自洽）
gap   = 948ms   （同 Worker 线程「上次接管返回 → 本次接管开始」）
周期  = mixin + gap = 999ms
```

**凡属「写回语义」的可归因项**：
- 全 air 写（相对 vanilla 仅非 air）→ 落在 `write` 内，其 air 份额 ≤ 3.5ms（未拆测）；
- 每格加锁（D2）→ 落在 `write` 内；
- 6 张高度图（相对 vanilla 2 增量 + 4 重建）→ 冗余份额 ≤ 0.22ms；
- 光照（D3）→ **0**（光照包零 Heightmap 引用；成本输入 = 内容 + 空标志，等价）；
- post-processing（D4）→ **≤0**（我们标记更少）；
- 序列化 / 区块状态（D6）→ **≈0**。

合计 ≤ **3.7ms / 999ms = 0.37%**。要成为 948ms 的「主体」，至少需要后续阶段出现 **≥900ms/chunk** 的增量；而我们能观测到的后续阶段（carve 1.3ms + feat 3.1ms = 4.4ms/chunk）根本没有这个体量，静态可核的光照/序列化/后处理也没有。

### 2) 为什么「CPU 型候选」在本案结构上不可能主导 gap

- coreswap 臂 CPU = 1.7 核（已核实）× 234s ≈ 398 核·秒；/4225 ≈ **94ms CPU/chunk**，其中 mixin 51ms（写回 3.7ms 占 3.9%）。
- 若 gap（948ms/周期）是 CPU 争用，则整机应接近 24 核饱和；实测 1.7/24 ⇒ **93% 空闲**，线程绝大多数时间不执行 CPU 工作 ⇒ gap 的主体是**排队/依赖等待/无任务**三类之一，而不是「谁多算了点 CPU」。
- 唯一能让写回影响 gap 的宽口径形式是「写回落在某个**互斥临界区**内，从而抬高吞吐周期」。检验：若每 chunk 进入一次临界区、临界区长 T，则吞吐 = 1/T，per-thread 周期 ≈ 23·T。代入 T=51ms ⇒ 23×51 = 1173ms，与实测 999ms 同量级 —— 这说明 gap ≈ **22×T 的排队**，而在该模型下写回只占 T 的 3.7/51 = **7.3%**（≤ 948×7.3% ≈ 69ms），仍非主体；并且该临界区位于何处（Rust 全局 `terrain_cache: Mutex` / `est_l2: Mutex` / `beardifiers: RwLock`，或 chunk 调度器）属**其他候选**，不属写回语义。写回本身操作的都是 **chunk 私有对象 + 每 section 私有锁**（D2），不构成共享串行资源。

### 3) 候选内含的**真实但小**的差异（值得记录，不构成回归主体）

1. 全 air 写：98304 次 vs ~非 air 数——可通过**零行为变化**开关消掉（§5 测量 3a）。
2. 6 张高度图：其中 4 张必被 `ChunkGenerating.java:135-137` 重建——同为零行为变化开关（§5 测量 3c）。
3. 每格 `lock=true`（D2）——可零行为变化切回 `false`（§5 测量 3b）。
4. 每 chunk 一次无条件 98304 项诊断扫描（`CppBridge.java:393-396`）——与本候选无关但同属「白付」。

---

## 反证/证伪条件

**本候选（b2）在以下任一条件下复活 / 我的 REFUTES 被推翻：**

1. **光照段实测 ≥ ~500ms/chunk**：给 `ServerLightingProvider.light(Chunk,boolean)` 加 chunk 级计时（挂入现有 `ChunkTiming`，门控 `coreswap.chunktime`），若 coreswap 臂显著高于 vanilla 臂且量级达数百 ms → 「写回 → 光照」成立（本候选复活）。静态预测：同量级、≤ 数 ms。
2. **「跳 air 写 / lock=false / 只补 2 张高度图」三臂合计的 wall 改善 ≫ 3.7ms/chunk**（例如 wall 从 234s 掉到 <150s）→ 说明存在我没推演出的下游放大机制（本候选复活）。当前预测：合计 ≤3.7ms/chunk（≤0.4% 周期）。
3. **内容等价性被证伪**：若同 seed 两臂逐格对拍发现 Rust 输出与 vanilla NOISE 输出**不等价**，则 §2 中「光照/序列化/高度图/空标志等价」全部失效，写回可能通过内容差异间接改变下游（此时主角变成内容差异而非写回方式）。
4. **线程状态 census 显示 Worker-Main 绝大多数时间 RUNNABLE 且整机 CPU ≈ 24 核**：那我引用的「1.7 核 / 93% 空闲」事实失效，gap 变成 CPU 型，则所有 CPU 项（含写回）都要重新加权。这是我最希望被优先检验的一条（它直接检验我论证的地基）。
5. **GC 停顿在生成期合计 ≫100ms**：写回产生 98304 次 Java 写 + 每 chunk ~393KB×2 JNI 流量（`CppBridge.java:377`、`jni_bridge.rs` 侧 buffer），若实测 GC 主导 gap，则要改判为「分配压力」（与写回弱相关，属 R15 域）。

**反向（若上述均不出现，则 b2 保持 REFUTED）**：光照计时与 vanilla 同量级、三臂改善 ≤3.7ms/chunk、内容等价、线程多处于 WAITING、GC 停顿可忽略。

---

## 建议测量（信息量排序）

> 全部为**诊断优先、零行为变化**；不建议任何语义大改。前两项即可基本定案。

**1. 线程状态 + mixin 并发度 census（最便宜、判别力最高）**
- mixin 内维护 `AtomicInteger` 当前并发/峰值/均值（chunk 级，非逐点；门控同 `ChunkTiming`），以及 `exit` 时记录本线程 `Thread.getState()` 分布；外部每 5s × 10 次 `jstack` 采样统计 `Worker-Main-*` 的状态分布。
- 预期读数：并发均值 ≈ 1.0–1.2（= 51/999 × 23 的推算值）；绝大多数线程 `WAITING/TIMED_WAITING`（parked）而非 `RUNNABLE`。
- 判据：若成立 ⇒ gap 是**排队/依赖等待**，任何 CPU 型候选（含写回）在结构上被排除；若并发均值 ≈ 23 且线程 RUNNABLE ⇒ 我的地基错，转条件 4。

**2. 光照段 chunk 级计时 + 工作量计数（本候选唯一的合理藏身处，且当前完全未测）**
- 计时：`ServerLightingProvider.light(Chunk,boolean)` HEAD/TAIL（`ServerLightingProvider.java:165-177`）；
- 计数：`initializeLight` PRE 任务里 `setSectionStatus` 调用次数（`:148-156`）、`ChunkLightProvider.doLightUpdates()` 的返回值（处理条目数，`ChunkLightProvider.java:138-154`）、`hasUpdates()` 结果（`:198-200`）per chunk 累计；
- 判据：两臂计数应**相同**（内容等价 ⇒ 触发数相同）。若计数相同而耗时差异巨大 ⇒ 差异来自**调度**（非写回）；若计数不同 ⇒ 内容不等价（转条件 3）。

**3. 零行为变化写回 A/B（三开关臂；只对新世界生成安全，因为 section 初始全 air）**
- a. `if (id == 0) continue;`（跳 air 写；`CppBridge.java:509-510` 处）——预测省 ≤3.5ms/chunk；
- b. `sec.setBlockState(x, sy, z, st, false)`（lock=false，对齐 `NoiseChunkGenerator.java:412`）——分离 `LockHelper` 成本（D2）；
- c. NOISE 只补 2 张 WG 高度图（`CppBridge.java:534-539`），其余 4 张交给 `ChunkGenerating.java:135-137`——分离 D3（预测 ≤0.22ms）。
- 三臂逐项 A/B，**不是叠加跑**；合计预测改善 ≤3.7ms/chunk。

**4. 两臂同 chunk 内容对拍（把载荷假设变成事实）**
- 同 seed，coreswap 臂与 vanilla 臂对同一 chunk dump 98304 格并做 hash/逐格比对（现有 `BlockProbe` / `block_probe` 通道）。
- 这是 §2 所有「光照/序列化/高度图/空标志等价」结论的**载荷前提**（@idk-1）。测完即可把 REFUTES 从「条件性」升为「无条件」。

**5. GC / JVM 停顿排除**
- `-Xlog:gc*` 跑同一 region，统计生成期总停顿与停顿次数。若总量 ≫100ms 需归因分配压力（与写回弱相关）。

**6. （低优先级）`writeChunk` 内部再分三段计时**
- `STATE_BY_ID` 映射 + 上界检查 / `setBlockState` 循环 / `populateHeightmaps`（后者已有 `hmap` 计时），并统计 air 写占比 —— 为测量 3 提供预期读数与 A/B 归因。

---

## @idk 清单

- ⚠️ **@idk-1**：**Rust 输出 ≡ vanilla NOISE 输出的方块内容等价性，在本 region 未以 chunk 级逐格对拍验证**。这是本候选全部「后续阶段成本等价」论证的**载荷前提**（光照/序列化/高度图/空标志/`runPostProcessing` 条目数全部由内容派生）。需要：同 seed 两臂逐格对拍（测量 4）。**在此之前本文件的 REFUTES 是「条件性」的**。
- ⚠️ **@idk-2**：98304 格中 air 格的实际数量（= 「跳 air 写」可省份额）——未实测，禁止估算。
- ⚠️ **@idk-3**：光照阶段每 chunk 的真实耗时与工作量计数——本波计时完全未覆盖光照片段（只有 carve/feat）。需要：测量 2。
- ⚠️ **@idk-4**：per-thread `gap` 948ms 的时间构成（阻塞于 chunk 调度器依赖门 / 共享锁 / 无任务可做 / IO）。需要：测量 1（线程状态 census）。**这是 b2 与 b1 之间的判别点。**
- ⚠️ **@idk-5**：mixin 段的**实测并发度**。我的「≈1.0–1.2」是由 `51ms/999ms × 23` 反推，属**推算非实测**。需要：测量 1。
- ⚠️ **@idk-6**：`writeChunk` 3.5ms 内部 air 写 vs 非 air 写的耗时占比（未拆测）。
- ⚠️ **@idk-7**：生成期 GC 停顿总量（写回/JNI 每 chunk ~393KB×2 分配是理论压力源，未测）。
- ⚠️ **@idk-8**：`Heightmap.populateHeightmaps` 6 张 vs 2 张的实测差——0.22ms 是 6 张**全量**的总耗时，其中 4 张冗余部分未拆出。
- ⚠️ **@idk-9**：两臂 `WorldChunk.runPostProcessing` 的实际条目数（我据静态推 coreswap 更少，未测）。
- ⚠️ **@idk-10**：coreswap 写回不持 section 锁导致的并发可见性风险（scout @idk-17 的正确性面）。本候选不判定；仅标注 **不是成本项**。
- ⚠️ **@idk-11**：vanilla 臂的 CPU 占用与线程 census 未实测。因此本文件不以「vanilla 也饱和 CPU」作为论据，只用 coreswap 臂已核实的 1.7 核。

---

## 越界声明

未跑任何命令/探针/构建；未修改任何源码；未写 `knowledge/`、`docs/`、其他 `.artifacts/` 文件。所有 `file:line` 为 `versions/1.21.6/data/mc_src_extract/` 与 `runtime/1.21.6/java/src/main/java/wg/bench/` 的一手源码引用；所有数字均来自任务书已核实事实或由其一阶推算（推算处已标 @idk）。结论 = **draft**（静态推导、Degraded），无 confirmed 级判断。
