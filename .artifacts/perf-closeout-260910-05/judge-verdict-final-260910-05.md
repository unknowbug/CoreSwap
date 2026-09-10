# judge verdict（final / 收尾交付审查）— 区块 260910-05：C7 收口 + nether/end 异步化推广

> 审查角色：core-judge（subagent 隔离执行、**只读**、**无 shell**）。审查节点：**收尾交付审查（§4.5 强制触发点：收尾交付前，三源核对）**。
> 审查层：**只审「收尾交付」这一层**——① 两轮 judge 条件是否真被应用 ② 知识库应用是否忠实于草稿 ③ 状态纪律 ④ 交接一致性（三源核对精神）⑤ `knowledge/INDEX.md` 格式 ⑥ 是否同意提交。**不重审** C7-①②③ 与 P4 的实质结论（除检查条件落地）。
> 审查对象（全部只读）：`.artifacts/perf-closeout-260910-05/{judge-verdict-c7-260910-05.md, judge-verdict-nether-end-260910-05.md, verdict-c7-260910-05.md, verdict-nether-end-260910-05.md, index-entry.yaml}`；`.investigations/perf-closeout-260910-05/{record.md, perf-closeout-errors.md, knowledge-draft-260910-05.md, patch-nether-end-async.md, cmd-output/**, java-snapshot/**}`；`knowledge/discovered/{workflow-patterns.md, build-tooling.md, algorithm-fingerprints.md}`；`knowledge/INDEX.md`；`versions/1.21.6/docs/10-timewise-archive.md`；`NEXT_SESSION.md`；`.artifacts/index.yaml`；`runtime/1.21.6/java/**`（工作树应用版，只读）。
> 方法：逐行回读一手文件 + 全树 grep 交叉核对 + 与一手 `cmd-output/*.txt` 数字逐格比对。**未跑任何命令，未复算任何 sha**（无 shell）。
> **本文件只出审查意见：不改任何被审文件、不改任何 status（含 `index.yaml`）；confirmed 只能由人类授予。本文件是本次审查的唯一写入。**

## 判定：**PASS-with-conditions**（必改 **9** 条：FIN-C1..FIN-C9；全部静态可修，无一项需要新测量）

主干成立：两轮 judge 的**实质项全部真被应用**——C7-C1 两处 `blocks` 已按一手 diff 改正（我逐格复核：`375,545,856` / `375,549,952`）、C7-C3 方向句已纠正、C7-C4 以 §15.4 取代记录表达且 **04 原文 `:106` 逐字未动**、C7-C6 两条口径边界在位、nether-C2/C4/C6/C9/C10 均在位；知识库编号连续无冲突（#110/#111/#112、#49/#50、#21）、**零占位符残留**、草稿 §G 的 10 项「不写」未被违规写入；状态纪律干净（两份 verdict 均 candidate，index-entry/index 均 candidate|draft，无自授 confirmed，「待用户 confirmed」在 NEXT 与 index 一致标注）；`INDEX.md` 脚手架已清除、围栏配平（全文 0 个 ```）、04 段已展开为正式段落、05 追加段位于文件末尾。
但收尾层存在 **1 项自相矛盾（FIN-C1，实质）**、**1 项新引入的数字错（FIN-C3，同 C1/E4 家族）**、**1 项一手证据归档不完整（FIN-C4）**，以及若干**条件应用不完整/新证据未回填**（FIN-C2/C5/C6/C7/C8/C9）。因全部为静态修正，判 **PASS-with-conditions**，不判 FAIL。
**提交意见：不同意在 FIN-C1..FIN-C9 应用前 commit**；应用后（且 commit message 不含任何 confirmed 语义）**同意提交**——本块为 candidate 层产物，提交不等于 confirmed。

---

## 一、两轮 judge 条件逐条点名（真被应用 / 不完整 / 核不到）

### 1.1 C7 judge（`judge-verdict-c7-260910-05.md`）C1-C9

| 条件 | 应用处（被审文件实测） | 判定 |
|---|---|---|
| **C1** 表两处 `blocks` | `verdict-c7:53` = `375,545,856`（③）、`:57` = `375,549,952`（⑦），`:61` 有复核说明 | ✅ **真应用**（一手：本块 `diff-r3-r2_vs_r3sync-r1.txt:74`/`diff-r3sync-r2_vs_vanilla.txt:74` 与 04 `verdict-260910-04.md:103/104` 一致；本块 5 份 diff 的 `blocks` 行我逐格核过，9 行表其余 7 行全部吻合） |
| **C2** 正对照终止点可核锚点 | `verdict-c7:27`（865 chunks/20.47% @18:57:08、CHUNKTIME n=1536、1711、CHUNKTIME +9% 超计口径） | ✅ 真应用 |
| **C3** 方向纠正 | `verdict-c7:65`（async 略高 +7%，方向与 #67 一致；既不证实也不否证） | ✅ 真应用；全树 grep「async 未比 sync 更大」仅剩两处 judge 原文与 `:65` 的历史说明，**无残留断言** |
| **C4** §15.4 取代 + 触发器读法声明 | `verdict-c7:89-97`（supersedes/superseded-by 双指针 + 一行理由 + 数量级/严格数值两读法 + 「不需新课题」） | ✅ 真应用；**04 原文未动**——`verdict-260910-04.md:106` 逐字仍为原句，04 侧无取代指针（按 C4 只要求新件记取代，可接受；NEXT:43 已把「04 侧补一行指针」列为 confirmed 后动作） |
| **C5** 登记补全（A3 越界高度图哨兵 / C2 混合语义）+ 量级口径 | `verdict-c7:35`（全局 atomic 23 线程轮转 ⇒ 只作量级/上界；中位 1,575、区间 0-37,893）、`:37`（scout §4 A1-A7/C1-C2 指针 + 哨兵 + 「Rust features + Java structures」） | ✅ 真应用 |
| **C6** 分母语义 + chunk 集缺口 | `verdict-c7:70-71`（`blocks = 4096 × 两侧 section 并集`；`common=7959` vs 臂内 4225、成因未独立复核） | ✅ 真应用（一手 `diff-*.txt:1-2` = 7959 全数核对） |
| **C7** 产物索引契约 | `index-entry.yaml`（4 条）+ 根 `index.yaml:1188-1229`（手工追加 + `:1189` 就地「勿跑 merge_index.py」警告） | ✅ 真应用 |
| **C8** 「不可达」精确化 + 会计不得复用全局计数 | `verdict-c7:39-42` + `:36` | ✅ 真应用 |
| **C9** record 事实更正 | `record.md:47`（E2 根因按**未核**声明、不再用「脚本已修 ⇒ 已解决」）、`:54`（987+ → **1711**）、`:42-45`（结果回填：1711 / 243s / inflight 1） | ⚠️ **不完整**：record.md 本体已改，但**错误台账 E2 与 `verdict-c7:87` 仍把「重复采样覆盖」写成已证实的真根因 + 「脚本已修」**（见 FIN-C1）；`record.md:58` 仍留 `pending_writes≈1221/chunk`（见 FIN-C8） |

### 1.2 nether/end judge（`judge-verdict-nether-end-260910-05.md`）C1-C10

| 条件 | 应用处（被审文件实测） | 判定 |
|---|---|---|
| **C1** nether「自身噪声带」写准（async 代理） | `verdict-nether-end:41`（§3.1 ⚠️ 口径：无 sync run-to-run 对、0.1406% 出自 async 同形态对、代理合理性引 overworld） | ⚠️ **不完整（1/2 处）**：C1 点名的 **§5.2 未改**——`:84` 仍写「nether 0.1020% 落在**自身**噪声带内」而无代理定语（见 FIN-C6）；§3.1 附带读数 `:43` 亦仍用「nether 自身（async 代理）噪声」措辞（有代理但「自身」未删） |
| **C2** end 7 块 vs 0 差 + 限定 + 删绝对化 | `:49`（7 块明记）、`:59`（限定：仅 coreswap 臂内 + 该 region 集；不得外推「该维不存在 run 级非确定」；不得与 0 并列；`ev` 仅 1 run ⇒ 无法归因）；全树 grep「该维不存在 run 级非确定」仅剩 judge 原文 | ⚠️ **基本真应用，C2④ 落字不完整**：§6 `:93` 未按字面内联「（common=7567 / section 并集 494,100,480）」，只以「限定在 §3/§5 声明的范围」指针代替（见 FIN-C7） |
| **C3** end sanity 缺口闭合 | `:25`（`eaS-r1` 实跑：`initEnd enabled=true stageMask=3`、`Total time 0:00:07`、`populateNoise(end) intercepted` 4761 + `[WG-FILL]` 4761、region `DIM1\region` 16 mca） | ⚠️ **证据真实但归档不完整**：`.tmp/perf-reg-260910-05/logs/eaS-r1.log` 我逐行核到（`:96` initEnd、`:14425` Task finished 0:00:07、intercepted/WG-FILL 各 4761、`:94` dllsha=abd7d8893d22e030），`cmd-output/region-eaS-r1/` 16 个 mca 在位；**但** `cmd-output/MANIFEST-sha256.txt`（251 条）**不含 region-eaS-r1 与 logs/eaS-r1.\***，`cmd-output/results-260910-05.txt`（13 行）**无 eaS 行**（`.tmp` 的 results.txt 有 14 行）⇒ C3 闭合的一手日志载体只有 `.tmp`（不入库）（见 FIN-C4） |
| **C4** 比值口径点明 + wallgen 并列 + 禁「端到端」 | `:74-77`（3.3-3.5×/3.0× = Chunky 进程内计时；wallgen 2.34×/2.00×；`wallgen−sec` 1.1-10s 不稳定；**禁用「端到端」**）；`:118` 判据已按口径限定 | ✅ 真应用 |
| **C5** CPU 解读 + 根因按未核 | `:79`（86 CPU·s/6s ⇒ 更多总 CPU 换更低延迟；延迟结论非效率结论）+ `:85` §5.3（根因 = **未核**；两轮 judge 各见一版脚本、归档副本均为修后版 ⇒ 文件级不可复核） | ⚠️ **不完整**：`verdict-c7:87` 与错误台账 E2 与之**直接矛盾**（仍写「真根因 = 重复覆盖…脚本已修」），`run_arms3.ps1:89/112` 注释亦仍以既成语气写「E2 真根因 = 重复采样覆盖」（脚本事实面：现版**无**停服后采样块，`:90-91` 停服前采、`:112` 是「不得重复采样」注释、`:113` 算 cores ⇒ 与 nether-C5② 一致，与 C7-C9① 的 `:112-113` 描述不一致）⇒ 两条件冲突须消解（见 FIN-C1）；另 `唯一臂采到 CPU` 表述已随 eaS 补跑失效（见 FIN-C2） |
| **C6** dllsha 仅 7 个 coreswap 臂 + 未复算声明 | `:16`（nv-r1/ev-r1 为 vanilla ⇒ `dllsha=-`；「与 target dll 一致」本块未复算按继承声明） | ✅ 真应用（一手 `results-260910-05.txt:6/10` = `dllsha=-` 核对通过） |
| **C7** 单对点估计内联 | `:42`（§3.1 ⚠️ 每形态 1 对/无置信区间/检验力低）+ `:94` | ✅ 真应用 |
| **C8** 门控范围 + 4761 语义 | `:12`（只门控 println）、`:27`（4761 = 调用次数，> 4225 ⇒ 重复接管调用） | ✅ 真应用（一手：`eaS/naS` 日志三处行计数各 4761，Chunky `Processed: 4225`） |
| **C9** end「0 差」可核论证 | `:54-58`（灵敏度 7 块 / common=7567 无单侧排除 / 分母 494,100,480 非空 / 三 region sha 互异） | ✅ 真应用（一手 `diff-E_sync_vs_vanilla.txt:74-77`、`MANIFEST` 三方 end region 同名 mca sha 互异核对通过） |
| **C10** 根索引摘要同批修正 | `index.yaml:1214-1222`（7 coreswap 臂 + 口径点明 + async 代理 + 7 块不得并列 + 确定性限定 + CPU 根因未核 + 禁「端到端」） | ✅ 真应用（`:1217` = 「**禁用「端到端」措辞**（未测）」，无未加口径的 3.3-3.5×/2.7× 残留） |
| N5（建议）patch 状态回填 | `patch-nether-end-async.md:3`「状态：**已应用**」 | ✅（建议项已顺手落实） |

### 1.3 两轮条件的**冲突**（C7-C9① ↔ nether-C5②）——收尾层必须显式消解

- C7 judge **C9①** 按当时读到的一版脚本判定「E2 真根因 = 停服后重复采样覆盖（`:112-113`）」，要求 record 改记为「重复覆盖，非旧解析版本」。
- nether judge **C5②** 逐行读现版脚本（两份副本）后指出**不存在重复采样块**，要求「CPU 缺失的实际根因 = 未核，**禁止**写成『脚本已修 ⇒ 已解决』」。
- 我复核被审的一手文件：`cmd-output/run_arms3.ps1:89`（注释「E2：采样点错误」）、`:90-91`（停服前采样）、`:112`（注释「此处**不得**重复采样（会覆盖成 -1，E2 真根因）」）、`:113`（算 cores）——**现版确无停服后采样块**；`.tmp` 与 `cmd-output` 两份副本同源，`MANIFEST` 亦只覆盖该修后版本 ⇒ **预修版本文件级不可复核**，nether-C5② 成立。
- 但主会话的落地是**半边**：`record.md:47` 与 `verdict-nether-end:85` 采新条件（未核），而**错误台账 E2（本项目最高价值载体）+ `verdict-c7:87` + 脚本注释**仍按旧条件写既成事实 ⇒ 同一事实三处互斥。→ **FIN-C1**（以 nether-C5② 为准，并把冲突与取舍理由写进 E2，防下一轮再翻）。

---

## 二、知识库应用是否忠实于草稿（`knowledge-draft-260910-05.md` → 现网）

1. **编号**：草稿 §0.1 建议 `<N>=#110 / <N+1>=#111 / <N+2>=#112`、`<M>=#49 / <M+1>=#50`、`<K>=#21`；现网实测 `workflow-patterns.md:1820/1840/1853` = #110/#111/#112、`build-tooling.md:928/939` = #49/#50、`algorithm-fingerprints.md:456` = #21。**连续、无冲突**（各文件既有最大号分别为 #109 / #48 / #20，全树无重号）。✅
2. **占位符残留**：`grep '<N>|<N+1>|<N+2>|<M>|<M+1>|<K>'` 于整棵 `knowledge/` = **0 命中**；家族案例（`#107 家族补充案例（260910-05）`、`#26 家族补充案例（260910-05）`）按草稿「不占新号」惯例落地。✅
3. **草稿 §G「判为不写」是否被违规写入**：抽查全部 10 项——一次性墙钟/CPU 读数未成条（仅作判据的证据各引一次）、9 行与 6 行对拍表未搬入、`pending_writes` 完整分布未搬入（仅「区间 0-37,893 / 中位 1,575」一次引用）、`dllsha` 哨兵/4225 换算未成条、`placedFeature skipped 36864/73728` 与 `[CA]` 样例行未入（`grep 36864|73728` 于 knowledge = 0 命中）、单臂摆动未成条、脚本实现细节未成条、未做测量清单未成条、`index-entry.yaml` schema 冲突未另立条、judge C5 的「0 → 21,416」未入（`grep 21,416|1221` 于 knowledge = 0 命中）。**无违规写入**。✅
4. **忠实度逐条比对**：#110/#111/#112 与草稿 A1/A2/A3 正文一致（仅占位符替换 + 家族索引交叉引用改为实号）；A4（#107 家族案例）的置信度行由「judge J3/J4 待审」更新为「J3/J4 = PASS-with-conditions，C1-C10 已应用」（**忠实更新，非失真**）；B1/B2/B3、C1 与草稿一致（`build-tooling #50` 引的 `index.yaml` 行数 1224 已陈旧 → 建议 N1；`algorithm-fingerprints #21` 沿用草稿「端到端确定」措辞 → 建议 N2）。✅（两项为建议级）
5. **`INDEX.md` 追加段**：`:133`（文件末行）逐项列出 #110/#111/#112/#107 家族/algorithm-fingerprints #21/build-tooling #49/#50 **+ 错误台账 `perf-closeout-errors.md`（grep 命中）**，与实际写入条目一致，无未写条目被列入、无已写条目被漏列。✅
6. **错误台账**：`.investigations/perf-closeout-260910-05/perf-closeout-errors.md`（新文件，49 行）含 **E1-E4 五段式 + 末尾「错误→根因」速查表 4 行**（与正文一一对应）、编号说明（与 04 台账 E1-E8 的课题目录边界已声明）。**唯一缺陷 = E2 根因陈述与 nether-C5② 冲突**（→ FIN-C1），以及 E2 的「前四臂」与一手表不符（→ FIN-C2）。
7. **时间线**：`versions/1.21.6/docs/10-timewise-archive.md:66-88` 260910-05 块格式与现网块一致（状态标注 + 口径声明 §9.7 + `状态：` 行），已按条件更新（`:83` async 代理、`:84` J3/J4 已完成且 C1-C10 应用、`:85` 根因未核）；残留两处陈旧：`:82`「ea-r2 是唯一采到 CPU 的臂」（→ FIN-C2）、`:78`「CPU 本轮因脚本缺陷缺失」（→ 建议 N5）。

---

## 三、状态纪律

| 检查项 | 实测 | 判定 |
|---|---|---|
| `verdict-c7` / `verdict-nether-end` status | `verdict-c7:3` = candidate；`verdict-nether-end:3` = candidate | ✅ 无自授 confirmed |
| `index-entry.yaml` | 4 条：2 × kind=analysis/status=candidate，2 × kind=review/status=draft | ✅ 合法（审查意见不进 candidate） |
| 根 `.artifacts/index.yaml` 260910-05 四条目 | `:1193` candidate、`:1209` draft、`:1213` candidate、`:1229` draft | ✅ 合法；无 confirmed |
| 「待用户 confirmed」一致性 | NEXT 标题/`§当前状态`/`会话切换检查`（`:70` 未勾选）与 index 四条目注释均标「confirmed 留用户」 | ✅ 一致 |
| judge 自身是否改 status | 两轮 judge 均声明只出意见；本次审查亦未写 status | ✅ |

> 结论：**状态纪律干净，无自授 confirmed**。

---

## 四、交接一致性（三源核对精神）——对不上的陈述点名

**源① 产物快照**（`.artifacts/perf-closeout-260910-05/`：4 文件 + `index-entry.yaml`）与 **源② git worktree**（⚠️ `/runtime/`、`.tmp/`、`/target/` 均 gitignore——`.gitignore:95/89/100`，我核对通过；Java 侧以 `java-snapshot/pre|post` 为唯一 diff 载体）之间**一致**：

- `java-snapshot/{pre,post}/SHA256SUMS.txt` 各 4 行：`build.gradle`(09c8c8a1…)、`ChunkTiming.java`(e60e757f…) **pre/post 同 sha**；`CppBridge.java`(4cda71a9…→2cd61e0f…)、`NoiseChunkGeneratorMixin.java`(5d22dc66…→aa5522c2…) 变 ⇒ 与两份 verdict 的改动面声明一致（sha 值我未复算，只看「哪些文件变了」）。
- 工作树应用版 = post 快照：`runtime/1.21.6/java/.../NoiseChunkGeneratorMixin.java` 三处 `supplyAsync`（`:148/:186/:222`）、`SYNCFILL:61`、nether/end `intercepted`（`:155/:192`）与 post 快照逐处同位；`CppBridge.java` 的 `MIXLOG:35` + 两处门控 `[WG-FILL]`（`:452/:496`）亦与 post 一致。✅
- `build.gradle:126/130/131/135` 的 `-PrustStages`/`mixlog`/`chunktime`/`syncfill` → `-D` 映射在位，与 NEXT「复测口径」段一致。✅

**源③ 验证记录**与 NEXT 的**对不上项**（点名）：

| # | NEXT/被审文本陈述 | 一手源实际 | 判定 |
|---|---|---|---|
| 1 | `NEXT_SESSION.md:66`「251 条 MANIFEST-sha256（**15 臂** + 11 份对拍 + region + 驱动脚本 + 结果表）」 | MANIFEST 251 条 = 14 顶层 + **26 日志（13 臂）** + 209 region + results + run_arms3；`cmd-output/logs/` 只有 **13 臂**（+ eaS 补跑 = 14 臂，未归档） | ❌ **「15 臂」无出处**（→ FIN-C9；数字口径与 14 顶层/13 臂/11 对拍混合计数） |
| 2 | `NEXT:4`「cmd-output/（251 条 MANIFEST）」+ `index.yaml:1205` 同句；`verdict-c7:102`「本块 cmd-output/ **未做** MANIFEST-sha256 清单…dim 批次完成后一并生成」 | MANIFEST **已在位**（251 条） | ❌ 两处**互相矛盾**（index/NEXT 说有、verdict-c7 说没有）→ FIN-C5 |
| 3 | `verdict-nether-end:25` / `:71` / `record.md:119` 的 eaS 读数（7s、99 CPU·s、4761 行、DIM1 16 mca） | 日志与 results 仅存 `.tmp`（**不入库**，`NEXT:58` 明说）；`MANIFEST` 不含 `region-eaS-r1/*`；`cmd-output/results-260910-05.txt` 无 eaS 行 | ❌ **归档不完整**（→ FIN-C4） |
| 4 | `record.md:47` / `:117` / `verdict-nether-end:85` / 时间线`:82`「（修后）**唯一**臂采到 CPU」 | 一手 `cmd-output/results-260910-05.txt:13` = ea-r2 86，`.tmp/results.txt:14` = eaS-r1 99 ⇒ **两臂** | ❌ 新证据未回填（→ FIN-C2） |
| 5 | `verdict-nether-end:44`「`na-r1 × ns` 有 1 个 chunk 单侧排除（**7567/7543** 差）」 | 一手 `diff-N_async_vs_sync.txt:1-2` = vanilla **7543** / coreswap **7542** / common=7542（7567 是 **end** 的 common） | ❌ **跨维串号的新数字错**（与 C1/E4 同族）→ FIN-C3 |
| 6 | `record.md:47`「本批**前 7 臂** `serverCpu=-1`」；`perf-closeout-errors.md:20`「本批**前四臂**」；nether judge 引「8/9 臂」 | 一手归档表 13 行中 **12 行** `serverCpu=-1`，仅 `ea-r2` 采到 | ❌ 三个计数互斥且均与一手表不符（→ FIN-C2） |
| 7 | `verdict-c7:87`「真根因 = 重复覆盖…脚本已修」 | `run_arms3.ps1:89-113` 无重复块；预修版本未归档 ⇒ 根因未核；`record.md:47`、`verdict-nether-end:85` 已按未核 | ❌ 同事实三处互斥（→ FIN-C1） |
| 8 | `record.md:58`「mask=5 臂实测 `pending_writes≈1221/chunk`（单 chunk 级入缓冲次数）」 | `verdict-c7:35` 已按 C5② 改为量级/上界（全局 atomic、23 线程轮转、区间 0-37,893/中位 1,575） | ❌ 双口径并存（→ FIN-C8） |
| 9 | NEXT 复测臂清单（`:55`）「r3/r3log/r3feat/r3sync + nv/ns/na/naS + ev/es/ea/eaS」 | `run_arms3.ps1:32` 等臂定义逐字吻合（含 eaS）；`chunky center -48 -11`（`:75`）、seed `417950215108767439`（`:9`）亦吻合 | ✅ **对得上** |
| 10 | NEXT 关键读数（`:12` CALines 0/1711、`:15` sync 0.0086%/async 0.0092%、`:18` 5.8-7.2×/23↔1、`:21` 0.1219%≤0.1406%、`:22` 4761、`:23` 86 CPU·s） | 逐项回一手：`results-260910-05.txt`、`diff-*.txt:74`、`r3log-r2/r3feat-r2` 日志、`.tmp` results | ✅ 全部核到 |
| 11 | `verdict-nether-end:16`「7 个 coreswap 臂 `dllsha=abd7d8893d22e030`；nv/ev 为 vanilla」 | 一手 9 行表 = 7 coreswap（ns/na1/na2/es/ea1/ea2/naS）+ 2 vanilla 全对；补跑 eaS 亦为该 sha（`.tmp/results.txt:14`）但**未计入该矩阵** | ✅ 核到（建议 N3 注明 eaS 为第 8 个 coreswap 臂） |
| 12 | `verdict-nether-end:17`「mixin 69 行差异」/「CppBridge 10 行差异」 | 行数净变：pre mixin 196 → post 253 = **+57**；「69」应为变更行数（未声明口径），我无 shell 不能算 diff | ⚠️ **未核**（建议 N4 注明口径） |

**源①②③ 三方交叉结论**：块级结论与一手 diff/日志**对齐**；不一致集中在**「补跑 eaS 之后未回填/未归档」**与**「E2 根因双口径」**两类，均为收尾层问题，不动摇 C7 与 P4 的实质结论。

---

## 五、`knowledge/INDEX.md` 格式

1. 全文 **133 行**；`grep '```'` = **0 命中**（围栏完全配平——不是「偶数」，而是零围栏，符合现网「正文用 `>` 引用段落」的既有风格）。✅
2. 260910-04 草稿脚手架**已清除**：`grep '## D INDEX 追加段|\*\*目标文件\*\*|\*\*插入位置\*\*|\*\*占位替换\*\*'` = **0 命中**；04 段（`:130-131`）已作为**正式引用段落**展开（`> 260910-04 追加：…` / `> 260910-04 R3 追加：…`）。✅
3. 本次新增段 `:133`「`> 260910-05 追加：…`」位于**文件末尾**（末行），符合「追加到末尾」纪律；内容与会话实际写入条目一致（见 §二.5）。✅
4. 遗留观感项（非格式违规）：`:126-129` 有 4 行连续空行（脚手架删除后的留白），可顺手压缩。

---

## 六、必改条件（FIN-C1..FIN-C9，逐条可执行；均为静态修正）

- **FIN-C1（MUST，实质）E2 根因全链一致化——消解 C7-C9① ↔ nether-C5② 冲突（以新条件为准）。**
  ① `perf-closeout-errors.md:18-24` + 速查表`:47`：把「根因（机制）= 停服后重复采样覆盖」改为「**根因（机制候选，推断·未核）**」，并写明证据边界：「预修版本未归档——`.tmp` 与 `cmd-output` 两份 `run_arms3.ps1` 均为修后版（`:89-91` 停服前采样、`:112` 为『不得重复采样』注释、`:113` 算 cores，**无停服后采样块**），故『停服后覆盖』无法文件级复核；行为侧旁证 = 修后跑的两臂（`ea-r2`/`eaS-r1`）采到 CPU」；「修复」段删「后续臂恢复采集」的因果断言，改为「调用序修正（以归档版为证）+ 旁证」。② `verdict-c7:87` 删去「真根因 = 重复覆盖…judge C9 已核实，脚本已修」，改为与 `record.md:47` / `verdict-nether-end:85` 同口径（根因未核；引 nether-C5②）。③ `run_arms3.ps1:89/112` 两处注释补「（推断·未核）」标注（仅注释，不改行为）。④ 在 E2 内加一行「条件冲突记录：C7-C9① 与 nether-C5② 冲突，以 nether-C5②（现版脚本直读 + 归档版本唯一）为准」。
- **FIN-C2（MUST）CPU 采集臂数与陈述回填（4 处 + 1 个计数口径）。** 依据一手 `cmd-output/results-260910-05.txt`（13 行中 **12 行** `-1`，仅 `ea-r2`=86）与 `.tmp/perf-reg-260910-05/results.txt:14`（`eaS-r1`=99）：① `record.md:47`「本批前 7 臂」→「归档 13 行中 12 行为 `-1`，`ea-r2`(86)/`eaS-r1`(99) 采到」；② `record.md:47` 末「中途修复后唯一臂采到 CPU」、`record.md:117` 表注「唯一采到 CPU 的臂」、`verdict-nether-end:85`「修后唯一臂采到 CPU」、`10-timewise-archive.md:82`「`ea-r2` 是唯一采到 CPU 的臂」→ 一律改「**两臂**（`ea-r2` 86 / `eaS-r1` 99）」；③ `perf-closeout-errors.md:20`「本批前四臂…唯 ea-r2」→ 同一口径。
- **FIN-C3（MUST）nether 门单侧排除数字更正。** `verdict-nether-end:44`「（**7567/7543** 差）」→「（**7543/7542** 差）」，依据一手 `cmd-output/diff-N_async_vs_sync.txt:1-2`（vanilla chunks=7543 / coreswap chunks=7542 / `common=7542`）；`7567` 是 **end** 的 `common`，属跨维串号，须在 `perf-closeout-errors.md` 的 E4 家族补一行（「条件应用期新引入的数字错 = 同族三犯」）。
- **FIN-C4（MUST）eaS 一手证据归档补齐**（`verdict-nether-end` C3 闭合的证据载体）：① 复制 `.tmp/perf-reg-260910-05/logs/eaS-r1.log` 与 `eaS-r1.log.err` 到 `cmd-output/logs/`；② `cmd-output/results-260910-05.txt` 回填 `eaS-r1` 行（取自 `.tmp/.../results.txt:14`），或另存 `results-260910-05-full.txt` 并在 verdict 注明；③ `cmd-output/MANIFEST-sha256.txt` 增补 `logs/eaS-r1.log{,.err}` + `region-eaS-r1/*`（16 条）+ 更新后的 results 条目；④ `verdict-nether-end` §2 注明 eaS 一手证据的归档位置，`NEXT_SESSION.md:4/66` 的 MANIFEST 描述同步。
- **FIN-C5（MUST）`verdict-c7:102`（§5）陈旧陈述更正。** 「本块 `cmd-output/` 未做 MANIFEST-sha256 清单（judge N6）：dim 批次完成后一并生成（含 region 80+ 文件与 diff 输出）」与实况矛盾且与 `index.yaml:1205`/NEXT`:66` 冲突 ⇒ 改为「MANIFEST 已生成：251 条（13 臂日志 + region 209 + 11 份 diff + 脚本/结果表）；eaS 补跑证据见 FIN-C4 补齐后的条目」。
- **FIN-C6（MUST）nether-C1 的 §5.2 落地点补改。** `verdict-nether-end:84`「nether 0.1020% 落在**自身**噪声带内」→「落在 **async 同形态代理噪声（0.1406%）**带内（该维无 sync run-to-run 对，代理口径见 §3.1）」；`:43` 的「nether 自身（async 代理）噪声」亦建议统一为「async 同形态代理噪声」。
- **FIN-C7（MUST）nether-C2④ 内联限定落字。** `verdict-nether-end:93`（§6）的「end 跨形态 **0 块差**」处内联补「（比对集 `common=7567` chunks、两侧 section 并集 494,100,480 块）」，不以「限定在 §3/§5 声明的范围」指针替代（judge 原文要求内联）。
- **FIN-C8（MUST）`record.md:58` 的 `pending_writes≈1221/chunk` 口径更正。** 按 C7-C5②/判据一致化改为「量级/上界：`CA_PENDING_WRITES` 为全局 atomic、每 chunk `store(0)`、23 线程并发轮转 ⇒ 不得读作 per-chunk 精确值；一手区间 0–37,893、中位 1,575」，与 `verdict-c7:35` 同口径。
- **FIN-C9（MUST）`NEXT_SESSION.md` 自述数字更正。** `:66`「15 臂」→「13 臂日志（MANIFEST 覆盖）+ eaS 补跑 = 14 臂（FIN-C4 归档后）」；`:4`/`:66` 注明 MANIFEST 当前不含 eaS（FIN-C4 补齐后删注）；`:55` 臂清单建议直接引用 `run_arms3.ps1` 臂定义以免计数漂移。

## 七、建议项（N1..N5，不阻塞；不改变判定）

- **N1**：`build-tooling.md:942`「现网 `.artifacts/index.yaml`（共 **1224** 行）」已陈旧（现 **1229** 行，且注释随块增补继续增长）——建议改为「（260910-05 时 1224 行）」或删行数、只留「307 行注释」的机制判据。
- **N2**：`algorithm-fingerprints.md:456/466` 与 `INDEX.md:133` 的「end = 0（**端到端确定**）」——nether-C4 已禁「端到端」措辞、C2 把确定性限定在比对集内 ⇒ 建议改为「end = 0（**比对集内两 run 逐块零差**）」，避免知识库与 verdict 的口径边界不一致。
- **N3**：`verdict-nether-end:16`「7 个 coreswap 臂」建议补一句「`eaS-r1`（补跑）为同 sha 的第 8 个 coreswap 臂，不入本 9 臂矩阵」。
- **N4**：`verdict-nether-end:17` 的「mixin **69 行差异** / CppBridge **10 行差异**」未声明口径（净增为 mixin 196→253 = +57）；建议注明「= 变更行数（增+删）」，或补净增值。
- **N5**：`10-timewise-archive.md:78`「CPU 本轮因脚本缺陷缺失」与 FIN-C1 的「根因未核」口径不一致，建议同批改为「CPU 采集缺失（根因未核）」。

## 八、对提交（commit）的明确意见

- **不同意在 FIN-C1..FIN-C9 应用前提交**：其中 **FIN-C1（事实层自相矛盾，落在最高优先级资产错误台账）**、**FIN-C3（一手数字错，与 C1/E4 同族三犯）**、**FIN-C4（C3 闭合证据未入库）** 是「带错交付」级；FIN-C2/C5/C6/C7/C8/C9 是口径/回填级（低成本）。
- **FIN-C1..FIN-C9 应用后：同意提交**。理由：本块两件工作均已过 judge（PASS-with-conditions）且**实质项已验证落地**（§一），剩余为静态修正；提交**不等于** confirmed——两份 verdict 与根索引四条目保持 `candidate`，`NEXT_SESSION.md:70` 的「用户 confirmed」保持未勾选，commit message 不得出现 confirmed/verified 语义。
- **不得**在提交时把「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响 / 接管已对齐 / 端到端持平-反超」写入任何文本（延续两轮 judge 立场；本块证据不支持）。
- **建议**：FIN-C1/C4 的修正完成后，把「条件冲突消解（C7-C9① ↔ nether-C5②）」与「eaS 归档补齐」在 NEXT 的「会话切换检查」中各占一行，作为下一会话可复核的闭合证据。

## 九、未核项与诚实边界

1. **无 shell**：未跑任何命令（未 `git status/diff`、未复算 sha256、未跑 diff 工具、未数 MANIFEST 行数以外的一致性校验）。`HEAD=42d46e7`、`/runtime/`+`.tmp/`+`/target/` gitignore（`.gitignore:95/89/100` 我直接读到）、`java-snapshot` 的 sha 值、MANIFEST 的 sha 值、`target/release/worldgen1216.dll` 相等性，均为**继承/未复算**。
2. **未核**：`region-*/**.mca` 二进制内容（只核文件存在、每臂 16 个、MANIFEST 覆盖 13 臂 209 条 + `region-eaS-r1` 存在但未入 MANIFEST）；`diff_arms.py` 的解码/BitPack 正确性（只核其 `blocks=` 行与我抽样的一手输出自洽）。
3. **未核**：`mixin 69 / CppBridge 10` 的差异行数（无 diff 工具；仅核到 pre/post 净行数 196→253 与「69 应为变更行数」的解释空间）。
4. **未核**：`.tmp/perf-reg-260910-05/logs/r3feat-r2.log.err` 的 `pending_writes` 分布（中位 1,575 / 区间 0-37,893）——我未做全量统计，只核到该口径在 `verdict-c7:35`、`record.md:58`（旧值）与草稿 §0.5 三处出现且**不一致**。
5. **未核**：`eaS-r1` 的精确 4761/4761 行计数（我核到日志内两类行均大量存在、`:96 initEnd`/`:14425 Task finished 0:00:07`，未逐行计数；`naS`/`eaS` 日志总行数与「3 + 2×4761」一致，支持该计数）。
6. 我**未修改任何被审文件**（四份 verdict、record、errors、draft、patch、cmd-output 全部文件、java-snapshot、runtime 源、knowledge 三文件、INDEX.md、时间线、NEXT、两处 index 均未动）；**本文件是本次审查的唯一写入**。
