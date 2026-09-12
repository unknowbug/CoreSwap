# scout-map — 1.21.6「强制 bulk 写回臂」崩解缺陷勘探（260912-02 / Phase 1）

> 角色：**scout（勘探，只读）**。本文件是**唯一**写入物；未改动/新建任何其他文件，未 git add/commit，未构建、未运行游戏、未动 dll。
> 纪律：**不做裁决**——不写 confirmed/candidate，不下「根因就是 X」的定论；只列事实（带 `文件:行号` / 日志行号）、候选、判别方式。
> 上游依据：`.investigations/000-架构设计/架构计划-260912-02-1.21.6强制bulk缺陷定位.md`（K1–K5）、`.investigations/shared-java-core-260912-01/{errors-260912-01.md, record-260912-01.md}`、`knowledge/INDEX.md` + 引用条目 #22/#23/#24/#25/#54/#55（正文不复制）。
> 一手证据载体（本次实际读到的）：`java-core/src/main/java/wg/bench/{CppBridge,BulkWb}.java`、`versions/{1.20.1,1.21.6}/data/mc_src_extract/**`（两版 vanilla 反编译源，**仓库内既有产物**）、`.gradle-home/caches/fabric-loom/**`（mappings/命名 jar/refmap 产物）、`versions/{1.20.1,1.21.6}/java/build/classes/java/main/*-refmap.json`、`.investigations/shared-java-core-260912-01/evidence/log-1.21.6-{bulk,default}.txt` 等。

---

## 0. 继承结论的廉价独立复核（交接验证纪律；MUST 项）

### 0.1 ✅ 复核成立：两版 `ChunkSection` 字段名/类型同名 + intermediary 名逐字相同

| 断言（写在 1.21.6 accessor 头注释里，属上一轮 scout 结论） | 复核结果 | 我实际读到的证据位置 |
|---|---|---|
| 两版 `ChunkSection` 字段名与类型同名（1.20.1 `:21-24` / 1.21.6 `:21-24`） | **成立（逐字）** | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/chunk/ChunkSection.java:21-24` 与 `versions/1.21.6/.../ChunkSection.java:21-24`：`private short nonEmptyBlockCount / randomTickableBlockCount / nonEmptyFluidCount;` + `private final PalettedContainer<BlockState> blockStateContainer;` |
| intermediary 名两版逐字相同（1.21.6 `mappings-base.tiny:88534-88538` = 1.20.1 refmap:29-32） | **成立（且我直接读了 tiny 源，不依赖 refmap 转述）** | 1.21.6 `.gradle-home/caches/fabric-loom/1.21.6/net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2/mappings-base.tiny:88534-88538` = `f S field_12877 nonEmptyBlockCount` / `f S field_12881 nonEmptyFluidCount` / `f S field_12882 randomTickableBlockCount` / `f Lnet/minecraft/class_2841; field_12878 blockStateContainer`；1.20.1 同名 tiny `:69402-69406` **逐字相同** |
| refmap 内 4 条访问器全部登记 | **成立** | `versions/1.21.6/java/build/classes/java/main/coreswap1216-refmap.json:35-40` 与 `versions/1.20.1/java/build/classes/java/main/coreswap-refmap.json:28-33`（4 条同值，见 §3） |

### 0.2 ⚠ 复核**不成立**（关键：计划 §1 事实 3 的限定被证据推翻一半）

计划把「异常发生在**写回路径自身的读**内部，**不是**游戏后续阶段读到坏容器」当作**判据前提**。tracked 日志显示这是**两件事的合并**：

1. **130 条** `[CppBridge] DIAG write threw chunk(...)` = 写回路径内部读抛（`catch (Throwable t)` 打 `t.toString()`，**无栈**）——`java-core/src/main/java/wg/bench/CppBridge.java:495-499`。✅ 该半句成立。
2. **同一条日志里另有完整崩溃栈**（`---- Minecraft Crash Report ----` header 在 `log-1.21.6-bulk.txt:526`），顶层异常 `log-1.21.6-bulk.txt:532`，栈顶三帧 `:533-536` = `ArrayPalette.get(:76) ← PalettedContainer.get(:163/:157) ← ChunkSection.getBlockState(:51)`，但**其调用者是 vanilla carver**：
   - `:537 ProtoChunk.getBlockState(ProtoChunk.java:109)`
   - `:538 Carver.carveAtPoint(Carver.java:133)` → `:539-546` `carveRegion / CaveCarver.carveTunnels / … / NoiseChunkGenerator.carve / ChunkGenerating.carve / ChunkGenerationStep.run`
   - `:547-565` 经 `ServerChunkLoadingManager.generate → … → ForkJoinPool worker`（与「末栈停 ForkJoinPool worker」吻合）
   - 第二处同型栈 `:574-614`，`Caused by` 处 `:712`
   ⇒ **容器被写坏后，vanilla carver 阶段读同一容器同样炸**；「不是游戏后续阶段读坏容器」**不成立**。这不是「我们的探针读错」，而是「装进去的容器本身坏了」——该区分直接影响候选判别（见 §6 反证条件）。

### 0.3 ✅ 复算：`132 次` = 130 DIAG + 2 条崩溃栈行

- `[CppBridge] DIAG write threw` + `Minecraft Crash Report` 合计 **131**（grep 精确计数）= 130 DIAG + 1 header。
- `Missing Palette entry for index 2|8` 合计 **132** = 130 DIAG + `:532`（index 8）+ `:712`（index 8）。
- `log-1.21.6-default.txt`：`EntryMissingException` / `Crash Report` / `Exception generating new chunk` **0 匹配**（对照组干净，与计划事实 1 一致）；`[WG-CONTENT-WB]` 匹配 **625**（默认臂门全通）。

### 0.4 🔎 新事实（本轮回读，尚未见于台账）：异常索引**只有两个取值**

`log-1.21.6-bulk.txt` 全量索引分布：`index 8` = **30**（其中 28 是 DIAG、2 是崩溃栈行）⇒ `index 2` = **102**（全部 DIAG）。
即 130 条写回内部异常**只出现 2 与 8 两个索引值**（无 3…7、无 9+）。该分布是 §6 b2a 候选的强签名（推导见 b2a 的「预测签名」）。

---

## 1. 写回路径调用链（K1）

分派与顺序（overworld）：`CppBridge.fillChunk` → `CppBridge.java:496 writeChunk(...)` → `CppBridge.writeChunk(:651-671)`：

| # | 代码点 | 说明 | 相对「装新容器」的位置 |
|---|---|---|---|
| 1 | `CppBridge.java:653` | `BulkWb.writeSections(chunk, cx, cz, buf, height)`（`BULKWB` 真臂；假臂走 `:655 writeChunkPerBlock`） | 装容器动作本身 |
| 2 | `BulkWb.java:139-188` | 逐 section：全空气短路 `:157-164`（`old.isEmpty()` = **counter 读，不是 palette 读**）→ `buildContainer` `:167` → **`acc.wgSetBlockStateContainer(pc)` `:172`** + 三个计数写 `:173-175` | **装容器（写侧）** |
| 3 | `BulkWb.java:195-277`（`buildContainer`） | 去重/计数 → `pb.writeByte(bits)` `:244` → palette 段 `:246/:259-260` → **`pb.writeLongArray(pa.getData())` `:262`** → `pc.readPacket(pb)` `:268` | 装容器**之前**（构造新容器） |
| 4 | `CppBridge.java:660-666` | `Heightmap.populateHeightmaps(chunk, {WORLD_SURFACE_WG, …, MOTION_BLOCKING_NO_LEAVES})` → `Heightmap.java:59` **`chunk.getBlockState(mutable)`** → `Chunk.getBlockState` → **`ChunkSection.getBlockState`**（1.21.6 extract `:48-50`） | **之后**（读**新**容器） |
| 5 | `CppBridge.java:670` | `BulkWb.contentLine(chunk, cx, cz)`（**两形态共用**，Tier1 门控在方法内部） | **之后** |
| 6 | `BulkWb.java:360-366` | nz 扫描：`if (!sec.isEmpty())` → **`sec.getBlockState(x, y, z)` `:365`** | **之后**（读新容器；**第一个必到的 palette 读点**） |
| 7 | `BulkWb.java:370-379` | dh 层：`ChunkSectionAccessor` 三个 getter（**不是** palette 读） | 之后 |
| 8 | `BulkWb.java:381` → `BulkWb.java:338-355`（`readbackHash`） | `[WG-CONTENT-WB]` 指纹行：**`sec.getBlockState(x,y,z)` `:345`** → `Block.STATE_IDS.getRawId(...)` | 之后（`hash=` 与 `dh=` 同处打印） |

**结论性事实（K1 答）**：写回路径内部**所有** palette 位置读（`ChunkSection.getBlockState` / `PalettedContainer.get`）都发生在 `BulkWb.java:172` 装新容器**之后**，读的就是**我们自己刚 `readPacket` 出来的容器**（`sec.blockStateContainer` 是新对象）。写回路径里**不存在**「装容器之前」的 palette 读——装之前只有 counter 读（`old.isEmpty()`，`BulkWb.java:161`）。异常栈 `ChunkSection.getBlockState` 的调用者因此只能是 §1 表的 #4（Heightmap）或 #5/#6/#8（contentLine/readbackHash）之一；**130/130 全覆盖**这一事实更利于 #6（contentLine 对每个非空 section 从 `y=0,z=0,x=0` 起全格扫描，必然踩到坏槽位；Heightmap 列扫描自顶向下、遇首个不透明块即停，是否到达 `sy=0,x=14/15` 是概率性的）——**该归属是推断，非实测**（日志无区分两者的行）。

其他**非本臂**读点（避免误认，供排除）：`CppBridge.java:695`（逐块路径 `WBCHECK`，需 `-Dcoreswap.wbcheck=1`，bulk 臂不走）；`CppBridge.java:543`（nether MIXLOG 16 点读回）/ `:597`（end 同）；`BulkWb.java:310`（`selfTest` 内 `pc.get(x,y,z)`，需 `-Dcoreswap.bulkwbtest=1`）。

---

## 2. 两版静态结构对拍表（K3）

载体说明（**重要**）：下表「1.20.1 / 1.21.6」列取自仓库内 `versions/<ver>/data/mc_src_extract/`（反编译源，**内容可信、行号不可信**，见 §5）。两版**方法体差异均为实测逐行读出**，非转述。

| 成员/方法 | 1.20.1 | 1.21.6 | 差异 | 证据位置 |
|---|---|---|---|---|
| `PalettedContainer` 字段集 | `idList`、`volatile Data<T> data`、`paletteProvider`、`LockHelper lockHelper` | **同名同序同类型**（:33-38 两版逐行相同） | **无差异**（无额外缓存字段） | `versions/1.20.1/.../PalettedContainer.java:33-38` vs `1.21.6/.../PalettedContainer.java:33-38` |
| `readPacket` 骨架 | `lock(); int i = buf.readByte(); data = getCompatibleData(this.data, i); data.palette.readPacket(buf); **buf.readLongArray(data.storage.getData());** this.data = data; unlock()` | 同，**唯 :216 换成 `buf.readFixedLengthLongArray(data.storage.getData())`** | **长数组帧格式变更**（见下两行） | 1.20.1 `:203-215`（读语句 `:210`）vs 1.21.6 `:209-221`（`:216`） |
| storage long 段 **写**格式 | `PacketByteBuf.writeLongArray(long[])` = `writeVarInt(len)` + longs（`PacketByteBuf.java:859-867`） | `writeFixedLengthLongArray` = 仅 longs，无长度前缀（`PacketByteBuf.java:751-755`，且 `writeLongArray` 现由 `:741-744` = VarInt + fixed 组合） | **1.21.6 新增 fixed-length 变体，且 `PalettedContainer.Data.writePacket` 改用 fixed 版**（1.20.1 `:386` vs 1.21.6 `:393`） | `versions/{1.20.1,1.21.6}/data/mc_src_extract/net/minecraft/network/PacketByteBuf.java` |
| storage long 段 **读**格式 | `readLongArray(toArray, maxSize)`：先读 **VarInt 长度**再读该长度 long（`:924-939`） | `readFixedLengthLongArray(buf, values)`：读 `values.length` 个 long，**不消费 VarInt**（`:800-806`） | 同上（同一变更的两面） | 同上 |
| `Data.getPacketSize()` | `1 + palette.getPacketSize() + **PacketByteBuf.getVarIntLength(storage.getSize())** + data.length*8`（`:380`） | `1 + palette.getPacketSize() + data.length*8`（`:387`）——**VarInt 长度项被删** | vanilla 自身已不再计前缀长度（自洽佐证） | `PalettedContainer.java:380` vs `:387` |
| `getCompatibleData` / 位宽映射 | `createDataProvider(idList, bits)`：0→SINGULAR、1..4→ARRAY(bits 归一 4)、5..8→BI_MAP(bits)、≥9→ID_LIST；`getBits` = `provider.factory()==ID_LIST ? i : provider.bits()`（`:480-484`） | **同形**（`:487-491`） | **未观察到语义差**（KB #22 的位宽表两版一致） | 1.20.1 `:125/:480-484`；1.21.6 `:131/:487-491` |
| `get(x,y,z)` / `get(int)` | 两层 `get`（容器 → palette）：`get(x,y,z)` `:180` → `get(int)` `:184` | 同形：`:186` / `:190` | 无差异（仅行号位移） | 1.20.1 `PalettedContainer.java:180/:184` vs 1.21.6 `:186/:190` |
| `ArrayPalette.get(int)` 前置条件 | `id>=0 && id<size` 返回 `array[id]`，否则 `throw new EntryMissingException(id)`（`:76-80`） | **逐字同语义**（`array` 长度 = `1<<bits`，`size` = `readPacket` 读入的 VarInt） | 无差异 | 1.20.1 `ArrayPalette.java:76-80` vs 1.21.6 `ArrayPalette.java:76-83`（throw 在 `:80` / `:81`） |
| `ArrayPalette.readPacket` | `size = readVarInt(); for i<size: array[i] = idList.getOrThrow(readVarInt())` | 逐字同（`VarInts` 包名不同：1.21.6 导入 `net.minecraft.network.encoding.VarInts`） | 无差异（仅 1.21.5+ 包重命名） | 1.20.1 `:85-92` vs 1.21.6 `:86-92`（import `:6`） |
| `ChunkSection` 字段集/构造器/`getBlockState` | 三 short 计数 + `final PalettedContainer blockStateContainer` + `ReadableContainer biomeContainer`；`getBlockState` = `blockStateContainer.get(x,y,z)`（extract `:40-42`） | **同形**（extract `:21-25`、`:48-50`）；`getBlockState` 运行时行为一致 | **无差异**（唯一差别在 `ChunkSection(Registry)` 的生物群系 API：`entryOf` → `getOrThrow`） | `versions/{1.20.1,1.21.6}/.../ChunkSection.java`（1.20.1 `:36` vs 1.21.6 `:44`） |
| `ChunkSection` 字段名/类型同名 + refmap intermediary 同名 | ✅ 已复核成立 | ✅ | — | 见 §0.1 |

**K3 小结（事实面）**：`PalettedContainer` 的**字段集无变化**、位宽协商语义**未见变化**、`ArrayPalette.get` 前置条件**无变化**；两版**唯一** wire-level 结构差异是 **storage long 段的帧格式（VarInt 长度前缀 ↔ 定长）**，且 `getPacketSize` 的差异从属佐证同一变更。

---

## 3. mixin 生效性（K5）：证据或缺口

**静态证据（充分）**
1. 源码：`versions/1.21.6/java/src/main/java/wg/bench/mixin/ChunkSectionAccessor.java:30-60` —— `@Mixin(ChunkSection.class)` + `@Mutable @Accessor("blockStateContainer") wgSetBlockStateContainer` + 三计数 setter/getter。
2. 注册：`versions/1.21.6/java/src/main/resources/coreswap.mixins.json:31` 含 `"ChunkSectionAccessor"`（`"required": true`，`injectors.defaultRequire=1`，`:2/:33-35`）→ 未生效会**装载失败**而非静默（KB #40 家族）。
3. refmap 命中（**两版同值，逐字节相同**）：`versions/1.21.6/java/build/classes/java/main/coreswap1216-refmap.json:35-40` =
   `blockStateContainer → field_12878:Lnet/minecraft/class_2841;`、`nonEmptyBlockCount → field_12877:S`、`nonEmptyFluidCount → field_12881:S`、`randomTickableBlockCount → field_12882:S`；1.20.1 `coreswap-refmap.json:28-33` 同值。tiny 源同值（§0.1）。
4. dev 运行态为 **named(yarn) 映射**：`log-1.21.6-bulk.txt:60` `Loaded Fabric development mappings for mixin remapper!`（`Environment=SERVER` `:59`）。

**运行期证据（本轮新读出，两条独立）**
5. **访问器接口确实被织入**：`BulkWb.java:171` 有 `(ChunkSectionAccessor) (Object) old` 强制转换；若 mixin 未生效，异常类型会是 `ClassCastException`——而 130 条 DIAG 全部是 `EntryMissingException`（`log-1.21.6-bulk.txt:155-524` 计数见 §0.3）⇒ 该 cast 在运行期成功。
6. **getter 侧在 1.21.6 默认臂实测工作**：默认臂 625 行 `[WG-CONTENT-WB]` 每行都带 `dh=<hex>`（`dh` 由三个 accessor getter 计算，`BulkWb.java:370-379`），例 `log-1.21.6-default.txt:156`。
7. **container setter 生效（推断）**：新 chunk 的 section 初始容器是 `new PalettedContainer<>(STATE_IDS, AIR, BLOCK_STATE)`（SINGULAR 无 storage，`ChunkSection.java:42`）；`allAir && old.isEmpty()` 会短路跳过（`BulkWb.java:161`）⇒ 未被替换的 section 是全空气单态容器，**读它不可能抛 `EntryMissingException`**。异常既然出现（且 vanilla carver 也读同一容器抛），说明被读的容器是**替换后的容器**。此条为**推断**（逻辑排除）。

**缺口（诚实声明）**：`-Dcoreswap.mixlog=1` **不**提供任何 mixin-apply/refmap 自证行——它只开门控诊断打印：`CppBridge.java:428`（`MIXLOG`）、`NoiseChunkGeneratorMixin.java:51`（同 property，门控 `[Mixin] populateNoise intercepted` `:109` / `buildSurface skipped` `:249`）。日志中**无** `Mixin apply failed` / `Cannot find target method` 行（grep 0 匹配）——属**弱证**（INFO 级 grep 可能漏，KB #84），但第 5/6 条已给出更强的运行期正向证据。若要**直接**证明「容器字段被换成我们构造的对象」，需 `-Dcoreswap.mixlog=1` 下一行身份比较打印（见 §8 命令 C3）。

---

## 4. Rust buf 发射事实（K4）：文件:行号 + 是否分版

| 事实 | 证据位置 |
|---|---|
| 「buf」= **`int[16*16*height]` 的 vanilla raw block id**，无 header/位宽/palette/长度段（Rust 不发射任何帧结构） | `worldgen-core/src/api.rs:129-130` 注释 + `:176 outs[i].write(&blocks)`（`SendOut::write`）；`:134-183` `wg_fill_blocks_multi` |
| JNI 出口 = `Java_wg_CppWorldgen_fillBlocks`，**本地 buffer 按 Java 侧 `out` 数组实际长度分配**（overworld 98304 / nether 65536），非硬编码 | `versions/1.21.6/rust/src/jni_bridge.rs:179`、`:204 out_len = env.get_array_length(&out0)`、`:205 vec![0i32; out_len]`（历史坑注释 `:199-203`；常量 `:19 BLOCK_COUNT` 仍存但已不用于分配） |
| 1.21.6 薄壳**只**做 Java ABI 适配，无编码/版本分支 | `versions/1.21.6/rust/src/lib.rs:1-3`（仅 `pub mod jni_bridge;`） |
| `worldgen-core` 内**不存在** MC 版本 feature gate | grep `^#[cfg(` / `feature = "`:仅 `#[cfg(test)]`、`#[cfg(windows)]`、`#[cfg(target_feature="avx")]`（无 mc1216/mc-version 门）；`mc-1216` 仅出现在注释与测试名中 |
| Java 侧 per-thread 目标 buffer | `BulkWb.TL.bb = Unpooled.buffer(8192)`（`BulkWb.java:121`）；`CppBridge` 的 `BUF`（overworld 384）、`BUF_NETHER`（`:504-505`）、end 同族 |

**K4 答**：Rust 侧写回缓冲**不分版**，两版同契约（raw block id 的 `int[]`）；KB #22 描述的「逐字节编码契约」**全部在 Java 侧**——由 `BulkWb.buildContainer` 自建 `PacketByteBuf`（`BulkWb.java:242-263`）后交给 vanilla `PalettedContainer.readPacket`。因此本缺陷的版本敏感性只能来自 **Java 侧自建 buffer 的帧格式** 与 **vanilla 读端** 的配对（见 §6 b2a），Rust 侧不存在对应变量。

---

## 5. mapped 源码可及性（K2）：实际找到的路径

| 载体 | 路径 | 可用性 |
|---|---|---|
| **1.21.6 反编译源（仓库内既有产物）** | `versions/1.21.6/data/mc_src_extract/net/minecraft/**`（如 `world/chunk/PalettedContainer.java`、`world/chunk/ArrayPalette.java`、`world/chunk/ChunkSection.java`、`network/PacketByteBuf.java`、`world/Heightmap.java`、`world/gen/carver/Carver.java`） | ✅ **内容级对拍可直接用**；两版指纹可区分（1.21.6 `ArrayPalette.java:6` 导入 `net.minecraft.network.encoding.VarInts`，1.20.1 无；1.21.6 `ChunkSection.java:44 getOrThrow` vs 1.20.1 `:36 entryOf`）。⚠ **行号与运行时不一致**（见下） |
| 1.20.1 反编译源 | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/chunk/**`（另有 `.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/**` 同族副本） | ✅ 同上 |
| **命名（yarn）mapped 字节码 jar** | `.gradle-home/caches/fabric-loom/minecraftMaven/net/minecraft/minecraft-merged/1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2/minecraft-merged-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar`；1.20.1 同族 `…/1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2/…jar` | ✅ 可用于 `javap`（**权威字节码**）。运行时帧正是它：`log-1.21.6-bulk.txt:737` 显示 `minecraft-merged-b07cf08c30-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar`（`b07cf08c30` 对应 `runtime/1.21.6/java/.gradle/loom-cache/source_mappings/b07cf08c30b5f305700b5075675404956dfcbab8.tiny`） |
| **mapped sources jar（MC 本体）** | ❌ 全仓库 `**/*sources*.jar` 只命中 forge/fabric-api 等，**无 MC 本体 sources jar** | 需退化为 mapped 字节码 `javap`（命令见 §8） |
| yarn mappings（named + intermediary） | `.gradle-home/caches/fabric-loom/<ver>/net.fabricmc.yarn.<…>/mappings-base.tiny`、`mappings.tiny`、`intermediary-v2.tiny`、`net.fabricmc.yarn.<…>/mappings.jar` | ✅ 已用于 §0.1 复核 |
| loom mixin-map | `versions/<ver>/java/build/loom-cache/mixin-map-net.fabricmc.yarn.<…>.main.tiny`、`versions/<ver>/java/build/classes/java/main/*-refmap.json` | ✅ |

**⚠ 行号不可比（实测，必须写进结论使用限制）**：tracked 崩溃栈的行号来自运行时的 mapped jar（**保留 Mojang 官方源码行表**），与仓库内反编译源**不成常数偏移**：

| 帧 | 运行期日志行号 | 仓库 extract 行号 | 差 |
|---|---|---|---|
| `ArrayPalette.get`（throw） | 76（`log-1.21.6-bulk.txt:533`） | `versions/1.21.6/.../ArrayPalette.java:81` | −5 |
| `PalettedContainer.get(x,y,z)` | 157（`:535`） | `:186` | −29 |
| `PalettedContainer.get(int)` | 163（`:534`） | `:190` | −27 |
| `ChunkSection.getBlockState` | 51（`:536`） | `:48-50`（调用行 49） | +2 |
| `ProtoChunk.getBlockState` | 109（`:537`） | `:91`（调用行 97） | +12 |
| `Carver.carveAtPoint` | 133（`:538`） | `:120`（调用行 131） | +2 |

⇒ 行号一致性**不可作判据**；本次全部结论均以**方法体内容 + 结构**为准，并建议主会话用 `javap`（命令 A1/A2）在**权威字节码**上做一次交叉确认。

---

## 6. 候选清单（互斥；含判别探针 / 预测签名 / 反证条件）

> 预置 b1–b4 保留，但把 b2 拆成两个**互斥子机制**（b2a/b2b），因为二者在同一父机制下判别方式完全不同（这是本轮勘探的主要产出）。**证据面**评价只描述「支持/不支持」，不构成裁决。

### b2a｜**storage long 段帧格式差**（1.20.1 `writeLongArray`/`readLongArray` 带 VarInt 长度前缀 ↔ 1.21.6 `writeFixedLengthLongArray`/`readFixedLengthLongArray` 无前缀）⇒ Java 侧按 1.20.1 契约自建的 buffer 在 1.21.6 上多出 2 字节前缀，长数组整体移位 16 bit，解码出的局部索引错位。

- **机制链（全部带行号）**：写侧只按 1.20.1 契约打包（`BulkWb.java:262 pb.writeLongArray(pa.getData())`；bits==0 支 `:247`）→ 1.21.6 读侧不消费该前缀（`versions/1.21.6/.../PalettedContainer.java:216 readFixedLengthLongArray`，实现 `PacketByteBuf.java:800-806`）→ reader 的 2048 字节窗口整体前移 2 字节 → 4-bit（ARRAY）元素解码 = 真实元素 `i∓4`，且**每个 long 的首尾 16 bit 被前缀字节污染的位置恰是线性索引 12..15**（`(y<<8)|(z<<4)|x` ⇒ `y=0,z=0,x=12..15`）→ 局部索引 8 / 2 越出 `palette.size` → `ArrayPalette.get` 抛 `EntryMissingException`（`:76-81`）。
- **判别探针 P1（单变量、可执行、零代码改动）**：`-Dcoreswap.bulkwb=1 -Dcoreswap.bulkwbtest=1`（自检在 `BulkWb.java:141/293-322`，四支合成用例、逐格读回比对）——1.21.6 与 1.20.1 各跑一臂。
- **预测签名**：1.21.6 = **SINGULAR 支（distinct=1）PASS**、ARRAY（distinct=3）/BI_MAP/ID_LIST 支 **FAIL 或直接抛异常**（`AssertionError [WG-BULKWB-TEST] FAIL` 或 `EntryMissingException`）；1.20.1 = 4×`[WG-BULKWB-TEST] PASS` + `all branches PASS`。
- **预测签名（观测面，已在既有日志上部分命中）**：DIAG 异常索引取值 ⊆ `{nibbles of VarInt(len)}`；ARRAY 分支 `len = 4096×4/64 = 256`，`VarInt(256) = 0x80 0x02` ⇒ nibbles `{8,0,0,2}` ⇒ **只可能看到 2 与 8**（0 永不越界）。**实测正是 `index 2`×102 + `index 8`×28，无其他取值**（§0.4）✓。另一致性：1.20.1 bulklog 汇总 `max_distinct=7 / idlist_hits=0`（`evidence/log-1.20.1-bulklog.txt:2567`）⇒ 所有被替换 section 的 `palette.size ≤ 7 ≤ 8` ⇒ **每个 chunk 至少一个 section 必抛**（`[WG-CONTENT-WB]` 恰好 0 行）✓，且 `size=2` → index 2、`size=3..7` → index 8（两个取值都出现）✓。
- **反证条件（任一命中即否掉）**：① 在 1.21.6 上给 `buildContainer` 加一行「`readPacket` 后 `pb.readableBytes()`」打印，实测残留 = **0**（本候选预测 2；SINGULAR 支预测 1）——若为 0，帧格式无差，本候选死；② 权威字节码 `javap` 显示 1.21.6 `readPacket` 仍调 `readLongArray`（带前缀）；③ 重复跑 1.21.6 bulk 臂出现 `{2,8}` 之外的索引值（会否掉「前缀 nibble 污染」这一具体机制，需回到 b2b/b1）；④ 只把 `:262` 改成定长写（1.20.1 侧保持 `:262` 原样，用分版缝）后 1.21.6 bulk 臂**仍**出现同型异常。

### b2b｜**位宽协商语义差**（`readPacket` 的 `getCompatibleData(this.data, i)` 或 `createDataProvider` 的位宽映射变化 ⇒ 声明位宽与 storage 位宽/palette 条目数不一致）

- **证据面**：**不被支持**。两版 `getCompatibleData`/`createDataProvider`/`getBits` 与 `readPacket` 的 byte 读取**同形**（§2 表）；且该机制预测解码值将散布在 `0..15`（或 `0..31`）多处，而实测只有 `{2,8}`（§0.4）。仍未排除到 0（未在权威字节码上核）。
- **判别探针**：`javap -p -c` 对两版 `PalettedContainer.readPacket` + `PaletteProvider$…createDataProvider` 归一化对拍（命令 A1/A2）；或探针 P1 的分布判读（若出现 3..15 多值 ⇒ 指向本候选）。
- **预测签名（若为本机制）**：1.21.6 上 ARRAY 支的 `storage.getElementBits()` 与 `i`（buffer 首字节）不一致（例：读到 3 或 4 之外的归一值）⇒ 解码值大范围散布。
- **反证条件**：探针打印 `storage.getElementBits()` 恒等于期望的「1..4→4 / 5..8→bits / ≥9→ceilLog2(STATE_IDS.size())」⇒ 本候选死。

### b1｜**1.21.6 `PalettedContainer` 内部结构/语义差**（例如额外缓存字段 ⇒ 原地换容器后读到旧/非活动数据）

- **证据面**：**不被支持**。字段集两版逐行相同（`:33-38`），`readPacket` 末尾 `this.data = data` 两版相同（1.20.1 `:211` / 1.21.6 `:217`），无额外缓存；且该机制预测「读到**有效但陈旧**的 palette 索引」⇒ **不应**出现 `EntryMissingException`。
- **判别探针**：`javap -p` 两版字段清单对拍（命令 A2 附带）+ P1。
- **预测签名**：异常类型会是「读到旧方块」类**静默错值**（无异常），或 `getBlockStateContainer()` 身份与 `readPacket` 结果不一致。
- **反证条件**：一旦观察到 `EntryMissingException`（已观察到 130 次），本候选的「陈旧读」形态即被否。

### b3｜**Rust buf 分版编码差**（header/idlist/state id 域）

- **证据面**：**不被支持**。Rust 侧无帧结构、无版本门（§4）；若 Rust 域错位，现象应为 `idList.getOrThrow` 抛未注册 raw id（`ArrayPalette.readPacket`）或静默错块，而非「读越界索引」；且 `[WG-CONTENT]` 指纹在 bulk 臂正常产出（`log-1.21.6-bulk.txt` 大量 `[WG-CONTENT]` 行，例 `:501`）说明 buf 层数据自身自洽。
- **判别探针**：对同一 chunk 比较 1.20.1/1.21.6 的 `[WG-CONTENT] hash=` 多重集（既有 `fp-*` 载体，`record-260912-01.md:381` 三要素已声明口径）。
- **预测签名（若为本机制）**：同 chunk 两版 `[WG-CONTENT]` 指纹面出现系统性差异；或 `readPacket` 阶段抛 `IllegalArgumentException/NullPointer`（未注册 id）。
- **反证条件**：`[WG-CONTENT]` 两版多重集相等（既有 Wave 2 已做过同族对照）⇒ 本候选死。

### b4｜**accessor/映射未真正生效**（refmap/`@Mutable` on `final`/目标类差异）

- **证据面**：**不被支持**（两条独立运行期正向证据）：接口 cast 成功（否则 `ClassCastException`）+ getter 在 1.21.6 默认臂产出 `dh=`（§3.5/3.6）；且「setter 静默无效」形态预测读的是**未替换的 SINGULAR 全空气容器** ⇒ 不可能抛 palette 越界（§3.7 推断）。
- **判别探针**：C3（同一 section swap 后 `sec.getBlockState(0,0,0)` 与 `pc.get(0,0,0)` 身份/取值一致性打印）；或 `-Dcoreswap.mixlog=1` 下观察是否出现 `Mixin apply failed`（弱证）。
- **预测签名（若为本机制）**：`ClassCastException` / mixin apply 失败 / 读回恒为 air。
- **反证条件**：观察到 `EntryMissingException` 或 `dh=` 正常产出（二者均已观察到）⇒ 本候选死。

**证据面小结（供主会话派 worker 时定优先级，非裁决）**：**b2a 是唯一被证据面强支持的候选**（机制链两端都有行号、`{2,8}` 签名 + `max_distinct=7` 分布三重自洽）；b1/b3/b4 各有独立反证；b2b 未被支持但未在权威字节码上核到 0。**fan-out 预置的 b1/b2/b3/b4 建议改为 b2a / b2b / b1+b3+b4（合并为「非帧格式」对照支）** 三支并行，避免把已被反证的两支与 b2a 平权。

---

## 7. 降级 / 未解声明

| 未解项 | 原因 | 需要什么才能答 |
|---|---|---|
| U1｜130 条 DIAG 异常**具体出自哪个读点**（Heightmap `:660` vs contentLine `:670`） | 日志只打 `t.toString()`（无栈），两条路径共用 `ChunkSection.getBlockState`；无法从既有载体区分 | 需要一次带栈打印（或在 `writeChunk` 内加 `try` 分段插桩），命令 C2 |
| U2｜`versions/1.21.6/data/mc_src_extract/` 的**生成出处**（无 README/清单） | 仓库内无 provenance 记录 | 需要主会话在 `.tmp`/历史块记录中查生成脚本；或按命令 A1 直接在 mapped jar 字节码上核（**已足够支撑内容级结论**） |
| U3｜**行号级**定位（计划/E5 引用的 `:76/:157/:163/:51`） | 运行期行号来自 mapped jar 保留的官方行表，与仓库反编译源偏移不成常数（§5 表） | 命令 A3（`javap -l`） |
| U4｜**未跑任何运行探针**（P1/C1–C3 均未执行） | 本子任务纪律：scout 不构建、不运行、不动 dll；且本人无 shell | 主会话执行 §8；P1 一次即可给出决定性判读 |
| U5｜`ChunkSectionAccessor` setter 的**直接**身份证据缺失 | 既有日志无该打印；§3.7 为逻辑推断 | 命令 C3 |
| U6｜1.21.6 侧 bulk 臂的 `[WG-BULKWB] max_distinct`（缺 `-Dcoreswap.bulkwblog=1`，仅默认臂/1.20.1 有） | 该臂配方未开 log | 命令 C1 加 `-Dcoreswap.bulkwblog=1`（用于验证「无 size≥9 的 section」这一预测） |

**验证分层声明**：本文件全部结论为 **Degraded（静态审查：源码/映射/字节码级 + 既有日志回读）**，无任何本轮新采集的运行期数据；量化主张（130/102/28/625/132/131）均为**既有日志的 grep 精确计数**，载体 = tracked evidence 副本，口径与 `record-260912-01.md:379-382` 同源可比。

---

## 8. 需要主会话执行的命令清单（可直接复制；含期望输出与判读）

> 前置：命令 A* 为纯静态（无游戏、无 dll，最先跑，零污染风险）；命令 C* 为运行臂（串行！两版共用端口；每臂必须先读 `[CppBridge] dll=` 自证行核对血统，E1）。

**A1 — 权威字节码确认 1.21.6 读端帧格式（K3 交叉核）**
```powershell
$j1216 = ".gradle-home\caches\fabric-loom\minecraftMaven\net\minecraft\minecraft-merged\1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2\minecraft-merged-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar"
javap -p -c -classpath $j1216 net.minecraft.world.chunk.PalettedContainer | Select-String -Pattern "readPacket|readFixedLengthLongArray|readLongArray|writeFixedLengthLongArray|writeLongArray" -Context 1,3
```
判读：`readPacket` 体内应出现 `readFixedLengthLongArray`（且**不**出现 `readLongArray`）；`Data.writePacket` 应出现 `writeFixedLengthLongArray`。与 1.20.1 对照（同命令换 jar = `…\1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2\…jar`）应出现 `readLongArray`/`writeLongArray`。**期望：与 §2 表一致 ⇒ 帧格式差在权威字节码上成立**（b2a 的机制前提）。若 1.21.6 仍用 `readLongArray` ⇒ b2a 立即死，转 b2b/b1。

**A2 — 字段/`getPacketSize`/`getCompatibleData` 字节码对拍（K3 补强）**
```powershell
javap -p -c -classpath $j1216 "net.minecraft.world.chunk.PalettedContainer`$Data" | Select-String -Pattern "getPacketSize|writePacket|getVarIntLength" -Context 1,6
```
判读：1.21.6 `Data.getPacketSize` 内**不得**出现 `PacketByteBuf.getVarIntLength`（1.20.1 应出现）——与源码 `:387` vs `:380` 一致。

**A3 — 行号口径确认（解 U3）**
```powershell
javap -l -p -classpath $j1216 net.minecraft.world.chunk.ChunkSection | Select-String -Pattern "getBlockState" -Context 0,6
```
判读：`LineNumberTable` 应把 `blockStateContainer.get` 标在 **51** 行（与崩溃栈一致）⇒ 证明运行期行号 = 官方行表、仓库反编译源行号不可比；此后**禁止**用 extract 行号质疑栈。

**C1 — 决定性探针 P1（1.21.6，单变量；首跑一次自检）**
```powershell
$env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=1 -Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1 -Dcoreswap.bulkwbtest=1"
pwsh .investigations\shared-java-core-260912-01\evidence\tool-run_1216_w2.ps1 -Tag probe-b2a-test -Radius 160
```
（`CORESWAP_EXTRA_JVM` 由该脚本 `:27-29` 注入；日志落 `.tmp\shared-java-core-260912-01\w2\1216-probe-b2a-test.log`）
判读（**先核血统**：日志首条 `[CppBridge] dll= … sha256=abd7d889…`，`log-1.21.6-bulk.txt:94` 同值）：
- b2a 命中 = `[WG-BULKWB-TEST] PASS distinct=1` 之后**紧接** `AssertionError [WG-BULKWB-TEST] FAIL branch distinct=3 …` 或 `EntryMissingException`（`BulkWb.java:313-314` / `:310` 抛出）；
- 附带：`[WG-BULKWB] … max_distinct=?`（验证「无 `size≥9` 的 section」；`idlist_hits` 应为 0）；
- 反证 b2a = 四支全 PASS（则帧格式无差）。

**C2 — U1 归属探针（可选，一次性）**：在 `CppBridge.java:660` 与 `:670` 之间插一行 `System.out.println("[DBG] hmap ok chunk("+cx+","+cz+")")`（或把 `writeChunk` 的 `Heightmap` 调用单独包 `try/catch` 打标记），跑 `-Dcoreswap.bulkwb=1 -Dcoreswap.mixlog=1`。判读：若 `[DBG] hmap ok` **不**出现 ⇒ 异常出自 Heightmap；出现 ⇒ 出自 contentLine。

**C3 — K5 直接身份证据（可选，一次性）**：在 `BulkWb.java:172` 之后加一行
`if (System.getProperty("coreswap.swapcheck") != null) System.out.println("[DBG] swap " + (old.getBlockStateContainer() == pc) + " sec=" + s);`
跑 `-Dcoreswap.bulkwb=1 -Dcoreswap.swapcheck=1`。判读：应恒为 `true`；若 `false` ⇒ 容器字段未换成我们构造的对象（b4 复活）。注：`getBlockStateContainer()` 是公开 getter（extract `ChunkSection.java:150-152`）。

**C4 — 修复式验证（HOOK-C，需用户批准后再做）**：把 `BulkWb.java:262`（及 `:247`）改为**按版本分支**的帧写入（1.21.6 = 定长；1.20.1 保持 VarInt 前缀，V1b 必须逐字节不变），再跑 bulk 臂，判据 = `[WG-CONTENT-WB]` 恢复 625 行且与默认臂逐 chunk `hash`/`dh` **多重集**相等、非 WMI 异常 0。**注意此改动落在共享源 `java-core`，必须带分版缝**（`WgCompat` 同族），并需 judge 三源核对。

**D1 — 1.20.1 对照（P1 的反面板）**：以 `tool-run_pre_1201.ps1` 为模板复制一份到 `.tmp/`，在其 `JAVA_TOOL_OPTIONS` 后追加 `-Dcoreswap.bulkwb=1 -Dcoreswap.bulkwbtest=1`（原脚本 `:24` 未接 `CORESWAP_EXTRA_JVM`），跑后判读 `[WG-BULKWB-TEST] PASS ×4 + all branches PASS`。若 1.20.1 该探针不是全 PASS ⇒ 探针本身不可用（需先修探针）。

---

## 9. 已验证事实 vs 推断

### 9.1 已验证事实（均有 `文件:行号` 或日志行号）

1. 写回内部 palette 读**全部在换容器之后**：`CppBridge.java:653`(换) → `:660-666`(Heightmap 读) → `:670`(contentLine 读)；读点 `BulkWb.java:345/:365`；换容器前的读只有 `old.isEmpty()`（`BulkWb.java:161`）。
2. Java 侧自建 buffer 按 1.20.1 契约写长数组：`BulkWb.java:262 pb.writeLongArray(pa.getData())`、`:247`（EMPTY_LONGS 支）、`:244 writeByte(bits)`、`:259-260`（palette 段）、`:268 readPacket`。
3. 1.21.6 读端为定长帧：`versions/1.21.6/data/mc_src_extract/net/minecraft/world/chunk/PalettedContainer.java:216 readFixedLengthLongArray`；`PacketByteBuf.java:800-806` 实现（不消费 VarInt）；`Data.writePacket` `:393` 用定长写；`getPacketSize` `:387` 无 VarInt 项。
4. 1.20.1 读端为 VarInt 前缀帧：`versions/1.20.1/.../PalettedContainer.java:210 readLongArray`；`PacketByteBuf.java:924-939`（读 VarInt 后再读）；`Data.writePacket` `:386`；`getPacketSize` `:380` 含 `getVarIntLength`。
5. `ArrayPalette.get` 越界即抛、无其它路径：`versions/1.21.6/.../ArrayPalette.java:76-81`（1.20.1 `:76-80`）。
6. 两版 `ChunkSection` 字段名/类型/intermediary 名逐字相同（§0.1 四组证据）。
7. mixin 在 1.21.6 **运行期确有生效**：接口 cast 成功（130 条为 `EntryMissingException` 而非 `ClassCastException`，`log-1.21.6-bulk.txt:155-524`）+ 默认臂 `dh=` 正常（`log-1.21.6-default.txt:156` 等 625 行）+ refmap 4 条同值（`coreswap1216-refmap.json:35-40`）。
8. `-Dcoreswap.mixlog=1` 只开门控诊断打印，**不**提供 mixin/refmap 自证：`CppBridge.java:428`、`NoiseChunkGeneratorMixin.java:51/:109/:249`；日志中 `Mixin apply failed`/`Cannot find target method` 0 匹配。
9. 异常计数与索引分布：DIAG 130（index2 102 / index8 28）+ 崩溃报告 1 份（`:526`，栈 `:532-565`、`:574-614`、`Caused by :712`）= 132；默认臂 0 异常、`[WG-CONTENT-WB]` 625；bulk 臂 `[WG-CONTENT-WB]`/`[WG-BULKWB]`/`[WG-PERBLOCK]` 均 0 行（`log-1.21.6-bulk.txt` grep）。
10. 崩溃栈的调用者是 **vanilla carver**：`log-1.21.6-bulk.txt:537-546`（`ProtoChunk.getBlockState` → `Carver.carveAtPoint` → … → `ChunkGenerationStep.run`），末栈 `ForkJoinPool`。
11. Rust 侧无版本门、buf = raw block id `int[]`、本地 buffer 按 Java 数组长度分配：`worldgen-core/src/api.rs:129-130/:176`、`versions/1.21.6/rust/src/jni_bridge.rs:19/:179/:204-205`、`versions/1.21.6/rust/src/lib.rs:1-3`；`worldgen-core/src` 无 mc-version `#[cfg]`。
12. 1.20.1 bulk 臂实测 `max_distinct=7`、`idlist_hits=0`、`sections_replaced=4856`、`air_sections_skipped=9712`、`calls=607`：`evidence/log-1.20.1-bulklog.txt:2567`。
13. 运行期载体 = named mapped jar，其 source_mappings 哈希 `b07cf08c30…` 与 `runtime/1.21.6/java/.gradle/loom-cache/source_mappings/b07cf08c30b5f305700b5075675404956dfcbab8.tiny` 对应：`log-1.21.6-bulk.txt:737`。
14. 两版反编译源可区分版本（非同一份拷贝）：1.21.6 `ArrayPalette.java:6` 导入 `net.minecraft.network.encoding.VarInts`（1.20.1 无）；1.21.6 `ChunkSection.java:44` 用 `getOrThrow`（1.20.1 `:36` 用 `entryOf`）。

### 9.2 推断（显式标注）

- **I1（推断）**：130 条 DIAG 的抛出点主要在 `BulkWb.contentLine` 的 nz 扫描（`BulkWb.java:365`）而非 Heightmap（`:660`）：依据 = 130/130 全覆盖 + contentLine 对每个非空 section 从 `(x0,y0,z0)` 全格扫，必然踩到坏槽位；Heightmap 自顶向下且遇首块即停在概率上不该 100% 命中。U1 探针（C2）可判。
- **I2（推断）**：`{2,8}` 两值来自 `VarInt(256)=0x80 0x02` 的 nibble 与「前缀污染恰好落在每个 long 首尾 16 bit（线性索引 12..15）」的组合；推导依赖 `PackedIntegerArray` 元素 0 在低位（`versions/1.21.6/.../PackedIntegerArray.java:307-311`：`j=(index-e*elementsPerLong)*elementBits; l >> j & maxValue`）——**机制推导**，其正确性应由 P1/C1 与「残留字节 = 2」探针确认。
- **I3（推断）**：容器 setter 生效（§3.7，基于「未替换的 SINGULAR 容器不可能抛 palette 越界」的排除）。
- **I4（推断）**：运行期栈行号 = Mojang 官方行表（故与 yarn 反编译源不可比）——A3 可核。
- **I5（推断）**：`versions/1.21.6/data/mc_src_extract/` 由某次反编译流程产出，**内容**属 1.21.6（多处版本指纹一致），**行号**不保真。

### 9.3 本文件**不**主张

- 不主张「根因 = b2a」（仅称「证据面强支持」）；不写 confirmed/candidate；不给修复方案定稿（C4 只是判别性验证，落点/分版缝/兼容策略属 HOOK-C，交主会话 + 用户）。
- 不主张 1.20.1 侧存在同缺陷（1.20.1 bulk 已 confirmed 正常，且其读端本就是 VarInt 前缀帧 = 与写侧契约匹配）。
