# scout-map 审查意见 — 260912-01（judge，只出意见，不改任何文件/状态）

> **审查对象**：`.investigations/shared-java-core-260912-01/scout-map.md`（284 行，status **draft**）
> **级别**：SHOULD（计划 §7「Phase 1 抽取清单」）
> **审查基线（三源）**：① 交付快照 = scout-map.md 本体；② 现树一手源（源码 / 资源 / `data/mc_src_extract` / 冻结清单 `.tmp/shared-java-core-260912-01/pre/*.tsv`）；③ 计划 `架构计划-260912-01-共享Java适配核.md`（§1/§3/§5/§13/§14）+ 过程记录 `record-260912-01.md`（§2/§4.1）
> **纪律声明**：本意见**不含任何状态修改动作**；scout-map 保持 **draft**（推荐：本轮**不授予 candidate**，见条件 C1/C2）。审查者读到的与 scout 表述不符处一律如实登记（见 §1 ⚠ 明细）。

---

## 审查结论

**PASS-with-conditions**（scout-map 抽核到的**事实性断言无一处被证伪**；分类/闭包/清单结论成立，但存在 **1 处执行边界前置条件缺失**、**2 处 §6 清单缺项**、**1 处 §7 诚实声明账目失真**）

### 条件（MUST，Phase 2 之前逐条完成）

| # | 条件 | 为什么必须 | 建议动作（不改 scout 结论，只补记录） |
|---|---|---|---|
| **C1** | **S-1 越界事实必须回 HOOK-2 取批准**：托管共享 `BulkWb` 到 1.21.6 需**新增分版 mixin 件** `versions/1.21.6/java/src/main/java/wg/bench/mixin/ChunkSectionAccessor.java` + 改 `coreswap.mixins.json`，这落在计划 §13 「mixin 全部本计划不动；任何超范围改动 MUST 回 HOOK-2」的**禁止面**内 | scout 已给足事实（§3.1，我已独立复核 S-1 全部前提成立），但只写「Phase 0 裁决」，**未点明它触发 HOOK-2**；若不取批准就实施，Phase 2 首个波次即构成超范围改动 | 计划追加一条 §14 式补登：① 声明 S-1 是范围 A 的**编译期前置**；② 明确它是「分版 mixin 件（内容可与 1.20.1 逐字节相同）」而非范围 C 的「mixin 体抽取」；③ 记 HOOK-2 结论 |
| **C2** | **§6.1「会变条目」清单补 `wg/bench/CppBridge$1.class`（两版）**，并显式声明 1.20.1 侧 `BulkWb.class`/`BulkWb$TL.class` 是否变（取决于 S-2 常量落点） | 该条目**存在于两版 pre jar**（冻结清单 1.20.1 `:22`、1.21.6 `:20` 均为 `wg/bench/CppBridge$1.class`）；共享 `CppBridge` 必然改变源行布局 ⇒ 匿名类 `LineNumberTable` 位移 ⇒ 至少 1.21.6 侧必变。清单漏项会让 §14.1 **V1b-2**（变更条目逐条声明）在 Wave 2 判据上直接失败 | scout 补 row（若 scout 已不再可派，则由 Phase 2 执行者在 wave 记录里补声明，judge 复核） |
| **C3** | **语义方向项与「等价移动」波次必须拆开**：`ChunkTiming` 取 1.21.6 超集（→1.20.1 打印恒 0 列，S-7）、`MIXLOG` 门控整块化（→1.21.6 热路径去掉无条件扫描/println，S-6）、`StallWatch.start()` 接线（S-5）三项**都不是** D-4(i) 覆盖的「1.21.6 默认关」范畴，且**改变 1.20.1 既有诊断口径**（`[CHUNKTIME]` 格式）与 1.21.6 生产行为 | 计划 §13 的抽取前提是「抽取 = 等价重构（走 V1 条目级 sha 门）」；把语义翻转混进移动波次会同时破坏 V1 的字节锚与 V3 的可比性口径（§9.7） | Phase 2 分两波：wave-a = 纯等价移动（含 WgCompat 常量占位，两侧默认值与现行为一致）；wave-b = 语义统一项，逐项 commit + 各版 A/B；`ChunkTiming` 超集**方向**（1.21.6→1.20.1）须 HOOK-2/HOOK-3 明确批准 |
| **C4** | **§7 尾部账目与表内条目对齐**：现写「3 条需运行期/构建期实证、3 条设计裁决、4 条 B/C、3 条 jar 布局/dll」与表内实际不符（实际约 **7 条需运行期/构建期**、**1 条设计裁决**（#8）、**3 条 B/C 另案**（#7/#9/#10）、**1 条 jar 布局**（#12））；且 **§7-12 应降级为「可静态已核」**（冻结清单 tsv 已存在，我一次 grep 即得 `coreswap1216-refmap.json`） | 诚实声明账目失真会让后续「哪些真需要运行期」的排期判断失准；#12 属「其实已可静态核实却推给运行期」的错位 | 修正 §7 尾部统计；#12 标注「已由 `manifest-1.21.6-pre.tsv:6` 静态核实：条目名 = `coreswap1216-refmap.json`」 |

### 建议（SHOULD，不阻塞 Phase 2，但建议随 C2 一并修）

- **S-a**：§1.1 三行「分类 = —」（0 diff 文件）与 §1.2 的「API 签名变更 / shape / API 移除」均**超出 §1 自声明分类域** {纯 API 改名, 加功能漂移, 注释差异, 硬分叉}；建议加值域外标注或补第五类「单版独有 / 无语义差」。
- **S-b**：§1.1 `StallWatch` 行「分类 = 加功能漂移」与 §5 末段自述「类本体零版本差，接线差见 S-5」互相拉扯——漂移在**调用点**（`CppBridge.java:88`）而不在该文件；建议分类改为「无语义差（接线差见 S-5）」。
- **S-c**：§2.2-B4「`MIXLOG` 定义位置/文档 `:30-35`」表述略宽（javadoc `:30-34` + 常量定义 `:35`）；内容成立，仅范围精度问题。

---

## 1. 锚点抽核（21 处，覆盖 14 个文件/资源；全部亲自 `read`/`grep`，未采信转述）

| # | scout 断言（节） | 我实际读到的 | 判定 |
|---|---|---|---|
| 1 | `StallWatch` 全文件仅 JDK API；`System.getProperty :21`、`Thread.getAllStackTraces() :41`（§1.1） | `StallWatch.java:21` `System.getProperty("coreswap.stallwatch")`；`:41` `Thread.getAllStackTraces()`；全文件 59 行、无 MC import | ✔ |
| 2 | 1.20.1 `ChunkTiming` 精简版自声明「不打印恒 0 的假分项」`(:10-12)`；方法集 `inflightEnter/Exit :37-46`、`enter/exit :49-62`、`report :64-71`（§1.1） | 逐条命中，行号精确；文件 72 行 | ✔ |
| 3 | 1.21.6 `ChunkTiming` 多 7 分项：`addJni..addFeat :64-70`、`featTick :51-59`、`FEAT_* :46-48`、`CARVE/FEAT :22-23`、report `:88-102`（§1.1） | 逐条命中（`:64-70` 恰为 7 个 `addXxx`）；文件 103 行；report 格式含 `featInterval` | ✔ |
| 4 | 1.20.1 `CppBridge` A1a：`WBCHECK :540`、`SKIPAIR :543`、两个 AtomicLong `:544-547`、`id==0` 分支 `:596-606`、`if (SKIPAIR) continue :605`、destroy 汇总 `:644-648` | 全部命中，行号精确 | ✔ |
| 5 | 1.20.1 `CppBridge` A1b：overworld O(1) 短路 `:410-418`；`MIXLOG :370`；A2 `wgBufHash :381-391`；`[WG-CONTENT]` 整块在 `MIXLOG` 内 `:419-425` | 全部命中；`:414` `if (buf[0] == 0)` 短路 + 首个非零即停；`:381-391` 为 FNV-1a 64 逐 int 逐字节 | ✔ |
| 6 | 1.20.1 `CppBridge` 分派 `:561-566`；`stateById :629-638`（M14 注释 `:620-628`）；`BulkWb.reportSummary` 在 destroy `:650` | 全部命中；`:562` `if (BulkWb.ON)` / `:565` `writeChunkPerBlock(...)` | ✔ |
| 7 | 1.21.6 `CppBridge` B4 `MIXLOG :30-35`；A1b「overworld `:398-405` 无条件全量扫描 + 无条件 println」；`rustFeaturesTakeover :165-176` | `:35` 常量定义（javadoc `:30-34`）；`:399-405` 全量 `for (k<buf.length)` 扫描 + `buf-all-air`/`buf-sparse` **未受 MIXLOG 门控**；`:173-176` 谓词 | ✔（B4 范围略宽，见 S-c） |
| 8 | `BulkWb` 依赖 `ChunkSectionAccessor`：`import :14`、`:170` 写侧强转、`:371` 读侧强转；`ON` 默认 true `:72`（§1.1/S-2/S-1） | 三条全中；`:72` `static final boolean ON = !"0".equals(System.getProperty("coreswap.bulkwb"))` ⇒ 默认 true | ✔ |
| 9 | 1.21.6 `NoiseChunkGeneratorMixin:58` `WG_FILL_POOL=Util.getMainWorkerExecutor().named("coreswap_fill_noise")`、`:61` `SYNCFILL`（§1.2/§3.3 X-4） | 命中（`:57-58` 常量声明、`:61` SYNCFILL） | ✔ |
| 10 | `rustFeaturesTakeover` 消费者 = `mixin/ChunkGeneratorFeaturesMixin.java:68`（§2.2 B1） | `:68` `if (CppBridge.rustFeaturesTakeover()` | ✔ |
| 11 | `CoreSwapFixHelper` 差异仅一处注释：1.20.1 `:44-46`（3 行）→ 1.21.6 `:44-49`（6 行），净 +3 ⇒ 后续行号整体 +3（§1.1） | 两侧注释实测 3/6 行；`isDataCacheStale(target)` 1.20.1 `:47` vs 1.21.6 `:50` = **+3** ⇒ 位移方向与量级成立 | ✔ |
| 12 | `ChunkSectionAccessor`（1.20.1）56 行、`@Accessor` 4 写侧 + 3 读侧 `:29-55`；json 登记 `coreswap.mixins.json:29`（§3.1） | 56 行；写侧 `:29-42`、读侧 `:48-55`；json 第 29 行 = `"ChunkSectionAccessor"` | ✔ |
| 13 | refmap `coreswap-refmap.json:28-33` 为 `wg/bench/mixin/ChunkSectionAccessor` 段，intermediary 名与 1.21.6 映射一致（§3.1） | `:28` 段名、`:29-32` 四字段 = `field_12878/12877/12881/12882`（与 scout 引用的 yarn tiny 逐字一致）；**实测是 `build/classes/java/main/` 下的产物**，非源码目录 | ✔（位置描述属实但属构建产物） |
| 14 | 两版 mixin json 各 24 条、数组 `:6-31`；条目集合差 = {+ChunkSectionAccessor,+ThreadedAnvilChunkStorageAccessor} vs {+NoiseChunkGeneratorTimingMixin,+ChunkGeneratorFeaturesMixin}（附 B） | 逐条数：两版均 24 条、数组确实跨 `:6-31`；集合差完全吻合（1.20.1 `:29/:30`、1.21.6 `:8/:9`） | ✔ |
| 15 | 1.21.6 `fabric.mod.json:8` / 1.20.1 `:15` entrypoint = `wg.bench.BenchMod`（§1.1） | 两侧命中（1.20.1 `:15`、1.21.6 `:8`） | ✔ |
| 16 | `ChunkSection` 三计数 + 容器字段两版同名同类型 `:21-24`（§3.1，S-1 可行性核心） | 1.20.1 `:21-24` 与 1.21.6 `:21-24` **逐行相同**（`private short`×3 + `private final PalettedContainer<BlockState>`） | ✔ |
| 17 | `CppWorldgen` 唯一外部引用 `CoreSwapFixHelper.extractNativeDll()` 在 `:31`/`:36`（§1.1） | `java-core/src/main/java/wg/CppWorldgen.java:31`、`:36` 两处调用；文件 108 行 | ✔ |
| 18 | S-8 `resolveStageMask` 缺省两版相同：1.20.1 `:71`、1.21.6 `:77` 均 `return 0b011`（§3.2） | 两侧命中（1.20.1 `:69-74`、1.21.6 `:75-80`） | ✔ |
| 19 | 附 C ⚠ 行：1.21.6 `PalettedContainer.readPacket :209` + `:216 readFixedLengthLongArray`（方法名不同、写侧线格式等价）（§3.4） | `PalettedContainer.java:209` `readPacket`、`:216` `buf.readFixedLengthLongArray(data.storage.getData())`；`1.20.1` 对应调用名不同但 BulkWb 只走**写侧**构造 | ✔（⚠ 行判定成立） |
| 20 | 1.21.6 全树 `WG-CONTENT/wgBufHash/skipair/wbcheck/StallWatch` **零命中**（§2.1 A1a/A2、§2.1 StallWatch 行） | 对 `versions/1.21.6/java/src/main/java` 一次 grep：**No matches found**（五项全零） | ✔ |
| 21 | §0：两版 `build.gradle` 已接线共享源根（1.20.1 `:31`、1.21.6 `:33`）；1.20.1 build.gradle 现 256 行；`java-core/` 含 3 类 | `1.20.1/java/build.gradle:31` 与 `1.21.6/java/build.gradle:33` 均为 `srcDir '../../../java-core/src/main/java'`；1.20.1 文件 256 行；`java-core/**` glob = `wg/CppWorldgen.java`、`wg/bench/WgDiag.java`、`wg/bench/BenchMod.java` | ✔ |

**✘ 0 处；⚠ 1 处**（#7 的 B4 范围表述略宽，见 S-c；不改变结论）。**未发现任何被证伪的事实性断言。**

---

## 2. 分类一致性

**结论：一致（无互相矛盾），仅分类域有未定义取值 + 1 处归属表述与自述张力。**

1. **§1.1 表「分类」× §2/§3 正文**：逐行交叉无矛盾——
   - `CppBridge`「加功能漂移（双侧超前）」↔ §2.1（1.20.1 超前 6 组）× §2.2（1.21.6 超前 3 组）；§2.3 结论「两侧超前项互不冲突」与 §3.2 S-2..S-7 的「默认常量/门控」定位自洽。
   - `BulkWb`「加功能漂移（1.20.1 独占 C 交付物）」↔ §3.1 S-1（编译期必需）+ S-2（默认常量）；§4 将「依赖件 mixin」与「不可共享」分开登记，用语一致。
   - `ChunkTiming`「1.21.6 超前」↔ §3.2 S-7「超集会打印恒 0 列」；§5 末段「必须靠缝/常量」与 §3.2 一致。
   - `CoreSwapFixHelper`「注释差异」↔ §6.1 行号位移项一致。
2. **「建议归属」× 计划 §3 D-2 范围 A（8 个类）**：**8/8 全覆盖、无漏项、无越界新增类**（CppWorldgen/WgDiag/BenchMod/CppBridge/BulkWb/StallWatch/ChunkTiming/CoreSwapFixHelper），`WgCompat` 按 D-2 记为**分版**（§5），探针 16 件与 mixin 24 件全部判为范围外（§1.2/§4/§5）——与 D-2「mixin 全部留分版、探针不做」一致。
3. **越界风险（唯一一处，非表格归属问题而是执行边界）**：§3.1 S-1 的落地形态要求**在 1.21.6 新增一个分版 mixin 文件 + json 登记**，这在计划 §13「mixin 全部本计划不动」之外 ⇒ 触发 **HOOK-2**（见条件 C1）。scout 已标「Phase 0 裁决」，只是没点出 HOOK-2 通道。
4. **分类域未定义取值（⚠，S-a/S-b）**：§1 头声明分类 ∈ 4 类，但表内出现「—」（3 行）、§1.2 出现「API 签名变更」「shape」「加功能漂移/API 移除」；另 `StallWatch` 行的「加功能漂移」与 §5「类本体零版本差」措辞拉扯（漂移在调用点）。

---

## 3. 缝清单闭包主张的可证伪性（逐条尝试证伪）

**待证伪命题（§3.4/§4/§4.1）**：「范围 A 内无硬分叉、无不可共享项，8 个共享候选文件的**全部 MC API 两版等价**」。

**证伪动作（4 路）与结果**：

| 路 | 动作 | 结果 |
|---|---|---|
| R1 | 对两版 `CppBridge` 的 `^import` 行与 `net.minecraft.* / net.fabricmc.*` 全限定引用做**对称差**（grep） | **空差**：两版 import 集（`:3-14`）逐行相同；FQN 使用集逐条同源（仅行号偏移）。无 1.20.1 独有 API 引用。**未证伪** |
| R2 | 对 1.20.1 独有代码路径（A1a/A1b/A2/分派/`writeChunkPerBlock`/`stateById`）逐 API 在 1.21.6 侧找等价物 | 全部存在：`ChunkSection.getBlockState(int,int,int)` 1.21.6 `:48`、`setBlockState` `:64`、`isEmpty` `:103`、`BlockState.isOf(Block)` 经 `AbstractBlock.AbstractBlockState` 1.21.6 `:1593`（1.20.1 `:1515`）、`Block.getDefaultState()` 1.21.6 `:716`（1.20.1 `:676`）、`Chunk.getPos()` `:209`、`Chunk.getSection(int)` `:175`、`Registries.BLOCK.get/getId/getRawId`（1.21.6 现行代码已在用）、`Heightmap.populateHeightmaps`、`ChunkTiming.addXxx`（共享类新增）。**未证伪** |
| R3 | 三个「已共享且两版都在编译」的类（`CppWorldgen`/`WgDiag`/`BenchMod`）是否含版本相关 API | `CppWorldgen` 用 `net.fabricmc.loader.api.FabricLoader`；`BenchMod` 用 `net.fabricmc.api.ModInitializer` + `fabric.api.event.lifecycle.v1.ServerLifecycleEvents`；`WgDiag` JDK-only。三者已由 **Wave 1 实证**（两版 jar sha 不变）⇒ 两版编译通过，API 面兼容。**未证伪（且获实证支撑）** |
| R4 | 「无不可共享项」是否被 S-1 反证 | **部分成立**：`BulkWb` 的 import 无法在 1.21.6 解析，必须**在范围 A 之外新增一份分版 mixin**（S-1）——这不是「不可共享」，但**是范围 A 无法自闭合的编译期前置**。scout 已如实登记为编译期必需缝 |

**闭包结论：成立（本审查覆盖的 API 面内未发现 1.21.6 缺失/形状变更的 API）**，附两条限定：
- **限定 1（证据完备性）**：scout 的闭包论证依赖 §3.4「附 C」，但附 C **并非穷举**——`Chunk.getPos()`、`ChunkSection.getBlockState`、`BlockState.isOf`、`Block.getDefaultState()`、`io.netty.*`、`FabricLoader`/`ServerLifecycleEvents` 等**未列入附 C**（后两者由 Wave 1 实证兜底）。建议把「未列 API」补入附 C 或加一行「附 C 非穷举，兜底证据 = Wave 1 两版编译实证」。
- **限定 2（边界）**：闭包成立**以 S-1 被批准实施为前提**（见 C1）；否则 `BulkWb` 在 1.21.6 不可编译，闭包不成立。

---

## 4. §6 预期差异清单的完备性（按当前树实测）

**逐类核对（对照冻结清单 `.tmp/shared-java-core-260912-01/pre/manifest-{1.20.1,1.21.6}-pre.tsv`，条目数 1081/1798 已独立复核 = record §2.1 一致）**：

| §6.1 覆盖项 | 实测核对 | 判定 |
|---|---|---|
| 新增类 `WgCompat`（两版）、1.21.6 `BulkWb`/`BulkWb$TL`/`StallWatch`/`ChunkSectionAccessor` | 1.20.1 pre 清单 `:17/:18`（BulkWb$TL/BulkWb）`:37`（StallWatch）`:50`（ChunkSectionAccessor）**在**；1.21.6 pre 清单**五项全不在**（我 grep 零命中）⇒ 与「新增 / +4+1」吻合 | ✔ |
| 1.21.6 `coreswap.mixins.json`、`coreswap1216-refmap.json` 改 | mixin json 集合差已核（§1 抽核 #14）；refmap 条目名 `coreswap1216-refmap.json` 由 pre 清单 `:6` 证实 | ✔ |
| `fabric.mod.json` / `native/worldgen.dll` / `worldgen-data/**` / `META-INF/*` 不变 | 这些条目在 pre 清单中均存且本抽取不触碰资源；dll 版本相关说明正确 | ✔ |
| `CoreSwapFixHelper` 注释位移（两版） | §1 抽核 #11 已证实 3→6 行、后续 +3 ⇒ 两版 `LineNumberTable` 均变 | ✔ |
| 新增类计数（1.20.1 +1、1.21.6 +5） | 与 pre 清单存在性核对一致 | ✔ |
| **`wg/bench/CppBridge$1.class`（两版）** | **pre 清单 1.20.1 `:22` / 1.21.6 `:20` 均存在**；§6.1 只列 `CppBridge.class`，**未列此条目**。共享 `CppBridge` 必改源行布局 ⇒ 匿名类行号表位移 ⇒ 至少 1.21.6 侧必变 | **✘ 缺项**（C2） |
| **1.20.1 `BulkWb.class`/`BulkWb$TL.class`** | 若 S-2 把默认常量迁到 `WgCompat.BULKWB_ON`（scout §3.2 S-2 的表述即如此），1.20.1 `BulkWb` 源码必改 ⇒ 该两条目**变**；§6.1 未列（且 §6.1 亦未把 1.20.1 `BulkWb` 归入「不变」组，属**未表态**） | **✘ 条件缺项**（C2） |
| 1.20.1 `StallWatch.class` 不变 | 类本体零版本差（§5）且不引用 WgCompat（S-5 只改调用点）⇒ 不变成立，未列可接受 | ✔ |
| 1.20.1 `ChunkTiming` 改 / 1.21.6 `ChunkTiming` 预计不变 | 与 §1.1/§3.2 一致；但 §6.1 把「取 1.21.6 超集」当**既定方向**，而 §3.2 S-7 声明为 Phase 0 裁决项 ⇒ **内部张力**（C3 建议同步） | ⚠ |

**§6 结论：清单**不完备**——漏 `CppBridge$1.class`（硬缺），未表态 1.20.1 `BulkWb*`（条件缺）；其余覆盖正确。「会变的条目」判据本身（V1b-2 逐条声明）方向正确，但清单不完整会直接导致 Wave 2 判据缺声明。**

---

## 5. §7 未核项质量（13 条）

| # | scout 定性 | judge 判定 | 理由 |
|---|---|---|---|
| 1 | 28 个逐字节相同文件移入后条目是否不变（运行期） | **合理** | 真需 V1 工具跑；已有工具与基线 |
| 2 | 共享超集后 1.21.6 差异条目清单 | **合理** | 需实现后才可测 |
| 3 | 1.21.6 新增 `ChunkSectionAccessor` 的 refmap 段确切内容 | **合理** | 需 mixin AP 实跑 |
| 4 | `BulkWb` 在 1.21.6 的运行期正确性 | **合理** | 静态等价已核，运行期未测 |
| 5 | `readFixedLengthLongArray` 限幅语义是否完全等价 | **可降级为已核（或将范围收窄）** | 两侧实现文本已可读并已定位行号（1.21.6 `:771-803` / 1.20.1 `:900-935`），逐行对拍是**静态可完成**动作；且 scout 自述「不影响 BulkWb 写侧」⇒ 要么补对拍、要么直接声明「不影响范围 A，关闭」 |
| 6 | `feedBeardifier` 1.21.6 生产 remap 后反射是否成功 | **合理** | intermediary 名已静态核，remap 后行为需运行期 |
| 7 | `StructureTerrainAdaptation.ENCAPSULATE` 对 Rust 消费者影响 | **合理（域外登记）** | 诚实移交 Rust 域，未伪装已核 |
| 8 | 恒 0 列是否违反 1.20.1 自声明 | **合理** | scout 自己标「设计裁决项（非事实缺失）」——事实部分（超集有 7 adder、1.20.1 无钩子）已在 §1.1/§3.2 确立，标注准确 |
| 9 | `LightDataDump` opacity 值域是否相同 | **合理** | 需逐块对比产物（范围 B 另案） |
| 10 | 删除 `releaseLightTicket` 后 1.21.6 光照等价性 | **合理** | 范围 C 另案 |
| 11 | `MIXLOG` 区块化后的实测性能影响 | **合理** | 需基准运行 |
| 12 | 1.21.6 jar 内实际条目布局（含 refmap 条目名） | **表述错位（可降级为已核）** | scout 以「无 shell，不能列 jar」为由推给未知，但 **§0 自己登记的冻结清单 `manifest-1.21.6-pre.tsv` 就是 jar 条目表**；我一次 grep 即读到 `:6 coreswap1216-refmap.json` ⇒ 该断言**静态已可核实**（C4） |
| 13 | `@Mutable` 对 1.21.6 final 字段是否生效 | **合理** | 字段声明两版一致（`private final` 已核），但 mixin apply 需运行期（KB #25 前兆） |

**附加发现（账目）**：§7 尾部统计「3 条需运行期/构建期实证、3 条设计裁决项、4 条 B/C 另案、3 条 jar 布局/dll 资源细节」与表内实际分布不符（实际约 **7 / 1 / 3 / 1**，且 #7 是 Rust 域外、#12 是 jar 布局、无「dll 资源细节」条目）⇒ **统计失真**（C4）。除 #12 外，**未发现「把已可静态核实的事实写成设计裁决」的错位**；#8 的设计裁决标注经核准确。

---

## 6. 与计划的一致性（exec/P1 措辞冲突定性）

**事实层（已抽核）**：1.20.1 `CppBridge.java` 全文 `grep exec|Semaphore|maxinflight|Executor` = **零命中** ⇒ scout 的反驳成立；`exec/P1/wgDispatch` 的实测宿主是 `mixin/NoiseChunkGeneratorMixin.java`（scout §2.3-2 与附 B 的锚点，我抽核了同族的 1.21.6 `:58/:61` 与 `mixin/ChunkGeneratorFeaturesMixin:68`）。

**判定：这是计划的表述缺陷，scout 未越界。** 理由：

1. 计划 §3 D-3.1 把「exec、P1」与 A1a/A1b/A2/StallWatch/C-bulk 并列写入「**1.20.1 超前项 → 进共享核**」；而同一计划 §13 批准的执行边界是「mixin 全部本计划不动，超范围 MUST 回 HOOK-2」，§4 Phase 2 也只做范围 A。**D-3.1 的措辞把范围 C 的项写成了「进共享核」**，与 D-2/D-3.2 自相矛盾——属计划侧表述不精确（另见 §1 目标1 与 §2 表格同样并列 exec/P1）。
2. scout 的处置是**只陈述事实 + 明确不做**（§2.3-2「它们是 `NoiseChunkGeneratorMixin`（范围 C）的超前项，本次抽取不涉及」），未提出任何 mixin 改动，也未把 exec/P1 纳入范围 A 归属表 ⇒ **无越界**。
3. 同类表述偏差还有一处（计划 §4 Phase 1 说对「19 个差异文件」分类）：scout §1.1+§1.2 恰为 3+16 = **19** 行覆盖 ⇒ 此项**无偏差**。

**建议**：按 §14 的「追加式补登」纪律，在计划中加一行：D-3.1 的 `exec/P1`（及任何 `NoiseChunkGeneratorMixin` 内项）**属范围 C（分版保留），不在范围 A 的「进共享核」清单内**；原正文不改。

---

## 7. 风险与缺口：若照本清单执行 Phase 2，最可能翻车的 3 个点

| # | 翻车点 | 为什么 | Phase 2 前应补什么 |
|---|---|---|---|
| **R-1** | **S-1 超范围实施 ⇒ Wave 2 判据 + mixin 装载双重风险** | ① 走 C1 未批准的路径直接新增 1.21.6 mixin = 违反计划 §13 执行边界；② 一旦落地，1.21.6 侧新增 mixin class + refmap 段 + json 条目（§6.1 三类条目），按 §14.1 **V1b-3 必须补跑 V2+V3**；③ `@Accessor` 对 `private final blockStateContainer` 走 `@Mutable`，1.21.6 mixin apply 未验（§7-13，KB #25 前兆判据）——若失败，`BulkWb` 在 1.21.6 直接不可用，而 `BulkWb` 是范围 A 的成员 | 先取 HOOK-2；再把 S-1 作为**独立前置波次**：单独提交 mixin 件 + json，先跑 1.21.6 `build` + dev run 验 mixin APPLY 无 FAILED，再托管 `BulkWb`（默认关） |
| **R-2** | **V1b-2「逐条声明」在未声明条目上直接失败** | §6.1 漏 `CppBridge$1.class`，且 1.20.1 `BulkWb*` 未表态；共享 `CppBridge` 改行布局后匿名类必变（至少 1.21.6）。V1 工具只能「归类」，计划 §14.1 明确「**归类 ≠ 声明**」⇒ 判据会卡在一条 scout 清单里没有的差异条目上，Wave 2 只能回炉 | 补 §6.1（C2）；并在 wave 记录里预先声明「每版唯一字节锚」（§14.1 V1b-1）：对 1.20.1 取 `ChunkTiming` 锚、对 1.21.6 取 `ChunkTiming` 锚——`CppBridge` 两侧都变 ⇒ 须显式说明「为何两侧都必须变」 |
| **R-3** | **把语义翻转混进等价波次 ⇒ 行为/口径双失守** | `ChunkTiming` 取 1.21.6 超集 = 1.20.1 侧 `[CHUNKTIME]` 打印恒 0 列并改报告格式（既违反 1.20.1 类头自声明，又使**新旧日志口径不可比**，触 §9.7）；`MIXLOG` 整块化 = 1.21.6 每 chunk 无条件 98304 次扫描与 println 被移除（生产热路径行为变化，D-4(i) 未覆盖）；`StallWatch.start()` 接线属新增行为面。三者若与「机械移动」同波交付，则 V1（字节等价）与 V3（行为等价）同时失去判别力，且回退粒度变粗 | 按 C3 拆波；`ChunkTiming` 超集方向与 `MIXLOG` 门控范围**先落 HOOK-2 决策**；1.20.1 侧恒 0 列建议取「capability 常量让 `report()` 分版列」或「显式撤注释 + 记录口径变更」，二选一须留痕 |

（另：`WgCompat` 类名/成员命名仍是草案（scout 自述），不影响结论，但 V6「一个类只有一个家」双向核对必须在 wave 记录里体现。）

---

## 审查范围声明

**本次审查我「未」做的事（诚实声明）**：

1. **未跑任何构建/工具**：无 shell ⇒ 未跑 `gradle`、未跑 V1/V2/V3、未重算 jar sha、未跑 `jar_manifest_diff.py`。record §2.1 的整 jar sha（`1027f4f6…`/`16d5e5e7…`）我**只核了条目数 1081/1798**（对冻结清单计数）与关键条目存在性，**未核 sha 值本身**。
2. **未验运行期**：`BulkWb`/`CppBridge` 超集的运行期行为、mixin APPLY、refmap 重建结果、dll 加载、指纹门 V3 全部未验（属 Phase 2/2.5）。
3. **未整读大文件**：按读取预算，`CppBridge.java` 只读了若干行窗口（1.20.1 `:80-94/:360-429/:530-619/:618-657`；1.21.6 `:26-43/:160-179/:394-413`），**未整读 732/629 行**；`BulkWb.java` 只读 `:8-21/:70-75/:165-174/:366-373`。因此 diff 行数断言（+80/−183、+55/−24、+6/−3 等）**未独立复核**（依赖 record/plan 的实测记录）。
4. **未核 §1.2 的 16 个探针/mixin 行**：`DensityProbe:597-600`、`LightDataDump:38`、`RouterProbe:45/48/175`、`BlobProbe:18`、`ServerLightingProviderMixin:224-227` 等**逐条行号未抽核**（仅核了 1.21.6 `NoiseChunkGeneratorMixin:58/61`，以及 mixin 集合差与目录一致性）。范围外登记项对本审查结论影响有限，但其行号精度**未验证**。
5. **未核**：yarn `mappings-base.tiny:88534-88538`/`:109695-...` 等映射表锚点（scout 引用，我未读该文件）；`data/mc_src_extract` 中除 `ChunkSection`/`Chunk`/`PalettedContainer`/`Block`/`AbstractBlock` 外的其余附 C 行号；KB #12/#25/#40/#54/#116 正文；`.artifacts/index.yaml`（本次为 `.investigations/` 产物，无 artifacts 侧交付）；噪声卡历史（未查 Anchorlaw §3 台账）。
6. **未核** scout §5 关于 24 件 mixin 与目录「逐条对应」的**目录侧枚举**（只核了 json 条目数与集合差，未逐一比对目录文件清单）。

**抽核覆盖率**：抽核**锚点 21 处**、跨 **14 个文件/资源**（`StallWatch`、两版 `ChunkTiming`、两版 `CppBridge`、`BulkWb`、`CoreSwapFixHelper`、`ChunkSectionAccessor`、两版 `coreswap.mixins.json`、`coreswap-refmap.json`、两版 `fabric.mod.json`、两版 `build.gradle`、`CppWorldgen`、`ChunkGeneratorFeaturesMixin`、1.21.6 `NoiseChunkGeneratorMixin`、两版 `ChunkSection`、1.21.6 `PalettedContainer`/`Block`/`AbstractBlock`、冻结清单 tsv×2）；文内带 `文件:行` 的断言量级约 **100+ 条** ⇒ 按锚点计覆盖率约 **20%**（偏重 §1/§2/§3/§6 的核心可证伪断言；§1.2 范围外登记行与映射表类锚点未覆盖）。**✘ 0 / ⚠ 1（表述精度）**。

**纪律声明**：本审查**未修改任何源码/文档/知识库**，**未改动任何 status**（scout-map 保持 **draft**）；仅新建本文件。审查意见为**建议**，`candidate`/`confirmed` 由主会话与用户裁决；条件 C1–C4 属「补记录/取批准」，**不构成对 scout 事实结论的驳回**。
