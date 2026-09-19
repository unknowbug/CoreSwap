# b3 — F-C「grace/ticket 竞态」整改方案候选（260919-08，draft，只做方案设计）

- 角色: fan-out 候选 worker .b3（辖区 = F-C；F-A/F-B 归属其他分支，本文只做耦合序评估）
- 事实地基: T8 归因 confirmed（.artifacts/t8-attrib-260919-07/verdict-260919-07.md §1.2 = 520 chunk unlit，grace(10s) 结构性 ≫ ticket 窗的饱和竞态，档① 520/520 直证）
- 机制地图: .investigations/fix-term-260919-08/scout-map.md §3（一手源: LightDomainBatch.java 全文已直读复核；yarn 侧 TACS:565-566/:684-692 为 F 级摘录）
- 纪律: 未实施、未编译、未运行任何验证；本文全部方案为 draft 设计，接入点存在性 ≠ 可行性
- 一手源状态: `versions/1.20.1/java/src/main/java/wg/bench/LightDomainBatch.java`（134 行全文 F 级）；Mixin:582-596/:601-744 引 scout-map F 级摘录（本分支未重读 mixin 全文，标注外推边界）

---

## 0. 520 unlit 的 F-C 结构成因复述（设计靶心）

grace 计时从**域首中心提交**起算（LightDomainBatch.java:67-68，packed 同构 :92-93），9 中心到齐才提前封板（:81-83/:109-111），grace 到点 `seal(s,true)`（:116-128，幂等 remove :117）。封板后同 domainKey 迟到提交 → `cur==null` 分支重新成立 → **新建 State + 新 grace(10s)**（:65-68）→ 该 chunk 光照完成至少再等一个 grace。ticket 窗（加入 TACS:565-566 / 摘除 Mixin:595→TACS:684-692，经 mainThreadExecutor 再异步一跳）与 grace 窗**无耦合**：chunk 的 LIGHT ticket 在 S4 即被摘除，与其实际 lit 与否无关 → 摘票后 chunk 可被下游推进/卸载，而写回还在 grace/队列里 → 停服/卸载时点早于写回完成 = unlit。观测旁证: domain 臂 457 task 中 115 timedOut——超时封板是常态路径。

---

## 1. 候选方案

### C1 — grace 收敛策略重构（接入点 LightDomainBatch.java:21）

- **机制**: 保留域级共享窗，但把「固定 10s 定时封板」改为**空闲重置窗**（idle-restart）：`delayedExecutor` 回调触发时若 `s.futures.size()` 在窗内仍有增长（或距最后提交 < ε），则重排下一次 grace（重排次数上限 K 防活锁）；或直接调小 GRACE_MS 缺省（10s → 1-2s 量级）配合自证计数观察降级率。
- **file:line**: :21（参数语义扩展，`gracems` → 空闲窗参数族，需过 `check_switch_mapping.py` 门禁——新 `-D` 消费点 MUST 同步 build.gradle 映射，B6-1 门）；:67-68/:92-93（回调体内加重排逻辑）。
- **前提依赖**: 无新外部依赖；K 上限与 ε 需预登记。
- **影响面**: 吞吐——grace 收敛速度直接决定域批收益；收敛更快 → 更多多镇超时封板但**封板时域更满**（空闲重置窗比固定窗收益好：只在真有提交流入时延窗）。9 中心提前封板路径（:81-83）**完全保留**（不动 fast path）。固定调小 GRACE_MS 则域平均 size 下降 → taskFactory 降级重放（Mixin:724 臂）比例上升，域批收益被侵蚀。
- **风险/失效模式**: ① 重排有活锁风险（持续有提交 → 永不封板）→ K 上限硬顶；② 重排逻辑在 `compute` lambda 外操作 State，与 :81-83 提前封板竞争 → 依赖 :117 幂等 remove 兜底（已有）；③ 只治「等太久」不治「封板后重建域」——520 的迟到重建分支（§0）在 C1 下依然重建新 State，只是新 grace 更短。
- **验收判据草案（预登记）**:
  - 主判据（520 存在性差→0 的可观测代理）: **「尾窗 unlit 计数 = 0」**——修复后同 seed 同 workload 的域臂 run 中，`snap 审计 lit 总数 == vanilla 臂 lit 总数`（档①形态同口径，9450 key 集逐 key 存在性对拍），即存在性差 = 0/9450。
  - 副判据: TIMEDOUT/SEALED 比值不劣化 > x%（x 预登记灰区归属交 judge）；降级重放计数 DEGRADED 有界。
  - **preconditions（§15.1 前置集）**: `key=SELFCERT, expected=all-green, check=采集脚本 SELFCERT 硬门（lightInit ok / hook armed / dll sha / fallback 计数 / world 身份项）非零即挂`; `key=VOID, expected=rc==0, check=采集脚本 VOID 通道非零退出 = 判据 suspended`。前置失效 → 判据 suspended、status 不自动变更。
- **回退**: 运行时 flag 回旧语义（`gracems=10000` + 重排开关 `-D` 置 off）；代码回退单 commit。

### C2 — 逐 chunk 独立窗（接入点 LightDomainBatch.java:67-68 / :92-93）

- **机制**: 计时起点从「域首中心提交」改为「逐 chunk deadline」：每个中心提交时记自身 deadline（submit 时刻 + CHUNK_WAIT_MS）；域封板条件 = 9 到齐（不变）或「当前时刻 ≥ 已提交中心中**最早**的 deadline」（即任何一个 chunk 等到自己的上限就封板整个域，已提交者随任务产出，未提交者走 §0 迟到路径）。效果 = 域内任一 chunk 的最大等待有界，不再被「域首早鸟」拖满 10s。
- **file:line**: State 增 `firstSubmitNanos`/`minDeadline` 字段（:37-56）；:67-68/:92-93 的 delayedExecutor 起点改锚到**最近一次提交**（或保留双窗取先到）；:81-83/:109-111 封板条件加 deadline 分支。
- **前提依赖**: CHUNK_WAIT_MS 参数（新 `-D`，同 B6-1 门）；「任一 chunk deadline 到 → 封全域」的语义选择需 judge 确认（另一形态 = 到点 chunk 单独走降级重放 future 完成、域继续收——该形态侵入 taskFactory 契约，风险更大，本候选不取）。
- **影响面**: 吞吐——饱和场景（115/457 timedOut = 提交流速慢于 10s）下域在「最早 deadline」封板，平均域 size 介于「固定 10s」与「无 grace」之间；9 中心 fast path 保留性 = **完整保留**（9 到齐先于任何 deadline 时行为不变）。Saturation 竞态面（首中心等满 10s）被结构性消除。
- **风险/失效模式**: ① 「最早 deadline 封全域」对慢提交者仍是重建域（未到者走迟到路径）→ 不完全消 §0 结构，只缩窗；② deadline 计时用提交线程时钟，跨线程语义（nanoTime vs wall）需登记；③ 与 C1 相同为「缩窗」而非「消重建」。
- **验收判据草案**: 主判据同 C1（存在性差 0/9450，代理形态同）；增判据 = 「域首提交到封板时延 P50/P95 下降且 ≥ CHUNK_WAIT_MS 上界成立」（逐域 timestamps，来自 §3 trace 提案）。preconditions 同 C1（SELFCERT 硬门 + VOID 非零退出）。
- **回退**: deadline 分支参数置 0 = 退化为现行为（deadline=∞），天然 flag 回退。

### C3 — seal 迟到并入：消「封板后重建域 + 二次 grace」（接入点 LightDomainBatch.java:116-128）

- **机制**: 封板（:117 remove 成功）后，同 domainKey 的新提交不再走 `cur==null` 重建路径（:65-68），而是**立即封板的单中心 State**：`submit` 检测「本 key 已封板过（sealedKeys 存在性标记）」→ 构造 size=1 的 State 直接 `seal(st,false)`（非 timedOut，但 taskFactory 侧对 size<9 的域走既有降级 per-chunk 重放臂 Mixin:724/829-837）→ 该 chunk 的光照 = 一个任务投递延迟，**零 grace 等待**。等价表述: 迟到中心并入既有「任务生产线」而非新开 grace 窗。
- **file:line**: :58（增 `ConcurrentHashMap<Long,Boolean> SEALED_KEYS` 或 State 内 sealed 标志 + DOMAINS.remove 后写入）；:64-72（compute lambda 分支：cur==null 且 key 已 seal → 单中心即刻 seal 路径）；:116-128（seal 不变，幂等性由既有 remove(key,st) 保持；单中心 State 走同一 taskFactory）；packed 同构 :88-113。
- **前提依赖**: taskFactory（Mixin:503-505）对 size<9 State 的既有行为已存在（timedOut 域本就 <9 中心，降级臂按现役 per-chunk 路径重放，Mixin:724）——**无需新 mixin 逻辑**，LightDomainBatch 侧纯 Java 改动，影响面封闭。需确认降级臂对「size=1 且非 timedOut」的 State 无隐含假设（外推边界: 本分支未逐行复核 Mixin:601-744，此确认让渡 F-A 辖区/主会话）。
- **影响面**: 吞吐——**正向**：520 的结构性成因（二次 grace）被直接消除；迟到 chunk 不再等 10s。9 中心 fast path 完整保留（未封板域行为零改动）。单中心即时封板略增任务投递次数（每次一个 taskFactory 调用），但降级臂本就 per-chunk，量级不变。
- **风险/失效模式**: ① SEALED_KEYS 无界增长（域 key = 坐标函数，长运行服务器泄漏）→ 需 LRU/有界结构或随 chunk 卸载清理（候选形态：ticket 摘除点回调清理，与 C4 联动）；② 单中心路径与并发提交竞争（两线程同时迟到）→ putIfAbsent 去重（:73）已保证 future 唯一，seal 幂等（:117）保证任务单次；③ 若下游依赖「timedOut=true 才走降级」的隐含约定则 size=1 非 timedOut 域可能走错臂——需 mixin 侧确认（同前提依赖②）。
- **验收判据草案**: 主判据同 C1（存在性差 0/9450）；增判据 = 「DUP 后二次 grace 计数 = 0」——新增自证计数 `RESEALED`（封板后同 key 重建 State 次数），修复后 RESEALED=0 且 SELFCERT 成对正负证据（#118 形态：修复前同 workload RESEALED>0 为负臂）。preconditions 同 C1。
- **回退**: sealedKeys 检查分支由 flag 门控（`coreswap.light.domainbatch.latesubmit=merge|rebuild`），rebuild = 现行为。

### C4 — 摘票时机延迟：ticket 窗兜底（接入点 ServerLightingProviderMixin.java:595，yarn TACS:684-692）

- **机制**: S4（releaseLightTicket）从「写回入队后即时执行」延迟到「写回值确认入 LightStorage 且域内邻域静默」之后；S3（setLightOn(true)）同步后移复刻 vanilla PRE→POST 队列序（yarn SLP:179-183）。效果：LIGHT ticket 存活期覆盖 chunk 的「未 lit」期 → 下游 status 推进/卸载被 ticket level 持有（TACS:565-566 语义），**结构性保证「chunk 不会被推进/卸载到 unlit 状态」**——即使 grace 侧仍有迟到重建（C1-C3 都不完全消的场景），ticket 兜底使其不停服 unlit。
- **file:line**: Mixin:590-595（S1-S4 重排：S1/S2 入队 → 经 light 队列 POST 段回调执行 S3+S4，形态对齐 yarn SLP:195-217 阶段序）；与 F-A 辖区接入点 **Mixin:594-595 完全同一行对**（scout-map §4 F-A「S3+S4 解绑」条目即此）。
- **前提依赖**: ① light 队列 POST 段回调的挂接点（enqueue 完成通知）在 vanilla SLP 内部——需确认 `enqueue` 的 future/回调形态（scout-map: chain 到 ChunkTaskPrioritySystem 消息为止，**回调存在性未证**，盲区①）；若无现成回调需在 ChunkTaskPrioritySystem 消息处理侧 mixin（新接入点，风险升档）。② 失败臂对称：写回异常/降级路径必须也最终摘票（yarn TACS:676 失败臂形态），否则 ticket 泄漏 = chunk 永久持有。
- **影响面**: 吞吐——ticket 持有期延长 → chunk 更晚可被 FULL 推进/卸载 → 峰值 loaded-chunk 压力上升；但域批主收益（批量 JNI/传播）不受影响。9 中心 fast path 保留性：不受 LightDomainBatch 影响；但 S3+S4 后移使「lit 宣言」整体延后一个队列消化延迟（通常 ≪ grace，量级 ms 级 vs 10s）。
- **风险/失效模式**: ① ticket 泄漏（失败臂漏摘）→ 内存/推进卡死，**失效模式比 unlit 更重**；② mainThreadExecutor 再异步一跳（TACS:684-692）的时序仍不受我们控——摘票「延迟后」与写回完成的相对序需 POST 回调内串行保证；③ 与 vanilla 卸载路径（盲区：ChunkTicketManager 内部）交互未证。
- **验收判据草案**: 主判据同 C1（存在性差 0/9450——本候选直接针对「未 lit 被推进/卸载」）；增判据 = 「unlit chunk 的 LIGHT ticket 在停服时刻存活 = 100%」（trace 提案 §3 采样，即兜底有效性直证）+ ticket 泄漏负证据（运行结束后活 ticket 计数回落基线）。preconditions 同 C1 + 新增 `key=ticket-leak-check, expected=回落, check=停服后 ticket 计数对拍`。
- **回退**: S3+S4 重排由 flag 门控回即时执行（= 现行为）；本候选不回退则 C1-C3 独立可退。

### 候选取舍建议（draft，供 judge 对比）

- **C3 为主攻**：直接消除 §0 的结构性成因（封板后重建 + 二次 grace），改动封闭在 LightDomainBatch.java，与 mixin 契约解耦，吞吐正向。
- **C4 为兜底**：把「存在性差」从竞态问题变成 ticket 语义问题（不可推进/卸载 while 未 lit），但接入点与 F-A 重叠且依赖未证回调形态（盲区①），风险档更高。
- **C1/C2 为调参辅助**：都只缩窗不消重建，单独采用不能使主判据达 0（迟到分支仍在）；可作为 C3 的补充（C3 后 timedOut 域收敛更快）。
- **组合建议**: C3（+可选 C1 空闲窗）先行；C4 与 F-A 的 S3+S4 解绑**合并评审**（同一行对，见 §2）。

---

## 2. discovery #150 耦合序评估（F-C × F-A/F-B）

**#150 原则**（workflow-patterns.md:2585）：事实串行化会掩盖其下方的并发/值缺陷；**解除串行化的修复与被掩盖缺陷修复 MUST 同批**；「防御性修复可先行、解粘不得先行」。

**F-C 侧谁是「解除串行化」**: C3/C1/C2 的本质 = **让 520 类 chunk 更快 lit**（消二次 grace / 缩窗）。当前 unlit 状态对该 520 chunk 而言是 F-A 终态化固化缺陷的**天然掩蔽**——unlit chunk 无值可固化（存在性差而非值差）。F-C 修复落地而 F-A 未修 → 这 520 chunk 从「unlit（存在性差）」转为「lit 且固化错值（并入 d_cross 值差面）」→ **总错误不降、仅换形态**，且换到更难观测的值差面（存在性差 0/9450 达标，值差反而可能上升）。

**C4 与 F-A 的物理重叠**: C4 接入点 = Mixin:595 = F-A「S3+S4 解绑」的同一行对（Mixin:594-595）。两者必须**同一 patch 设计**（一次重排同时满足 F-A 的 vanilla 时序复刻与 F-C 的 ticket 兜底），分批做 = 同两行改两次 + 两次回归。

**结论（修复序建议，draft）**:
1. **F-A 值修复与 F-C-C3 MUST 同批**（#150 判据直配：F-C 解蔽 → F-A 缺陷暴露面扩大）。若资源强制分批，唯一合规序 = **F-A 先、F-C 后**（防御性修复先行——F-A 单独落地只是修值不改时序，不产生新暴露面；反向序 = 解粘先行，违 #150）。
2. **C4 并入 F-A 的 S3+S4 解绑同 patch**（同 行对，无独立批次可言）。
3. F-B（覆写面）与 F-C 无直接耦合（F-B 治「已 lit 后被邻居改写」，F-C 治「何时 lit」），可独立批次；但 C4 的「邻域静默」判据依赖 F-B 的边界带形态门（O1）结果，建议 F-B 判别实验（档②O1）先行或并行。

---

## 3. 520 逐 chunk 时序归属 — 廉价 trace 提案（≤1 轮，scout 盲区③闭合用）

**目标**: 把 520 成员逐一归属到三态——(a) 域内正常路但写回晚于停服 / (b) 封板后迟到提交→重建域（§0 结构） / (c) 提交从未发生。支撑验收判据（「存在性差→0 代理」的机理层证据与 C3 判据 RESEALED 的正负臂）。

**采集设计（现有 T8 台 + keep-world，零/最小改码）**:
1. **零改码面（先采）**: 现有 SELFCERT 行 + LightDomainBatch 自证计数（SEALED/TIMEDOUT/DUP，:131-133）+ T8 台既有 snap 审计（lit/unlit 集）在 keep-world 下停服后保留 → 对拍 unlit 集与 TIMEDOUT 域的空间包含关系（unlit chunk ⊆ timedOut 域？——形态门，若成立即支撑 (b) 为主因；域 key 由 chunk 坐标反推 floorDiv(cx,3),floorDiv(cz,3) 可从 snap 坐标离线算，零运行时成本）。
2. **最小改码面（若 (1) 不足）**: 单条 trace 行（chunk 级、env 门控、chunk 判定一次——诊断污染铁律）：`submit(pos, domainKey, stateCreatedFresh?, futuresSizeAtSubmit)` 于 :73 后 + `writeback(pos, t)` 于 Mixin:713 侧。停服时刻 `t_stop` 由脚本记录。归属判读（预登记三态映射，#150 家族判据同稿）：`无 submit 行 → (c)`；`有 submit 且 stateCreatedFresh=true → (b)`；`有 submit 且 writeback.t > t_stop → (a)`；`writeback.t ≤ t_stop 却 unlit → 新机制候选（升级，不在本方案解释域）`。
3. **keep-world 增益**: 同一 world 二次启动重跑 → 逐 chunk 两次归属对比，检验「跨 run 恒定」前提（verdict §4 未决声明）在时序维度的一致性。
4. **preconditions**: SELFCERT 硬门 + VOID 非零退出同 §1；灰区（writeback.t 与 t_stop 差 < 采样分辨率）预登记归属「灰区-停服竞态」，计数单列交 judge。

**成本**: (1) 纯离线 ≤ 半轮；(2) 两行埋点 + 一轮采集。总量 ≤1 轮成立。

---

## 4. 诚实登记

- **未实施**: 本文档全部候选未写一行代码、未编译、未运行；验收判据未与采集脚本定稿同稿（#150 判据 5：落地时 MUST 与采集脚本同批定稿）。
- **盲区**: ① LightStorage/ChunkLightProvider/ChunkTicketManager yarn 一手源缺失（C4 的回调存在性、ticket 内部调度、卸载交互均为 I 级参照）；② Mixin:601-744 任务体未逐行重读（C3 前提依赖②「size<9 非 timedOut 无隐含假设」未闭合）；③ setLightOn 消费点全表在缺失面内（C4 影响面半径未闭合）；④ 520 逐 chunk 归属未做（§3 提案闭合之）。
- **外推边界**: n=2 seed 不外推其他 seed/维度；1.20.1 域批形态不外推 1.21.6（ticket 机制整体移除，scout-map §5——C4 在 1.21.6 无对应物，C1-C3 的 LightDomainBatch 侧同构性亦未查，标注为独立工作）。
- **辖区外交付让渡清单**: ① C3 前提②与 C4 前提②（mixin 任务体/失败臂复核）→ F-A 辖区/主会话；② F-A/F-B 判别实验（档②O1/档③E-3a）→ 各自辖区；③ GRACE_MS 族新 `-D` 参数的 build.gradle 映射与 `check_switch_mapping.py --strict` 门 → 主会话工程面；④ trace 提案的运行时执行 → 主会话（subagent 无 shell）；⑤ confirmed 授予 → 用户。
