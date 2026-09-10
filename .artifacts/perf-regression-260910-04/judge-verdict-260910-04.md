# judge verdict — 1.21.6 性能回归定责（260910-04）

> 审查角色：core-judge（subagent 隔离、只读、无 shell）。审查对象：`.artifacts/perf-regression-260910-04/verdict-260910-04.md`（candidate）+ 同块 record/scout/fan-out/架构计划 + `.tmp/perf-reg-260910-04/` 原始行。
> **本文件只出审查意见，不改任何结论状态。** candidate/confirmed 的授予由人类裁决；以下「判定」是对**产物质量**的判定，不是状态变更。
> 审查时点：本轮审查进行期间 worktree 与 `.tmp` 原始日志均在变动（详见 §三源核对-2、§补充证据复核、§未核项-1），本意见对应的是一份**移动快照**。

## 判定：**PASS-with-conditions**

主结论（「5.15× 不是 Rust 引擎慢，而是重活同步压在世界生成的唯一串行车道上」）**证据链主干成立**：核心数字几乎全部可在原始行中定位，静态机制我独立核对了一手源码且与 b1 所述一致，无自授 confirmed、无模块越界、§9.7 三要素在场、index 契约完整。**审查期间新增的 in-flight 直接仪器进一步把「同一时刻至多 1 个接管段在飞」从推导升级为实测直证**（见 §补充证据复核）。但仍有 8 项必须修的条件，其中 C1（原始日志未归档且已被新 run 覆盖）、C2（±20% 摆动错标；「并行度 1.000」措辞须按新证据重新分配「已直证/仍推断」两部分）、C4（R1 的 −13% 越噪声带）会直接影响结论的可复核性，**不满足前不得进入 confirmed 流程**。

---

## 三源核对结果（逐项：核到 / 未核到 / 矛盾 + 文件:行）

### 1. artifacts 快照 vs 原始行 —— 数字出处（逐项）

| verdict 数字 | 核到原始出处 | 结论 |
|---|---|---|
| 1.21.6 native **39.86ms** / 1.20.1 **38.55ms** | `.tmp/perf-reg-260910-04/native-A2.txt:4`（`chunks=256 wall=10204.8ms per_chunk=39.86ms hash=0603683700a83f53`）、`:14`（`per_chunk=38.55ms`）；另 `:9` 39.43 / `:19` 42.66 | ✅ 核到（record §2.2 一致） |
| hash 哨兵 **6908dbfc… / 115641b8…** | `native-A.txt:54`（`hash=6908dbfc9a79c40a`，ca_min on）、`:108`（`hash=115641b86711d9cd`，ca_min off）；对照 `.investigations/camin-perf/probe-260909-06.md:10,52,67` 同两值 | ✅ 核到，**逐位一致成立**（record §2.1 的「与 260909-06 逐位同」为真） |
| coreswap Σ = **251.1+5.7+12.3 = 269.1s** | `logs/ct1-r1.log:165`（审查时读到的原文：`[CHUNKTIME] n=4608 … sumMixin=251.1s sumGap=5683.9s sumCarve=5.7s sumFeat=12.3s`） | ✅ 核到，且**与 wall 同批同臂**（同文件 `:167` `Processed: 4225 chunks (100.00%), Total time: 0:04:43`；`results.txt:7` `ct1-r1 … sec=283 wallgen=290.3`）→ 269.1/290.3 = **92.7%** 复算一致 |
| 每 chunk 线程时间 **1422ms / 299ms** | `ct1-r1.log:165` `featInterval=1422.69ms(n=4415)`；`vt1-r1.log:124` `featInterval=298.75ms(n=4352)` | ✅ 核到（4.76× vs wall 5.34× 的「同阶」复算成立） |
| vanilla Σ = **0+3.9+13.1 = 17.0s** / wall 60.1s / 28% | `vt1-r1.log:124`（`sumCarve=3.9s sumFeat=13.1s`）、`:125`（`Total time: 0:00:53`）；`results.txt:8`（`sec=53 wallgen=60.1 serverCpu=296`） | ✅ 核到；17.0/60.1 = 28.3% ✅ |
| **1.27–1.7 核** vs vanilla **4.93 核** | 1.7 = `results.txt:2`（c2 `serverCpu=488` / `wallgen=280.6` = 1.74）✅；4.93 = `results.txt:8` 296/60.1 = 4.93 ✅；**1.27 是 b1 的推算**（`b1-choreography.md:217` 23×55.4/999），非实测 | ⚠️ 部分核到（1.27 未标注为推算） |
| ct1 双跑 **234s vs 283s（±20%）** | `results.txt:1`（c1-r1 sec=234，**无 `-Pchunktime`**）/ `:7`（ct1-r1 sec=283，**带 `-Pchunktime=1`**） | ❌ **矛盾**（见 C2） |
| **在飞接管段 max=1（新增直证）** | 覆盖后重跑的 `logs/ct1-r1.log:107-127`（12 个报告窗，n=256→1792）：每行末 `inflight max=1`，`now` ∈ {0,1}；同期 23 个 Worker-Main、已处理 1513 chunks（`:128`） | ✅ **核到**（我独立读到原文；b1 反证条件 3「≥2 线程同时在 `fillChunk` 内 ⇒ 候选死」据此**未触发**） |
| vanilla 池 23→4：**52s→~100s** | `logs/rate-vanilla-pool4.log:111`：`Total time: 0:01:16`（**76s**），`:103-110` rate 52.6→64.3 cps | ❌ **量级不符**（76s ≠ ~100s；方向「变慢」成立）。`avgCores=2.66` 未落盘（仅探针 stdout）→ 未核到 |
| `-PcoreswapThreads=1` = 279s | `results.txt:2`（c2-r1 sec=279）+ `logs/c2-r1.log:165`（`Total time: 0:04:39`） | ✅ 核到（但语义归属错，见 C3） |
| R1 268→234（−13%） | 历史臂 `../hang-repro-260910/arms/server-coreswap.log:9655`（`Processed: 4225 chunks … Total time: 0:04:28` = 268s，全篇为 `[Mixin]` 行，如 `:9600-9654`）；c1 `logs/c1-r1.log:134`（`Total time: 0:03:54`）；`results.txt:1`（`mixinLines=0`） | ✅ 两侧端点核到；⚠️ 但比较**跨 dll + 跨批 + 单次对 + 带内噪声**（见 C4） |
| 历史基线 52s、4225 chunks 同口径 | `../hang-repro-260910/arms/server-vanilla.log:110`（`Total time: 0:00:52`） | ✅ 核到（5.15× = 268/52 ✅） |
| c3（CA_MIN=0）未跑完 | `logs/c3-r1.log` 止于 `:120`（53.47%，17:37:06），`results.txt` 无 c3 行 | ✅ 核到（record「未跑完」诚实） |
| c1 无 `chunks=`/`serverCpu=` 字段 | `results.txt:1` 缺两字段，但 `c1-r1.log:134` 有 4225 → 可回收 | ⚠️ 记录不全（minor） |

### 2. git 状态（三源第 ② 项）

- `.git/HEAD` = `ref: refs/heads/master`；`.git/refs/heads/master` = `ee0d46823435de48adc249ac1fa9750017e21ba0`（**审查时 HEAD**）。
- `.gitignore:95` = `/runtime/`、`:89` = `.tmp/` → **Java 侧本轮全部改动（ChunkTiming/CppBridge/2 个 mixin/1.20.1 mixin/build.gradle/mixins.json）无 git diff 源，也无 `git status` 可见性**。
- Rust 侧改动 `worldgen-core/src/worldgen_handle.rs:642-661`（`[WG-CONF]` 一次性行）在**受控树内** → git diff 可核（本会话无 shell，未核内容一致性，请主会话以 git diff 补第 ② 源）。
- **判定：构成证据缺口**。Java 侧「改了什么」目前只能靠文件本体 + record §5 自述。**建议补可核验替代证据**：对这 7 个 Java 文件 + 1 个 Rust 文件生成 **sha256 清单 + 行数**，写入 `.investigations/perf-regression-260910-04/cmd-output/`（并随交付表引用）；若能接受，同时在 record §5 表内逐行附 sha256。

### 3. 验证记录（行为等价门）

- **Rust 侧（✅ PASS）**：record §2.1 给了两个 hash（`6908dbfc…`/`115641b8…`）并与 `probe-260909-06.md:10` 的 on/off 哨兵值**逐位一致**——此项为真且可复核。`[WG-CONF]` 行为化证据在场：`logs/{c1,c2,ct1}-r1.log.err:76`（`ca_min=true … flags=3 skip_features=true skip_carver=true`）、`c3-r1.log.err:76`（`ca_min=false`）→ `-PcaMin=0` 的 env 通道确实生效（#81 意图达成）。
- **Java 侧（⚠️ Degraded 声明在场，但覆盖不足）**：instrument 默认关已核（`ChunkTiming.java:14` `ON = System.getProperty(...) != null`；`NoiseChunkGeneratorMixin.java:51` `MIXLOG` 默认关；1.20.1 同款 `:49`），未做同 seed region 逐格对拍，verdict §5.4 已诚实声明 Degraded——**该声明本身合规**。
- 但 **verdict 未继承 b2 的载荷前提**：`b2-writeback-sideeffects.md:178`（@idk-1）明确「内容等价性未逐格对拍 → REFUTES 是**条件性**的」，而 verdict §1/§2 表把「写回 REFUTED」与全部用时对比当无条件事实陈述 → 见 C6。
- **A2 臂的行为化证据不成立（新发现）**：`run_native_A2.ps1:12-13` 设 `WG_SKIP_FEATURES=1 WG_SKIP_CARVER=1`，但 `native-A2.txt:3,8,13,18` 的 `[WG-CONF]` 打印 `skip_features=false skip_carver=false`——因为该行 **读的是 `flags`（`worldgen_handle.rs:656-658`），不是实际生效分支（`:669` 用 env OR flags）**。故 A2 的「in-game 语义」只由 156.2→39.86ms 的时差佐证，不能由 CONF 行佐证。→ N1。

### 4. 置信度 / 模块边界 / 落盘契约

- 状态合法：verdict = candidate（`:3`）、b1/b2 = draft、judge = draft（`index-entry.yaml:8,12,16,20`）；**无 AI 自授 confirmed** ✅；根 `index.yaml:1162-1178` 已登记三件 ✅。
- 「静态推导 vs 实测」分离：b1/b2 各自标 Degraded/draft ✅；verdict 首行标 Full + 静态部分指向 b1 ✅。**例外**：verdict §1 把「并行度 = 1.000」写成定量事实（推理性近似，见 C2）；§2.2 把 1.27 核（推算）与 1.7 核（实测）并列（见 N5）。
- 模块边界：`.artifacts/perf-regression-260910-04/` 全目录 grep `skill|core.judge|core.fanout|core.worker|recode.|anchor.` = **0 命中** ✅ 无跨模块 skill 正文引用。
- scout 触发合规 ✅（`pipeline-cost-map.md` 存在、只落 `.investigations/`、标注越界声明）；fan-out 合规 ✅（2 个互斥候选 `.bN`，且 b1 明确作废了父候选的「依赖等待」措辞——这是 fan-out 的正确用途）。

### 5. 我独立核对的静态机制（不是只看产物自述）

| b1 主张 | 我读的一手源码 | 判定 |
|---|---|---|
| `worldgen` 单车道 | `versions/1.21.6/data/mc_src_extract/…/ServerChunkLoadingManager.java:192,196`（`SimpleConsecutiveExecutor(executor,"worldgen")` → `ChunkTaskScheduler`） | ✅ 成立 |
| `ChunkTaskScheduler` 单 entry 在飞 | `…/ChunkTaskScheduler.java:68-84`（`pollTask` 取 1 entry → `schedule` → `allOf(...).thenAccept(v -> pollTask())`） | ✅ 成立（下一个 entry 必须等本 entry 任务全完） |
| vanilla 重活异步 | `…/NoiseChunkGenerator.java:331-352`（`CompletableFuture.supplyAsync(…, Util.getMainWorkerExecutor().named("wgen_fill_noise"))`）+ `:336-349` section lock 在异步任务内 | ✅ 成立 |
| 我们同步压车道 | `runtime/1.21.6/java/…/mixin/NoiseChunkGeneratorMixin.java:91,98,102,106,108,114`（HEAD 内同步 `feedBeardifier`+`fillChunk` → `completedFuture`） | ✅ 成立 |
| `ChunkSection.lock()` 非互斥 | `NoiseChunkGenerator.java:336-349` 与我们的 `:108` 路径对照 | ✅ 方向一致（我们整段未执行 lock，属正确性面 @idk-10/@idk-17） |

---

## 补充证据复核（审查期间新增的 in-flight 直证，18:04–18:05 run）

**我独立读到的原文**（覆盖同一路径 `logs/ct1-r1.log`，文件现 128 行、生成进行到 35.81%）：`:107`（n=256 `… | inflight max=1 now=0`）、`:108`（`max=1 now=1`）、`:110-111`、`:113-114`、`:116-117`（n=1024，即主会话引用的两行）、`:119`、`:121`、`:123-124`、`:126-127`（n=1792）——**12 个报告窗、1792 个接管段样本，`inflight max` 恒为 1**，`now` 只在 {0,1} 取值。同期 23 个 Worker-Main 可用、Chunky 已处理 1513 chunks（`:128`）。仪器位置正确（`NoiseChunkGeneratorMixin.java:99-113` 的接管段首尾，`ChunkTiming.ON` 门控、每 chunk 一次原子操作，符合「测量/污染铁律」的 chunk 级非逐点要求），dll 不变（`:94` `sha256=abd7d8893d22e030…`，与旧 run 同）。

**① 是否足以把 §1 机制从「推导+算术自洽」升级为「实测直证」？**
**部分足以，且升级的是机制的上半段。** §1 实际是两个命题：
- **命题 A（并发结构）：世界生成同一时刻至多 1 个接管段在飞。** → 现在有 **直接实测**：`max=1` 在 1792 个样本、多个互不重叠的报告窗中成立；若机制为假（多车道/真并行），在 45–49ms 段长 × 1792 样本 × 23 空闲线程的条件下峰值 >1 的概率极高。**判定：A 从推导升级为实测直证（b1 反证条件 3 未触发，b1 存活）。**
- **命题 B（wall 归因）：world 的 wall ≈ 车道内同步工作之和。** → 仍由**同批同臂 Σ/wall = 92.7%** 支撑；in-flight 计数**不**判别「车道被我们的同步段占满」与「上游 ticket 喂不进来」——两种解释都会给出 `max=1`。**判定：B 维持「推断 + 同批比值证据」，保留限定（并见 ③）。**

**② 是否改变判定或必改条件？**
**判定维持 PASS-with-conditions**，不因该证据改变（它只强化 §1，未触及 C1/C3/C4/C5/C6/C8）。条件的**内容**有两处调整：
- **C2 放宽一半**：「同一时刻只允许 1 个接管段」不再需要 Σ≈wall 去间接证明 ⇒ verdict 应把 A 写成实测直证（引 `ct1-r1.log:107-127` 的 `inflight max=1`），B 保留限定措辞（§C2 已改写）。
- **C7 加重**：直证产生于**与旧 Σ 不同的构建态**（新构建含 INFLIGHT 代码），且两次 run **共用同一 tag/日志路径**——旧的 92.7% 原始行已被覆盖、新的只有 in-flight 段（生成未完成，无 wall/Σ 全量）。「证据与构建态绑定」从建议升级为必须声明（§C7）。

**③ 仍需保留的降级措辞（逐条）**
1. **Σ/wall 跨 run 摆动不得消除**：c1 234s / 旧 ct1 283s / 新 run 未完成；同批同臂可写 92.7%，**不得**写「并行度精确 1.000」或「wall 就是结构下限本身」。
2. **per-chunk 数字必须带 run 标识**：同一 tag 下已出现四组不同读数——旧 run `mixin=51.00`（n=1024）、旧 run 末值 `54.50`（n=4608，`ct1-r1.log` 原 `:165`）、新 run `44.45` / `48.09`（`:116-117`）；`featInterval` 相应为 **1422.7ms（旧）/ 1275.2 / 1272.1ms（新）**，vanilla 同尺子 299ms（`vt1-r1.log:124`）。新 run 的 mixin 均值（~46.5ms）比旧 run（~52ms）低 ~10–12%，说明**跨 run 的每 chunk 工作量本身在漂**——这正是「Σ≈wall 的比值可用、绝对值不可跨批」的理由。
3. **「车道饱和」vs「上游限流」未分离**：需 b1 建议测量 2（车道 busy/wall 累计 + `inFlight` 的 entry 级量）或建议测量 3（`worldgen-queue-size` / `loaders` 积压）才能把 §1 的因果句写成断言（现只能写「与车道饱和相容」）。
4. **新证据暂不可用于 wall/Σ 复算**：该 run 到 18:05:56 仅 35.81%（`:128`），**未完成**；且旧 run 的 Σ 行已不在盘上 ⇒ 只能引用 `inflight max=1` 这一结构性结论，不得引用其 wall。
5. **新增代码的行为中性未被独立验证**：`inflightEnter/Exit` 在 `if (!ON) return`（`ChunkTiming.java:32,38`）之后，CHUNKTIME 关时等价于一次布尔判断 + ThreadLocal/原子读；但本轮未对 Java 侧新增 instrument 做任何内容/sha 对拍（无 git 源，见 C7），故仍属 Degraded（verdict §5.4 的声明对此**依然适用、不因新证据豁免**）。

---

## 必改条件（C1..C8，逐条可执行）

**C1（MUST，最高优先）原始证据保全 + 关键 run 重放。**
`架构计划-260910-04 §6` 已写「关键日志在 Phase 2 结束前复制进 `.investigations/perf-regression-260910-04/cmd-output/`」——**未执行**（该目录不存在，`glob` 仅见 `record.md` / `pipeline-cost-map.md`）。且本轮审查期间 `logs/ct1-r1.log` 已被 18:03 起的新 run **第二次写入同一路径**（旧 194 行版本含 `:165` 的 Σ 行与 `:167` 的 0:04:43，现已被 128 行的新 run 内容覆盖）——即「同 tag 覆盖」在实际操作中已发生两次。
动作：① 建 `cmd-output/`，把驱动脚本（`run_arms2.ps1/probe_rate.ps1/run_native_A*.ps1/rcon_one.py`）+ 全部臂日志 + `results.txt` + `native-A/A2.txt` 复制入内并出 sha256 清单；② 需要重跑臂时用**新 tag**（如 `ct2-r1`），禁止覆盖已裁决引用的 tag；③ 在任何结论里引用 [CHUNKTIME] 行时，引 `cmd-output/` 内的归档副本路径。

**C2（MUST）±20% 摆动的口径修正 + 「并行度」命题的两段拆分。**
事实：`results.txt:3-7` 显示 ct1 只有 **1 次完整 run**（283s）+ 2 次 `FAIL_no_done` + 1 次未完成（wallgen 190.5 / 110.1）；234s 属 **c1-r1（仪器关）**。故 verdict §4 `:48`「ct1 同配置双跑 234s vs 283s」= 错标。
动作：① 改写为「c1-r1（仪器关）= 234s 与 ct1-r1（旧构建态、`-Dcoreswap.chunktime=1`）= 283s 差异 20%，仪器开销与机器噪声**未分离**」；② 把 §1 的「并行度 = 1.000」**拆成两段**表述（新证据后不再是对半开）：
   - **并发结构（已有直证，可写实测）**：`inflight max=1`（`ct1-r1.log:107-127`，n=256→1792 共 12 窗）⇒「同一时刻至多 1 个接管段在飞」= **实测事实**；
   - **wall 归因（仍为推断，须保留限定）**：「同批同臂 Σ/wall = 92.7%（Σ 为下界：末次 report 早于进程结束约 11s；`ct1-r1.log:165` 17:58:36 vs `:167` 17:58:47）⇒ 车道内已测串行工作解释 ≥9 成 wall，并发度 ≈1」；且注明「Σ 来自旧构建态 run，in-flight 直证来自新构建态 run，两者不可混算」（见 C7）。
③ 明确写出「跨批不可互推」的算术：把旧 ct1 的 Σ(269.1s) 对 c1 的 wall(234s) 会得到 **Σ/wall > 100% 的不可能值** ⇒ 两批读数不可互相代入，`b1:16` 的「4225×55.4ms=234.06s≈实测 234s（差 0.04%）」是**跨 run 混算**（per-chunk 均值来自 ct1 臂，234s 来自 c1 臂），应删除或改写为同臂算式。
④ 同时声明 Σ 的计数基准：`n=4608 ≠ Chunky 请求 4225`（`ct1-r1.log:165` vs `:167`），carve/feat/FEAT_N 各为独立计数（`n=4415`）→ 表中加注（见 N4）。
⑤ **不因新证据放宽的部分**：`inflight max=1` 对「车道被我们的同步段占满」与「上游（ticket/Chunky 请求）喂不进来」**两种解释同时成立**，故不能单独用它断言「车道饱和是 wall 成因」——该二分仍待 b1 建议测量 2（车道 busy/wall）或建议测量 3（队列深度/loaders 积压），见 N2。

**C3（MUST）「池大小不敏感（对我们）」的证据错配。**
`build.gradle:127` `-PcoreswapThreads` → `-Dcoreswap.threads`（Rust 每调用线程数，见 `CppBridge.java:40-45`；`[WG-CONF]` 打 `coreswap_threads=(unset)` 正因它走 Java property 而非 env）；Java worker 池是 `:133` `-PmaxBgThreads → -Dmax.bg.threads`。而 coreswap 的池臂 `logs/rate-coreswap-pool4.log` **仅 68 行、止于 17:49:02 启动段**（被并发 run 抢占，record 已诚实记「未取得」）。
动作：① 删除/改写「池大小不敏感（对我们）→线程 spawn/池容量都不是限流点」为「Rust 侧每调用线程数不是限流点（c2 279s，反证成立）」；② 若要保留「池不敏感」主张，必须补跑 coreswap `-PmaxBgThreads=1/4/23`（这正是 `b1:264` 反证条件 4 的判据，当前未执行）；③ vanilla 侧读数改写为 raw 值 **76s（`rate-vanilla-pool4.log:111`）**，2.66 核未落盘则删或补采。

**C4（MUST）R1 的「−13%」不得作为已测得的量值。**
比较链同时跨 **dll**（历史臂 `5e30187a…`（架构计划 §0 V1）→ 本轮 `abd7d889…`（`results.txt:1` 的 `dllsha=`））、跨批、单次对；而本轮自测摆动 ±20%（c1 234 / c2 279 / ct1 283），项目既有噪声带为 **±10%**（`probe-260909-06.md:68`），架构计划 §6 自己也写「差值 < 噪声带时不得作排除结论（#51）」。
动作：改写为「方向确定（每 chunk 2 行同步 log 已门控：历史臂全篇 [Mixin] 行 → `mixinLines=0`，且该成本在生产 jar 同样存在），量级 −13% **未与噪声带分离**，需同批 ABBA 配对 ≥3 对复现才可定量」。

**C5（MUST）§15.4 取代链落盘（双指针）。**
verdict §4 `:46` 只用一句内嵌「口径修正」处理了 `chunkradius=56`；`NEXT_SESSION`/`run_arm.ps1:2`/`架构计划 §0 V2` 的表述仍未被取代记录覆盖。
动作：为 ①「chunkradius=56（=4225）」口径（载体：架构计划 §0 V2、`run_arm.ps1:2`、`run_batch.ps1:96`、NEXT_SESSION）与 ② NEXT_SESSION「每 chunk 10 线程 spawn 超额订阅」嫌疑（被 `results.txt:2` c2 反证）各落一条 **supersedes 双指针**（新条目 + 一行推翻理由，原文不改，载体建议 artifacts/index-entry 或 knowledge 载体）。
附带：改述 `:46` 中「回显 `Incorrect argument`」——`.tmp` 全库 grep 无此串，**该回显无落盘证据**；可改为「本版 Chunky 无 `chunkradius` 子命令，命令未生效；7 臂日志的 `Processed: 4225 chunks` 逐臂核对一致（`arms/server-coreswap.log:9655`、`arms/server-vanilla.log:110`、`c1-r1.log:134`、`c2-r1.log:165`、`ct1-r1.log:167`、`vt1-r1.log:125`、`rate-vanilla-pool4.log:111`）」——后者才是可核实的判据。

**C6（MUST）继承 b2 的载荷前提（内容等价性）。**
verdict §1/§2 需显式写明：本结论假定「两臂同 seed/region 内容等价」，而该前提**在本 region 未逐格对拍**（`b2:178` @idk-1）⇒ 写回 REFUTED 与全部用时对比均为**条件性**。或在 candidate→confirmed 前补同 seed region 逐格对拍（b2 §5 测量 4）。verdict §5.4 目前只覆盖 Java instrument 的 Degraded，未覆盖这条载荷前提。

**C7（MUST）快照漂移固定（三源第 ② 项的可复核化）——新证据后从「建议」升级为「必须」。**
审查期间发现 **worktree 先于判据变更**：`ChunkTiming.java:27-40,93` 现有 `INFLIGHT/MAX_INFLIGHT` 与报告后缀 `| inflight max=%d now=%d`，`NoiseChunkGeneratorMixin.java:101,112` 调用 `inflightEnter/Exit()`。旧构建态的所有 `[CHUNKTIME]` 行**没有**该后缀（c1/c2/旧 ct1/vt1 全篇），新构建态的 run 行**有**（`ct1-r1.log:107-127`）⇒ **同一 `ct1-r1` tag / 同一日志路径下混存了两个不同构建态的证据，且旧态原始行已被覆盖**。此外 HEAD 期间未变（`ee0d468…`），即漂移全部发生在 gitignore 区（`/runtime/`、`.tmp/`）。
动作：① 记录并声明「每条数字对应哪一次构建」：dll sha（旧/新 run 均为 `abd7d889…`）+ Java jar/class 构建时刻 + 源码 sha256 清单；② 给 Rust 侧补 git diff 证据（worldgen-core 受控）；③ Java 侧补 C1 的 sha256 清单（`/runtime/` 被 gitignore，无 git 源，属**已确认的证据缺口**，需显式写入 verdict 的诚实边界）；④ 今后重跑必须新 tag（`ct2-r1`…），禁止复用已引用 tag 的路径。

**C8（MUST）evidence saturation / 失败轮声明。**
架构计划 §3 T2.5.3 要求饱和计数与声明，本轮 record/verdict 均缺。
动作：补一行「饱和计数 = 0 触发（每轮均有新数据层证据：native 四臂 / in-game 六臂 / 日志门控对照 / 池缩放探针）；失败轮 = ct1 FAIL_no_done ×2（`results.txt:3,4`）+ c3 未跑完（`c3-r1.log:120`）+ rate-coreswap-pool4 抢占中止（68 行）——属编排/工程失败，按 §9.4 不计数」。

---

## 建议项（N1..N8）

- **N1**：`[WG-CONF]` 的 `skip_features/skip_carver/skip_surface` 打的是 `flags`（`worldgen_handle.rs:656-658`），与实际生效分支（`:669` 的 `flags | WG_SKIP_*` env）不同源——A2 臂打印 `false` 而 env 为 1，CONF 行因此**不能**作为 skip 类 A/B 的行为化证据（#81 意图半落空）。建议改打「有效值」或附 env 名。
- **N2**：b1 建议测量 2 的**另一半**待补：in-flight 计数已落地并有直证（`ct1-r1.log:107-127`，`max=1`），但仍缺「车道 busy 累计 / wall」与 entry 级 in-flight ⇒ 尚不能区分「车道饱和 vs 上游 ticket 限流」（`ThrottledChunkTaskScheduler maxConcurrentChunks=4`，b1 @idk / verdict §5.2）。建议一并加 `ChunkLoader.run()` 门控累计。
- **N3**：补 vanilla 单线程每 chunk 成本（b1 建议测量 6）——这是「两臂同工作量、只差并发度」的直接对照，能把「Rust 慢」彻底排除到无需推算。
- **N4**：`[CHUNKTIME]` 表的计数基准声明：`n`（mixin 次数，含邻域 chunk）、`FEAT_N`、carve 计数三者不同基准，且 Σ 末值早于进程结束 ~11s；建议表注逐列写基准。
- **N5**：「1.27 核」标注为推算（`b1:217`），与实测 1.7 核（`results.txt:2`）分行写；「1.27-1.7」合并写法会误导为两次测量。
- **N6**：`results.txt:1`（c1）缺 `chunks=`/`serverCpu=` 字段（可从 `c1-r1.log:134` 回收 4225；CPU 未采）；驱动脚本字段补齐，避免"同口径逐臂核对"在表格上落空。
- **N7**：错误台账 E1–E5（record §6）与 §9.4/§9.7 声明在交付前并入落盘载体（现在只存在于 record/chat）；按知识库更新触发点走 subagent 草稿 + 主会话应用。
- **N8**：R3（异步化）实施前预置验收判据：同 seed region 逐格对拍 + hash 哨兵 + 端到端 wall ↔ vanilla 同阶；并显式接管 `writeChunk` 的 **section 锁语义**（vanilla `NoiseChunkGenerator.java:336-349` 在池线程内 lock/unlock；我方 `CppBridge.java` 写回不持锁 = scout @idk-17 / b2 @idk-10 的正确性面）。

---

## 独立复算/复核（实际读到的原始行引用）

1. `Σ/wall`：`logs/ct1-r1.log:165`（251.1+5.7+12.3=269.1s）+ `:167`（0:04:43=283s）+ `results.txt:7`（wallgen=290.3）→ **92.7%**（与 verdict 一致；同批同臂 ✅）。
2. `Σ/wall`（vanilla）：`vt1-r1.log:124`（3.9+13.1=17.0s）+ `:125`（0:00:53）+ `results.txt:8`（wallgen=60.1）→ **28.3%**。
3. 核数复算：vanilla 296/60.1 = **4.93**；coreswap(c2) 488/280.6 = **1.74** → verdict 的 4.93 精确复现，1.7 复现。
4. 线程时间比复算：1422.69/298.75 = **4.76**；wall 比 283/53 = **5.34** → 「同阶」成立。
5. hash 哨兵复核：`native-A.txt:54,108` 与 `probe-260909-06.md:10` 两值逐位一致 ✅。
6. native 复算：`native-A2.txt:4,14` 39.86 vs 38.55（差 1.31ms）；实机 jni 47.15（`ct1-r1.log:165` 的 jni 均值）− 39.86 = **7.29ms** 归 JNI 边界（verdict 的 7.3ms 自洽）。
7. 历史基线复算：`arms/server-coreswap.log:9655` = 268s（4:28）、`arms/server-vanilla.log:110` = 52s → 5.15× ✅；且历史 coreswap 全篇 `[Mixin]` 行（`:9600-9654`）+ `results.txt:1` 的 `mixinLines=0` → D1 前提成立。
8. 机制静态复核（§三源核对-5）全部与 b1 一致。
9. 仪器/映射复核：`build.gradle:130,131,133,127,226-237`、`coreswap.mixins.json:8`、`ChunkTiming.java:14`、1.21.6/1.20.1 mixin 的 MIXLOG 4 点 → record §5 改动清单**逐条可对**。
10. 反证项复核：c2（`results.txt:2`）证明「Rust 线程 spawn 假设」被反，但**不能**证明「Java 池不敏感」（见 C3）。
11. **in-flight 直证复核（新构建态 run）**：`logs/ct1-r1.log:107,108,110,111,113,114,116,117,119,121,123,124,126,127` 共 12+ 个报告窗，`inflight max=1` 恒成立、`now ∈ {0,1}`；同期 `:128`（18:05:56）已处理 1513/4225 chunks、23 个 Worker-Main 可用、dll sha 未变（`:94` `abd7d889…`）→ **b1 反证条件 3（≥2 线程同时在 `fillChunk`/JNI 内）未触发**；`mixin=44.45→49.36ms`、`featInterval≈1260–1326ms`、`sumMixin` 到 n=1792 为 88.4s（`：127`，中间态，**不可**用于 wall 复算）。

---

## 未核项与诚实边界

1. **本审查是移动快照**：审查期间 worktree（`ChunkTiming` 新增 INFLIGHT、mixin 调用点）与 `.tmp` 日志均在变动——`logs/ct1-r1.log` 先被 18:03 起的**新构建态 run** 覆盖（旧 194 行 → 97 行），随后同一路径继续增长到 128 行（18:05:56 仍在 35.81%，**该 run 未完成**）。文中 `ct1-r1.log:165/:167`、`vt1-r1.log:124/:125` 是我在**覆盖前**读到的旧构建态原文，现已**不在盘上**；`ct1-r1.log:107-127` 是覆盖后新构建态原文（`inflight max=1`）。两者分属不同构建态、且共用同一 tag 路径——这正是 C1/C7 的理由。
2. **无 shell**：不能跑 `git status/diff`、不能算 sha256、不能读 WMI/进程态。第 ② 源（git HEAD + worktree diff）只完成到「HEAD = ee0d468…、`/runtime/` 与 `.tmp/` 被 ignore、Rust 改动在受控树内」的判断；**Java 侧 diff 与 Rust 文件内容一致性未核**，请主会话补。
3. **未核到 raw 的项**：① `[CHUNKTIME] n=1024` 行（record §2.4 引用的 `mixin=51.00 gap=948.16 carve=1.32 feat=3.09 sumMixin=52.2s`）——原始行已随覆盖消失，我只见到同 run 后段的 n=4608 行；② record §2.4「两次独立 run 一致」——只有 1 次完整 ct1 run；③ `avgCores=2.66`（rate-vanilla-pool4）无落盘；④ Chunky 源码未读，`radius 500 方块 → 65×65=4225` 的映射只由「7 臂日志一致 + 算术自洽」支持，非原厂证据；⑤ `ThrottledChunkTaskScheduler` 上游限流未排除（b1 已自认）。
4. **`native-A2.txt` 的 `[WG-CONF] skip_*=false`** 是我发现的**证据面不一致**（不影响 39.86ms 的时差结论，但影响「行为化证据」的适用范围，见 N1）。
5. 我未修改任何被审文件；未写 `knowledge/`、`docs/`；本文件是本次审查的唯一写入。
