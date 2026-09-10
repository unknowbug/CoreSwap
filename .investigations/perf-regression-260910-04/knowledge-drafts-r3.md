# 260910-04 · R3 修复验证 知识库增量草稿（subagent 只读产出，未改任何现网文件）

> 产出者：知识库 subagent（只读 + 只产草稿）。
> 已读顺序：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（项目级规范，优先级最高）→ `knowledge/discovered/workflow-patterns.md` 发现 #107（1727-1741 行）+ #108/#109（1742-1762 行）→ `.artifacts/perf-regression-260910-04/verdict-260910-04.md`（candidate）→ `.tmp/perf-reg-260910-04/results.txt`（含 `r3-r1` / `r3sync-r1` 两行）→ `.investigations/perf-regression-260910-04/cmd-output/r3-async-r1.log`（异步臂完整日志，`[CHUNKTIME]` 末行 147 + Chunky Total 148）。
> **交叉核对补读**（为满足「数字可核对」）：`.tmp/perf-reg-260910-04/logs/r3sync-r1.log`（同步臂原始日志，末行 165 / Total 167）、`.tmp/perf-reg-260910-04/run_arms2.ps1`（两臂参数定义，25-26 行）、`runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`（57-58 / 60-61 / 145-149 行）、`runtime/1.21.6/java/build.gradle`（134-135 行）、`runtime/1.21.6/java/src/main/java/wg/bench/ChunkTiming.java`（27-40 行）、`.tmp/hang-repro-260910/diff_arms.py` + `diff-result.txt`（74 行历史基线）、`.investigations/perf-regression-260910-04/cmd-output/MANIFEST-sha256.txt`。
> 价值门结论：**高价值（必记）**——「接管类优化把重活搬出被串行化执行点」的**修复验证判据三件套**属可复用判据；本块的逐臂读数只作证据，不单独进知识库。
> **本文件只是草稿**：不修改 `knowledge/`、`docs/`、`.artifacts/`、`NEXT_SESSION.md` 任何现网文件；应用与验证由主会话执行。

---

## 应用指引（主会话）

- **目标文件**：`knowledge/discovered/workflow-patterns.md`。
- **插入位置**：**紧随发现 #107 块之后**（现网 = 1739 行 `家族索引` 行之后、1742 行 `## 发现 #108` 之前）——本条是 #107 的**修复验证追加**，贴在其定责主条之后读者才能连读「机制 → 验证」；若主会话偏好按时间戳追加到文件末尾（#109 之后），内容零改动，仅位置不同。
- **编号说明**：沿用 **#107 家族**（不新增顶层编号，标题即「发现 #107 修复验证（260910-04 追加）」）；若主会话坚持给独立编号，按现网递增应为 **#110**（现网已用到 #109 + #83/#51 家族补充）。
- **末尾 INDEX 一行**：见文末「INDEX.md 追加建议」。

---

## 增量段（可直接粘贴）

```markdown
### 发现 #107 修复验证（260910-04 追加）：异步化把并行度从 1 恢复到 23

- **时间 / 发现者 / 置信度 / module**：260910-04（实际 2026-09-10；异步臂 `r3-r1` 日志时间戳 18:13，同步臂 `r3sync-r1` 18:15-18:20，背靠背同批）；主会话（实测，同构建态单变量 A/B + 并发度直读 + 逐块对拍门）；**candidate（同批 A/B + 直证；confirmed 留人类）**；workflow-patterns / 接管形态修复验证（#107 的修复侧）。
- **来源定位**：
  - 修复点 = `runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:57-58`（`WG_FILL_POOL = Util.getMainWorkerExecutor().named("coreswap_fill_noise")`）+ `:145-149`（同步/异步二选一：`SYNCFILL ? completedFuture(work.get()) : supplyAsync(work, WG_FILL_POOL)`）；A/B 开关映射 `runtime/1.21.6/java/build.gradle:134-135`（`-Psyncfill=1` → `-Dcoreswap.syncfill=1`，默认异步）；并发度仪器 `runtime/1.21.6/java/src/main/java/wg/bench/ChunkTiming.java:27-40`（`inflightEnter/Exit` 原子计数）。
  - 机器可读逐臂记录 = `.tmp/perf-reg-260910-04/results.txt` 第 10-12 行（`r3-r1` / `r3sync-r1` / `vanilla1216-r1`）。
  - 异步臂完整日志 = `.investigations/perf-regression-260910-04/cmd-output/r3-async-r1.log`（第 147 行 `[CHUNKTIME] n=4608` 末行；第 148 行 Chunky `Total time: 0:00:42`；第 139 行 `Rate: 102.7 cps`）。
  - 同步臂完整日志 = `.tmp/perf-reg-260910-04/logs/r3sync-r1.log`（第 165 行 `[CHUNKTIME] n=4608` 末行；第 167 行 `Total time: 0:04:39`）——⚠️ **未归档到 `.investigations/`**（见诚实边界 3）。
  - 逐块对拍工具 = `.tmp/hang-repro-260910/diff_arms.py`；三臂 region 归档 = `.investigations/perf-regression-260910-04/cmd-output/region-{r3,r3sync,vanilla-260910-04}/`；历史双臂基线原文 = `.tmp/hang-repro-260910/diff-result.txt:74`。
  - 上游定责 = `.artifacts/perf-regression-260910-04/verdict-260910-04.md` §3 R3（candidate，待用户 confirmed）。
- **数据（同构建态 A/B，带 run 标识）**：

| 臂 | run id | 接管形态 | Chunky Total | wallgen | serverCpu | 派生核数 | `inflight max` | 每 chunk 线程周期（armAgnostic featInterval，末次报告） | 每 chunk 接管段（mixin） |
|---|---|---|---|---|---|---|---|---|---|
| **r3** | `r3-r1` | **异步**（提交 `coreswap_fill_noise` 池，默认） | **0:00:42（42s）** | 50.1s | 543 | **10.8 核** | **23** | **242.52ms**（n=4411） | 103.47ms〔jni 96.81 / write 6.19 / hmap 0.42 / beard 0.03〕 |
| **r3sync** | `r3sync-r1` | **同步**（`-Dcoreswap.syncfill=1`，同 dll sha `abd7d889…`） | **0:04:39（279s）** | 280.3s | 375 | **1.34 核** | **1** | **1413.65ms**（n=4416） | 53.87ms〔jni 50.19 / write 3.38 / hmap 0.21 / beard 0.02〕 |
| vanilla | `vanilla1216-r1` | 原版（`wgen_fill_noise`） | **0:00:52（52s）** | 60.1s | 316 | 5.26 核 | （无仪器） | （无仪器） | （无仪器） |

  - 派生核数 = `serverCpu / wallgen`（同表两列相除：543/50.1 = 10.84；375/280.3 = 1.34；316/60.1 = 5.26）；三臂同 seed `417950215108767439`、同 region **4225 chunks**（逐臂 `Processed: 4225 chunks` 核对一致）、同 dll sha `abd7d8893d22e030`。
  - **唯一变量 = 同步/异步形态**：**279s / 42s = 6.64×**；R3 后 coreswap 反超 vanilla：**52s / 42s = 1.24×**；并发度直读 **inflight max 23 vs 1**；每 chunk 线程周期 **1413.65 / 242.52 = 5.83×**（与 wall 比 6.64× 同阶自洽）。
  - **同一 Σ/wall 判据（#107/#108）在修复前后翻转**：`r3sync` 末次报告 Σ = sumMixin 248.3 + sumCarve 6.0 + sumFeat 12.2 = **266.5s / wallgen 280.3s = 95.1%**（≈ #107 定责时的 92.2%，即开关确实复现了「车道饱和」旧形态）；`r3` Σ = 476.8 + 7.4 + 15.8 = **500.0s / wallgen 50.1s = 9.98**（≈ 派生核数 10.8）⇒ 同一把尺子从 ≈1 翻到 ≈10-23 量级**（Σ/wall 是核数当量；线程数直读仍是 `inflight max`）。同线程间隔 `gap` 亦从 1224.00ms（`r3sync` 末行）降到 94.58ms（`r3` 末行）。
  - 口径备注：末次报告 n=4411/4416 > region 4225 chunks（+4.4%/+4.5%）——接管计含区域边界外邻 chunk 重生成，属既有语义；sumMixin 是**线程时间之和**（池内多线程累加），不是 wall，勿与 wall 混用。
- **行为等价门结果（全域逐块普查，非零容忍）**：三臂 region 全量逐 chunk 逐 section 对拍（工具 `diff_arms.py`，分母 = 共同 chunk 内「两侧 section 并集」的方块数，故每对分母随 section 存在性微移；`375,554,048 − 375,549,952 = 4,096 = 恰 1 个 section`）：

| 对拍对 | 差异方块 | 分母 | 占比 |
|---|---|---|---|
| r3 vs vanilla | 44,921 | 375,554,048 | **0.0120%** |
| r3 vs r3sync | 45,948 | 375,554,048 | **0.0122%** |
| r3sync vs vanilla | 41,809 | 375,554,048 | **0.0111%** |
| 历史双臂基线（260910-04 前，同作业线上） | 56,214 | 375,549,952 | **0.0150%** |

  ⇒ 三对差值 **0.0111-0.0122% 与历史基线 0.0150% 同量级且不高于它**：R3 **未引入超出既有 run 级非确定的额外内容差**（管线本身有 run 级非确定，历史双臂即非零）。⇒ 门是**同量级门**，不是零容忍门。
- **推广判据（MUST，可复用）——「接管类优化 = 把重活从被串行化的执行点搬到工作池」的验证三件套**：
  1. **① 同构建态单变量 A/B（形态开关，背靠背）**：把旧形态**保留为开关**而不是删掉（本案 `-Dcoreswap.syncfill=1`），同 dll sha / 同 seed / 同 region / 背靠背各 1 run ⇒ 差异唯一变量 = 同步/异步形态。禁止用「改前历史数字 vs 改后数字」代替（跨批 ±20% 摆动带会淹没或伪造结论，#103/#108）。
  2. **② 并发度直读（in-flight 计数：1 → N）**：在被搬动的重活**进出点各加一次原子计数**（本案 `ChunkTiming.inflightEnter/Exit`，随 `-Dcoreswap.chunktime` 门控），直接读出「同时刻在飞数」与历史峰值。判据 = 旧形态**恒为 1**（结构证明）、新形态达到池并发数（本案 **23** = 该池并发线程数）。这是「搬成功了」的**直接证据**，不要只靠 wall 推断。
  3. **③ 行为等价门（逐块对拍，与既有 run 级噪声基线同量级即可，非零容忍）**：逐 chunk 逐 section 全量对拍，判据 = 新形态差值 **≤ 既有 run 级非确定基线**（本案三对 0.0111-0.0122% ≤ 0.0150%）。**零容忍是错的门**——本管线有 run 级非确定（历史双臂同配置即 0.015%），零容忍会产生假阴性并逼出无意义的「逐位对齐」轮次。参考量级：介于噪声基线与基线 1.5× 之间需查；超出量级即拒收。
  - **附加结构证据**：搬动成功后，**同一 Σ(被搬出执行点内工作)/wall 判据应翻转**（本案 0.95 → 9.98）；**arm-agnostic 同尺复核**（#109 的 featInterval 1413.65 → 242.52ms = 5.83×）与 wall 比同阶 ⇒ 三路（wall / 直读并发 / 同尺周期）互证。
- **陷阱（MUST）**：**重活进池后「每 chunk 段内延迟」会上升，不得读成回归**——本案接管段 mixin 53.87 → **103.47ms**（jni 50.19 → 96.81ms），因为同池 23 并发争用；而 wall 反降 6.64×。判据只看 wall / 吞吐，段内数字此时测的是「池内争用下的段延迟」，与单车道时的段延迟**不同物**（同族：吞吐均值 vs 每 chunk 延迟分离铁律）。
- **诚实边界（缺口，不夸大）**：
  1. **同配置 run-to-run 基线对本块未采集**：只有「同构建态同步 1 run vs 异步 1 run」这一对 A/B；按本项目跨 run ±20% 摆动带（#108：234s vs 283s），**6.64× 远超摆动带 ⇒ 方向可信，但幅度是单对读数**；要量级置信区间 SHOULD 补同批 ABBA ≥3 对。
  2. 三臂逐块对拍数字（44,921 / 45,948 / 41,809）**目前未落盘**为任何 diff 输出文件（`.investigations/`、`.tmp/` 全树检索零命中）——门**可复现**（region 三臂归档 + `diff_arms.py`），但数字本身只能靠重跑复核；MUST 补跑并落盘 diff 输出。
  3. 归档链对 R3 臂**滞后**：同步臂原始日志只在 `.tmp/perf-reg-260910-04/logs/r3sync-r1.log`（未进 `cmd-output/`）；`cmd-output/results.txt` 仍是 R3 前的 9 行版（R3 两行只在 `.tmp` 的 12 行版里）；`MANIFEST-sha256.txt` 生成于 18:09，早于 R3 跑批（18:13/18:15），故 R3 臂日志与结果行未纳入清单。
  4. **worldgen 车道自身的 busy/inFlight 计数仍未做**（#107 判据 4 / b1 @idk）：本块 `inflight` 计的是**接管段在飞数（池侧）**，不是车道占用；「车道已不再是限流点」由 wall（42s ≤ vanilla 52s）与并发 23 推断，仍属推断（上游限流线未排除）。
  5. vanilla 对照臂 `vanilla1216-r1`（52s）**未开** `-Dcoreswap.chunktime`；仪器化对照是 `vt1-r1`（53s，serverCpu 296 → 4.93 核）。跨臂每 chunk 段内数字**不可**与 coreswap 臂直接比（仪器不对称）。
  6. **数字差异标注**：同步形态「每 chunk 线程时间 1325ms」出自 R3 **之前**的 ct1 臂（#109 / verdict §2.3）；本块同步臂**自身日志末值 = 1413.65ms**（+6.7%，同量级）。本条的 A/B 一律引用后者（同 run 同批自洽），1325ms 保留为历史读数，两者不混用。
- **家族索引**：#107（主条——本条为其修复验证）、#108（Σ/wall 判据使用要点——本条给出判据在同一课题上的**翻转**形态）、#109（臂无关尺子——本条补同 build A/B 对）、#103/#51/#18（噪声带 / 跨 run 绝对值不可引）、#83（性能分母 / 结构场景）、#100/#102（归因须在真实通路上核对多面）。
```

---

## INDEX.md 追加建议（一行）

> 260910-04 R3 追加：**#107 修复验证**——接管段异步化（提交 `Util.getMainWorkerExecutor().named("coreswap_fill_noise")`，对齐 vanilla `wgen_fill_noise` 形态）把并行度从 **1 恢复到 23**：同构建态单变量 A/B（`-Dcoreswap.syncfill=1`，同 dll sha/seed/region 背靠背）**42s vs 279s = 6.64×**、`inflight max` **23 vs 1**、serverCpu/wallgen **10.8 核 vs 1.34 核**、Σ/wall 判据同尺翻转 **0.95 → 9.98**、逐块对拍 **0.0111-0.0122% ≤ 既有 run 级基线 0.0150%**（非零容忍门）；可复用判据 = 「接管类优化 = 把重活从被串行化的执行点搬到工作池」**验证三件套**：① 同构建态单变量 A/B（旧形态留开关）② 并发度直读（in-flight 1→N）③ 行为等价门（与既有 run 级噪声基线同量级即可）；**candidate**（同批 A/B + 直证，confirmed 留人类），来源 `.investigations/perf-regression-260910-04/knowledge-drafts-r3.md` + `cmd-output/r3-async-r1.log`。

---

## 数字核对表（主会话应用前逐条核对用）

| 数字 | 值 | 出处（文件:行） |
|---|---|---|
| r3 Total / wallgen / serverCpu / dllsha | 0:00:42 / 50.1 / 543 / `abd7d8893d22e030` | `.tmp/perf-reg-260910-04/results.txt:10` |
| r3sync Total / wallgen / serverCpu / dllsha | 0:04:39 / 280.3 / 375 / `abd7d8893d22e030` | `.tmp/perf-reg-260910-04/results.txt:11` |
| vanilla1216 Total / wallgen / serverCpu | 0:00:52 / 60.1 / 316 | `.tmp/perf-reg-260910-04/results.txt:12` |
| 6.64× / 1.24× / 1.34 核 / 10.8 核 | 279/42、52/42、375/280.3、543/50.1 | 上三行派生（同表两列相除） |
| r3 `inflight max=23`、featInterval 242.52ms(n=4411)、mixin 103.47ms | — | `cmd-output/r3-async-r1.log:147` |
| r3 Chunky `Total time: 0:00:42` / `Processed: 4225` / Rate 102.7 cps | — | `cmd-output/r3-async-r1.log:148` / `:139` |
| r3 Σ = 476.8+7.4+15.8 = 500.0s → /50.1 = 9.98 | — | `cmd-output/r3-async-r1.log:147`（三列同日导出） |
| r3sync `inflight max=1`、featInterval 1413.65ms(n=4416)、mixin 53.87ms | — | `.tmp/perf-reg-260910-04/logs/r3sync-r1.log:165` |
| r3sync `Total time: 0:04:39` | — | `.tmp/perf-reg-260910-04/logs/r3sync-r1.log:167` |
| r3sync Σ = 248.3+6.0+12.2 = 266.5s → /280.3 = 95.1% | — | `.tmp/perf-reg-260910-04/logs/r3sync-r1.log:165` |
| 两臂参数（`r3` = chunktime；`r3sync` = chunktime + syncfill） | — | `.tmp/perf-reg-260910-04/run_arms2.ps1:25-26`；映射 `runtime/1.21.6/java/build.gradle:134-135` |
| 历史双臂基线 56,214 / 375,549,952 = 0.0150% | — | `.tmp/hang-repro-260910/diff-result.txt:74` |
| 三臂对拍 44,921 / 45,948 / 41,809（分母 375,554,048） | 任务书提供；**本 subagent 全树检索未命中落盘文件** | 无（诚实边界 2） |
| 同步形态历史读数 1325ms（R3 前 ct1 臂） | — | `.artifacts/perf-regression-260910-04/verdict-260910-04.md:23`（§2.3）；#109（workflow-patterns 1759 行） |
| `cmd-output/results.txt` 为 R3 前版本（9 行、无 r3 行） | — | `cmd-output/results.txt`（共 9 行）；`MANIFEST-sha256.txt:23`（1604B，18:09 生成） |
| 归档 MANIFEST 未含 `r3-async-r1.log` | — | `MANIFEST-sha256.txt`（28 行清单，无该文件） |

---

## 自检清单（按 `SUBAGENT-KNOWLEDGE-GUIDE.md` §四）

- [x] **先过价值门**：高价值（可复用判据 = 修复验证三件套 + 「段内延迟上升≠回归」陷阱）→ 详写；逐臂读数只作证据，不单独入库。
- [x] 每个错误/缺口都有机制层说明（诚实边界 1-6 均给「为什么」而非仅「未做」）。
- [x] 定位含诊断方法/工具（`diff_arms.py` 逐块对拍 / `ChunkTiming.inflight` 直读 / Σ·wall 同尺 / arm-agnostic featInterval）。
- [x] 判错经验已沉淀（零容忍门是错门、段内延迟与吞吐不同物、跨批绝对值不可引）。
- [x] 被排除/未闭合项标注：上游限流线 ⚠️ 未排除（诚实边界 4）、车道 busy 计数未做（#107 判据 4 / b1 @idk 保留）。
- [x] 写入载体正确：通用可复用判据 → `knowledge/discovered/workflow-patterns.md`（#107 家族追加）；本块不触 `docs/`（无主题篇结论）。
- [x] 无复用价值的一次性读数（42/279/52s 等）**只在条目内作证据**，未单列知识库条目。
- [x] 数字来自一手文件；**跨文件不一致处已显式标注并给出「以文件为准」**（诚实边界 6：1325ms vs 1413.65ms；核对表末三行标注未落盘项）。
- [x] 格式与目标文件现状对齐：先读 #107/#108/#109 现网正文，沿用「时间/置信度/module + 来源定位 + 数据 + 判据 + 家族索引」字段风格。
- [x] 只读纪律：本次仅新建本草稿文件，未修改 `knowledge/`、`docs/`、`.artifacts/`、`NEXT_SESSION.md`；未运行任何命令。
