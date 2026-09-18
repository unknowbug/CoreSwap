# scout-map — R9-b 并发义务缺口：生成期 sections 读者全清单（工作块 260918-10）

> scout 只读勘探（re-code / recode-scout 形态）；只勘探不裁决，覆盖状态判定留给主会话 T2。
> 验证分层：Degraded（纯静态源码对照；未运行任何命令/构建）。
> 一手源：CoreSwap 源码树（java-core / versions\1.20.1\java / worldgen-core / versions\1.20.1\rust）；vanilla 参照 = `.tmp\scout-260905-08\mcsrc\`（标注 1.20.1，见 §6 版本疑点）与 `.tmp\sentinel-260913-04\mcsrc1216\`（1.21.6）。
> 吸收：.investigations\form-audit-260915-01\p2-w2-light-rows.md（光照车道/线程域结论）、versions\1.20.1\docs\07-block-pipeline.md「2026-09-13 R9-b」三小节、docs\12-lighting.md。

---

## 0. 写侧事实锚（bulk 写回的时机与线程）

| 项 | 事实 | 证据 |
|---|---|---|
| 写回入口 | `BulkWb.writeSections(Chunk,…)` → 原地换容器（`acc.wgSetBlockStateContainer(pc)`，section 数组元素引用不变）+ 三计数 plain 直写 | java-core\src\main\java\wg\bench\BulkWb.java:163-222（:205-209） |
| 调用点 | `CppBridge.writeChunk` 分派（BULKWB 开 → BulkWb；否则 writeChunkPerBlock），随后 Heightmap.populateHeightmaps | java-core\...\CppBridge.java:651-670 |
| 触发时机 | populateNoise HEAD cancel 接管（目标 chunk 全新），= **ChunkStatus.NOISE 段**；写回与 SURFACE/heightmap 同在 fill 闭包内完成 | CppBridge.java:614（接管点注释）、NoiseChunkGeneratorMixin.java:42/219（1.20.1 树） |
| 线程域 | fill work 闭包线程（#107 后异步单跳 future；CoreSwap 自有池/调用车道，形态审计 W1 段），**非主线程** | form-audit p2-w3-writeback-rows.md + 07 篇形态矩阵 A 段（W1） |
| sentinel 覆盖 | chunk 级**写者×写者**在飞重叠（SENTINEL_ACTIVE putIfAbsent/remove），读者不登记 ⇒ **读者×写者零检测** | BulkWb.java:97-102, 313-340 |

---

## 1. 读者全清单（生成管线期 PROTOCHUNK→FULLCHUNK→INPLAY）

| # | 读者站点 | 读什么 | 线程域 | 获取途径 | 在 future 链上？ | 证据 |
|---|---|---|---|---|---|---|
| R1 | CoreSwap 光照收集（blocks9 直采）`wgLightCollectBlocks` | 3×3 邻 chunk 全部 24 section：`nc.getSectionArray()` → `sec.isEmpty()` / `sec.getBlockState(x,ly,z)` | LIGHT 状态任务车道（worldgen "worldgen" TaskExecutor 线程，FJP 上）；mixin HEAD 同步内联，无自有池 | `wgLightNeighbor` = `ChunkProvider.getChunk(x,z)` **裸引用 + `getStatus().isAtLeast(FEATURES)` 门**（**非 future join**） | ❌ 不 join future；仅轮询 status 字段 | ServerLightingProviderMixin.java:248-299（:263-294 直采）、:301-318（裸引用+门）、形态审计 W2 行 L1/L2（线程域） |
| R2 | CoreSwap 光照收集（packed 快路）`wgLightCollectPacked` + `wgLightFillSectionFast` | 同 R1，读面 = `sec.getBlockStateContainer().writePacket(pbuf)`（**读容器内部结构**，非快照） | 同 R1（light 调用线程；260916-01 崩溃修复后收集留在提交线程） | 同 R1 | ❌ 同 R1 | Mixin:326-394（:358 writePacket）、:433-491；Mixin:539 注释「容器读回提交线程」 |
| R3 | CoreSwap 域批任务 `wgLightDomainTaskRun`（-Dcoreswap.light.domainbatch） | **零 PalettedContainer 访问**（blocks9 快照在提交线程收集，任务线程只拼帧+JNI） | Util.getMainWorkerExecutor 线程 | 输入 = 提交线程拷贝快照（`Arrays.copyOf`） | ✅ 结构上隔离（读发生在 R1/R2 同线程） | Mixin:497-508（hook/executor）、:540-545（提交线程收集+copyOf）、:576-577 注释 |
| R4 | vanilla light engine 传播读（gate 关/fallback 时）`ChunkLightProvider` getStateForLighting → `lightSourceView.getBlockState(pos)` | 本 chunk + **传播越界的邻 chunk** section（含 PalettedContainer 读） | vanilla light 逻辑队列（TACR:188 TaskExecutor "light"，同一 FJP 物理池，**单队列串行 drain**）；tick 时 ServerChunkManager 主线程投递 | `chunkProvider.getChunk(x,z)`（LightingProvider 持有 ChunkProvider=TACS）裸引用；null → BEDROCK 默认 | ❌ 不 join future（vanilla 自身亦裸引用）；HB 依赖 status 依赖链 | mcsrc ServerLightingProvider.java:171-184（light 两段）、:127-138（enqueue 入 priority 队列）、:195-218（runTasks drain）；TACR:184-194（"worldgen"/"light" 双逻辑队列同池）；ChunkLightProvider.java:75-76,94-103（读块） |
| R5 | vanilla INITIALIZE_LIGHT 步 `initializeLight` | 本 chunk section 数组逐节 `isEmpty()` 扫描 | vanilla light 队列（同 R4） | light() 调用传入的 chunk 直接引用 | ✅ 该 chunk 自己的 LIGHT 前置步（同 status 链内），但执行线程是 light 队列 | mcsrc ServerLightingProvider.java:151-168（:154 getSectionArray + isEmpty） |
| R6 | 同 chunk 后续状态任务（SURFACE→CARVERS→FEATURES→…→FULLCHUNK） | getBlockState/setBlockState 经 chunk 引用 | worldgen 车道线程 | ChunkHolder future 链（thenApply/thenCompose） | ✅（R9-b 已论证的消费者类） | 07 篇 R9-b 节 1501 行；ChunkStatus.runGenerationTask :351-363（mcsrc） |
| R7 | 邻居任务的依赖读（taskMargin=1 邻 chunk 引用传入） | 邻 chunk 方块/高度图 | worldgen 车道线程 | future 链依赖装配（generationTask 拿 chunks 数组） | ✅（R9-b 论证的「邻居任务」类） | 同上 |
| R8 | 主线程消费（实体/tick/发包/serialize 存盘链） | section 容器读（ChunkSerializer:310-311 只写 block_states/biomes 等） | ServerThread（tick 限流 serialize 20/tick，形态审计 R2 行） | ChunkHolder 完成态 future（getValidFutureFor / getOrNull） | ✅/部分（R9-b 已论证「主线程/存盘链」类） | BulkWb.java:62-64 注释、07 篇 R9-b :1501；mcsrc ChunkHolder.java:186 |
| R9 | 诊断读者（默认关）：BulkWb.readbackHash / contentLine（写者同线程）；LightPalDump.section（R1/R2 线程）；FormProbe / LightDataDump / ReadWorldProbe / WgDiag | section 全量读 | 各随宿主路径线程 | 直接 chunk/section 引用 | 同宿主 | BulkWb.java:417-461；Mixin:279-281（LightPalDump）；BenchMod.java:48-49 |

**未发现**：任何「独立 Rust 光照线程直接经 JNI 读 Java 侧 sections」的形态——Rust 侧 light（worldgen-core\src\light\mod.rs）是**纯函数内核**（blocks9/blocks25 int[] 输入由 Java 收集后经 JNI 传入，输出 byte[] 回传），**Rust 不持有也不读 Java ChunkSection**。JNI 边界 = `CppWorldgen.lightCompute / lightComputePacked / lightComputeDomain / lightInit / lightDestroy`（java-core\...\CppWorldgen.java:93-126；versions\1.20.1\rust\src\jni_bridge.rs）。1.21.6 的 `versions\1.21.6\rust` 同构（薄壳）。

---

## 2. 时序交叉图（文字版）：bulk 写回 × light 读

```
chunk C 生命周期（1.20.1，接管形态）：
NOISE 任务(worldgen车道/fill线程)          LIGHT 任务(worldgen车道)           vanilla light 队列
├─ Rust fill → buf                        ├─ Mixin HEAD 接管(LIGHT_RUST)      （gate 关时）
├─ ★BulkWb.writeSections(C) ★写            ├─ R1/R2: 收集 C+8邻 sections ──────── R4: propagateLight(C)
├─ SURFACE(同闭包)                         │    邻门: 邻.status ≥ FEATURES      R5: initializeLight(C)
├─ heightmap 补写                          ├─ JNI lightCompute(Rust 纯函数)           ↑读 C 及邻 sections
└─ 阶段完成 → status 推进(标脏)             ├─ enqueueSectionData(写 nibble)
                                           └─ setLightOn + completedFuture(同步返回)

交叉窗口分析（事实列举，不裁决）：
W1  C 的写回（NOISE）vs C 自己被读（LIGHT/后续）：status 序保证 LIGHT 在 NOISE 之后；
    但 R1/R2 拿 C 是「light(chunk) 传参直接引用」，同步在 LIGHT 任务线程——理论上与 C 写回
    无时间重叠（同链后续），HB 是否成立 = status 推进的内存语义问题（见 OQ-1）。
W2  C 的写回（NOISE）vs C 作为「邻居」被 D 的 LIGHT 收集读：D 的 LIGHT 晚于 C 的 FEATURES
    （status 全序），时间上不重叠——前提同样是 status 门（Mixin:313）对写侧可见的 HB 成立（OQ-1）。
W3  C 的写回 vs vanilla 传播读（gate 关/fallback 时）：R4 传播按 LightStorage 状态可越过 C 的
    状态门槛读邻 chunk 块（ChunkLightProvider:75-76 对任意 loaded chunk）——vanilla 自身形态，
    HB 同样只由 status/票系统承载（OQ-2）。
W4  域批（domainbatch）：收集在提交线程（=R1 同线程同窗口），任务线程只碰快照 ⇒ 本形态
    不新增 reader×writer 面（Mixin:539 注释自证：容器读回提交线程）。
W5  P-β 空节瞬态读计数（Mixin:271/351 WG_BETA_SEC_CUR）：已存在「读到空节瞬态」的观测钩子，
    与「bulk 写回原子性（data volatile / section 引用非 volatile）」的组合语义待 T2 关联。
W6  sentinel 盲区：R1-R5 任何读者不进 SENTINEL_ACTIVE ⇒ 读者×写者在飞重叠即使发生也不 crash。
```

---

## 3. 开放问题清单（供主会话 T2 判定）

- **OQ-1（核心缺口候选）**：R1/R2 的获取途径 = `ChunkProvider.getChunk` 裸引用 + `nc.getStatus().isAtLeast(FEATURES)` 轮询（Mixin:303-318）——**不 join future**。R9-b 论证的「消费者经同一 future 链取 happens-before」是否覆盖「轮询 status 字段 + 读 section」形态？具体取决于：`ProtoChunk.setStatus`（:215-222，普通字段写）与 ChunkHolder future 完成链的 release/acquire 语义是否构成对读者线程的 HB。**这不是性能形态问题（形态审计 L1-L6 已另立），是可见性正确性问题。**
- **OQ-2**：gate 关（vanilla 路径）下 R4/R5 读邻 chunk 的 HB 同样依赖 status 链——若 T2 判 OQ-1 有缺口，需区分「CoreSwap 引入」vs「vanilla 同构既有」（1.20.1 vanilla 传播对本 chunk 读是 light 队列串行内，对邻 chunk 越界读同受 status 门）。
- **OQ-3**：域批开启时 LIGHT 门收紧为「9 中心各自邻域预检」（Mixin:516-526），窗口/并行度与 legacy 臂不同——缺口评估是否需按双开关矩阵（light.rust × domainbatch × oldcollect × blockabi × bulkwb）逐臂声明？
- **OQ-4**：sentinel 是否应扩展读者登记（读者×写者覆盖），或以「读侧登记 + 现有写侧」合成全覆盖检测器——加固成本/门控默认值待议。
- **OQ-5**：P-β 空节瞬态读（W5）与 bulk 发布原子性（PalettedContainer.data volatile、section 引用非 volatile）组合下，R2 `writePacket` 读容器内部（palette/storage 迭代，非单点 volatile 读）是否存在「读到半构造 data」窗口——`readPacket` 完成于发布前（07 篇 R9 :1285 已论证「读者只见旧或新」），但该论证的「读者」当时未枚举 R1/R2。
- **OQ-6**：R9 主线程消费（R8）FULLCHUNK 转换后 chunk 的 INPLAY 期读不在本勘探范围（生成管线期限定了 PROTOCHUNK→FULLCHUNK），INPLAY 期 block event/light checkBlock 增量读未枚举——是否需要扩展？

## 4. 1.20.1 vs 1.21.6 差异标注（快速可见，未深挖）

| 差异点 | 1.20.1 | 1.21.6 | 影响 |
|---|---|---|---|
| 光照 status 结构 | ChunkStatus.INITIALIZE_LIGHT / LIGHT 已存在（mcsrc ChunkStatus.java:158-180） | 同名 status，但经 `ChunkGenerationSteps` 注册链（:36-37），LIGHT `dependsOn(INITIALIZE_LIGHT,1)` | ⚠️ 07 篇 :1524「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 mcsrc 冲突（见 §5 版本疑点）；两版机制同构 |
| 单写者门 | ChunkHolder.futuresByStatus 缓存去重 | AbstractChunkHolder.chunkFuturesByStatus CAS + progressStatus CAS（:220-230，更强） | 1.21.6 写侧门更强；读者侧裸引用形态两版相同（Mixin 同源） |
| bulk 默认值 | BULKWB_ON = 开 | 缺省关（可 -Dcoreswap.bulkwb=1） | 1.21.6 缺省无写回 ⇒ R9-b 缺口实际暴露面 = 1.20.1 生产 + 1.21.6 A/B 臂 |
| 光照 packed 收集 | round3 B/C 已落地 | 未做（#26 帧差，12 篇 :118） | 1.21.6 读者面 = blocks9 直采形态（R1 旧路） |
| 管线证据 | 本 mcsrc（Degraded，版本存疑见 §5） | `.investigations\sentinel-260913-04\pipeline-audit-1216.md`（A1-A6 已核对，confirmed） | 1.21.6 侧 R9-b 覆盖已较硬，但**均未覆盖 light 读者面** |

## 5. 版本疑点（证据定位）

- `.tmp\scout-260905-08\mcsrc\` 标注 1.20.1，但 12 篇 :26 曾有「.tmp/net 源树疑非 1.20.1」排除记录；本勘探引用的 ServerLightingProvider/TACR/ChunkStatus 行号属该树，T2 若引用 HB 论证 MUST 先核对该树真实版本（廉价核对 = 特征行 diff 或 class 版本）。
- `markLightCompletion` 符号在两棵 mcsrc 树中 grep 均无命中——任务背景中的该名称未在参照源找到对应（1.20.1/1.21.6 均以 `chunk.isLightOn()` + `ChunkStatus.java:300` 形态表达「light 完成」判定）；若指其他版本的 API，需外部资料。
- 07 篇 :1524「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 mcsrc ChunkStatus.java:158 冲突（OQ 附带，建议 T2 顺手裁定后按 §15.4 补取代注）。

## 6. 吸收来源清单

- .investigations\form-audit-260915-01\p2-w2-light-rows.md（L1-L6 行集、.b1/.b2/.b3、线程域）
- versions\1.20.1\docs\07-block-pipeline.md :1285（R9 原结论）/ :1496-1529（R9-b 三小节）/ :1531-1566（形态矩阵与 CP 池）
- versions\1.20.1\docs\12-lighting.md（P2 收口、round1-3、形态审计 C 段、G3 归因）
- BulkWb.java / CppBridge.java / CppWorldgen.java / ServerLightingProviderMixin.java（一手源，行号见各表）
- worldgen-core\src\light\mod.rs（纯函数内核；RefCell→Mutex 修改已在源内 :71-77 注记）
- versions\1.20.1\rust\src\jni_bridge.rs（JNI 边界，blocks9/packed/domain 三 ABI）

（scout 状态：draft；不授 candidate/confirmed。）
