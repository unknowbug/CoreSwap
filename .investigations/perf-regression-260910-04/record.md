# 260910-04 · 1.21.6 性能回归定责（5.15×）——测量记录

> 起点：NEXT_SESSION 下轮开工点 1（用户拍板立项）。架构计划 = `.investigations/000-架构设计/架构计划-260910-04-1216-perf-regression.md`（用户已批准，含 R1 授权）。
> 本文件 = 过程 + 数据记录（主会话写）。结论性落盘（artifacts candidate + 知识库）另走流程。
> 日期锚：Get-Date 2026-09-10 17:01（260910-04）。

## 0. 口径（全部实测条件）

| 项 | 值 |
|---|---|
| 载体 | 1.21.6 Fabric 专用服（gradle runServer）+ Chunky 1.4.40，RCON 驱动 |
| seed | `417950215108767439`（与 260910-03 双臂基线同） |
| region | center chunk(-48,-11) + **Chunky 默认 radius 500 方块** = **4225 chunks**（⚠️ 修正：驱动里的 `chunky chunkradius 56` 在本版 Chunky **无效**（回显 `Incorrect argument`），历史基线与本轮一致走默认 → 4225 逐臂核对一致，可比；「chunkradius=56」是历史文档口径失真） |
| coreswap 臂参数 | `-PcppReplace=true -PcppLib=target\release\worldgen1216.dll -PcppWorldgenDir=versions\1.21.6\data\worldgen`，`[CppBridge] init ... enabled=true stageMask=3` |
| 执行体三元组 | dll sha256 `abd7d8893d22e030…`（本轮重编产物，size 2457088，mtime 17:06）；日志 `[CppBridge] dll=` 行核对一致 ✅ |
| 行为化配置证据（本轮新增 `[WG-CONF]`） | `ca_min=true est_l2=true ca_cap=2048 flags=3 skip_features=true skip_carver=true skip_surface=false coreswap_threads=(unset)` ✅ 实测钉死（scout @idk-13 关闭） |

## 1. 交接结论廉价验证（Phase 0 前置，§16.3）

| 项 | 方法 | 结果 |
|---|---|---|
| 5.15× 基线真实性与口径 | 读 `.tmp/hang-repro-260910/arms/*` + 驱动脚本 | ✅ 同 seed/同 region(4225)/同驱动/串行（#28 合规）；**但处理臂多 8450 行 per-chunk 同步日志 → 口径不对等（见 D1）** |
| `WG_CA_MIN` 默认 | `worldgen_handle.rs:619/1031` 直读 + `[WG-CONF]` 行为化 | ✅ 默认开（继承结论成立） |
| 执行体 | 日志 dll sha vs target 产物 | ✅ 一致（重编后新 sha） |

## 2. 关键数据（全部本轮实测）

### 2.1 A 组 native bench（`bin-diag/camin_bench`，串行、无 JNI；seed -8248…/(0,0)）
| 臂 | per_chunk | hash |
|---|---|---|
| 1.21.6 ca_min on（全阶段 flags=0） | 156.2ms | `6908dbfc…` ✅（与 260909-06 哨兵逐位同） |
| 1.21.6 ca_min off | 135.2ms | `115641b8…` ✅ |
| 1.21.6 est_l2 off | 166.0ms（**更慢** → est_l2 是净收益，非嫌疑） | ✅ 同 on |
| 1.20.1 ca_min on | 58.3ms | — |
| 1.20.1 ca_min off | 47.7ms | — |
> 结论①：**行为等价 hash 哨兵逐位不变 → 本轮 Rust 改动（`[WG-CONF]` 一行）行为中性**（#47 门 PASS）。
> 结论②：全阶段口径下 1.21.6 数据比 1.20.1 慢 2.7×——但**差异全部来自 Rust 侧 carver+features（实机 stageMask=3 不跑）**，见 A2。

### 2.2 A2 组 native bench（in-game 语义对齐：`WG_SKIP_FEATURES=1 WG_SKIP_CARVER=1`）
| 臂 | per_chunk |
|---|---|
| **1.21.6 terrain+surface（ca_min on）** | **39.86ms** |
| 1.21.6（ca_min off） | 39.43ms（ca_min 在此语义成本≈0） |
| **1.20.1 terrain+surface（ca_min on）** | **38.55ms** |
| 1.20.1（ca_min off） | 42.66ms |
> 结论③：**跨版本引擎无回归**（同一 `worldgen-core`，1.21.6 与 1.20.1 数据路径在实机语义下同速 ~39ms/chunk）。
> 结论④：**Rust 引擎不是 5.15× 的原因**——实机所跑的那部分只需 ~39ms/chunk。

### 2.3 C 组 in-game 判别矩阵（Chunky，4225 chunks，逐臂删 world）
| 臂 | 配置 | Total | cps（稳态） | 备注 |
|---|---|---|---|---|
| 历史 260910-03 | coreswap（含 per-chunk 日志） | **268s** | ≈15.8 | 基线 |
| c1 | 日志门控关（净日志） | **234s** | 17-22 | **−13%**（D1 量化） |
| c2 | c1 + `-PcoreswapThreads=1` | **279s** | 11-15 | **更慢 → 线程 spawn 假设 REFUTES** |
| c3 | c1 + `WG_CA_MIN=0` | （未跑完） | 18.4 | 与 c1 同量级 → CA_MIN 无改善 |
| vanilla（历史/本批） | `-PcppVanilla` | **52s** | 81 | 分母 |

### 2.4 仪器化分解（`-Dcoreswap.chunktime=1`，chunk 级、非逐点、默认关；两次独立 run 一致）
```
[CHUNKTIME] n=1024 perChunk(ms): mixin=51.00 [jni=47.15 write=3.50 hmap=0.22 scan=0.08 beard=0.04]
            gap=948.16 | stage carve=1.32 feat=3.09
            | sumMixin=52.2s sumGap=970.9s sumCarve=1.3s sumFeat=3.2s
```
- `mixin` = 我们的接管段（Java HEAD→返回）；`gap` = **同一 Worker-Main 线程「上次接管返回 → 本次接管开始」间隔**。
- **我们接管段 51ms + carve 1.3ms + features 3.1ms ≈ 55ms/chunk；其余 948ms（94.5%）在同线程上未被任何已测阶段占用。**
- Rust JNI 47.2ms ≈ native bench in-game 语义 39.9ms + JNI 边界（393KB 零初始化 + 拷回）→ **自洽**。
- 进程 CPU：生成期服务器进程 **~1.7 核 / 24 核**（c2 全程 488 CPU-s / 280s）。

### 2.5 池缩放探针（`-PmaxBgThreads=N` → `-Dmax.bg.threads`）
| 臂 | 池 | 速率 | 平均核数 |
|---|---|---|---|
| vanilla | 23（默认） | 81 cps（52s） | — |
| **vanilla** | **4** | **53.5→64.3 cps（~100s 完成）** | **2.66 核** |
| coreswap | 4 | （探针被并发 run 抢占，未取得） | — |
> 结论⑤：**vanilla 的 52s 本身不是 CPU 受限**（4 线程仍 60 cps、2.66 核）；两侧都是**延迟/驱动受限**的管线。因此 5.15× 的形态是「**每 chunk 延迟被放大 3-5×**」，不是「算力不够」。

## 3. 已排除项（每条带实测证据）

| 假设 | 判别实验 | 结果 |
|---|---|---|
| Rust 引擎算力/跨版本引擎回归 | A2 native bench（in-game 语义）+ 实机 jni 计时 | **REFUTES**（39.9ms native ↔ 47.2ms 实机；1.20.1 38.6ms） |
| 每 chunk 10 线程 spawn 超额订阅 | c2：`-PcoreswapThreads=1` | **REFUTES**（279s，比 c1 更慢 19%） |
| per-chunk mixin 日志（8450 行同步 log4j） | c1：日志门控关 | **部分成立：仅 13%**（268→234s）——仍作为 R1 修复保留（生产也存在） |
| CA_MIN 全局 Mutex/深拷贝/clear-all | A2（同语义成本≈0）+ c3（无改善） | **REFUTES**（在 stageMask=3 语义下 CA_MIN 工作量极小） |
| est_l2 全局 Mutex | A 组 est_l2 off 更慢 | **REFUTES**（净收益） |
| CARVERS 阶段（含 sampler 惰性创建） | ct1 carve 计时 | **REFUTES**（1.3ms/chunk） |
| FEATURES 阶段 | ct1 feat 计时 | **REFUTES**（3.1ms/chunk） |

## 4. 未闭合（本轮核心遗留）

**948ms/chunk 的「段外等待」未被归因**：它是 Worker-Main 线程的**非 CPU 等待**（进程仅 1.7 核），既不在我们的接管段，也不在 carve/features。
候选（fan-out 并行中，见 `.artifacts/perf-regression-260910-04/candidates/`）：
- **b1 编排候选**：vanilla 的 `populateNoise` 返回 future（异步，`Util.getMainWorkerExecutor().named("wgen_fill_noise")`）vs 我们 **HEAD 同步做完**（`NoiseChunkGeneratorMixin.java`）→ 可能改变 chunk 状态推进/任务派发节奏，使 Worker 线程大量时间无活可干或等依赖。
- **b2 写回副作用候选**：我们 `writeChunk` **写全部 98304 格（含 air）** + 全量重建 6 张高度图（vanilla 只写非 air `NoiseChunkGenerator.java:411` + 增量 2 张 + `markBlockForPostProcessing`）→ 可能让**后续 LIGHT/后处理**阶段成本或等待变重（light 阶段尚未装计时）。
- 次要：MC chunk 调度器/任务锁（`ChunkTaskScheduler`）与 per-chunk 依赖半径；Chunky 驱动批量模型（两侧同驱动，但延迟不同 → 需 b1 侧证据）。

## 5. 本轮落地的改动（R1 + 诊断，全部默认关/行为等价）

| 改动 | 文件 | 语义 |
|---|---|---|
| R1 per-chunk mixin 日志门控（默认关，`-Dcoreswap.mixlog=1` 开） | `runtime/1.21.6/java/.../mixin/NoiseChunkGeneratorMixin.java`（4 处）+ `runtime/1.20.1/java/...` 同款 4 处 | 行为等价（仅日志）；实测 −13% wall，且为生产同类成本 |
| `[WG-CONF]` 一次性行为化配置证据 | `worldgen-core/src/worldgen_handle.rs`（首次 fill 打一次） | 行为等价（hash 哨兵 PASS）；让 A/B 有「分支真生效」证据（#81/#37） |
| chunk 级分段计时（默认关，`-Dcoreswap.chunktime=1`） | 新增 `wg/bench/ChunkTiming.java`；`CppBridge`（jni/scan/write/hmap）；`mixin/NoiseChunkGeneratorMixin`（mixin 段 + gap）；新增 `mixin/NoiseChunkGeneratorTimingMixin`（carve）；`ChunkGeneratorFeaturesMixin`（feat） | 门控默认关；chunk 级非逐点（测量污染铁律）；`coreswap.mixins.json` 同步登记（#40） |
| gradle 映射 | `runtime/1.21.6/java/build.gradle`：`-Pmixlog` / `-Pchunktime` / `-PmaxBgThreads` / `-PcaMin` / `-PestL2` / `-PcaCap` | 走 property 通道绕 daemon env（#32）；⚠️ **必须在 `benchVmArgs` 闭包内**（`tasks.matching{}` 里 `run` 不可见） |

## 6. 错误台账（本块新增，供 errors 篇归并）

| # | 现象 | 根因 | 定位 | 修复 | 教训 |
|---|---|---|---|---|---|
| E1 | 跑批脚本「启动了服务器但从不发 RCON 命令」，进程挂死 | `Get-NetTCPConnection`（WMI/CIM 后端）在本沙箱**不可用**→ cmdlet 阻塞（`Get-CimInstance` 同样报「无法从客户端访问 CIM 资源」） | 手动 RCON 可用、日志无 RCON Client 行 → 定位到脚本卡点 | 脚本内**禁用一切 WMI/CIM cmdlet**；改用「WorkingSet 最大 / CPU 最高 的 java 进程」识别服务器 | 沙箱可用性判据要实测，不能凭常识；编排脚本每步都要可观测（transcript） |
| E2 | gradle 报 `Could not get unknown property 'run' for task ':runServer'` | 我把握手参数写进 `tasks.matching{ it.name=='runServer' }` 块——`run` 只在 `loom.runs` 的 `benchVmArgs` 闭包作用域内可见 | 构建日志 line 235 | 把映射移回 `benchVmArgs` 闭包 | 「编译过/配置过」≠「接线生效」；新增 -P→-D 映射必须先核**作用域**（#8/#19 家族新形态） |
| E3 | 两次 run 相互抢占（前一轮未结束时启动下一轮）→ 世界/端口冲突 + 双 FAIL | 我的编排错误（launch 前未等上一轮结束） | results.txt 出现重复 FAIL 行 | 改为单进程串行编排 + 启动前确认上一轮 `DONE` | 性能 run 必须严格串行（#28 的编排侧形态） |
| E4 | `jcmd <pid> Thread.print` 返回 `IOException: 拒绝访问` | 沙箱禁止 JVM attach（命名管道） | jcmd 直接报错、dump 文件 8 字节 | 改用 .NET `Process.Threads`（ThreadState/WaitReason）读线程状态 | 沙箱内 JVM attach 类工具（jcmd/jstack/jmap）不可用，需替代观测面 |
| E5 | 日志 tail 读取滞后（读到的是数分钟前内容，`LastWriteTime` 也不更新） | gradle 重定向 stdout 的缓冲/元数据刷新延迟 | 同一文件晚 2 分钟再读出现新行 | 监控改为「隔轮复核 + 以最终 results 行为准」 | 实时监控不可全信；结论以落盘终值为准 |

## 7. 产物清单

- 驱动脚本：`.tmp/perf-reg-260910-04/{run_arms2.ps1, probe_rate.ps1, run_native_A.ps1, run_native_A2.ps1, rcon_one.py}`
- 原始数据：`.tmp/perf-reg-260910-04/{native-A.txt, native-A2.txt, results.txt, logs/*.log, logs/*.log.err}`
- 本记录：`.investigations/perf-regression-260910-04/record.md`
- scout 成本图：`.investigations/perf-regression-260910-04/pipeline-cost-map.md`
- fan-out 候选：`.artifacts/perf-regression-260910-04/candidates/b1-*.md`、`b2-*.md`（并行中/见文件）

---

## 8. 勘误与 judge 条件应用（260910-04 18:10 追加）

- **§2.3 表述修正**：c1（234s）为 **instrument 关**臂；ct1（283s 早次 / **264s 完整 run**）为 **instrument 开**臂——二者不是「同配置双跑」。完整 run = cmd-output/ct1-inflight-r2.log（Total 0:04:24，4225 chunks，inflight max=1，Σ=232.4+5.4+11.4=**249.2s** / wall 270.3s = **92.2%**）。
- **§2.5 池缩放修正**：-PcoreswapThreads = **Rust 每次调用的线程数**（-Dcoreswap.threads），**不是 Java worker 池**（后者 = -PmaxBgThreads → -Dmax.bg.threads）。故「coreswap 对池大小不敏感」**撤回**（coreswap 池臂被并发 run 抢占，未取得）。vanilla 池 23→4 的 raw = **76s**（cmd-output/rate-vanilla-pool4.log，rate 52.6→64.3 cps），非「~100s」；vgCores=2.66 未落盘，不作判据。
- **§5 R1 表述降级**：268s→234s 为**跨 dll（5e30187a→abd7d889）+ 跨批 + 单次对**，量级落在 ±10-20% 噪声带内 → 只能写「**方向确定**（消除每 chunk 2 行同步 log4j 写，生产同样受益），量级待同批 ABBA ≥3 对」。
- **新发现（[WG-CONF] 语义）**：skip_* 打印的是 lags 位（worldgen_handle.rs:656-658），非生效分支（:669 = lags OR env）→ 
ative-A2.txt 打 skip_features=false 而脚本设了 WG_SKIP_FEATURES=1；**该行不可作 skip 类行为化证据**（A2 的 in-game 语义仅由 156.2→39.86ms 时差佐证）。修复待下轮。
- **证据保全（judge C1/C7）**：原始日志/脚本已归档到 .investigations/perf-regression-260910-04/cmd-output/（含 MANIFEST-sha256.txt）；更早一次 ct1 run 的 n=1024 Σ 行原文已被重跑覆盖（仅本文件转录，Degraded）；Java 侧改动在 /runtime/（gitignore）→ 无 git diff 源，sha256 清单为唯一完整性判据。
- **§15.4 取代链**：① 取代谢「线程超额订阅」嫌疑（见 verdict §5）；② 取代谢「chunkradius=56」口径表述（实际 = 默认 radius 500 = 4225 chunks；「回显 Incorrect argument」无落盘证据 = Degraded）。
- **证据饱和**：数据层推进 6 轮均产新证据，连续无新证据轮次 = 0；编排/抢占类失败轮（ct1×2、rate-coreswap×1、C 批挂死×1）属工程失败，不计入 §9.4。
