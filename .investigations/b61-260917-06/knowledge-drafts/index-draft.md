# B6-1（260918-01）知识库草稿：`knowledge/INDEX.md` 追加行

```yaml
status: draft
block: 260918-01（B6-1 第二轮收尾）
role: core.worker（knowledge subagent 草稿；主会话应用）
target: knowledge/INDEX.md 文件末尾（现尾行 = 191 行「260917-05 追加（二）」块，本行接其后，前置一个空行）
numbering: 占位编号 #168 / #169 / #170（workflow-patterns）与 #155（build-tooling）；应用时按现网尾号顺延确认
not_done: 未改动 INDEX.md 现有任何行；未写入 knowledge/（应用由主会话执行）
```

> 体例说明（对齐现网尾行）：单段 `>` 引用块 + 「`> <日期> 追加（<括注>）：` + 各载体「新增/补充案例」项 + `来源：` 指针 + `时间线 →` 指针。**日期与 commit 已一手核验**（HEAD `b30d008`；本块提交链 `3e0d434 → 1ff3a6f → 7abed4d → 56dc15c → cf77963 → ab91a0e → 0781f98 → 0b0c7ac → b30d008`）。

---

**（以下为可直接粘贴的行，单行不换行）**

> 260918-01 追加（**B6-1 开关生效机械对账第二轮收尾**：union 口径缺陷自曝 + 1.21.6 侧缺口实测 + 13 项补映射发射实证闭合）：workflow-patterns 新增**发现 #168（最高价值·错误优先）**（**聚合口径掩盖分项缺陷**——跨版本对账门对两版 `build.gradle` 取 **union** 后算 ORPHAN ⇒ **先被修好的版本会制造对后修版本的掩盖**：本块修好 1.20.1 的 `blobProbe.*`/`colprof.*`/`surfacedump.dim`/`bulkwb*`（ORPHAN 24 → 11，−13 与补映射逐名吻合）后，1.21.6 的**同名缺口 10 项静默消失**；per-version 口径一开即暴露 **1.20.1 缺口 11 / 1.21.6 缺口 21**（1.21.6 另有独有缺口 `coreswap.stallwatch`）；消费点一手核对确证 1.21.6 至少 3 项为真缺陷（`coreswap.bulkwblog` → `BulkWb.java:91`+`CppBridge.java:628`、`coreswap.stallwatch` → `StallWatch.java:21`、`coreswap.wbcontent`）而非旁路；判据 = ① 跨载体/版本聚合的对账门 MUST **同时给 per-carrier 视角**（掩盖方向与「哪个载体先被修好」耦合）② `--strict` 判据域须与「用户可踩到的路径」对齐（**任一单版缺口即失败**）③「并集口径 + 报告绿」= #163 silently-green 门的**聚合形态**；修复已落地 = `scripts/check_switch_mapping.py:104-140` 新增 per-version 缺口段 + `:160-164` `--strict` 语义升级（DEAD 非空 **或** 任一版本有缺口即非零退出），commit `0b0c7ac`；#105 载体偏差家族的**门禁口径**形态）+ **发现 #169 简记（中价值）**（**判据的判据：机械对账工具的覆盖面自身必须可核**——本块抽取器首版只扫 `versions/1.20.1/java/src` 一个消费载体得 DEAD 32 / ORPHAN 18，补全载体（+1.21.6 java src + java-core + worldgen-core）后变 **DEAD 4 / ORPHAN 24**（终态 = `switch-sources.json` M1 116 / M2 136）；判据 = 工具输出 MUST 随附 coverage（含 `NOT_SCANNED` 未扫面）+「DEAD N / ORPHAN N / 缺口 N」不可脱离覆盖面引用 + 覆盖面扩张即判据重跑触发点；#164 数据面）+ **发现 #170（中-高价值）**（**发射面直证法**——Gradle init script（`-I`）在 `runServer` 的 `doFirst` 打印 `t.jvmArgs` = JVM 参数的**发射点本体**（源码说「应该发射什么」，`jvmArgs` 是「**实际**发射了什么」）；关键增量 = **组内标定对照**：同轮同族三开关两个发射（`chunkX`/`chunkZ`）、一个不发射（`dim`）⇒ 同轮成功行自证仪器有效，缺席升为**二值强判据**；配 SCOPED 正向对照臂（heightProbe 族零 `-P` 先例 → 仅发射父门）+ **13/13 全实证闭合**（sentinel-D 一轮传齐 13 个 `-P`，替代此前 6/13 实证 + 7 项静态外推）；分层 = **Partial**（止于参数发射面，端到端行为未验证）；首轮三处缺陷（哨兵 B 输出不存在 / 哨兵 A 的 `BUILD FAILED` 未披露 / 缺标定对照）已如实留痕；#37/#81/#118 家族方法面）+ **发现 #171 简记**（**判据与豁免子句 MUST 合读**——「族内对称性判据」（族内 ≥1 个 `-P` 映射先例 ⇒ 未映射项 = DEFECT；族内零先例 ⇒ SCOPED）单独引用会把 3 项合法旁路误判为缺陷，完整表述 = `t4-adjudication.md` §2 规则 **+** §3.4 豁免子句（**探针专用路径** + **缺省权威**，须逐条登记 reason：`bench.threads` / `bench.worldgen` / `colDump.targets`）；另有「同一现象两种相反解释用**可 grep 的结构事实**区分而非读 javadoc 猜意图」的方法面——b1 持 javadoc（`-D` 用法明写）判旁路、b2 持族内映射事实判缺陷，双方各自持有对方没有的判据，裁决 = 两条合读）；build-tooling 新增**发现 #155（错误优先，五段式）**（**`.gitignore` 整目录忽略吃掉「门禁资产」**——原 `.gitignore:51` 的 `scripts/` 使 AGENTS.md 明文引用的门禁（`scan_cpp_anchors.py` / `check_switch_mapping.py` / `merge_index.py`）**在仓库中不存在**，其存续只依赖单机磁盘；根因 = **git 不支持在被排除的目录内 re-include**（`scripts/` + `!scripts/xxx.py` 是**静默失效**写法，`!` 行看着在、恒不生效）⇒ 修复 MUST 用 `scripts/*` + 白名单；已改并入库三个门禁脚本（commit `ab91a0e`），`protocol/` 口径未动；判据 = 被判据/文档明文引用的资产 MUST 在版本控制内 + 引用物清单对 `git ls-files`/`git check-ignore -v` 双向核验 + 修复后双向核验（该在的在、不该在的没进）；#24 同族第二形态 + workflow #164 数据面）。来源：`.investigations/b61-260917-06/`（`extension-per-version-gap.md`（union 掩盖 + 1.21.6 缺口实测 + 3 项确证）/ `t4-adjudication.md`（族内对称性判据 + §3.4 豁免子句）/ `t5-landing.md`（T5 落地 + §4.0 发射实证 + §8 遗留）/ `phase25-verification.md`（三哨兵 + 首轮错误披露）/ `candidates/.b1/.b2`（fan-out 两分支）/ `knowledge-drafts/`（本轮 subagent 草稿三份））+ 门禁实现 `scripts/check_switch_mapping.py` + 数据副本 `switch-sources{,-perversion,-postfix}.json`；judge `review-001.md`（PASS-with-conditions，M1-M3+S1-S4 已闭合）与 `review-002.md`（PASS-with-conditions，M-new-1/M-new-2 已闭合）。**状态：draft（candidate 待 judge / confirmed 留用户）**；1.21.6 的 21 项**未逐项定性**（多数疑为同类旁路，**不得按 21 个缺陷计**），1.21.6 boot **未实跑**（Degraded）——建议另立 B6-2。时间线 → `versions/1.20.1/docs/10-timewise-archive.md` 260918-01 块。

---

**应用提示（主会话）**

1. 追加位置 = `knowledge/INDEX.md` 末尾（第 191 行之后），前置一个空行，保持与 `260917-05 追加（二）` 行同一体例。
2. 编号：应用时先 `grep -n '^## 发现 #' knowledge/discovered/{workflow-patterns,build-tooling}.md` 复核现网尾号（起草时 = workflow **#167** / build-tooling **#154**），按结果顺延后再落 INDEX 行（本行占位 #168/#169/#170/#171 + #155）。
3. 若主会话**否决或合并**某条（见草稿 `discovered-draft.md` 的候选处置说明），MUST 同步删改本行对应括注，避免 INDEX 与 discovered 正文不一致。
