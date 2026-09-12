# 共享 Java 适配核抽取（范围 A）勘探地图 — 260912-01（scout，只读）

> **角色**：RE-Framework `recode.scout`（divergent，隔离子进程）。**只出事实与分类，不做裁决**——归属「建议」栏与缝设计均待 Phase 0 计划 / 用户拍板。
> **任务**：为「共享 Java 适配核抽取（范围 A）」产出抽取清单：19 个差异文件逐文件分类 + `CppBridge` 分歧 feature 级分解 + `WgCompat` 缝清单闭包 + 不可共享候选 + 「一个类只有一个家」映射表 + V1 预期差异清单 + 未核项。
> **纪律**：全部断言带 `文件:行` 锚点；无 shell，全部证据来自 `read`/`glob`/`grep`；未亲眼读到的写入 §7。参考的既有事实源已自核（下文凡引用均标「自核 ✅/引用」）。
> **范围**：范围 A = `wg/CppWorldgen`、`wg/bench/{CppBridge,BulkWb,StallWatch,ChunkTiming,CoreSwapFixHelper,WgDiag,BenchMod}`。探针（范围 B）与 mixin（范围 C）**本次不动**，仅登记差异性质。

---

## 0. 现盘状态（我到达工作树时的实测状态，会改变后续动作）

> 与任务书描述的「抽取尚未开始」不同：**Wave 1 已执行完毕**。以下为直接观察（非引用）。

| 事实 | 锚点 | 说明 |
|---|---|---|
| `java-core/` 已存在，含 3 个共享类 | `java-core/src/main/java/wg/CppWorldgen.java`、`java-core/src/main/java/wg/bench/WgDiag.java`、`java-core/src/main/java/wg/bench/BenchMod.java`、`java-core/src/main/java/wg/bench/.gitkeep`、`java-core/README.md` | glob 实测 |
| 这 3 个类已**从两版工程删除** | `glob versions/{1.20.1,1.21.6}/java/src/main/java/wg/CppWorldgen.java`、`.../wg/bench/{WgDiag,BenchMod}.java` = **0 命中** | 「一个类只有一个家」对已搬的 3 类成立 |
| 两版 `build.gradle` 均已接线共享源根 | `versions/1.20.1/java/build.gradle:31`（`srcDir '../../../java-core/src/main/java'`）、`versions/1.21.6/java/build.gradle:33`（同） | 两版一致 |
| 1.20.1 build.gradle 现为 **256 行**（我首次读时为 245 行、无 sourceSets） | 对比 `read` 两次结果 | ⇒ 该文件在**本 session 进行中被改**（主会话并行作业）；我的分类表一律以「当前树」为准 |
| V1 pre-freeze 基线与 Wave 1 判定已完成 | `.investigations/shared-java-core-260912-01/record-260912-01.md:23-41`（pre）、`:94-115`（Wave 1） | pre jar sha：1.20.1 `1027f4f6…`（1081 条目 / 57 类）、1.21.6 `16d5e5e7…`（1798 条目 / 54 类）；Wave 1 后**jar 逐字节相同** ⇒ PASS |
| 冻结件与工具 | `.tmp/shared-java-core-260912-01/pre/{pre-1.20.1.jar,pre-1.21.6.jar,manifest-*.tsv}`、`jar_manifest.py`、`jar_manifest_diff.py`、`class_home_check.py` | 引用 record §2 |
| `record-260912-01.md:89-90` §3「Phase 1 勘探」= 「（待回填）」 | 同文件 | ⇒ **本文件即该回填目标** |

**Wave 1 的 V1 结论（引用 record §4.1，对本文第 6 节极重要）**：3 个逐字节相同类移动后，两版 jar sha **未变**（条目差异 0）⇒ **「同源重编 + 换源根 ⇒ class 条目字节相同」在本载体上已被实证**，故本文 §6 中「逐字节相同文件的 class 条目预计不变」不再是纯推测。

---

## 1. 逐文件分类表

> 行数/diff 数取自计划 §2 实测（`架构计划-260912-01-共享Java适配核.md:54-66`，**已自核 ✅** 与任务书一致）；「差异构成」列由**逐行读两侧源码**得出，非按行数猜。
> 分类 ∈ {纯 API 改名, 加功能漂移, 注释差异, 硬分叉}；归属 ∈ {共享-原样, 共享+shim, 分版保留}。

### 1.1 范围 A 内（本次抽取对象）

| 文件 | 1.20.1 行 | 1.21.6 行 | 差异构成（行号锚点） | 分类 | 建议归属 | 依据 |
|---|---|---|---|---|---|---|
| `wg/CppWorldgen.java` | 108 | 108 | **0（逐字节相同）** | — | **共享-原样** | 已搬入 `java-core`（§0）；唯一外部引用 `wg.bench.CoreSwapFixHelper.extractNativeDll()` 在 `:31`/`:36` —— 共享后 FQN 不变 |
| `wg/bench/WgDiag.java` | 81 | 81 | **0** | — | **共享-原样** | 已搬入 `java-core`；调用点 = 8 个 mixin（全部仍为分版同名 FQN） |
| `wg/bench/BenchMod.java` | 98 | 98 | **0** | — | **共享-原样** | 已搬入 `java-core`；`fabric.mod.json` 两版 entrypoint 均 `wg.bench.BenchMod`（`fabric.mod.json:15` / `:8`） |
| `wg/bench/CppBridge.java` | 732 | 629 | +80/−183（38.6%）= **1.20.1 超前 6 组 + 1.21.6 超前 3 组 + 结构重排**（详见 §2） | 加功能漂移（**双侧超前**） | **共享+shim** | 逐行读两侧全文；全部 API 面两版可用（§3 附 C） |
| `wg/bench/BulkWb.java` | 400 | — | 1.20.1 独有 | 加功能漂移（1.20.1 独占 C 交付物） | **共享+shim** | 依赖 `wg.bench.mixin.ChunkSectionAccessor`（`:14`/`:170`/`:371`）⇒ 1.21.6 树内**必须补同形分版 mixin**（§3 缝 S-1） |
| `wg/bench/StallWatch.java` | 59 | — | 1.20.1 独有 | 加功能漂移（1.20.1 独占看门狗） | **共享-原样** | 全文件仅 JDK API（`Thread.getAllStackTraces()` `:41`、`System.getProperty` `:21`）⇒ **零 API 缝**；唯一差异 = 1.21.6 缺少 `CppBridge.init` 内的 `start()` 调用（§3 缝 S-5） |
| `wg/bench/ChunkTiming.java` | 72 | 103 | +55/−24 = **1.21.6 超前**：多 7 个分项 LongAdder + 7 个 `addXxx` 方法 + `featTick`/`FEAT_*` 臂无关尺子 + report 格式（1.20.1 为**精简版**，`ChunkTiming.java:10-12` 自声明「不打印恒 0 的假分项」） | 加功能漂移（1.21.6 超前） | **共享+shim** | 两侧方法集对照：1.20.1 有 `inflightEnter/Exit`(`:37-46`)、`enter/exit`(`:49-62`)、`report`(`:64-71`)；1.21.6 额外 `addJni/addWrite/addHmap/addScan/addBeard/addCarve/addFeat`(`:64-70`)、`featTick`(`:51-59`)、`FEAT_*`(`:46-48`)、`CARVE/FEAT`(`:22-23`) |
| `wg/bench/CoreSwapFixHelper.java` | 302 | 305 | +6/−3 = **纯注释**：同一处 3 行注释（1.20.1 `:44-46`）扩写为 6 行（1.21.6 `:44-49`）；**其余全文功能等价**（`isDataCacheStale` 两侧 `:75-86`/`:72-83` 逻辑同） | 注释差异 | **共享-原样**（择一叙述，属 Phase 0 裁决） | 逐行比对两侧全文；差异仅注释行数（净 +3 行 ⇒ 后续行号整体 +3） |

### 1.2 范围 A 外的差异文件（本次不动，登记性质）

| 文件 | 1.20.1 行 | 1.21.6 行 | 差异构成（锚点） | 分类 | 范围 |
|---|---|---|---|---|---|
| `wg/bench/mixin/NoiseChunkGeneratorMixin.java` | 418 | 253 | **硬分叉**：注入签名 5 参→4 参（`populateNoise(Executor,…)` 去首参；1.20.1 `:273` vs 1.21.6 `:81-86`）+ 调度形态不同（1.20.1 自有池 exec `:135`/P1 信号量 `:86`/`:102`/`wgDispatch` `:227-253`；1.21.6 `WG_FILL_POOL=Util.getMainWorkerExecutor().named("coreswap_fill_noise")` `:58` + `SYNCFILL` `:61` + 三分支 `:145`/`:183`/`:219`） | 硬分叉 | C（mixin 全留分版） |
| `wg/bench/DensityProbe.java` | 623 | 621 | +10/−12 = 7 处 `Identifier`/registry 改名（1.20.1 `:414/:509/:550/:559/:569/:583/:595`）+ **1 处 shape 变更**：`getEntry(RegistryKey)` 移除（1.20.1 `:597-600` 构造 `RegistryKey` 后 `getEntry(skey)`；1.21.6 `:597-598` 改 `getEntry(Identifier.of(...))`，注释明写「`getEntry(RegistryKey)` 已移除」） | 纯 API 改名 + shape | B（探针） |
| `wg/bench/RouterProbe.java` | — | — | +3/−3 = 3 处 `new Identifier` → `Identifier.of`（1.20.1 `:45/:48/:175`） | 纯 API 改名 | B |
| `wg/bench/BlockProbe.java` | — | — | +2/−2 = `new Identifier("deepslate")`（1.20.1 `:311`）、`getRegistryManager().get(BIOME)`（`:391`） | 纯 API 改名 | B |
| `wg/bench/Biome6Probe.java` | — | — | +2/−2 = `:17`/`:161` `new Identifier` → `.of` | 纯 API 改名 | B |
| `wg/bench/BiomeParamProbe.java` | — | — | +2/−2 = `:28` `getRegistryManager().get(…)`、`:33` `new Identifier(presetName)` | 纯 API 改名 | B |
| `wg/bench/BlobProbe.java` | — | — | +1/−1 = `:18` `new Identifier("the_nether")` | 纯 API 改名 | B |
| `wg/bench/mixin/BlobProbeMixin.java` | — | — | +1/−1 = `:81` `getRegistryManager().get(PLACED_FEATURE)` | 纯 API 改名 | B/C |
| `wg/bench/mixin/ConfiguredFeatureProbeMixin.java` | — | — | +1/−1 = `:60` `getRegistryManager().get(CONFIGURED_FEATURE)` | 纯 API 改名 | B/C |
| `wg/bench/DiagNetherProbe.java` | — | — | +1/−1 = `:18` `new Identifier("the_nether")` | 纯 API 改名 | B |
| `wg/bench/NoiseParamProbe.java` | — | — | +1/−1 = `:29` `getRegistryManager().get(NOISE_PARAMETERS)` | 纯 API 改名 | B |
| `wg/bench/NoiseProbe.java` | — | — | +1/−1 = `:52` `new Identifier(key)` | 纯 API 改名 | B |
| `wg/bench/SurfaceColDumpProbe.java` | — | — | +1/−1 = `:21` `new Identifier("the_nether")` | 纯 API 改名 | B |
| `wg/bench/LightDataDump.java` | — | — | +1/−1 = `:38` `state.getOpacity(null, null)`（1.20.1）→ `state.getOpacity()`（1.21.6，注释「签名去参」） | **API 签名变更**（语义见 §7-9） | B |
| `wg/bench/mixin/NoiseDumpProbeMixin.java` | — | — | +3/−2 = `@Inject` 目标签名：1.20.1 `:46-47` `populateNoise(Ljava/util/concurrent/Executor;…`，1.21.6 `:47-48` 起 `populateNoise(` + 形参去 `Executor`（`:53` vs `:54`） | 硬分叉（同 D1 家族） | C |
| `wg/bench/mixin/ServerLightingProviderMixin.java` | 240 | 241 | +5/−4 = 尾部语义：1.20.1 `:224-226` `setLightOn(true)` + `((ThreadedAnvilChunkStorageAccessor) chunkStorage).wgReleaseLightTicket(chunkPos)`；1.21.6 `:224-227` 删该调用 + 注释「1.21.2 管线重构：light ticket 机制整体移除（`releaseLightTicket` 全源码零命中，`ThreadedAnvilChunkStorage` 更名 `ServerChunkLoadingManager`）」 | 加功能漂移/API 移除 | C |

**范围 A 内无「硬分叉」项**（这是范围划分的直接依据：两个硬分叉文件都是 mixin）。

---

## 2. `CppBridge.java` 分歧的 feature 级分解（本任务最重要一项）

### 2.1 1.20.1 超前项（逐条双侧锚点）

| # | feature | 1.20.1 锚点 | 1.21.6 锚点（对应位置现状） | 能否「共享超集 + 分版常量」 |
|---|---|---|---|---|
| A1a | 写回跳空气 + `WBCHECK` 自检 + `WB_STALE_NONAIR` | `:536-547`（注释 + `WBCHECK` `:540` + `SKIPAIR` `:543` + 两个 AtomicLong `:544-547`）、`:596-606`（`id==0` 分支）、`:644-648`（destroy 汇总） | `:506-537` `writeChunk` 逐格**无 `id==0` 分支**；`grep skipair/wbcheck` 在 1.21.6 全树零命中 | ✅ 超集可行（`SKIPAIR` 默认值走分版常量，见缝 S-3）；`WBCHECK` 两版默认关、纯自检 |
| A1b | 诊断扫描门控：overworld 生产侧改 O(1) 短路；nether/end 的 nzBuf 全量扫描 + 16 点读回**整块**进 `MIXLOG` | overworld `:410-418`（`if (buf[0]==0)` 短路）；nether `:459-478`；end `:508-529`；注释 `:362-369` | overworld `:398-405`（**无条件** 98304 次全量扫描 + `buf-all-air`/`buf-sparse` 无条件 println）；nether `:442-443`（无条件 nzBuf 扫描）+`:452`（println 已 MIXLOG 门控）；end `:486-487`+`:495` | ✅ 超集可行，但 **1.21.6 生产行为会变**（去掉 unconditional 扫描/println）—— 见缝 S-6 与 §7-12 |
| A2 | 三维持内容指纹 `[WG-CONTENT]` + `wgBufHash`（FNV-1a 64，逐 int 逐字节） | 函数 `:381-391`；overworld `:419-425`；nether `:460-464`；end `:511-515`（`:509-510` 注释明写「end 缺失，A2 补」）⇒ **三维齐备** | `grep WG-CONTENT/wgBufHash/FNV` 1.21.6 全文**零命中**（629 行已逐行读） | ✅ 超集可行（`MIXLOG` 门控，默认关） |
| — | `StallWatch.start()` 接线 | `:88`（`init()` 内） | `init()` `:82-106` 内**无该调用** | ✅ 超集可行（类本体共享，见缝 S-5） |
| — | `writeChunk` 分派（bulk ↔ 逐块）+ `writeChunkPerBlock` + `stateById` 独立方法 | 分派 `:561-566`；旧路径 `:581-618`；`stateById` `:629-638`（含 M14 根因注释 `:620-628`）；destroy 汇总 `:649-656` | `writeChunk` 单一内联实现 `:506-549`（`STATE_BY_ID` 查表内联 `:519-531`，无独立方法） | ✅ 超集可行（取 1.20.1 结构；`BulkWb` 依赖 `CppBridge.stateById` 可见性 = 包私有 `static`，两版同包 `wg.bench` ✅） |
| — | `ChunkTiming` 分项钩子 | **无**（1.20.1 CppBridge 全文无 `ChunkTiming`） | `:385/:393` `addJni`；`:399/:402` `addScan`；`:406/:412` `addWrite`；`:540/:548` `addHmap` | ✅ 超集可行（1.20.1 侧钩子生效 ⇒ `[CHUNKTIME]` 报告格式变，见缝 S-7） |

### 2.2 1.21.6 独有项（逐条双侧锚点）

| # | feature | 1.21.6 锚点 | 1.20.1 对应位置现状 | 能否「共享超集 + 分版常量」 |
|---|---|---|---|---|
| B1 | `rustFeaturesTakeover()` 谓词（mask 非零且 bit1 清零） | 定义 `:165-176`；消费者 `wg/bench/mixin/ChunkGeneratorFeaturesMixin.java:68` | 1.20.1 全树 grep 零命中（无该方法、无消费者） | ✅ 超集可行：方法进共享核后 1.20.1 **无调用者**（消费者是 1.21.6 独有 mixin，留分版）⇒ 零行为影响 |
| B2 | `ChunkTiming` 分项钩子（见 A 表末行） | 同上 | 无 | ✅ 同 A 表末行 |
| B3 | carve 计时接线 | 在 1.21.6 独有 mixin（`mixin/NoiseChunkGeneratorTimingMixin.java`）内调 `ChunkTiming.addCarve`（依 `grep ChunkTiming` 1.21.6 命中，见 §3 附 C 调用点表） | 1.20.1 无该 mixin、无 `addCarve` 调用 | ✅ 超集可行（`addCarve` 方法进共享 `ChunkTiming`，1.20.1 侧恒 0） |
| B4 | `MIXLOG` 定义位置/文档（`:30-35`） | `:30-35` | `:362-370`（语义同，表述不同） | ✅（注释差异，无语义） |
| B5 | `writeChunk` 内联 `stateById` | `:519-531` | 独立方法 `:629-638` | ✅（取超集结构即可；两实现语义相同——1.21.6 内联段注释与 1.20.1 `stateById` 的 M14 说明同源） |

### 2.3 分解结论（事实层）

1. **两侧超前项互不冲突**：A1a/A1b/A2/StallWatch/分派结构与 B1/B2/B3 **可同时存在于一份超集源**——各自锚点区间不重叠（除 `writeChunk` 结构重排为同一函数，取 1.20.1 形态即可并保留 1.21.6 的 `ChunkTiming.addHmap` 钩子，锚点 `:540/:548` vs 1.20.1 `:569-575`）。
2. **`CppBridge` 的 diff 中「exec 模式 / P1 信号量 / maxinflight」在 `CppBridge.java` 内并不存在**（1.20.1 `CppBridge.java` 全文无 `exec`/`maxinflight`/`Semaphore` 字样；实测 `grep` 命中均在 `mixin/NoiseChunkGeneratorMixin.java:71/86/102/135/227`）。任务书把它们列为 `CppBridge` 超前项属**表述偏差**：它们是 `NoiseChunkGeneratorMixin`（范围 C）的超前项，本次抽取**不涉及**。
3. **`CppBridge` 的 3 组结构差异全部可由超集吸收，无需 `WgCompat` 方法缝**；真正需要分版参与的是**默认常量**（S-2/S-3/S-5/S-6/S-7），不是 API 适配。

---

## 3. `WgCompat` 缝清单（范围 A 闭包 + 范围外登记）

> 缝 = `缝名/签名 | 1.20.1 实现 | 1.21.6 实现 | 调用点清单（文件:行） | 是否编译期必需`。
> **闭包方法**：对范围 A 的 8 个共享候选文件，逐文件列出其引用的全部 MC API 与全部「版本相关常量/语义」，再对每个条目在两版 `mc_src_extract` 中核签名（结果见附 C）。凡两版等价的 API **不构成缝**，本节只登记差异。

### 3.1 编译期必需缝（缺则 1.21.6 编译失败）

| # | 缝 | 1.20.1 实现 | 1.21.6 实现 | 调用点清单 | 编译期必需 |
|---|---|---|---|---|---|
| **S-1** | `wg.bench.mixin.ChunkSectionAccessor`（分版 mixin 件，非 `WgCompat` 成员） | **已存在**：`versions/1.20.1/java/src/main/java/wg/bench/mixin/ChunkSectionAccessor.java`（56 行；`@Accessor` 4 个写侧 + 3 个读侧，`:29-55`；json 登记 `coreswap.mixins.json:29`；refmap 条目 `coreswap-refmap.json:28-33`） | **不存在**（`glob versions/1.21.6/java/src/main/java/wg/bench/mixin/ChunkSectionAccessor.java` 零命中；`coreswap.mixins.json` 无该条） | `BulkWb.java:14`（import）、`:170`（`(ChunkSectionAccessor)(Object) old`）、`:371`（同，读侧） | ✅ **必需**（共享 `BulkWb` 的 import 无法解析） |

**S-1 可行性的两版证据（自核，重要）**：
- `ChunkSection` 三计数 + 容器字段两版**同名同类型**：1.20.1 `ChunkSection.java:21-24` vs 1.21.6 `:21-24`（`nonEmptyBlockCount`/`randomTickableBlockCount`/`nonEmptyFluidCount` = `private short`；`blockStateContainer` = `private final PalettedContainer<BlockState>`）。
- **intermediary 名两版一致**（决定 refmap 条目相同）：1.21.6 yarn `1.21.6+build.1:v2` 映射表 `.gradle-home/caches/fabric-loom/1.21.6/net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2/mappings-base.tiny:88534-88538`：`field_12877`→`nonEmptyBlockCount`、`field_12881`→`nonEmptyFluidCount`、`field_12882`→`randomTickableBlockCount`、`field_12878`→`blockStateContainer`，与 1.20.1 refmap（`coreswap-refmap.json:29-32`）**逐字相同**。
- ⇒ 结论：1.21.6 侧补一份**同形副本**（含 json 登记 + refmap 自然重建）即可，无需改共享源。（是否照 D-2「mixin 全留分版」处理 = Phase 0 裁决；本 scout 只给事实。）

### 3.2 非编译期（语义/默认值）缝

| # | 缝 | 1.20.1 实现 | 1.21.6 实现 | 调用点清单 | 编译期必需 |
|---|---|---|---|---|---|
| **S-2** | `WgCompat.BULKWB_ON`（分版默认常量） | `BulkWb.java:72` `static final boolean ON = !"0".equals(System.getProperty("coreswap.bulkwb"))` ⇒ **默认 true** | 无该类；D-4(i) 定为 **默认 false**（1.21.6 不改变行为） | 分派 `CppBridge.java:562`；`BulkWb.writeSections` 内 `LOG`/`WBTEST` 等分支 `:140/:145/:146/:161/:169/:176/:185/:195/:264/:268/:272/:325/:385` | ❌（但语义必需） |
| **S-3** | `WgCompat.SKIPAIR_ON`（A1a 默认） | `CppBridge.java:543` `SKIPAIR = !"0".equals(System.getProperty("coreswap.skipair"))` ⇒ **默认 true**；`WBCHECK` `:540`、`WBLOG` `:550` 默认关 | 1.21.6 无对应项；D-4(i)「1.21.6 默认关」需明确覆盖 `SKIPAIR`（否则共享超集一上来就改 1.21.6 行为） | `:597`（WBCHECK 自检）、`:605`（`if (SKIPAIR) continue`）、`:644`（destroy 汇总） | ❌ |
| **S-4** | `WgCompat.MIXLOG` 语义对齐 | `CppBridge.java:370`（overworld 短路 + nether/end 整块门控、含 `[WG-CONTENT]`） | `CppBridge.java:35`（同 prop 名；但 overworld 仍无门控、nether/end 仅 println 被门控） | `:419`、`:460`、`:511` | ❌ |
| **S-5** | `WgCompat.STALLWATCH_WIRED`（或等价「是否调用 `StallWatch.start()`」） | `CppBridge.java:88` 调用 | 1.21.6 `init()` 无调用（`:82-106`） | 仅 `:88` | ❌（默认关，纯诊断；但**属行为差异**） |
| **S-6** | **门控语义缝**（A1b）：`MIXLOG` 内是否含「全量扫描 + 16 点读回 + `[WG-CONTENT]`」整块 | 整块（`:419-425`/`:460-478`/`:511-529`） | overworld 与净界/末地**部分或全部在门控外**（`:398-405`；`:442-443`+`:452`；`:486-487`+`:495`） | 同上 | ❌（**1.21.6 生产热路径行为变化项**） |
| **S-7** | `WgCompat` 的「分项计时存在性」常量（可选设计） | 1.20.1 `ChunkTiming.java:10-12` 自声明「不打印恒 0 的假分项」 | 1.21.6 全分项（`:91-101` report） | 报告 `ChunkTiming.java:64-71`（1.20.1）/`:88-102`（1.21.6） | ❌（若共享超集，1.20.1 将打印 `jni/write/hmap/carve/feat` 恒 0 列 ⇒ **违反其自身声明**；是否加「capability 常量」属 Phase 0 裁决） |
| **S-8** | `resolveStageMask` 缺省值 | `CppBridge.java:71` `return 0b011` | `CppBridge.java:77` `return 0b011` | `:69-74` / `:75-80`；消费者 `:82`、`:130`、`:150`，`rustFeaturesTakeover` `:174` | ❌ **两版相同（自核 ✅）⇒ 不构成缝** |
| **S-9** | `StructureTerrainAdaptation` 值域 | 4 常量 `NONE/BURY/BEARD_THIN/BEARD_BOX`（`StructureTerrainAdaptation.java:6-9`） | **5 常量**（+:11 `ENCAPSULATE("encapsulate")`） | `CppBridge.java:342`（`CppWorldgen.setBeardifier(..., p, ...)`，`terrainOrdinal` 出自 `Piece.terrainAdjustment.ordinal()`） | ❌ **Java 侧无需缝**（共享代码只转发 ordinal）；语义影响在 Rust 侧（本文范围外，§7-7） |

### 3.3 范围外（B/C）缝登记（本次不动，供后续范围 B/C 复用）

| # | 缝 | 1.20.1 | 1.21.6 | 调用点（文件:行） | 编译期必需 |
|---|---|---|---|---|---|
| X-1 | `Identifier` 构造 | `new Identifier(String[, String])` | `Identifier.of(String[, String])` | 探针 9 文件 15 处：`BlobProbe.java:18`、`Biome6Probe.java:17,:161`、`BiomeParamProbe.java:33`、`DensityProbe.java:414,:509,:550,:559,:569,:583,:599`、`DiagNetherProbe.java:18`、`BlockProbe.java:311`、`RouterProbe.java:45,:48,:175`、`NoiseProbe.java:52`、`SurfaceColDumpProbe.java:21` | ✅（范围 B 抽取时） |
| X-2 | `DynamicRegistryManager.get(key)` | `DynamicRegistryManager.java:38` `default <E> Registry<E> get(...)`（可返 null） | `:27` `default <E> Registry<E> getOrThrow(...)`（**改名**） | `BiomeParamProbe.java:28`、`DensityProbe.java:595`、`BlockProbe.java:391`、`NoiseParamProbe.java:29`、`mixin/BlobProbeMixin.java:81`、`mixin/ConfiguredFeatureProbeMixin.java:60` | ✅ |
| X-3 | `Registry.getEntry(RegistryKey)` | `Registry.java:363` 存在；`getOrThrow(RegistryKey)` 在 `:263` | `getEntry(RegistryKey)` **移除**（只剩 `getEntry(int)` `:343`、`getEntry(Identifier)` `:349`）；`getOrThrow` 改名 `getValueOrThrow` `:256` | `DensityProbe.java:600`（1.20.1）→ `:598`（1.21.6 用 `getEntry(Identifier.of(...))`） | ✅ |
| X-4 | `Util.getMainWorkerExecutor()` 返回型 | `Util.java:229` `public static ExecutorService getMainWorkerExecutor()`（**无** `.named()`） | `Util.java:262` `public static NameableExecutor getMainWorkerExecutor()`（`:103`/`:234`，**有** `.named()`） | `mixin/NoiseChunkGeneratorMixin.java:58`（1.21.6）；1.20.1 `:68` 仅取池 | ✅（范围 C 抽取时；`WgCompat.FILL_POOL`） |
| X-5 | `BlockState.getOpacity` 签名 | `getOpacity(BlockView, BlockPos)`（`LightDataDump.java:38` 传 `null,null`，异常兜底 `-1` `:40`） | `getOpacity()`（`LightDataDump.java:38`） | `LightDataDump.java:38` | ✅（范围 B） |
| X-6 | `populateNoise` 注入签名 | 5 参（首参 `Executor`） | 4 参 | `mixin/NoiseChunkGeneratorMixin.java:273`（1.20.1）/`:86`（1.21.6）；`mixin/NoiseDumpProbeMixin.java:46-47` / `:47-48` | ✅（范围 C） |
| X-7 | light ticket 机制 | `ThreadedAnvilChunkStorageAccessor`（1.20.1 独有 mixin）+ `wgReleaseLightTicket` | 机制移除（`ServerLightingProviderMixin.java:224-227`） | 1.20.1 `mixin/ServerLightingProviderMixin.java:226` | ✅（范围 C，且需语义替代） |

### 3.4 附 C：范围 A 共享文件的 API 可用性核对结果（**不构成缝的条目**）

> 方法：两侧 `data/mc_src_extract/` 逐文件核签名 + 行号。**「1.21.6 现状可编译」由现有构建产物佐证**（`versions/1.21.6/java/build/classes/java/main/wg/bench/CppBridge.class` 等存在，且 record §2.1 记录两版 `BUILD SUCCESSFUL`）。

| API | 1.20.1 | 1.21.6 | 判定 |
|---|---|---|---|
| `Chunk.getSectionArray()` | `Chunk.java:163-164` public 返活引用 | `Chunk.java:171-172` public | ✅ 等价（B2 scout 已述；自核行号） |
| `Chunk.getSection(int)` | `:167-168` | `:175-176` | ✅ |
| `ChunkSection.isEmpty()/setBlockState(x,y,z,st)` | `:95-96` / `:56-57` | `:103-104` / `:64-65` | ✅ |
| `ChunkSection` 公开构造器 `(PalettedContainer, ReadableContainer<RegistryEntry<Biome>>)` | `:27` | `:35` | ✅ |
| `PalettedContainer` 3 参公开构造器 `(IndexedIterable, T, PaletteProvider)` | `:110` | `:116` | ✅ |
| `PalettedContainer.PaletteProvider.BLOCK_STATE` | `:420` | `:427` | ✅ |
| `PalettedContainer.readPacket(PacketByteBuf)` | `:203`（`:210` `buf.readLongArray(storage.getData())`） | `:209`（`:216` `buf.readFixedLengthLongArray(storage.getData())`） | ⚠ **方法名不同但线格式相同**：1.21.6 `readFixedLengthLongArray(long[])` 无长度前缀读取（`:771-772`/`:800-803`），其调用方 `readPacket` 前已读 palette 段；1.20.1 `readLongArray(long[])` 读 VarInt 长度（`:924-935`）——**两侧 `readPacket` 逐字节契约一致**（BulkWb 只构造写侧 buffer，不依赖读侧名）⇒ Java 共享源**不受影响** |
| `PalettedContainer.DataProvider` 为包私有 record | `:398` | `:405` | ✅ 同（⇒ 必须继续走 `readPacket` 路线，两侧原因相同） |
| `BLOCK_STATE.createDataProvider` 位宽 switch（0→SINGULAR;1-4→ARRAY 恒4bit;5-8→BI_MAP bits;else→ID_LIST ceilLog2） | `:424-427` | `:431-434` | ✅ **逐行相同**（BulkWb `:194-280` 自复刻这 6 行合法） |
| `PackedIntegerArray(int,int)` / `set` / `getData` / `getElementBits` | `:252/:297/:316/:326` | `:252/:297/:316/:326`（**同行号**） | ✅ |
| `PacketByteBuf(ByteBuf)` / `writeByte` / `writeVarInt` / `writeLongArray(long[])` | ctor `:?`；`writeLongArray` `:859-864`（VarInt 长度 + longs） | ctor `:218`；`writeLongArray` `:736-743` → `writeFixedLengthLongArray`（`:751-757`）**同为 VarInt 长度 + longs** | ✅ 线格式等价（BulkWb `:251-261` 写侧两版可用） |
| `Block.STATE_IDS`（`IdList<BlockState>`） | `Block.java:93` | `Block.java:98` | ✅ |
| `MathHelper.ceilLog2` | `:334` | `:373` | ✅ |
| `Heightmap.populateHeightmaps(Chunk,Set<Type>)` + 6 个 Type 常量 | `:37`；`:142-149` | `:43`；`:150-158` | ✅ |
| `NoiseConfig.getNoiseRouter()` / `DensityFunction.UnblendedNoisePos` / `DensityFunctionVisitor` | `:152`/`:177`/`:105-106` | `:152`/`:177`/`:105-106`（**同行号**） | ✅ |
| `StructureAccessor` / `StructureWeightSampler` 包路径与 yarn 名 | `net/minecraft/world/gen/`；`pieceIterator` `:33`/`junctionIterator` `:34`/`record Piece(box,terrainAdjustment,groundLevelDelta)` `:172` | 同路径；`:33`/`:34`/`:174` | ✅ |
| **`CppBridge.feedBeardifier` 的反射名对（intermediary）** | `field_28744`/`field_28745`/`comp_682`/`comp_683`/`comp_684`/`method_35415..35420`/`method_16609..16611` | **1.21.6 映射表实测同名**：`mappings-base.tiny:109695-109696`（field_28744/745）、`:109736-109741`（comp_682/683/684）、`:98188-98264`（method_35415..35420）、`:4297-4301`（method_16609..16611） | ✅ **不构成缝**（此前怀疑的「intermediary 名按版本不同」经实测否定） |
| `Registries.BLOCK.get(int)/getId/getRawId/size` | `Registry.java:215/219/:417/:422` | `:211/219/:378/:383` | ✅（且 1.21.6 现状 `CppBridge` 已在用，编译产物为证） |
| JDK-only 类（`StallWatch`/`ChunkTiming`/`CoreSwapFixHelper`/`WgDiag`） | — | — | ✅ 零 MC API ⇒ 零 API 缝 |

---

## 4. 不可共享候选清单

| 候选 | 范围 | 为什么（编译期/语义） | 建议处置 |
|---|---|---|---|
| **范围 A 内：无** | A | 8 个共享候选文件的全部 MC API 两版等价（附 C），全部版本差异可由分版 mixin 件（S-1）+ 分版常量（S-2/S-3）吸收 | ⇒ **范围 A 抽取不存在「无法用共享源 + 缝表达」的项**（这是本次最重要的负面结论，可关闭 Phase 0 对「硬分叉」的担心） |
| `wg.bench.mixin.ChunkSectionAccessor` | A（依赖件） | **不是「不能共享」而是「按 D-2 留分版」**；Mixin 类必须与 `coreswap.mixins.json` 的 `package` + 每版 refmap 同生（KB #12：mixin 包内禁非 mixin 类；refmap 由 mixin AP 按版本生成） | **留分版**：1.21.6 新建同形副本（内容可与 1.20.1 逐字节相同，`@Accessor` 字面量为 yarn 名 ⇒ 两版通用，证据见 S-1） |
| `wg/bench/mixin/*`（24 件，含 2 个 1.21.6 独有） | C | 注入签名硬分叉（X-6）+ 调度形态硬分叉（exec/P1 vs shared pool）+ API 移除（X-7）+ `NameableExecutor`（X-4） | **留分版**（D-2 已定；范围 C 另案） |
| 16 个探针文件 | B | 纯 API 改名可缝（X-1~X-3、X-5），但 `LightDataDump.getOpacity` 的**数值语义**变化需先核对（§7-9） | **留分版**（D-2 已定；范围 B 另案） |
| `ChunkTiming` 的「1.20.1 不打印假分项」意图 | A | **语义意图**无法由「共享超集」自动表达（超集会打印恒 0 列） | 需 Phase 0 决策：①接受恒 0 列并撤 1.20.1 注释；②`WgCompat` 暴露 capability 常量让 `report()` 分版列（等于在共享类内留分支 ⇒ 与「共享文件零版本分支」纪律张力，需裁决） |
| `CppBridge` 共享超集对 1.21.6 的**行为改变**（S-5/S-6） | A | 非编译期，但 `MIXLOG` 门控整块化会移除 1.21.6 每 chunk 的无条件全量扫描与 println（**生产热路径**）；`StallWatch.start()` 接线新增（默认关） | 需 Phase 0 / HOOK-2 确认是否接受（D-4(i) 只声明了「1.21.6 默认关」，未逐项列 `SKIPAIR`/`StallWatch`/`MIXLOG` 门控范围） |

---

## 5. 「一个类只有一个家」映射表（范围 A）

| 类（FQN） | 目标路径 | 归属 | 两版都存在？ | 语义相同？ | 备注 |
|---|---|---|---|---|---|
| `wg.CppWorldgen` | `java-core/src/main/java/wg/CppWorldgen.java` | 共享 | 是 | **相同**（逐字节） | ✅ 已就位（§0） |
| `wg.bench.WgDiag` | `java-core/src/main/java/wg/bench/WgDiag.java` | 共享 | 是 | **相同** | ✅ 已就位 |
| `wg.bench.BenchMod` | `java-core/src/main/java/wg/bench/BenchMod.java` | 共享 | 是 | **相同** | ✅ 已就位；引用全部探针类（FQN 不变，留分版无碍） |
| `wg.bench.StallWatch` | `java-core/src/main/java/wg/bench/StallWatch.java` | 共享 | **否**（1.20.1 独有） | — | 类本体零版本差；接线差见 S-5 |
| `wg.bench.CoreSwapFixHelper` | `java-core/src/main/java/wg/bench/CoreSwapFixHelper.java` | 共享 | 是 | 代码相同，**注释行数不同** | 择一叙述 ⇒ 另一版 `LineNumberTable` 变（§6 预期差异） |
| `wg.bench.BulkWb` | `java-core/src/main/java/wg/bench/BulkWb.java` | 共享 | **否**（1.20.1 独有） | — | 依赖 S-1；默认开关 S-2 |
| `wg.bench.ChunkTiming` | `java-core/src/main/java/wg/bench/ChunkTiming.java` | 共享 | 是 | **不同**（1.20.1 精简 / 1.21.6 全分项） | 取超集 ⇒ 恒 0 列问题（S-7） |
| `wg.bench.CppBridge` | `java-core/src/main/java/wg/bench/CppBridge.java` | 共享 | 是 | **不同**（§2：1.20.1 超前 6 组 / 1.21.6 超前 3 组） | 取超集后两版行为均有变化，逐项见 §2/§3 |
| `wg.bench.WgCompat` | `versions/<ver>/java/src/main/java/wg/bench/WgCompat.java` | **分版**（同名两类，各一版各一家） | 是（新建） | **不同**（分版常量） | ⚠ 必须置于 `wg.bench`，**不得**放 `wg.bench.mixin`（KB #12） |
| `wg.bench.mixin.*`（24 件） | `versions/<ver>/java/src/main/java/wg/bench/mixin/` | **分版** | 部分 | 部分 | 本次不动；1.21.6 需**新增** `mixin/ChunkSectionAccessor.java`（S-1） |
| `wg.bench.<探针类>`（16 件） | `versions/<ver>/java/src/main/java/wg/bench/` | **分版** | 是 | 部分（X-1~X-3） | 本次不动 |

**「两版都存在但语义不同、因而必须靠缝/常量」的共享类**：`CppBridge`（默认常量 + 门控范围）、`BulkWb`（默认开关；1.21.6 新托管）、`ChunkTiming`（分项存在性）、`CoreSwapFixHelper`（仅注释⇒无语义缝）。`CppWorldgen`/`WgDiag`/`BenchMod`/`StallWatch` **无语义差**。

---

## 6. V1 等价门（条目级 sha）预期差异清单

> 基线（引用 record §2.1，**已冻结**）：1.20.1 `1027f4f6…`（1081 条目 / 57 类）；1.21.6 `16d5e5e7…`（1798 条目 / 54 类）。
> 判据（计划 §V1）：**非 mixin 条目 sha 全等**，差异条目须逐条声明为「预期」。
> 参照事实（record §4.1）：Wave 1 的 3 个逐字节相同类移动后，**两版 jar sha 完全未变** ⇒ 「同源换源根重编不改变 class 字节」已被实证；故本节的「预计不变」有实验支撑，「预计变」为静态推理。

### 6.1 预计**会变**的条目

| 版本 | 条目 | 变化类型 | 依据 |
|---|---|---|---|
| 1.20.1 | `wg/bench/CppBridge.class` | 改（源码=超集：新增 `ChunkTiming` 钩子 `:385/:393/:399/:402/:406/:412/:540/:548` 形态、`MIXLOG` 区块化） | §2 |
| 1.20.1 | `wg/bench/ChunkTiming.class` | 改（取 1.21.6 超集：+7 `LongAdder`、+7 `addXxx`、+`featTick`、report 格式） | §1.1 / S-7 |
| 1.20.1 | `wg/bench/CoreSwapFixHelper.class` | 改（**注释行数 3→6 使全部后续行号位移 ⇒ `LineNumberTable` 全变**）——若 Phase 0 选择保留 1.20.1 注释，则改为 1.21.6 侧变 | `:44-46` vs `:44-49` |
| 1.20.1 | `wg/bench/WgCompat.class` | **新增** | §5 |
| 1.21.6 | `wg/bench/CppBridge.class` | 改（源码=超集：新增 A1a/A1b/A2/StallWatch 接线 + `writeChunk` 分派结构） | §2 |
| 1.21.6 | `wg/bench/CoreSwapFixHelper.class` | 改（与 1.20.1 同因，方向相反） | 同上 |
| 1.21.6 | `wg/bench/BulkWb.class` + `wg/bench/BulkWb$TL.class` | **新增** | §5 |
| 1.21.6 | `wg/bench/StallWatch.class` | **新增** | §5 |
| 1.21.6 | `wg/bench/WgCompat.class` | **新增** | §5 |
| 1.21.6 | `wg/bench/mixin/ChunkSectionAccessor.class` | **新增**（S-1） | §3.1 |
| 1.21.6 | `coreswap.mixins.json` | 改（`mixins` 数组新增 `ChunkSectionAccessor`；现 24 条 `:6-31`） | S-1 |
| 1.21.6 | `coreswap1216-refmap.json` | 改（新增 `wg/bench/mixin/ChunkSectionAccessor` 段，形态同 `coreswap-refmap.json:28-33`） | S-1 + 映射实测 |
| 1.20.1 | `coreswap-refmap.json` / `coreswap.mixins.json` | **预计不变**（不新增 1.20.1 mixin） | — |
| 两版 | `wg/bench/ChunkTiming.class`（1.21.6 侧） | **预计不变**（若直接以 1.21.6 现文件为共享源） | Wave 1 实证 |
| 两版 | 其余 26 个逐字节相同文件对应条目 + 全部 mixin 条目 | **预计不变** | Wave 1 实证（同类情形） |
| 两版 | `fabric.mod.json`（`${version}` 展开后）、`native/worldgen.dll`、`worldgen-data/**`、`META-INF/*` | **预计不变** | 本抽取不触碰资源；注意 dll 为版本相关输入（1.20.1 `worldgen.dll` / 1.21.6 `worldgen1216.dll`→rename），两版本就不同 |

### 6.2 已核对的「预期差异 vs 意外差异」判别依据

| 观察 | 判别 |
|---|---|
| refmap 条目**名**两版不同（1.20.1 `coreswap-refmap.json` vs 1.21.6 `coreswap1216-refmap.json`） | **预期，非缺陷**：根因 = `base.archivesName` 不同（`1.20.1/java/build.gradle:11` `'coreswap'` vs `1.21.6/java/build.gradle:9` `'coreswap1216'`）；loom 版本两版相同（`fabric-loom 1.10.5`，`build.gradle:2` 各） |
| mixin 类条目在「源未改」时是否变 | 预计不变（Wave 1 已证同族情形）；若变 ⇒ 按 record §4.1 的写死判据 **MUST 补跑 V2（run 回归）** |
| 条目**总数**变化 | 1.20.1 预计 **+1 class 条目**（`WgCompat`；无其他新增文件）；1.21.6 预计 **+4 class 条目**（`BulkWb`、`BulkWb$TL`、`StallWatch`、`WgCompat`）+1（`ChunkSectionAccessor`）= **+5**；两版资源条目数不变 |

---

## 7. 未核项 / 需运行期核实的断言（诚实声明）

| # | 未核断言 | 为什么未核 | 后果/待动作 |
|---|---|---|---|
| 1 | 1.20.1 与 1.21.6 两版**全部 28 个逐字节相同文件**在移入 `java-core` 后 class 条目是否逐条不变 | 无 shell，不能跑 V1 工具；仅 Wave 1 的 3 个类有实证（record §4.1） | 按 record 既有流程逐波次 V1 判定即可（已具备工具与基线） |
| 2 | `CppBridge` 共享超集后 1.21.6 侧 `wg/bench/CppBridge.class` 的具体差异条目清单 | 需实际实现超集后才可测 | Phase 2 后按 `jar_manifest_diff.py` 逐条归类 |
| 3 | 1.21.6 侧新增 `ChunkSectionAccessor` 后 refmap 段的**确切**内容 | 未跑 mixin AP；仅由 1.20.1 refmap + 1.21.6 映射表推断（S-1） | Phase 2 实测；若出现 `Cannot find target method`/mixin 装载失败按 KB #25/#40 处理 |
| 4 | `BulkWb` 在 1.21.6 的**运行期**正确性（Tier 1 指纹门 / 派生计数 / region 对拍 / 分项计时） | 静态等价已核（附 C），但 `PalettedContainer.readPacket` 的运行时位宽契约在 1.21.6 未实测 | 需 1.21.6 侧自身 A/B（计划非目标 §1 已声明「每项需 1.21.6 自身验证」） |
| 5 | 1.21.6 `PalettedContainer.readFixedLengthLongArray` 的 size 上限语义（`:790-798` 的 `readableBytes/8` 限幅）与 1.20.1 `readLongArray(toArray,maxSize)` 是否**完全**等价 | 只读了 1.21.6 `:200-232`/`:771-803` 与 1.20.1 `:196-226`/`:900-935`，未逐行对拍限幅分支 | 只影响 vanilla 网络/存档读路径，**不影响** BulkWb 自建 buffer（写侧等价已核） |
| 6 | `CppBridge.feedBeardifier` 在 **1.21.6 生产（remap 后）** 的反射是否成功 | 已核 intermediary 名两版一致（附 C，映射表锚点），但未运行 1.21.6 生产 jar | 低风险；生产验证时观察 `[BEARD]`/异常日志 |
| 7 | `StructureTerrainAdaptation.ENCAPSULATE`（1.21.6 新增 ordinal 4）对 Rust `setBeardifier` 消费者的影响 | Rust 侧不在本任务范围；仅核了 Java 枚举差（S-9） | 若 Rust 侧按 ordinal 分支 ⇒ 需另案核对（KB #87 已登记该枚举家族） |
| 8 | 共享 `ChunkTiming` 超集是否使 1.20.1 的 `[CHUNKTIME]` 出现恒 0 列、是否违反 `ChunkTiming.java:10-12` 的自声明 | 设计裁决项（非事实缺失） | 交 Phase 0 / HOOK-2（§4 候选表末行） |
| 9 | `LightDataDump` 的 `getOpacity(null,null)→-1` 兜底（1.20.1）与 `getOpacity()`（1.21.6）产出的 `light_data.json` **opacity 值域**是否相同 | 未逐块对比产物；两侧都读 `getOpacity` 但参数/兜底不同 | 范围 B 开工前须核（可能改变光照数据表） |
| 10 | `ServerLightingProviderMixin` 删除 `releaseLightTicket` 后 1.21.6 光照接管的**功能等价性** | 仅核了 diff 构成与注释声明 | 范围 C 另案（KB 需登记「机制整体移除」） |
| 11 | `MIXLOG` 区块化后 1.21.6 生产**实测**性能影响（移除每 chunk 无条件扫描/println） | 无 shell，不能运行基准 | 属 D-4(i)「1.21.6 逐项翻转各自 A/B」范畴 |
| 12 | 1.21.6 jar 内**实际条目布局**（如 refmap 条目名是否为 `coreswap1216-refmap.json`） | 无 shell，不能列 jar；仅由 `build/classes/java/main/coreswap1216-refmap.json` 存在推断 | 以 `jar_manifest.py` 产出的 tsv 为准（已存在） |
| 13 | `ChunkSection` 的 `@Mutable` 对 1.21.6 的 final 字段（`blockStateContainer`）是否同 1.20.1 生效 | 字段声明两版一致（`private final`），但未运行 1.21.6 mixin apply | Phase 2 编译+运行验证（KB #25 前兆判据） |

---

## 附 A：范围 B（探针）差异性质汇总（一行级）

16 个探针差异文件中：**14 个 = 纯 1.21.6 API 改名**（`new Identifier`→`Identifier.of` 15 处；`DynamicRegistryManager.get`→`getOrThrow` 6 处）；**1 个 = API 签名变更**（`LightDataDump:38` `getOpacity(View,BlockPos)`→`getOpacity()`，⚠ 数值语义待核 §7-9）；**1 个 = shape 变更**（`DensityProbe:597-600`→`:597-598`，`getEntry(RegistryKey)` 移除 ⇒ 改 `getEntry(Identifier)`）。**无加功能漂移、无硬分叉**。

## 附 B：范围 C（mixin）差异性质汇总

| 类别 | 文件 | 一句话 |
|---|---|---|
| 硬分叉·注入签名 | `NoiseChunkGeneratorMixin`、`NoiseDumpProbeMixin` | `populateNoise` 去首参 `Executor`（1.20.1 5 参 / 1.21.6 4 参） |
| 硬分叉·调度形态 | `NoiseChunkGeneratorMixin` | 1.20.1 自有有界池 + P1 信号量 + `wgDispatch` 三级优先级（`:71/:86/:102/:135/:227-253`） vs 1.21.6 vanilla 共享池 `NameableExecutor.named` + `SYNCFILL` 回退（`:58/:61`） |
| API 移除·语义替代 | `ServerLightingProviderMixin` | light ticket 机制 1.21.2 起移除 ⇒ 1.20.1 尾部 `releaseLightTicket` 无对应物 |
| 独有件 | `ChunkGeneratorFeaturesMixin`、`NoiseChunkGeneratorTimingMixin`（1.21.6） | `rustFeaturesTakeover` 消费点 / carve 计时；1.20.1 无对应 |
| 独有件 | `ChunkSectionAccessor`、`ThreadedAnvilChunkStorageAccessor`（1.20.1） | 前者被共享 `BulkWb` 依赖（S-1）；后者随 light ticket 机制消失 |
| 纯 API 改名 | `BlobProbeMixin`、`ConfiguredFeatureProbeMixin`、`SquarePlacementModifierMixin` 等 | 1~2 行 registry 改名 |
| mixin json | 两版各 24 条（1.20.1 `:6-31` / 1.21.6 `:6-31` 数组），条目集合差 = {+`ChunkSectionAccessor`,`ThreadedAnvilChunkStorageAccessor`} vs {+`NoiseChunkGeneratorTimingMixin`,`ChunkGeneratorFeaturesMixin`} | 与目录 24 个 mixin 文件**逐条对应**（KB #40 情形**未出现**：两版 json 条目数 = 目录文件数 = 24，自核 ✅） |

---

## 交付边界声明

- 本文件为**只读勘探产物**，落 `.investigations/shared-java-core-260912-01/`（scout 契约：不写 `.artifacts/`，不改任何源码）。**本次未修改任何文件**。
- 归属列均为**建议**；`WgCompat` 成员命名（`BULKWB_ON`/`SKIPAIR_ON`…）为待定草案，非定案。
- 置信度：**draft**（本文为事实与分类，非结论）；`candidate` 授予前 MUST judge（Phase 3），`confirmed` 留给用户。
- 未核项 13 条（§7），其中 3 条为「需运行期/构建期实证」、3 条为「设计裁决项」、4 条为「范围 B/C 另案」、3 条为「jar 布局/dll 资源细节」。
