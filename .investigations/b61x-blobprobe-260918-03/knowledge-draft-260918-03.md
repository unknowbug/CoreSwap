# knowledge-draft-260918-03（subagent 草稿，主会话应用后生效）

> 来源工作块：260918-03 BlobProbeMixin stats 读取修复（candidate，judge PASS-with-conditions M-1 已闭合；confirmed 留用户）。
> 主记录：`.investigations/b61x-blobprobe-260918-03/record-260918-03.md`；结果镜像 `.artifacts/blobprobe-260918-03/verdict-260918-03.md`；judge 意见 `review-260918-03-001.md`。
> 编号建议基于现库尾：workflow-patterns max=#175、build-tooling max=#156（应用时如库已前移须顺延重排，先重读目标文件尾）。

## 归属建议总表

| 条目 | 建议发现编号 | 归属文件 | 一句话理由 |
|---|---|---|---|
| mixin 类本体不可经反射自载——stats/计数读取类诊断必须把可观测状态放普通类 | workflow-patterns **#176** | knowledge/discovered/workflow-patterns.md | 这是「探针/诊断设计模式」层的通用教训（可观测状态放哪），跨版本/跨项目可复用，载体应是通用模式库 |
| WRITTEN 通路「0==0 恒等对照 PASS」被判据问题（M-1）——重构后的写通路必须有过滤器相交的正例轮 | workflow-patterns **#177** | knowledge/discovered/workflow-patterns.md | 属判据有效性/验证纪律层（#110/#129 家族），非工具坑；与既有判据家族同源，合族索引可检索 |
| mixin 自载失败运行时签名与 #40 家族区分判据 | build-tooling **#157** | knowledge/discovered/build-tooling.md | #40（mixin json 失同步）就在 build-tooling，本条是其直接姊妹形态（同为 mixin 装载面失败、判据在时点区分），放同文件便于合读 |

**不写项（价值门裁决）**：本块的修复实现细节（BlobProbeStats.java 具体 diff、死字段清理 S-4、假阴性删 world 复跑）为一次性工程记录，不复用价值，只进时间线块，不进 discovered；compiler-idioms 不收（本条无语言惯用法语义，是运行时装载面/工具链问题，非编译期语义）。

---

## 条目一（workflow-patterns #176 全文）

## 发现 #176（最高价值·错误优先）: mixin 类本体不可经反射自载——stats/计数读取类诊断必须把可观测状态放普通类，禁止 `Class.forName` 读 mixin 类内部（260918-03）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-03）；CoreSwap 主会话（排查 + judge 闭合）+ 本 knowledge subagent 起草；**candidate**（Full 分层：原始日志重采样 + 17 份跨树日志普查 + 修复后实机正/负成对自证 ×2 轮；judge PASS-with-conditions M-1 已闭合，confirmed 留用户）；workflow-patterns / mixin 诊断设计与可观测状态放置。
- **来源定位**：`.investigations/b61x-blobprobe-260918-03/record-260918-03.md`（主记录，含候选裁决表）；`.artifacts/blobprobe-260918-03/verdict-260918-03.md`（结果镜像）；修复前基线 `.investigations/b62-260918-02/cmd-output/sentinel-1216-all12.log:137-143`；跨树普查 `.tmp/blob-probe/*.log`（17 份 1.20.1 树日志，grep `mixinCalls|stats read failed` 命中 16 文件 32 行全部失败）。

- **五段式（错误优先）**：
  - **现象**：1.21.6 runServer 日志 `[BLOB-PROBE] stats read failed: RuntimeException: Mixin transformation of wg.bench.mixin.BlobProbeMixin failed`（sentinel-1216-all12.log:142，12:56:40）；同轮 ：137-139 `[BLOB-PROBE] start / handler active / dimSeen` 正常（12:56:07）；`mixinCalls=-1 written=-1`（哨兵初值恒定）。表象像「mixin 织入失败/版本不兼容」。
  - **根因（机制）**：`BlobProbe.java:39,42` 经 `Class.forName("wg.bench.mixin.BlobProbeMixin").getDeclaredField("CALLS"/"WRITTEN")` 反射读取计数。**mixin 织入目标类不需要加载 mixin 类本体**——所以注入/采集前半段全部正常；stats 读取点是全 run **唯一一次加载 mixin 类本体**的时点，Knot/Sponge transformer 对这次**自载**尝试变换失败抛 RuntimeException。即：**不是织入失败，是反射自载触发 transformer 自变换失败**；也非 1.21.6 问题——1.20.1 树自 09-02 首跑起 17 份日志同样失败（v61+JAVA_17 匹配仍失败，候选 B「字节码 v65/compatibilityLevel 失配」被证伪）。附带缺陷：`BlobProbe.java:46` 只打印顶层消息，底层 cause 被吞（transformer 拒绝自变换的更深层机制 open，不阻塞处置）。
  - **定位（怎么发现的，可复用）**：① 时点核对——失败行时间戳（12:56:40）与注入日志（12:56:07）分离，失败只在 stats 读取时点而非 boot，排除 boot 期装载问题；② 源码 grep 找到 Class.forName 反射路径，识别「全 run 唯一 mixin 本体加载时点」；③ **跨树普查**（17 份 1.20.1 树历史日志全命中同一失败 + `mixinCalls=-1` 恒为哨兵）一步证伪版本回归假设——跨版本历史日志普查是「版本问题 vs 设计灰区」最便宜的判别臂。
  - **修复**：计数器外移普通 holder 类 `wg/bench/BlobProbeStats.java`（volatile int，普通类加载无 transformer 介入）；`BlobProbeMixin.java` 改累加 BlobProbeStats 并删 @Unique 字段；`BlobProbe.java` 免反射直读，删 Class.forName 路径。验证（Full）：正/负成对——修复前 sentinel 日志必现 stats_fail；修复后负例轮 stats_fail=0 + mixinCalls=7147（CALLS 通路直证）；judge M-1 闭合正例轮 written=1152 csv=1152（WRITTEN/csv 通路正例覆盖，见 #177）。handler active 在位（织入面未受损）。
  - **教训（判据，MUST）**：
    1. **mixin 类本体不可经反射自载**——`Class.forName("<Mixin类>")` 是全 run 唯一触发 transformer 对 mixin 本体自身变换的时点，失败签名 = 注入正常 + stats/读取时点失败 + 计数恒为哨兵初值。
    2. **stats/计数读取类诊断的可观测状态 MUST 放普通类**（holder/伴生类），mixin 类只留织入逻辑；读取侧免反射直读。设计期规则，不是失败后补丁。
    3. **「织入失败」与「stats 读取失败」的区分判据 = 失败时点**：注入日志在位（织入成功）+ 失败仅在读取时点 ⇒ 排除织入面；与 #40 的区分见 build-tooling #157。
    4. **探针异常不得吞 cause**——只打印顶层 `+ t` 使本例底层机制未开箱即登记 open；诊断代码 MUST 打完整异常链。
    5. **反模式**：把「计数恒为哨兵初值」读成「计数通路失效/探针整体不可用」——本例 blob origins.csv 等注入面数据大概率有效，失效面仅 stats 读取；结论范围 MUST 按失效面收窄（本次 NEXT_SESSION 原措辞即按 §15.4 取代记录收窄）。
- **家族索引**：build-tooling **#40**（mixin json 失同步——同为 mixin 装载面失败，时点判据区分，见该条）；**#55**（mixin 反射字符串不被 remapper 重写——「反射读 mixin 内部」同域的另一个坑，两条合读 = mixin 内部状态禁止外部反射访问的两形态）；workflow-patterns #54（mixin 内门坐标——mixin 诊断面家族）。

---

## 条目二（workflow-patterns #177 全文）

## 发现 #177（高价值·判据延伸）: 恒等对照（0==0）≠正例覆盖——重构后的写通路必须有过滤器相交的正例轮，诚实声明不能替代判据成立（260918-03，#110/#129 家族）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-03，judge M-1 条件闭合过程沉淀）；CoreSwap 主会话 + 本 knowledge subagent 起草；**candidate**（正例轮已实跑闭合：written=1152 csv=1152，Q1-Q4 全过）；workflow-patterns / 验证判据设计与正例覆盖纪律。
- **来源定位**：`.investigations/b61x-blobprobe-260918-03/review-260918-03-001.md` §M-1（judge 抓出零正例覆盖全过程）；`record-260918-03.md` §judge 复审闭合（正例轮参数与结果）；`.artifacts/blobprobe-260918-03/verdict-260918-03.md` §验证判据（负例基线 + 两轮读数）。

- **五段式（错误优先）**：
  - **现象**：WRITTEN 通路重构后首轮验证 P4 = 「origins csv 行数 == written」，实测 0==0 判 PASS；`mixinCalls=7147` 只直证 CALLS 通路——若 `BlobProbeStats.WRITTEN++` 或 csv 写路径在重构中被破坏，P1-P4 全部仍 PASS（judge 复盘原话级结论）。
  - **根因（机制）**：**恒等对照的两侧都在「空集」上比较**——过滤器 `blobProbe.chunkX=11` 与 region(200,200) 不相交，written 天然为 0，`0==0` 对写通路的任何破坏都不敏感。写通路（计数 + 落盘）的有效性只有「非空产出」能证明：判据读数恒为初值/空集时，比较成立 ≠ 通路工作。机制上这是 #129「判据只有满足/未满足两种终态」在**正例覆盖维**的形态：判据文本执行了，但判据的**判别力**对被验证目标为零——零判别力的判据 PASS 不构成证据。另附 M-1 的第二半：P4 预登记文本（文件**存在**）与实际执行（不存在按 0 处理）被静默放宽——判据预登记→执行之间不得弱化，违反可失败性纪律。
  - **定位（怎么发现的，可复用）**：judge 三源核对时直查 `Test-Path origins-1216.csv = False`，把「0==0 PASS」还原为「文件根本不存在」；record 原本已诚实声明「恒等对照非正例覆盖」——**诚实声明让问题可见，但声明本身不闭合问题**，judge 据此立 M-1 强制二选一（补正例轮 or 收窄结论）。
  - **修复**：补正例轮（run-1216-positive.ps1，过滤器改 `chunkX=200` 与 region **相交**）：mixinCalls=7147 **written=1152 csv=1152**，stats_fail=0，handler_active=1——WRITTEN/csv 通路获非空正例覆盖，M-1 闭合。
  - **教训（判据，MUST）**：
    1. **恒等对照（0==0）≠ 正例覆盖**——恒等对照只能证「两端一致」，不能证「通路产出过东西」；重构/新增的**写通路**（计数→落盘）MUST 至少一次「过滤器与采样区相交」的正例轮，验 written>0 且 落盘行数==计数。
    2. **判据预登记文本与实际执行 MUST 逐字一致**——执行时静默放宽（存在→不存在按 0 处理）违反判据可失败性；执行不了预登记文本时 MUST 回改判据文本并收窄结论范围，不得按弱化口径判 PASS。
    3. **诚实声明 ≠ 判据成立**——record 诚实标注「非正例覆盖」值得肯定，但候选结论在缺口处置前不得提请 confirmed；正确的下一步是补判据（正例轮），不是靠声明降级通过。
    4. **正例轮设计要点**：只改过滤器参数即得正例（chunkX=200 与 region 相交），零新仪器成本——设计判据时就应把「相交参数的正例臂」排进验证计划，而不是等 judge 抓。
- **家族索引**：**#110**（机制断言证明三件套——空集/不可达断言的证明义务，本条为其「正例覆盖」维）；**#129**（判据只有满足/未满足两种终态——本条为其「零判别力判据」形态）；#130（行为门前置判据：该维须先有载体行——正例轮即「先造载体行」）；#27（负向测试是断言生效的唯一证明——本条为对偶面：写通路的正例同样是唯一证明）；#132（分母语义——本例 written 口径 = 过滤后落盘行数，须同行声明）。

---

## 条目三（build-tooling #157 全文）

### 发现 #157: mixin 变换失败的两家族时点判据——boot 即失败 = 装载面问题（#40 家族），注入正常 + 读取时点失败 = 反射自载 mixin 本体（260918-03）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-03）；CoreSwap 主会话 + 本 knowledge subagent 起草；**candidate**（同 workflow-patterns #176 的验证面）；build-tooling / mixin 运行时失败签名分型。
- **来源定位**：`.investigations/b61x-blobprobe-260918-03/record-260918-03.md` §候选裁决表（候选 C「#40 json 失同步」证伪过程）；260918-02 哨兵日志 `.investigations/b62-260918-02/cmd-output/sentinel-1216-all12.log:142`（本家族第一现场，当时被归入 #40 家族，后修正）。
- **观察/证据**：两种「Mixin transformation failed」同字符串不同机制——
  - **#40 家族（装载面）**：mixin 配置 json 与 mixin 类失同步，**boot 期即失败**，织入未发生，目标方法无注入痕迹；
  - **本家族（自载面）**：注入/织入全程正常（`handler active` 等 BLOB-PROBE 注入日志在位），失败出现在**运行时 `Class.forName` 反射读取 mixin 类本体的时点**（stats 读取点），且 `mixinCalls` 恒为哨兵初值；跨版本普遍（1.20.1/1.21.6 两棵树同样失败），与字节码版本/compatibilityLevel 无关。
- **如何利用（区分判据）**：见到 `Mixin transformation of <mixin类> failed` 时第一动作 = **核对失败时点与注入日志**——boot 即失败查 json 注册/条目同步（#40 判据）；注入正常、失败在某个读取时点 → grep 源码找 `Class.forName("<mixin类>")` 反射路径，判定为自载失败（处置 = 可观测状态外移普通类，见 workflow-patterns #176），**不要立版本兼容课题、不要重查 json**。时点判据先于任何静态审查。
- **家族索引**：**#40**（json 失同步——同为该异常字符串的装载面形态，本条为其分型对照）；build-tooling **#25 补充案例**（mixin AP 警告 = APPLY FAILED 前兆——编译/装载期形态，与本条运行时形态互补）；workflow-patterns **#176**（机制与修复的通用模式面——本条为其工具链签名面）。

---

## 时间线块草稿（versions/1.21.6/docs/10-timewise-archive.md 末尾追加）

## 260918-03（实际 2026-09-18，Get-Date 锚开工时点）：BlobProbeMixin「mixin transformation failed」根因定位 + stats 修复 + judge M-1 正例轮闭合—— ✅ judge PASS-with-conditions（M-1 已闭合）；confirmed 待用户授予

> 过程产物 `.investigations/b61x-blobprobe-260918-03/`（`record-260918-03.md` 主记录 + `scout-map.md` + `review-260918-03-001.md` judge 三源核对 + `knowledge-draft-260918-03.md`）+ `.artifacts/blobprobe-260918-03/verdict-260918-03.md`（结果镜像，S-1 已登记根 index）。上游：260918-02 哨兵日志的附带观察（stats read failed，当时误归 #40 家族）。通用模式 → workflow-patterns **#176/#177** + build-tooling **#157**（subagent 草稿 → 主会话应用）。

- ✅ **核心结论（candidate）**：`Mixin transformation of wg.bench.mixin.BlobProbeMixin failed` = **反射自载 mixin 类本体触发 transformer 自变换失败**（`Class.forName` 读 CALLS/WRITTEN 计数是全 run 唯一 mixin 本体加载时点）——非织入失败、非 1.21.6 问题（1.20.1 树 17 份日志自 09-02 起全部同样失败，`mixinCalls=-1` 恒为哨兵初值）；修复 = 计数器外移普通类 `BlobProbeStats`，免反射直读（仅 1.21.6 树）。
- ❌ **被证伪候选**：B 字节码 v65/compatibilityLevel 失配（1.20.1 同样失败）；C #40 json 失同步（注入已生效 handler active）；D #12 static nested（无嵌套类）。
- ✅ **验证（Full，实机正/负成对）**：负例基线 sentinel-1216-all12.log:142 必现；修复后 stats_fail=0 + mixinCalls=7147；**judge M-1 闭合正例轮**（chunkX=200 与 region 相交）：written=1152 csv=1152，Q1-Q4 全过。open 保留：transformer 拒绝自变换的底层 cause（打印层吞了 cause，未开箱，不阻塞）。
- ❌→修正 **M-1 教训（错误优先）**：WRITTEN 通路首轮「0==0 恒等对照 PASS」被 judge 抓出零正例覆盖 + P4 预登记文本被静默放宽——诚实声明 ≠ 判据成立，补相交正例轮闭合（→ #177）。
- ⚠️ **诚实边界**：judge 无 git 通道，diff 以主会话转录 + 文件现状交叉核对；普查为 grep 全量但 judge 仅抽查 1/17（S-5，record 已注明口径）。
- 状态：✅ judge PASS-with-conditions（M-1 闭合、S1-S5 已应用）；**confirmed 待用户授予**。
