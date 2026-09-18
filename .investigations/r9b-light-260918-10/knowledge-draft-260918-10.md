# knowledge-draft-260918-10 —— R9-b light reader × bulk writer SENTINEL 覆盖判定（subagent 草稿，主会话应用）

> 工作块 260918-10 / 知识库更新 subagent 草稿（core.worker 形态；主会话只应用 + 验证，不代写）。
> 来源：`.investigations/r9b-light-260918-10/record.md` + `.b1/.b2/.b3` 候选文件（全部已读；file:line 与候选件逐一对表）。
> 依据：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（价值门 / 五段式 / 载体映射 / 自检清单）。
> **状态：全草稿；置信度一律 candidate（judge 待走，confirmed 留用户）。本文件不是正式知识库件——正式应用由主会话执行。**

---

## 〇、价值门声明（GUIDE §〇，先过门再选载体）

| 内容 | 价值档 | 判定理由 | 载体 |
|---|---|---|---|
| 「读者×写者并发义务判定 = 先核读者门字段级内存语义，再谈 future join 论证」 | **高**（可复用判据——若再遇同类读者形态，不想重新想一遍） | 跨课题复用的并发论证判据；含反面教训（b2 自证伪路径） | `knowledge/discovered/workflow-patterns.md` 发现 #187 |
| 「检测面盲区 ≠ 正确性缺口」定性区分 | **高**（判错经验——b3 曾在此混淆） | 防「无 race 可修却立项修 race」弯路 | 并入 #187 判据 2 |
| 07 篇 :1524「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 源冲突 | **高**（文档错误，错误优先原则——含「为什么错」与定位方法） | 项目特定事实勘误，防后续块按错误表述续推 | `versions/1.20.1/docs/07-block-pipeline.md` §15.4 取代注 |
| R9-b 读者×写者覆盖判定四点结论 | **中**（结论简记 + 诚实边界；复用判据已进 #187，此处只记「是什么 + 证据在哪」） | 项目特定结论，主题篇追加小节 | `versions/1.20.1/docs/07-block-pipeline.md` 追加小节 |
| scout → fan-out 三候选 → b2 自证伪 → 收敛 → judge 过程 | **中**（过程简记，含被推翻假说 b2——保留 ❌ 链条） | 时间线归口（10 篇），防重走弯路 | `versions/1.20.1/docs/10-timewise-archive.md` 追加条目 |

无低价值内容混入（未写任何一次性数值/快照）。

---

## 一、草稿 A：`knowledge/discovered/workflow-patterns.md` 追加发现 #187

> 追加位置：文件末尾 `## 发现 #186` 小节之后（`---` 分隔，格式对齐 #143/#144/#184-#186）。应用后同步 `knowledge/discovered/INDEX.md` 尾注行（建议行文见附录 1）。

```markdown
## 发现 #187: 读者×写者并发义务判定 MUST 先核读者门的字段级内存语义（volatile/CAS/普通）再谈 future join 论证；「检测面盲区 ≠ 正确性缺口」须分开裁决（260918-10，#143 四件套可见性链件的读者形态扩展）

- **发现时间 / 发现者 / 置信度 / module**：260918-10；主会话（scout R1-R9 读者清单 + T2 三候选收敛）+ fan-out worker（.b1/.b2/.b3，其中 .b2 自证伪）；**candidate**（Degraded 全静态一手源对拍 + 双树 sha256 7/7 对账；judge 待走，confirmed 留人类）；workflow-patterns / 并发读侧义务判定（**论证判据，不含本块具体判定结果**——判定在 `.investigations/r9b-light-260918-10/record.md`）。
- **来源定位**：`.investigations/r9b-light-260918-10/record.md` §1（三候选收敛裁决）/ §2（mcsrc 版本疑点消解）；候选件 `.b1-candidate-hb-covered.md`（成立）/ `.b2-candidate-real-gap.md`（不成立·自证伪，本条反面教训来源）/ `.b3-candidate-vanilla-isomorph.md`（成立·限缩形态）；一手源锚点：`ProtoChunk.java:42`（`private volatile ChunkStatus status`——读者门语义的整个支点）、`ChunkStatus.java:357-363`（setStatus 在 doWork 后的 thenApply 内 = volatile release 写）、`ServerLightingProviderMixin.java:303-318`（读者 = 裸引用 :307 + `getStatus().isAtLeast(FEATURES)` 门 :313）、`ChunkLightProvider.java:72-77`（vanilla 读者连门都没有）。
- **判据（可复用）**：
  1. **读者×写者并发义务判定，先核读者门的字段级内存语义，再谈 future join 论证**——「读者拿裸引用 + 轮询 status 门、不 join writer future」的形态**看似在 #143 的 future 链可见性论证之外**，实则该 volatile status 字段**独立构成 release/acquire HB 对**（写者 plain 写 → setStatus volatile 写 = release，:357-363；读者 volatile 读 = acquire，Mixin:313；门过即 NOISE 段全部 plain 写可见，含 BulkWb :205-209 换容器+计数）。**反面教训（错误优先）**：本例 .b2 候选以「status 普通写且无同步边 ⇒ 不 join future = 无 HB」立「真缺口说」，一轮一手源实读即自证伪——前提里「普通写」不成立（ProtoChunk.java:42 volatile，两棵参照树逐字一致）。**判错方法：见到「轮询门」先 grep 门所读字段的声明（volatile / CAS / 普通），这一轮 grep 成本远低于在「无 HB」前提下推整条缺口论证。**若字段确为普通写，才轮到 future join / 池共享 / 票调度等次级承载机制排查。
  2. **「检测面盲区 ≠ 正确性缺口」**：sentinel 类在飞检测器对某类参与者（本案读者）零覆盖，只说明「重叠不可观测」（检测覆盖缺口），不构成存在 data race 的证据——两者 MUST 分开裁决。本例读者不登记 SENTINEL_ACTIVE（BulkWb.java:97-102 只登记写者） ⇒ 检测面盲区（OQ-4），而正确性由判据 1 的 volatile 门闭合；加固选项（读者登记 / future join）从「义务修复」降级为 SHOULD 级可观测性增强，且「volatile 化」一臂实为无操作（字段已是 volatile）。混淆两者的代价 = 给不存在 race 的形态立项修 race（.b3 曾滑向此）。
  3. **同构义务的新站点 ≠ 零新增面**：CoreSwap mixin 新增读者站点（R1/R2）替换 vanilla 同职责槽位（LIGHT status 站），获取途径同构（同一 ChunkProvider 裸引用通道）且门更严（vanilla 传播读无门，ChunkLightProvider.java:72-77）——成立形态 =「同构义务的新站点 + 写者等价替换」，**不独立于判据 1 的 HB 论证**；读者读面扩展处（如 packed 路径 writePacket 迭代容器内部，Mixin:358）的闭合仍依赖门内无并发写者论证，不能由「同构」一词带过（.b3 §4② 限缩）。
- **家族索引**：#143（「单写者不变量」四件套——本条是其「可见性链」件对**读者形态**的扩展：#143 论证消费者走同一 future 链，本条论证消费者不碰 future 链时 volatile 门独立承载 HB）、#36 家族（交接前提廉价验证精神面——.b2 自证伪即「前提先过一轮一手核对」的正面执行）、#90 家族（机制措辞按源码不按直觉——「volatile 字段轮询门」读作「无同步边」即直觉错误）。
```

---

## 二、草稿 B：`versions/1.20.1/docs/07-block-pipeline.md`

### B-1：:1524 勘误取代注（§15.4 形态——原句不改，紧跟其后插注记）

> 应用位置：`07-block-pipeline.md` :1524 行（「…光照拆出 `INITIALIZE_LIGHT` 新状态（1.21.6 新增）；…」）**原句不动**，紧接该行后插入下列注记行（缩进与所在列表项一致）：

```markdown
  - ⚠️ **§15.4 取代注（260918-10）**：上行「`INITIALIZE_LIGHT` 新状态（1.21.6 新增）」**「1.21.6 新增」部分被取代**——INITIALIZE_LIGHT 在 **1.20.1 源已存在**（`ChunkStatus.java:158`，常量集含 INITIALIZE_LIGHT(:158)/LIGHT(:170)；勘误依据 = 双树逐字节对账 `.tmp/scout-260905-08/mcsrc` × `versions/1.20.1/data/mc_src_extract` 关键 7 文件 sha256 **7/7 一致**，见 `.investigations/r9b-light-260918-10/record.md` §2 + `.b3` §0）。1.21.6 侧的**真实结构差异** = ChunkStatus 任务改为经 `ChunkGenerationSteps` 注册链（`Builder(previousStep)` 结构化承担有序性，本文件 :1524 前文已述），**不是** INITIALIZE_LIGHT 状态本身的引入。该行其余对照要点（futuresByStatus 上移 / TACS 改名 / 注册链 / 邻居前置结构化）不受影响。
```

> 错误五段式（供 record/时间线引用，正式台账载体 = 本取代注 + 10 篇时间线条目）：
> - **现象**：07 篇 :1524 记「INITIALIZE_LIGHT 新状态（1.21.6 新增）」；260918-10 一手实读 1.20.1 源发现 ChunkStatus.java:158 已有该常量。
> - **根因**：260913-04 写该对照要点时，1.21.6 侧「光照拆出 INITIALIZE_LIGHT 状态站」的结构变化被转写为「该状态为 1.21.6 新增」——把「1.21.6 把光照拆成两站（INITIALIZE_LIGHT + LIGHT）的注册链形态」与「INITIALIZE_LIGHT 状态存在性」混淆；1.20.1 侧该状态已存在（仅任务划分不同）。
> - **定位**：260918-10 对 R9-b 读者门做一手源核对时，.b1 §4 特征核对直接命中冲突（INITIALIZE_LIGHT(:158) 在「旧」树存在）→ 双树 sha256 7/7 对账排除「引用树非 1.20.1」的逃逸解释（scout §5 版本疑点消解）。
> - **修复**：原句不改，:1524 行后补取代注（上文 B-1 文本）；07 篇 R9-b 区追加 260918-10 小节（B-2）时复述正确差异。
> - **教训**：跨版本对照要点 MUX 双重断言（「结构 X 存在」+「结构 X 为版本 N 新增」）时，「新增」这半句 MUST 对**旧版本树**做一次存在性 grep 再落笔；版本对账（sha256 双树）是「疑点 → 确证」的廉价终审手段。

### B-2：R9-b 区追加小节正文草稿

> 应用位置：`07-block-pipeline.md` 末尾「形态审计 260915-01」等既有小节之后追加（追加不覆盖；小节标题带工作块号与状态，对齐既有「2026-09-13 R9-b 追加」格式）：

```markdown
## 260918-10 追加：light reader × bulk writer 覆盖判定（无正确性缺口）— **candidate**（judge 待走；confirmed 留用户）

> 承接上文 R9-b 两节（单写者不变量 + sentinel 落地）：本块补齐其**读者侧**覆盖判定——「R1/R2 光照读者 × bulk 写回是否存在读写竞争缺口」。载体与依据：`.investigations/r9b-light-260918-10/`（record + scout-map + .b1/.b2/.b3 候选件）。**验证分层 = Degraded（全静态一手源对拍 + 双树 sha256 7/7 对账；无 trace/behavior 级证据）**。通用模式 → `knowledge/discovered/workflow-patterns.md` #187。

### 结论（4 点）
1. **R1/R2 光照读者 × bulk 写回：无 data race 缺口**——读者门 `getStatus().isAtLeast(FEATURES)`（`ServerLightingProviderMixin.java:313`）所读 `ProtoChunk.status` 为 **volatile** 字段（`ProtoChunk.java:42`），写者链 `BulkWb` plain 写（:205-209）→ `setStatus(NOISE)` volatile 写（`ChunkStatus.java:357-363`，thenApply 内）= release，读者 volatile 读 = acquire ⇒ 门过即 HB 成立；**不依赖 future join，不依赖 ChunkHolder futuresByStatus（读者根本不触碰）**。
2. **OQ-5 闭合**：门内读者 `writePacket` 迭代容器内部结构无半构造窗口——新容器在换入前完整构造（`BulkWb.java:201`），门过后无并发写者；`PalettedContainer.data` volatile（:36）为 vanilla 固有兜底（旧/新原子切换）。
3. **SENTINEL 读者不登记 = 检测面盲区（OQ-4），非正确性缺口**——vanilla 本无读者在飞检测器；加固（读者登记 / future join）从「义务修复」降级为 SHOULD 级可观测性增强，**scope 决策交用户，倾向不实施**（见时间线 260918-10 决策点）。
4. **OQ-2 大部闭合**：vanilla 自身读者（`ChunkLightProvider.java:72-77`）连 status 门都没有，CoreSwap R1/R2 读者侧更严 ⇒ 形态 ∈ vanilla 既有义务类；bulk 写将可见中间态从「逐块渐进」收敛为「按 section 旧或新完整容器」= 原子性增强。

### 证据定位
- 三候选收敛：`.investigations/r9b-light-260918-10/record.md` §1（.b1 成立 / .b2 不成立·自证伪 / .b3 成立·限缩）；逐条 file:line 证据表见 `.b1`（E1-E9）/ `.b2`（E1-E7）/ `.b3`（V1-V6 + C1-C3 + D1-D2）。
- mcsrc 版本疑点消解：record §2 + `.b3` §0（双树 7/7 sha256 + 版本特征符双通道，1.20.1 确证）。
- 勘误：本文件 :1524「INITIALIZE_LIGHT 1.21.6 新增」已按 §15.4 加取代注（同上）。

### 诚实边界（Degraded 边界，随结论携带）
- **全静态（Degraded）**：HB 为 JMM 演绎非运行时实测；无 LIGHT_BETAPROBE 类 behavior 证据。
- **R4/R5（gate 关 vanilla 回退路径）的 TicketManager margin 装配细节未实读**——对 R4 不做同强度主张（结构性同链保护成立，细节留待需要时补读）。
- **OQ-6（INPLAY 期读者×写者）超本块范围**，维持 open。
- **sentinel 读者登记 = 未实施的可选项**（OQ-4），实施与否待用户拍板；未实施前读者×写者在飞重叠不可观测。
```

---

## 三、草稿 C：`versions/1.20.1/docs/10-timewise-archive.md` 260918-10 时间线条目

> 应用位置：文件末尾（260918-07 条目之后）追加：

```markdown
## 260918-10 块（实际 2026-09-18，Get-Date 锚；R9-b light reader × bulk writer SENTINEL 覆盖判定——三候选收敛「无正确性缺口」）🔍 candidate（judge 待走；confirmed 留用户；Degraded 全静态）

> 承接 260913-02/03 R9-b（单写者不变量 + sentinel 落地，均 confirmed）遗留的**读者侧**判定缺口。架构 = `.investigations/000-架构设计/架构计划-260918-10-R9b-light-reader-sentinel.md`（用户批准）。过程产物 `.investigations/r9b-light-260918-10/`（record + scout-map + .b1/.b2/.b3）。

- ✅ **scout 勘探**：读者清单 R1-R9（裸引用/门/获取途径逐站点分类）+ 时序交叉 W1-W6 + OQ-1~OQ-6 开放问题面（scout-map.md）；mcsrc 双树版本疑点登记（scout §5，后由 worker 消解）。
- ✅ **fan-out 三候选**（判定树 ≥2 互斥候选，强制触发）：.b1「已覆盖说」（volatile status 门独立构成 HB——成立）/ .b2「真缺口说」（不 join future = 无 HB——**不成立，自证伪**）/ .b3「vanilla 同构既有说」（义务承载三要素原样保留——成立，附限缩：R1/R2 为新增站点，同构义务论证不独立于 b1）。
- ❌→✅ **b2 自证伪（本块关键错误链，教训已沉淀 → workflow-patterns #187）**：b2 前提「status 普通写且无同步边」被一轮一手源实读直接证伪——读者门所读 `ProtoChunk.status` 是 **volatile** 字段（ProtoChunk.java:42，双树逐字一致）；b2 按诚实规则如实改判「HB 闭合」，并给出加固选项清单（读者登记 / future join / volatile 化——最后一项实为无操作）。**判错方法**：见「轮询门」先 grep 门所读字段声明（volatile/CAS/普通），再谈 HB 有无；**「检测面盲区 ≠ 正确性缺口」分开裁决**。
- ✅ **T2 收敛裁决**：① R1/R2 × bulk 写回无 data race（volatile 门 HB）；② OQ-5 闭合（无半构造窗口，门后无并发写者）；③ SENTINEL 读者不登记 = 检测面盲区非正确性缺口（OQ-4）；④ OQ-2 结构性闭合（vanilla 读者更弱——连门都没有，ChunkLightProvider.java:72-77）。全文 → record.md §1。
- ✅ **mcsrc 版本疑点消解**：双通道（sha256 关键 7 文件 **7/7 一致** + 版本特征符）确证引用树 = 1.20.1，行号引用可信（.b3 §0）。
- ⚠️ **附带勘误（§15.4）**：07 篇 :1524「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 源冲突（ChunkStatus.java:158 已存在）——取代注 + 小节草稿已备（knowledge-draft-260918-10.md B-1/B-2），原句不改。
- 📌 **决策点（用户拍板）**：OQ-4 scope——**sentinel 读者登记不实施**（检测面增强属 SHOULD 级可选项，非义务修复；实施需评估光照热路径开销）。OQ-6（INPLAY 期）维持超范围。
- 🔍 **诚实边界**：Degraded 全静态（无 behavior 证据）；R4/R5 TicketManager 装配细节未实读；judge MUST 待走（三源核对）。
- 📌 **记录指引**：通用模式 → workflow-patterns **#187**（subagent 草稿 + 主会话应用；草稿 `.investigations/r9b-light-260918-10/knowledge-draft-260918-10.md`）；07 篇勘误取代注 + R9-b 追加小节 → 同草稿 B-1/B-2；INDEX 尾注行同批落盘。
```

---

## 附录 1：`knowledge/discovered/INDEX.md` 尾注行（建议，随 #187 同批落盘）

```markdown
- #187 读者×写者并发义务判定先核读者门字段级内存语义（volatile/CAS/普通）再谈 future join；检测面盲区 ≠ 正确性缺口（260918-10，#143 读者形态扩展）
```

## 附录 2：产出自检清单核对（GUIDE §四，逐条）

- [x] **先过价值门**：三条内容均高/中档（§〇 逐条判定）；无低价值结论写入 docs。
- [x] **五段式**：勘误错误链完整五段（B-1 附），无「只记已修复」条目。
- [x] **根因机制层**：勘误根因 = 「拆站形态」与「状态存在性」混淆，非现象复述；b2 根因 = 前提「普通写」未核对即采用。
- [x] **定位含诊断方法**：双树 sha256 对账 / 门字段声明 grep / 特征符核对，均可复用。
- [x] **判错经验已沉淀**：#187 判据 1（先 grep 门字段声明）+ 判据 2（盲区≠缺口）。
- [x] **被排除假说标注**：b2「真缺口说」❌ 保留（不成立·自证伪），时间线与 #187 均保留链条。
- [x] **载体正确**：判据→discovered；勘误+结论→07 主题篇；过程→10 时间线；草稿本体→.investigations（非正式件）。
- [x] **无复用价值内容未写 docs**：逐候选完整论证不进 docs（留候选件），docs 只记收敛结论 + 指针。
- [x] **速查表**：本块无独立错误台账文件（唯一错误链 = b2 自证伪 + 勘误，均五段式内嵌于草稿 B-1/#187/时间线，载体按上表）——主会话若认为需独立成篇，可从 B-1 五段式抽出，当前判定不构成「散落错误」反模式。
- [x] **数字来自候选件实读记录**：全部 file:line（ProtoChunk.java:42 / ChunkStatus.java:158,:357-363,:359-363,:361 / Mixin:303-318,:307,:313,:314-316,:358 / BulkWb.java:97-102,:163-171,:177,:200-210,:201,:205-209 / PalettedContainer.java:36 / ChunkLightProvider.java:72-77 / CppBridge.java:614,:651-671 / ChunkHolder.java:49）逐一与 .b1/.b2/.b3 证据表对表一致；sha256 7/7、区块数字 522/576 等均转录自候选件，无编造、无占位符。
- [x] **格式与目标文件末尾现状对齐**：#187 对齐 #143/#144 结构（发现时间行/来源定位/判据/家族索引）；07 篇小节对齐既有「2026-09-13 R9-b 追加」标题与引用块格式；10 篇条目对齐 260917-0x/260918-0x 条目格式（状态行 + 状态 emoji + 📌 记录指引）；07 篇 :1524 现文已实读（:1524 = 1.21.6 结构变化对照要点行，「1.21.6 新增」确在其中）。
- [x] **置信度一律 candidate**；confirmed 表述均为「留用户」，无越权授予。

## 附录 3：主会话应用清单（本草稿 → 正式件）

1. `knowledge/discovered/workflow-patterns.md` 末尾追加草稿 A（#187）+ INDEX.md 尾注行（附录 1）。
2. `versions/1.20.1/docs/07-block-pipeline.md`：:1524 行后插 B-1 取代注；末尾追加 B-2 小节。
3. `versions/1.20.1/docs/10-timewise-archive.md` 末尾追加草稿 C。
4. 应用后核对：#187 序号未被并发占用（本草稿撰写时 #187 空缺，已 grep 确认）；INDEX 同步；record.md §5 待办勾销第 3 项。
