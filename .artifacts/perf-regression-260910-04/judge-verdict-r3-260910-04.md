# judge verdict（R3 增量）— 1.21.6 性能回归 · R3 异步化修复 + 同批 A/B 验证

> 审查角色：core-judge（subagent 隔离、只读、无 shell）。审查节点：**用户授权 confirmed 前的增量审查** —— R3（judge 之后新增的代码改动 + 验证）。
> 审查对象：`verdict-260910-04.md` §9（R3 修复验证）与 §1 顶部追加；代码本体 `runtime/1.21.6/java/.../mixin/NoiseChunkGeneratorMixin.java`、`runtime/1.21.6/java/build.gradle`、`ChunkTiming.java`、`CppBridge.java`；原始证据 `.investigations/perf-regression-260910-04/cmd-output/`（results/logs/diff/region/MANIFEST）；`knowledge/discovered/workflow-patterns.md` #107 增量（1742-1788）+ `knowledge/INDEX.md:134`；`NEXT_SESSION.md`。
> 上一轮 judge（`judge-verdict-260910-04.md`，PASS-with-conditions）审的是「机制定责 candidate」且 R3 尚未实施；本文件只审 **R3 增量**，不重审 §1-§8 主体（除与 §9 一致性相关的引用）。
> **本文件只出审查意见，不改任何结论状态、不改任何被审产物。** 审查方式：逐行回读原始证据 + 一手 MC 1.21.6 源码（`versions/1.21.6/data/mc_src_extract/`）+ Rust 侧共享状态静态核对；**无 shell，未跑 git、未算 sha256**。

## 判定：**PASS-with-conditions**

R3 增量的**主干成立且方向可信**：A/B 的机制设计（旧形态保留为开关、同 dll/同 seed/同 region、背靠背）合规；`inflight max` 23→1、wall 42s→279s、CPU 543→375、三个逐块对拍数字**全部可逐行核到**；结论的状态纪律干净（candidate，无自授 confirmed）；异常契约经一手源码核对**比 §9 自述更强**（有 MC 设计的崩溃路径）。但增量里有 **1 处数字错标（把 R3 前 ct1 臂的 1325ms 写成 r3sync 臂读数，而该臂自身末值是 1413.65ms）**、**1 处措辞强于代理基线所能支撑**、**3 项新暴露的并发面未声明**（其中 `pending_cross_writes` 顺序敏感是等价门解释的直接竞争假说）、**归档对 R3 臂的驱动脚本锁了错修订**、以及**知识条目 2 条诚实边界已对现网过时（现为假陈述）**。这些都不需要新测量即可修（唯一例外见 C7），故判 **PASS-with-conditions**，不判 FAIL。

---

## 三源核对（逐项：核到 / 未核到 / 矛盾 + 文件:行）

### 1. 数字可核对性（§9 逐个数字）

| verdict §9 数字 | 核到来源（文件:行） | 判定 |
|---|---|---|
| r3 **42s** | `cmd-output/r3-async-r1.log:148`（`Processed: 4225 chunks (100.00%), Total time: 0:00:42`）+ `cmd-output/results.txt:10`（`sec=42`） | ✅ 核到 |
| r3 **543 CPU-s** | `results.txt:10`（`serverCpu=543`） | ✅ 核到 |
| r3 **10.8 核** | 派生 543/50.1 = 10.84（`results.txt:10` `wallgen=50.1`）；知识条目表格明标「派生核数」✅ | ✅ 核到（派生，已声明） |
| r3 **inflight max=23** | `r3-async-r1.log:147`（`… \| inflight max=23 now=19`）；且 :110-146 每行均为 `max=23`（`now` 0-23 波动） | ✅ **核到（本臂日志原位）** |
| r3 **242.5ms** | `r3-async-r1.log:147`（`armAgnostic featInterval=242.52ms(n=4411)`） | ✅ 核到（截断 1 位） |
| r3sync **279s** | `r3sync-r1.log:167`（`Total time: 0:04:39`）+ `results.txt:11`（`sec=279`） | ✅ 核到 |
| r3sync **375 CPU-s** | `results.txt:11`（`serverCpu=375`） | ✅ 核到 |
| r3sync **1.34 核** | 派生 375/280.3 = 1.338（`results.txt:11` `wallgen=280.3`） | ✅ 核到（派生） |
| r3sync **inflight max=1** | `r3sync-r1.log:107,108,110,112,114,115,117,119,120,122,124,125,127,129,131,132,134,136,137,140,141,143,144,146,148,150,151,154,155,157,158,160,162,164,165` 逐行 `inflight max=1`（含末行 :165） | ✅ **核到（本臂日志原位）** |
| r3sync **1413.65ms** | ❌ **verdict §9 未使用该值**；表内写的是 `~1325ms` | ❌ 未核到（见下行矛盾） |
| r3sync **~1325ms**（§9 表 :90） | `1324.99` 只出现在 `cmd-output/ct1-inflight-r2.log:163`（R3 **之前**、18:08 那次同步 run）；`r3sync-r1.log` 全篇**无 1325**，该臂自身末值 `:165` = **1413.65ms(n=4416)** | ❌ **矛盾（错标）** |
| vanilla **52s** | `results.txt:12`（`sec=52`）+ `.tmp/perf-reg-260910-04/logs/vanilla1216-r1.log:107`（`Total time: 0:00:52`，18:21:52） | ✅ 核到（⚠️ 该臂日志**未进 cmd-output**，只在 gitignore 的 .tmp） |
| vanilla **316 CPU-s / 5.26 核** | `results.txt:12`（`serverCpu=316` `wallgen=60.1`）→ 316/60.1 = 5.26 | ✅ 核到（派生） |
| vanilla **299ms** | 实为 `vt1-r1.log:124`（18:00 那次 53s 的 vanilla run）的 `featInterval=298.75ms`；R3 批的 `vanilla1216-r1` **未开** `-Dcoreswap.chunktime`（其日志全篇无 `[CHUNKTIME]`） | ❌ **未核到（跨批借用，且与「（同批）」标注并列）** |
| 三对 diff **44,921 / 45,948 / 41,809** 与 **0.0120% / 0.0122% / 0.0111%** 及 sections | `diff-r3-vs-vanilla.txt:74`（`blocks=375554048 diff=44921 (0.0120%) sections same/diff=91176/512`）、`diff-r3-vs-r3sync.txt:74`（`…45948 (0.0122%) … 91289/397`）、`diff-r3sync-vs-vanilla.txt:74`（`…41809 (0.0111%) … 91154/533`）与 §9 表逐格一致 | ✅ **核到（三臂全域普查输出已落盘）** |
| 历史双臂基线 **0.015%** | `.tmp/hang-repro-260910/diff-result.txt:74`（`diff=56214 (0.0150%) sections same/diff=91191/496`） | ✅ 核到（⚠️ 未归档，见 C5） |
| **6.64× / 1.24×** | 279/42 = 6.643；52/42 = 1.238 | ✅ 核到（派生） |
| r3 **Rate 102.7 cps** | `r3-async-r1.log:139`（Chunky `Rate: 102.7 cps`） | ✅ 核到 |
| **vanilla 81 cps** | 无任何日志行给出 81：同批 `vanilla1216-r1` 末次实测 `Rate: 84.1 cps`（`.tmp/…/vanilla1216-r1.log:106`）、`vt1-r1.log:121` = 84.6 | ❌ **未核到**（81 = 4225/52s 派生；与实测 Rate 并列易被读作实测） |
| Σ/wall **0.95 → 9.98**（knowledge/INDEX/NEXT 用，§9 未用） | 266.5/280.3 = 0.951（`r3sync-r1.log:165`：248.3+6.0+12.2）；500.0/50.1 = 9.98（`r3-async-r1.log:147`：476.8+7.4+15.8） | ✅ 核到（本增量两文件一致，§9 未引不构成矛盾） |

**小计**：§9 的 20 个数字里 **17 个逐行核到**（含 2 个派生已声明、1 个 1413.65 被错值取代）；**2 个未核到（vanilla 299ms、vanilla 81 cps）**；**1 个矛盾（r3sync ~1325ms 应为 1413.65ms）**。

### 2. 单变量性（r3 vs r3sync 是否只差 `-Dsyncfill=1`）

| 核对项 | 结果 |
|---|---|
| `conf=` | `results.txt:10` 与 `:11` **逐字相同**（`ca_min=true est_l2=true ca_cap=2048 flags=3 skip_features=true skip_carver=true skip_surface=false coreswap_threads=(unset)`）；两臂 `.err` 的 `[WG-CONF]` 行亦逐字相同（`r3-r1.log.err:76` / `cmd-output/r3sync-r1.log.err:76`） ✅ |
| `dllsha=` | 两行同为 `abd7d8893d22e030`；两臂日志原位亦同（`r3-async-r1.log:97`、`r3sync-r1.log:94`，`size=2457088`，sha 均为截断 16 hex） ✅ |
| `chunks=` | 两行均 `chunks=4225`；两臂日志均 `Processed: 4225 chunks (100.00%)` ✅ |
| `total=` | 42s vs 279s（差异本身即被测对象） ✅ |
| seed / worldgenDir | 两臂 `[CppBridge] init seed=417950215108767439 … worldgenDir=E:\PYTHON\CoreSwap\versions\1.21.6\data\worldgen enabled=true stageMask=3`（`r3-async-r1.log:96` / `r3sync-r1.log:93`） ✅ |
| **构建态同源（关键旁证）** | 两臂日志**都**打印 `inflight max=` ⇒ 都含 R3 改动集的仪器；而 `-Dcoreswap.syncfill` 只存在于 R3 后的 build.gradle（`:134-135`）⇒ 两臂必为**同一 post-R3 jar**；旧 jar 无该仪器，故「同步臂其实是旧 jar」被排除 ✅ |
| 世界残留 / 抢跑 | 驱动脚本每臂 `Get-Process java \| Stop-Process` + `gradle --stop` + `Remove-Item run\world` + 覆写 seed（`.tmp/perf-reg-260910-04/run_arms2.ps1:42-48`）；三臂日志时间戳连续无重叠（r3 18:13:06-58 → r3sync 18:15:08-19:57 → vanilla 18:21:02-52） ✅ |
| **驱动脚本参数（开关层证据）** | ❌ **矛盾**：`cmd-output/run_arms2.ps1`（MANIFEST-sha256.txt:31 所锁修订，121 行）是 **R3 前版本**——arm 表只有 c*/ct1(:24)/vt1(:27)，**无 r3/r3sync**；含两臂定义的是 `.tmp/perf-reg-260910-04/run_arms2.ps1:25-26`（123 行，两臂唯一差异 = `-Psyncfill=1`）。该文件在 `.gitignore` 区、未归档；`knowledge-drafts-r3.md:93` 引的 `run_arms2.ps1:25-26` 指的正是 .tmp 版 ⇒ **同名归档物 ≠ 产出证据的修订** |
| 开关字面记录 | 全 `cmd-output/` grep `syncfill` = **0 命中**（日志不打印 JVM args，results.txt 无该字段）⇒ 单变量的「开关层」无一手归档记录；只能靠**行为面**证明（279s + `inflight max=1` + `gap≈1224ms` 只有 SYNCFILL 分支能产生；且该臂带 post-R3 仪器） |

**结论**：设计层面的单变量性**成立且证据充分**（同 dll/同 conf/同 seed/同 region/背靠背 + 构建态同源旁证）；但**证据层面的单变量性有一处归档裂缝**（见 C5-①）。

### 3. 行为等价门的解释是否过度（三臂同量级 ⇒ "R3 未引入超出既有 run 级非确定的内容差"）

- **数字与工具**：三对数字、占比、`sections same/diff` 逐格核到；工具 `cmd-output/diff_arms.py:104,125,162` 的输出格式与落盘 `diff-*.txt` 完全一致（`common=` / `===全域差分普查===` / `blocks=… diff=… (…%) sections same/diff=`）⇒ 工具修订可自洽（⚠️ 但未做逐行算法复核）。
- **基线可比性（§9.7 口径）**：判定为「**门成立、因果陈述过度**」：
  1. 基线 `0.0150%` 来自 **coreswap vs vanilla 的跨实现对**（R3 前，`diff-result.txt:74`），它混合了「跨臂系统差 + run 级非确定」；三对里最接近同配置基线的是 **r3 vs r3sync**（同 dll、同管代码、只差形态）0.0122%。§9:105 已**如实声明**「同配置 run-to-run 基线对未采集…当前用『同步 vs vanilla』作基线代理」✅ 诚实边界到位。
  2. 但 §9:104 的箭头句「⇒ R3 **未引入超出既有 run 级非确定的内容差**」比代理能支撑的更强：知识库 **#67**（`workflow-patterns.md:1223`）已定论「features 固有 **run 级非确定** = chunk **完成序路径依赖**」；R3 把完成序自由度从 1 提到 23 ⇒ 依 #67 机制，异步臂自身的 run 间非确定**预期变大**，用 R3 前的跨臂基线当上界在方向上偏乐观。应改写为「未超出**代理**基线；同配置 run-to-run 基线未采集 ⇒ 不排除异步形态自身 run 间非确定更大」。
  3. **未声明的竞争假说（静态可核，§9 全文未提）**：管线存在**顺序敏感的跨 chunk 写**路径 —— `worldgen-core/src/worldgen_handle.rs:1085-1091`（越界写 → `pending_cross_writes` 缓冲）与 `:1050-1063`（目标 chunk features 前 overlay，`ca_min` **默认 true**，`:1050`）。若目标 chunk **已过**该 overlay 点，缓冲条目**永不被应用**（写丢失 + map 常驻）。inflight=1 时填充序 = 车道序（确定）；R3 后序不定 ⇒ 同一臂自身 run 间内容可漂，且 async 与 sync 的**系统差**可藏在 45,948 块里。⇒ 「等价门未过（同量级）」并不能区分「run 噪声」与「丢失写」，故门对异步形态的证明力弱于对同步形态。
  4. 判据强度提示：§9:107 的③写「与既有 run 级噪声基线同量级即可（非零容忍）」——作为**筛选门**可接受；但 §9:109 ④ 用「行为等价门已过」为 section 锁偏差背书，对**竞态型**问题不构成证明（门只覆盖一次 run 的内容表现）。
- **是否需要作为 confirmed 前置**：不需要新测量即可如实声明（C2/C3）；但**量化**（跨 chunk 写序会计）被列为高价值后置项（C7）——若结果不利，按 §15.4 取代重审等价门。

### 4. 代码正确性风险（只读静态；一手源码 + 本体）

| 审查点 | 结论（含 file:line） |
|---|---|
| `WG_FILL_POOL` 静态初始化顺序/类加载风险 | ✅ **无风险**：`Util.getMainWorkerExecutor()` 返回 `MAIN_WORKER_EXECUTOR`（`mc_src_extract/net/minecraft/util/Util.java:103` **static final** = `createWorker("Main")`），getter 无 null 前置条件（`:262-263`）⇒ 调用即触发 `Util.<clinit>` 建池，任何时刻非 null。Mixin 静态字段初始化并入目标类 `NoiseChunkGenerator.<clinit>`（唯一副作用 = 池可能比 vanilla 早建，无后置依赖） |
| `SYNCFILL` 默认 | ✅ 一致：`mixin:61` `System.getProperty("coreswap.syncfill") != null` ⇒ 默认 false ⇒ 默认异步，与 §9:84 表述一致；映射 `build.gradle:134-135` 在 `benchVmArgs` 闭包内（闭包 `:58`，调用点 `:196/:203`）✅ |
| 异常「打印后 rethrow」/ future 是否被吞 | ✅ **mixin 层成立，且比自述更强**：`mixin:131-136` 打印 + `throw t`；返回的 future 异常由 `AbstractChunkHolder.java:69-77` 接管（`CrashReport.create(throwable,"Exception chunk generation/loading")` + `MinecraftServer.setWorldGenException(new CrashException(...))`）⇒ 走 MC **设计的 worldgen 崩溃路径**（vanilla 走 `ChunkGenerating.java:83-99` 的 `.thenApply` 同契约）。且 mixin 的 HEAD 注入本身**不抛**（只 `setReturnValue`）⇒ `ChunkTaskScheduler.schedule`（`:79-84` `allOf(...).thenAccept(v -> pollTask())`）不会因子任务异常而**停摆** ✅。<br>⚠️ **但 §9:84 的表述端到端不成立**：`CppBridge.fillChunk` 内部 catch 后 **print + return（不抛）**（`CppBridge.java:383-386`、`:388-391`、`:403-405`），`feedBeardifier` 同型（`:360-362`）⇒ 原生失败/写回失败在该层被吞（既有行为，非 R3 引入）；R3 后这些点在 23 线程上并发出现，半填 chunk **不会中止运行** → 见 C4 |
| nether/end 仍同步是否如实声明 | ⚠️ **部分**：`mixin:154-167` 确仍 `completedFuture`（同步）；`NEXT_SESSION.md:34` ③ **已显式**列出「本块只改了 overworld 分支；nether/end 仍同步」；但 **verdict §9 只写「overworld 分支」（隐含），未显式声明**，§8/§9「仍开放」亦未列 → 建议补一行（C3-c） |
| 并发面（23 线程并发触达 Rust 句柄） | **锁类型层：无数据竞争** — `beardifiers: RwLock<HashMap>`（`worldgen_handle.rs:116`）、`terrain_cache: Mutex`（`:141`）、`est_l2: OnceLock<Option<Arc<Mutex<EstL2>>>>`（`:133`）、`pending_cross_writes: Mutex`（`:144`）。<br>**语义层：三项新暴露/未声明** ——（a）`pending_cross_writes` 顺序敏感（见 §三源-3.3）**未声明**；（b）**每调用 scoped 线程 × 23 并发**：`api.rs:140-169` 每次 `wg_fill_blocks_multi` 起 `adaptive_threads()` 个 `thread::scope` 线程，默认 = 物理核-2 = **10**（`api.rs:24-41`；`[WG-CONF] coreswap_threads=(unset)`）⇒ R3 下最坏 **23×10 ≈ 230 OS 线程 / 24 逻辑核**；实证同向：CPU 375→543（+45%）、段内 mixin 53.87→103.47ms（+92%，`r3sync-r1.log:165` vs `r3-async-r1.log:147`）——**这使 §2.4/§7-C3「Rust 每调用线程数不是限流点（c2 反证）」不能外推到新形态**（c2 测于 inflight=1 形态）**未声明**；（c）`beardifiers` 每 chunk insert（`:536`）**无移除**、Java 侧无 `clearBeardifier` 调用（`runtime/` 全 grep 无命中；唯一 clear 在 `:539-541`）⇒ 会话内无界增长（**既有**，非 R3；锁语义安全）。<br>**内容中性核对**：`terrain_cache` 的 `len>=cap → clear`（`:630-633`）与 `EstL2` 淘汰（`aquifer.rs:277`）只改命中率、不改值（纯 memo，键→纯函数值）⇒ 内容中性 ✅（前提与设计一致） |
| `writeChunk` 池线程 section 锁语义 | ✅ §9:109 ④ 的**事实陈述核到**：`CppBridge.java:528` 调 4 参 `ChunkSection.setBlockState(x,sy,z,st)` → `ChunkSection.java:64-65` 委托 `setBlockState(...,lock=true)` → `blockStateContainer.swap(...)`（逐格自锁）；vanilla 则在池线程内**整段** `lock()/unlock()` 24 个 section（`NoiseChunkGenerator.java:336-349`）。⇒ 偏差存在、已声明；判据强度见 §三源-3.4 |
| 代码与 §9:84 改动描述一致性 | ✅ 「feedBeardifier + fillChunk + writeChunk 移入 future」核到：lambda（`mixin:119-144`）内 `feedBeardifier`(`:127`)+`fillChunk`(`:129`)，而 `writeChunk` 由 `fillChunk` 传递调用（`CppBridge.java:402`）⇒ 三者确在异步任务内；`cir.setReturnValue(supplyAsync(work, WG_FILL_POOL))`（`:148`） |

### 5. 状态纪律 / 落盘契约 / 取代链一致性

- ✅ **无自授 confirmed**：260910-04 全目录 grep `status: confirmed` **0 命中**；`index-entry.yaml:8,12,16,20` = candidate/draft/draft/draft；`knowledge/INDEX.md:134` 明写「**candidate**（同批 A/B + 直证，confirmed 留人类）」；`NEXT_SESSION.md:47` 待办「用户 confirmed ← 待用户拍板」仍在。
- ⚠️ **索引登记不齐**：根 `.artifacts/index.yaml:1163-1178` 只登记 verdict/b1/b2（3 条），**未登记** `judge-verdict-260910-04.md`（子 `index-entry.yaml:17-20` 有）⇒ confirmed 后应补登本轮 judge-r3 + 同步状态（建议，我不改）。
- ❌ **取代链内部不一致（verdict §9 vs 知识条目）**：`workflow-patterns.md:1757`（#107 增量表）用 **1413.65ms**，`:1786`（诚实边界 6）明确「1325ms 出自 R3 **之前**的 ct1 臂（#109 / verdict §2.3）…本条一律引用后者（1413.65），两者不混用」；而 **verdict §9 表 :90 把 `~1325ms` 放在 r3sync 行** ⇒ **知识条目正确、verdict §9 错**（须以文件为准改 verdict）。`NEXT_SESSION.md:12` 未列每 chunk 线程时间 ⇒ 不冲突。
- ❌ **知识条目 2 条诚实边界已对现网为假（过时陈述）**：`workflow-patterns.md:1782` 称三臂 diff 数字「目前**未落盘**…全树检索零命中」、`:1783` 称「同步臂日志未进 `cmd-output/`；`cmd-output/results.txt` 仍是 **9 行**版；MANIFEST 生成于 **18:09**，R3 臂未纳入清单」——**现网事实**：`cmd-output/diff-{r3-vs-vanilla,r3-vs-r3sync,r3sync-vs-vanilla}.txt` 均在位且入 MANIFEST（`:16-18` 有 sha）、`results.txt` **12 行**（MANIFEST:30 已锁）、`r3sync-r1.log` 已归档（MANIFEST:23）、MANIFEST 头 `:1` = 「生成于 2026-09-10 **18:30:03**（R3 修复后刷新）」⇒ 归档滞后**已清偿**，条目仍写旧态（对知识资产是**假陈述**）→ C6。
- **归档完整性（MANIFEST 30 条 vs 磁盘）**：MANIFEST 锁 30 个文件；**未覆盖** `region-{r3,r3sync,vanilla-260910-04}/**.mca`（48 个 .mca = 等价门一手载体，无 sha）、未含 `r3-r1.log.err`（异步臂 [WG-CONF] 行）、未含 `vanilla1216-r1.log`（同批 vanilla 臂）、未含 `.tmp/hang-repro-260910/diff-result.txt`（基线 0.0150% 唯一出处）；异步臂日志以**改名**形式归档（tag `r3-r1` → 文件 `r3-async-r1.log`）→ C5。
- ⚠️ **时标早于证据**：§9 头与 §1 顶部追加均标「2026-09-10 **18:15**」，但 §9 内容依赖 18:15:08-18:19:57 的 r3sync run 与 18:30:03 的归档刷新 ⇒ 建议改「18:1x-18:3x（实际完成时刻）」（建议项 N3）。

---

## 必改条件（C1..C6，逐条可执行；C7 为条件性）

**C1（MUST）§9 表两处数字修正（confirmed 文本里不得留错值）。**
动作：① `:90` r3sync 单元格 `~1325ms` → **`1413.65ms（n=4416，cmd-output/r3sync-r1.log:165）`**，并加脚注「1325ms = R3 前 ct1 臂旧读数（`ct1-inflight-r2.log:163`），本条不与本臂混用」；② `:92` vanilla 行 `299ms` 标注来源（`vt1-r1.log:124`，18:00 那次 53s run）并在表注写明「vanilla1216-r1 未开 chunktime，段内数字跨批不可比」，或删除该格；③ `:94`「vanilla 81 cps」标注为派生（4225/52s）或改引实测 84.1/84.6。

**C2（MUST）等价门因果句降级（§9:104）。**
动作：改为「三者同量级（0.0111-0.0122% ≤ R3 前跨臂代理基线 0.0150%）⇒ **未超出代理基线**；**同配置 run-to-run 基线未采集**，且按 #67（完成序路径依赖）R3 把完成序自由度由 1 提到 23、异步形态自身 run 间非确定**预期更大** ⇒ 本门为**同量级筛选门**，不构成『R3 未引入额外内容差』的定论」。保留 `:105` 的 ⚠️ 声明（已合规），把句子与它绑定。

**C3（MUST）补「并发面诚实边界」三条（§9 新增小节或并入「仍开放」）。**
① `pending_cross_writes` **顺序敏感**：`worldgen_handle.rs:1050-1063`（目标 features 前 overlay）/`:1085-1091`（越界写缓冲），目标已过点 ⇒ 写丢失 + 条目常驻；inflight=1 时序确定、R3 后序不定 ⇒ 异步臂自身 run 间内容可漂、async-vs-sync 系统差可能藏在 45,948 块中（等价门的**竞争假说**）。
② **每调用线程数 × 并发**：`api.rs:140-169` + `:24-41` 默认 10 scoped 线程/调用 ⇒ 23 并发最坏 ≈230 线程 / 24 逻辑核；同向实证 +45% CPU（375→543）、+92% 段内 mixin（53.87→103.47ms）；并**限定 §2.4/§7-C3 的适用范围**为「旧（inflight=1）形态」。
③ **nether/end 仍同步**（`mixin:154-167`，已在 `NEXT_SESSION.md:34` 声明）：§9 补一行，以免 confirmed 文本被读成三维修好。

**C4（MUST）异常表述收窄（§9:84）。**
动作：改为「mixin 包装层不吞、打印现场后原样抛出，异常由 `AbstractChunkHolder.java:69-77` 走 MC 的 worldgen 崩溃路径（CrashReport + `setWorldGenException`）；**但 `CppBridge.fillChunk/feedBeardifier` 内部为 print+return（`CppBridge.java:383-391,403-405,360-362`）⇒ 原生/写回失败在该层被吞（既有，非 R3 引入）**」；并把「是否改为 rethrow」列为技术债（与崩溃日志铁律的取舍交人类）。

**C5（MUST）归档补全（使 R3 证据可独立复核）。**
① 把 `.tmp/perf-reg-260910-04/run_arms2.ps1`（**R3 修订**，两臂 `:25-26`；现 MANIFEST:31 锁的同名文件是**不能产出该证据的旧修订**）复制进 `cmd-output/` 并刷新 MANIFEST；
② 补 `region-{r3,r3sync,vanilla-260910-04}/**.mca`（48 文件）sha256 清单；
③ 补 `r3-r1.log.err`（异步臂 [WG-CONF]）、`vanilla1216-r1.log`（同批 vanilla 臂）、`.tmp/hang-repro-260910/diff-result.txt`（基线 0.0150% 出处）；
④ 建议 `results.txt` 每行增 `args=` 字段（记录该臂 extra args）——永久消除「开关无一手记录」。

**C6（MUST）修正知识条目过时陈述（`workflow-patterns.md:1782-1783`）。**
动作：改写为「归档滞后**已清偿**：diff 输出三份已落盘并入 MANIFEST（`:16-18`）、`results.txt` 12 行、`r3sync-r1.log` 已归档、MANIFEST 18:30 刷新」，保留一行历史说明（原滞后事实）+ 本轮 judge 发现的剩余缺口（cross-chunk 写序会计未完）。

**C7（条件性 MUST——若用户要把「R3 修复效果」也纳入同一 confirmed 措辞，则须先满足；仅确认机制结论时不阻塞）等价门收口：跨 chunk 写序会计 + 同配置 run-to-run 基线对。**
各 1 次即可、成本低：① 两臂各计 `pending_cross_writes` 的**生产 / 应用 / 遗留**三数（现有 `CA_PENDING_WRITES` 只计生产，`worldgen_handle.rs:1081,1087`）——遗留量即「丢失写」上界；② `r3` 再跑 1 run 与本地 `r3` region 逐块对拍，得同配置 run 级基线。若 ① 显示 async 遗留 ≫ sync 或 ② 基线与三对同量级或更大 ⇒ 按 §15.4 取代重审等价门与「未引入内容差」结论。

---

## 建议项（N1..N5，不阻塞）

- **N1**：把 `.artifacts/index.yaml`（根）与子 `index-entry.yaml` 一并更新：登记 `perf-regression-260910-04:judge-r3`（本文件，kind=review，status=draft）与已存在的 `judge-verdict-260910-04.md`（根索引现缺）；confirmed 授予后仅在 verdict 条目上标 confirmed（附授予人与时间），judge 条目保持 draft。
- **N2**：把 `knowledge-drafts-r3.md:30,64,97,98` 的过时证据边界（"未归档/MANIFEST 18:09/9 行"）在草稿层加一行「已被 C5 清偿」，避免下次有人照草稿复述。
- **N3**：§1 顶部追加与 §9 头的「18:15」改为实际完成时刻（如「18:13-18:30」）。
- **N4**：`r3` 臂名与归档文件名不一致（tag `r3-r1` → `r3-async-r1.log`）；建议归档保持 tag 同名（`r3-r1.log`）或在校验清单加改名说明，避免以后 `results.txt` 行 ↔ 日志文件对不上。
- **N5**：§9 表可加一列「线程时间比 1413.65/242.52 = 5.83×」与 wall 比 6.64× 并列（知识条目 `:1761` 已有，§9 缺），让三路互证（wall / 直读并发 / 同臂周期）在 verdict 本体可见。

---

## 对 confirmed 授予的明确意见

**支持 now confirmed** —— 对以下**范围**分别表态：

1. **机制结论（§1 命题 A「单车道 ⇒ 并发恒为 1」）→ 支持**。A 已是直接实测（`inflight max=1` 在 1 run 多窗恒成立），与本轮 R3 修复后的 `inflight max=23` 构成同一仪器的前后对照，机制命名、判据（Σ/wall 翻转 0.95→9.98）、以及修复方向自洽。
2. **R3 修复效果（异步化）→ 支持**，但**建议措辞限定为**：方向与量级阶可信（同构建态单变量 A/B 42s vs 279s = 6.64×，**远超**本项目 ±20% 跨 run 摆动带）；幅度是**单对读数**（同配置 run-to-run 基线未采集）；「R3 未引入额外内容差」为**代理基线级筛选结论**，非定论。
3. **前置条件**：C1-C6（**全部为数字修正 / 措辞收窄 / 声明补齐 / 归档补全 / 知识条目纠错，不需任何新测量**）应先应用；C7 的两项测量**不阻塞 now confirmed**，但须在 confirmed 后列为第一优先收口项，且 §9 须写明「若 C7 结果不利则按 §15.4 取代重审」。
4. **我不同意**把任何形式的「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响」写入 confirmed 文本——现有证据不支持（等价门为同量级门，且 §三源-3.3 给出竞争机制）。

## 未核项与诚实边界

1. **无 shell**：未跑 `git status/diff`（HEAD/worktree 第②源不可核；仅继承前 judge 的 `HEAD=ee0d468…` 与 `/runtime/`、`.tmp/` 在 `.gitignore` 的判断），**未计算任何 sha256** ⇒ MANIFEST 的 sha 值我**未独立复核**；我只核「清单条目 ↔ 磁盘文件」对应性，并在此发现 `run_arms2.ps1` 条目锁的是**错修订**（C5-①）。
2. **未核**：`region-*/**.mca` 内容（仅核文件名与数量 16/16/16，及三份 diff 的 `chunks=7959 common=7959` 自述）；`diff_arms.py` 的算法正确性（只核输出格式与 `:104,125,162` 一致，未逐行复核分母「两侧 section 并集」口径）。
3. **未核**：`serverCpu` 采集口径的正确性（依赖脚本「WorkingSet 最大的 java 进程」启发式与 `cpu0` 起点，`run_arms2.ps1:70-72,97-99`；无法验证采样时刻/进程选择），以及 `wallGen`(50.1s) 与 Chunky `Total`(42s) 的 8s 差未归因。
4. **未核**：日志中的 dll sha 只有 16 hex（`abd7d8893d22e030...`）⇒ 无法与完整 sha256 逐字对上；`target/release` 与 `.tmp/java-tmp/coreswap-native` 两个 dll 副本一致性与 `runtime/1.21.6/java` 的编译产物新鲜度（R3 是否已重编进 jar、jar 构建时刻）均未核（无 git 源、无 shell）。
5. **未核**：`vanilla 81 cps` 另有出处（`.tmp/…/vanilla1216-r1.log` 只有 84.1）；三臂 region 的 `y-section dist` 大量差异的定位（无逐 chunk 定位输出，§三源-3.3 属**假说**而非已证机制）。
6. 我**未修改**任何被审文件（verdict/代码/知识库/INDEX/NEXT_SESSION/已有 judge 文件均未动）；本文件是本次审查的唯一写入。
