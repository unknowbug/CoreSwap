---
id: b4-fa-branchA
block: fix-term-260919-08
jurisdiction: F-A 甲分支 = 写回时机/终态化条件（Mixin:582-596 重验/重算、Mixin:594-595 S3+S4 解绑、Mixin:714 complete 条件、Mixin:856-892 HEAD 不 cancel 外壳）
status: draft
confidence: draft（纯静态方案设计，未实施、未编译、未运行、零命令）
verification-tier: Degraded（静态审查；LightStorage/ChunkLightProvider/ChunkTicketManager/setLightOn 消费点 = I 级参照盲区）
date-tag: 260919-08
inputs:
  - .artifacts/t8-attrib-260919-07/verdict-260919-07.md（confirmed）
  - .artifacts/fix-term-260919-08/verdict-260919-08.md（draft/candidate，档②O1 PASS）
  - .investigations/fix-term-260919-08/scout-map.md（§1 S1-S6、§4 接入点清单）
  - .investigations/fix-term-260919-08/b1-fb-write-side.md（W4 让渡承接）
  - .investigations/fix-term-260919-08/b2-fb-read-side.md（FB-1(a)/FB-2 关系评估）
  - .investigations/fix-term-260919-08/b3-fc-grace-ticket.md（C4 同 patch 面承接；#150 修复序）
supersedes: none
---

# .b4 F-A 甲分支候选方案设计：写回时机/终态化条件（260919-08，draft）

- 角色：fan-out 候选 worker（.b4，F-A 甲分支）。只做方案设计，不改代码，未运行任何命令。
- 辖区：写回**时机**与**终态化条件**——针对 2038 恒定面（终态化固化候选机制）+ S3 排序窗口。接入点四组：Mixin:582-596（写回前重验/重算）、Mixin:594-595（S3+S4 POST 解绑）、Mixin:714（complete 条件）、Mixin:856-892（HEAD 不 cancel 外壳）。
- 承接让渡：.b1-W4（complete 同体后移，Mixin:714 交界）、.b2-FB-1(a)（ticket 保留，F-C 硬依赖面）、.b3-C4（Mixin:595 同行对，同 patch 合并）。
- 值来源事实地基（scout-map §1-S1 F 级）：写回值 = 提交线程快照（Mixin:550-552/:562-571）的 Rust 计算结果；S1/S2 经 PRE_UPDATE 队列入库（异步）；S3 在任务线程即时 setLightOn(true)（排序窗口）；S4 即时摘票（再异步一跳）。**本辖区是唯一能改「值是什么」与「终态化何时/以什么条件成立」的辖区。**

---

## 1. 候选方案

### FA-1 — S3+S4 POST 队列解绑（Mixin:594-595；吸收 .b3-C4 + .b2-FB-1(a)）

- **机制（file:line 级）**：将 `wgLightDomainWriteBack`（Mixin:584-596）内的 S3（`chunk.setLightOn(true)`，:594）与 S4（`tacs.wgReleaseLightTicket(chunkPos)`，:595）从「域任务线程即时执行」改为**经光照队列 POST_UPDATE 段执行**（形态 1:1 复刻 vanilla `light()` 尾部 yarn SLP.java:179-183——vanilla 中 setLightOn 与 releaseLightTicket 在 POST 段，与同批 PRE 段 section 入库有队列顺序保证，yarn SLP:195-217 先 PRE 后 POST）。实现形态：S1/S2（:590-591）保持 PRE 入队不动；在 :593 前改为投递一个 POST 段任务，任务体内执行 setLightOn + releaseLightTicket。同构副本同步应用于降级臂 `wgLightLegacyTakeover`（Mixin:829-837，S6 纪律）。**本候选即 scout-map §4 F-A「S3+S4 解绑」条目的正体，同时是 .b3-C4（摘票延迟至写回确认+邻域静默）与 .b2-FB-1(a)（保留 LIGHT ticket 兜底）的合并 patch 面**——三者在 :594-595 同一行对上，分批做 = 同两行改两次 + 两次回归（b3 §2 结论原样采纳）。
  - C4 吸收的具体语义：POST 任务体内先 setLightOn 后 releaseLightTicket，且 releaseLightTicket 的执行时点由「写回入队后即时」变为「section 数据已真正入 LightStorage 之后」（PRE→POST 队列序保证）→ LIGHT ticket 存活期覆盖「数据未入库 + 未 lit」期，结构性堵住「摘票后 chunk 被推进/卸载到 unlit」窗口（520 面的 ticket 兜底半边）。
  - 失败臂对称（b3-C4 前提②承接）：写回异常路径（Mixin:728-729 completeExceptionally 臂）必须仍最终摘票（yarn TACS:676 形态），否则 ticket 泄漏——失败臂内保留一次性 releaseLightTicket 调用，登记为实施清单必查项。
- **前提依赖**：
  - **C1 判别（档②/档③）**：中性——本候选不改值、只改时序与摘票条件，对「固化 vs 内核差」两读法均合法。
  - **依赖盲区①（b3-C4 前提①，未证）**：POST 段任务的挂接形态。vanilla POST 批 = ChunkTaskPrioritySystem 的 POST_UPDATE 消息段（yarn SLP:195-217 F 级）；投递接口存在性可静态复核 `enqueue`/`runTasks` 的消息分型（SLP:131-138 一侧有源），**LightStorage/ChunkTaskPrioritySystem 内部排序与摘票后 completedLevelSupplier 行为 = 盲区**（b1 §5-②同面）。若 POST 投递无现成接口 → 需 ChunkTaskPrioritySystem 侧新 mixin（风险升档，登记实施前置确认项）。
  - **依赖盲区②**：releaseLightTicket 延迟后，mainThreadExecutor 再异步一跳（yarn TACS:684-692）的相对序不受控面不变——但 POST 序保证已把「写回确认 → 摘票」串行化到光照线程，比现状（任务线程即时摘、跨两线程无序）严格更优。
- **影响面**：
  - **性能（免全量重光收益保留度）**：**完整保留**——本候选不放弃 lit、不触发任何全量重光，只重排终态化时序；域批「一次 JNI 取代 9 次重光」收益零回退。新增开销 = 每 chunk 一次 POST 任务投递（线性小量）+ ticket 持有期延长（≈ 一个队列消化周期，ms 级 vs grace 10s）。
  - **时序**：S3+S4 整体后移一拍到 POST 段；「lit 宣言」晚于「数据入库」恢复 vanilla 不变量 → **S3 排序窗口（scout §1-S3 排序偏差、盲区#2）被结构性消除**——这是本候选对本辖区命题（S3 窗口）的直接正解。
  - **与 S6 降级臂交互**：Mixin:829-837 同构副本必须同 patch 处理，否则主臂 POST / 降级臂即时的分叉成为新差异源（S6 纪律，b1 §5-4 同款）。
  - **对 520 面**：ticket 兜底半边改善（未 lit 不被卸载）；但**不消**「封板后重建域 + 二次 grace」结构（那是 .b3-C3）——两候选互补不互替。
  - **对 2185 面**：间接——排序窗消除后「lit 时数据未入库」的错位消掉，但覆写主体（邻居后到改写已入库值）不经此候选直接消除；主攻面见 FA-1 与 FB-2 的组合关系（§3）。
- **风险与失效模式**：① POST 投递接口需新接入点（盲区①，实施前静态确认，存在即低险）；② ticket 泄漏（失败臂漏摘）——失效重于 unlit，必须失败臂对称 + 泄漏计数判据；③ POST 段任务与后续 PRE 任务交错序未证（盲区，可能引入新排序面——但优于现状的无序）；④ 邻域静默语义未包含（C4 全语义含「邻域静默」，本候选只做队列序半边——邻域静默门控属 FB-3/W2 族，见 §1-FA-3 边界）。
- **验收判据草案（预登记）**：
  - preconditions（§15.1 形态）：`key=lightInit, expected=ok, check=SELFCERT 硬门`；`key=hookArmed, expected=true, check=[LIGHT-DOMAIN] 行存在`；`key=dllSha, expected=<build 记录值>, check=sha256`；`key=fallbackCount, expected=DEGRADED 登记值, check=计数器`；`key=worldId, expected=<seed+维度>, check=log 身份项`；`key=ticketLeak, expected=停服后活 LIGHT ticket 计数回落基线, check=对拍`（前置失效 → 判据 suspended，status 不自动变更）。
  - 主判据：n=2 keep-world 双 run 下 d_self 显著下降（方向 ≈0，与 FB-2 组合判据合流，§3）∧ lit 总数 = vanilla 臂 lit 总数（9450 同口径，存在性差 0/9450）∧ S3 排序窗直证 = 0（新增埋点：「setLightOn 执行时同 chunk section 已 PRE 入库」断言计数，修复前 >0 为负臂，修复后 =0——b3 §3 trace 提案同稿形态）。
  - VOID：任一 precondition 失效、ticket 泄漏计数非零、lit 总数下降、或出现新 unlit → 非零退出（机械信号，PI-1 形态）。
  - 灰区全轴（#154）：判据灰区声明四轴——① d_self 下降但非显著（阈值灰区交 judge 定归属）；② S3 断言在采样分辨率内的竞态（归属「灰区-队列竞态」，计数单列）；③ ticket 回落基线本身的抖动轴；④ n=2 单 seed 单维度外推边界轴（不外推 #162）。
- **回退**：S3+S4 重排由 flag 门控回即时执行（= 现行为）；命名走 #19 家族：`coreswap.light.writeback.post-finalize`（点分族，1.20.1 现役点分/驼峰并存面上新开关 MUST 过 `check_switch_mapping.py` 门禁 + build.gradle `-P` 映射同步，B6-1 门）。代码回退单 commit。

### FA-2 — 写回前重验/重算：值改用「写回时终态世界」（Mixin:582-596）

- **机制**：`wgLightDomainWriteBack` 对每个中心提交 S1/S2 前（:590-591 之前），对**当前（写回时点）世界状态**重验提交时快照仍有效：低成本形态 = 重算该 chunk 的 blocks/sky 输入指纹（对比提交时快照 hash，Mixin:562-571 同源），不一致或未验证时走重算（再次 JNI，输入取写回时点世界）后再入队。等效把「提交时快照值」替换为「写回时值」——终态化固化的值来源正解。
- **前提依赖**：**硬依赖 C1 判别（档③ E-3a）**——本候选的全部预期收益建立在「2038 恒定面 = 终态化固化」读法上（verdict-07 §1.1 读法甲）。档② O1 只裁决了 2185 覆写面，对 2038 面不裁决（verdict-08 §2.2 明示）；若档③判出「2038 = 确定性 Rust-vs-Java 内核语义差」，本候选**对该面零贡献**（值照样错，只是换成写回时点算出的错值）。⇒ **判据前置集显式携带 `key=c1-e3a-verdict, expected=固化读法成立, check=档③记录在案`，前置失效 → 本候选全部判据 suspended（premise-expired）**，且实施序 MUST 在档③之后（不得抢先实施）。
- **影响面**：
  - **性能（核心约束）**：重验（hash 对比）成本 = 输入收集一次的量级（chunk 级一次，非每点——诊断污染铁律同款约束）；重算（JNI 二次）只在 hash 失配分支发生，理想域批（快照=终态）下零额外 JNI → **免全量重光收益保留度 ≈ 100%**（最坏全失配 = 双倍 JNI，仍 ≪ vanilla 全量重光 9 次/域）。hash 失配率的观测本身是档③ 的廉价前置数据（诚实声明：不是 E-3a 本体，E-3a 是输入 hash 跨 run 直证）。
  - **时序**：写回线程内串行追加，无新跨线程面。
  - **与 S6**：降级臂 Mixin:829-837 同构重验（同纪律）。
- **风险与失效模式**：① 「重算取写回时点世界」期间世界又变（TOCTOU 递归）——理论存在，工程上收窄到「重算值 ≥ 提交时值新鲜」，诚实登记不承诺终态严格性；若需严格终态须与 FA-3 域静默条件耦合；② Rust 输入收集接口是否支持按需二次调用（Mixin:562-571 路径复用性）未逐行复核（本分支对 Mixin:540-600 为 scout F 级摘录引用，未全文重读——外推边界，实施前确认项）；③ 若 C1 备选成立则白付 hash 成本（量级小）。
- **验收判据草案**：preconditions 如上（含 c1-e3a-verdict 硬前置）。主判据：n=2 下 d_cross（恒定面 key 集）较 2038 基线显著下降（目标 ≈0）且 d_self 判据与 FA-1/FB-2 分账不混报；副判据 = hash 失配率分布落盘（支撑「固化 vs 内核差」的独立旁证）。VOID：d_cross 下降伴随 d_self 同幅下降（混账信号）或 d_cross 不动（内核差读法反向证据，升级人类）→ 非零退出。
- **回退**：重验分支 flag 门控（`coreswap.light.writeback.reverify`，同 B6-1 门），默认关 = 逐字节现状；失败臂不重验只计数。

### FA-3 — complete 条件扩展（Mixin:714；承接 .b1-W4）

- **机制**：`f.complete(ch)`（Mixin:714）从「写回已入队即完成」改为「本 chunk S1/S2 入库确认后完成」——**最小形态**（与 FA-1 的 POST 序天然耦合：POST 任务体内 setLightOn/releaseTicket 执行后 complete）。**不采纳 W4 的完整形态**（「+ 邻域静默」）：邻域静默门控（等 3×3 全 lit）是 FB-3/W2 族的机制，其消费点盲区（lightOn 读取全表、scout 盲区#4）与死锁面（b1-W2 §1、b2-FB-3 风险①）在本辖区同样成立，且 b2 已论证其辖区纯粹性差——**本候选只取「complete 晚于数据入库确认」这半边**（直接封 b1 §0.6 耦合窗：下游不在数据未入库时推进 status），邻域静默让渡读侧辖区/后续。
- **前提依赖**：与 FA-1 同 patch 面（POST 任务体尾部 complete 即最小实现）；独立于 C1。
- **影响面**：性能——免全量重光保留度 100%（不改值不改光路，只改 complete 时点，一拍延迟）；时序——LIGHT status 占位时长 +1 队列周期（ms 级），**无 W4 完整形态的 520 恶化面**（那是「邻域静默等邻居」的 grace 级延迟，本候选只是一拍）；与 S6——降级臂 complete（Mixin:714 与 :724 臂各自的完成点）同构后移。
- **风险与失效模式**：① 若下游 status 推进依赖 complete 而非 lightOn/ticket，后移一拍可能轻微拉长推进链（量级 ms，可观测但不预期显著）；② complete 异常路径（:728-729）不得被后移逻辑吞掉——失败 future 仍即时。
- **验收判据草案**：preconditions 同 FA-1 共用；主判据 = 「complete 时同 chunk section 已入库」断言计数 =0 违例（同 FA-1 S3 断言同埋点族）∧ 存在性差不劣化；VOID 同形态。
- **回退**：随 FA-1 同一 flag（`coreswap.light.writeback.post-finalize`）回退；不设独立开关（同 patch 面不独立交付）。

### FA-4 — HEAD 不 cancel 外壳：终态化交还 vanilla（Mixin:856-892）

- **机制**：`wgLightRustTakeover` HEAD 路不再 cancel vanilla light() 的 PRE/POST 外壳（保留 yarn SLP:171-184 全语义：PRE 段 enqueueSectionData + super.propagateLight + POST 段 setLightOn/releaseTicket），仅替换传播内核（Rust JNI 出 nibble 值）。终态化条件、时序、ticket 语义**全部交还 vanilla**——本辖区四个接入点的偏差面（S3 窗、摘票序、complete 条件）被整体消除，因为 vanilla 框架语义原样保留。
- **前提依赖**：独立于 C1（值仍出自 Rust 快照，只换外壳）；但隐含前提 = Rust 内核能在 vanilla light() 调用点**同步**供给 nibble（现状域批为异步批量 JNI，改同步逐 chunk = 域批架构级改动，跨出本候选评估域——诚实登记：这是架构回退级候选，非增量修复）。
- **影响面**：**性能——免全量重光收益大幅回退**：vanilla 逐 chunk light() 恢复 = 域批「一次 JNI 取代 9 次重光」的结构性收益被拆散（每次 light() 仍免 vanilla 传播内核但付调度/JNI 逐次开销）。量化不可静态闭（依赖 vanilla 调度占比），定性 = 保留度最低的候选。正确性收益 = 最大（全部时序偏差面消失，propagateLight 恢复 = FB-2 机制被整建制包含）。
- **风险与失效模式**：① 域批架构瓦解（并行批量收益归零）；② 同步 JNI 在光照线程的延迟传导；③ 作为回退基线的价值大于作为修复候选的价值。
- **验收判据草案**：若立项：preconditions 同 FA-1；主判据 = d_self/d_cross 全账对拍 vanilla 臂 + 域批吞吐回归 ≤ 预登记阈值（性能主判据，与 FA-1-FB-2 包对比择优）；VOID = 吞吐劣化超阈值或任一值差面恶化。
- **回退**：HEAD cancel 逻辑单 flag 恢复（`coreswap.light.takeover.vanilla-shell`）。
- **取舍得论**：**不推荐作为修复包成员**；登记为正确性上界参照 + FA-1/FB-2 包失效时的回退基线（b2-FB-1(b) 同族定位）。

---

## 2. 组合推荐（甲分支方案包）

**推荐包 = FA-1 + FA-3（同一 patch，一个 flag）为主攻；FA-2 挂起待档③；FA-4 不入包。**

捆绑理由：
1. **FA-1 与 FA-3 是同一 POST 任务体的首尾**（POST 体内：setLightOn → releaseTicket → complete），物理不可分批——一个 POST 投递改造同时消 S3 排序窗口 + 摘票提前窗 + b1 §0.6 complete 耦合窗；也同时是 .b3-C4 与 .b2-FB-1(a) 的合并 patch 面（三方同一行对，#150 同批纪律的直接适用：分批 = 同两行改两次 + 两次回归 + 中间态产生新暴露面）。
2. **#150 同批纪律（b3 §2 结论采纳）**：F-A 值修复与 F-C-C3 MUST 同批——本包的 FA-1 与 .b3-C3 组成跨辖区同批（F-A 侧消时序偏差面 + ticket 兜底半边；F-C 侧消二次 grace 结构）。若资源强制分批，唯一合规序 = **F-A 先、F-C 后**（防御性修复先行，b3 §2 已论证反向序违 #150）。
3. **FA-2 为什么挂起**：2038 恒定面未裁决（verdict-08 §0/§2.2 明示，档③ E-3a 是唯一裁决通道）；FA-2 的收益预期 100% 建立在固化读法上——抢跑实施 = 在判据前置（c1-e3a-verdict）失效面上动工，违 §15.1。但 FA-2 设计**现在定稿**，档③出结果即可零延迟立项；且 FA-2 的 hash 失配率采集可与档③ 数据互补。
4. **FA-4 为什么不入包**：性能回退不可量化且定性最大，与「免全量重光收益保留度是核心约束」直接冲突；其正确性收益被 FA-1+FB-2 组合以极小代价覆盖（S3 序 + 传播补全分别消掉），保留为回退基线。

**对两个值差面的预期贡献（诚实量化声明）**：

| 值差面 | FA-1+FA-3 包的贡献 | FA-2 的贡献 |
|---|---|---|
| **2185 抖动面（d_self，O1 已证覆写形态成立）** | **间接、部分**：消 S3 排序窗（lit 时数据未入库的错位子面）+ ticket 兜底（卸载竞态子面）；但覆写主体（邻居后到改写已入库值、边界无传播成员资格）不由本包直接消除——主攻在 FB-2（补 propagateLight）。预期：与 FB-2 捆绑后 d_self 方向 ≈0；**单包幅度不可静态量化**（依赖邻居加载次序分布与 LightStorage 盲区语义，I 级） | 对 2185 面零直接贡献（不改邻居传播时机）；若重算反而改写入库值序，可能出现 d_self 微扰——判据设计须监测 |
| **2038 恒定面（d_cross，未裁决）** | **≈0**（不改值）——诚实声明：若固化读法成立，本包对该面零修复；若 C1 备选（内核差）成立，本包同样零影响。中性 | **唯一对 2038 面有机制贡献的候选**：固化读法成立时预期 ≈100% 消除该面；内核差读法成立时 ≈0。**两个数字均不可静态量化，取决于档③ 未出的裁决——判据已挂 suspended 前置** |

520 存在性差：本包贡献 = ticket 兜底半边（未 lit 不被卸载）；主攻在 .b3-C3（消二次 grace），跨辖区同批。

---

## 3. 与 FB-2（.b2 首选）的关系：**捆绑**（非独立、非互斥）

- **捆绑结论**：FA-1 与 FB-2 应**同一批次交付**。理由：
  1. **同一改动体**：FB-2 接入点 = 「写回方法内 S1/S2 入队之后」（.b2 §1-FB-2：Mixin:590-592 之后、:593 前插入 + 降级臂 :833 后）——与 FA-1 的 POST 重排发生在同一个 `wgLightDomainWriteBack` 方法体（Mixin:584-596），两候选各自的「实施清单」必然重排同一批行；分批做 = 同方法体改两次 + 中间态（只 FB-2 无 POST 序：propagateLight 排程先于 lit 宣言，次序反 vanilla；只 FA-1 无传播：边界仍非传播体系成员）。
  2. **机制互补对同一目标面（2185）**：FA-1 消「lit 时数据未入库 + 摘票过早」的时序错位；FB-2 消「边界从未入传播队列」的成员资格缺失——2185 的覆写机制（O1 已证）两条腿都被踩到才预期收敛 ≈0。
  3. **判据同批采集**：两者的主判据同为 d_self + lit 总数 + d_cross 分账（.b2 §1-FB-2 判据与本文件 FA-1 判据合流为同一采集轮），分批 = 两轮 keep-world 采集重复成本。
- **与 FB-1 的关系**：FB-1(a)（S3 不 lit + ticket 保留）被本辖区 FA-1 **部分吸收**（ticket 保留半边 = C4 语义）；FB-1 的「读侧不 lit 重算」半边与 FA-1 的「POST 后 lit」互斥（一个晚 lit、一个不 lit），择 FA-1——理由：FB-1 性能回退不可量化（.b2 §1-FB-1 自己声明）+ 推进卡闸盲区，FA-1 零回退。

---

## 4. 诚实登记

1. **未实施**：本文纯方案设计，零代码改动、零编译、零命令运行；全部判据为预登记草案，未与采集脚本定稿同稿（#150 判据 5 落地时补）。
2. **yarn 盲区引用面（I 级参照，不得当 F 续推）**：① POST 段任务投递接口与 POST/PRE 交错序（LightStorage/ChunkTaskPrioritySystem 源缺失，FA-1 实施前置确认项）；② 摘票后 completedLevelSupplier 对优先级影响（继承 .b1 §5-②）；③ setLightOn(true) 消费点全表（scout 盲区#4，S3 语义半径未闭合——FA-1 的「S3 窗消除」论证到「与 vanilla 同队列」为止，消费侧行为在盲区）；④ ticket 窗内部调度与卸载交互（FA-1 ticket 兜底半边）；⑤ LightStorage 对「已入库未传播」section 的处理路径（§3 捆绑论证的成分面）。补齐路径 = 一手源入 `.tmp/light-yarn/`。
3. **外推边界**：1.20.1 域批臂限定——1.21.6 无域批路径、无 S4（scout §5），FA-1/FA-3 在 1.21.6 无对应物，FA-2 同构面仅到内联路（未设计，标注独立工作）；n=2/单 seed/overworld 判据设计不外推（#162）；E1 档推理不外推 E2 对拍。Mixin:540-600 逐行复用性（FA-2 前提②）与 Mixin:601-744 任务体（.b3 让渡的 size<9 确认）未全文重读，实施前静态复核。
4. **辖区外让渡清单（#73，只标归属）**：
   - grace/封板/计时起点（.b3-C1/C2/C3）→ **F-C 辖区**；与 FA-1 组成跨辖区同批（#150），批次协调 → 主会话。
   - S1/S2 写侧暂持/后移变体（.b1-W1/W2）→ **F-B 写侧辖区**；本包未纳入（时序整改已由 POST 重排覆盖其主收益面，且 .b1 §3 已论证纯写侧不消 S3 窗）。
   - propagateLight 补做 → **.b2（FB-2）**，本辖区仅做捆绑关系裁决（§3）。
   - 邻域静默门控（W2/FB-3 完整形态）→ 读侧/后续，本包明确不取。
   - 档③ E-3a 判别实验 → 主会话判别线（FA-2 的前置裁决者）。
   - 新 `-D` 开关的 build.gradle 映射 + `check_switch_mapping.py --strict` 门 → 主会话工程面。
   - confirmed 授予 → 用户。
