# 260910-05 记录（C7 收口 + nether/end 异步化推广）

> 区块标签：260910-05（实际 2026-09-10 18:4x 起，Get-Date 实测）。日期锚：`Get-Date` 2026-09-10 18:45。
> 架构：`.investigations/000-架构设计/架构计划-260910-05-c7收口.md`（HOOK-1 用户批准范围 B + 附加项 = runtime Java 快照归档）。
> 上游：260910-04 confirmed（机制=单车道同步接管；R3=异步化 42s vs 279s）。

## 1. 开工前核验（主会话，已完成）

### 1.1 交接声明核验（廉价独立验证，AGENTS.md §三 交接结论验证纪律）
- R3 落地直证：`runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`（mtime 2026-09-10 18:11:39）`:61 SYNCFILL` / `:119-144 work` / `:145-149 sync|async 分支`；`ChunkTiming.java:28-39 inflight`。✅
- 驱动脚本：`.tmp/perf-reg-260910-04/run_arms2.ps1`（18:12）+ 归档副本 `cmd-output/run_arms2.ps1:25-26` 均含 r3/r3sync 臂。✅
- 执行体三元组（R1 风险闭合）：`target\release\worldgen1216.dll` 与 `.tmp\java-tmp\coreswap-native\worldgen.dll` **sha 前 16 位同为 `abd7d8893d22e030`**，与归档日志 `dllsha=abd7d8893d22e030` 一致。✅
- 归档 C5 条件：`vanilla1216-r1.log(.err)`、`r3-async-r1.log.err`、`r3sync-r1.log`、`diff-result-historical-260910-03.txt`、region 三目录均在位。✅
- nether/end 句柄在 1.21.6 可用：归档 `r3-async-r1.log:98-99` `initNether/initEnd enabled=true stageMask=3`。✅

### 1.2 🔴 前提核验（改变 C7-① 做法）
**静态链（全部带引证）**：
1. `CppBridge.java:69-75 resolveStageMask()`：`prop==null ⇒ 0b011`（默认 mask=3 = SKIP_CARVER|SKIP_FEATURES）。
2. 两 R3 臂运行时就地证据：`cmd-output/r3-async-r1.log.err:76` 与 `r3sync-r1.log.err:76` 均 `flags=3 skip_features=true skip_carver=true skip_surface=false`。
3. `worldgen-core/src/worldgen_handle.rs:669-671`：`let skip_features = flags & FLAG_SKIP_FEATURES != 0 || env WG_SKIP_FEATURES; if !skip_features { self.apply_features(...); }`。
4. `pending_cross_writes` 生产点 `:1086-1091` 与消费点 `:1051-1063` **都在 `apply_features` 内**（全仓 grep：只有 `:144/:506/:1053/:1088` 四处）。

⇒ **判定（静态）**：默认出货语义（vivo 亦 `stageMask=3`，见 workflow-patterns #66）下 `apply_features` 不执行 ⇒ `pending_cross_writes` **结构不可达** ⇒ judge C7-① 的「写丢失」竞争假说在当前语义下是**空集**。
**行为化直证（本轮执行）**：`r3log-r2`（async、mask=3、`WG_CA_LOG=1`）应产出**零 `[CA]` 行**；正向对照两路——① 历史同载体 `calog-e2-260909-04.err` **257 行 `[CA]`**（features ON 臂，`:CA` 行如 `chunk(0,0) out_reads=10385 pending_writes=1594 placed=108`）；② 本轮 `r3feat-r2`（mask=5、features ON、`WG_CA_LOG=1`）应产出 `[CA]` 行。
**设计后果**：不先加会计计数（避免造不可执行路径）；会计降级为**条件项**，仅当行为化核验推翻静态链时启用。

### 1.3 工具坑（本块新增候选发现，待 K1 落库）
- `scripts/merge_index.py` 写回根 `.artifacts/index.yaml` 时**只保留 `{id,path,kind,status}` 四字段**（`merge_index.py:131-138`）⇒ **根索引里的注释/长说明会被整体丢弃**（本仓库根 index.yaml 大量使用注释承载结论摘要）⇒ 有注释的根索引 **MUST NOT** 直接跑该工具做合并（或跑后手工恢复注释）；本块 P5 采用手工追加 entry。

## 2. 运行记录

### 2.1 驱动脚本
`.tmp/perf-reg-260910-05/run_arms3.ps1`（基于 260910-04 R3 版；新增维度臂 / `WG_CA_LOG` 正负对照 / `-Dcoreswap.rust.stages` 经 `JAVA_TOOL_OPTIONS` / results 行含 `args=`+`stages=`+`calog=`+`world=`+`inflightMax=`+`CALines=`）。
- 踩坑 E1：`pwsh -File ... -Arms a,b,c` 会把逗号串当**单个字符串**（`unknown arm r3,r3log,...`）→ 必须 `& script.ps1 -Arms @("a","b")`。
- 臂定义：`r3`(async 默认 mask=3) / `r3log`(同前 + WG_CA_LOG=1) / `r3feat`(mask=5 features ON + WG_CA_LOG=1) / `r3sync`(-Psyncfill=1) / `nv,ns,na`(nether vanilla|sync|async) / `ev,es,ea`(end …)。

### 2.2 overworld 臂（C7）
（运行中，结果见 `cmd-output/`）

| tag | 形态 | wall | chunks | inflight max | CALines | flags | 备注 |
|---|---|---|---|---|---|---|---|
| `r3-r2` | async 默认 | 41s | 4225 | **23** | 0 | 3 | C7-② 基线对（vs `r3-r1`，260910-04 归档 region） |
| `r3log-r2` | async + `WG_CA_LOG=1` | 39s | 4225 | 23 | **0** ✅ | 3 | C7-① **负对照成立**（mask=3 ⇒ `apply_features` 不执行） |
| `r3feat-r2` | async + mask=5 + `WG_CA_LOG=1` | 主动终止（865/4225 chunks） | 4225(未跑满) | 23 | **1711** ✅ | **5** | C7-① **正对照**（>0 行；`flags=5` + JVM banner 双通道证明） |
| `r3sync-r2` | sync（`-Psyncfill=1`） | 243s | 4225 | **1** | 0 | 3 | C7-② sync 自身基线对（vs `r3sync-r1`）+ 幅度第二读数 |

- 脚本缺陷 E2（**根因按未核声明**，judge C5/FIN-C1）：本批 **13 行归档表 12 行** `serverCpu=-1 / cores=-1`，仅 `ea-r2`（86 CPU·s）与补跑 `eaS-r1`（99 CPU·s）**两臂**采到。**两轮 judge 各读到一版脚本**（C7 judge 见 `:90-91` 停服前采样 + `:112-113` 停服后重复采样；J3/J4 judge 见现版——`:90-91` 停服前采样、`:112` 为「不得重复采样」注释、`:113` 算 cores，**无重复块**），且 `.tmp` 与 `cmd-output` 两份归档副本均为**修后**版本 ⇒ 预修版本**文件级不可复核**，故**根因（重复采样覆盖）标注为未核**；行为侧旁证 = 中途修复后采到 CPU 的是**两臂**（`ea-r2`/`eaS-r1`）。**不再使用「脚本已修 ⇒ 已解决」的因果表述**。幅度口径矩阵改用 wall/wallgen（6 组组合），CPU 证据引 260910-04 归档值（`r3sync-r1=375 CPU-s / r3-r1=543 CPU-s`）。

### 2.2.1 C7-① 行为化核验结论（正负对照齐备）

| 判据 | 臂 | 证据（就地） | 结论 |
|---|---|---|---|
| 负对照：mask=3 ⇒ `apply_features` 不执行 | `r3log-r2`（`WG_CA_LOG=1`） | `[CA]` 行 = **0**；`[WG-CONF] flags=3 skip_features=true` | ✅ 零行 |
| 正对照：mask=5 ⇒ 路径可达 | `r3feat-r2`（`WG_CA_LOG=1`） | `[WG-CONF] flags=5 skip_features=false skip_carver=true skip_surface=true`；`[CA]` 行终态 **1711**（`chunk(1,-10) out_reads=23734 all_reads=6351100 pending_writes=1221 placed=111`）；`[Mixin] placedFeature skipped (rust takeover) count=36864` | ✅ >0 行 |
| 通道证明（`-D` 真送达 / env 真送达） | `r3feat-r2` | 同一臂同时证明 ① `-Dcoreswap.rust.stages=5`（经 `JAVA_TOOL_OPTIONS`）生效（`stageMask=5` 打印）② `WG_CA_LOG` env 生效（`[CA]` 行） | ✅ 双通道到位 |

⇒ **C7-① 收口：judge 的「跨 chunk 写序 → 写丢失」竞争假说在当前出货语义（`resolveStageMask()` 默认 `0b011` = skip_features）下为「空集」**——`pending_cross_writes` 生产点/消费点/邻域快照预取全在 `apply_features` 内，而该函数唯一调用点被 `FLAG_SKIP_FEATURES` 门死（静态链 + 行为化直证 + 双对照）。
⇒ **前置条件登记（未来重启 Rust features 接管 MUST 先做）**：mask=5 臂实测 `pending_writes` **量级 = 10³–10⁴**（全局 atomic 在 23 线程并发下每 chunk 重置 ⇒ **只作量级/上界**，**不得**读作 per-chunk 精确值；一手分布 = n=1711 / 区间 0-37,893 / 中位 1,575）⇒ 该路径一旦启用即**真实承载大量跨 chunk 写**；会计点须新增「迟到写（目标已跑 features）/ 已生效 / 常驻」三数（现成 `CA_PENDING_WRITES` 只计入缓冲次数，无法区分，见 scout §2.2 + 待验 2）。

### 2.3 scout（P1a，subagent 只读）结论摘要
产物：`.investigations/perf-closeout-260910-05/scout-vanilla-crosswrite.md`（253 行，全部 `文件:行` 引证）。
1. **vanilla 跨 chunk 写 = write-through + 立即持久**（`ChunkRegion.java:274-310`；Ore 走真实邻 chunk section + per-section 锁 `OreFeature.java:137-147`/`ChunkSectionCache.java:26-62`），写半径 = 1（`ChunkGenerationSteps.java:27`），超半径 = logErrorOrPause + 拒写（`ChunkRegion.java:242-271`）。
2. **vanilla 无延迟写缓冲** ⇒ Rust `pending_cross_writes` 是 **lossy 语义分歧**（4 面：drop / 可见性缺失 / 顺序反转 / 常驻条目陈旧注入；Rust 自认 IDK-cA2 `worldgen_handle.rs:142-144`）。**但**：vanilla 自身也有吞写窗口（目标已 FULL → `WrapperProtoChunk(...,false)` 静默吞写，`WrapperProtoChunk.java:74-78` + `ChunkGenerating.java:186`）⇒ 分歧点在**窗口位置与可见性**，而非「有无丢失」。
3. **mask=3 下结构不可达（支持静态结论）**：生产/消费点全在 `apply_features` 内；`neighbor_terrain` 两个调用点（`:1124/:1242`）也在其中 ⇒ 连邻域读快照预取都不执行；mask=3 下 fill 路径唯一共享写 = `terrain_cache` upsert（`:631-635`，纯 memo）；`api.rs` 无任何跨写投递通道。
4. **未来重启 bit1 前的内容敏感面清单**（A1 跨写投递[架构级] / A2 越界读可见性 / A3 越界高度图哨兵 / A4 目标 heightmap 未更新 / A5 写半径语义 / A6 无两阶段区域列 / A7 3×3 biome 近似）+ 内容中性面（三处缓存淘汰、HashMap 迭代序、beardifiers、诊断计数）；C++ 历史形态是 regionCols + 串行 phase2（不丢）。
- 关键提醒（对 C7-① 会计路线的增量）：**现有 `CA_PENDING_WRITES` 只计「入缓冲次数」，无法区分「已生效 / 永丢」** ⇒ 若行为化核验推翻静态链、进入会计路线，必须新增「迟到写（目标已跑 features）」计数点，而不是复用现成计数。

## 3. C7-② 对拍结果（工具 `cmd-output/diff_arms.py`，与 260910-04 同修订副本）

| 对 | 类型 | diff | % | sections same/diff |
|---|---|---|---|---|
| `r3sync-r1 × r3sync-r2` | 同形态 sync run-to-run | 32,169 | **0.0086%** | 91352/331 |
| `r3-r1 × r3-r2` | 同形态 async run-to-run | 34,459 | **0.0092%** | 91294/391 |
| `r3-r1 × r3sync-r1`（既有） | 跨形态 | 45,948 | 0.0122% | 91289/397 |
| `r3-r2 × r3sync-r1`（本轮） | 跨形态 | 49,000 | 0.0130% | 91267/419 |
| `r3-r1 × vanilla`（既有） | 跨实现 | 44,921 | 0.0120% | 91176/512 |
| `r3-r2 × vanilla`（本轮） | 跨实现 | 42,867 | 0.0114% | 91155/532 |
| `r3sync-r1 × vanilla`（既有） | 跨实现 | 41,809 | 0.0111% | 91154/533 |
| `r3sync-r2 × vanilla`（本轮） | 跨实现 | 55,761 | 0.0148% | 91206/482 |
| 历史代理基线（R3 前跨实现） | — | 56,214 | 0.0150% | 91191/496 |

- 全部 `common=7959` chunks（三臂全域普查；`blocks` 行因 section 并集口径逐对略有差异）。
- **读数**：同配置 run-to-run 噪声 **0.0086%/0.0092%**（sync/async 同量级）；跨形态 0.0111-0.0130%；跨实现 0.0111-0.0148%；全部 ≤ 历史代理基线 0.0150%。
- **对 judge C2 预期的实测回答（方向已按 judge C3 更正）**：async 点估计**略高于** sync（0.0092% vs 0.0086%，相对 +7%），方向与 #67 预期一致；但每形态仅 1 对读数、差幅远小于项目 ±20% 摆动带 ⇒ **既未证实也未否证**「异步自身非确定更大」，只能判**同阶**。跨形态高于同形态基线约 0.004pp ⇒ 存在小的**未归因**形态相关分量（假说：填充序影响 vanilla 结构/装饰放置），低于同批跨实现最高读数。
- 产物：`cmd-output/diff-*.txt`（5 份）；region 目录 `cmd-output/region-*`（5 组 ×16 mca）。

## 4. 幅度（C7-③）
`r3-r1=42s / r3-r2=41s / r3log-r2=39s`（async）vs `r3sync-r1=279s / r3sync-r2=243s`（sync）；vanilla 52s。
⇒ **范围 5.8×–7.2×**（6 组组合：5.79/5.93/6.23/6.64/6.80/7.15）；相对 vanilla 快 1.24-1.33×；`inflight max` 23（async×3）vs **1**（sync×2）。
- 未做 ABBA ≥3 对（HOOK-1 未选）；CPU 本轮因脚本 E2 缺失（引 260910-04 归档 543/375 CPU-s）。

## 5. P4 nether/end 异步化（进行中）

### 5.1 代码改动（已应用 + 编译通过）
- `NoiseChunkGeneratorMixin.java`：nether/end 两分支改为与 overworld R3（`:112-150`）同构（`Supplier<Chunk> work` + `print+rethrow` + `ChunkTiming` 包裹 + 共用 `SYNCFILL` 开关 + `supplyAsync(work, WG_FILL_POOL)`）。
- `CppBridge.java`：新增 `MIXLOG`（`coreswap.mixlog`），两处每 chunk `[WG-FILL]` 读回自证行改门控（R1 同族拉平；原为无条件每 chunk 一行）。
- 编译：`gradle compileJava` **BUILD SUCCESSFUL**（23s，仅既有 deprecation 提示）。
- 快照：`java-snapshot/pre|post`（pre 4 文件 sha 见 §1.1；post 见 `post/SHA256SUMS.txt`；diff 行数 = mixin 69 / CppBridge 10，ChunkTiming 与 build.gradle UNCHANGED）⇒ **R2（runtime/ 无 VCS 源）已闭合**。
- Rust 侧零改动 ⇒ 无需重编 dll（执行体与 260910-04 同源，`dllsha=abd7d8893d22e030` 每臂核对）。

### 5.2 P4a 载具首用 sanity（`naS-r1`，async + `-Pmixlog=1`）
- ✅ `chunky world minecraft:the_nether` 被接受；`Task finished for minecraft:the_nether. Processed: 4225 chunks (100.00%), Total time: 0:00:21`。
- ✅ `populateNoise(nether) intercepted` = **4761** 行（新异步分支生效）+ `[WG-FILL]` = **4761** 行（Rust 写回生效）。
- ✅ region 采集：`run\world\DIM-1\region` 16 mca → `cmd-output\region-naS-r1`（驱动脚本 `reg="DIM-1\region"` 路径规则正确）。
- 结论：**载具首用风险 R6（维度名/region 路径/接管生效）已排除**，可进 A/B。

### 5.3 A/B 矩阵（已完成）
| 臂 | 维度 | 形态 | Chunky Total | wallgen | inflight max | 备注 |
|---|---|---|---|---|---|---|
| `nv-r1` | nether | vanilla | 56s | 60.0 | — | 跨实现参照 |
| `ns-r1` | nether | sync | 69s | 70.1 | **1** | 单车道形态 |
| `na-r1` | nether | async | 21s | 30.0 | **23** | |
| `na-r2` | nether | async | 20s | 30.0 | **23** | 同形态第二读数 |
| `ev-r1` | end | vanilla | 5s | 10.0 | — | |
| `es-r1` | end | sync | 18s | 20.0 | **1** | |
| `ea-r1` | end | async | 6s | 10.0 | **23** | |
| `ea-r2` | end | async | 6s | 10.0 | **23** | 本批两臂之一采到 CPU（86 CPU·s；另一臂 = 补跑 `eaS-r1` 99 CPU·s） |
| `naS-r1` | nether | async + mixlog | 21s | 30.0 | **23** | 载具 sanity（写回直证） |
| `eaS-r1` | end | async + mixlog | 7s | 20.0 | **23** | 载具 sanity（judge C3 缺口闭合；99 CPU·s / cores 4.95） |

- **形态效应（口径点明，judge C4）**：**Chunky 进程内任务计时** nether 69 → 20/21s（**3.3-3.5×**）、end 18 → 6s（**3.0×**）；**脚本侧 wallgen** nether 70.1 → 30.0（**2.34×**）、end 20.0 → 10.0（**2.00×**）。`wallgen − sec` 尾巴 **1.1-10s 不稳定**（启停/落盘/停服）。**禁用「端到端」措辞**（端到端 = vs Java 生产口径，本块未测）。`inflight max` 1 → **23** 两维复现（同仪器）。
- **相对 vanilla（同口径）**：Chunky 计时 nether async 快 2.7-2.8×（56 → 20/21s）、end async 6s ≈ vanilla 5s；wallgen 口径 nether 2.00×（60.0→30.0）、end 1.00×（10.0→10.0）。**sync 形态 wallgen 净亏损**（70.1>60.0；20.0>10.0）。
- 口径：每臂同 seed / 同 region 中心（-48,-11）/ 同构建态（P4 后 jar）；**7 个 coreswap 臂** `dllsha=abd7d8893d22e030`（`nv/ev` 为 vanilla 臂 `dllsha=-`）；「与 target dll 一致」本块未复算。
- **CPU（judge C5）**：`ea-r2` = **86 CPU·s / 6s**（≈14 核均值）⇒ **更多总 CPU 换更低延迟**（延迟结论，非效率结论）；nether 侧总 CPU 未测；缺失**根因未核**（见 §2.2 E2）。
- `interceptDim=0` 属预期：该计数依赖 `-Pmixlog=1`（仅 sanity 臂开），A/B 臂按设计未开日志 ⇒ 形态证据以 `inflight max` 为准。

### 5.4 维度对拍（已完成）
6 对：`N_sync_vs_vanilla` / `N_async_vs_sync` / `N_async_vs_async2` / `E_*`（同上三对）。判定规则（预登记）：
- **接管基线**（sync × vanilla）：1.21.6 nether/end 接管**首次验证**（历史 `end-takeover`/`nether-save-full` 均为 1.20.1）；若差异大 ⇒ 属既存接管分歧，不记到异步改造账面（R6 设计）。
- **形态等价门**（async × sync）：判据 = ≤ 同维度**同形态**噪声（async1 × async2）。
- 工具同 overworld 修订（可比）；其内嵌 overworld 定点段对维度无意义（输出中显示 `NOT in both arms`，声明）。

| 对 | 维度 | 类型 | common | blocks | diff | % | sections same/diff |
|---|---|---|---|---|---|---|---|
| `ns × nv` | nether | 接管基线 | 7542 | 209,965,056 | 214,240 | 0.1020% | 50555/706 |
| `na-r1 × ns` | nether | **跨形态（门）** | 7542 | 209,981,440 | 255,942 | **0.1219%** | 50508/757 |
| `na-r1 × na-r2` | nether | 同形态噪声 | 7543 | 209,981,440 | 295,191 | **0.1406%** | 50436/829 |
| `es × ev` | end | 接管基线 | 7567 | 494,010,368 | **7** | 0.0000% | 120601/7 |
| `ea-r1 × es` | end | **跨形态（门）** | 7567 | 494,055,424 | **0** | **0.0000%** | 120619/0 |
| `ea-r1 × ea-r2` | end | 同形态噪声 | 7567 | 494,100,480 | **0** | **0.0000%** | 120630/0 |

- **nether 门 PASS**：跨形态 0.1219% ≤ 同形态噪声 0.1406%（**该噪声是 async 代理**——nether 只有 1 个 sync run，无 sync run-to-run 对；judge C1）。附带读数：接管基线 0.1020% **低于**自身（async 代理）噪声 ⇒ 差异落在噪声带内（**不等于**「接管已验证」）。`na-r1 × ns` 有 1 个 chunk 单侧排除（不影响结论）。
- **end 门 PASS**：跨形态 **0 块差**、同形态噪声 **0**；**接管 vs vanilla 有 7 块差（> 同形态噪声 0）⇒ 跨实现差在该维是可检出的**（不得与零差并列）；「0 差」经四重论证（同链检出 7 块有灵敏度 / common=7567 无单侧排除 / section 并集 494,100,480 非空 / 三 region sha 互异）。「确定性」限定在 **coreswap 臂内 + 该 region 集**；`ev` 仅 1 run ⇒ 7 块差**无法**归因（删「该维不存在 run 级非确定」表述，judge C2）。
- **nether 同形态（async 代理）噪声显著高于 overworld**（0.1406% vs 0.0092%，~15×）——成因未查，登记后续项。
- 维度间不可互引（#33）：nether 与 overworld 噪声量级差 15×，各维自比自判（已写入 verdict §9.7 声明）。
- sanity 计数口径（judge C8）：`populateNoise(nether|end) intercepted` 与 `[WG-FILL]` **均为 4761 = 调用次数**（非 4225 chunks；>4225 ⇒ 存在**重复接管调用**，与 `feedBeardifier`/`fillChunk` 同 chunk 交错属同族既有风险面，登记后续项）。

## 6. 待办与状态
- [x] P0 架构批准（范围 B）
- [x] P1a scout / P1c pre+post 快照
- [x] P2 C7-① 行为化核验（空集判定成立；正负对照 + 通道证明）
- [x] P3 C7-② 基线对 + C7-③ 幅度范围
- [x] P4b/P4c nether/end A/B 行为门（8 臂 + 2 sanity；两维门 PASS）
- [x] J1/J2 judge（PASS-with-conditions，C1-C9 已应用）；J3/J4 judge（PASS-with-conditions，C1-C10 已应用）
- [x] 证据归档 + MANIFEST（251 条 sha256）
- [x] INDEX.md 脚手架残留修复（260910-04 草稿的 `## D INDEX 追加段` + 未闭合 ```markdown 围栏 + 两行说明；136→131 行，备份 `knowledge/INDEX.md.bak-260910-05`）
- [ ] P5 知识库（K1 草稿已产出：`knowledge-draft-260910-05.md`，待主会话应用）/ index.yaml / NEXT_SESSION / 提交

### 6.0 收尾审查（J5，260910-05）
- 判定 **PASS-with-conditions**，必改 9 条 **FIN-C1..C9 已全部应用**；建议项 N1/N2/N4/N5 已应用（N3 已并入 §5.4 注记）。
- 关键更正：① E2 根因全链统一为「**推断·未核**」+ 记录 C7-C9① ↔ nether-C5② 的时点冲突与取舍；② CPU 臂数 = 13 行表 12 行为 `-1`，仅 `ea-r2`（86）与 `eaS-r1`（99）两臂采到；③ `verdict-nether-end:44` 跨维串号 `7567/7543` → **`7543/7542`**（C1/E4 家族三犯）；④ eaS 证据归档 + MANIFEST 刷新 **269 条**；⑤ nether 噪声第二处口径（async 代理）；⑥ end 零差内联限定；⑦ `pending_writes` 量级口径。
- 审查文件：`.artifacts/perf-closeout-260910-05/judge-verdict-final-260910-05.md`。

### 6.2 confirmed + 测试构建（2026-09-10 19:58）
- 用户四项 confirmed：C7-①（空集）/ C7-②③（基线+幅度）/ §15.4 取代 / nether/end 推广；已写入两份 verdict 头 + 根 `index.yaml` 四条 + `index-entry.yaml` + NEXT。
- 1.21.6 测试 jar 构建：`runtime/1.21.6/java/build/libs/coreswap1216-1.21.6-0.1.0.jar`，sha256 `16d5e5e7adf0780c…`，jar 内 dll 与 target 一致（`abd7d8893d22e030…`）。
- 1.20.1 亦构建（1.0.27，三元组 MATCH），用户明确暂不处理。

### 6.1 登记后续项（本块未做，留给下轮）
1. **nether 同形态非确定 ~0.14%（vs overworld 0.0092%）成因未查**（候选：nether 结构/熔岩湖按 chunk 完成序放置、region 目录含非本臂生成 chunk）。
2. **重复接管调用**（`populatedNoise` 调用 4761 > chunks 4225）成因未查；与 `feedBeardifier`/`fillChunk` 同 chunk 交错的既有风险面同族。
3. **延迟换 CPU 代价面**：异步下单块 mixin 段膨胀（nether 10-11→25-37ms、end 3.5→20-22ms）、平均并发度 5.75/15.4 vs 峰值 23（judge N1）。
4. **`[WG-FILL]` 读回成本仍在热路径**（门控只包 println；`nzBuf` 扫描 + 16 点读回每 chunk 无条件执行）。
5. **1.21.6 nether/end 接管正确性专项**未做（本块只给单读数「接管 vs vanilla」；nether 差异落在噪声带内、end 7 块差无法归因）。
6. **C7 前置条件**：重启 Rust features 接管前先加会计三数（按 chunk/线程分桶）。
7. `knowledge/INDEX.md` 的 04 段脚手架修复属**结构性修补**（内容为 260910-04 已 confirmed 段落，非新结论），备份见 `INDEX.md.bak-260910-05`；建议下轮顺手核一遍 INDEX 全文格式（同类残留可能还有）。
8. **编号异常（既有，非本块引入）**：`knowledge/discovered/build-tooling.md:854` 存在一条 **#96**（位于 #44 与 #45 之间，文件序非单调）⇒ 该文件「数值最大号」= #96、「追加序尾号」= #48。本块新增条目按**追加序尾号**续编为 **#49/#50**（与紧邻的 #45-#48 连续、与 INDEX 段一致）；若要改为「数值最大号续编」（#97/#98），需同时改条目标题 + INDEX 段 + 根 `index.yaml` 摘要（正文零改动）。**建议下轮统一裁决编号口径**（另一处同类：workflow-patterns 有 #109 以 `###` 形态出现，`##` 头最大为 #108）。
