# F-A 乙分支处置设计 — b5-fa-branchB（260919-08，draft）

---
status: draft
block: fix-term-260919-08
fanout: F-A 乙分支候选（.b5，core.fanout；本文件 = 唯一写产物，方案设计不改代码）
inputs: .artifacts/t8-attrib-260919-07/verdict-260919-07.md §1.1 C1 备选（confirmed 主体的未判别备选）；.artifacts/fix-term-260919-08/verdict-260919-08.md（O1 PASS=覆写形态 candidate；2038 恒定面未裁决→档③）；.investigations/fix-term-260919-08/scout-map.md §1（S1/S2 值来源 = Mixin:562-571/550-552 提交线程快照）
scope_note: 辖区 = 乙读法（2038 恒定面 = 确定性 Rust-vs-Java 内核/边界语义差）的判别实验设计 + 证实后的修复方向清单 + 双分支决策矩阵；甲读法（终态化固化）的方案包设计不在本辖区（#73 让渡主会话/甲分支 worker）
---

## 0. 辖区课题一句话

verdict-260919-07 §1.1 C1 备选：**2038 恒定面（d_cross 跨 run 逐 key 恒定的 3703-2185 差值面）有两种未判别解释**——甲 = 提交时快照被终态化固化；乙 = 确定性 Rust-vs-Java 内核/边界语义差。档② O1 过门只裁决了 2185 抖动面 = 覆写形态（verdict-260919-08 §2），乙对 2038 恒定面的主张**完好无损**（并存形态），其唯一裁决通道 = 档③ E-3a 输入 hash 直证。本文件为档③给出改码方案 + 采集协议 + 预登记判据草案，并预置乙证实后的修复方向与 HOOK-4 决策矩阵。

## 1. 档③ E-3a 判别实验设计

### 1.1 判别原理（为什么输入 hash 能判别）

乙读法的核心命题是「**输入相同、输出不同**」——Rust 内核在接收到相同 blocks25 邻域快照时产出与 Java 不同的光值（或 Rust 两路径间有边界语义差）。甲读法（终态化固化）则隐含「**输入本身已异于终态世界**」（提交时快照 ≠ 写回时世界），即跨 run 的恒定错值来自**恒定的上游输入差**（域批几何决定、逐 run 复现）。故：对每个写回 chunk 的 light 内核输入（blocks25 邻域快照）打 hash、跨 run 对比：

- **输入同 hash 而输出异** → 差异产生于内核本体 → 乙读法直证（verdict-260919-07 §2「内核静态确定性复核」只排除了 Rust 内部两路差，未排除 Rust-vs-Java——此实验正面击穿该轴）。
- **输入异** → 差异产生于上游（快照时机/域批几何/邻域装配）→ 甲读法（或上游差家族）支持，乙被削弱。

### 1.2 最小改码方案（打点位置 / 内容 / 格式 / 开关与回退）

| 项 | 设计 |
|---|---|
| **打点位置** | `ServerLightingProviderMixin.wgLightDomainTaskRun` 内、调用 Rust light 计算之前的输入收集点——即 scout-map §1-S1/S2 的值来源处（提交线程快照，Mixin:562-571 blocks9 / :550-552 packed 收集完成后、写回 enqueue 之前）。**一个打点，不碰 Rust 内核本体、不碰 enqueue/setLightOn/摘票任何 S1-S4 副作用** |
| **打点内容** | ① 逐 chunk 的 blocks25（或实传的 blocks9+邻域组装面，以实参为准——登记实参形状）输入字节流 sha256（8 位缩写+全 hash）；② 写回产出的 24 节 nibble 数组逐节 sha256（输出 hash，复用 region 解析器同款算法口径以便与 .mca 互验）；③ chunk 坐标 + 域 key + seed |
| **输出格式** | 每行一条 `[E3A] seed=<s> cx=<x> cz=<z> blk25=<sha8> out=<per-section sha8 csv 24 项>`，写采集 log（同一 SLF4J/log 通道，无新文件句柄） |
| **开关** | 新增 `-P` 映射 → `-D` 消费点，命名 **`coreswap.light.e3aInputHash`**——与 `coreswap.light.domainbatch.gracems`（LightDomainBatch.java:21）**同族点分先例对齐**（#19 家族对称性：light 域族为点分族）。**默认关闭**；消费点 = 任务级 `boolean` 读一次 `System.getProperty`（缓存进任务局部变量，不每点读） |
| **门控纪律（诊断污染铁律）** | **chunk 级判断一次**：开关读数在域任务入口取一次；hash 计算只在开关为 true 时执行，且每 chunk 一次（24 节 hash 一次算完）——**不在任何 per-block/per-nibble 循环内加分支**。关闭态开销 = 任务入口一次布尔读 ≈ 0；开启态仅诊断采集 run 使用，bench/性能 run 永不带此开关 |
| **回退** | 关闭开关即全量回退（零行为差）；改码面 = 一个 if 块 + build.gradle 一行映射。**改码后 MUST 跑 `python scripts\check_switch_mapping.py --strict`**（B6-1 门：新映射行 + 新 -D 消费点 = MUST 触发时机） |
| **影响面评估（改码批准输入）** | ① 触碰文件 = ServerLightingProviderMixin.java + build.gradle，共 2；② 不改任何 S1-S4 副作用序（打点为纯只读观察，enqueue 前）；③ 1.21.6 无域批路径（scout-map §5）→ 本打点为 1.20.1 专属，1.21.6 不映射（per-version 缺口按族内对称性登记豁免 reason：该版本无消费闭包）；④ 内联路 Mixin:829-837 同构副本**不打点**（本档判别域 = 域批臂，与档②一致，verdict-260919-08 §6）——登记为覆盖面限定非遗漏 |

### 1.3 采集协议（不能复用 r3/r4 归档——原因与替代）

**为何不能复用**：r3/r4 归档 = region `.mca` 光输出 + log，**无生成期输入 hash**；blocks25 输入 hash 只能在生成时打点产出 → **须重跑采集**（verdict-260919-07 §4 已预登记此性质）。

协议（对齐档② r3/r4 采集链形态）：

1. **臂定义**：r5、r6 = 同 seed（8576294172403134396）、同 dll（sha8=cc4e39fe 或新 dll 若打点改码致 sha 变——**变 sha 则判据前置集与 SELFCERT 的 dll 项同步更新并登记 sha 漂移原因**，#161 世界链不因诊断 sha 断）、同 FP-DRV、`-Dcoreswap.light.e3aInputHash=true` 的 E1 双 run（keep-world 协议：r6 前 rmtree 重建 world + `--cutover` mtime 身份核对，#161）。
2. **四件套（#195）**：本判据文件预登记（升级为 `.investigations/fix-term-260919-08/e3a-criteria.md`，采集前定稿 mtime 可核 #112）+ 比较器 `e3a_cmp.py`（只读输入、只写 `--out` json）+ 采集 raw log 落 `.investigations/fix-term-260919-08/cmd-output/` + 结果 json。comparator 三轮内含 VOID 记录照档②先例断链不进判读（#160）。
3. **前置集（preconditions，v0.22 §15.1，草案）**：P01-P06 双臂 SELFCERT 五项（lightInit/hook armed/dll sha/seed/FB-DRV move5，log 子串机械命中）+ P07 world 身份链（r6 全 region mtime > rmtree cutover 时刻，未提供 cutover = VOID）+ P08 E3A 行完备性（两臂 `[E3A]` 行 chunk 数 ≥ region 实存 lit chunk 数的 99%，缺口率 >1% → VOID——打点遗漏不可静默）+ P09 输出 hash 与 region `.mca` 实读 nibble hash 抽样互验（≥100 chunk 全等，防打点面与实际写出面漂移）。任一失效 → 全体判据 suspended + 比较器 exit 1 `[VOID]`，数据不进判读，status 永不自动变更（§15.4）。
4. **采集流程**：主会话执行命令（subagent 无 shell，§八.12），原始输出落盘交解读；失败轮不回滚重采前先落盘（#146 禁自动回退）。

### 1.4 判据预登记草案（两分支读法写死，#112——采集前写死，禁止事后挑读法）

设比较器按 (cx,cz) 对齐两臂 `[E3A]` 行，集 `S` = 双臂均存在的 chunk：

| 分支 | 机械条件 | 读法（写死） |
|---|---|---|
| **乙直证** | ∃ chunk ∈ S：`blk25(r5)==blk25(r6)` 且输出 hash 集（24 节）不同——且该 chunk 属 d_cross 恒定面成员（对账 t8 json 的 2038 面 key 集） | **输入同而输出异 → 乙读法（确定性内核/边界语义差）获数据层直证**；甲读法对该 chunk 的「终态化固化」解释被证伪（输入既同，固化与否不产生 run 间恒定差——恒定差在内核）。R6 跨线程可见性同步形式清零（同输入双 run 输出 hash 若相同则 R6 亦不成立于该 chunk；若输出 hash 相同则不落本分支） |
| **甲/上游差支持** | ∃ chunk ∈ S：`blk25(r5)!=blk25(r6)` 且该 chunk 属恒定面成员 | **输入异 → 差异源在上游（快照时机/域批几何/邻域装配），非内核本体 → 乙读法对恒定面被削弱**；甲读法（终态化固化，快照异于终态世界）与「上游确定性输入差」家族共同获得支持空间，细分归后续（甲分支辖区） |
| **混合（显式归属，#154 全数轴覆盖）** | 两分支成员集均非空 | **分账式登记**（对齐 T8 分账先例）：逐 chunk 归属计数落盘，不机械归边、交 judge 强制复核——恒定面可能确为「部分内核差 + 部分上游差」并存 |
| **不可判** | 恒定面 key 集与 `[E3A]` 行集对不上（<80% 交集）或 B=0 类空载体 | exit 5 `[NO-JITTER]` 同构：**不是 PASS**（空载体不报通过，#130 家族），登记「判据不可判（数据有效）」，决策矩阵走保守默认行 |

配套机械细则（草案级，定稿入 e3a-criteria.md）：口径三要素（§9.7）= 载体 blocks25 输入流 + 24 节 nibble 输出 sha256 / 覆盖面 = 双臂 E3A 行交集 ∩ 恒定面 key 集 / 与既有口径可比性 = region `.mca` 口径经 P09 抽样互验挂钩、per-chunk light json 口径不同粒度不可直比。等价档位 E1（同构建态双 run），不外推 E2/E3、单 seed 单维度（#162）。

### 1.5 §9.8 副作用与逆登记

| 副作用 | 性质 | 逆（独立字段，不并入 source） |
|---|---|---|
| r6 采集 rmtree 重建 world | 破坏性 in-place | 逆1 = r5 region 归档（跑后 `regions\r5\`）；逆2 = r6 region 归档（`regions\r6\`）；逆3 = 两臂采集 log。**无自动回退**（#146） |
| server.properties level-seed | in-place | 逆 = `.bak-formprobe` 备份件（既有） |
| build.gradle 新增映射行 + Mixin 打点块 | in-place（共享源码树） | 逆 = git 可寻址（单 commit / 单 diff 可 revert，提交信息 `feat(diag): e3a input hash probe`）；诊断件经 `git revert <sha>` 一键摘除，不并拢进其他改动 |
| 新增 `-Dcoreswap.light.e3aInputHash` 消费点 | in-place | 逆 = 同上 commit；且 `check_switch_mapping.py --strict` 跑后登记输出（DEAD/ORPHAN 对账面） |
| 判据/比较器/采集产物 | derived（新命名产物） | 天然合规 |

### 1.6 改码批准影响面小结（给批准人）

- **改码最小面**：2 文件、1 个纯观察打点块 + 1 行映射；不动 S1-S4 副作用、不动 Rust 内核、不动热路径循环。
- **诊断污染防护**：开关默认关、任务级读一次、关闭态近零开销（对齐「诊断代码绝不放热路径每点执行」铁律——本打点为 chunk 级一次，合规形态）。
- **风险**：打点致 dll sha 变化 → 下游所有以 cc4e39fe 为前置的判据 **premise-expired**（§15.1），须在 e3a-criteria 前置集内显式声明新 sha 并登记漂移原因；这是本改码最大连锁面，批准时须知。
> **⚠️ §15.4 取代注记（260920-01）**：本节「打点致 dll sha 漂移 → #202 premise-expired 连锁」判读被实证取代——r5/r6 采集实测 dll sha cc4e39fe 不变（打点纯 Java 面），连锁解除。见 `.investigations/fix-term-260920-01/verdict-260920-01.md` §3.2（supersedes 本节该句；原文不改，仅此注记）。
- **批准后执行序**：改码 → `check_switch_mapping.py --strict` → 编译 → 判据定稿（#112 mtime 序）→ r5/r6 采集 → 比较器 → 判读。

## 2. 若档③证实乙读法：内核语义差修复候选方向清单（只列方向 + 廉价探针，不做深设计）

| # | 方向 | 一句机制 | 廉价验证探针 |
|---|---|---|---|
| K1 | **blocks9 快照 vs 终态 blocks 输入差** | 提交线程快照（Mixin:562-571/550-552）采集时机早于邻域终态，即使「输入跨 run 恒定」也恒错（乙的「上游恒定差」变体——输入 hash 同但同错） | 探针：同一 chunk 在写回时点重读 blocks25 再 hash，对比提交时 hash（差值分布；若恒定差 chunk 的重读 hash 与 Java 参照侧输入一致 → 快照时机差实锤） |
| K2 | **边界条件/邻域装配差** | 域 3×3 分片边界 chunk 的 blocks25 装配：越界/未加载邻节占位值（空 nibble/0 填充）与 Java 语义差 → 恒定错值集中于边界节 | 探针：d_cross 2038 面 chunk 的分片边界对齐率统计（纯数据侧，复用已归档 t8 json + 域 key 几何；若恒定面与边界带强对齐 → 装配差升位） |
| K3 | **内核本体逐位语义差**（坐标/nibble 索引/负坐标 floorDiv/int 溢出族，AGENTS §二易错点清单） | Rust light 内核在某输入类上与 Java 传播语义逐位不一致 | 探针：取输入同 hash 而输出异的实锤 chunk，将其 blocks25 输入字节流固化 → 分别喂 Rust 内核与 Java 参照（Java 侧离线复算或 block_probe Full 口径）→ 差异 nibble 空间分布指纹（是否 (15,15) 列/负坐标集中 → 定位到具体算子） |
| K4 | **sky/block 通道语义差** | SKY 通道（S2）的 sky 角度/柱传播语义差独立于 BLOCK | 探针：2038 面 diff 按 channel 拆分（纯数据侧，region 已含 sky/block 通道）；单通道集中 → 直接收窄到对应 provider |
| K5 | **内联路 vs 域批路 Rust 内核版本漂移** | 域批臂与内联接管路（Mixin:829-837）调用的内核参数/输入形状不一致 | 探针：同 chunk 内联路重跑一次输出 hash 对比域批路（cheap A/B，本档判别域外、仅作方向探针） |

每方向独立可证伪、互不依赖；K1/K2/K4/K5 均可先于改码用已归档数据 + 既有探针做，K3 需 K1 排除后固化输入做。**全部为方向候选，取舍与深设计让渡后续 worker/fan-out（#73）**。

## 3. 双分支决策矩阵（HOOK-4 拍板输入，给主会话）

| 档③ 结果 | 甲方案包（终态化固化读法） | 乙方案包（内核/边界语义差读法） | 拍板建议 |
|---|---|---|---|
| **乙直证**（输入同 hash 输出异） | 甲的恒定面对该 chunk 集解释被证伪 → 甲包中「写回重验/重算」项仍可治标但不治本 | **乙包立项**：按 §2 K1→K5 探针序收窄到具体算子/装配面后修内核（修的是值本体，不是固化时机） | 乙包为主、甲包仅保留 S3+S4 解绑（scout-map §4，治覆写抖动面 2185——该面已被档②归覆写、与乙独立） |
| **甲/上游差支持**（输入异） | **甲包立项**（verdict-260919-07 §1 主读法 + scout-map F-A 接入点表直接可用）；恒定面错值随「写回时重验/重算」自然消解 | 乙包降级为「内核无差」登记；§2 方向清单封存不立项 | 甲包为主；乙面只留档②已裁决的覆写抖动面交 F-B |
| **混合**（两分支成员集均非空） | 甲包立项于「输入异」子集 | 乙包立项于「输入同输出异」子集（K 探针只跑该子集固化输入） | **分账双包**，逐子集验收；judge 强制复核归账 |
| **档③ 未跑 / 不可判**（含 VOID、NO-JITTER、改码未批） | 未裁决 | 未裁决 | **保守默认 = 甲包方向但降级为「双读法兼容设计」**：接入点选型只做对两读法均无损/中性的项（S3+S4 解绑、S3 延迟类——它们治的是已被档② candidate 级证实的覆写面，与恒定面归属无关）；**任何以「恒定面=甲」为前提的深度整改（免全量重光整体重构）不得在未判别前立项**；不得据此把任何读法升 confirmed |

矩阵使用约束：本矩阵只提供方向包选择输入，**status 变更仍走 candidate→judge→用户 confirmed 链**（judge 意见不改 status，§15.4）。

## 4. 诚实登记（未实施 / 盲区 / 外推 / 让渡）

1. **全部内容为方案设计，未实施**：无代码改动、无采集、无命令执行（本 worker 零命令）。判据为草案级，e3a-criteria.md 定稿（含 P01-P09 逐项机械 check 复核）属后续工作。
2. **打点实参形状未核实**：blocks25 vs blocks9+组装面的实际输入形状按 scout-map §1 的「blocks9/ packed 快照」面登记为待核（F 级 Mixin:562-571 是收集点、具体传参宽度未逐行核对本 worker 辖区）——判据定稿前 MUST 先以一手 file:line 核实。
3. **盲区继承**：LightStorage/ChunkLightProvider/ChunkTicketManager yarn 一手源缺失（scout-map §6-1）——K3 探针的 Java 参照侧复算依赖 block_probe Full 口径替代，非 yarn 直读。
4. **外推边界**：本设计与档②同域 = E1、单 seed、单维度 overworld、1.20.1 域批臂专属；1.21.6 无域批 → E-3a 在 1.21.6 无对应实验（内联路若有恒定差需另行立项）。
5. **辖区外让渡（#73）**：甲读法方案包深设计、S3 排序偏差量化（scout-map §6-2）、520 时序归属（§6-3）、修复取舍与风险评估——均不在本 worker 辖区。
6. **K 探针廉价性声明**：K1/K2/K4/K5 的「廉价」以已归档数据复用为前提（t8 json + region 归档在盘），若后续归档被清理则成本上升——引用前须核对归档在盘（#161 世界链同族）。
