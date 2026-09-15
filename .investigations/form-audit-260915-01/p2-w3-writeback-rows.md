# P2-w3 — D 段（写回）执行形态错配矩阵行

---
工作块: 260915-01（形态审计 P2 fan-out，w3 = writeback 段）
角色: core-worker（对照 P1a/P1b 两份 scout 产物 + 一手源核对）
计划: .investigations/000-架构设计/架构设计-260914-04b-形态审计.md
日期锚: 2026-09-15（继承工作块）
状态: draft（worker 产物，candidate 授予需 judge + 主会话；confirmed 仅人类）
辖区: D 段 = CppBridge.writeChunk → BulkWb.writeSections（bulk / 逐块开关）+ writeChunk 内 heightmap 补设。存盘序列化本身 = vanilla 让位段（P1a §1.3），只做交互论证。
一手源核对: java-core/.../CppBridge.java:608-710（writeChunk/writeChunkPerBlock/SKIPAIR/BULKWB）；java-core/.../BulkWb.java（全文，163-311 主体）；P1b §3.2/§3.8（vanilla NOISE 写 / serialize）。
---

## 0. 前置澄清：D 段的 vanilla 参照系是双重的（关键裁决前提）

任务书把「写回/序列化」并提，但一手形态对照必须拆开——**我方 D 段写的是内存态 Chunk section（ProtoChunk，NOISE 阶段内）**，vanilla 有两个不同操作：

- **R1（同类操作）**= vanilla `populateNoise` 内部对 ChunkSection/heightmap 的写（NoiseChunkGenerator.java:330-364，P1b §3.2）：异步、main worker 池、逐格 `setBlockState` 语义、无限流。**这才是 D 段的真同位参照**。
- **R2（下游交互）**= `ChunkSerializer.serialize`（P1b §3.8）：同步主线程、每 tick 限 20 chunk、nibble 从 LightStorage 直读。这是**让位段**（我方零接管），与 D 段的关系只有「autosave/unload 何时拾取我方写回结果」这一交互面。

⚠️ 若直接拿 R2 的「主线程 20/tick」对照我方 work 线程写回，会得出假错配——20/tick 限流保护的是**主线程同步 NBT 构建不被卡死**，与 work 线程内存写回无对应关系。下文行内凡引 R2 均标 [R2-交互]。

## 1. 错配矩阵（5 维度）

### 行 D-① 同步性 —— 分类：形态不同但等价（附 1 个 open 交互项，见 D-①-b）

**我方形态**：随 A 段 work 异步——`fillBlocks` JNI 返回后同线程立即 `writeChunk`（P1a §2.4①，CppBridge.java:494-500 链）。无独立同步点、无限流。A/B 开关 `-Dcoreswap.syncfill=1` 可内联同步。

**vanilla [R1]**：populateNoise 的 section 写同样在 supplyAsync work 内异步执行（P1b §3.2），无限流。

**等价论证**：写回时机相对 chunk 生命周期完全同位（都是「NOISE 任务 work 内、future 完成前」）；调用方（ChunkStatus.NOISE 任务）在两种形态下都阻塞在同一 future 上，MC 状态机对下游可见性一视同仁——**下游阶段看到的都是「future 完成 ⇒ NOISE 完成」**，不感知写回发生在哪个线程/何时。⇒ 同步性维度等价。

### 行 D-①-b [R2-交互] autosave/unload 与 work 线程写回的并发面 —— 分类：**open（低嫌疑，推理级）**

- **时序论证**：TACR unload/save（:488-518）只对 ticket 释放后的 chunk 执行；持有 NOISE 任务在飞的 chunk 其 future 未完成 ⇒ 状态机保证 unload 不与在飞写回竞争（与 vanilla R1 依赖同一保证——vanilla 用 section lock :342-346 额外防的是**同 chunk 其它 worker 访问**，我方 HEAD cancel 后 populateNoise 原体不再跑，等价面成立，且有 `-Dcoreswap.bulkwbsentinel=1` 并发哨兵可实证（BulkWb.java:75-84，语义复刻老 LockHelper））。
- **内存压力论证（推演，非证据）**：写回无限流 + A 段生成加速 ⇒ 单位时间到达「可 serialize 状态」的 chunk 更多，主线程 20/tick 的 serialize 限流可能成为新瓶颈（排队变长 → 常驻已生成 chunk 内存增长 + dirty chunk 落盘延迟增大）。这是**加速的下游后果**而非 D 段形态错误；修复面不在 D（如需处理归 serialize 侧/e2e 调度）。**归属建议：e2e 候选池登记，不立项 D 段**；量级推演 = serialize 吞吐上限 20 chunk/tick = 400 chunk/s，超过该生成速率才触发（多线程 worldgen 大世界可达）。
- 验证方式（如立项）：unloadChunks 队列深度/needsSaving 积压计数探针。

### 行 D-② 线程放置 —— 分类：已知已修（#107）+ 1 个 open 随行项（归属建议 A 段）

**我方形态**：与 A 段同线程 = 自有池 `CoreSwap-Fill-N`（缺省 EXEC_MODE，260911-03 起默认）或 CallerRuns 回退时的 worldgen 车道（P1a §2.4②）。

**vanilla [R1]**：main worker ForkJoinPool（`Util.getMainWorkerExecutor()`，宽 = clamp(cores-1,1,255)，asyncMode）。

- **池归属差**：单车道问题已由 #107 修复（EXEC_MODE 自有池缺省开）——**已知已修**。
- **open 随行项：自有池与 ChunkTaskPrioritySystem 优先级脱钩**——vanilla 所有 worldgen 任务经 ticket-level 优先级排序（P1b §1），玩家近处 chunk 可插队；我方 LinkedBlockingQueue(128) FIFO + CallerRuns，**不感知优先级**。爆载时表现为「远处 chunk 与近处 chunk 同等待遇」，玩家跟随地形的 chunk 供给延迟升高。此缺陷属 A 段池设计（D 只是随行执行者），**归属建议：A 段矩阵行主报，D 行标注随行**。量级推演：仅队列打满/CallerRuns 触发时可见（128 深 + 池宽 logical/2-2）。
- CallerRuns 臂的写回落在「worldgen 车道调用线程」= 与 vanilla R1 线程语义反而更近——无错配。

### 行 D-③ 批粒度 —— 分类：形态不同但等价（已验证链完备）

**我方形态**：chunk 内**按 section 整段替换**（`BulkWb.writeSections` 逐 section 重建 PalettedContainer + 原地换容器 + 直写三计数，BulkWb.java:163-222）；旧路逐块 98,304 次 `setBlockState`（`-Dcoreswap.bulkwb=0` A/B 回退臂）。

**vanilla [R1]**：逐格增量写（cell 插值 → `ChunkSection.setBlockState` 语义，P1b §3.2）。

**等价论证（证据链已闭合，260911-05/260913-03 工作块）**：
1. 计数语义复刻**增量**路径而非 `calculateCounts()`（BulkWb.java:51-64 注释 + ② 段实现）——消除了 calculateCounts 与增量语义的两处不一致（非空气流体重复计/静水 randomTicks 差），且实测 72.5µs→237ns/section；
2. Tier 1/2 写回后逐 chunk 指纹门（`-Dcoreswap.wbcontent`，hash + 派生计数双层）；Tier 4 对称计时同 run A/B；WBCHECK 跳空气前提逐格实证（CppBridge.java:612-615/689-696）；
3. 合成自检覆盖 SINGULAR/ARRAY/BI_MAP/ID_LIST 四编码支（`-Dcoreswap.bulkwbtest`，ID_LIST 自然不可达故刻意压测）；
4. 并发哨兵恢复老 LockHelper 检测能力（judge SHOULD-2 过，260913-03）。

⇒ bulk 是「形态不同但等价且更优」的范本行，**无需动作**。vanilla [R2] 的「1 chunk、20/tick」粒度不适用（见 §0 前置澄清）。

### 行 D-④ 增量 vs 全量 —— 分类：形态不同但等价 + 1 个 **open**（needsSaving 门控）

**我方形态**：全量整块写——前提 = 接管点 populateNoise HEAD cancel ⇒ chunk 全新（CppBridge.java:612-615 论证 + WBCHECK 门控实证）；A1a 空气不写（`-Dcoreswap.skipair` 1.20.1 缺省开，语义 no-op 论证 :612-615，前提「当前格恰为 Blocks.AIR」由 WBCHECK 严格化核查 :689-696）；均质空气 section 短路（BulkWb.java:185-198，含「旧 section 非空则不短路」的安全侧）。

**vanilla [R1]**：调用形态增量（逐格 setBlockState），但 chunk 初始全空 ⇒ **语义上同样是全量建设**——等价。

**open 子项 D-④-b：bulk 路径是否遗漏 needsSaving/dirty 标记**：
- vanilla 逐格路径经 `setBlockState` 链（ProtoChunk 侧）自然进入 chunk 脏标记体系；我方 bulk 路径**原地替换容器 + accessor 直写三计数**（BulkWb.java:203-210），全程不经过任何 setBlockState/标记调用。
- 推演风险面：TACR save 有 needsSaving 布尔门控（P1b §3.8，:799-802）——若某 chunk 在我方写回后、任何 vanilla 后续写（carver/features）发生前被 unload 存盘，且 ProtoChunk 的 unload-save 同样受该门控且无人置位 ⇒ **生成的方块可能不被落盘**。
- 缓解事实（降低嫌疑但未闭合）：① 我方接管链上 chunk 必然继续走 vanilla CARVERS/FEATURES（stageMask 缺省 0b011 让位，P1a §1.3），这些阶段的写会走 setBlockState 正常标脏；② vanilla R1 的 NOISE 写是否本身置 needsSaving 未在本轮核实（populateNoise 写 section 的路径同样偏底层）。**本行是推理级 open，需一轮源码核对闭合**（核对点：ProtoChunk/Chunk.setNeedsSaving 的全部调用方 + TACR:797-802 门控是否区分 ProtoChunk unload save）。归属：D 段保留（写回路径的副作用完整性属 D 辖区）。

### 行 D-⑤ 缓存层 —— 分类：形态不同但等价

**我方形态**（写者/读者逐一）：
- `STATE_BY_ID` AtomicReferenceArray（进程级，写 = 首次惰性，读 = 写回全程）——vanilla 无此层（直接持 BlockState 引用），我方多出 raw-id→state 映射需求，幂等无竞争；
- BulkWb per-thread `TL`（scratch/stamp epoch 免清零 + localIdx/cnt/pal + ByteBuf 复用，BulkWb.java:136-149）——写者 = 单写线程私有，读者 = 同线程 ⇒ 零共享竞争，epoch 回绕处理正确（:358-366）；
- 并发哨兵 SENTINEL_ACTIVE map——门控默认关，仅 `-Dcoreswap.bulkwbsentinel=1` 在场，chunk 级一次 map 操作（#98 诊断门控纪律合规）。

**vanilla [R1]**：无写回专属缓存；跨阶段复用 ChunkNoiseSampler（挂在 chunk 上）——我方无对应需求（Rust 侧管线自含）。

**等价论证**：双方写回路径均无跨线程共享可变缓存；我方多出的两层（state 映射 + TL scratch）均为无竞争结构。vanilla [R2] 的 LightStorage nibble 直读属 C 段/序列化让位段——**辖区外，归属建议：C 段矩阵行**。

## 2. 辖区外项归属建议（只标注不展开）

| 项 | 归属 |
|---|---|
| serialize 主线程 20/tick 限流形态本身 | vanilla 让位段（无 mixin），只在 D-①-b 作交互论证 |
| LightStorage nibble 直读/uncached 快照 | C 段（光照）行 |
| 自有池优先级脱钩（ChunkTaskPrioritySystem 不感知） | A 段（NOISE/fill）行主报，D 随行 |
| Rust buf 生成/threads/terrain_cache | A 段行 |
| heightmap 补设 6 种 | **D 辖区内**，但独立见 §3 |

## 3. 附行：heightmap 一次性补 6 种（writeChunk 内，CppBridge.java:657-667）

**分类：形态不同但等价（有条件）——建议 judge 抽查条件**

- vanilla：populateNoise 只设 WORLD_SURFACE_WG/OCEAN_FLOOR_WG 两 WG 型（P1b §3.2），正式 4 型在 FEATURES 步前 `Heightmap.populateHeightmaps` 全量重算（P1b §3.5）——即 vanilla 正式 heightmap 反映 **carve 后**状态。
- 我方：NOISE 末（carve 前）一次性补全部 6 型。**等价条件** = 后续 vanilla CARVERS/FEATURES 的写块经会增量维护已注册 heightmap 的路径（ProtoChunk.setBlockState 链），使我方早期值被增量修正到 carve 后正确态。该条件与 vanilla 自身依赖的机制同源（vanilla 若无 populateHeightmaps 重算也同样依赖增量维护），成立面大，但「ProtoChunk 写路径对全部 6 型 heightmap 的增量更新覆盖」未在本轮逐行核实——列为条件项交 judge 决定是否补核。

## 4. 汇总

| 行 | 维度 | 分类 |
|---|---|---|
| D-① | 同步性 | 形态不同但等价 |
| D-①-b | autosave/unload 交互 | **open**（低嫌疑，归属建议 e2e 候选池） |
| D-② | 线程放置 | 已知已修（#107）+ open 随行（归属 A 段） |
| D-③ | 批粒度 | 形态不同但等价（已验证链完备，范本行） |
| D-④ | 增量 vs 全量 | 形态不同但等价 |
| D-④-b | needsSaving 门控 | **open**（推理级，D 辖区保留，一轮源码核对可闭合） |
| D-⑤ | 缓存层 | 形态不同但等价 |
| D-hm | heightmap 补 6 型 | 形态不同但等价（有条件，交 judge） |

**错配行数：8 行（open 2 + open 随行 1 + 有条件等价 1 + 已知已修 1 + 形态不同但等价 3）**

open 嫌疑摘要：
1. **D-④-b（主嫌疑）**：bulk 原地替换不经过 setBlockState/标脏链，ProtoChunk 早 unload 场景下 needsSaving 门控可能吞掉生成方块不落盘——需一轮源码核对（TACR:797-802 + ProtoChunk 脏标记调用方）闭合；
2. **D-①-b（次嫌疑，建议不立项 D）**：写回无限流 + 生成加速 ⇒ serialize 20/tick 主线程上限可能成为新瓶颈（>400 chunk/s 生成速率时），内存常驻增长——e2e 候选池登记；
3. **D-② 随行（归属 A）**：自有池 FIFO 不感知 ticket 优先级，爆载时玩家近处 chunk 供给延迟无插队。

retry 轮次：1（本轮全部差异点均有代码级证据或明确标注推理级，无证据饱和触发）。
自检：本文件未授 confirmed/candidate；所有行号经一手源/两份 scout 产物核对；推理级论断（D-①-b/D-④-b/D-hm 条件）均显式标注未闭合。
