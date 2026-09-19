# 写回终态化通路 + grace/ticket 机制地图 — scout-map（260919-08，draft，只读勘探产物）

- 角色: scout（recode-scout 形态，静态只读；本文件 = 唯一写产物）
- 来源课题: T8 归因 confirmed（.artifacts/t8-attrib-260919-07/verdict-260919-07.md）——F-A 终态化固化 / F-B 覆写抖动 / F-C grace-ticket 竞态三子面的修复方案设计前置事实地基
- 纪律: 零命令、零修改既有文件；F = 一手 file:line 直读，I = 推断（显式标注）；yarn 缺失面标 I 级参照/盲区
- 一手源面声明: `.tmp/light-yarn/` 存在 6 个 yarn 一手源（ServerLightingProvider / LightingProvider / ThreadedAnvilChunkStorage / WorldChunk / ChunkNibbleArray / ChunkStatus）——**ChunkLightProvider / LightStorage / ChunkTicketManager 源不在其中**，相关面标 I 级参照/盲区

---

## 1. 写回终态化点（Mixin:582-596 全副作用清单）

注入点所在类: `wg.bench.mixin.ServerLightingProviderMixin`（@Mixin(ServerLightingProvider.class)，Mixin:67）。终态化代码 = `wgLightDomainWriteBack`（Mixin:582-596），由域批任务体 `wgLightDomainTaskRun`（Mixin:601-744，跑在 `Util.getMainWorkerExecutor()` 线程，Mixin:504-505）对每个中心逐 chunk 调用（Mixin:713）。同构副本也存在于现役内联路 `wgLightLegacyTakeover`（Mixin:829-837）。

**方法体内全部副作用清单（F）**：

| # | 副作用 | 位置 | 机制 | 与「免全量重光」读法的关系 |
|---|---|---|---|---|
| S1 | `provider.enqueueSectionData(BLOCK, sp, nibble)` ×24 节 | Mixin:590 | 经 ServerLightingProvider.enqueueSectionData（yarn ServerLightingProvider.java:117-125）→ `enqueue` 投 `ChunkTaskPrioritySystem` 消息（:131-138）→ 真正写入 LightStorage 发生在光照线程 `runTasks` 的 **PRE_UPDATE** 段（:195-208）——**异步、非本任务线程同步完成** | 写回值进入 vanilla LightStorage 的唯一通道；值 = 提交时 blocks9/ packed 快照的 Rust 计算结果（快照在 light 调用线程收集，Mixin:562-571），不是终态世界值 → 若「免全量重光」读法成立，此值被固化 |
| S2 | `provider.enqueueSectionData(SKY, sp, nibble)` ×24 节 | Mixin:591 | 同 S1，SKY 通道 | 同 S1 |
| S3 | `chunk.setLightOn(true)` | Mixin:594 | **在本域批任务线程立即执行**（不投 POST_UPDATE 队列）。复刻 vanilla `light()` 尾部（yarn ServerLightingProvider.java:179-183），但 vanilla 该句在 POST_UPDATE 段、经 `runTasks` 与 PRE 写回**同队列顺序保证**（先 PRE 后 POST，yarn :200-217） | 直接的「免全量重光」开关：`lightOn=true` 后下游 status 推进不再触发该 chunk 全量重光（I 级参照：消费点在 TACS/ChunkHierarchy，yarn 源不全）。⚠️ **本处与 vanilla 存在排序偏差（F 推 I）**：S1/S2 是排队任务，S3 是即时执行——存在 `setLightOn(true)` 先于 section 数据实际入 LightStorage 完成的窗口，vanilla 无此窗口 |
| S4 | `tacs.wgReleaseLightTicket(chunkPos)` | Mixin:595（Invoker: ThreadedAnvilChunkStorageAccessor.java:13-14）| → TACS.releaseLightTicket（yarn ThreadedAnvilChunkStorage.java:684-692）→ `mainThreadExecutor.send(removeTicketWithLevel(LIGHT,...))`——**再异步一跳**（主线程执行器）| 摘除 LIGHT ticket（ticket 在 LIGHT status 申请时加入，yarn TACS:565-566）→ chunk 不再被 LIGHT 级持有，可继续向 FULL 推进/满足卸载条件。与「免全量重光」关系：摘票后 vanilla 不会因 ticket 机制回头重光该 chunk；S3+S4 成对构成终态化宣言 |

**任务体级补充副作用（wgLightDomainTaskRun，非 writeBack 方法体但同通路，F）**：
- S5 `f.complete(ch)`（Mixin:714）：写回成功后完成 future → 下游 ChunkStatus 管线推进的触发点。时序上在 S1-S4 同一任务线程、S1/S2 入队之后。降级臂改为 `wgLightLegacyTakeover`（Mixin:724）或 `completeExceptionally`（Mixin:728-729）。
- S6 降级重放臂内部自含一份 S1-S4 副本（wgLightLegacyTakeover Mixin:829-837）。
- 与 C1 备选读法的关系：S1/S2 的值来源（提交线程快照，Mixin:567/550-552）同样是「确定性内核语义差」读法的候选入口——S1-S4 本身对两种读法中性，判别归档②/③（verdict §1.1）。

## 2. LightStorage 接入面

**写侧调用链（F，逐层有源）**：
```
Mixin:590-591 provider.enqueueSectionData(LightType, ChunkSectionPos, nibble)
  → ServerLightingProvider.enqueueSectionData  (yarn SLP.java:117-125) enqueue→executor.send
  → ChunkTaskPrioritySystem 消息 → 光照线程 runTasks PRE_UPDATE 段执行
  → LightingProvider.enqueueSectionData        (yarn LP.java:121-129)
  → ChunkBlockLightProvider/ChunkSkyLightProvider.enqueueSectionData(long, nibble)
  → LightStorage（★ 盲区：.tmp/light-yarn/ 无 ChunkLightProvider.java / LightStorage.java）
```
- runTasks 阶段顺序（F，yarn SLP.java:195-217）：PRE_UPDATE 批执行 → `super.doLightUpdates()`（:208）→ POST_UPDATE 批执行。**写回排队任务与 vanilla light() 尾部 POST 同队列**；域批 mixin 的 S3/S4 脱离该队列（§1 S3）。
- `doLightUpdates`（yarn LP.java:43-54）→ 各 provider 的增量传播引擎——后加载邻居触发的 vanilla 增量传播从 `checkBlock`/`setSectionStatus`/`propagateLight` 进入同一 LightStorage。**入口链到 ChunkLightProvider 为止有源；LightStorage 内部如何对已写 nibble 就地改写 = I 级参照/盲区**（F-B 覆写面的 vanilla 增量传播语义 judge C3 已内联限定）。
- 邻居后加载传播触发点（I 级参照）：`propagateLight(ChunkPos)`（yarn LP.java:79-87，server 侧包装 SLP.java:97-104）由 TACS 在 chunk status 推进时调用（TACS:565-566 附近管线，调用点全表在盲区）；`updateChunkStatus`（yarn SLP.java:69-83）对新 chunk 清空 section 数据 + 标 notReady——这是后加载邻居改写已写回值的最可能结构入口（I，未直证）。

## 3. grace 与 ticket 生命周期

**grace(10s)（F，全链有源）**：
- 计时源: `GRACE_MS = Integer.getInteger("coreswap.light.domainbatch.gracems", 10_000)`（LightDomainBatch.java:21）。
- 起点假设: 域 State **首次** submit 时 `compute` 内注册 `CompletableFuture.delayedExecutor(GRACE_MS).execute(() -> seal(s, true))`（LightDomainBatch.java:67-68，packed 同构 :92-93）——计时从域第一个中心提交起算。
- 终点: ① 9 中心到齐即 `seal(st,false)`（:81-83 / :109-111，提前于 grace）；② grace 到点 `seal(s,true)`（timedOut）——封板幂等由 `DOMAINS.remove(key,st)` 保证（:117）。
- 封板后: `taskFactory`（= mixin 注册的 `wgLightDomainTaskRun`，Mixin:503-505）执行 → 逐中心 S1-S6。
- **封板后迟到提交**（F 推 I，F-C 几何相关）：seal 已 `remove` DOMAINS 条目 → 同 domainKey 新提交会**新建 State + 新 grace 计时**（compute 的 cur==null 分支重新成立，:65-68）→ 该 chunk 的光照完成至少再等一个 grace 或新域凑满 9 中心。
- 观测旁证（F，上块 scout-map §1.3）：domain 臂 457 task 中 115 条 timedOut=true——grace 超时封板是常态路径非异常路径。

**ticket 生命周期（F，链到 mainThreadExecutor 为止）**：
- 加入: chunk 进入 LIGHT status 时 `ticketManager.addTicketWithLevel(ChunkTicketType.LIGHT, pos, ...)`（yarn TACS:565-566）。ChunkTicketManager 内部 = 盲区。
- 移除: `releaseLightTicket`（yarn TACS:684-692）→ mainThreadExecutor 异步 remove；两个调用点：① vanilla/域批 mixin 的 light 尾部（yarn SLP:181 / Mixin:595、:837）；② 生成失败臂（yarn TACS:676）。
- **时序关系（F 推 I）**：ticket 窗 = ticket 加入（LIGHT status 进入）→ S4 摘票生效（主线程执行）。grace 窗 = 域首中心提交 → 封板。两窗**无耦合**：域批下 chunk 的 light() 调用线程只是提交快照即返回 future（Mixin:559/574），chunk 实际 lit 要等全域 grace 收敛 + 任务执行 + S1/S2 队列消化——grace(10s) ≫ 单 chunk vanilla light 处理窗，520 unlit 面的代码层对应结构 = 「chunk 停服时点早于其所在（迟到重建的）域任务写回 + 队列消化完成时点」（精确到每个 520 成员的归属需逐 chunk 时序 trace，本勘探未做，I）。

## 4. 修复方案候选接入点清单（不做取舍）

### F-A（写回时机 / 终态化条件）
| 接入点 | 机制描述 |
|---|---|
| Mixin:582-596 `wgLightDomainWriteBack` | 写回前对终态世界重验/重算（把「提交时快照」换成「写回时值」）；或把 S3 推迟到域级静默确认后 |
| Mixin:594-595 S3+S4 对 | 拆开 S3/S4 与 S1/S2 的绑定：setLightOn 延迟到 S1/S2 真正入 LightStorage 之后（经 POST_UPDATE 队列复刻 vanilla 时序，yarn SLP:179-183 形态），消除 §1-S3 排序偏差 |
| Mixin:714 `f.complete(ch)` | future 完成条件从「写回已入队」改为「域内全部中心写回完成 + 邻域静默」，控制下游 status 推进触发重光与否 |
| Mixin:856-892 `wgLightRustTakeover` HEAD | 不 cancel vanilla light() 的 PRE/POST 外壳（保留 yarn SLP:171-184 全语义），仅替换传播内核——终态化交还 vanilla |

### F-B（写侧延迟 vs 读侧重算）
| 接入点 | 机制描述 |
|---|---|
| Mixin:590-591 S1/S2 | 写侧延迟：section 数据暂持（不 enqueue），待 3×3 邻域全部 lit 后再入队（等价于把域批写回整体后移） |
| Mixin:594 S3 | 读侧重算：不 setLightOn(true)（或设 false），让 vanilla 管线在后加载邻居到达时按未 lit 语义全量/增量重光该 chunk |
| yarn SLP:97-104 `propagateLight`（复刻调用点 Mixin 侧新增） | 写回后主动排程 vanilla propagateLight 补传播（对齐 vanilla light() PRE 段 :174-178 被跳过的 `super.propagateLight(chunkPos)`）——现实现 cancel 后此步从未发生（F：Mixin:59-60 注释自认「跳过 vanilla propagateLight」）|
| LightStorage 层（★盲区） | 邻居加载时对已写 chunk 的 section 重新 enqueue 数据——需 LightStorage/ChunkLightProvider 一手源补齐后才能定位 file:line（I 级参照，接入点存在性未证）|

### F-C（grace/ticket 竞态）
| 接入点 | 机制描述 |
|---|---|
| LightDomainBatch.java:21 GRACE_MS | grace 时长/封板策略重构（如按 ticket 窗对齐、或 grace 内迟到中心触发重封板）|
| LightDomainBatch.java:67-68 / 92-93 计时起点 | 计时起点从「域首中心提交」改为「逐 chunk 独立窗」，消除域级共享窗的饱和竞态面 |
| LightDomainBatch.java:116-128 `seal` | 封板时未到齐中心的处理路径：现行为 = 未提交者无 future（不产出）；可改为迟到中心并入既有任务（消「封板后重建域 + 二次 grace」结构）|
| Mixin:595 S4（yarn TACS:684-692） | 摘票时机：延迟 releaseLightTicket 至写回值确认入 LightStorage 且邻域静默，用 ticket 窗兜底未 lit chunk 不被推进/卸载（yarn TACS:676 失败臂已示范摘票与 chunk 生命期挂钩形态）|

## 5. 1.21.6 同构面（仅标注）

- 同构写回点**存在**：`versions/1.21.6/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:220-221`（enqueueSectionData ×2）+ **:224-227（仅 `chunk.setLightOn(true)`）**——1.21.2 管线重构后 light ticket 机制整体移除（releaseLightTicket 全源码零命中、TACS 更名 ServerChunkLoadingManager，:225-226 注释自证）→ **1.21.6 无 S4 副作用，S1-S3 同构**。
- 域批路径（wgLightDomainWriteBack / wgLightDomainSubmit）在 1.21.6 源码树 **无命中**——域批为 1.20.1 专属形态，F-A/F-C 的域批面在 1.21.6 无直接对应物（grep 证据：仅 :45 注释、:53 类声明、:220-221/:224-227 内联路命中）。

## 6. 接入点清单表 + 盲区/未决清单

### 接入点汇总表
| 子面 | 接入点 | file:line | 一句机制 |
|---|---|---|---|
| F-A | 写回终态化方法 | ServerLightingProviderMixin.java:582-596 | 写回值改用写回时终态世界重验/重算 |
| F-A | S3+S4 解绑 | ServerLightingProviderMixin.java:594-595 | setLightOn 经 POST 队列复刻 vanilla 时序 |
| F-A | future 完成条件 | ServerLightingProviderMixin.java:714 | 域级静默后才 complete 控制下游推进 |
| F-A | HEAD 不 cancel 外壳 | ServerLightingProviderMixin.java:856-892 | 终态化交还 vanilla light() 全语义 |
| F-B | S1/S2 写侧延迟 | ServerLightingProviderMixin.java:590-591 | 数据暂持待邻域 lit 再入队 |
| F-B | S3 读侧重算 | ServerLightingProviderMixin.java:594 | 保持未 lit 让 vanilla 邻居到达时重光 |
| F-B | 补 propagateLight | (新增，复刻 yarn SLP.java:97-104) | 写回后主动排程被 cancel 掉的传播步 |
| F-B | LightStorage 层重灌 | ★盲区 | 需一手源补齐 |
| F-C | grace 参数/策略 | LightDomainBatch.java:21 | grace 与 ticket 窗对齐重构 |
| F-C | 计时起点 | LightDomainBatch.java:67-68, 92-93 | 域级共享窗改逐 chunk 独立窗 |
| F-C | seal 迟到处理 | LightDomainBatch.java:116-128 | 迟到中心并入既有任务免二次 grace |
| F-C | 摘票时机 | ServerLightingProviderMixin.java:595 (yarn TACS:684-692) | 摘票延至写回确认，ticket 窗兜底 |

### 盲区/未决清单
1. **LightStorage / ChunkLightProvider / ChunkTicketManager yarn 一手源缺失**（.tmp/light-yarn/ 仅 6 文件）——F-B 覆写面的 vanilla 增量传播内部语义、F-B LightStorage 层接入点、ticket 窗内部调度均为 I 级参照/盲区（与 verdict §4 降级声明一致）。
2. **§1-S3 排序偏差**（setLightOn 先于 section 入库的窗口）为本勘探新识别的结构事实，其对 F-A/F-B 的贡献未量化（I，待 worker）。
3. **520 成员逐 chunk 时序归属未做**——F-C 几何成因到具体 chunk 的映射需时序 trace，本勘探只给了结构对应（I）。
4. **setLightOn(true) 的消费点全表**（谁读 lightOn 决定免重光）在 yarn 缺失面内，F-A 接入点 S3 的确切语义半径未闭合（I 级参照）。
5. 接入点存在性 ≠ 可行性：全部条目未做方案取舍与风险评估（让渡 worker/fan-out）。
