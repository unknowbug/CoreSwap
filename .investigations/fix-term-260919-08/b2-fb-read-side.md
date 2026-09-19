---
id: b2-fb-read-side
block: F-B 读侧策略（fan-out 候选 .b2，互斥辖区：vanilla 增量传播接管/补传播；不含写侧暂持/后移 = .b1 辖区）
status: draft
date: 260919-08
inputs:
  - .artifacts/t8-attrib-260919-07/verdict-260919-07.md (confirmed)
  - .investigations/fix-term-260919-08/scout-map.md
一手源: versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java（F）；yarn SLP.java:97-104/171-184（F，.tmp/light-yarn/ 在案）；LightStorage/ChunkLightProvider/ChunkTicketManager/setLightOn 消费点（★盲区，I 级参照）
---

# F-B 读侧策略候选方案（.b2，draft，未实施）

## 0. 辖区与共同前提

- 辖区 = 让 vanilla 增量传播接管/补传播：Mixin:594 S3 读侧不 lit、补 propagateLight（yarn SLP.java:97-104，Mixin:59-60 注释自认 cancel 后跳过）、部分 lit / 延迟 lit 变体。**不含** S1/S2 写侧暂持（.b1）。
- **C1 依赖性（verdict §1 备选分解）**：「恒定 2038 = 终态化固化」与「恒定 2038 = 确定性 Rust-vs-Java 内核语义差」两读法未判别分离。本块全部候选的收益预期依赖前者读法：若 C1 备选成立（Rust 值本身系统性错），读侧策略只保证「按 vanilla 语义收敛」，收敛到的仍是 Rust 输入驱动的 vanilla 传播结果，**2038 面不会被本块候选消除**——判据设计须与档②/③判别实验解耦（见各验收判据 preconditions）。
- **盲区约束**：LightStorage 内部改写语义、setLightOn(true) 消费点全表（scout 盲区 #4）、ticket 窗内部调度均为 I 级参照。本块机制论证到 SLP/TACS 入口链为止（F），链内行为为 I 级外推，逐条标注。

## 1. 候选清单

### FB-1　S3 读侧不 lit（Mixin:594 保持未 lit / setLightOn(false)）

- **机制**：删除（或改 false）Mixin:594 `chunk.setLightOn(true)`（域批路 Mixin:594 + 内联路 Mixin:836 两处同构副本）。chunk 保持 lightOn=false → 下游（TACS/ChunkHierarchy 消费点，★盲区 I）在后加载邻居到达、触发 status 推进/propagateLight（yarn SLP:97-104 ← LP:79-87）时按未 lit 语义对该 chunk 重光。
- **前提依赖（C1）**：中性——不依赖固化 vs 内核差判别。
- **关键前提依赖（非 C1）——S4 摘票必须同步变更**：现实现 S3+S4 成对（Mixin:594-595 / 836-837）。若仅去 S3 保留 S4（Mixin:595 摘票照旧），LIGHT ticket 消失后 vanilla **再无结构入口回头调用 light()** → chunk 永久 unlit = 520 存在性差（verdict §1.2）被系统性放大，这是硬失效模式。因此 FB-1 必须二选一：
  - (a) 保留 LIGHT ticket（去掉 Mixin:595），由 ticket 窗兜底直至某处（★盲区：vanilla 无自动「邻域静默后补 lit」检查点，需 mixin 自建检查点）→ **摘票时机属于 F-C 辖区（scout-map §4 F-C 第 4 行），本候选对 F-C 有硬依赖**；
  - (b) 完全不 cancel vanilla light()（等于退回 vanilla 全重光，Rust 光照性能收益归零）——此臂实质是「放弃接管」，仅作回退基线登记，不作推荐候选。
- **影响面（免全量重光目标回退量化）**：
  - 现设计目标 = Rust 一次域批 JNI 取代 vanilla 全量重光。FB-1(a) 下，每个后加载邻居到达时触发对未 lit chunk 的重算，重算路径 = vanilla 增量传播引擎（LP:43-54 doLightUpdates，经 checkBlock/setSectionStatus/propagateLight 入口）。**代价幅度不可静态量化（诚实声明）**：触发次数取决于邻居加载次序分布（2185/3703 ≈ 59% center 集被覆写抖动暗示触发面不小），且单次重算是「增量传播」还是接近全量取决于 LightStorage 内部就绪判定（★盲区）。可静态声明的外推下界：每个 unlit chunk 至少被重光一次（否则永久 unlit）→ 重光总量 ≥ 现在的 0 次；上界无法静态闭。
  - 对 status 推进：lightOn=false 时 chunk 能否继续向 FULL 推进本身就在 ★盲区消费点内（scout 盲区 #4 明示语义半径未闭合）——若下游以 lightOn 作为推进闸，FB-1(a) 在自建检查点亮灯前会卡推进，等于把 F-C ticket 窗问题变形放大。
- **风险/失效模式**：① 上述永久 unlit 放大（S4 未联动时，确定性失效）；② 推进卡闸（消费点盲区，无法静态排除）；③ 性能回退不可静态量化；④ 域批 9 中心共享一份 out 帧的语义被打破——重光引擎直接改 LightStorage，Rust 快照与 vanilla 传播结果混合共存，跨 run 恒定性（verdict §1「成员资格恒定」）依据被移除，diff 面变为动态。
- **验收判据草案**：
  - preconditions：{key: light-hook, expected: betaprobe armed, check: [LIGHT-PATH] enter 计数 > 0}；{key: selfcert, expected: exit 0}；{key: c1-separation, expected: 档②/③判别已出结论, check: 判别记录在案}——**c1-separation 失效 → 本候选全部判据 suspended（premise-expired）**，因 d_cross 预期无法设定。
  - 主判据：n=2 重跑 d_self（同 key 集）→ 预期较 2185 显著下降且 lit 总数 = 9450（520 面不恶化）；lit 恒定面为主判据兜底。
  - VOID 判据：unlit 计数 > 0 或 lit 总数 < 9450 → 判据 VOID（存在性差恶化 = 硬失败）。
- **回退**：恢复 Mixin:594/836 两行 + Mixin:595/837（单一 commit revert；-Dcoreswap.light.rust 关闭即零影响）。

### FB-2　补 propagateLight（写回后主动排程被 cancel 掉的传播步）——推荐

- **机制**：在写回方法内、S1/S2 入队之后，调用 vanilla 包装 `ServerLightingProvider.propagateLight(ChunkPos)`（yarn SLP.java:97-104），即 vanilla `light()` PRE 段（yarn SLP:174-178）里被 cancel 跳过的 `super.propagateLight(chunkPos)`。接入点两处同构：域批 `wgLightDomainWriteBack`（Mixin:590-592 之后，:593 前插入）与内联 `wgLightLegacyTakeover`（Mixin:833 之后）。该方法本身是 enqueue 型投递（同 ChunkTaskPrioritySystem 队列），与 S1/S2 保持同队列顺序（先写数据后排程传播边界），不改 Mixin:594/595 的 POST 复刻语义。
- **前提依赖（C1）**：同 FB-1 中性；且对 2038 面的作用方向在两种读法下不同（见 §2 针对关系论证）。
- **影响面（性能）**：vanilla 本来就在 light() 内为每个 chunk 付一次 propagateLight 代价；本候选是**恢复**被 cancel 跳过的既有步骤，不是新增开销——相对现状（跳过态）是每 chunk 一次边界排程的净增，相对 vanilla 语义是持平。域批收益（一次 JNI 取代 9 次重光）**完整保留**，免全量重光目标零回退（这是与 FB-1 的本质区别：FB-2 不放弃 lit，只补传播）。
- **风险/失效模式**：① propagateLight 投递与 S1/S2 的跨 chunk 交错序不可静态闭（★盲区：LightStorage 是否在数据未 PRE-UPDATE 入库前处理传播消息）；② 若 C1 备选成立（Rust 内核差），补传播后收敛值仍含 Rust 输入偏差，2038 面不改——判据必须与 d_cross 分账，不得混报；③ 排程量增大对光照线程队列的压力（每 chunk +1 消息，线性，风险低）。
- **验收判据草案**：
  - preconditions：同 FB-1 三项 + {key: queue-order, expected: PRE 段先于 doLightUpdates, check: yarn SLP:195-208 静态复核 + 一次 betadump 采样}。
  - 主判据：n=2 重跑 d_self ≤ 2185 的显著下降（目标方向 ≈0，即覆写抖动面被 vanilla 收敛语义吸收），且 lit 总数恒 9450、d_cross 单独分账报告。
  - VOID 判据：d_self 上升或出现新 unlit → VOID；d_self 下降伴随 d_cross 同幅下降 → 判 VOID（说明混账，C1 未分离）。
- **回退**：删除插入的 1-2 行调用；开关面零改动。

### FB-3　部分 lit / 延迟 setLightOn 至邻域静默（同族变体）

- **机制**：保留 S1/S2 + S4 时机不变，把 S3（Mixin:594）从「写回线程立即执行」改为「邻域静默检查点后执行」：域任务完成后注册 3×3 邻域 lit 状态观察（可复用 LightDomainBatch 的 State/seal 结构做域级 barrier），全部邻居 lit 才补 `setLightOn(true)`。等价于把 vanilla POST 段的「同队列顺序保证」（yarn SLP:200-217，scout §1-S3 指出的排序偏差修复）外推为域级语义。
- **前提依赖**：① **F-C 硬依赖**——「静默」的兜底机制只能是 LIGHT ticket 不摘（Mixin:595 延迟），摘票时机属 F-C 辖区（scout-map §4）；② C1 中性；③ 邻域 lit 状态的查询接口在 ★盲区消费点面内（lightOn 读取者全表未闭合，检查点实现可能需要额外 Accessor，存在性未证）。
- **影响面**：免全量重光目标保留（lit 最终仍发生，只是后移）；status 推进在检查点亮灯前可能被卡（同 FB-1 风险②，盲区不可静态排除）；排序偏差（scout §1-S3 / 盲区 #2）被顺带修复是附带收益。
- **风险/失效模式**：① 检查点死锁/泄漏（某邻居永久不 lit → 永不点灯，需 grace 式超时兜底——引入第二套计时器，复杂度最高）；② 消费点盲区使「未 lit 期间下游行为」完全不可静态推理；③ 与 F-B「读侧」定位渐行渐远——本质是 F-A（写回时机/终态化条件，scout-map §4 F-A 第 2-3 行）+ F-C（摘票时机）的杂交，辖区纯粹性差。
- **验收判据草案**：preconditions 同 FB-1 + {key: lit-consumer-table, expected: setLightOn 消费点全表在案, check: yarn 一手源补齐}——**lit-consumer-table 失效 → suspended**（检查点正确性不可验证）。主判据：d_self 显著下降 + lit 最终 9450 + 无推进卡死（server 启动到 FULL 的 chunk 完成时间分布不劣化）。VOID：出现永久 unlit 或推进卡死。
- **回退**：恢复 Mixin:594 原位执行；删除检查点。

## 2. 「补 propagateLight」与覆写抖动面（2185）的针对关系论证

**2185 的产生机制**（verdict §1 + judge C3 内联限定）：写回值已入 vanilla LightStorage → 后加载邻居触发 vanilla 增量传播 → **就地改写**已写值 → d_self=2185 抖动。即：现状下 vanilla 传播已经在碰我们的数据，但是**无序、不完整**的——因为本 chunk 的边界从未进入传播队列（propagateLight 被 cancel，Mixin:59-60 自认），邻居到达时只有邻居侧的边界入队，跨边界传播读到的是我们未参与传播体系的孤立快照值。

**补 propagateLight 的针对性**：调用 propagateLight(chunkPos)（yarn SLP:97-104）正是把本 chunk 的 24 节边界块加入传播队列的动作（vanilla light() PRE 段 :174-178 的原生次序：先 enqueueSectionData 后 propagateLight，本候选 1:1 复刻该次序）。补上后：

1. 写回瞬间，本 chunk 即成为传播体系的正式成员（边界入队），后加载邻居到达时触发的是 **vanilla 原生收敛路径**（邻居边界入队 → 跨边界增量传播双向收敛），与 vanilla 自身 chunk 后加载场景的语义**完全同构**——这正是辖区命题「让 vanilla 增量传播接管」的最小实现。
2. 2185 的成分被改写：现状的「孤立快照被随机时机部分改写」（非收敛、不可复现次序）→ 补传播后的「完整传播体系内的确定性收敛」。**预期 d_self 下降但收敛终值未必等于 Rust 快照值**——诚实声明：若终值 ≠ 快照值且 ≠ vanilla 独立重光值，说明残余差在 Rust 输入/内核层（C1 备选读法的证据方向），d_cross 分账判据即为此设。
3. **不可静态闭的部分（I 级）**：LightStorage 内部是否对「数据已 PRE 入库但未传播」的 section 有不同处理路径，是 2185 精确成分判定的前提（★盲区，scout 盲区 #1）；O1 边界带形态门（verdict §4）需档② keep-world 采集后才能校验本论证的预测力。

## 3. 诚实登记

- **未实施**：本块全部候选为静态方案设计，零代码改动、零运行验证；验收判据全部为草案，主判据数值预期（d_self 下降幅度）不可静态量化，仅方向性。
- **yarn 盲区引用面**：LightStorage / ChunkLightProvider / ChunkTicketManager 一手源缺失；setLightOn(true) 消费点全表（FB-1/FB-3 的语义半径核心）；propagateLight 在 ChunkTaskPrioritySystem 内的精确消息语义与排序保证（FB-2 的 queue-order 前置只能静态复核到 SLP:195-208 层）。相关论证全部 I 级外推。
- **外推边界**：n=2/单 seed/overworld 观测不外推；E1 档单臂推理不外推 E2 对拍；2185 成员资格（100% ⊆ center 集）基于当前 run 集，不外推全部加载次序分布；FB-2 的收敛性论证未经任何运行时证据支持，属机制推理。
- **辖区外交付让渡清单**：
  - **F-C（.b3/对应辖区）**：FB-1(a) 与 FB-3 的硬前置——LIGHT ticket 摘除时机（Mixin:595 / yarn TACS:684-692）的重构、邻域静默的兜底窗设计、grace 参数对齐。没有 F-C 配合，FB-1/FB-3 不可独立交付。
  - **F-A（.b1 辖区）**：2038 恒定面（提交时快照 vs 终态世界）的写侧修复；FB-2 对该面的作用是间接的（经传播收敛），不承诺消除。
  - **档②O1 / 档③E-3a（C1 判别实验）**：本块全部主判据以 C1 分离为前置；判别实验本身不属本块。
  - **1.21.6 同构面**：域批无对应物（scout §5），FB-2 的内联路插入点 1.21.6 有同构（:220-221 后），但未做任何设计，仅标注。

## 4. 推荐倾向

**FB-2（补 propagateLight）为首选**：唯一零回退「免全量重光」收益、最小改动面（1-2 行/路 × 2 路）、对 2185 有直接机制针对性（§2）、C1 中性。FB-1/FB-3 均因「消费点盲区 + F-C 硬依赖 + 性能回退不可量化」列为次选/不推荐。建议 FB-2 判据与档②/③判别实验同批采集以分账 d_self/d_cross。
