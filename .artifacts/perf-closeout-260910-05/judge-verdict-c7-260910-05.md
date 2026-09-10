# judge verdict — 260910-05 · C7 收口（跨 chunk 写序竞争假说 + 同配置 run-to-run 基线）

> 审查角色：core-judge（subagent 隔离执行、**只读**、**无 shell**）。审查节点：**C7 收口结论的 confirmed 前审查**（内容 artifact §5 自标 candidate）。
> 审查对象：`.artifacts/perf-closeout-260910-05/verdict-c7-260910-05.md`(candidate)；过程记录 `.investigations/perf-closeout-260910-05/record.md` + `scout-vanilla-crosswrite.md`；原始证据 `cmd-output/`（5 份 diff + `diff_arms.py` + 4×16 region）与 `.tmp/perf-reg-260910-05/{results.txt,logs/}`；Java 快照 `java-snapshot/{pre,post}/`；上游 `perf-regression-260910-04/*`（含 judge C7 `judge-verdict-r3-260910-04.md:117-118`）。
> **本文件只出审查意见，不改任何被审文件、不改任何 status**；confirmed 只能由人类授予。审查方式：逐行回读一手证据 + Rust/Java 源码独立复核 + 全仓 grep 交叉验证。
> **范围**：C7-①（竞争假说是否空集）/ C7-②（同配置基线）/ C7-③（幅度表述）/ 对 260910-04 等价门措辞的处置建议（verdict §4）。**不含** nether/end 异步化（P4 在途，`NoiseChunkGeneratorMixin.java` 的 nether/end 分支已异步化 = 背景，不作判据）与 R3 主体（已 confirmed）。

## 判定：**PASS-with-conditions**

主干成立且方向可信：**C7-①「空集」结论经我独立静态复核逐环节成立且未发现复活出口**（§1）；**C7-② 同配置 run-to-run 基线的 5 对新增读数逐格可核到**（§3）；C7-③ 幅度范围算术自洽、`inflight max` 定位到臂内原位；状态纪律干净（candidate、无自授 confirmed）；四个 C7 臂的 `results.txt` 新增 `args=`/`stages=`/`calog=` 字段**闭合了 260910-04 judge C5-④ 的「开关无一手记录」缺口**；`-Dcoreswap.rust.stages=5` 的送达有**双通道直证**（JVM banner + JNI read-back）。
但存在 **9 项必改**，其中 3 项是实质性的：**①（C1）§2 表 2 个 `blocks` 数字与一手 diff 文件矛盾（其中 1 个全树无从核到）；②（C3）§2.1 对自身表格的方向读数写反（async 点估计高于 sync，却被写成「async 未比 sync 更大」）；③（C4/C9）§4 的处置建议会导向就地删改 confirmed 正文（框架 §15.4 禁），且 record.md 的「E2 已修」与归档脚本实际不符**。这些都可静态修（无需新测量），故 **PASS-with-conditions**，不判 FAIL。

---

## 一、逐条回答（对应任务书 7 问）

### 1. C7-①「空集」是否成立、前置条件登记是否穷尽 —— **成立；登记不穷尽（C5）**，我的置信度：**高（静态链）／高（行为面不可达）**

**逐环节独立复核（全部一手）**：

| # | 环节 | 我的核对 | 结论 |
|---|---|---|---|
| 1 | 默认掩码 | `java-snapshot/pre/CppBridge.java:69-74`：`prop==null \|\| empty ⇒ 0b011`；`"all" ⇒ 0`；数字直通。三处句柄（`:82/:128/:148`）都 `setFlags(handle, resolveStageMask())` | ✅ 与 verdict §1.1-① 一致 |
| 2 | Rust 门 | `worldgen-core/src/worldgen_handle.rs:669-671`：`flags & FLAG_SKIP_FEATURES != 0 \|\| env WG_SKIP_FEATURES` ⇒ 不进 `apply_features`；`FLAG_SKIP_FEATURES = 1<<1`（`:148-150`） | ✅ |
| 3 | 调用点唯一性 | 全仓 grep（含 `versions/*/rust`）：`apply_features(` 只有**定义 `:1036` + 调用 `:671`**；`fill_chunk_blocks` 的生产调用者唯一 = `api.rs:156`（其余全为 `bin/`、`bin-diag/` 诊断程序） | ✅ 无第二入口 |
| 4 | 缓冲四引用 | `pending_cross_writes` 全树仅 4 处：`:144` 声明 / `:506` 初始化 / `:1053` 消费（在 `apply_features` 内）/ `:1088` 生产（闭包同在 `:1036-1375` 内）；闭包唯一注入点 = `:1283`（`ca_min` 门控），`bin-diag/b5b6_smoke.rs:126` 为显式 `None` | ✅ 无漏出口 |
| 5 | 邻域路径 | `neighbor_terrain`（`:999-1022`）两个调用点 `:1124`/`:1242` 均在 `apply_features` 内 ⇒ mask=3 下连邻域快照预取也不执行（verdict §1.1-4 正确） | ✅ |

**「有没有别的出口 / 别的顺序敏感共享状态让竞争假说复活」——我另做了穷尽性检查，结论：无复活出口**：
- `beardifiers: RwLock<HashMap<(i32,i32),…>>`（`worldgen_handle.rs:116`）**只按 chunk 键写读**（`:536` 写、`:710/:857` 读己键），全局 `clear_beardifier()`（`:539-540`）的唯一 JNI 出口 `Java_wg_CppWorldgen_clearBeardifier`（`versions/1.21.6/rust/src/jni_bridge.rs:265-276`，注释自称「Java 侧可能未声明」）——**我在 `runtime/1.21.6/java` 全树 grep `clearBeardifier` = 0 命中**（`CppWorldgen.java:78` 只声明 `setBeardifier`）⇒ 全局清空**不可达** ⇒ 非顺序敏感内容状态（只余内存面）。
- `terrain_cache`（`:141`、`:631-635` cap→clear）与 `est_l2`（`aquifer.rs:248-288` FIFO）为纯 memo：命中值 = 重算值，且 `fill_chunk_blocks` 命中缓存时 `(*e.col).clone()`（`:624`）后才跑 features ⇒ 缓存条目永不被 feature 写改写（无别名写）。
- mask=3 的 fill 主路径（`:613-678` → `fill_terrain_column:683-837`）内唯一共享可变量就是上述 memo 的 upsert。
- `api.rs` 导出的 `wg_*` 面（`:59-225`）无任何跨写投递/查询通道。

⇒ **在出货语义（默认 mask=3）下 Rust 侧退化为「per-chunk 纯函数 + 纯 memo」，完成序自由度对内容无影响**，故 04 §三源-3.3 的 `pending_cross_writes` 顺序敏感竞争假说**是空集**——且这比 verdict 所述更强（verdict 只证明该路径不可达，未点出「其余共享态皆内容中性」这一半；见 C5）。

**但「前置条件登记」作为未来重启的核对清单不穷尽**（C5）：§1.3 冒号后列了 6 项，而 scout §4 的清单是 A1-A7 + C1/C2 共 9 项，**漏了** ①A3「越界高度图读哨兵」（`feature.rs:196/204`、`placement.rs:419/450/478`）②C2「Java 让位只 redirect `PlacedFeature.generate`，结构件放置段仍是 Java write-through ⇒ 重启后是『Rust features + Java structures』混合语义，验收口径须先声明」。另：登记里的量级依据「mask=5 臂 `pending_writes≈1221/chunk` 级」来自 **`CA_PENDING_WRITES` 这个全局静态计数**（`:1081` 声明、`:1084` 每 chunk `store(0)`、`:1087` 累加），23 线程并发下「本 chunk 读数」被跨线程写污染；实测行跨度 **0 → 21,416**（`r3feat-r2.log.err` 首行 790 … 末行 21416，非「≈1221 级」）⇒ 该数字只能作数量级/上界表述。

### 2. 「不执行」的行为化证据是否足够、两个薄弱点是否致命 —— **足够；两点均不致命，但一处数字错、一处口径须补（C2/N2/N5）**，置信度：**高**

- **负对照**：`r3log-r2`（async、mask=3、`WG_CA_LOG=1`）——`results.txt:2` `CALines=0 calog=True inflightMax=23`；我 grep 该 `.err` 全文 `[CA]\|skip_features\|flags=` **只有 1 行命中**（`r3log-r2.log.err:76` 的 `[WG-CONF] flags=3 skip_features=true …`）⇒ **`[CA]` 行确为 0**；且该臂是**跑满**的（`r3log-r2.log:145` `Processed: 4225 chunks (100.00%), Total time: 0:00:39`）。
- **正对照**：`r3feat-r2`（mask=5、`WG_CA_LOG=1`）——`results.txt:3` `CALines=1711 stages=5`；我 grep `^\[CA\] chunk` **恰为 1711 行**；`[WG-CONF] flags=5 skip_features=false skip_carver=true skip_surface=true`（`.err:76`）。**对照有效**（同构建态/同 seed/同 region/背靠背，差量只有 `-Dcoreswap.rust.stages` 与 `WG_CA_LOG`）。
- **薄弱点①（正对照早期终止）——不致命**：门是 `fill_chunk_blocks` 内每次调用读同一常量 flags（`:669`），**chunk 无关**；1711 行远超「偶然可达」。但 **verdict §1.2 注里的「n≈768/4225 时主动终止」核不到且与日志后续行矛盾**（见 C2：末次 Chunky 采样 865 chunks/20.47%、末条 CHUNKTIME `n=1536`；且 CHUNKTIME 的 `n` 相对 chunk 数有 ~+9% 超计——跑满臂 `n_max=4608` vs 4225 chunks，故 r3feat 实际完成约 1.4k chunks；`CALines=1711` 亦不能当 chunk 数，历史同载体 `calog-e2-260909-04.err:52-53` 有 `chunk(0,0)` 出现两次）。
- **薄弱点②（经 `JAVA_TOOL_OPTIONS` 传 `-D`）——不致命，且证据比 `-P` 通道更强**：① JVM banner 直证送达：`r3feat-r2.log.err:1` 与 `:12` 均为 `Picked up JAVA_TOOL_OPTIONS: -Djava.io.tmpdir=… -Dcoreswap.rust.stages=5`；② 行为直证：日志 `stageMask=5`（`r3feat-r2.log:93/95/96`）打印的是 **`CppWorldgen.getFlags(handle)`**（`pre/CppBridge.java:85`）——即**经 JNI 从 Rust 句柄读回**的值，证明「sysprop → Java 解析 → setFlags → Rust 存值」全链到位；③ `results.txt:3` 另记 `stages=5`、`conf=…flags=5…`。
  说明：`runtime/1.21.6/java/build.gradle:125-126` **确有** `-PrustStages` → `-Dcoreswap.rust.stages` 的内嵌映射通道（任务书所指），本块**未用**它而改走 `JAVA_TOOL_OPTIONS`——**这反而多出一条 banner 级证据**，不构成缺陷；仅建议（N2/N5）见下。
- **残余歧义（建议级）**：负对照臂缺「`WG_CA_LOG` 真送达」的 in-process 自证（`[WG-CONF]` 不打印 `ca_log`），现靠「正对照同驱动同通道送达」推断；若 env 未送达，负对照的 0 行仍是 0 行。→ N2。

### 3. C7-② 数字可核对性（9 行表逐行） —— **7/9 行逐格核到；2 行 `blocks` 矛盾**（C1），置信度：**高**

| 表行（verdict:43-51） | 一手出处 | 判定 |
|---|---|---|
| ①`r3sync-r1 × r3sync-r2` 375,533,568/32,169/0.0086%/91352-331 | `cmd-output/diff-r3sync-r1_vs_r3sync-r2.txt:74` | ✅ 逐格一致 |
| ②`r3-r1 × r3-r2` 375,541,760/34,459/0.0092%/91294-391 | `diff-r3-r1_vs_r3-r2.txt:74` | ✅ |
| ③`r3-r1 × r3sync-r1` 375,**562,240**/45,948/0.0122%/91289-397 | `perf-regression-260910-04/cmd-output/diff-r3-vs-r3sync.txt:74` = `blocks=375545856`；04 verdict `:103` 亦为 375,545,856 | ❌ **`blocks` 矛盾**；且 `375,562,240` **全树 grep 0 命中**（无任何出处） |
| ④`r3-r2 × r3sync-r1` 375,545,856/49,000/0.0130%/91267-419 | `diff-r3-r2_vs_r3sync-r1.txt:74` | ✅ |
| ⑤`r3-r1 × vanilla` 375,554,048/44,921/0.0120%/91176-512 | `diff-r3-vs-vanilla.txt:74`；04 verdict `:102` | ✅ |
| ⑥`r3-r2 × vanilla` 375,549,952/42,867/0.0114%/91155-532 | `diff-r3-r2_vs_vanilla.txt:74` | ✅ |
| ⑦`r3sync-r1 × vanilla` 375,**554,048**/41,809/0.0111%/91154-533 | `diff-r3sync-vs-vanilla.txt:74` = `blocks=375549952`；04 verdict `:104` | ❌ **`blocks` 矛盾**（写成与⑤同值） |
| ⑧`r3sync-r2 × vanilla` 375,554,048/55,761/0.0148%/91206-482 | `diff-r3sync-r2_vs_vanilla.txt:74` | ✅ |
| ⑨历史代理基线 56,214/0.0150%/91191-496 | `perf-regression-260910-04/cmd-output/diff-result-historical-260910-03.txt:74` | ✅（该行 `blocks=375549952`，表内以 `—` 省略，无矛盾） |

- **`r3sync-r2 = 243s` / `r3-r2 = 41s`**：✅ 就地核到——`.tmp/perf-reg-260910-05/results.txt:4`（`sec=243 total=0:04:03 chunks=4225 inflightMax=1`）与 `:1`（`sec=41 chunks=4225 inflightMax=23`）；并与臂内 Chunky 原位行一致（`r3sync-r2.log:163` `Processed: 4225 chunks (100.00%), Total time: 0:04:03`；`r3-r2.log:145` `Total time: 0:00:41`）。`r3log-r2=39s` ✅（`results.txt:2` + `r3log-r2.log:145`）。
- **单变量核对**：`results.txt:1` vs `:4` 的 `args=` 仅差 `-Psyncfill=1`，`conf=`/`dllsha=abd7d8893d22e030`/`chunks=` 逐字相同 ⇒ 设计层面单变量成立；`stages=default`、`calog` 逐臂显式记录（C5-④ 已闭合 ✅）。
- ④行与③行「`blocks` 撞值」（都为 375,545,856）与⑦行与⑤行撞值（375,554,048）是同一症状：**表内数字未回到一手 diff 文件复核**——与本块上游 04 的 C1（数字错标）同族，故按 confirmed 文本不得留错值处理。

### 4. 解释是否过度、§9.7 三要素是否齐备诚实 —— **三要素齐备 ✅；两处解释需收（C3/C6），两处边界未声明（C6）**，置信度：**中高**

- **§2.1「async 自身 run 间非确定未比 sync 更大」——写反了（C3）**：本表自身给的是 sync 0.0086%（32,169）/ async 0.0092%（34,459），**async 无论绝对值还是占比都略高（+7%）**，方向与 judge C2 的预期（#67 完成序自由 1→23 ⇒ 预期变大）**一致**，不是「未被实测支持」。正确表述 = 「点估计方向与预期一致（+7%），但每形态仅 1 对读数 ⇒ 不足以判定显著，故预期『在数量级上被放大』未被证实」。「跨形态 0.0122/0.0130% 高于同形态基线约 0.004pp」（实为 +0.0030~0.0044pp，取整可接受）**方向有 2×2 读数支持**，但成因括号句属**假说**（→N4）。
- **§9.7 三要素齐备 ✅**：载体（1.21.6 + Chunky radius 500 = 4225 chunks）/覆盖面（region 全域逐块）/可比性（同载具同 seed 同 region 中心 -48,-11）都在场，且 `dllsha`、`args=`、`stages=` 使「同载具」可核。
- **应声明而未声明（C6）**：① **分母语义**——`diff_arms.py:140-162` 的 `blocks = 4096 × (两侧 section 并集)`（`if vb is None and cb is None: continue`，全 air section 不计），≈11.5 section/chunk ≈ 理论 24 section 栅格的 48%，且 `blocks` 随对变化（375,533,568–375,554,048）；② **chunk 集缺口**——比对集 `common=7959` chunks（`diff-*.txt:1-2`）而臂内 Chunky `Processed: 4225 chunks`，**7959 与 4225 的差 3734 无出处**（各臂跑前 `Remove-Item run\world`，`run_arms3.ps1:52`；region 采集为整目录拷贝 `:119-127`）⇒「覆盖面 = 逐块普查 7959 chunks」缺一行来源说明。
- **已声明的诚实边界**（列齐）：每形态各 1 对读数 ✅；CPU/核数缺失 + E2 ✅；`sections same/diff` 与跨实现对同量级 ✅；ABBA 置信区间未做 ✅。
- **未声明**：异步/同步两形态的**幅度读数也只有 2 个 wall 点/形态**，且 sync 两 run 相差 13%（279 vs 243s）远大于 async（42 vs 41s，2.4%）——这条不对称对「范围」表述有影响（→N3）。

### 5. 措辞合规 —— **合规 ✅（无越界）**，置信度：**高**

- 全文无「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响」**断言式**用法；§2-门性质行是**引用禁令**（"不得写…"）✅；§3 只报 `inflight max=23/1` 实测值，未写 1.000 ✅。
- 等价门全程写「**同量级筛选门**」，未升级为定论 ✅（且明确「本轮把锚升级为同配置实测基线」）。
- 唯一措辞瑕疵是 §4 的处置动作本身（→C4）与 §1.2 的终止点数字（→C2），不属禁令词范畴。

### 6. 状态纪律 —— **合规 ✅；但「未实现」的理由表述需精确化（C8）**，置信度：**高**

- 无自授 confirmed：`verdict:3/80` = candidate、confirmed 留用户 ✅；judge 未改任何 status ✅。
- 会计代码未实现：**理由方向成立**（C7-① 空集 ⇒ 该计数在出货配置内无观测对象；已转「前置条件登记」），但「**属不可执行路径**」措辞不精确：该路径在 `mask=1/5` 下**可达**（本块 `r3feat-r2` 即已证明），精确说法 = 「在出货语义（mask=3）下不可达 ⇒ 不可在出货配置内测量；改动 mask 即偏离被测配置」。并应在登记里写明：**新增会计计数不得复用全局 `CA_*` 静态计数**（并发污染，见 §1 第 1 问）。

### 7. 对 260910-04 等价门措辞的处置建议（§4）是否恰当 —— **方向对，动作与判据两处必须修正（C4）**，置信度：**高**

- **恰当的**：保留「代理基线级筛选结论」框架、把锚升级为同配置实测基线、删掉「偏乐观句」的**意图**、以及「无需重审 R3 性能/机制主结论」——都对（C7-① 已空集、性能结论未被推翻）。
- **必须修正 ①「删除/改写」动作**：被指向的句子是 **260910-04 已 confirmed 正文** `verdict-260910-04.md:106`（"⇒ 代理基线级筛选结论：R3 未引入超出既有 run 级非确定的内容差"）。按 AGENTS.md §一.8 / Anchorlaw §15.4（**supersedes 双指针，原结论正文不删不改**），**不得就地删改**；应记录取代/补充条目（指向 `perf-regression-260910-04:verdict §9:106`，一行推翻理由 + 新数字）。
- **必须修正 ②「无需 §15.4 取代」的判据**：04 judge C7-② 的**预登记触发器**是「基线与三对**同量级**或更大 ⇒ 按 §15.4 取代重审」。实测基线 0.0086/0.0092% 与三对 0.0111–0.0130% **在同一数量级**（比 1.35–1.5×），若按「同数量级」读法**触发器成立**；若按项目既有「±20% 跨 run 摆动带」读法（04 judge-verdict-r3 曾用）：基线比三对低 21–34% ⇒ **不成立**。verdict §4 用了「② 未见异常放大」这一**未预登记的判据**替换了预登记判据而未声明 ⇒ 必须显式写出所采用的口径与数值；并且——**无论触发器如何读**，新基线（0.0086/0.0092%）严格低于跨形态读数（0.0111–0.0130%），即「R3 未引入超出既有 run 级非确定的内容差」这句在**新锚点下已被数据部分推翻**（存在小幅形态相关分量），故该句需要取代记录，而不是「偏乐观措辞」级的下调。
- **不需要 fan-out、不需要新课题才能收口**：当前无 ≥2 互斥候选待并行验证（空集已闭合；成因归因需要**新的测量设计**——冻结顺序载体，scout 待验 6——属新课题，后置即可，不阻塞 C7）。这一点 verdict §4 的判断我同意。

---

## 二、三源核对（spec §4）

| 源 | 可核性 | 结论 |
|---|---|---|
| ① 交付快照 `.artifacts/perf-closeout-260910-05/` | 可核 | 仅 1 文件（verdict）；**无 `index-entry.yaml`、根 `.artifacts/index.yaml` 无 260910-05 条目**（grep 只有 04 的 `:1162-1184`）⇒ 契约缺口（C7） |
| ② git HEAD + 工作区 diff | **不可自核（无 shell）** | 继承任务书 `HEAD=42d46e7`；替代载体 = `java-snapshot/{pre,post}/SHA256SUMS.txt`：`build.gradle`(09c8c8a1…) 与 `ChunkTiming.java`(e60e757f…) pre/post **同 sha**，`CppBridge.java`/`NoiseChunkGeneratorMixin.java` 不同 ⇒ 在途改动只落在 mixin + CppBridge 的 nether/end 异步化与 `[WG-FILL]` 日志门控（`patch-nether-end-async.md:7-64`；`post/CppBridge.java:31-32` MIXLOG 注释）——**`resolveStageMask()` 逻辑未变**（pre `:69-74` → post `:75-80`），故 C7 静态链不受在途改动影响（行号漂移见 N1） |
| ③ 验证记录 | 可核，含 3 处矛盾 | 5 份 diff（③⑦行 `blocks` 矛盾，C1）；4 臂 `results.txt`+日志（§1.2/§3 数字全核到，除「768」，C2）；`run_arms3.ps1` 显示 CPU 采样点缺陷**仍在**（C9） |

**构建态一致性旁证**：4 臂 `dllsha=abd7d8893d22e030` 与 260910-04 一致；`patch-nether-end-async.md:68` 明记「Rust 侧零改动」⇒ C7 臂与 04 同执行体（Rust），Java 侧 = pre 快照（P4 冻结纪律）。

---

## 三、必改条件（C1..C9，逐条可执行；均为静态修正）

- **C1（MUST）§2 表两处 `blocks` 修正。** ③行 `375,562,240` → **`375,545,856`**（`perf-regression-260910-04/cmd-output/diff-r3-vs-r3sync.txt:74`；04 verdict `:103`）；⑦行 `375,554,048` → **`375,549,952`**（`diff-r3sync-vs-vanilla.txt:74`；04 verdict `:104`）。修正后全表 9 行的 `diff/%/sections` 已逐格可核，`blocks` 亦全可核。
- **C2（MUST）§1.2 注「n≈768/4225 时主动终止」改写为可核锚点。** 现数字在日志中**核不到**，且被后续行反驳。建议改为：「终止锚点：末次 Chunky 采样 `r3feat-r2.log:128` = `865 chunks (20.47%)`（无 `Task finished` 行）；末条 CHUNKTIME `:135/:137` = `n=1536`；kill 后 `CALines=1711` ⇒ 终止时完成约 **1.4k–1.7k / 4225 chunks（33–40%）**」并加一句口径注：「CHUNKTIME `n` 相对 chunk 数有 ~+9% 超计（跑满臂 `n_max=4608` vs 4225 chunks，`r3-r2.log:144`/`r3log-r2.log:144`），`[CA]` 行亦非严格 1:1（`calog-e2-260909-04.err:52-53` 同 chunk 两行）⇒ 终止点以 Chunky/日志锚点区间表述」。**终止理由（门 chunk 无关）可保留**——该判据成立。
- **C3（MUST）§2.1 方向纠正。** 把「async 未比 sync 更大 / 未被实测支持」改为：「同形态点估计 sync 0.0086%（32,169）/ async 0.0092%（34,459）——**async 略高 +7%，方向与 #67 预期一致**；但每形态 1 对读数 ⇒ 不足以判定显著，故『异步把 run 间非确定放大』在本量级上**未被证实，也未被否证**」。
- **C4（MUST）§4 处置动作与判据修正。** ①**不得就地删改** 04 confirmed 正文（`verdict-260910-04.md:106`）——改为记录取代/补充条目（supersedes 双指针 + 一行理由：新测同配置基线 0.0086/0.0092% 低于跨形态 0.0111–0.0130%，故「未引入超出既有 run 级非确定的内容差」在新锚点下不成立，余下框架保留）；②显式写出判定「C7-② 预登记触发器（基线与三对同量级或更大）未触发」所采用的**口径与数值**（数量级读法 / 项目 ±20% 摆动带读法），不得以「未见异常放大」这一未预登记判据替代而不声明。
- **C5（MUST）§1.3 前置条件登记补全。** ①补 scout §4 漏列项或改为指针式：「完整清单 = scout-vanilla-crosswrite.md §4 A1-A7 + C1/C2（本处摘要，未列 A3 越界高度图哨兵、C2 Rust-features+Java-structures 混合语义）」；②量级句「`pending_writes≈1221/chunk` 级」改为数量级/上界表述并注明计数污染：`CA_PENDING_WRITES` 为**全局静态**计数、每 chunk `store(0)` 后由 23 线程共同 `fetch_add` ⇒ 单 chunk 读数不可解释为 per-chunk；`r3feat-r2.log.err` 实测区间 **0–21,416**。
- **C6（MUST）§2 补两条口径边界。** ①**分母语义**：`blocks = 4096 × 两侧 section 并集`（全 air/缺失 section 不计入 ⇒ ≈11.5/24 section，≈理论栅格 48%；`blocks` 随对变化 375,533,568–375,554,048）；②**chunk 集缺口**：比对集 `common=7959` chunks vs 臂内 `Processed 4225`，说明 7959 的来源（或标注为未核边界）。
- **C7（MUST）产物契约补齐。** 新增 `.artifacts/perf-closeout-260910-05/index-entry.yaml` 并在根 `.artifacts/index.yaml` 登记（id/kind/status=candidate；judge 条目 draft）——**手工追加**，勿对该带注释根索引跑 `scripts/merge_index.py`（`record.md:28` 工具坑 E1：会丢注释）。confirmed 前完成。
- **C8（MUST）「未实现」措辞精确化 + 会计规格补一条。** ①「属不可执行路径」→「在出货语义（mask=3）下该路径不可达 ⇒ 不可在出货配置内测量；`mask=1/5` 下可达（本块 `r3feat-r2` 已证）但属另一配置」；②前置条件里写明：新增「迟到写/已生效/终态常驻」三数**不得复用** `CA_*` 全局静态计数（并发污染），须用 thread-local 或 chunk 作用域计数。
- **C9（MUST）record.md 事实更正（错误台账为本项目最高优先级资产）。** ①`record.md:47`「E2…**已修**（`run_arms3.ps1`）」与归档脚本实际**矛盾**：`run_arms3.ps1:90-91` 在停服前正确采样后，`:112-113` 又**重复采样并覆盖** `$cpuSec`（此时 java 已 kill ⇒ 恒为 -1）。⇒ `serverCpu=-1` 的真根因 = **采样点重复覆盖**，非「本批仍走旧解析版本」；须改记并把「nether/end 批 CPU 可采」的预期一并作废，或先删 `:112-113` 再声明可采。②`record.md:54`「`[CA]` 行 **987+**」→ **1711**；③`record.md:44-45` 两行「（运行中）」回填实际读数（正对照 1711、sync 243s/inflight 1）。

## 四、建议项（N1..N6，不阻塞）

- **N1**：代码引用锚定到 **pre 快照**（P4 在途改动使 runtime 行号漂移 ~+6：`resolveStageMask` pre `:69-74` / post `:75-80`）；另 `CA_PENDING_WRITES` 引用宜写 `:1081`（声明）+`:1087`（累加），非 verdict §1.3 的 `:1080-1081`（`:1080` 是 `CA_OUT_READS`）。
- **N2**：负对照的 env 送达自证——建议 `[WG-CONF]` 行增打 `ca_log=`（一行、一次性），即可把「env 未送达也得 0 行」的残余歧义彻底消除。
- **N3**：幅度范围口径——`r3log-r2=39s` 是**诊断口径**（per-chunk 日志开）却参与了 6 组组合的下界；建议范围只用非诊断臂（41–42s）或显式标注「诊断口径参与下界」。同时可注明 sync 形态 wall 的 run 间差（279→243，13%）远大于 async（42→41，2.4%）。
- **N4**：跨形态分量的成因（填充序影响 vanilla 结构/装饰放置）标为**假说**并登记为后置课题（归因需冻结序载体，`scout:238-239` 待验 6）；不建议现在开 fan-out。
- **N5**：注明 `r3feat`（mask=5）同时清 bit2（`skip_surface=true`），与未来真接管臂（`stages=1`）**非同一配置**——可达性证明不受影响，但避免被读作接管基线。
- **N6**：本块归档补 MANIFEST（5 份 `diff-*.txt` + 4 臂 `results/logs` + `run_arms3.ps1`/`diff_pairs_260910-05.ps1` 的 sha 清单），避免重演 04 的 C5（同名归档物 ≠ 产出证据的修订）。

## 五、对 confirmed 授予的明确意见

**条件支持**（C1–C9 全部应用后再授予；C1/C3/C4/C9 为实质项，其余为口径/契约补齐）。范围：

1. **C7-① 「默认出货语义（mask=3）下跨 chunk 写序竞争假说 = 空集」→ 支持 confirmed**。静态链我已逐环节独立复核（含「`beardifiers` 全局清空不可达」这一新增旁证），未发现复活出口；行为面负/正对照有效性成立（两薄弱点均不致命）。
2. **C7-② 「同配置 run-to-run 基线实测 = 0.0086%（sync）/0.0092%（async）」→ 支持 confirmed**（5 对读数逐格可核；与 04 既有三对同尺）。**但派生解释**「async 未比 sync 更大」**不得**随结论一起 confirmed（C3）。
3. **C7-③ 「幅度范围 5.8×–7.2×、R3 相对 vanilla 1.24×–1.33×」→ 支持**，附口径声明（C 无关的 N3：39s 为诊断口径）。
4. **对 04 等价门的处置 → 不支持 verdict §4 的「删除/改写」与「无需 §15.4」两项表述**：该句须以取代记录处理，并明确预登记触发器的读法（C4）。**04 的 R3 性能/机制主结论不需要重审**，我同意这一点。
5. **明确不同意的措辞**（延续 04 judge 立场）：不得把「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响」写入任何 confirmed 文本——本块证据不支持，且新增数据反而**显示存在小幅形态相关分量**。
6. **本块范围外**：nether/end 异步化（P4，在途）与本判定无关，不得借本 verdict 一并 confirmed。

## 六、未核项与诚实边界

1. **无 shell**：未跑任何命令（未 `git status/diff`、未算 sha256、未跑 diff 工具）；`HEAD=42d46e7`、`runtime/`/`.tmp/`/`target/` 在 `.gitignore` 内均为**继承任务书**，非我自核。第②源以 `java-snapshot` pre/post SHA256SUMS + 源码直读替代（sha 值本身我**未复算**，只看「哪些文件变了」）。
2. **未核**：`region-*/**.mca` 内容（只核文件数 16/16/16/16 与 diff 输出自述）；`diff_arms.py` 的算法正确性（只核其分母口径为「两侧 section 并集」这一读法，未逐位验证解码/BitPack 正确性）；`7959 vs 4225` chunk 集缺口的成因（无命令可数）。
3. **未核**：`r3feat-r2` 的精确终止 chunk 数（日志锚点给出 865 / n=1536 / 1711 三值且口径不同，我以区间表述）；`[CA]` 行与 chunk 的严格 1:1 关系（历史同载体见到同 chunk 两行）。
4. **未核**：`serverCpu` 采集口径（本批全 -1；04 的 543/375 只从 `perf-regression-260910-04/cmd-output/results.txt:10-11` 继承，未独立复算）。
5. **未核**：`calog-e2-260909-04` 之外的历史对照臂；`knowledge/discovered` 的 `#66/#67` 只核到引文存在（`workflow-patterns.md:1211/:1223`），未重审其结论。
6. 我**未修改任何被审文件**（verdict、record、scout、diff 输出、results/logs、java-snapshot、根 index.yaml、04 目录均未动）；**本文件是本次审查的唯一写入**。
