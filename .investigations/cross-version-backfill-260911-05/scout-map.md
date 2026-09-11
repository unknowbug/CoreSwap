# 跨版本回灌核查勘探地图（260911-05 B2，只读）

> scout 只读勘探；不含结论性判断；未核项单列。
> 任务：对「1.20.1 侧后续发现/修复 → 1.21.6 侧是否存在（或形式不同但等价）」做逐项事实核对。
> 二分前提（架构事实，AGENTS.md §〇 / 架构计划-260911-05 §1）：`worldgen-core/` = 跨版本共享 Rust 引擎（rlib）；
> `versions/<ver>/rust/` = 每版薄壳（cdylib）；`versions/<ver>/java/` = 每版独立 Java 工程（mixin/CppBridge/build.gradle）。
> 本次未跑任何命令（沙箱禁 cargo/gradle/python/git）：全部判定来自**读文件 + grep/glob**；凡未亲眼读到的行写入 §3。
> 参考的 1.20.1 侧改动记录：`versions/1.20.1/docs/10-timewise-archive.md:3127`（260911-04 块）+
> `.investigations/a1-opt-pool-260911-05/record-260911-05.md`、`a2-dim-gate-260911-05.md`、
> `.investigations/000-架构设计/架构计划-260911-05.md`。

## 1. 判定表

| # | 项 | 1.20.1 锚点 | 1.21.6 锚点 | 判定 | 备注 |
|---|---|---|---|---|---|
| 1 | exec 模式（自有有界执行器，缺省开，`-Dcoreswap.exec=0` 回退，`[WG-EXEC]` 自证） | `NoiseChunkGeneratorMixin.java:135-146`（`EXEC_MODE=resolveExecMode()`，prop 空→`return true`）、`:148-164`（`-Dcoreswap.execpool`，缺省 `logical/2-2`）、`:170-197`（`ThreadPoolExecutor(N,N,60s,LinkedBlockingQueue(128),namedFactory,CallerRunsPolicy)` + `allowCoreThreadTimeOut(true)` + 线程名 `CoreSwap-Fill-N`）、`:199-206`（`[WG-EXEC] mode=own-pool pool=… queue=128 rejectedPolicy=CallerRuns`）、`:227-253`（`wgDispatch` 优先级 `SYNCFILL > EXEC_MODE > P1 信号量`） | `NoiseChunkGeneratorMixin.java:57-61`（`WG_FILL_POOL = Util.getMainWorkerExecutor().named("coreswap_fill_noise")`）、`:145-149`/`:183-187`/`:219-223`（三分支仅 `SYNCFILL ? completedFuture : supplyAsync(work, WG_FILL_POOL)`）；`CppBridge.java` 全文无 exec 相关 | **缺失** | 1.21.6 = R3 `supplyAsync` 共享池 + `SYNCFILL` 回退；无自有池、无 `[WG-EXEC]` 自证行 |
| 2 | P1 信号量 / `-Dcoreswap.maxinflight` / `[WG-INFLIGHT]` | `NoiseChunkGeneratorMixin.java:86-116`（`WG_MAX_INFLIGHT=resolveMaxInflight()`，缺省 `max(1, logical/2-2)`，`N<=0` 不限）、`:88-99`（`[WG-INFLIGHT] max_inflight=… logical=… syncfill=…`）、`:101-103`（`Semaphore`，`N<=0` 时为 null）、`:237-252`（调用线程 `acquireUninterruptibly`，`whenComplete` 释放，提交失败 `permits.release()`） | 全树 grep `maxinflight`/`WG-INFLIGHT`/`Semaphore` 零命中；`NoiseChunkGeneratorMixin.java` 无对应字段/方法 | **缺失** | 1.20.1 注释 `:118-134` 明确 exec 与 P1 互补、exec 开时 `maxinflight` 被忽略 |
| 3 | `[WG-CONTENT]` 逐 chunk 指纹门 + `wgBufHash` | `CppBridge.java:372-391`（`wgBufHash` = FNV-1a 64，逐 int 逐字节混入，位置敏感）、overworld `:419-425`、nether `:460-478`、end `:509-529` ⇒ **三维齐备** | `CppBridge.java` 全文 grep `WG-CONTENT`/`wgBufHash`/`FNV` 零命中（629 行全文已读） | **缺失（三维全缺，非部分）** | 1.20.1 end 维指纹行是 260911-05 A2 补的（`a2-dim-gate-260911-05.md:10-12`，提交 `8d8075a`） |
| 4 | `StallWatch` 停滞看门狗（`-Dcoreswap.stallwatch`） | `StallWatch.java:1-59`（整文件：daemon 线程 + `Thread.getAllStackTraces()` + `[STALLWATCH]` 前缀）、`CppBridge.java:87-88`（`init` 内 `StallWatch.start()`）、`build.gradle:139-140`（`-Pstallwatch=<秒>` 映射） | 无 `StallWatch.java`（1.21.6 `wg/bench` glob 无此文件）；`CppBridge.java:82-103`（init）无调用；`build.gradle` grep `stallwatch` 零命中 | **缺失** | 1.20.1 侧存在理由：沙箱禁 JVM attach（`StallWatch.java:7-9`） |
| 5 | A1a 写回跳空气（`writeChunk` raw id 0 = air 跳过 + `-Dcoreswap.skipair=0` + `WBCHECK` 自检） | `CppBridge.java:536-547`（A1a 注释 + `WBCHECK` + `SKIPAIR = !"0".equals(prop)` + 两个 AtomicLong）、`:562-572`（`if (id == 0) { if (WBCHECK) {…isOf(Blocks.AIR)…} if (SKIPAIR) continue; }`）、`:606-611`（`destroy()` 内 `[WB-CHECK] air_skipped=… stale_nonair=…` 汇总） | `CppBridge.java:506-537`（`writeChunk` 逐格：`int id = buf[base+x]` → 越界检查 → `STATE_BY_ID.get(id)` → `sec.setBlockState(...)`，**无 id==0 分支**）；grep `skipair`/`wbcheck` 零命中 | **缺失** | 1.21.6 `writeChunk` 现状 = 1.20.1 改造前形态（每 chunk 98304 次 `setBlockState` 含空气）。另：1.20.1 侧 A1a **尚未随任何 release 出货**（`record-260911-05.md:96`） |
| 6 | A1b 诊断扫描门控（`nz` 全 buffer 扫描、nether/end 16 点读回是否恒执行） | overworld `:410-425`（全量扫描移入 `MIXLOG`；生产侧改 O(1) 短路：`if (buf[0]==0){ 遇首个非零即停 }` 保留 all-air 信号）、nether `:459-478`（`nzBuf` 扫描 + 16 点读回**整块**在 `if (MIXLOG)` 内）、end `:508-529`（同） | overworld `:399-405`（`for (int k=0;k<buf.length;k++) if (buf[k]!=0) nz++;` **无条件** 98304 读 + `buf-all-air`/`buf-sparse` **无条件 println**）、nether `:441-456`（`nzBuf` 全量扫描**无条件**、16 点读回**无条件**，仅 println 受 `MIXLOG`）、end `:485-500`（同） | **缺失** | 1.21.6 现状 = 1.20.1 A1b **之前**形态。1.21.6 自身注释 `CppBridge.java:30-35` 说明其 R1 门控只覆盖 nether/end 的 `[WG-FILL]` println |
| 7 | A1d `adaptive_threads` count=1 clamp（Rust 共享层） | **共享层** `worldgen-core/src/api.rs:38-43`（`threads.min(count).max(1)`，含 260911-05 A1d 注释）、`:144-152`（`[WG-THREADLOG]` 一次性自证，`WG_THREADLOG=1` 门控）；唯一调用点 `api.rs:143`；per-call `std::thread::scope`（`api.rs:158-181`，无池） | 薄壳确认同一 core：`versions/1.21.6/rust/Cargo.toml:14-16`（`WorldgenRust = { path = "../../../worldgen-core" }`）、`lib.rs:1-3`（算法全在 core，本 crate 只做 ABI 适配）、`jni_bridge.rs:208-214`（`wg_fill_blocks_multi(..., threads)` 透传） | **存在（等价）** | 1.20.1 薄壳同构：`versions/1.20.1/rust/Cargo.toml:13-15` ⇒ 共享层一次覆盖两版，无 Java 侧移植项 |
| 8 | `ca_min`/`terrain_cache` 门控微修（260911-01 patch：`skip_features` 时不走缓存） | **共享层** `worldgen-core/src/worldgen_handle.rs:619`（`let ca_min = env("WG_CA_MIN").map(|v| v!="0").unwrap_or(true)`）、`:620-626`（flags 提前 load + `skip_features` 同源提取 + `if ca_min && !skip_features { 缓存 } else { 直算 }`）、`:673-675`（features 门控复用同一 `skip_features`）；`resolveStageMask` 缺省 = `CppBridge.java:69-74`（`return 0b011`） | 同一 `worldgen-core`（见 #7 的 Cargo.toml 锚点）；`resolveStageMask` 缺省 = `CppBridge.java:75-80`（`return 0b011`） | **存在（等价）** | **两版 `resolveStageMask` 缺省值相同（均 0b011）**，无差异；`CA_MIN` 缺省亦同源 = 开（`worldgen_handle.rs:619` `unwrap_or(true)`；1.21.6 侧记录 `docs/10-timewise-archive.md:27`）。1.21.6 独有 gradle wiring `-PcaMin`（`build.gradle:232-234`），1.20.1 `build.gradle` grep `caMin` 零命中 |
| 9 | `perfprofile` 临时件残留 | 源码零命中；仅 `build.gradle:6`（注释「移除 perfprofile 临时件」）、`docs/10-timewise-archive.md:3127/3132-3133`、`knowledge/discovered/workflow-patterns.md:2101` 提及 | **Java 与 Rust 侧均零命中**（全树 grep `perfprofile`；glob `*perfprofile*` 仅命中 `.tmp/vivo-stutter-260911-02/ab-threads/logs/perfprofile.log` 与 `.log.err`） | **不适用（两版皆无源码残留）** | 剩余 2 个 `.tmp` 运行日志（临时区不入库，非源码） |
| 10 | `-Dcoreswap.rust.stages` / stageMask 语义 | `CppBridge.java:66-74`（bit0=SKIP_CARVER bit1=SKIP_FEATURES；`all`→0；非法→0b011）；**无** `rustFeaturesTakeover`（全树 grep 仅 1.21.6 命中） | `CppBridge.java:72-80`（同注释、同缺省 0b011）、`:165-176`（`rustFeaturesTakeover()` = `enabled && mask!=0 && (mask & 0b010)==0`，注释明确排除 `all`=0 防误判）；消费点 `ChunkGeneratorFeaturesMixin.java:68` | **缺省值等价 + 语义消费者分叉** | 1.21.6 侧接管范围决策 = 「矿物/树/装饰层**不接管**、mod 兼容留 Java 侧、性能收益小」「mask 翻转撤销、0b011 维持」：`versions/1.21.6/docs/10-timewise-archive.md:13`、`:26` |
| 11 | 反向回灌（1.21.6 独有、1.20.1 侧是否缺）——只列事实与锚点 | 见下方 11(a)~(h) 逐条 | 见下方 11(a)~(h) 逐条 | **反向分叉（8 条子项，均列事实不判断）** | — |

### 11 子项（反向：1.21.6 独有）

| 子项 | 1.21.6 锚点 | 1.20.1 侧 | 事实 |
|---|---|---|---|
| (a) features 接管 mixin | `ChunkGeneratorFeaturesMixin.java:41-148`（整文件：HEAD 门控 ThreadLocal + `@Redirect` 拦 `PlacedFeature.generate` + `setDecoratorSeed` ordinal=1 探针 + `[JFEATURE]`/`[JFEATURE-FID]` env 门控 `WG_FEATURESEQ`） | 1.20.1 `wg/bench/mixin` glob **无此文件** | 1.21.6 独有 |
| (b) `rustFeaturesTakeover()` | `CppBridge.java:165-176` | 1.20.1 `CppBridge.java` 无该方法（grep 零命中） | 1.21.6 独有 |
| (c) `ChunkTiming` 分项 | `ChunkTiming.java:16-29`（MIXIN/JNI/WRITE/HMAP/SCAN/BEARD/CARVE/FEAT/GAP + INFLIGHT/MAX_INFLIGHT）、`:42-60`（`featTick` + `FEAT_GAP` 臂无关尺子 + `REPORT_EVERY=256`）、`:88-102`（报告含 `jni/write/hmap/scan/beard` 分项 + `armAgnostic featInterval`） | `ChunkTiming.java:21-31`（仅 MIXIN/GAP/N）、`:37-71`（仅 inflight + mixin + gap 报告）；类注释 `:10-12` **主动声明**「1.20.1 侧 CppBridge 未挂分项钩子…不打印恒 0 的假分项」 | 1.21.6 分项更多；1.20.1 为声明式精简移植 |
| (d) `CppBridge` 分项计时钩子 | `CppBridge.java:385-393`（`addJni`）、`:399-402`（`addScan`）、`:406-412`（`addWrite`）、`:540-548`（`addHmap`） | 1.20.1 `CppBridge.java` 全文**无任何 `ChunkTiming` 调用** | 1.21.6 独有 |
| (e) carve 阶段计时 mixin | `NoiseChunkGeneratorTimingMixin.java:23-29`（`WG_CARVE_T0` + `addCarve`） | 1.20.1 无此文件 | 1.21.6 独有 |
| (f) `[WG-FILL]` 门控 | `CppBridge.java:30-35`（R1 门控注释）+ `:452`/`:496`（println 受 `MIXLOG`）；扫描/读回**不受**门控（见 #6） | 1.20.1 `:363-370` 注释 + `:459-478`/`:508-529`（**整块**门控，A1b 后形态） | 1.20.1 侧在此项**超前于** 1.21.6（非缺失） |
| (g) build.gradle 属性映射 | 1.21.6 `:132-133`（`-PmaxBgThreads` → `-Dmax.bg.threads`）、`:232-234`（`-PcaMin` → env `WG_CA_MIN`）、`:130-131`（mixlog/chunktime）、`:135`（syncfill） | 1.20.1 `:136-138`（mixlog/chunktime/syncfill）、`:139-140`（`-Pstallwatch`）；grep `maxBgThreads`/`caMin` **零命中**；grep `exec`/`maxinflight`/`skipair`/`wbcheck` 亦零命中（exec/P1/A1a 仅有 `-D` 直传，无 `-P` 映射） | 双向 wiring 差异（1.21.6 有 maxBgThreads/caMin；1.20.1 有 stallwatch） |
| (h) 文件级差异 | 1.21.6 独有：`ChunkGeneratorFeaturesMixin.java`、`NoiseChunkGeneratorTimingMixin.java` | 1.20.1 独有：`StallWatch.java`、`mixin/ThreadedAnvilChunkStorageAccessor.java:1-15`（`releaseLightTicket` invoker，光照 ticket 用途，与本次核查主题无关） | 两版文件集不同（同函数分叉见 §2） |

## 2. 两版行为分叉清单（同文件同函数不同实现）

| # | 文件 / 函数 | 1.20.1 实现 | 1.21.6 实现 | 分叉点 |
|---|---|---|---|---|
| D1 | `NoiseChunkGeneratorMixin.wgPopulateNoise` 注入签名 | `:273-283` 首参 `java.util.concurrent.Executor`（5 参 + CompletableFuture） | `:86-96` **无 Executor**（4 参 + CompletableFuture），注释 `:81-85` 记录 1.21.6 `populateNoise` 签名迁移与证据 | 上游签名不同（非选择） |
| D2 | 同上 · 重活分派 | `wgDispatch(work)`（`:227-253`）：`SYNCFILL → EXEC_MODE（自有池）→ P1 信号量 → 共享池` | 三分支各自内联 `SYNCFILL ? completedFuture(work.get()) : supplyAsync(work, WG_FILL_POOL)`（`:145-149`/`:183-187`/`:219-223`） | 分派策略（exec/P1 缺失 → 见 #1/#2） |
| D3 | 同上 · 填充池构造 | `:67-68` `Util.getMainWorkerExecutor()`（**无** `named()`；注释 `:63-65` 说明 `NameableExecutor` 是 1.21.6 才有） | `:57-58` `Util.getMainWorkerExecutor().named("coreswap_fill_noise")` | API 可用性 + 命名 |
| D4 | 同上 · Beardifier 计时 | 无（`CppBridge.feedBeardifier` 调用前后无打点） | `:126-128`/`:165-167`/`:201-203` `tb0` + `addBeard` | 1.21.6 独有分项计时 |
| D5 | 同上 · 类注释声明的锁语义 | `:41-44`、`:217-226`（不加外层 sections 锁；引 260910-06 错误台账 E1 自锁死） | **无对应段落**（1.21.6 无 `wgDispatch` 注释块） | 声明差异（1.20.1 移植时补的说明） |
| D6 | `CppBridge.fillChunk`（overworld） | `:410-418` O(1) 短路保留 all-air 信号；`:419-425` 全量 nz + `[WG-CONTENT]` 指纹**均在 `MIXLOG` 内** | `:399-405` 无条件全量 nz 扫描 + 无条件 `buf-all-air`/`buf-sparse` println；`:406-412` `addWrite` 计时 | 见 #3/#6 |
| D7 | `CppBridge.fillChunkNether` / `fillChunkEnd` | `:459-478` / `:508-529`：nzBuf 扫描 + 16 点读回 + 指纹行**整块**在 `if (MIXLOG)` 内 | `:441-456` / `:485-500`：nzBuf 扫描与 16 点读回**恒执行**，仅 println 在 `MIXLOG` 内；无指纹行 | 见 #3/#6 |
| D8 | `CppBridge.writeChunk` | `:549-601`：`id==0` → `WBCHECK` 自检 + `SKIPAIR` continue；**无** `ChunkTiming` | `:506-549`：逐格无条件写；**有** `:540-548` `addHmap` | 见 #5 + 分项计时 |
| D9 | `CppBridge` 方法集 | 无 `rustFeaturesTakeover`；`init` 内 `StallWatch.start()`（`:87-88`） | 有 `rustFeaturesTakeover`（`:165-176`）；`init` 无 StallWatch | 见 #4/#10/#11(b) |
| D10 | `ChunkTiming` | `:21-31`/`:37-71` 精简（MIXIN/GAP/inflight）；注释 `:10-12` 声明不打印假分项 | `:16-29`/`:42-60`/`:88-102` 全分项 + `featTick` 臂无关尺子 | 见 #11(c) |
| D11 | `build.gradle` 诊断属性映射 | `:133-140`（mixlog/chunktime/syncfill/stallwatch）；无 exec/maxinflight/skipair/wbcheck/caMin/maxBgThreads 的 `-P` 映射 | `:128-135`（mixlog/chunktime/maxBgThreads/syncfill）、`:232-234`（caMin）；无 stallwatch | 见 #11(g) |
| D12 | `versions/<ver>/rust/Cargo.toml` | `:1-15` 包名 `worldgen`（产物 `worldgen.dll`） | `:1-16` 包名 `worldgen1216`（产物 `worldgen1216.dll`，build.gradle rename 回 `worldgen.dll` 打包） | 薄壳命名（预期差异，非缺陷）；两者 `WorldgenRust` 依赖路径相同 |

## 3. 未核 / 需人工核

1. **1.20.1 侧结论性落盘状态**：`versions/1.20.1/docs/10-timewise-archive.md` grep `260911-05` **零命中**（尾块 = 260911-04，`:3127`）⇒ A1/A2 结论目前只在 `.investigations/a1-opt-pool-260911-05/`（record / a2-dim-gate / review），未进主题篇/时间线。是否属预期（结论性落盘纪律）需人工核。
2. **`-PmaxBgThreads` 是否构成 exec 的「形式不同但等价」**：1.21.6 `build.gradle:132-133` 有 `-PmaxBgThreads=N → -Dmax.bg.threads=N`（注释：「MC `Util.getAvailableBackgroundThreads` 上限」）。该参数对 MC 侧池的实际消费点、与 1.20.1 exec（自有池退出共享池）是否可视为等价 —— **本次未读 MC 侧消费点**，需人工核。
3. **`.gitignore` 覆盖风险**：本次 grep 可能跳过被忽略路径（build/、.gradle/ 等）。已用 glob `*perfprofile*` 与 grep 双向核对，但未逐目录枚举 ⇒ 1.21.6 侧「零残留」结论的**低风险未核面**（尤其 #9）。
4. **1.21.6 是否还有其它未门控 per-chunk println**：本次只核了 `CppBridge.java`（3 处 `[WG-FILL]`）与 `NoiseChunkGeneratorMixin.java`（4 处 `MIXLOG` 门控：`:109/155/192/249`）；`WgDiag`、各 `*Probe.java` 未逐一读 ⇒ 未核。
5. **1.20.1 侧 commit/工作树状态**：`8d8075a`（A2 补 end 指纹行）等提交号来自记录文件（`a2-dim-gate-260911-05.md:12`），本次**未核 git**（沙箱禁命令）⇒ 需人工核。
6. **1.21.6 薄壳 dll 打包 rename**：`Cargo.toml:6-8` 注释称 build.gradle `processResources` 把 `worldgen1216.dll` rename 回 `worldgen.dll`；本次未读 1.21.6 `build.gradle` 对应段 ⇒ 未核（载体 sanity 前置，见架构计划 §3 B3）。
7. **1.21.6 是否有 exec 的路线图/占位**：grep 无命中；是否已在 1.21.6 侧立项/记录需人工核（本次只读源码与少量 docs）。
8. **1.21.6 `[WG-CONTENT]` 缺失是否影响既有行为门结论**：`versions/1.21.6/docs` 中 260910-05/06 的 `common`/差异口径建立在何种载体上（是否依赖指纹门）——本次未逐段读 1.21.6 docs ⇒ 未核（与 B1 E5 复算口径相关）。
9. **`rustFeaturesTakeover` 与 1.20.1 的「不适用」判定**：1.20.1 无 features 接管 mixin，是否属「不适用」而非「缺失」——本次**不判断**，只列事实（§1 #10/#11(a)(b)）。

## 4. 建议的核查顺序（按风险，不含实施决策）

1. **#1 exec 模式**（用户点名；1.21.6 仅共享池形态，与 1.20.1 已定位的 vivo 根因同族；且 1.21.6 侧 `WG_FILL_POOL` 同时承载网格构建）。
2. **#2 P1 信号量**（与 #1 同文件同函数、互补路径；1.21.6 无 in-flight 上限）。
3. **#5 A1a 写回跳空气**（热路径每 chunk 98304 次 `setBlockState` 含空气；1.21.6 逐格写）。
4. **#6 A1b 诊断扫描门控**（1.21.6 每 chunk 恒执行全量扫描 + 16 点读回；与 1.20.1 已修形态同族）。
5. **#3 `[WG-CONTENT]` 指纹门**（1.21.6 **无行为门载体** ⇒ 后续任何 1.21.6 A/B 缺判据；A2 已立「载体先行」纪律，`a2-dim-gate-260911-05.md:13-14`）。
6. **#4 `StallWatch`**（1.21.6 停滞类问题缺观测面；沙箱禁 attach）。
7. **#10 + #11(a)(b) stageMask/features 语义**（先确认 1.21.6 接管范围决策是否仍为 0b011，再谈 mask 消费者差异）。
8. **#7 / #8 Rust 共享项**（确认即可，风险低：Cargo.toml 依赖路径 + `api.rs:38-43` + `worldgen_handle.rs:619-626`）。
9. **#9 perfprofile**（已无源码残留，仅需收尾声明）。
10. **#11(c)~(h) 仪器/wiring 差异**（按需：ChunkTiming 分项、gradle 属性映射、文件级差异）。

---

### 判定表摘要

- **缺失**：6 条（#1 exec、#2 P1、#3 `[WG-CONTENT]`、#4 StallWatch、#5 A1a、#6 A1b）
- **存在（等价）**：2 条（#7 A1d clamp、#8 ca_min/terrain_cache 门控——均为 Rust 共享层，Cargo.toml 依赖路径已核）
- **缺省值等价 + 语义消费者分叉**：1 条（#10 stageMask）
- **不适用**：1 条（#9 perfprofile，两版皆无源码残留）
- **反向分叉**：1 条（#11，含 8 个子项 a~h）
- **行为分叉（同函数不同实现）**：12 条（D1~D12，§2）
- **未核 / 需人工核**：9 条（§3）
