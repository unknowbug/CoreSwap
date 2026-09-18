# B6-1（260918-01）知识库草稿：discovered 新发现

```yaml
status: draft
block: 260918-01（B6-1 第二轮收尾）
role: core.worker（knowledge subagent 草稿；主会话应用）
applies_to:
  - knowledge/discovered/workflow-patterns.md（尾部追加；现网最大编号 = #167，本批起 #168）
  - knowledge/discovered/build-tooling.md（尾部追加；现网最大编号 = #154，本批起 #155）
value_gate: 高价值详记（A/B/C/E）；中价值简记（D/F）
numbering: 编号为占位建议（#168-#170 / #155），主会话应用时按当时现网最大编号顺延确认
not_done:
  - 未改动任何既有条目 / 未改任何 status（全部 status: draft）
  - 未写入 knowledge/ 或 docs/（本文件仅为草稿，应用由主会话执行）
```

**编号占用声明（应用前必读）**：起草时现网尾号为 workflow-patterns **#167**、build-tooling **#154**。本批草稿占位 **workflow-patterns #168 / #169 / #170**、**build-tooling #155**；若主会话应用前知识库又有新增，MUST 顺延取号后再写（编号即身份，不可回头改）。

**一手材料**（本草稿全部数字与判据均取自此六项，无外部推断）：
1. `.investigations/b61-260917-06/extension-per-version-gap.md`（union 口径掩盖单版本缺口）
2. `.investigations/b61-260917-06/t4-adjudication.md`（T4 汇聚裁决：族内对称性判据 + §3.4 豁免子句）
3. `.investigations/b61-260917-06/t5-landing.md`（T5 落地 + §4.0 发射实证 + §8 遗留）
4. `.investigations/b61-260917-06/phase25-verification.md`（三哨兵 + 首轮错误披露）
5. `scripts/check_switch_mapping.py`（门禁实现：union + per-version 两段）
6. `.investigations/b61-260917-06/candidates/.b1-bypass-classification.md` / `.b2-defect-classification.md`（fan-out 两分支）

---

## 一、追加到 knowledge/discovered/workflow-patterns.md 末尾

### 发现 #168（最高价值·错误优先，五段式）: 聚合口径掩盖分项缺陷——取 union 的对账门让「先被修好的载体」制造对「后修载体」的掩盖（260918-01）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-01 B6-1 第二轮收尾）；主会话延伸核查（触发 = 遗留项 #3「1.21.6 侧是否同病」）+ 本 knowledge subagent 起草；**draft**（本块门禁口径修复为**已落地并入库**：commit `0b0c7ac`，静态抽取 + 门禁实跑；1.21.6 侧「真缺陷」判定的行为化验证未做——1.21.6 boot 未实跑，Degraded）；workflow-patterns / 门禁与对账工具设计（**#105 载体偏差家族**的门禁形态 + **#163 计数门/silently-green 门家族**的聚合维）。
- **来源定位**：`.investigations/b61-260917-06/extension-per-version-gap.md` §1-§3/§6 + `scripts/check_switch_mapping.py:104-140`（`collect()` 的 `pv_m1/pv_m2` per-version 段 + `:127-139` 缺口打印）+ `:160-164`（`--strict` 语义升级）+ `.investigations/b61-260917-06/switch-sources.json`（M1 116 / M2 136 / DEAD 4 / ORPHAN 24）与 `switch-sources-perversion.json`（M1 **129** / M2 136 / ORPHAN **11**，两版缺口 1.20.1 = **11** / 1.21.6 = **21**）。

- **五段式（错误优先）**：
  - **现象**：B6-1 首轮交付后，门禁在 union 口径下报 ORPHAN 24，修复 1.20.1 的 13 项后报 ORPHAN 11（**−13**，与补映射项逐名吻合，judge 独立复算确认零回归）——**看起来是干净收敛**。但同一批开关在 1.21.6 侧的**同名缺口 10 项**（`blobProbe.{chunkX,chunkZ,size,dim}` / `colprof.{x,z}` / `surfacedump.dim` / `coreswap.{bulkwblog,wbcontent,bulkwbtest,bulkwbsentinel}` 中的大部分）**从未出现在任何一版报告里**；per-version 口径一开，立刻暴露 **1.21.6 缺口 21 / 1.20.1 缺口 11**，且 1.21.6 另有独有缺口 `coreswap.stallwatch`（1.20.1 已声明）。
  - **根因（机制）**：门禁原实现对两版 `build.gradle` 的 `-D` 名取**并集**（M1）、消费面也取并集（M2），`ORPHAN = M2 ∖ M1`。于是**任一版本声明过**的名字都进了 M1 ⇒ 该名字对所有版本的缺口**在集合差里同时消失**。掩盖的**方向不是随机的，而与「哪个载体先被修好」耦合**：先修的载体把名字**注入并集**，从而**制造**对仍未修载体的掩盖——修得越早、掩盖越隐蔽。这正是 #105（载体偏差：「载体全绿对真实故障零判别力」）在**门禁口径**上的同构形态，也是 #163（判据只落在「存在/数量」层 ⇒ 静默绿）的**聚合形态**：聚合是「数量对」的又一种伪装。
  - **定位（怎么发现的，可复用）**：① **跨载体对照核查**——按遗留项沿「另一版是否同病」直接对 1.21.6 侧跑同一套抽取，per-version 口径下缺口 21 现身；② **差集交叉**——把 21 项与「本块刚在 1.20.1 修好的 13 项」取交，得 10 项同名（掩盖的直接指纹）；③ **消费点一手核对确证非旁路**——抽 3 项读 1.21.6 侧消费点（`coreswap.bulkwblog` → `BulkWb.java:91` + `CppBridge.java:628`；`coreswap.stallwatch` → `StallWatch.java:21`；`coreswap.wbcontent` 同族写回读回层），而 1.21.6 `build.gradle` **零命中** ⇒ 确证为真缺陷而非合法旁路。
  - **修复**：`scripts/check_switch_mapping.py` 新增 **per-version 缺口段**——按版本分别算 `consumed_ver ∪ consumed_shared ∖ declared_ver`，打印「`-- per-version 缺口（union 口径掩盖的单版本缺陷）--`」段 + 每版缺口清单；`--strict` 语义升级为 **DEAD 非空 或 任一版本有缺口** 即非零退出（判据域对齐「用户可能只跑其中一版」）；并打印显式警示行「union 口径的 ORPHAN 会掩盖单版本缺口——请以 per-version 段为准」。落地 commit = `0b0c7ac`。
  - **教训（判据，可复用）**：
    1. **判据（MUST）**：凡「跨多个载体/版本聚合的对账门」，MUST **同时给出 per-carrier 视角**；聚合视角（union / intersection）会掩盖单载体缺口，且**掩盖方向与「哪个载体先被修好」耦合**（先修的载体制造对后修载体的掩盖）。
    2. **判据（MUST）**：`--strict` 类门禁的**判据域须与「用户可踩到的路径」对齐**——用户可能只跑其中一版，故**任一单版缺口都应是失败**（不能以「并集里没有」作通过依据）。
    3. **反模式**：「并集口径 + 报告绿」是与 #163 silently-green 门同构的**聚合形态**——差集数字漂亮不代表覆盖面完整。
    4. **配套（可复用操作）**：聚合门升级时**保留聚合视图**（它是跨载体去重的廉价前置），把 per-carrier 视图作为**强判据**；两者的判据域与结论强度 MUST 分别声明（同 #163 判据 4 的「计数门前置 + 内容门后置」形态）。
- **边界（idk）**：1.21.6 的 21 项中大部分（`bench.*` / `height.*` / `chunkRandom.seed` / `java.io.tmpdir`）**疑为与 1.20.1 同类的合法旁路，未逐项定性**——原文明确「**不得直接按 21 个缺陷计**」。本发现只确证「union 口径会掩盖单版缺口」这一**口径事实**（已落地实证），不主张 21 项均为缺陷（已确证 3 项）。
- **家族索引**：#105（载体偏差——本条为其**门禁口径**形态：不是验证载体不同，而是对账口径把多载体压成一个集合）/ **#163（silently-green 门——本条补「聚合」这一新形态：判据落在并集这个更高层级上）**/ #164（覆盖面——同属「覆盖面未声明即结论不可引用」家族，见同批 #169）/ #132（跨臂门的分母语义 MUST 声明——同属「口径错位制造缺口假象」）。

---

### 发现 #169（中价值，简记）: 判据的判据——机械对账工具的**覆盖面自身**必须可核，差集数字不可脱离 coverage 引用（260918-01）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-01）；主会话（B6-1 抽取器两轮口径迭代）+ 本 subagent 起草；**draft**（两轮计数均为一次实测记录；「首版只扫单一载体」的定位见下，属过程记录）；workflow-patterns / 对账工具覆盖面声明（**#164「无脚本覆盖面」家族的数据面** + **#163 覆盖面维**）。
- **来源定位**：`.investigations/b61-260917-06/extension-per-version-gap.md` §2 表（1.20.1 声明 128 / 1.21.6 声明 106 / 缺口 11 与 21）+ `scripts/check_switch_mapping.py:33-51`（`GRADLE_FILES` 2 项 + `SRC_DIRS` 4 项 + `NOT_SCANNED` 4 条显式未扫面声明）+ `:141-151`（`--json` 导出的 `coverage` 段）+ `.investigations/b61-260917-06/switch-sources.json:2-17`（首轮 coverage：`declaration_carriers` 2 + `consumption_carriers` 4 + `not_scanned` 3 条）。
- **观察（现象）**：B6-1 抽取器的**覆盖面在一轮内发生过扩张**——首版只扫 `versions/1.20.1/java/src` 一个消费载体，得 DEAD 32 / ORPHAN 18；补全载体（+ 1.21.6 java src + java-core + worldgen-core）后变为 **DEAD 4 / ORPHAN 24**。（对照：脚本落地后经补映射与 per-version 口径，终态数据 = `switch-sources.json` M1 116 / M2 136 / DEAD 4 / ORPHAN 24 与 `switch-sources-postfix.json` M1 129 / M2 136 / DEAD 4 / ORPHAN 11。）
- **判据（可复用）**：**对账/扫描工具的差集数字是「覆盖面」的函数，不是「仓库事实」的函数**——故 ① 工具输出 MUST 随附 coverage 声明（扫描了哪些载体 + **明确未扫面**清单）；② 任何「DEAD N / ORPHAN N / 缺口 N」的引用 MUST 与该次运行的 coverage 同格出现，跨覆盖面引用数字无效（数字变了不代表仓库变了，可能只是扫描面变了）；③ 覆盖面扩张 = 判据重跑触发点（新增载体目录 → 同步 `GRADLE_FILES`/`SRC_DIRS` 后重跑），未扫载体一律不判 DEAD。
- **家族索引**：#164（多副本安装树的「无脚本覆盖面」——本条为其**数据面**：不只脚本要有覆盖面，工具产出的数字也要带覆盖面）/ #105（载体偏差）/ #163（判据层级——同属「结论强度受工具覆盖面约束」）。

---

### 发现 #170（中-高价值）: 发射面直证法——用 init script 在**发射点本体**取 JVM 参数，并以「组内标定对照」把缺席变成二值强判据（260918-01）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-17 ~ 09-18（260917-06 三哨兵 → 260918-01 sentinel-D）；主会话执行（judge M2 要求补标定对照）+ 本 subagent 起草；**draft → 建议 candidate**（观测面为发射点本体；**分层 = Partial 非 Full**——止于参数发射面，「补映射后端到端行为如预期」未验证）；workflow-patterns / 行为化验证方法（**#37「env 判别实验的生效证据必须行为化」家族的方法面** + **#81「A=B 恒等不证分支生效」家族的对偶面** + **#118 自证行硬门的正例**）。
- **来源定位**：`.investigations/b61-260917-06/phase25-verification.md` §1（方法）/§3（三哨兵原文）/§4（判别力结论）/§5（覆盖面与降级声明）；`.investigations/b61-260917-06/t5-landing.md` §2（补前缺席 → 补后发射的前后对照）+ §4.0（13/13 全实证，`cmd-output/sentinel-D-all13.log`）。
- **方法（为什么强，可复用）**：
  1. **观测面 = 发射点本体**：用 Gradle **init script**（`-I print_vmargs.gradle`）在 `runServer` 的 `doFirst` 打印 `t.jvmArgs` —— 源码（`build.gradle`）说的是「**应该**发射什么」，`jvmArgs` 是「**实际**发射了什么」；直接读源码属静态层，读 `doFirst` 上的 `jvmArgs` 才是**发射面直证**。
  2. **组内标定对照（本方法的关键增量）**：单点「未发射」观测**无法排除「仪器整体不响应」**（发射面根本没工作 / 脚本没挂上）。解法 = **同一次运行、同一开关族、三个 `-P` 一起传**：surfaceDump 族实测 `-Dsurfacedump.probe=1` + `-Dsurfacedump.chunkX=1` + `-Dsurfacedump.chunkZ=2` **发射了**，而 `-Dsurfacedump.dim` **缺席** ⇒ 同轮内的两条成功行构成**仪器有效性自证**，缺席因此升级为**二值强判据**（而非「可能是仪器没工作」）。
  3. **正向对照臂的第三条腿**：SCOPED 侧（heightProbe 族零 `-P` 先例）实测**仅发射父门** `-Dheight.probe=true`、零子参数 —— 与「族内零先例 ⇒ SCOPED」规则一致（判据的正/负两侧都有观测）。
  4. **覆盖面闭合**：13 项补映射经 **sentinel-D 一轮传齐全部 13 个 `-P`，实测发射 13/13 无缺席**，把「6/13 实证 + 7 项静态外推」闭合为**全实证**——外推与实证的分界 MUST 显式声明（本条闭合动作本身即该纪律的实例）。
- **判据（可复用）**：① 参数/开关类「生效」结论 MUST 在**发射点/消费点本体**取证，不得止于「源里写了」；② 「未观测到」类结论 MUST 配**同轮阳性对照**（组内标定），否则不是判据只是观测；③ 结论 MUST 声明分层（本方法 = Partial，止于发射面）与覆盖面（多少项实证、多少项外推）。配套：`doFirst` 早于 mod remap 失败点，故同批 `BUILD FAILED`（Chunky jar 文件锁）不影响 vmArgs 证据有效性——但**该关系 MUST 如实披露**（`phase25-verification.md` §2 首轮错误三处：哨兵 B 输出不存在 / A 的 FAILED 未披露 / 缺标定对照，均在 v2 修正，错误优先留痕）。
- **家族索引**：**#37（env 生效证据必须行为化）**/ #81（恒等不证生效——本条是「生效」侧的方法模板）/ #53（env 默认值当公理——同属「不读消费点就下结论」）/ #118（自证硬门——本条为「正/负成对观测」的参数面形态）/ #20（死参数制造假判别——发射面直证是死参数判定的最廉价入口）。

---

## 二、追加到 knowledge/discovered/build-tooling.md 末尾

### 发现 #155（错误优先，五段式）: `.gitignore` 整目录忽略吃掉「门禁资产」——被 AGENTS.md 明文的门禁在仓库中不存在；且 git 不支持在被排除目录内 re-include（260918-01）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-01）；主会话 B6-1 落地过程中发现（`t5-landing.md` §5.1）+ T5 处置 + 本 subagent 起草；**draft**（git 侧事实已一手核验：`git ls-files scripts/` 现含三脚本 / HEAD `b30d008` / 修复 commit `ab91a0e`；原「未入库」状态为发现时实测）；build-tooling / 仓库资产与 gate 资产存续（**#24（gitignore 目录级 prune 使 `!` 白名单失效）的同族第二形态** + workflow-patterns **#164（无脚本覆盖面）的数据面**）。
- **来源定位**：`.investigations/b61-260917-06/t5-landing.md` §5.1（发现与建议）+ §6（§9.8 副作用与逆表：`.gitignore` 改动 = in-place / 逆 = git 提交 `ab91a0e` 可 revert）；`.gitignore:49-58`（现行形态）；commit `cf77963`（发现记录）→ `ab91a0e`（修复 + 脚本入库）；一手核验 = `git ls-files scripts/` 返回 `check_switch_mapping.py` / `merge_index.py` / `scan_cpp_anchors.py` 三项跟踪。

- **五段式（错误优先）**：
  - **现象**：B6-1 交付的门禁 `scripts/check_switch_mapping.py` 在本机可跑，但仓库里**没有它**——`.gitignore:51` 曾有 `scripts/` 一行，整目录被忽略 ⇒ **AGENTS.md §一.5/§一.13 明文引用的门禁脚本**（`scan_cpp_anchors.py` / `check_switch_mapping.py`）与 `merge_index.py` **全部未入版本管理**；其存续只依赖单机磁盘状态，换机/清理即失，CI/新环境无法执行 AGENTS 明文要求的「改动后跑门禁」。
  - **根因（机制）**：**「目录级 ignore」与「白名单 re-include」在 git 语义下不兼容**——git 的 `!` 规则**无法重新纳入一个被排除目录内部的路径**（目录本身被 prune ⇒ 内部任何 `!` 永不生效）。因此「想忽略目录里的大部分、只放行几个门禁」这个**直觉写法**（`scripts/` + `!scripts/xxx.py`）**静默失效**：`!` 行看着在、其实恒不生效，且**无任何提示**。更深一层：仓库把「脚本」整体当作「本地工具」（与 `runtime/`、`NEXT_SESSION.md`/`AGENTS.md` 同类，见 `.gitignore:49` 注释的原始口径），但**其中三个是 AGENTS 明文引用的门禁资产**——「本地工具」与「门禁资产」被同一条规则吞掉，资产随规则一起消失。
  - **定位（怎么发现的，可复用）**：① **交付物交叉核对**——把「本块新建/引用的产物清单」与「`git ls-files` 实际跟踪清单」对表，差集即未入库资产（本块 `t5-landing.md` §5 落地产物清单 vs `git ls-files scripts/`）；② **读 `.gitignore` 定位规则行**；③ **核 git 语义**（「被排除目录内能否 re-include」= 否），避免用直觉写法「修」；④ **修复后双向核验**——`git ls-files scripts/` 应含三脚本且**只含**这三项。
  - **修复**：`.gitignore` 由 `scripts/`（整目录）改为 **`scripts/*`（目录放行 + 逐文件忽略）+ 三条 `!` 白名单**（`!scripts/check_switch_mapping.py` / `!scripts/merge_index.py` / `!scripts/scan_cpp_anchors.py`），并就地写注释说明「`scripts/` 不能整目录 ignore，否则无法用 `!` 白名单重新纳入」；三个门禁脚本入库。commit = `ab91a0e`（**注意**：`protocol/` 仍整目录忽略，口径未动）。
  - **教训（判据，可复用）**：
    1. **判据（MUST）**：**被判据/指令/文档明文引用的资产（门禁脚本、schema、规范副本）MUST 在版本控制内**——「文档引用了它」与「仓库里有它」必须机械核对（核对动作 = 把引用物清单对 `git ls-files`/`git check-ignore -v` 双向核验）。引用了不存在的产物 = 新环境下的静默能力缺失（与 #137「引用外部锚须属主工具自核」同构：引用完整性是独立义务）。
    2. **判据（MUST）**：**git 不支持在被排除的目录内 re-include**——凡需要「忽略目录内大部分、放行少数」，MUST 写 `dir/*`（放行目录、逐文件忽略）再配 `!dir/file`；写成 `dir/` + `!dir/file` 是**静默失效**形态（`.gitignore` 不报错），且该形态与 **#24**（目录级 prune 使白名单失效）是同一坑的两个版本。
    3. **修复 MUST 双向核验**：修完不只看「期望的在里面」，还要看「不该在的没进去」（本例 `git ls-files scripts/` 恰为三项）。
    4. **处置边界（诚实）**：本块**只记录不擅自改**到 T5 才处置——因为「哪些脚本是门禁资产、哪些是本地工具」是**口径决策**（可能刻意设计）；口径决策的产物化动作 = 在 AGENTS.md 写明「脚本目录的口径 + 哪些必须入库」，否则下一次仍靠人肉发现。
- **家族索引**：**#24（gitignore 目录级 prune 使 `!` 白名单失效——同族第二形态）**；workflow-patterns **#164（多副本安装树的「无脚本覆盖面」——本条为其数据面）**、#163（silently-green 门——门禁本身不入库是更强的静默形态）、#137（引用完整性）、#105（载体偏差——「本机有」≠「仓库有」）。
