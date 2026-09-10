# 260910-05 知识库草稿包（core.worker 只读产出，未改任何现网文件）

> 产出者：知识库 subagent（core.worker 角色，隔离执行，只读 + 只产本草案文件）。
> 已读：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（项目级错误记录规范，优先级最高）→ `knowledge/INDEX.md`（总入口 + 错误台账载体 + 写入规则/记录价值门）→ `.artifacts/perf-closeout-260910-05/{verdict-c7-260910-05.md, verdict-nether-end-260910-05.md, judge-verdict-c7-260910-05.md}` → `.investigations/perf-closeout-260910-05/{record.md, scout-vanilla-crosswrite.md}` → 现网目标文件末尾（`workflow-patterns.md` / `build-tooling.md` / `algorithm-fingerprints.md`）→ **一手产物复核**（`.tmp/perf-reg-260910-05/{results.txt,logs/}`、`cmd-output/diff-*.txt`、`scripts/merge_index.py`、`worldgen-core/src/worldgen_handle.rs`、`java-snapshot/pre/CppBridge.java`、`runtime/1.21.6/java` 树）。
> **纪律**：本文件**只是草稿**——未修改 `knowledge/**`、`versions/**`、`.artifacts/**`、`scripts/**` 任何现网文件，未改代码，未跑任何改盘命令（只做只读核对）。
> **事后注记（主会话，应用后补）**：① §0.4 提到的 nether/end J3/J4 judge **已在本块内完成**（`judge-verdict-nether-end-260910-05.md`，PASS-with-conditions，C1-C10 已应用，含 `eaS-r1` 补跑闭合 end sanity 缺口）；② 应用时编号按 §0.1 建议落地为 workflow-patterns #110/#111/#112、build-tooling #49/#50、algorithm-fingerprints #21（**编号口径注**：`build-tooling.md:854` 存在既有 #96 一条、文件序非单调，故按**追加序尾号**续编）；③ `knowledge/INDEX.md` 的 04 段脚手架残留已由主会话修复（136→131 行）；④ 本稿 §G「不写清单」已遵办。
> **价值门结论（先判后写）**：A（4 条）中 <N>/<N+1> 高价值详写、<N+2> 高价值简记（判据短小但可复用）、#107 家族补充案例 中价值简记；B（2 条 + 1 家族案例）：<M> 中价值简记（环境坑，高优先但内容短）、<M+1> 高价值详写（静默数据销毁）、#26 家族补充案例 中价值简记；C（1 条）中价值简记；E 错误台账 4 条全高价值详写；D/F 为索引与时间线。**判为「不写」的项见 §G**（比多写更重要）。

---

## 0 应用前须知（主会话必读，均为本次只读核对结果）

### 0.1 现网编号现状与占位对照

| 目标文件 | 现网当前最大号 | 位置 | 本批建议 |
|---|---|---|---|
| `knowledge/discovered/workflow-patterns.md` | **#109**（形态为 `### 发现 #109 简记`，非 `##`；`## ` 头最大 = #108） | `workflow-patterns.md:1803`（#109）/ `:1789`（#108） | **<N> = #110、<N+1> = #111、<N+2> = #112**；另 1 条 `#107 家族补充案例（260910-05）`**不占新号**（沿用 #83/#51、#27 家族案例的现网惯例） |
| `knowledge/discovered/build-tooling.md` | **#48**（`## 发现 #48 简记`） | `build-tooling.md:918`（文件末条） | **<M> = #49、<M+1> = #50**；另 1 条 `#26 家族补充案例（260910-05）`**不占新号** |
| `knowledge/discovered/algorithm-fingerprints.md` | **#20**（`## 发现 #20 简记`） | `algorithm-fingerprints.md` 末条 | **<K> = #21** |

> 全文条目正文一律用 `<N>`/`<N+1>`/`<N+2>`/`<M>`/`<M+1>`/`<K>` 占位；应用时按上表与主会话实际定号替换（§D 索引段同规则）。

### 0.2 插入位置（现网末尾实测）

- `workflow-patterns.md` 共 **1818 行**，末条 = `### 发现 #83/#51 家族补充案例（260910-04）…`（`:1812-1818`）⇒ A 组 4 条**顺序追加**到文件末尾。
- `build-tooling.md` 共 **926 行**，末条 = `## 发现 #48 简记…`（`:918-926`）⇒ B 组 3 条**顺序追加**到文件末尾。
- `algorithm-fingerprints.md` 共 **456 行**，末条 = `## 发现 #20 简记…` ⇒ C 条**追加**到文件末尾。

### 0.3 `knowledge/INDEX.md` 现状（有一处需主会话处置）

- 文件共 **134 行**；**真正的**追加段列表止于 `:125` = `> 260910-03 追加：…`。
- `:127-134` 存在 **260910-04 草稿脚手架残留**：`## D INDEX 追加段` + 「**目标文件**/**插入位置**」说明 + ` ```markdown ` 围栏，围栏内才是两段 `> 260910-04 追加：…` / `> 260910-04 R3 追加：…` 正文（即 04 的索引段目前**以代码块形态**留在文件内，未作为正式索引段展开）。
- ⇒ 建议：应用 §D 之前先**处置该残留**（把围栏内两段提升为正式段落或整体上移），再把 `> 260910-05 追加：…` 追加为最后一段。**本稿未改该文件**。

### 0.4 产物契约现状（judge C7 已应用，勿重复登记）

- `.artifacts/perf-closeout-260910-05/index-entry.yaml` **已存在**（4 entries，`:1-20`，含 `judge-nether-end` / `judge-c7` 两条 review）。
- 根 `.artifacts/index.yaml`（共 **1224 行**）**已手工追加** 260910-05 块（`:1188-1224`，含结论摘要注释）；`:1189` 已就地写下「**勿跑 scripts/merge_index.py**：其写回只保留 {id,path,kind,status} 四字段，会丢弃本文件全部注释」——本条即 §B 的 <M+1>（工具契约，本次已回源码复核，见 §B）。
- `cmd-output/MANIFEST-sha256.txt` 已在位（judge N6 已履行）。
- ⚠️ `index-entry.yaml:17-20` 登记了 `perf-closeout-260910-05/judge-verdict-nether-end-260910-05.md`，但该文件**当前不存在**（目录内仅 `judge-verdict-c7-260910-05.md`）⇒ nether/end 的 J3/J4 judge 仍缺位。**本稿所有 nether/end 内容一律标 candidate（judge 待审，confirmed 留用户）**，§F 时间线块同此口径。

### 0.5 本次一手复核发现的两处「引用数字 ↔ 一手产物」不符（勿转抄旧值）

| 项 | 现网文本 | 一手复核（本次只读执行） | 本稿采用 |
|---|---|---|---|
| `pending_writes` 量级 | judge C5 写「实测行跨度 **0 → 21,416**（首行 790 … 末行 21416）」 | `.tmp/perf-reg-260910-05/logs/r3feat-r2.log.err`：`[CA] chunk` 行 **n=1711**，`pending_writes` **min=0 / max=37,893 / 中位=1,575 / p95=12,634**；**首行=790、末行=0**；值 `21,416` 全文件**仅 1 行**（`:1844` = `chunk(-23,-23) … pending_writes=21416`，**既非最大值也非末行**） | **0–37,893，中位 1,575**（与 verdict §1.3 / record §2.2.1 一致）；C5 的「21,416」是**单行样本被误述为跨度上界/末行**，本稿不引 |
| `[Mixin] placedFeature skipped` count | verdict-c7 §1.2 引 `count=36864` | 同为该 `.log`，**末行 = `count=73728`**（终止瞬间值；36864 为中途快照） | **本稿不引该数字**（判据不依赖它） |

> 另：本稿全部引用数字已逐项回一手产物核对通过——`results.txt` 15 行（r3-r2 41s/inflight 23/CALines=0 · r3log-r2 39s/CALines=0/calog=True · r3feat-r2 CALines=1711/stages=5/flags=5 · r3sync-r2 243s/inflight 1 · naS-r1 nether 21s/mixinLines=4761/interceptDim=4761 · nv 56s · ns 69s/inflight 1 · na 21/20s · ev 5s · es 18s · ea 6/6s · ea-r2 serverCpu=86/cores=8.6）；基线 pair `diff-r3sync-r1_vs_r3sync-r2.txt:74` = `375533568 diff=32169 (0.0086%) sections same/diff=91352/331`；`diff-r3-r1_vs_r3-r2.txt:74` = `375541760 diff=34459 (0.0092%) 91294/391`；维度 6 对 diff 文件 `common`/`blocks` 行与 verdict-nether-end §3.1/§3.2 逐格一致。

---

## A `knowledge/discovered/workflow-patterns.md` 拟新增条目

### A1 — 追加到文件末尾（现网末条之后）

```markdown
## 发现 <N>: 「路径在当前出货语义下不可达（空集）」类机制断言的证明三件套 + 复活出口穷尽 + 前置条件登记（260910-05）

- **发现时间 / 发现者 / 置信度 / module**：260910-05（实际 2026-09-10 18:4x-19:2x）；主会话（静态链 + 行为化实测）+ judge（独立静态复核 + 复活出口穷尽 + 「其余共享态皆内容中性」增量）；**candidate**（judge PASS-with-conditions，C1-C9 已应用；confirmed 留用户）；workflow-patterns / 机制断言的可证伪化。
- **来源定位**：`.artifacts/perf-closeout-260910-05/verdict-c7-260910-05.md` §1/§1.2/§1.4；同目录 `judge-verdict-c7-260910-05.md` §一.1 + §三 C5/C8；`.investigations/perf-closeout-260910-05/record.md` §1.2/§2.2.1；同目录 `scout-vanilla-crosswrite.md` §3/§4。
- **背景（为什么需要这条判据）**：上游 judge 要求补「跨 chunk 写序会计」以排除 R3 异步化的写丢失竞争假说（judge-verdict-r3-260910-04.md:117-118）。核验发现该假说的**前提为假**：默认阶段掩码使 Rust `apply_features` 整体不执行 ⇒ `pending_cross_writes` 缓冲结构不可达。
- **机制（静态链四要素，每条带 file:line）**：
  1. **默认值来源**：`java-snapshot/pre/CppBridge.java:69-74 resolveStageMask()`（`prop==null || isEmpty` ⇒ `0b011`＝ SKIP_CARVER|SKIP_FEATURES，见 `:71`；`:82/:128/:148` 三处句柄均 `setFlags(handle, resolveStageMask())`）。
  2. **消费侧门控**：`worldgen-core/src/worldgen_handle.rs:669-671`（`let skip_features = flags & FLAG_SKIP_FEATURES != 0 || env WG_SKIP_FEATURES; if !skip_features { self.apply_features(...) }`；`FLAG_SKIP_FEATURES = 1<<1`，`:148-149`）。
  3. **唯一调用点**：全仓 grep `apply_features` = **定义 `:1036` + 调用 `:671`**（无第二入口）；`fill_chunk_blocks` 的生产调用者唯一 = `api.rs:156`（`wg_fill_blocks_multi`），其余调用者全在 `bin/`、`bin-diag/`（诊断程序，不参与 JNI 出货路径）。
  4. **该函数体内唯一生产/消费点**：`pending_cross_writes` 全仓仅四处 —— `:144` 声明 / `:506` 初始化 / `:1053` 消费 / `:1088` 生产（后两者同在 `apply_features` 体 `:1036-1375` 内）；闭包唯一注入点 `:1283`（`octx.pending_cross`，`ca_min` 门控）。
  5. **同族不可达**：`neighbor_terrain`（`:999-1022`）两个调用点 `:1124`/`:1242` 同在 `apply_features` 内 ⇒ **连邻域读快照预取也不执行**。
- **判据三件套（MUST，可复用——「不执行 / 空集」类断言的标准证明结构）**：
  1. **静态链**：默认值来源 → 门控布尔 → 唯一调用点 → 该函数体内唯一生产/消费点；**每一环都要能全仓 grep 到计数**（「唯一」必须显式排除诊断 bin / 测试程序）。
  2. **行为化负对照**：在**出货配置**下打开该路径的专属日志门（本案 `WG_CA_LOG=1`，mask=3）⇒ 该路径的日志行数 = **0**，且该臂**跑满**（本案 `Processed: 4225 chunks (100.00%) Total time: 0:00:39`）——「跑满却零行」才排除了「没跑到」。
  3. **正对照 + 通道证明**：改配置使路径可达（本案 `-Dcoreswap.rust.stages=5`）⇒ 行数 **>0**（本案 1711）——**同时**证明「改配置的通道真送达」：JVM banner `Picked up JAVA_TOOL_OPTIONS: … -Dcoreswap.rust.stages=5`（送达证据）+ 打印出 `stageMask=5` 的取值来自 `CppWorldgen.getFlags(handle)`（**JNI 回读**，即「sysprop → Java 解析 → setFlags → Rust 存值」全链到位）。双通道缺一不可（#37/#81 家族）。
- **复活出口穷尽（judge 增量，MUST 做——只说「这条路径不可达」不够）**：逐项核对**其它**顺序敏感共享可变状态是否也内容中性：① `beardifiers`（`:116` RwLock<HashMap<(chunk)>`>）只按 chunk 键读写（写 `:536`、读己键 `:710/:857`），其**全局清空**出口 `clear_beardifier()`（`:539-540`）的唯一 JNI 函数 `Java_wg_CppWorldgen_clearBeardifier`（`jni_bridge.rs:267`）在 `runtime/1.21.6/java` 全树 grep `clearBeardifier` = **0 命中**（`CppWorldgen.java:78` 只声明 `setBeardifier`）⇒ 全局清空不可达 ⇒ 非顺序敏感内容状态；② `terrain_cache`（`:631-635`）与 `est_l2`（`aquifer.rs:248-288`）为**纯 memo**（命中后 `clone()` 再跑 features，无别名写）；③ `api.rs` 导出面无任何跨写投递/查询通道。⇒ 出货语义下 Rust 侧退化为「**per-chunk 纯函数 + 纯 memo**」，完成序自由度对内容无影响——**这比「该路径不可达」强**（后者只否一条通道）。
- **「不可达」必须写精确（MUST）**：出货语义（mask=3）下不可达 ⇒ **不可在出货配置内测量**（任何会计计数恒 0）；`mask=1/5` 下**可达且可测**（本块正对照臂即证）但**不是出货语义**（`mask=1` = 接管臂、`mask=5` = 本块正对照组合，非接管配置）。⇒ 「不实现会计计数」的理由 = 「前提已证空集 + 实现后只能在不属于出货语义的配置下被验证」，不是「跳过不做」。
- **前置条件登记（未来重启该路径前 MUST 先做，本节即登记载体）**：① 会计不得复用现成全局计数——`CA_PENDING_WRITES`（`:1081` 声明、`:1084` 每 chunk `store(0)`、`:1087` 累加）是**全局 atomic**，23 线程并发下「本 chunk 读数」被跨线程写污染 ⇒ 只能作**量级/上界**（一手日志实测 n=1711、区间 **0-37,893**、中位 1,575；不得读作 per-chunk 精确值）；② 会计口径须新增「**迟到写**（目标已跑 features）/ **已生效** / **终态常驻**」三数，且**按 chunk 或按线程分桶**（thread-local / chunk 作用域）；③ 完整语义分歧面清单 = `scout-vanilla-crosswrite.md` §4 的 A1-A7 + C1/C2（含**越界高度图哨兵 `min_y-1`**——Ore 的 s/t 扫描必然跨界 ⇒ 可达；以及**混合语义**：Java 让位只 redirect `PlacedFeature.generate`，结构件放置段仍在 Java ⇒ 重启后是「Rust features + Java structures」）。
- **家族索引**：#20/#53（默认值当公理——本条是其**证明侧**：默认值决定路径可达性）、#37/#81（生效证据必须行为化——本条给出「**不**生效」的行为化对偶）、#66（同 dll 两种执行语义 / mask 三态）、#42（静态机制断言未实测当公理——本条给出「静态链 + 行为化互证」的模板）、#105/#36（执行体与覆盖面口径）。
```

### A2 — 紧随 <N> 之后追加

```markdown
## 发现 <N+1>: 内容等价门的噪声锚应是「同配置 run-to-run 实测基线」，不是跨实现代理基线——且锚必须分形态/分维度（260910-05）

- **发现时间 / 发现者 / 置信度 / module**：260910-05；主会话（本轮新采 5 对读数 + 逐格回一手 diff 文件）+ judge（§9.7 三要素/口径边界复核）；**candidate**（judge 条件支持 confirmed，C1-C9 已应用；confirmed 留用户）；workflow-patterns / 等价门口径。
- **来源定位**：`.artifacts/perf-closeout-260910-05/verdict-c7-260910-05.md` §2/§2.1/§2.2/§4；`.investigations/perf-closeout-260910-05/{record.md §3, cmd-output/diff-*.txt}`（数字在 `:74`）；`.artifacts/perf-closeout-260910-05/judge-verdict-c7-260910-05.md` §3/§4。
- **观察（锚升级实测）**：升级前锚 = **跨实现代理基线 0.0150%**（R3 前的跨实现读数，`perf-regression-260910-04/cmd-output/diff-result-historical-260910-03.txt:74`）。本轮实测**同配置 run-to-run**：sync `0.0086%`（`375,533,568 / 32,169 / sections 91352-331`）、async `0.0092%`（`375,541,760 / 34,459 / 91294-391`）；跨形态 `0.0122%`/`0.0130%`；跨实现 `0.0111-0.0148%`；全部 ≤ 0.0150%。
- **判据（MUST，可复用）**：
  1. **等价门的噪声锚 MUST 取「同配置 run-to-run 实测」**（同形态 + 同载具 + 同 seed 同 region + 同工具修订）——跨实现代理基线是「实现差 + 形态差 + run 级非确定」的混合量，**只能作上界**；用它会把一个真实存在的分量判成「噪声内」⇒ **因果指认错**（本案：代理基线 0.0150% > 同配置基线 0.0086-0.0092%，跨形态读数比同配置基线高约 **0.004pp**，「差异全部由 run 级非确定解释」在新锚点下不成立）。
  2. **锚必须分形态、分维度采集**：同形态自身非确定随执行序自由度变化（本案 async 点估计略高于 sync，+7%，方向与 #67 预期一致；但每形态仅 1 对读数 ⇒ 只能判「同阶」，**既未证实也未否证**）；维度间量级可差 ~15×（见 algorithm-fingerprints <K>）。
  3. **门的性质 MUST 写死**：「同量级筛选门」≠「内容零差」；**禁令措辞**——不得写「内容零差 / 精确 1.000 / 绝对无内容影响」（本块证据反而不支持：存在**未归因**的形态相关分量）。升级锚 ≠ 升级结论强度。
  4. **数字必须随口径一起声明**（§9.7 三要素 + 两条边界）：载具/覆盖面/可比性；**分母语义** `blocks = 4096 × 两侧 section 并集`（全 air/缺失 section 不计，≈11.5/24 section）⇒ `%` 的分母不是单臂满额；**chunk 集缺口**（比对集 `common=7959` vs 本臂 Chunky 只生成 `4225`，差额含 spawn 预生成等非本臂 chunk）。
- **派生动作（与 §15.4 联动）**：以新锚点数上游已 confirmed 正文的措辞 ⇒ **不删改原文**，改记取代记录（supersedes 双指针 + 一行推翻理由，本案见 04 `verdict-260910-04.md:106`）；取代**理由**在两种触发器读法下都成立（见 <N+2>）。
- **家族索引**：#51（噪声基线与信号同阶——本条给「锚怎么选」）、#33（载具可比性——本条为跨实现/跨形态口径面）、#67（run 级非确定）、#18/#103（跨 run / 跨批绝对值不可引）、#83（性能/内容分母分场景）。
```

### A3 — 紧随 <N+1> 之后追加（简记）

```markdown
### 发现 <N+2> 简记: 预登记判据的「读法」必须在预登记时写死——同一句「基线与三对同量级或更大」在数量级读法与严格数值读法下结论相反（260910-05）

- **发现时间 / 发现者 / 置信度 / module**：260910-05；主会话（预登记原文核对 + 两读法演算）+ judge（要求显式声明所采口径，C4）；**candidate**；workflow-patterns / 判据纪律。
- **来源定位**：260910-04 的 C7-② **预登记触发器原文** =「基线与三对**同量级**或更大」；本块处置 = `.artifacts/perf-closeout-260910-05/verdict-c7-260910-05.md` §4；judge = 同目录 `judge-verdict-c7-260910-05.md` §一.7。
- **观察**：实测基线 0.0086/0.0092% vs 三对 0.0111-0.0130%（比 1.35-1.5×）——**数量级读法** ⇒ 同数量级 ⇒ 触发器**成立**（应记 §15.4 取代）；**严格数值读法**（基线 < 三对）⇒ 触发器**不成立**；项目既有第三种读法（**±20% 跨 run 摆动带**）⇒ 基线比三对低 21-34% ⇒ 也不成立。两种读法之差 = **是否显式记取代**（结论本身不受影响）。
- **判据（MUST）**：预登记触发器 MUST 三件齐写：① **读法**（数量级 / 严格比较 / 摆动带容差）；② **比较对象的形态**（同形态 / 跨形态 / 跨实现）；③ **触发后的动作**（记取代 / 重审 / 仅登记）。否则「事后可挑选读法」= 判据失效（同一实测数据可支持相反处置）。若预登记原文含糊：裁决时 MUST **显式写出所采口径与数值**，**不得**以未预登记的判据（本案「未见异常放大」）替代而不声明。
- **附带手法**：把「**动作**」（是否记取代）与「**理由**」（因果指认是否被推翻）拆开写——本案动作随读法而变，理由在两种读法下都成立，拆开后表述不随读法摇摆。
- **家族索引**：#51（噪声基线）、#18（跨 session 数字可比性）、#63（命题必须限定）、#90（措辞转抄漂移）、#94（简报/历史字段不进一手核对）。
```

### A4 — 紧随 <N+2> 之后追加（家族补充案例，不占新号）

```markdown
### 发现 #107 家族补充案例（260910-05）: 「把重活搬出被串行化执行点」在其它维度/载体推广——形态证据跨维复现 + 逐维自噪声门 + 「接管基线差异落在噪声带内 ≠ 接管已验证」

- **发现时间 / 发现者 / 置信度 / module**：260910-05；主会话（同构建态单变量 A/B ×2 维 + 接管生效 sanity）；**candidate**（nether/end verdict 仍待 judge J3/J4，confirmed 留用户）；workflow-patterns / 接管形态推广（#107 修复侧的载体维）。
- **来源定位**：`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md` §1-§4；`.investigations/perf-closeout-260910-05/record.md` §5；一手 `.tmp/perf-reg-260910-05/results.txt`（九臂）+ `cmd-output/{diff-N_*,diff-E_*}.txt:74`。
- **观察（同款改造，两个新维度复现）**：nether `69s → 20/21s`（**3.3-3.5×**）、end `18s → 6s`（**3.0×**）；`inflight max` **1 → 23 两维复现**；相对 vanilla：nether async **反超 2.7-2.8×**（56s → 20/21s）、end async `6s ≈ 5s`（sync 形态 `18s` 反而**慢 3.6×**）⇒ **单车道形态的净亏损是形态通例，不是 overworld 特例**；Rust 侧零改动 ⇒ 执行体与 260910-04 同源（`dllsha=abd7d8893d22e030` 每臂核对）。
- **判据（MUST，推广#107 三件套）**：
  1. **先在目标维度做「接管生效 sanity」再开 A/B**：本案 `chunky world minecraft:the_nether` 被接受 + `[Mixin] populateNoise(nether) intercepted` = **4761** 行 + `[WG-FILL]` = **4761** 行（两项均依赖 `-Pmixlog=1`），region 采集自 `run\world\DIM-1\region`。**否则「新维度根本没跑」会伪装成「优化无效」**（同族 #20/#53/#81）。
  2. **行为门的判据 MUST 用该维度自身的同形态噪声**：nether 跨形态 `0.1219%` ≤ 同形态噪声 `0.1406%` ⇒ PASS；end 跨形态 **0 块差** = 同形态噪声 **0** ⇒ PASS（位级一致，强于 nether 的门）。维度间噪声量级可差 ~15× ⇒ 跨维引用判据无效（#33 维度面）。
  3. **「接管基线（sync × vanilla）差异落在自身噪声带内」≠「接管已验证」**：nether 接管基线 `0.1020%` < 自身噪声 `0.1406%`，只能读作「差异未超出 run 级噪声」——1.21.6 nether/end 接管属**首次验证**，不能据此宣布对齐。
  4. **形态证据以 `inflight max`（1 → 23）为准，不以日志行数**：A/B 臂按设计未开 `-Pmixlog`（双臂对称），`interceptDim=0` 属预期；日志门控只影响日志不影响行为（门控改动本身需双臂对称声明）。
- **家族索引**：#107（主条 + 修复验证）、本批 <N+1>（噪声锚分形态/分维度）、#33（载具/维度可比性）、#26（Chunky 区域级载体）、#103（机器噪声带）、#51（噪声与信号同阶）。
```

---

## B `knowledge/discovered/build-tooling.md` 拟新增条目

### B1 — 追加到文件末尾（现网末条 #48 之后）

```markdown
## 发现 <M> 简记: PowerShell `pwsh -File script.ps1 -Arms a,b,c` 把逗号串当**单个字符串**——多值参数必须用数组 `@("a","b")`（260910-05）

- **发现时间 / 置信度 / module**：260910-05；**签名侧确定**（一手报错原文 + 修复后跑通）；build-tooling / 驱动脚本参数传递。
- **来源定位**：`.investigations/perf-closeout-260910-05/record.md` §2.1「踩坑 E1」；脚本 `.investigations/perf-closeout-260910-05/cmd-output/run_arms3.ps1`（臂定义处）。
- **现象**：`pwsh -File run_arms3.ps1 -Arms r3,r3log,r3feat,r3sync` ⇒ 脚本报 **`unknown arm r3,r3log,r3feat,r3sync`**——被拒的名字是**带逗号的整串**，即逗号串作为一个臂名进入参数（零臂可跑）。
- **根因（机制）**：逗号数组语法在 **`-File` 的实参串**上未生效，整串被当成单个字符串绑定到数组形参（得到长度为 1 的元素）。⚠️ 本块未做 PowerShell 参数绑定的最小复现 ⇒ **机制侧 Degraded（表述以实测签名为准）**，签名侧确定。
- **定位**：看**报错里的名字形态**即可判别——出现「带分隔符的整串」= 分隔符未解释；若只取到第一个名字则是另一类签名（绑定截断）。本案属前者。
- **修复**：调用侧显式构造数组 `& <script> -Arms @("r3","r3log",...)`（或在脚本内对字符串参数做 `-split ','` 兜底）。
- **教训/判据**：① `-File` 传参一律按**字符串语义**预期，多值 MUST 用 `@(...)` 显式数组；② 驱动脚本启动时**回显实际收到的参数**（本案 results 行含 `args=`/`stages=`/`calog=`/`world=`，使每臂口径事后可回查）——#37/#81「生效证据必须行为化」的**编排侧**形态；③ 批量跑批脚本的臂名解析失败应**立即非零退出**（本案是响亮失败，优于 #20/#53 家族的静默不生效）。
- **家族索引**：#37/#81（生效证据行为化）、#20/#53（参数静默不生效——本条为「响亮失败」对偶）、#28（冒烟/存档口径参数集）。
```

### B2 — 紧随 <M> 之后追加

```markdown
## 发现 <M+1>: `merge_index.py` 写回根 index 是「解析 + **重建**」——注释与四字段以外的一切被静默丢弃（本仓库根索引 307 行注释承载结论摘要）⇒ 带注释的根索引禁跑该工具（260910-05）

- **发现时间 / 置信度 / module**：260910-05；**确定**（一手读工具源码 + 现网根索引实测注释行数；本块已按规避方案执行）；build-tooling / index 合并链（#48 同脚本的**第二形态**：一个是崩溃，一个是静默损毁）。
- **来源定位**：`scripts/merge_index.py`：`:37` `ENTRY_KEYS = ('id','path','kind','status')` + `:49-56` `norm_entries`（每条 entry 白名单化为四字段，`:55` = `{k: e.get(k,'') for k in ENTRY_KEYS}`）+ `:129-139`（写回时**新建整个文档** `{schema_version, project, module, entries}` 后 `yaml.safe_dump` **覆写**根文件）；现网 `.artifacts/index.yaml`（共 1224 行，以 `#` 开头者 **307 行**）；规避实例 = `.artifacts/index.yaml:1188-1189`（手工追加 + 就地写下「勿跑」警告）+ `record.md` §1.3。
- **现象/风险**：对该根索引跑合并后，**全部注释**与**非四字段信息**消失（本仓库根索引正用注释承载各块结论摘要，如 `:1194-1204` 的 260910-05 块）；无警告、无备份、**退出码 0**。
- **根因（机制）**：工具是 **parse → 重建 → dump**，而不是「原地编辑」。三处叠加：① 条目被 `norm_entries` 白名单化成四字段（未知/扩展字段丢）；② 写回 dict 只含四个顶层键（顶层其它键丢）；③ YAML 加载器**不保留注释**（注释丢）。⇒ 只要写回路径经过 parse/dump，**注释必然丢**（语言层面事实，无需复现）。
- **定位**：读脚本「写回」段（看它是 edit 还是 rebuild）+ 数现网目标文件的注释行数（307）+ 对照本仓库根索引的实际用法（注释型承载）⇒ 直接判定不兼容。
- **修复/规避**：**带注释的根索引 MUST NOT 跑 `merge_index.py`**——改**手工追加** entry（本案做法；或先备份注释、跑完恢复）。根治方向：工具改 ruamel.yaml round-trip，或把结论摘要从注释迁为条目字段。现网已在 `.artifacts/index.yaml:1189` 就地写下警告。
- **教训/判据**：① 对「**承载结论/说明的索引文件**」跑任何自动合并工具前，MUST 先核「写回是**原地编辑**还是**重建**」——重建式工具 = 注释与未知字段一律丢，且通常静默；② 本工具的契约是「**四字段 entries 集合**」，**不是**本仓库根索引的维护器：#48（片段形态崩溃）+ 本条（写回语义）合起来 = 两侧都不适配，本仓库应固定用「手工追加 + 注释承载」；③ **静默数据销毁签名** = 退出码 0 + 无警告 + 目标文件结构被替换（比报错危险得多，同族 #27 existence-only marker 静默 fallback、#16「死分支」）。
- **家族索引**：#48（同脚本 · 片段形态面）、#27 家族（静默退化/静默销毁）、#18（产物在盘 ≠ 本次生成——本条为「工具成功 ≠ 内容保留」）、#10（产物判新旧用内容指纹）。
```

### B3 — 紧随 <M+1> 之后追加（家族补充案例，不占新号）

```markdown
### #26 家族补充案例（260910-05）: Chunky 载具的**维度扩展**——`chunky world minecraft:the_nether|the_end` 可用 + region 路径 `DIM-1`/`DIM1` + 首用 sanity 判据

- **发现时间 / 置信度 / module**：260910-05；确定（一手九臂 `results.txt` + 日志）；build-tooling / 验证载体（#26 的维度面）。
- **来源定位**：`.investigations/perf-closeout-260910-05/record.md` §5.2/§5.3；`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md` §2；一手 `.tmp/perf-reg-260910-05/results.txt`（`naS-r1` 行：`world=minecraft:the_nether … mixinLines=4761 interceptDim=4761`）。
- **内容（三点）**：① 命令 `chunky world minecraft:the_nether` / `minecraft:the_end` **被接受**（`Task finished for … Processed: 4225 chunks (100.00%)`）；② 存档 region 路径按维度分目录——nether = `run\world\DIM-1\region`、end = `run\world\DIM1\region`（驱动脚本用 `reg` 字段显式指定，别照抄 overworld 的 `region`）；③ 首用 **sanity 判据** = `[Mixin] populateNoise(nether|end) intercepted` 行数 **+** `[WG-FILL]` 行数（本案两维各 4761 = 4761）；两者均依赖 `-Pmixlog=1`，故 **A/B 臂未开日志时 `interceptDim=0` 属预期**，不能读成「接管没生效」（形态证据改用 `inflight max`）。
- **判据**：新维度载具首用 MUST 先过**三查**再开 A/B——维度名被接受 / 接管生效有行数（含写回行）/ region 路径存在；「接管计数为 0」先核**日志门控是否开**（#25/#8 家族门控），再怀疑管线。
- **家族索引**：#26（Chunky 区域级载体——本条为其维度扩展）、#25（mixin 门控 sysprop/env）、#80（时序/触发条件）、#107 家族补充案例（本批 A4——本条为其**载体侧**判据）。
```

---

## C `knowledge/discovered/algorithm-fingerprints.md` 拟新增条目

### C1 — 追加到文件末尾（现网末条 #20 之后）

```markdown
### 发现 <K> 简记: MC 维度确定性指纹——同载体同 seed 下 run 级非确定是**维度属性**：overworld ~0.009%、nether ~0.14%（~15×）、end = 0（端到端确定）（260910-05）

- **发现时间 / 发现者 / 置信度 / module**：260910-05；主会话（region 全域逐块对拍，每维同形态两 run）；**candidate**（每形态 1 对点估计）；algorithm-fingerprints / 维度指纹（载体选择与判门前提）。
- **来源定位**：`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md` §3.1/§3.2/§5.3；同目录 `verdict-c7-260910-05.md` §2.1；一手 `.investigations/perf-closeout-260910-05/cmd-output/`：`diff-N_async_vs_async2.txt:74`（nether 同形态 `0.1406%`）、`diff-E_async_vs_async2.txt:74`（end 同形态 **0**）、`diff-r3-r1_vs_r3-r2.txt:74`（overworld async 同形态 `0.0092%`）。
- **指纹（同载具 1.21.6 + Chunky radius 500 / 同 seed / 同 region 中心 / 同工具修订，region 全域逐块普查）**：

  | 维度 | 同形态 run-to-run（async） | 跨形态（async × sync） | 接管 vs vanilla | 读数 |
  |---|---|---|---|---|
  | overworld | **0.0092%**（sync 0.0086%） | 0.0122% / 0.0130% | 0.0111-0.0148% | 低噪声维 |
  | nether | **0.1406%** | 0.1219% | 0.1020% | **~15× overworld** |
  | end | **0** | **0**（块差 0） | **7 块 / 494M** | **端到端确定** |

- **判据（MUST）**：① **维度间不可互引噪声基线**（#33 的维度面）——选载体 / 判门 MUST **先测该维自身** run 级噪声，禁拿 overworld 的 0.009% 去判 nether 的 0.12%；② **确定性维度上「非零差异即信号」**——end 噪声 = 0（两 run 逐块零差）时，等价门可升级为**位级门**（本案跨形态 0 块差是比「≤ 噪声」更强的证据）；③ 反之在 nether 这类高噪声维上，0.10-0.12% 级差异**落在噪声带内不构成任何结论**，措辞须写「未超出 run 级噪声」而**不得**写「已验证/已对齐」；④ 本指纹**机制未定**（候选：nether 结构/熔岩湖等按 chunk 完成序放置，或 region 内含非本臂生成的 chunk）⇒ 指纹可作判据使用，成因标 open（不阻塞使用）。
- **家族索引**：#33（载具/维度可比性）、#51（噪声基线与信号同阶）、#67（run 级非确定）、#83（分母/场景）、#26（载具）。
```

> **载体选择说明**：本条亦可归入 `workflow-patterns.md` 作为 #33/#51 家族补充案例（它是方法论判据）；本稿按现网 `algorithm-fingerprints` **#20 简记「MJT0 树基高 h 指纹作探针」**的先例放在指纹类（属性属"算法/载体指纹"），**主会话可改归**——改归时正文零改动，仅需同步 §D 索引段措辞。

---

## D `knowledge/INDEX.md` 拟追加段

**目标文件**：`knowledge/INDEX.md`。
**插入位置**：处置 §0.3 所述 `:127-134` 残留脚手架后，**追加到文件末尾**（现网真实末段 = `> 260910-03 追加：…`，`:125`）。
**占位替换**：`<N>`→#110、`<N+1>`→#111、`<N+2>`→#112、`<M>`→#49、`<M+1>`→#50、`<K>`→#21（以主会话实际定号为准）。

```markdown
> 260910-05 追加：workflow-patterns 新增**发现 <N>**（「路径在当前出货语义下不可达（空集）」类机制断言的**证明三件套** + 复活出口穷尽 + 前置条件登记——静态链四要素（默认值来源 → 门控 → 唯一调用点 → 体内唯一生产/消费点，`CppBridge.resolveStageMask()` 默认 `0b011` → `worldgen_handle.rs:669-671` 不进 `apply_features` → `pending_cross_writes` 生产 `:1088`/消费 `:1053` 同在其中，唯一注入点 `:1283`）+ 行为化负对照（mask=3 + `WG_CA_LOG=1` ⇒ 零 `[CA]` 行且该臂跑满 4225 chunks）+ 正对照与双通道证明（mask=5 ⇒ 1711 行；JVM banner 有 `-Dcoreswap.rust.stages=5` + `stageMask=5` 系 JNI 回读）；judge 独立复核穷尽复活出口后结论**更强**（`beardifiers` 全局清空出口在 Java 全树 grep 0 命中、`terrain_cache`/`est_l2` 纯 memo 无别名写 ⇒ 出货语义下 Rust 侧 = per-chunk 纯函数 + 纯 memo）；前置条件登记（重启该路径前 MUST 加「迟到写/已生效/终态常驻」三数且按 chunk/线程分桶，**不得**复用全局 `CA_PENDING_WRITES`——23 线程并发污染，实测区间 0-37,893、中位 1,575）+ **发现 <N+1>**（内容等价门的**噪声锚**应取「同配置 run-to-run 实测基线」而非跨实现代理基线，且锚分形态/分维度——同配置 sync 0.0086%/async 0.0092% vs 跨形态 0.0122%/0.0130%、跨实现 0.0111-0.0148%、历史代理基线 0.0150%；用代理基线会把真实的形态相关分量判成「噪声内」⇒ 因果指认错；数字须随 §9.7 三要素 + 分母语义（`4096 × 两侧 section 并集`）+ chunk 集缺口（`common=7959` vs 本臂 4225）一并声明）+ **发现 <N+2> 简记**（预登记判据的「读法」必须预登记时写死——「基线与三对同量级或更大」在数量级/严格数值/±20% 摆动带三种读法下结论相反，事后可挑选读法 = 判据失效）+ **#107 家族补充案例（260910-05）**（同款异步化在 nether/end 复现：69s→20/21s = 3.3-3.5×、18s→6s = 3.0×、`inflight max` 1→23 两维复现，nether 相对 vanilla 反超 2.7-2.8×、end 6s≈5s（sync 18s 慢 3.6×）⇒ 单车道净亏损是形态通例；推广三件套 = 目标维先做接管生效 sanity（`[Mixin] populateNoise(nether|end) intercepted` 4761 行 + `[WG-FILL]` 4761 行，依赖 `-Pmixlog=1`；region 路径 `DIM-1`/`DIM1`）+ 行为门用**该维自身**同形态噪声（nether 0.1219% ≤ 0.1406%）+ 「接管基线差异落在噪声带内 ≠ 接管已验证」）；algorithm-fingerprints 新增**发现 <K> 简记**（**维度确定性指纹**——同载体同 seed 下 run 级非确定是维度属性：overworld ~0.0092%、nether ~0.1406%（~15×）、end **= 0**（端到端确定）；判据 = 维度间不可互引噪声基线 + 确定性维上「非零差异即信号」（可升位级门）；成因未查标 open）；build-tooling 新增**发现 <M> 简记**（PowerShell `pwsh -File script.ps1 -Arms a,b,c` 把逗号串当**单个字符串**（报错 `unknown arm a,b,c`）⇒ 多值参数必须 `@("a","b")`；判别签名 = 报错里出现带分隔符的整串）+ **发现 <M+1>**（`merge_index.py` 写回根 index 是「解析 + **重建**」——`:37` 四字段白名单 `norm_entries`（`:49-56`）+ `:129-139` 重建整个文档 + YAML 不保留注释 ⇒ 注释与四字段以外一切**静默丢失**（现网根索引 307 行注释承载结论摘要），退出码 0 无警告；⇒ 带注释的根索引 **MUST NOT** 跑该工具，改手工追加（现网 `.artifacts/index.yaml:1189` 已就地写下警告）；与 #48（片段形态崩溃）同一脚本的两侧不适配）+ **#26 家族补充案例（260910-05）**（Chunky 载具维度扩展——`chunky world minecraft:the_nether|the_end` 可用 + region 路径 `DIM-1`/`DIM1` + 首用 sanity 三查；A/B 臂未开 `-Pmixlog` 时 `interceptDim=0` 属预期）；错误台账 `.investigations/perf-closeout-260910-05/perf-closeout-errors.md` 新增 **E1-E4**（`pwsh -File` 逗号串单字符串 · CPU 采样点**重复覆盖** ⇒ `serverCpu=-1` 恒成立（初诊「旧解析版本」被证伪，真根因 = 同名变量两处赋值、后者在 stop 之后）· 草稿把 async 0.0092% > sync 0.0086% **写成「async 未比 sync 更大」**（方向写反，judge C3 纠正；「不显著」≠「方向相反」）· 汇总表/注的数字未逐格回一手文件复核（两处 `blocks` 与一手 diff 矛盾、其中一处全树 0 命中，且与既有值撞值 = 未复核签名；终止点数字核不到——judge C1/C2，与 260910-04 C1 **同族二犯**））。来源：`.artifacts/perf-closeout-260910-05/{verdict-c7-260910-05.md,verdict-nether-end-260910-05.md,judge-verdict-c7-260910-05.md}` + `.investigations/perf-closeout-260910-05/{record.md,scout-vanilla-crosswrite.md}`（C7 收口 = candidate，judge PASS-with-conditions 条件 C1-C9 已应用，confirmed 留用户；nether/end = candidate，judge J3/J4 待审）。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260910-05 块。
```

---

## E 错误台账草稿（文件全文，待主会话落盘到 `perf-closeout-errors.md`）

**应写入路径**：`.investigations/perf-closeout-260910-05/perf-closeout-errors.md`（**新文件**，当前目录下不存在；项目级错误台账载体 = `.investigations/<课题>/<课题>-errors.md`，见 `knowledge/INDEX.md`「错误台账载体」节）。

```markdown
# 260910-05 · C7 收口 + nether/end 异步化推广 错误台账

> 课题：`.investigations/perf-closeout-260910-05/`（C7 收口 + nether/end 异步化推广；实际 2026-09-10 18:4x-19:2x，日期锚 `Get-Date` 2026-09-10 18:45）
> 载体：CoreSwap 项目级指定错误台账（独立成篇）——`knowledge/INDEX.md`「错误台账载体」节 + `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md` §三
> 格式：五段式（现象 / 根因 / 定位 / 修复 / 教训）+ 末尾「错误→根因」速查表
> 来源：`.investigations/perf-closeout-260910-05/record.md`（§2.1 E1 / §2.2 E2）+ `.artifacts/perf-closeout-260910-05/{verdict-c7-260910-05.md,judge-verdict-c7-260910-05.md}` + 一手日志与脚本 `.tmp/perf-reg-260910-05/`（`run_arms3.ps1` / `results.txt` / `logs/`）
> 置信度：E1/E2 = 本块一手实测（确定级）；E3/E4 = 草稿被 judge 逐格核对后更正的事实（一手产物已由本台账复核）
> 编号说明：本台账属**新课题目录**（`perf-closeout-260910-05`），按「同一课题目录内续号、换课题目录重起」惯例自 **E1** 起。上游 `.investigations/perf-regression-260910-04/perf-regression-errors.md` 的 **E1-E8 为另一课题目录**的编号，勿混引（若主会话希望两册连续编号，只需整体改为 **E9-E12**，正文零改动）。

## E1: `pwsh -File script.ps1 -Arms a,b,c` 把逗号串当**单个字符串** —— 多值参数必须用数组 `@("a","b")`

- **现象**：跑批驱动脚本按臂清单传参 `pwsh -File run_arms3.ps1 -Arms r3,r3log,r3feat,r3sync`，脚本立刻报 **`unknown arm r3,r3log,r3feat,r3sync`**——报错里被拒的名字是**带逗号的整串**（一个臂都没跑起来，本批首轮零产出）。
- **根因（机制）**：`-File` 调用下逗号串**未被解释为数组分隔符**，整串作为一个字符串绑到数组形参（长度 1 的元素），脚本的臂名匹配遂整体落空。⚠️ 机制表述为推断（本块**未做** PowerShell 参数绑定的最小复现）⇒ 机制侧 Degraded；**签名侧确定**（报错原文即证）。
- **定位**：看报错文本里名字的**形态**——出现「带分隔符的整串」⇒ 分隔符未生效；若只报第一个名字，则属另一类签名（绑定截断）。本案是前者，一步可判，无需调试脚本逻辑。
- **修复**：调用侧显式构造数组：`& .\run_arms3.ps1 -Arms @("r3","r3log","r3feat","r3sync")`（兜底：脚本内对字符串参数 `-split ','`）。
- **教训**：① `-File` 传参一律按**字符串语义**预期，多值一律 `@(...)`；② 批量脚本启动时**回显实际收到的参数**（本案 `results.txt` 每行含 `args=`/`stages=`/`calog=`/`world=`，使口径事后可回查）——#37/#81「生效证据必须行为化」在**编排侧**的形态；③ 这类失败是**响亮的**（脚本直接拒），优于 `-P` 映射静默不生效家族；遇到时先修调用侧语义，别改脚本内部解析。

## E2: CPU 采样点**重复覆盖** ⇒ `serverCpu=-1 / cores=-1` 恒成立 —— 初诊「本批仍走旧解析版本」被证伪

- **现象**：本批前四臂 `results.txt` 全部 `serverCpu=-1 cores=-1`（含 `r3-r2`/`r3log-r2`/`r3feat-r2`/`r3sync-r2`，以及 nether/end 部分臂）；唯 `ea-r2` 采到 `serverCpu=86 cores=8.6`（**脚本修好之后**跑的臂）。⇒ CPU 证据整批缺失，幅度口径被迫改用 wall。
- **根因（机制）**：驱动脚本里 **CPU 采样块出现两处**——停服**前**（此前修复时加入，正确）与停服**后**（原版残留），后者在 java 已 kill 之后再次采样并**覆盖** `$cpuSec` ⇒ 采到的恒为 `-1`。**同名变量两处赋值、后者位于 stop 之后** ⇒ 无论前一处采得多好都被覆盖。（此前记录写的「本批 r3sync 仍走旧解析版本」是**误判**：脚本的解析逻辑不是问题，问题是**覆盖顺序**。）
- **定位**：① 先复现「恒 -1」的**不变性**——四臂同值 ⇒ 不是测量偶然，是**写入路径**问题；② 排除「解析版本/Python 环境」假设（同一脚本解析逻辑在 `ea-r2` 上采到了 8.6 核 ⇒ 解析能力在位）；③ 回到脚本全文搜同名变量赋值 ⇒ **两处采样块**，后者在 `kill/stop` 之后（`.tmp/perf-reg-260910-05/run_arms3.ps1` 原 `:112-113`，已删）。
- **修复**：删掉停服后的重复采样块（保留停服前那处）⇒ 后续臂（`ea-r2` 起）恢复采集。
- **教训（判错经验，可复用）**：① **「某读数恒为常数/恒为 -1」先查写入路径，不要先查解析器/工具版本**——恒值签名指向**赋值覆盖**或**未初始化**，不指向解析能力（解析错通常产生**多样**的错值，而非恒定值）；② 具体动作 = 在同一文件内 grep 同名变量**全部赋值点**，核**顺序**（尤其「在 kill/stop/teardown 之后是否还有一次赋值」）；③ 「用旧脚本/旧版本」是最省事的假设，也最容易**掩盖真根因**——本案该假设让我们把「覆盖」误记成「版本」；④ 判据升级：任何「恒值」异常，**先做覆盖链核对，再做环境归因**。

## E3: 草稿把 `async 0.0092% > sync 0.0086%` 写成「async **未比** sync 更大」——点估计方向写反（judge C3）

- **现象**：260910-05 知识库/verdict 草稿 §2.1 原文写「async 自身 run 间非确定**未比** sync 更大 / 该预期未被实测支持」；而**同一张表**的数字是 sync `0.0086%`（32,169 块）、async `0.0092%`（34,459 块）——async **无论绝对值还是占比都略高（+7%）**，方向与上游预期（完成序自由度 1→23 ⇒ 自身非确定变大）**一致**。judge 逐格回读表格后判定 C3（必改项）。
- **根因（机制）**：**用结论意图组织文字，而不是照表内数字方向写**——想表达的其实是「单对读数不足以判定显著」，却写成了「方向相反」。这是「不显著」与「方向相反」两个断言的偷换：前者是关于**统计强度**，后者是关于**方向**；把前者写成后者 = 文字与自己的数据矛盾（数字对、文字反）。
- **定位**：judge 的定位动作就是**回读同一张表的数字与紧邻的文字**（无需任何新测量）——「表格数字与文字方向自洽性」是一步可查的静态检查。
- **修复**：改为「同形态点估计 sync 0.0086% / async 0.0092%——async 略高 **+7%**，方向与 #67 预期一致；但每形态仅 1 对读数 ⇒ 不足以判定显著，故『异步把 run 间非确定放大』在本量级上**既未证实、也未否证**」。
- **教训（判错经验，可复用）**：① **点估计的方向 MUST 照表内数字写**，写完**回读一遍表**核方向（方向错是「数字对但结论反」的高危形态，比数字错更难被发现）；② 「未证实/不显著」**不得**写成「方向相反/未被支持」——单对读数的正确措辞是「同阶 + 不显著（既不证实也不否证）」；③ 一条读数要同时报**方向**与**强度**两个维度，缺一个都会被读成另一个。

## E4: 汇总表/注的数字未逐格回一手文件复核 —— 两处 `blocks` 与一手 diff 矛盾（其中一处全树 0 命中）、终止点数字核不到（judge C1/C2，与 260910-04 C1 **同族二犯**）

- **现象**：verdict 汇总表 9 行中有 **2 行 `blocks` 与一手 diff 文件不符**：③行 `r3-r1 × r3sync-r1` 写 `375,562,240`（一手 `perf-regression-260910-04/cmd-output/diff-r3-vs-r3sync.txt:74` = **`375,545,856`**，且 `375,562,240` **全树 grep 0 命中**）；⑦行 `r3sync-r1 × vanilla` 写 `375,554,048`（一手 `cmd-output/diff-r3sync-vs-vanilla.txt:74` = **`375,549,952`**）。**撞值签名**：③ = ④（同为 375,545,856 域）、⑦ = ⑤。另 §1.2 注写「正对照在 `n≈768/4225` 时主动终止」——该数字在日志中**核不到**且被后续行反驳（一手锚点：末次 Chunky `Processed: 865 chunks (20.47%)` @18:57:08、末条 `[CHUNKTIME] n=1536` @18:57:12、`CALines` 终值 1711）。
- **根因（机制）**：**表格/注释里的数字来自上游文档转抄或记忆重排，而非回一手产物逐格复核**。两类错误同根：① 行间复制粘贴导致**相邻行的分母撞值**（③=④、⑦=⑤）——未复核的典型签名；② 中途快照数字（终止点、`skipped count`）被当作终值写入正文（本案还有一处：`[Mixin] placedFeature skipped` 在正文引 `36864`，一手日志**末行 = 73728**，`36864` 是中途快照）。
- **定位**：judge 的动作 = **逐行把表内三列（`blocks`/`diff`/`%`）回一手 diff 文件 `:74` 逐格比对** + 对可疑值做**全树 grep**（0 命中即无出处）+ 回一手日志核终止锚点。⇒ 判据：**任何汇总数字都应有唯一可 grep 的一手出处**；若某值全树 0 命中，或与相邻行撞值，则**未复核**。
- **修复**：按一手文件改正两处 `blocks`（`375,545,856` / `375,549,952`）；终止点改写为可核锚点区间（`865 chunks (20.47%)` / `n=1536` / `CALines 1711`，并声明 CHUNKTIME `n` 相对 chunk 数有 ~+9% 超计、`[CA]` 行与 chunk 非严格 1:1）；正文不再引用中途快照数字。
- **教训（判错经验，可复用）**：① **汇总表数字 MUST 逐格回一手产物复核**（本案 = diff 文件的 `blocks=` 行）；「表内数字自洽」不构成复核（③=④ 的撞值反而说明未复核）；② **撞值 / 全树 0 命中 = 未复核签名**，见到即全表重核；③ **中途快照 ≠ 终值**——引用任何计数（终止点、行数、count）MUST 取**落盘终值**或显式标注为快照；④ 本错误与 260910-04 的 C1（数字错标）**同族**（judge 原话：同一症状「表内数字未回到一手 diff 文件复核」）⇒ **二犯**，说明「表格数字复核」尚未成为机械动作，须并入应用前自检清单。

## 错误→根因 速查表

| # | 一句话现象 | 根因（机制） | 判据 / 家族 |
|---|---|---|---|
| E1 | 脚本报 `unknown arm r3,r3log,…`（整串被拒） | `pwsh -File` 把 `a,b,c` 当**单个字符串**绑到数组形参 | 多值参数用 `@("a","b")`；签名 = 报错里出现**带分隔符的整串**（#37/#81 编排侧、#20/#53 对偶） |
| E2 | `serverCpu=-1 / cores=-1` **恒成立**（多臂同值） | 脚本 CPU 采样块**两处**，停服后那处**覆盖**了停服前的值（同名变量二次赋值） | **恒值异常先查写入路径（覆盖/未初始化），不要先查解析器/工具版本**；grep 同名变量全部赋值点核顺序（同族 #23 mtime、#37/#81） |
| E3 | 文字写「async 未比 sync 更大」，表内 async 0.0092% > sync 0.0086% | 用结论意图组织文字：把「不显著」偷换成「方向相反」 | 点估计方向 MUST 照表内数字写 + 写完回读表；「未证实」≠「方向相反」（本批 workflow-patterns <N+1>/<N+2>） |
| E4 | 汇总表两处 `blocks` 与一手 diff 矛盾（一处全树 0 命中）、终止点数字核不到 | 表内数字未回一手产物逐格复核（转抄/记忆重排 + 中途快照当终值） | 逐格回一手文件（`:74`）+ 全树 grep；**撞值 / 0 命中 = 未复核签名**；中途快照 ≠ 终值（**与 260910-04 C1 同族二犯**） |
```

---

## F `versions/1.21.6/docs/10-timewise-archive.md` 拟追加的 260910-05 时间线块

**插入位置**：现网该文件共 **64 行**，末块 = `## 260909-06（实际 2026-09-09 21:51 起）…` ⇒ **追加到文件末尾**。
**口径**：条目带状态标注（✅/❌/🔍/已结案）；C7 = candidate（judge PASS-with-conditions，C1-C9 已应用，confirmed 留用户）；nether/end = candidate（judge J3/J4 待审）；一次性读数只作证据不重复铺陈。

```markdown
## 260910-05（实际 2026-09-10 18:4x-19:2x，日期锚 Get-Date 18:45）：C7 收口（跨 chunk 写序竞争假说 = 空集）+ 同配置 run-to-run 基线 + nether/end 异步化推广

> 过程产物 `.investigations/perf-closeout-260910-05/{record.md, scout-vanilla-crosswrite.md, patch-nether-end-async.md}` + `.artifacts/perf-closeout-260910-05/{verdict-c7, verdict-nether-end, judge-verdict-c7}`；数据 `.tmp/perf-reg-260910-05/`（不入库）+ `cmd-output/`（region 5+4 组、diff 12 份、MANIFEST-sha256 251 条）；Java 快照 `java-snapshot/pre|post`；通用模式 → workflow-patterns <N>/<N+1>/<N+2> + #107 家族补充案例 + algorithm-fingerprints <K> + build-tooling <M>/<M+1> + #26 家族补充案例 + 错误台账 `perf-closeout-errors.md` E1-E4（subagent 草稿 → 主会话应用）。
> 上游：260910-04 confirmed（机制 = 单车道同步接管；R3 = 异步化 42s vs 279s）；judge C7（`perf-regression-260910-04/judge-verdict-r3-260910-04.md:117-118`）为本块任务来源。

- ✅ **开工前核验（交接结论廉价独立验证）**：R3 落地直证（`NoiseChunkGeneratorMixin.java:61/119-149`、`ChunkTiming.java:28-39 inflight`）；驱动脚本含 r3/r3sync 臂；**执行体三元组** sha 前 16 位 `abd7d8893d22e030` 与归档日志一致；nether/end 句柄在 1.21.6 可用（`initNether/initEnd enabled=true stageMask=3`）。
- 🔴→✅ **前提核验改变做法（本块最关键的一次转向）**：judge 要求的「跨 chunk 写序会计」其**前提为假**——静态链证明默认 `resolveStageMask() = 0b011` ⇒ `skip_features=true` ⇒ Rust `apply_features` 不执行 ⇒ `pending_cross_writes` **结构不可达**（生产 `:1088`/消费 `:1053` 同在 `:1036-1375`，唯一调用点 `:671`；`neighbor_terrain` 调用点 `:1124/:1242` 同族不可达）⇒ 会计降级为条件项，改成**行为化核验**（避免造不可执行路径）。
- ✅ **行为化核验（正负对照 + 通道证明齐备）**：负对照 `r3log-r2`（mask=3 + `WG_CA_LOG=1`）`[CA]` 行 = **0** 且跑满 4225 chunks（39s）；正对照 `r3feat-r2`（mask=5 + `WG_CA_LOG=1`）`[CA]` 行终态 = **1711**、`[WG-CONF] flags=5 skip_features=false`；双通道直证 = JVM banner `Picked up JAVA_TOOL_OPTIONS: … -Dcoreswap.rust.stages=5`（送达）+ `stageMask=5` 系 `CppWorldgen.getFlags(handle)` 的 **JNI 回读**。⇒ C7-①「跨 chunk 写序 ⇒ 写丢失」竞争假说 = **空集**，无复活出口（judge 独立复核：`beardifiers` 全局清空出口 `clearBeardifier` 在 Java 全树 grep **0 命中**；`terrain_cache`/`est_l2` 纯 memo 无别名写 ⇒ 出货语义下 Rust 侧 = per-chunk 纯函数 + 纯 memo，**比原结论更强**）。
- ⚠️ **正对照臂为主动终止（判据不受影响）**：末次 Chunky `Processed: 865 chunks (20.47%)` @18:57:08、末条 `[CHUNKTIME] n=1536` @18:57:12、`CALines` 终值 1711 ⇒ 完成度 ~20%（口径：CHUNKTIME `n` 有 ~+9% 超计、`[CA]` 行与 chunk 非严格 1:1）。门是 chunk 无关的常量 flags 判断，故终止不削弱判据（负对照臂跑满得零行）。
- ✅ **C7-② 同配置 run-to-run 基线（本轮新采）**：sync `0.0086%`（375,533,568 / 32,169）、async `0.0092%`（375,541,760 / 34,459）；跨形态 `0.0122%`/`0.0130%`（高于同形态基线约 0.004pp ⇒ **未归因**形态相关分量，假说：填充序影响 vanilla 结构/装饰放置）；跨实现 `0.0111-0.0148%`；历史代理基线 `0.0150%`。⇒ 等价门的锚由「跨实现代理基线」升级为「同配置实测基线」，门性质仍为**同量级筛选门**。
- ❌→更正（judge C3）**草稿方向读数写反**：把 async（0.0092%）> sync（0.0086%）写成「async 未比 sync 更大」⇒ 改为「async 略高 **+7%**，方向与 #67 预期一致；每形态 1 对读数 ⇒ 不足以判定显著（既未证实也未否证）」。教训：「不显著」≠「方向相反」（→ 台账 E3）。
- ❌→更正（judge C1/C2）**汇总表两处 `blocks` 与一手 diff 矛盾**（③行 `375,562,240` 全树 0 命中；⑦行与⑤行撞值）+ 终止点数字「768/4225」核不到 ⇒ 改为一手值（`375,545,856` / `375,549,952`）与可核锚点区间。**与 260910-04 C1 同族二犯**（→ 台账 E4）。
- ✅ **C7-③ 幅度范围**：async `42s/41s`（诊断口径另有 39s）vs sync `279s/243s` ⇒ **5.8×-7.2×**（非诊断臂口径 5.8×-6.8×）；相对 vanilla 快 **1.24×-1.33×**；`inflight max` **23（async ×3）vs 1（sync ×2）**；CPU 本轮因脚本缺陷缺失（引 260910-04 归档 543/375 CPU-s）。
- ✅ **§15.4 取代记录（不删改 confirmed 正文）**：supersedes `260910-04/verdict-260910-04.md:106`「代理基线级筛选结论：R3 未引入超出既有 run 级非确定的内容差」——新锚点下该**因果指认**不成立（跨形态 0.0111-0.0130% 高于同配置基线 0.0086-0.0092% 约 0.004pp），替代陈述 = 「差异高于同配置基线但 ≤ 历史代理基线」；**预登记触发器读法已显式声明**（数量级读法 ⇒ 触发器成立 ⇒ 记取代；严格数值读法 ⇒ 不成立但取代理由仍由数值直接支撑）——两种读法结论差 = 「是否显式记取代」（→ workflow-patterns <N+2>）。**R3 性能/机制主结论不需重审**。
- ✅ **P4 代码改动（Java 侧纯调度形态，Rust 零改动）**：nether/end 两分支改为与 overworld R3 同构（`Supplier<Chunk> work` + print+rethrow + `ChunkTiming` 包裹 + 共用 `SYNCFILL` + `supplyAsync(work, WG_FILL_POOL)`）；`CppBridge` 新增 `MIXLOG` 门控 nether/end 每 chunk `[WG-FILL]` 行（R1 同族拉平）；`gradle compileJava` **BUILD SUCCESSFUL**（23s）；`runtime/` 无 VCS ⇒ 以 `java-snapshot/pre|post`（mixin 69 行 + CppBridge 10 行差异，`ChunkTiming`/`build.gradle` UNCHANGED）承载 diff。
- ✅ **P4a 载具首用 sanity（新载体：Chunky 维度 + DIM region）**：`chunky world minecraft:the_nether` 被接受（`Processed: 4225 chunks (100.00%), Total time: 0:00:21`）；`[Mixin] populateNoise(nether) intercepted` = **4761** + `[WG-FILL]` = **4761**（依赖 `-Pmixlog=1`）；region 采集 `run\world\DIM-1\region`（end 为 `DIM1\region`）⇒ 维度名/路径/接管生效三风险排除后才开 A/B。
- ✅ **P4b A/B 矩阵（9 臂）**：nether vanilla 56s / sync 69s（inflight 1）/ async 21s、20s（inflight 23）；end vanilla 5s / sync 18s（1）/ async 6s、6s（23）；`ea-r2` 是唯一采到 CPU 的臂（`cores=8.6`，脚本已修后）。⇒ **形态效应** nether **3.3-3.5×**、end **3.0×**；相对 vanilla nether **反超 2.7-2.8×**、end async 6s ≈ 5s，而 **sync 18s 慢 3.6×** ⇒ 单车道形态在三维都是净亏损，异步化后转为不亏/反超。
- ✅ **P4c 维度行为门**：nether 跨形态 `0.1219%` ≤ 同形态噪声 `0.1406%` ⇒ PASS；end 跨形态 **0 块差**（同形态噪声 0，接管 vs vanilla 仅 **7 块 / 494M**）⇒ PASS（位级）。附带读数：nether 接管基线 `0.1020%` **低于**自身噪声 ⇒ 差异落在噪声带内，**≠「接管已验证」**（1.21.6 nether/end 首次验证）。
- 🔍 **open（登记后续项）**：① nether run 级非确定显著高于 overworld（0.1406% vs 0.0092%，~15×）**成因未查**；② 跨形态 vs 同形态基线的高出分量（~0.004pp）**未归因**（需冻结顺序生成载体，属新测量设计，后置，不阻塞 C7）；③ 未来重启 Rust features 接管（bit1 清零）MUST 先加「迟到写/已生效/终态常驻」三数且**按 chunk/线程分桶**（现成 `CA_PENDING_WRITES` 是全局 atomic、23 线程并发下每 chunk 重置 ⇒ 只作量级：实测区间 0-37,893、中位 1,575）+ 逐项裁定 scout §4 的 A1-A7/C1/C2（含越界高度图哨兵、Rust features + Java structures 混合语义）；④ nether/end 的 J3/J4 judge 未做（`judge-verdict-nether-end-260910-05.md` 尚不存在）。
- ❌ **工具坑（→ 台账 E1/E2）**：`pwsh -File … -Arms a,b,c` 逗号串当单字符串（`unknown arm a,b,c`）；CPU 采样点**重复覆盖** ⇒ `serverCpu=-1` 恒成立，初诊「旧解析版本」被证伪（真根因 = 同名变量两处赋值、后者在 stop 之后），脚本已删重复块。
- ⚠️ **工具契约（→ build-tooling <M+1>）**：`scripts/merge_index.py` 写回根 index 是「解析 + 重建」（四字段白名单 + YAML 丢注释）⇒ **带注释的根索引禁跑该工具**，本块 P5 改手工追加 entry（现网 `.artifacts/index.yaml:1189` 已就地写下警告）。
- 口径声明（§9.7）：**载体** = 1.21.6 + Chunky radius 500 = 4225 chunks/臂；**覆盖面** = region 全域逐块普查（overworld `common=7959`、nether `7542/7543`、end `7567`，非抽样）；**可比性** = 同载具/同 seed（`417950215108767439`）/同 region 中心/同工具修订；分母 = `4096 × 两侧 section 并集`；chunk 集缺口 = 比对集 7959 vs 本臂生成 4225（含非本臂 chunk）；**维度间不可互引噪声基线**（nether 与 overworld 差 ~15×）。
- 状态：C7-①/②/③ + §15.4 处置 = **candidate**（judge PASS-with-conditions，C1-C9 已应用；§5 产物契约 C7 已补 = `index-entry.yaml` + 根索引手工登记）；nether/end = **candidate**（judge J3/J4 待审）；两者 confirmed 均留用户。git 基线 `42d46e7`（继承任务书）。
```

---

## G 未写清单 + 理由（记录价值门判定，按 `knowledge/INDEX.md` §写入规则 + `SUBAGENT-KNOWLEDGE-GUIDE.md` §〇）

**判为「不写」（低价值）——不是「简写」，是不进任何知识库载体**：

1. **本块各臂的一次性 wall / CPU 读数**（`r3-r2 41s`、`r3log-r2 39s`、`r3sync-r2 243s`、nether `nv 56s/ns 69s/na 21/20s`、end `ev 5s/es 18s/ea 6/6s`、`ea-r2 8.6 核`、归档 `543/375 CPU-s`）——属「某次跑出的数值」；只在条目/时间线里作为**判据的证据**各引一次，不单独成条（#18/#103 家族已确立「跨 run 绝对值不可引」）。
2. **9 行对拍表 + 6 行维度对拍表的具体数值快照**——同上；本稿只在 <N+1>/<K>/A4 的证据段各引需要的量级（0.0086%/0.0092%/0.1406%/0）作**锚级读数**，不把两张表搬进知识库。
3. **`pending_writes` 的完整分布**（中位 1,575、p95 12,634、区间 0-37,893）——一次读数；知识点是「**全局 atomic + 每 chunk 重置 + 23 线程并发 ⇒ 只能作量级、不得作 per-chunk 值**」这条口径判据（已收进 <N>），分布数字只作 1 次证据引用。
4. **执行体 sha 哨兵值**（`dllsha=abd7d8893d22e030`）与 `region` 中心坐标 / 4225 chunks 换算——现成复现锚 + 一次性区域定位，重编即失效；**已在 260910-04 的 #10 补充案例确立**「sha 基线必须带记录日构建上下文」，本条不重复。
5. **`[Mixin] placedFeature skipped count`（36864 中途 / 73728 终值）与 `[CA]` 行样本内容**——证据细节，判据不依赖；且中途快照与终值不一致，进条目反而制造新的「数字无出处」风险（→ 台账 E4 的教训）。
6. **`r3sync-r2` 比 `r3sync-r1` 快 13%（279s→243s）这类单臂摆动**——属机器/运行级摆动带的又一读数，#103（±10% 噪声带）/#108（±20% 跨 run 摆动）已在库，不新增条目（仅在 <N+1> 的口径边界中作为「锚是分布、单对只给点估计」的一句理由）。
7. **`diff_arms.py` / `diff_dims_260910-05.ps1` 的脚本实现细节**——工具本身已由 #26（Chunky 区域级载体）覆盖；本块只新增「维度路径 + sanity 三查」，已收进 B3。
8. **nether/end 接管正确性专项未做、`naS` 与 A/B 臂的 `[WG-FILL]` 门控 A/B 对照未做**——**未做的测量清单**（诚实边界），属课题内部待办（留在 verdict §5/时间线 🔍），不是可复用知识。
9. **`index-entry.yaml` 片段 schema 与根索引注释的冲突（judge C7 条件）**——**未写**为独立条目：其根因（写回白名单 + 重建）已被 B2（<M+1>）覆盖，且现网根索引 `:1189` 已就地记录；重复登记只会稀释。
10. **`judge C5` 文中 `0 → 21,416` 与一手日志不符**（一手：`pending_writes` max=37,893、末行=0、`21,416` 仅是 `:1844` 的单行样本）、**`placedFeature skipped count 36864` 为中途快照**（终值 73728）——属「引用数字 ↔ 一手产物」核对结果，**归 §0.5（应用前须知）与台账 E4**，不进知识库（一次性文本勘误，判据形态已由 E4 承载）。

**判为「中价值、只简记」**：<N+2>（预登记读法）、<M>（pwsh 逗号串）、#107 家族补充案例（跨维推广）、#26 家族补充案例（维度载具）、<K>（维度确定性指纹）——均只记「是什么 + 判据」，不铺陈过程。
**判为「高价值、详写」**：<N>（空集证明三件套）、<N+1>（噪声锚）、<M+1>（merge_index 静默销毁）、台账 E1-E4。

---

## H 自检清单（按 `SUBAGENT-KNOWLEDGE-GUIDE.md` §四 逐条打勾）

- [x] **先过价值门**：高/中/低三档已在 §G 逐项声明（含 10 项「不写」及理由）；低价值项未进任何知识库载体。
- [x] **每个错误都有五段式**：台账 E1-E4 均含 **现象 / 根因 / 定位 / 修复 / 教训**，无「只记已修复」条目（E2 明确写出初诊假设被证伪的链条；E3/E4 属表述/数字层错误，同样按五段组织）。
- [x] **根因是机制层面**：E1 = `-File` 实参串语义（分隔符未生效）；E2 = 同名变量两处赋值、后者在 stop 之后**覆盖**；E3 = 把「不显著」偷换成「方向相反」的断言偷换；E4 = 数字未回一手产物逐格复核（转抄 + 中途快照当终值）。
- [x] **定位含诊断方法/工具**：E1 = 报错里名字形态判别；E2 = 恒值不变性 → 排除解析假设 → 同名变量赋值点 grep 核顺序；E3 = 回读同表数字与文字方向；E4 = 逐格回一手 diff `:74` + 全树 grep（0 命中/撞值签名）+ 回一手日志核锚点。
- [x] **判错经验已沉淀**：「恒值异常先查写入路径/赋值覆盖，不要先查解析器版本」（E2）、「点估计方向照数字写 + 回读表；不显著 ≠ 方向相反」（E3）、「撞值 / 全树 0 命中 = 未复核签名；中途快照 ≠ 终值」（E4）、「路径不可达断言的三件套 + 复活出口穷尽」（<N>）、「噪声锚选同配置实测 + 读法预登记写死」（<N+1>/<N+2>）。
- [x] **被排除假说有标注（❌/⚠️）**：C7-① 竞争假说 = **空集**（❌ 排除，含复活出口穷尽）；「serverCpu=-1 是旧解析版本」= **证伪**（真根因 = 覆盖）；「async 未比 sync 更大」= ❌ 方向写反已更正；正对照终止 = ⚠️ 标注（不削弱判据的论证已给出）；nether 接管「差异落在噪声带内」= ⚠️ 明写「≠ 已验证」；本稿未删除任何被排除项。
- [x] **写入载体正确**：错误 → `.investigations/perf-closeout-260910-05/perf-closeout-errors.md`（独立成篇，新文件）；通用可复用判据 → `workflow-patterns.md`（4 条）；工具/构建坑 → `build-tooling.md`（3 条）；维度指纹 → `algorithm-fingerprints.md`（1 条）；过程/被排除假说/工具演进 → `versions/1.21.6/docs/10-timewise-archive.md`（260910-05 块）；索引 → `INDEX.md` 追加段。**未动主题篇（01-09）**（本块无「对 1.20.1 的验证结论」需落主题篇——全部结论属 1.21.6 性能/机制面）。
- [x] **无复用价值的结论未写 docs**：见 §G 十项（一次性读数、sha 锚、分布快照、未做测量清单、脚本实现细节、单次勘误文本）。
- [x] **错误台账末尾速查表已同步**：4 行，一行一错，与正文 E1-E4 一一对应。
- [x] **数字来自一手记录，无编造无占位符**：全部引用数字回一手产物核对（`.tmp/perf-reg-260910-05/{results.txt,logs/}`、`cmd-output/diff-*.txt:74`、`worldgen_handle.rs` / `CppBridge.java` / `merge_index.py` 行号）；两处现网文本 ↔ 一手不符已在 §0.5 显式列出并说明本稿取值；**未核到**项已在文中写明（如 B1 的 PowerShell 参数绑定机制未做最小复现 ⇒ Degraded；`pending_writes` 的 C5 区间 21,416 未复现）。
- [x] **格式与目标文件末尾现状对齐**：已先读三份目标文件末尾条目与字段风格（时间/发现者/置信度/module + 来源定位 + 观察/证据/判据 + 家族索引；详写条 `## 发现 #N:`、简记/家族案例 `### 发现 #N 简记:` / `### #家族 补充案例`）+ INDEX 现有 `> YYMMDD-## 追加：` 单段风格 + 时间线现网块格式（`## 2609xx-xx（实际 …）` + 状态标注条目 + `> 过程产物` 引注 + `口径声明（§9.7）` + `状态：` 行）。
- [x] **只读纪律**：本次仅新建本草稿文件；未修改 `knowledge/**`、`versions/**`、`.artifacts/**`、`scripts/**` 任何现网文件；未改代码；未执行任何写盘命令（只读核对：读文件、`Select-String`、`Get-Content`）。
