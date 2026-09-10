# verdict — nether/end 异步化推广（R3 同款改造 + 各自行为门）

> 区块：260910-05。状态：**confirmed**（用户 260910-05 明确授权，2026-09-10 19:58；judge J3/J4 = PASS-with-conditions，C1-C10 已应用，另有收尾审查 FIN-C1-C9 已应用）。
> 上游：260910-04 R3（overworld 异步化，confirmed）；本块 `verdict-c7-260910-05.md`（C7 收口）。
> 证据链：`.investigations/perf-closeout-260910-05/`（`record.md` §5 / `java-snapshot/pre|post` / `cmd-output/`，含 251 条 MANIFEST-sha256）+ `.tmp/perf-reg-260910-05/`（原始日志与 `results.txt`，不入库）。

## 1. 代码改动（Java 侧，纯调度形态；Rust 零改动）

| 文件 | 改动 | 依据 |
|---|---|---|
| `mixin/NoiseChunkGeneratorMixin.java` | nether / end 两分支改为与 overworld R3 分支**同构**：`Supplier<Chunk> work`（`feedBeardifierNether/End` + `fillChunkNether/End`）→ `print + rethrow` 异常路径 → `ChunkTiming.enter/exit + inflightEnter/Exit` → 共用同一 `SYNCFILL` 开关 → `CompletableFuture.supplyAsync(work, WG_FILL_POOL)` | 原为 `completedFuture(...)` **同步**形态（pre 快照 `:154-168`）；单车道串行化在三维一致 |
| `CppBridge.java` | 新增 `MIXLOG`（`coreswap.mixlog`），nether/end 两处**无条件每 chunk** `[WG-FILL]` 行的 `println` 改为门控 | 与 260910-04 R1（overworld per-chunk mixin 日志门控）同族；见 §5 边界 4（**只门控 println，读回计算仍每 chunk 执行**） |

- judge J3 独立结论：**逐要素同构、无遗漏**（work supplier / `feedBeardifier→fillChunk` 顺序 / print+rethrow / `enter|inflightEnter|exit|inflightExit` 配对 / 共用开关 / `supplyAsync` 目标池）；**overworld 分支 pre/post 逐字未动**；句柄与 ThreadLocal 缓冲未串味。
- 编译：`gradle compileJava` **BUILD SUCCESSFUL**（23s）。
- **执行体**：Rust 侧零改动 ⇒ 无重编 dll。本块 **7 个 coreswap 臂**日志 `[CppBridge] dll=…sha256=abd7d8893d22e030…`（`nv-r1`/`ev-r1` 为 vanilla 臂 ⇒ `dllsha=-`，`results-260910-05.txt:6/10`）；⚠️「与 `target/release/worldgen1216.dll` 一致」本块**未重新复算 sha**（260910-04 已核，本块按继承声明）。
- **VCS 替代（R2）**：`runtime/` 被 `.gitignore:95` 忽略 ⇒ 以 `java-snapshot/pre|post`（各 4 文件 + `SHA256SUMS.txt`）承载 diff：本块 = mixin（`Compare-Object` 口径 69 行差异；文件行数 net 196→253 = **+57**）+ CppBridge（`Compare-Object` 10 行），`ChunkTiming.java` / `build.gradle` UNCHANGED。
- **日志门控对 A/B 的对称性**：A/B 臂（`nv/ns/na/ev/es/ea`）**均未开** `-Pmixlog`（对称）；sanity 臂 `naS`/`eaS` 单独开。

## 2. 载具首用 sanity（P4a，新载体：Chunky 维度 + DIM region 路径）

| 臂 | 维度 | 结论 | 就地证据 |
|---|---|---|---|
| `naS-r1` | nether | ✅ 维度名/路径/接管/写回全通 | `chunky world minecraft:the_nether` 被接受；`Task finished for minecraft:the_nether. Processed: 4225 chunks (100.00%), Total time: 0:00:21`；`populateNoise(nether) intercepted` = **4761** 行 + `[WG-FILL]` = **4761** 行；region `run\world\DIM-1\region` 16 mca 采集成功 |
| `eaS-r1` | end | ✅ 同上（judge C3 缺口已闭合） | `[CppBridge] initEnd seed=417950215108767439 enabled=true stageMask=3`；`Task finished for minecraft:the_end … Total time: 0:00:07`；`populateNoise(end) intercepted` = **4761** 行 + `[WG-FILL]` = **4761** 行；region `run\world\DIM1\region` 16 mca |

归档（FIN-C4）：`naS-r1`/`eaS-r1` 的日志与 region 均已归档——`cmd-output/logs/{naS,eaS}-r1.log{,.err}` + `cmd-output/region-{naS,eaS}-r1/`（各 16 mca）；`cmd-output/MANIFEST-sha256.txt` 已刷新（269 条，含 eaS 行）。`results-260910-05.txt` = 14 行（13 条 A/B/sanity 臂 + eaS 补跑）。`eaS` = 该维第 8 个 coreswap 臂。`mixin 69 行差异` 为 `Compare-Object` 口径（文件净增 +57 行）。

⚠️ **4761 是「调用次数」不是 chunk 数**（judge C8）：三处行计数（`WG-FILL` / `intercepted(nether|end)` / `buildSurface` skip）**同为 4761**，而 Chunky 只生成 4225 chunks ⇒ 4761 > 4225 表明存在**重复接管调用**（同 chunk 被 `populateNoise` 多次进入）。该现象与「`feedBeardifier` 与 `fillChunk` 在同 chunk 上交错」属同族既有风险面，**登记为后续项**（本块未查成因；不影响形态 A/B，因双臂同样存在且 `[WG-CONF] flags=3`）。

## 3. 行为门（每维三对；工具 `cmd-output/diff_arms.py`，与 overworld 同修订）

§9.7 三要素：**载体** = 1.21.6 + Chunky 默认区域（脚本未发 radius 命令，实测 **4225 chunks/臂 = 65×65**，与 260910-04 同口径；⚠️「radius 500 方块」的**配置来源本块未核**）；**覆盖面** = region 目录全域逐块普查（各对 `common` 见下表，非抽样）；**与既有口径可比性** = 同载具/同 seed/同 region 中心/同工具修订，与 overworld 三对同构；⚠️ **维度间不可互引**（#33 载具可比性）——nether 噪声量级与 overworld 差 ~15×，故各维自比自判。

### 3.1 nether（16 mca × 各臂）
| 对 | 类型 | common | blocks | diff | % | sections same/diff |
|---|---|---|---|---|---|---|
| `ns × nv` | 接管基线（跨实现） | 7542 | 209,965,056 | 214,240 | 0.1020% | 50555/706 |
| `na-r1 × ns` | **形态等价门（跨形态）** | 7542 | 209,981,440 | 255,942 | **0.1219%** | 50508/757 |
| `na-r1 × na-r2` | **同形态噪声（async 代理）** | 7543 | 209,981,440 | 295,191 | **0.1406%** | 50436/829 |

**判据（预登记）**：跨形态 ≤ 同形态噪声 ⇒ **0.1219% ≤ 0.1406% ✅ PASS**。
⚠️ **口径（judge C1）**：nether 只有 **1 个 sync run**（`ns-r1`）⇒ **无同配置 sync run-to-run 对**；上述 0.1406% 出自 **async 同形态对**（`diff-N_async_vs_async2.txt:74`），作**代理**使用。代理合理性依据：同机制在 overworld 的实测同形态噪声极小（sync 0.0086% / async 0.0092%）。
⚠️ **单对点估计（judge C7）**：每形态各 1 对、无置信区间、**检验力低**；门为「同量级筛选门」，不构成「无内容差」。
**附带读数**：接管基线 0.1020% **低于** nether 自身（async 代理）噪声 ⇒ 在本载体下 1.21.6 nether 接管与 vanilla 的差异**落在 run 级非确定带内**（≠「接管已验证」）。
**单侧排除**：`na-r1 × ns` 有 1 个 chunk 单侧排除（**7543/7542** 差；一手 `diff-N_async_vs_sync.txt:1-2`）——量级 ~0.01% 级，不影响结论（judge N2）。

### 3.2 end
| 对 | 类型 | common | blocks | diff | % | sections same/diff |
|---|---|---|---|---|---|---|
| `es × ev` | 接管基线（跨实现） | 7567 | 494,010,368 | **7** | 0.0000% | 120601/7 |
| `ea-r1 × es` | **形态等价门（跨形态）** | 7567 | 494,055,424 | **0** | **0.0000%** | 120619/0 |
| `ea-r1 × ea-r2` | **同形态噪声（coreswap 臂内）** | 7567 | 494,100,480 | **0** | **0.0000%** | 120630/0 |

**判据**：跨形态 0 ≤ 同形态 0 ⇒ **PASS**。
**「0 差」可信性论证（judge C9，写入支持性证据）**：
1. **灵敏度**：同一条工具链在同一维度上**检出了** `es × ev` 的 **7 块差** ⇒ 工具并非恒零；
2. **无单侧排除**：三对 `common = 7567` 完全一致，无 chunk 被单侧剔除；
3. **分母非空**：两臂 section 并集 = 494,100,480 块 ≫ 0，且 `sections same` = 120,630（非「无数据」）；
4. **输入互异**：三个 region 目录在 MANIFEST 中 sha256 互异（`cmd-output/MANIFEST-sha256.txt`）⇒ 是三次独立 run 的产物，不是同一文件被重复比对。
⚠️ **限定（judge C2）**：`0 差` 仅适用 **coreswap 臂内 + 该 region 集**（common=7567、section 并集 494,100,480）；**不得**外推为「该维不存在 run 级非确定」（`ev` 只有 1 run，7 块差**无法**归因「实现差 vs vanilla 自身 run 噪声」）；也**不得**把 7 块与 0 块**并列**——7 > 0 说明跨实现差在该维**可检出**。

## 4. 形态证据与性能（同构建态单变量 A/B）

| 臂 | 维度 | 形态 | Chunky Total（进程内任务计时） | wallgen（脚本侧墙钟） | inflight max | CPU·s |
|---|---|---|---|---|---|---|
| `nv-r1` | nether | vanilla | 56s | 60.0 | — | 未测 |
| `ns-r1` | nether | sync | 69s | 70.1 | **1** | 未测 |
| `na-r1` / `na-r2` | nether | async | 21s / 20s | 30.0 / 30.0 | **23** / **23** | 未测 |
| `ev-r1` | end | vanilla | 5s | 10.0 | — | 未测 |
| `es-r1` | end | sync | 18s | 20.0 | **1** | 未测 |
| `ea-r1` / `ea-r2` | end | async | 6s / 6s | 10.0 / 10.0 | **23** / **23** | 未测 / **86 CPU·s** |
| `naS-r1` / `eaS-r1` | nether/end | async + mixlog | 21s / 7s | 30.0 / 20.0 | 23 / 23 | 未测 / 99 CPU·s（cores 4.95） |

- **形态效应（同口径）**：
  - Chunky 进程内任务计时：nether **69 → 20/21s（3.3-3.5×）**；end **18 → 6s（3.0×）**。
  - 脚本侧 wallgen：nether **70.1 → 30.0（2.34×）**；end **20.0 → 10.0（2.00×）**。
  - ⚠️ **口径点明（judge C4）**：3.3-3.5× / 3.0× 是 **Chunky 进程内任务计时**；wallgen 口径只有 **2.34× / 2.00×**，因 `wallgen − sec` 尾巴在 1.1-10s 间**不稳定**（启停/落盘/停服开销）。**禁用「端到端」措辞**——项目铁律下端到端（vs Java 生产口径）本块**未测**。
- **相对 vanilla（同口径）**：Chunky 计时 nether async **快 2.7-2.8×**（56 → 20/21s）；end async 6s vs vanilla 5s（同量级，略慢）；wallgen 口径 nether **2.00×**（60.0 → 30.0）、end **1.00×**（10.0 → 10.0）。**sync 形态在净口径上都是净亏损**：nether sync 70.1 > vanilla 60.0、end sync 20.0 > vanilla 10.0（慢 2×）。
- **`inflight max` 1 → 23 在两维复现**（同一仪器 `-Pchunktime=1`，`ChunkTiming` 双 guard + 各臂日志原位行：`na/ea` 系列 `inflight max=23`，`ns/es` 系列 `inflight max=1`）⇒ 形态改造的直接证据。
- **CPU 解读（judge C5）**：`ea-r2` = **86 CPU·s / 6s 任务**（≈14.3 核均值；`eaS-r1` = 99 CPU·s / 7s，cores 4.95 系除以 wallgen=20 的口径）⇒ 异步形态是**「更多总 CPU 换更低墙钟延迟」**，属**延迟结论，非效率结论**；nether 侧总 CPU 未测。另 judge 观察到单块 mixin 段在异步下膨胀（nether 10-11 → 25-37ms、end 3.5 → 20-22ms）与平均并发度（5.75 / 15.4 vs 峰值 23）——**登记为后置课题**（延迟换 CPU 的代价面）。
- 幅度性质：同构建态单变量 A/B（同 dll/同 seed/同 region/背靠背）；**非** ABBA 多对（HOOK-1 未选）⇒ 只报读数，不给置信区间。

## 5. 诚实边界 / 未核项
1. **每形态仅 1 对**读数（nether 的「同形态」还是 async 代理，见 §3.1）；**检验力低**。
2. **1.21.6 nether/end 接管属首次验证**：本块只给「接管 vs vanilla」单读数，**未**做接管正确性专项（历史 `end-takeover`/`nether-save-full` 均 1.20.1 载体）；nether 0.1020% 落在 **async 同形态代理噪声（0.1406%）**带内（该维**无 sync run-to-run 对**），**不能**读成「接管已对齐 vanilla」；end 的 7 块差**无法**归因。
3. **CPU 缺失根因 = 未核**（judge C5/FIN-C1）：本批 13 行归档表 **12 行** `serverCpu=-1`，仅 `ea-r2`（86）与补跑 `eaS-r1`（99）**两臂**采到。两轮 judge 各读到一版脚本（一处称 `:112-113` 有重复采样、另一处称现版无重复块），且归档副本（`.tmp` 与 `cmd-output` 两份）均为**修后**版本 ⇒ **文件级不可复核**；行为侧旁证：修后采到 CPU 的是两臂（`ea-r2`/`eaS-r1`）。**不再使用「脚本已修 ⇒ 已解决」的因果表述**。
4. **`[WG-FILL]` 门控范围**（judge C8）：只包住 `println`；`nzBuf` 全 buffer 扫描 + 16 点 `getBlockState` 读回**仍每 chunk 无条件执行**（`CppBridge` post 快照 `:441-453` / `:485-499`；overworld `fillChunk` 无此块）⇒ 门控只消除日志成本，读回成本仍在（R1 家族残余项）。
5. **重复接管调用**（4761 > 4225，§2）成因未查；登记后续项。
6. **Chunky 区域配置来源未核**（脚本无 radius 命令；`4225 = 65²` 由实测 Total 行反推）。
7. 工具内嵌 overworld 定点段对维度无意义（输出 `NOT in both arms`）；`blocks` 分母 = 4096 × 两侧 section 并集（同 overworld 口径）；**region 目录口径**下 `common`（7542/7567）> 臂内 Chunky 4225 chunks（与 260910-04 的 7959 vs 4225 同族缺口，成因未证）。
8. **未核**（无 shell 侧）：region 二进制内容、`diff_arms.py` 逐位算法、HEAD/sha；本文件数字均引自 `cmd-output/` 已归档一手输出。

## 6. 结论（candidate）
**nether / end 分支的 R3 同款异步化成立**：形态证据（`inflight max` 1→23 ×2 维）、性能（Chunky 计时 nether 3.3-3.5× / end 3.0×；wallgen 口径 2.34× / 2.00×；nether 同口径反超 vanilla 2.7-2.8×、end 与 vanilla 同量级）、行为门（nether 跨形态 ≤ async 代理噪声；end 跨形态 **0 块差**（**common=7567 / 两侧 section 并集 494,100,480**）且经灵敏度/无单侧排除/分母非空/输入互异四重论证）三面齐备；Rust 侧零改动 ⇒ 与 260910-04 同一执行体，无跨版本/跨构建风险。
**限定**：单对点估计、检验力低；「0 差」与「接管已验证」均**限定在 §3/§5 声明的范围**。建议 candidate → 用户裁决 confirmed（HOOK-3）。

## 7. judge 条件应用记录（C1-C10）
| 条件 | 应用处 |
|---|---|
| C1 nether 噪声带 = async 代理 | §3.1 ⚠️ 口径 |
| C2 end 零差限定 + 删除绝对化表述 | §3.2 限定 + §6 |
| C3 end sanity 缺口 | §2 新增 `eaS-r1` 实跑证据（非仅静态锚） |
| C4 比值口径点明 + wallgen 并列 + 禁「端到端」 | §4 |
| C5 CPU 解读 + 根因按未核 | §4 + §5.3 |
| C6 dllsha 范围（7 coreswap 臂）+ 未复算声明 | §1 |
| C7 单对点估计/检验力低内联 | §3.1 + §3.2 |
| C8 门控范围（仅 println）+ 4761 = 调用次数 | §1 + §2 |
| C9 end 零差的可核论证四要素 | §3.2 |
| C10 根 `index.yaml` nether-end 条目摘要同批修正 | `.artifacts/index.yaml` 该条目注释 |
| N5 `patch-nether-end-async.md` 状态回填 | 该文件头「已应用（post 快照为准）」 |
| N4 重复接管交错风险登记 | §2 + §5.5 |

## 8. §15.4 取代记录（260910-06，2026-09-10）

> §15.4「结论取代链」形态：**原文不删不改**（本节为附加记录，双指针 + 一行理由）。被取代段落保持原样。

**取代指针（supersedes）**：本文件 **§5.5**「重复接管调用（4761 > 4225，§2）成因未查；登记后续项」，
及其依赖的 **§2 ⚠️ 段**（「4761 > 4225 表明存在**重复接管调用**（同 chunk 被 `populateNoise` 多次进入）」）
与 **§7 N4 行**（「重复接管交错风险登记」）。

**一行取代理由**：260910-06 的独立复核以「**坐标去重**」判据证明该推断为假——`[Mixin] populateNoise(nether|end) intercepted`
共 **4761 行 = 4761 个互异 chunk 坐标**（重复坐标组 **0**，且恰好填满 69×69 方阵；nether/end 两臂同），
超出 Chunky 计数的 **536 = 69² − 65²** 来自**额外不同 chunk**（NOISE 之上的邻域依赖环），**不存在「同 chunk 被多次进入」**。

**替代陈述**：`populateNoise(nether|end)` 的接管调用数与 chunk 数**一一对应**；`4761 > 4225` 是
**计数面差异**（Chunky「Processed」= 65×65 方块，实际生成面 = 69×69），不是重复调用。

**证据（可复现）**：`.investigations/nether-end-verify-260910-06/handoff-verify-260910-06.md` §V1（含坐标去重复现命令）；
原始日志 `.investigations/perf-closeout-260910-05/cmd-output/logs/{naS,eaS}-r1.log`。

**补充收敛（非取代，同一复核）**：本文件 **§5.7**「`common`（7542/7567）> 臂内 Chunky 4225 chunks，成因未证」——
260910-06 复核收敛为两条：① 每臂 region **全部槽位**的写入时间戳均落在**本臂自身**运行窗口内（`Remove-Item run\world` 生效，无跨臂残留）；
② 缺口 chunk 为 **`status=structure_starts` 的低状态残缺 chunk**（section 全空，非地形产出）⇒ 行为门分母含 ≈37% 非地形 chunk，
**有效可对比地形集 ≈ 4761 chunk/维**（后续引用 `common` 作分母时 MUST 连此口径一并声明）。
证据：同文件 §V2/§V3。

**补充收敛（E5，260910-06 追记；非取代，原文不删不改）**：本文件 §3 的 `common` 与差异百分比由 `diff_arms.py` 计算，
该工具 `t == 7`（TAG_Byte_Array）分支把载荷长度按 **1 字节**读（NBT 规范 = **TAG_Int(4B)**；一手源码证据 = 本目录
`cmd-output/diff_arms.py:22` = `if t==7: return r.n(r.u1())`），**静默丢弃 chunk 并使 section/块分母严重失真**。
260910-06 以同载体对照实测（`.investigations/perf-reg-260910-06/cmd-output/tag7-impact-output-260910-06.txt`
+ 该块 `diffs/` vs `diffs-fixed/`）：

| 口径（region-os-r1 自比） | 旧读法（本文件所用） | 规范读法 | 倍数 |
|---|---|---|---|
| 可解析 chunk 数 | 7703（bad 46） | 7749（bad 0） | 1.006 |
| section 并集 | 85,047 | 185,976 | **2.187** |
| 块分母 | 348,352,512 | 761,757,696 | **2.187** |
| sections/chunk | 11.04 | 24.00 | — |

⇒ **本文件的 `common` 与差异百分比分母不可按固定比例折算**（新旧映射**非单调**：同批对照中 cross 0.0648/0.0576→0.0298/0.0301、
async 锚 0.0467→0.0292、sync 锚 0.0209→0.0174、正对照 0.0351→0.0325 略降）。
**「量级与定性结论不变」为未经复算的推断**（本文件结论量级是否保持，须复算后才能判）——**引用本文件任何 `diff`/`%` 前
MUST 先用规范读法复算**。**复算登记为显式待办**（后续项）：用规范读法重跑本块全部 region 对拍并出补充读数。
工具修正见 `diff_1201.py`（1.20.1 侧）+ knowledge/discovered/build-tooling 发现 #53。
