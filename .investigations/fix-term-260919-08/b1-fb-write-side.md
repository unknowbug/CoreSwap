---
id: b1-fb-write-side
block: fix-term-260919-08
jurisdiction: F-B 写侧策略（写回数据暂持/后移类；Mixin:590-591 S1/S2 及同族变体）
status: draft
confidence: draft（纯静态方案设计，未实施、未验证）
verification-tier: Degraded（静态审查；LightStorage/ChunkLightProvider/ChunkTicketManager 为 I 级参照盲区）
date-tag: 260919-08
supersedes: none
---

# .b1 辖区产物：F-B「写侧策略」候选方案设计（260919-08，draft）

- 角色：fan-out 候选 worker（.b1），只做方案设计，不改代码、未跑任何命令
- 输入依赖：scout-map（.investigations/fix-term-260919-08/scout-map.md）、T8 verdict（.artifacts/t8-attrib-260919-07/verdict-260919-07.md）、Mixin 一手直读（ServerLightingProviderMixin.java:540-749）、yarn SLP 一手直读（.tmp/light-yarn/ServerLightingProvider.java:90-219）
- 辖区边界：只覆盖 S1/S2 写侧（Mixin:590-591）及其同族暂持/后移变体；**不覆盖** S3 读侧重算（Mixin:594 不 lit）与 propagateLight 补做（.b2 辖区）

## 0. 辖区事实地基（F，一手直读，限定后续论证）

1. 写回值来源 = **提交线程快照**（Mixin:550-552 / :567），域任务线程只做拼帧 + JNI + 逐中心 writeBack（Mixin:601-744）。⇒ **写侧延迟只改变「值何时进入 LightStorage」，不改变「值是什么」**。这是本辖区全部方案的上界定理。
2. 现实现里 9 个中心的 S1/S2 已经在同一任务线程同一循环内连续入队（Mixin:707-715）——即**域内 3×3 邻域对彼此而言「入队即齐」**，不存在域内互等。邻居风险来自：① 相邻**域外/未接管** chunk；② 相邻域的**另一批任务**（不同 domainKey、不同 grace 窗、不同任务轮）。
3. enqueue 底层（yarn SLP.java:117-138）= PRE_UPDATE 阶段消息，pendingTasks ≥1000 触发 runTasks；runTasks 顺序 = PRE 批 → doLightUpdates → POST 批（SLP:195-218）。9 中心 × 48 enqueue = 432 < 1000，单域不触发自清。
4. S3（`chunk.setLightOn(true)`，Mixin:594）在本辖区**所有纯写侧方案中保持原位即时执行**——除非变体显式捆绑 S3 后移（那属 F-A :594-595 接入点，见 §3）。
5. S6 降级臂（wgLightLegacyTakeover，Mixin:829-837）自含一份 S1-S4 副本：**任何写侧暂持方案必须声明降级臂是否同构处理**，否则形成「主臂 hold / 降级臂立即」的分叉时序面。
6. `f.complete(ch)`（Mixin:714）触发下游 status 推进：**若 S1/S2 暂持而 complete 不暂持，下游可能在数据未入库时推进**——写侧延迟与 Mixin:714（F-A 接入点）存在强制耦合面（§2 各候选登记）。

---

## 1. 候选方案

### W1 — 域级整体后移：S1/S2 批量入队改为 POST_UPDATE 阶段单任务

- **机制**（file:line 级）：`wgLightDomainWriteBack`（Mixin:584-596）中，将 24 节 ×2 的 48 次 `provider.enqueueSectionData`（:590-591）替换为**一次** PRE 阶段任务，其体内对 9 中心全部 nibble 调 `super`-级批量入队；或在任务体尾（Mixin:715 循环后）以 yarn light() 尾部形态（SLP.java:179-183）投 POST_UPDATE 任务承载全部 48×9 入队 + （可选，见 §3 边界）S3。等效于把「逐中心立即入队」改为「域任务原子单元后移一拍」。
- **前提依赖**：**独立于 C1 判别**——W1 不改值，只改入库时序；对 C1 两种读法都合法但收益面不同（见 §4 论证）。依赖：POST 阶段任务同样经 `enqueue`→ChunkTaskPrioritySystem（SLP:131-138 的 completedLevelSupplier 取 chunk 完成等级），**摘票后（S4 已执行）等级 supplier 的返回值行为 = 盲区**（I），可能降优先级导致饿死。
- **影响面**：时序——写回入库统一后移一拍，域内原子性更强；内存——快照 byte[]（`out`，WG_DOMAIN_OUT_LEN 量级 ~4.3MB/9 中心）存活期延长一个队列消化周期，可接受；与 S6 交互——降级臂必须同步改为同构后移，否则主/降臂时序分叉（登记为实施前置）。
- **风险与失效模式**：① POST 任务与后续 PRE 任务（如邻居 initializeLight 的 setSectionStatus，SLP:151-163）交错顺序未证（盲区），可能反而引入新排序面；② 若 chunk 在队列消化前被卸载，对已卸载 section 入队行为 = LightStorage 盲区；③ 摘票后优先级饿死（见上）。
- **验收判据草案（预登记）**：
  - preconditions：`key=lightInit, expected=ok, check=SELFCERT`；`key=hookArmed, expected=true, check=domain task 日志行存在`；`key=dllSha, expected=<build 记录值>, check=sha256`；`key=fallbackCount, expected=DEGRADED==0 或登记值, check=计数器`；`key=worldId, expected=<seed+维度项>, check=level.dat`
  - 主判据：n=2 双 run 下 d_self（覆写抖动集）≤ 基线的预登记阈值（形态门沿用 verdict O1 的 ≥80%/≤10% 边界带框架，采集口径 = 档② keep-world）；且 520 存在性差不劣化（新增 unlit 集与基线逐 key 同一）。
  - VOID：任一 precondition 失效或 d_self 采集面不满 → 非零退出（机械信号，§15.4 PI-1 形态）。
- **回退**：单点还原 writeBack 方法体（git revert 该 hunk 即回到 Mixin:588-592 逐节入队），无跨文件状态。

### W2 — 邻域 lit 门控延迟入队（数据暂持，watcher 补投）

- **机制**：writeBack 中 S1/S2 不立即 enqueue，将 (provider, 24×ChunkSectionPos, nibble[]) 存入 per-chunk 暂持结构；以 `CompletableFuture.delayedExecutor` 短窗轮询（或事件回调）检查 3×3 邻域 chunk 全部 `isLightOn()==true` 后，批量执行 48 次 enqueue。域批路径天然满足「域内 9 中心已写」部分；门控真正等待的是**相邻域/未接管 chunk** 的 lit 状态。
- **前提依赖**：独立于 C1（同 W1 不改值）。强依赖盲区：`isLightOn` 消费点全表（scout-map 盲区 #4）与「lit 判定 = 安全入库前提」的等价性未证——`lightOn==true` 不必然意味着该邻居的 section 数据已入 LightStorage（S3 排序偏差在 vanilla 侧也可能存在于域外 chunk？I，未证）。死锁面：邻居永不 lit（520 unlit 族）→ 暂持永不释放 → 必须配上限 + 超时强制入队。
- **影响面**：时序——每个中心写回入库额外等待邻域 lit，最坏 = grace 窗叠加；内存——per-chunk ~96KB nibble 暂持，受域批并发上限约束，量级可忽略；与 S6——降级臂同构登记（同 W1）；与 S4——**摘票先于入库确认**的结构未变（S4 仍即时，Mixin:595），ticket 窗兜底不增强，该面属 F-C 辖区（让渡）。
- **风险与失效模式**：① 暂持期间 chunk 卸载 → nibble 指向失效 provider/section（需 unload 钩子 flush 或丢弃，丢弃 = 回到 T8 固化面）；② 轮询线程与光照线程竞争 enqueue 顺序（enqueue 自身线程安全经 executor.send，SLP:132，风险低但未证）；③ 超时强制入队 = 新增一条「半成品入库」路径，需计数与日志 loud 面。
- **验收判据草案**：preconditions 同 W1 + `key=holdLeak, expected=0, check=暂持未释放计数`；主判据同 W1 主判据 + 新增「超时强制入队计数=0 或 ≤预登记上限」；VOID 同形态。
- **回退**：移除暂持分支（门控默认关，env 开关灰度，如 `coreswap.light.writeside.hold`，遵守 #19 点分命名族 + check_switch_mapping 门禁）；默认路径逐字节等于现状。

### W3 — per-section 分批入队

- **机制**：48 次 enqueue 按 24 节分批（或按 BLOCK/SKY 通道分批）投递。
- **评估**：一手事实（§0.3）显示单域 432 < 1000 不触发 pendingTasks 自清，且 pendingTasks 为有序队列，分批**不改变顺序、不改变值、不改变时机**——对 T8 三个子面（终态化固化 / 覆写抖动 / S3 窗口）**均无机制贡献**。登记为**低价值候选，不建议立项**；列出仅为穷举完整性（scout §4 F-B「per-section 分批入队」变体的闭合）。

### W4 — 写侧整体后移至「域静默点」：complete 与入队同体延迟

- **机制**：S1/S2 暂持（同 W2 门控或简化为固定一拍延迟），且 `f.complete(ch)`（Mixin:714）同体后移至入库确认之后——即「写回完成」语义从「已入队」改为「已入库 + 邻域静默」。
- **边界声明**：Mixin:714 future 完成条件本身是 scout-map §4 F-A 接入点——W4 是 F-B 与 F-A 的**交界候选**，本辖区只登记形态与耦合面，**取舍裁决建议让渡 F-A 辖区或合并评审**。
- **机制价值**：唯一同时封住「下游在数据未入库时推进 status」窗口的写侧候选（§0.6 耦合面的正解）。
- **风险**：complete 延迟直接拉长 LIGHT status 占位时长 → 与 520 unlit 面同向恶化风险（chunk 停服时点更可能早于完成时点）——**与 F-C grace 几何强耦合**，需 F-C 辖区联评；单臂实施此候选风险最高。
- **验收判据草案**：preconditions 同 W2 + `key=520Delta, expected=新增 unlit 集与基线同一, check=档① 形态审计`；主判据 = W1 主判据 ∧ 520 不劣化；VOID 同形态。
- **回退**：同 W2 开关灰度 + 默认关。

---

## 2. 候选汇总表

| 候选 | 改动点 | C1 依赖 | 目标子面 | 建议优先级 |
|---|---|---|---|---|
| W1 域级 POST 后移 | Mixin:584-596（+:715 循环尾） | 独立（不改值） | 覆写抖动暴露窗、域内原子性 | 高（最简单、单点可回退） |
| W2 邻域 lit 门控 | Mixin:590-591 + 新暂持结构 | 独立（不改值） | 覆写抖动（直接对准邻域未 lit 入库） | 中（盲区依赖最重） |
| W3 分批入队 | Mixin:588-592 | — | 无 | 不立项（穷举登记） |
| W4 complete 同体后移 | Mixin:590-591 + :714（F-A 交界） | 独立但受 F-C 几何制约 | §0.6 耦合窗 + 下游推进 | 交界候选，让渡合并评审 |

---

## 3. 「S3 排序偏差在写侧策略下是否被顺带消除」——静态论证

**结论：纯写侧方案（W1 的 S1/S2-only 形态、W2、W3）不消除 S3 排序偏差；仅 W1 的捆绑变体与 W4 消除，且该消除属 F-A 接入点（Mixin:594-595）的机制，非写侧策略本身。**

论证（全部基于 F 事实）：
1. S3 = `chunk.setLightOn(true)` 在域任务线程**即时执行**（Mixin:594），排序偏差 = 「S3 即时完成」先于「S1/S2 经 PRE_UPDATE 队列真正写入 LightStorage」（scout-map §1-S3）。偏差的存在条件只取决于 S3 的执行时机，与 S1/S2 数据**何时**入队无关。
2. W1/W2/W3 都只推迟或重组 S1/S2 的入库时刻：S3 仍在 writeBack 调用点即时执行 ⇒ 「lightOn=true 先于数据入库」的窗口**不减反可能增大**（W1/W2 后移入库，窗口拉长一拍到域队列消化）。即：**写侧延迟在未捆绑 S3 时序时，静态上恶化而非缓解 S3 偏差窗口**。
3. 消除路径只有把 S3 移入与数据入库同队列同阶段的单元（vanilla 形态 = light() 尾部 POST 任务，SLP:179-183：setLightOn 与 releaseLightTicket 在 POST 段、必然晚于同批 PRE 入库）。该改动落在 Mixin:594-595 = scout-map §4 F-A「S3+S4 解绑」接入点。W1 的捆绑变体（POST 任务内含 S3）在机制上等价于 F-A 该条——**归属建议：S3 时序整改的取舍裁决让渡 F-A**，本辖区仅声明 W1 捆绑形态存在且可行。
4. 推论（诚实登记）：若修复目标是「消除 S3 排序偏差」，写侧策略单独不充分；若修复目标是「收窄覆写抖动暴露窗」，写侧策略有效但不触及 S3。两目标须分开验收。

---

## 4. 对 C1 判别结果的依赖结构（上界定理引用）

§0.1 上界定理 ⇒ **本辖区全部候选对 d_cross 2038（终态化固化子面）零贡献**：写回值仍是提交时快照计算结果，后移只改变入库时刻。因此：
- 若 C1 判别结果 = 「恒定 2038 = 确定性内核语义差」成立，写侧策略对其无效（预期内，不构成否决）。
- 若「终态化固化」读法成立，写回值需改用「写回时值」才能修 2038——那是 F-A 写回重验/重算接入点（让渡），写侧策略仍只管 2185 覆写抖动面与时序面。
- 覆写抖动（d_self 2185）收益论证为 **I 级**：其成立前提「vanilla 增量传播会就地改写已入库 nibble」本身是 verdict C3 内联限定的 I 级参照（LightStorage 盲区），故 W1/W2 的主判据必须按 §1 形态门采集后才能升级 candidate。

---

## 5. 诚实登记（盲区 / 外推边界 / 未实施）

1. **未实施**：本文为纯方案设计，零代码改动、零命令运行；全部判据为预登记草案，未采集任何数据。
2. **yarn LightStorage 盲区（I 级参照）引用面声明**：本方案的以下推理环节引用了盲区语义，全部为 I 级、不得当 F 级续推——① POST 任务与后续 PRE 任务的交错顺序；② 摘票后 completedLevelSupplier 对 enqueue 优先级的影响（W1 饿死风险）；③ 邻居 chunk `lightOn==true` ⇔ 其 section 数据已入 LightStorage 的等价性（W2 门控正确性前提）；④ 对已卸载 chunk 入队的行为（W1/W2 失效模式）；⑤ 「覆写抖动由增量传播引起」的机制本身（verdict C3 内联限定）。补齐路径 = ChunkLightProvider/LightStorage 一手 yarn 源入 `.tmp/light-yarn/`。
3. **外推边界**：本设计基于 1.20.1 域批臂；1.21.6 无域批路径、无 S4（scout-map §5），W1/W2/W4 的同构面仅到内联路 Mixin:220-221，**不外推**至 1.21.6。n=2 判据设计沿用 verdict 的 E1 档限定，不外推其他 seed/维度（#162）。
4. **S6 降级臂**：所有候选都要求降级臂同构处理（W1 同步后移 / W2 同暂持），实施清单须显式包含 Mixin:829-837，否则主/降臂时序分叉成为新差异源。
5. **开关纪律**：W2/W4 若实施须走 `-P` 映射门禁（check_switch_mapping.py，#19 点分命名族）。

---

## 6. 辖区外交付纪律（#73 让渡清单，只标归属不解释）

- S3+S4 解绑（setLightOn 经 POST 队列）→ **F-A（Mixin:594-595）**
- propagateLight 补做 → **.b2**
- 读侧重算（S3 不 lit）→ **.b2**
- grace/封板/计时起点/摘票时机（W4 的 520 恶化面联评、ticket 窗兜底）→ **F-C**
- 写回值改「写回时值」（2038 面正解）→ **F-A（Mixin:582-596 重验/重算）**
- C1 备选分解判别实验（档②O1 / 档③E-3a）→ **主会话判别线（verdict §1.1）**
