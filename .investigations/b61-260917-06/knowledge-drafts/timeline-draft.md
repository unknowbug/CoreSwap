# B6-1（260918-01）知识库草稿：`versions/1.20.1/docs/10-timewise-archive.md` 追加块

```yaml
status: draft
block: 260918-01（B6-1 第二轮收尾）
role: core.worker（knowledge subagent 草稿；主会话应用）
target: versions/1.20.1/docs/10-timewise-archive.md（3360 行，末尾「---」之后追加）
position_verified: 文件末尾 = 260917-05 块（正文 + 「同块追记：框架升级执行面」子块）→ 第 3360 行 `---`；本块为其后的新日期块
date_anchor: Get-Date 实测 2026-09-18 10:43:46 +08:00；本块工作跨 260917-06（T1-T5 + 哨兵，实际 09-17）与 260918-01（第二轮收尾，实际 09-18）两日，按 §三.5 日期纪律归入新日期块 260918-01
not_done: 未改动文件现有内容；未写入 docs/（应用由主会话执行）
```

**应用位置**：现网末行 = `---`（第 3360 行，260917-05 块的收尾分隔线）。本块接其后追加，块首为 `## 260918-01（…）`，块尾亦以 `---` 收束（与 260917-03/04/05 三块体例一致）。

---

## 260918-01（实际 2026-09-18，Get-Date 锚 10:43；**B6-1 开关生效机械对账第二轮收尾**——union 口径缺陷自曝 + 1.21.6 侧缺口实测 + 13 项补映射发射实证闭合）🔍 draft（judge review-001/review-002 均 PASS-with-conditions 且条件已闭合；**confirmed 留用户**；1.21.6 侧为 static 抽取 Degraded）

> 承接 260917-06（B6-1 立项块：T1/T2 抽取 → fan-out b1/b2 → T4 汇聚裁决 → 三哨兵 → T5 落地，提交链 `3e0d434`→`1ff3a6f`→`7abed4d`→`56dc15c`）；本块（260918-01）处理其收尾遗留并自曝门禁口径缺陷。判据依据 = `.investigations/b61-260917-06/t4-adjudication.md`（族内对称性判据 + §3.4 豁免子句）+ `phase25-verification.md`（三哨兵）。
> **本块提交链**：`cf77963`（scripts/ gitignore 发现记录）→ `ab91a0e`（judge-002 条件闭合 + 脚本入库）→ `0781f98`（sentinel-D：13 项映射发射实证闭合）→ `0b0c7ac`（门禁 per-version 段修复）→ `b30d008`（1.21.6 确证 + B6-2 建议）。HEAD = `b30d008`。

- ✅ **门禁 per-version 口径修复（本块最重要产物，且是「门禁自曝缺陷」的实证）**：`scripts/check_switch_mapping.py` 新增 **per-version 缺口段**（按版本分别算 `consumed_ver ∪ consumed_shared ∖ declared_ver`），`--strict` 语义升级 = **DEAD 非空 或 任一版本有缺口** 即非零退出，并打印「union 口径的 ORPHAN 会掩盖单版本缺口——请以 per-version 段为准」警示行。→ 通用模式 **workflow-patterns #168**（commit `0b0c7ac`）。
- ⚠️→✅ **本块自曝缺陷：union 口径掩盖单版本缺口（五段式见草稿 #168）**：原实现对两版 `build.gradle` 的 `-D` 名取**并集**（M1）、消费面取并集（M2），`ORPHAN = M2 ∖ M1` ⇒ **任一版本声明过的名字对所有版本的缺口同时消失**，且**掩盖方向与「哪个载体先被修好」耦合**——本块修好 1.20.1 的 `blobProbe.*`/`colprof.*`/`surfacedump.dim`/`bulkwb*`（ORPHAN **24 → 11**，−13 与补映射项逐名吻合、judge 独立复算零回归）后，**1.21.6 的同名缺口 10 项静默消失**。判据 = 跨载体聚合门 MUST 同时给 per-carrier 视角 + `--strict` 判据域与「用户可踩到的路径」对齐（任一单版缺口即失败）。家族 = **#105 载体偏差的门禁口径形态 + #163 silently-green 门的聚合形态**。
- ⚠️ **1.21.6 侧同病实测（per-version，含共享 carrier，static 抽取 Degraded）**：1.20.1 声明 128 / 本版+共享消费 136 / **缺口 11**；1.21.6 声明 106 / 本版+共享消费 124 / **缺口 21**（另含独有缺口 `coreswap.stallwatch`）。21 项中 **10 项 = 本块刚在 1.20.1 修好的同名缺陷**。**消费点一手核准确证至少 3 项为真缺陷**（非旁路）：`coreswap.bulkwblog`（`BulkWb.java:91` + `CppBridge.java:628`，1.21.6 `build.gradle` 零命中 ⇒ 生产写回链诊断仪器不可达）、`coreswap.stallwatch`（`StallWatch.java:21` ⇒ 停滞看门狗无法经 `-P` 启用）、`coreswap.wbcontent`（同族写回读回层，同 #150 家族）。**未逐项定性**：其余多数疑为与 1.20.1 同类的合法旁路（`bench.*` / `height.*` / `chunkRandom.seed` / `java.io.tmpdir`），**不得按 21 个缺陷计**；1.21.6 boot **未实跑**（Degraded）。
- ✅ **13 项补映射发射实证闭合（原 7 项静态外推已消解）**：sentinel-D 一轮传齐**全部 13 个 `-P`**，实测 **13/13 发射无缺席**（`cmd-output/sentinel-D-all13.log` 原文：`-Dcoreswap.exec=0` / `-Dcoreswap.maxinflight=8` / `-Dcoreswap.bulkwblog=1` / `-Dcoreswap.wbcontent=1` / `-Dcoreswap.bulkwbtest=1` / `-Dcoreswap.bulkwbsentinel=1` / `-DblobProbe.chunkX=1` / `-DblobProbe.chunkZ=2` / `-DblobProbe.size=3` / `-DblobProbe.dim=minecraft:overworld` / `-Dcolprof.x=4` / `-Dcolprof.z=5` / `-Dsurfacedump.dim=minecraft:the_nether`）→ T5 §8 遗留项 4 闭合；分层仍为 **Partial**（止于参数发射面）。→ 通用模式 **workflow-patterns #170**（发射面直证法，commit `0781f98`）。
- ✅ **scripts/ gitignore 缺陷处置（原「仅记录」→ 本块落地）**：`.gitignore` 由 `scripts/`（整目录）改为 **`scripts/*` + 三条 `!` 白名单**（`check_switch_mapping.py` / `merge_index.py` / `scan_cpp_anchors.py`），三脚本入库（commit `ab91a0e`；`protocol/` 仍整目录忽略）。**根因** = **git 不支持在被排除的目录内 re-include** ⇒ `scripts/` + `!scripts/xxx.py` 是**静默失效**写法（`!` 行看着在、恒不生效）；此前 AGENTS.md 明文引用的门禁在仓库中**不存在**（只有本机有），换机/CI 即失。核验 = `git ls-files scripts/` 恰含三项。→ 通用模式 **build-tooling #155**（#24 同族第二形态）。
- 📌 **判据完整表述的合读纪律（judge M1 补，本块登记）**：**「族内对称性判据」单独引用会误判**——完整表述 = `t4-adjudication.md` **§2 规则 + §3.4 豁免子句**（**探针专用路径** ∧ **缺省权威**，二者须逐条登记 reason）；三条豁免项 = `bench.threads` / `bench.worldgen` / `colDump.targets`（族内虽有 `-P` 先例，但其先例服务的是「把探针输出导向文件」类通用参数，而该项是诊断性覆盖且缺省即权威 ⇒ SCOPED-DIRECT-D）。另一方法面 = **同一现象的两种相反解释用可 grep 的结构事实区分**：b1 持消费点 javadoc（明写 `-D` 用法）判旁路、b2 持族内映射事实判缺陷——双方各自持有对方没有的判据，裁决 = 两条合读（`colprof.*` 类「族内有先例」判缺陷；`height.*` 类「族内零先例」判旁路；b2 从未判 `height.x/z` 为缺陷，本动作属对其让渡项的确认而非推翻）。→ 通用模式 draft #171（INDEX 行已含；正文可选）。
- ❌ **被排除/让渡项（保留不删）**：① `max.bg.threads` = **UNRESOLVED**（消费方疑在 MC 发行 jar 内 `Util.getAvailableBackgroundThreads`，不在本仓库扫描面 ⇒ **不判缺陷也不判废弃**，**不得与 fjp1 同格引用**——机制不同：fjp1 = JVM 读了但被专用池架空，该项 = 消费方可能在别处）；② 死开关**只加注记未删**（`biome6oct` / `cpp.noBatch` / `ForkJoinPool.common.parallelism`）——删除改变用户可见开关面，属行为面变更，**待用户裁决**；③ 1.21.6 侧 b2 声明的 `BulkWb.java:81` 类注释与 `1.21.6/10-timewise-archive.md:112`「`BULKWB_ON` 已翻转」**状态不一致**（本分支未复核）⇒ 转 1.21.6 覆盖课题。
- 🔍 **遗留（交用户裁决 / 建议另立 B6-2）**：① **B6-2：1.21.6 侧开关映射补齐**（范围 = gate per-version 清单 → T4 族内对称性定性 → 补映射 → 哨兵验证；可立即做的最小项 = 补 `-Pstallwatch`/`-Pbulkwblog`/`-Pwbcontent`/`-Pbulkwbtest`/`-Pbulkwbsentinel` 五行）；② 死开关删除裁决；③ `max.bg.threads` 补核（需反编译或实测）；④ **端到端 Full 验证未做**（本块止于参数发射面）；⑤ 其余 ~6 项 DEFECT（探针族子参数未逐个补）按族代表 + 规则外推，未逐项哨兵。
- 📌 **记录指引**：通用模式 → workflow-patterns **#168/#169/#170**（+ #171 简记）+ build-tooling **#155**（subagent 草稿 + 主会话应用；草稿见 `.investigations/b61-260917-06/knowledge-drafts/`）；证据指针 → `t5-landing.md` §4.0/§7/§8 + `extension-per-version-gap.md` §2/§6 + `phase25-verification.md` §3/§4 + `review-001.md`/`review-002.md`（judge 独立复算 ORPHAN 24→11 零回归）；INDEX 尾注行同批落盘。

---
