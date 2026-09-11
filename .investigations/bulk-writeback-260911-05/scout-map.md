# bulk writeback 可行性勘探地图（260911-05，只读）

> 只读勘探；不含方案评价与实施决策；未核项单列。
> 勘探范围：`E:\PYTHON\CoreSwap`（versions/1.20.1、versions/1.21.6、worldgen-core、.investigations、.tmp、knowledge）。
> vanilla 一手源（两套，逐项交叉核对一致）：
> - 1.20.1 = `.tmp/scout-260905-08/mcsrc/`（yarn），与 `versions/1.20.1/data/mc_src_extract/` **同文件同行号**（已核 ChunkSection / Heightmap 两文件）。
> - 1.21.6 = `versions/1.21.6/data/mc_src_extract/`（yarn 命名）。
> 所有引用行号 = 本次实际读到的行号。未亲眼读到的写「未核」。

---

## 1. 当前写回路径（A）

### A1. `versions/1.20.1/java/src/main/java/wg/bench/CppBridge.java`（687 行）

**调用链（overworld）**：`feedBeardifier(...)`（**独立函数**，末尾 `:354`；`CppWorldgen.setBeardifier` 调用点 `:336` / `:342`）→ `fillChunk(chunk)` `:393-431`：`:397` `int[] buf = BUF.get();` → `:400-401` `CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz}, new int[][]{buf}, THREADS)` → `:414-418` O(1) all-air 短路 → `:419-425` MIXLOG 内 nz + 指纹 → `:427` **`writeChunk(chunk, cx, cz, buf, 384);`**（5 参：chunk/cx/cz/buf/**height**）。尾部无 `ChunkTiming`（1.20.1 无分项钩子）。

| 行号 | 事实（原文片段） |
|---|---|
| `:33-36` | `private static final int MAX_ID = 4096;` + `AtomicReferenceArray<BlockState> STATE_BY_ID`（进程级 raw id→BlockState 缓存，并发可见性用 atomic，注释「vanilla 注册表运行期冻结」） |
| `:37-39` | `private static final ThreadLocal<int[]> BUF = ThreadLocal.withInitial(() -> new int[16*16*384]);`（**per-thread 复用**，每 worker 线程 384KB；注释「M=1 无锁模型」）。**三个 per-dimension buffer**：`BUF`(384) `:38-39`、`BUF_NETHER`(16*16*256) `:434-435`、`BUF_END`(16*16*128) `:482-483`（1.21.6 对应 `:44-45`、`:416-417`、`:460-461`） |
| `:370` | `private static final boolean MIXLOG = System.getProperty("coreswap.mixlog") != null;`（per-chunk 日志门控，默认关） |
| `:381-391` | `wgBufHash(int[] buf)` = FNV-1a 64，对 buf **逐 int 位置敏感**混入 |
| `:414-418` | O(1) all-air 短路（A1b/judge C2）：`if (buf[0] == 0) { … for (int k = 1; k < buf.length; k++) if (buf[k] != 0) { allAir = false; break; } … }` —— 保留「Rust 输出全 0」信号且摊销常数 |
| `:419-425` | MIXLOG 内：全量 nz 计数（`:420-421`）+ `[WG-CONTENT] chunk(cx,cz) hash=… nz=…`（`:423-424`） |
| `:400-401` / `:427` | `fillBlocks(h, new int[]{cx}, new int[]{cz}, new int[][]{buf}, THREADS)`（每 chunk 新建 3 个小数组）/ `writeChunk(chunk, cx, cz, buf, 384)` |
| `:437-479` | `fillChunkNether`（`netherHandle`/`netherEnabled`，buffer 16*16*256）；`:444-445` fillBlocks；`:455` `writeChunk(chunk, cx, cz, buf, 256);`；`:460-478` MIXLOG 内 16 点读回 + 指纹（`:463-464`） |
| `:485-530` | `fillChunkEnd`（`endHandle`，buffer 16*16*128）；`:492-493` fillBlocks；`:503` `writeChunk(chunk, cx, cz, buf, 128);`；`:509-529` 同上（A2 260911-05 补的 end 指纹行，`:514-515`） |
| `:540` | `private static final boolean WBCHECK = System.getProperty("coreswap.wbcheck") != null;` |
| `:541-547` | A1a A/B 开关与自检计数器：`:543` `SKIPAIR = !"0".equals(System.getProperty("coreswap.skipair"))`（**默认开**）；`:544-545` `WB_AIR_SKIP`；`:546-547` `WB_STALE_NONAIR` |
| `:549` | **`private static void writeChunk(Chunk chunk, int cx, int cz, int[] buf, int height)`**（全函数 `:549-601`） |
| `:550-552` | `int secCount = height / 16;` + `ChunkSection[] sections = new ChunkSection[secCount];` + 预取 `sections[secIdx] = chunk.getSection(secIdx);`（**section 引用一次性 hoist 到局部数组**） |
| `:553-558` | 循环结构：**`by`（y）外层** 0..height → `sec = sections[by >> 4]`、`sy = by & 15` → `z` → `x`；`int base = by * 256 + z * 16;` ⇒ 遍历顺序 = y→z→x（非 section→block） |
| `:560-561` | `if (id < 0 || id >= MAX_ID) throw new IllegalArgumentException(...)`（越界 id 直接抛，不静默） |
| `:562-572` | A1a 跳空气：`if (id == 0) { if (WBCHECK) { WB_AIR_SKIP++; if (!sec.getBlockState(x, sy, z).isOf(Blocks.AIR)) WB_STALE_NONAIR++; } if (SKIPAIR) continue; }`（判定用 `isOf(Blocks.AIR)` 而非 `isAir()`，理由见 `:565-568`） |
| `:573-585` | raw id → BlockState：`STATE_BY_ID.get(id)` 未命中则 `Registries.BLOCK.get(id)` → `b.getDefaultState()`，`null → AIR`，`STATE_BY_ID.set(id, st)`（`:575-580` 注释记录 M14 根因：buf 是 **block 注册表 raw id 域**，不是 `Block.STATE_IDS` state id 域） |
| `:586-588` | 注释「必须用 `ChunkSection.setBlockState`（内部=container.set + nonEmptyBlockCount 更新）：直写 container.set 不更新计数 → `isEmpty()` 误判 true → 全部读成空气（历史根因）」+ `sec.setBlockState(x, sy, z, st);`（**逐块**，`lock=true` → `PalettedContainer.swap` → 每次 lock/unlock） |
| `:594-600` | 6 张高度图 `Heightmap.populateHeightmaps(chunk, Set.of(WORLD_SURFACE_WG, WORLD_SURFACE, OCEAN_FLOOR_WG, OCEAN_FLOOR, MOTION_BLOCKING, MOTION_BLOCKING_NO_LEAVES))`（`:592-593` 注释理由：buildSurface 被跳过需一次性补齐） |
| （无） | `ChunkTiming.addWrite(...)` —— 1.20.1 **无此行**（无 ChunkTiming 分项；1.21.6 才有，`:412`） |
| `:603-611` | `destroy()`：`:607-611` WBCHECK 汇总（`air_skipped` / `stale_nonair`，`stale_nonair=0 ⇒ 跳过空气写与逐格写等价`）——1.20.1 独有 |

**writeChunk 对 chunk 做的全部操作（1.20.1）**：① `chunk.getSection(secIdx)` × `secCount`（= height/16，**一次性 hoist**，`:551-552`）；② 按 y→z→x 遍历，对每格 `sec.setBlockState(x, sy, z, st)`（A1a 后 = 非空气格数；`:588`）；③ 6 张高度图 `populateHeightmaps`（`:594-600`）。**不做**：block entity、`ChunkStatus`、光照、biome、section 对象替换。**无** `Chunk.setBlockState` 调用（绕开 `ProtoChunk/WorldChunk.setBlockState` 的 heightmap/light/blockEntity 开销——见 `:532` 注释「直写 PalettedContainer（跳过 chunk.setBlockState 的 heightmap/blockEntity 开销）」）。

### A2. `versions/1.21.6/java/src/main/java/wg/bench/CppBridge.java`（629 行）与 1.20.1 的差异

`writeChunk` = `:506-549`（1.20.1 = `:549-601`）。同口径：`:507-509` `secCount`/`sections[]` hoist（同 1.20.1 `:550-552`）；`:510-512` 循环头（`by` 外层 + `sec = sections[by >> 4]`）；`:516-518` id 越界抛；`:519-531` raw id→state 缓存；`:534` `sec.setBlockState(x, sy, z, st);`；`:540-548` 6 张高度图（同集合）+ `ChunkTiming.addHmap`；`fillChunk` 调用点 `:408`。**`:506-537` 内无任何 `id == 0` 分支**（逐块写含空气）。

**逐项差异行（1.20.1 → 1.21.6）**：

| # | 项 | 1.20.1 | 1.21.6 | 性质 |
|---|---|---|---|---|
| D1 | 跳空气（A1a） | `:543` `SKIPAIR` 开关 + `:562-572` `if (id == 0) {…WBCHECK…; if (SKIPAIR) continue;}` | **无**（`:506-537` 无 `id == 0` 分支）⇒ 逐块写含空气 | 1.21.6 缺 |
| D2 | raw id→state 缓存 | `:573-585` `STATE_BY_ID` + `Registries.BLOCK.get(id).getDefaultState()` | `:519-531` 同形态（`STATE_BY_ID`/`MAX_ID=4096`/`getDefaultState`） | 等价 |
| D3 | 分项计时钩子 | **无**（ChunkTiming 1.20.1 版只报 mixin/gap/inflight，见 `ChunkTiming.java:9-12` 声明） | `fillChunk`：`:385` `tj0`、`:393` `addJni`、`:399-402` `ts0`/`addScan`、`:406-412` `tw0`/`addWrite`（`tw0` 包住 `writeChunk` 全段）；`writeChunk`：`:540` `th0`、`:548` `addHmap`；`report()` 打印 `write = max(WRITE - HMAP, 0)`（`ChunkTiming.java:95`） | 1.21.6 独有 |
| D4 | nz 扫描 + println | `:414-418` O(1) 短路（buf[0] 非零即停）+ `:419-425` MIXLOG 内全量 nz | `:399-405` **无条件**全量 nz（98304 读）+ 无条件 `buf-all-air`（`:403`）/ `buf-sparse`（`:404-405`）println；`addScan` 计时包住该扫描 | 1.21.6 缺 A1b 且多 0.09ms/chunk 扫描 |
| D5 | `[WG-CONTENT]` 指纹门 | `:381-391` `wgBufHash` + `:419-425`/`:460-478`/`:509-529` 三维打印（MIXLOG 门控） | **全文 grep 零命中**（转引 `.investigations/cross-version-backfill-260911-05/scout-map.md:18`） | 1.21.6 缺 |
| D6 | WBCHECK / `skipair` / `destroy()` 汇总 | `:540-547` + `:606-611` | 无 | 1.21.6 缺 |
| D7 | nether/end 读回自证 | **整块门控**：`:460-478`（nether）/ `:509-529`（end）整段在 `if (MIXLOG)` 内——nzBuf 扫描（`:461-462`）+ 指纹（`:463-464`）+ 16 点读回（`:465-474`）全部只在门控内执行 | **仅 println 门控**：nether `:442-443` nzBuf 扫描**无条件** + `:444-451` 16 点读回**无条件** + `:452-453` println 由 MIXLOG 门控；end `:486-487` 扫描 + `:488-495` 读回无条件 + `:496-497` println 门控 | 1.21.6 每 chunk 多 65536/32768 次读（门控关时仍执行） |
| D8 | 阶段掩码 | 同（`:69-74`） | 同（`:75-80`） | 等价 |
| D9 | `StallWatch` / exec 模式 / P1 限流 | 有 | 缺（转引 `.investigations/cross-version-backfill-260911-05/scout-map.md:71-74`，本次未独立复核） | 1.21.6 缺 |

### A3. `buf` 分配 / 生命周期 / 映射 / native 签名

| 问题 | 事实 | 锚点 |
|---|---|---|
| 谁 new | Java：`BUF = ThreadLocal.withInitial(() -> new int[16*16*384])`，**per-thread 复用**（不 per-chunk 分配） | 1.20.1 `CppBridge.java:38-39`；1.21.6 `:44-45` |
| 长度如何确定 | Java 侧硬编码 16×16×384；Rust 侧 JNI 按 **Java 传入数组的实际长度**分配本地 buffer（overworld 98304 / nether 65536）——注释记录历史越界根因（硬编码 98304 拷进 65536 数组被吞） | `versions/1.20.1/rust/src/jni_bridge.rs:199-206`（`let out_len = env.get_array_length(&out0)?`；`local: Vec<Vec<i32>>`）；`versions/1.21.6/rust/src/jni_bridge.rs` 同（`:179-227`） |
| 是否 per-thread 复用 | Java buf：是（ThreadLocal）；Rust 本地 buffer：**否**，每次 JNI 调用 `Vec<Vec<i32>>` 新建 + 拷回后丢弃 | 同上 `:205` |
| 拷贝次数 | Rust 生产 → `col.data().to_vec()`（384KB 拷贝）→ JNI 本地 `local[i]`（拷贝）→ `set_int_array_region` 拷回 Java int[] | `worldgen-core/src/worldgen_handle.rs:682`；`jni_bridge.rs:205`、`:217-223` |
| raw id → BlockState | `Registries.BLOCK.get(id)` → `getDefaultState()`，缓存进 `AtomicReferenceArray<BlockState> STATE_BY_ID`（`MAX_ID=4096`）；**不用** `Block.STATE_IDS`（state id 域），**不用** `Registries.BLOCK.getRawId` | 1.20.1 `:33-36`、`:573-585`；1.21.6 `:39-42`、`:519-531` |
| JNI 方法签名 | `public static native int fillBlocks(long handle, int[] chunkXs, int[] chunkZs, int[][] outs, int threads);`（Javadoc：`outs[i] = int[16*16*384]`，vanilla raw block id，索引 `(y-MIN_Y)*256 + z*16 + x`，返回 count） | `wg/CppWorldgen.java:71`（**两版逐字相同**，已 diff 全文 108 行） |

---

## 2. 接管后的 vanilla 义务（B）

`NoiseChunkGeneratorMixin` 在 `populateNoise` HEAD cancel 后，vanilla 侧各义务归属如下（均为 vanilla 一手源）：

| 义务 | 谁负责 | 锚点（1.20.1 / 1.21.6） |
|---|---|---|
| **PRE_CARVER 高度图**（`OCEAN_FLOOR_WG` / `WORLD_SURFACE_WG`） | ① `ProtoChunk.setBlockState` 惰性补填（按当前 status 的 heightmap 类型集合，缺失才 `populateHeightmaps`）② CoreSwap `writeChunk` 自己填了 6 张（超集） | `ProtoChunk.java:135-151`；`ChunkStatus.java:33`（`PRE_CARVER_HEIGHTMAPS`）；`CppBridge.java:594-600`（1.20.1）/ `:541-547`（1.21.6） |
| **POST_CARVER 高度图**（`OCEAN_FLOOR`/`WORLD_SURFACE`/`MOTION_BLOCKING`/`MOTION_BLOCKING_NO_LEAVES`） | vanilla **FEATURES 阶段任务**在 `generateFeatures` 之前填充（不是 populateNoise 干的） | 1.20.1 `ChunkStatus.java:143-157`（`Heightmap.populateHeightmaps(chunk, EnumSet.of(MOTION_BLOCKING, MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, WORLD_SURFACE))` `:150-152`）；1.21.6 `ChunkGenerating.java:131-142`（`:135-137`） |
| **存档加载时高度图** | `ChunkSerializer` / `SerializedChunk` 读 NBT，缺失的才 `populateHeightmaps` | 1.20.1 `ChunkSerializer.java:199-208`；1.21.6 `SerializedChunk.java:289` |
| **光照初始化** | `ChunkStatus.INITIALIZE_LIGHT` / `LIGHT` 阶段 → `lightingProvider.initializeLight/light`（CoreSwap 另有 `ServerLightingProviderMixin` 在 `light()` HEAD 接管，`-Dcoreswap.light.rust` 门控） | 1.20.1 `ChunkStatus.java:158-179`；1.21.6 `ChunkGenerating.java:144-157`；`ServerLightingProviderMixin.java:52-229` |
| **block entity** | `populateNoise` **不创建 block entity**；由 features/结构放置（`start.place(...)`）与 `populateEntities` 负责 | 1.21.6 `ChunkGenerator.java:363/369`（结构 `start.place`）、`:401/:405`（`placedFeature.generate`）；`ChunkGenerating.java:159-167`（`populateEntities`） |
| **ChunkStatus 推进** | 由 MC 生成管线（`ChunkGenerating.populateNoise` 任务返回 future）驱动；cancel 只替换 generator 方法体，任务本身与后续阶段（`thenApply` 的 BelowZeroRetrogen）照常执行 | 1.21.6 `ChunkGenerating.java:78-100`（`context.generator().populateNoise(...).thenApply(...)` `:83-99`）；1.20.1 `NoiseChunkGenerator.java:329-357`（返回 `CompletableFuture`，`:348-350`） |
| **注意（下游改写者）** | `populateNoise` future 完成后 vanilla 还会跑 `BelowZeroRetrogen.replaceOldBedrock` / `fillColumnsWithAirIfMissingBedrock`，二者走 **`chunk.setBlockState(pos, …)` 逐块路径**（非 section 直写） | 1.21.6 `ChunkGenerating.java:87-96`；`BelowZeroRetrogen.java:59-76`（`:63`、`:76`） |

**vanilla `populateNoise` 自身的写回语义（对照基线）**：
- 写：`chunkSection.setBlockState(y, u, ab, blockState, false);` —— **`lock=false`**（`swapUnsafe`），且**只 track 两张 WG 高度图**：`heightmap.trackUpdate(...)` / `heightmap2.trackUpdate(...)`（`OCEAN_FLOOR_WG` / `WORLD_SURFACE_WG`）。
  - 1.20.1 `NoiseChunkGenerator.java:359-364`（取两张 WG 高度图）、`:416-418`
  - 1.21.6 `NoiseChunkGenerator.java:355-360`、`:412-414`
- 锁：外层方法对涉及的 section 逐个 `lock()`，future 完成时统一 `unlock()`（1.20.1 `:340-346`/`:351-355`；1.21.6 `:336-340`/`:346-349`）。**HEAD cancel 会跳过这层锁**（CoreSwap 侧注释：`NoiseChunkGeneratorMixin.java:41-44` 记录「本工程写回自带逐次 lock/unlock，外层再加锁 = 同线程自锁死（E1 实测 Chunky `Processed: 0`）」）。

### B5. `ChunkSection.setBlockState` 内部语义与派生状态

**1.20.1 `ChunkSection.java`（mcsrc）**：
- `:56-58` `setBlockState(x,y,z,state)` → `setBlockState(..., true)`；`:60-93` 实体：`lock ? blockStateContainer.swap(...) : swapUnsafe(...)`，随后维护 **3 个 short 派生计数器**：`nonEmptyBlockCount`（`:70-75`/`:81-86`）、`randomTickableBlockCount`（`:72-74`/`:83-85`，`state.hasRandomTicks()`）、`nonEmptyFluidCount`（`:77-79`/`:88-90`，非空流体）。
- 派生状态**除方块数组外还有**：`nonEmptyBlockCount` / `randomTickableBlockCount` / `nonEmptyFluidCount`（`:21-23`）+ `biomeContainer`（`:25`）。
- **全量重算入口**：`calculateCounts()` `:111-140`（`blockStateContainer.count(counter)` 逐 palette 项 × 权重）。**它被 public 构造器自动调用**：`:27-31` `ChunkSection(PalettedContainer, ReadableContainer)` → `this.calculateCounts();`。
- 消费方：`isEmpty()` `:95-97`、`hasRandomTicks()` `:99-109`、`toPacket` `:164-168`（`writeShort(nonEmptyBlockCount)`）。
- 每块写入的调色板代价：`PalettedContainer.swap` → `private swap(index,value)` `:158-162` 内 `this.data.palette.index(value)`（`ArrayPalette.index` = 线性扫描 `:46-62`；`BiMapPalette.index` = `Int2ObjectBiMap` 哈希 `:39-50`）→ `storage.swap`。锁只在外层 public `swap` `:141-152`（`lock`/`unlock` try-finally）。
- `PalettedContainer.count()` 优化：palette 只有 1 项时按 `storage.getSize()` 一次计数（1.20.1 `PalettedContainer.java:334-343`，`:336-337` 单 palette 短路；否则 `Int2IntOpenHashMap` 逐 storage 项，`:339-341`）⇒ 均质 section 的 `calculateCounts` 为 O(1)，非均质为 O(4096)。

**1.21.6 `ChunkSection.java`**：`:64-101` 同语义；派生字段 `:21-23` 同；`calculateCounts()` `:119-148` 同；**额外**：`private ChunkSection(ChunkSection section)` 拷贝构造 `:27-33` + `public ChunkSection copy()` `:205-207`（1.20.1 **没有** copy）。

⇒ 事实：**整段替换若走 `ChunkSection` 的 public 构造器，派生计数器会被 `calculateCounts()` 自动重算**（不需要手工维护）；但若改为「只替换 `blockStateContainer` 字段」，则 3 个 short 计数器**不会**自动更新（该字段 `private final`，无 setter）。

---

## 3. 可用 API 与既有 accessor 模式（C）

### C6. `ChunkSection` / `PalettedContainer` / `PackedIntegerArray` 公开构造器与 PaletteProvider

**`ChunkSection`（两版逐字相同签名）**
```java
// 1.20.1 ChunkSection.java:27-31
public ChunkSection(PalettedContainer<BlockState> blockStateContainer,
                    ReadableContainer<RegistryEntry<Biome>> biomeContainer) {
    this.blockStateContainer = blockStateContainer;
    this.biomeContainer = biomeContainer;
    this.calculateCounts();
}
// :33-38
public ChunkSection(Registry<Biome> biomeRegistry) { ... PalettedContainer.PaletteProvider.BLOCK_STATE ... }
// :142-144  public PalettedContainer<BlockState> getBlockStateContainer()
// :146-148  public ReadableContainer<RegistryEntry<Biome>> getBiomeContainer()
```
- 1.21.6 同：`:35-39`（public ctor）、`:41-46`、`:150-152`、`:154-156`、`:205-207`（`copy()`）。
- ⇒ **可从「Rust 数据构造的 PalettedContainer」+「既有 biomeContainer」直接 new 出 ChunkSection**（vanilla 自己就这么用：`ChunkSerializer.java:124` `ChunkSection chunkSection = new ChunkSection(palettedContainer, readableContainer);`；1.21.6 `SerializedChunk.java:170` 同）。

**`PalettedContainer` 公开构造器**
```java
// 1.20.1 PalettedContainer.java:92-102（1.21.6 :92-102 逐字相同）
public PalettedContainer(IndexedIterable<T> idList,
                         PalettedContainer.PaletteProvider paletteProvider,
                         PalettedContainer.DataProvider<T> dataProvider,
                         PaletteStorage storage,
                         List<T> paletteEntries) {
    this.idList = idList; this.paletteProvider = paletteProvider;
    this.data = new PalettedContainer.Data<>(dataProvider, storage,
        dataProvider.factory().create(dataProvider.bits(), idList, this, paletteEntries));
}
// :110-115（1.21.6 :116-121）
public PalettedContainer(IndexedIterable<T> idList, T object, PalettedContainer.PaletteProvider paletteProvider)
```
- `PaletteProvider.BLOCK_STATE`：1.20.1 `:420-430`；1.21.6 `:427-437`。`createDataProvider(idList, bits)` 分派：`0→SINGULAR`；`1..4→ARRAY(bits=4)`；`5..8→BI_MAP(bits)`；`default→ID_LIST(ceilLog2(idList.size()))`。
- `getBits(idList, size)` 1.20.1 `:480-484`（1.21.6 `:487-491`）：`i=ceilLog2(size)`，返回 `factory()==ID_LIST ? i : dataProvider.bits()`。
- `computeIndex(x,y,z)` 1.20.1 `:465-467`（1.21.6 `:472-474`）：`(y<<edgeBits | z)<<edgeBits | x`。
- `swap` / `swapUnsafe` / `set`：1.20.1 `:141-152` / `:154-156` / `:164-172`（`swap` 走 `LockHelper`，`swapUnsafe` 不走）。
- `copy()`：1.20.1 `:325-327`；`Data.copy`：`:389`（`record Data<T>` 定义于 `:361`）。
- `get(x,y,z)` `:180`；public `set(x,y,z,value)` `:164-172`（**锁**：`lock()` + try/finally `unlock()`）；private `set(index,value)` `:174-177`（不锁）；`slice()` `:329-332`。

**`PackedIntegerArray`（两版构造器逐字相同）**
```java
// 1.20.1 PackedIntegerArray.java:223  public PackedIntegerArray(int elementBits, int size, int[] data)
// :252  public PackedIntegerArray(int elementBits, int size)
// :256  public PackedIntegerArray(int elementBits, int size, @Nullable long[] data)
// :257-270  Validate.inclusiveBetween(1,32,elementBits); data.length 必须 == (size+elementsPerLong-1)/elementsPerLong，否则抛 InvalidLengthException
```
（1.21.6 同文件同行号 `:223`/`:252`/`:256`。）

**palette 索引语义（决定「Rust 产出 palette + packed data」契约的机械约束）**

| Palette 实现 | 索引语义 | 锚点 |
|---|---|---|
| `SingularPalette` | 只有 index 0 | 1.20.1 `SingularPalette.java:20-27`、`:38-46`、`:57-64` |
| `ArrayPalette` | **list 顺序 = 索引顺序**（`for i<list.size() array[i]=list.get(i)`） | `ArrayPalette.java:20-32`、`:46-62` |
| `BiMapPalette` | `entries.forEach(map::add)` ⇒ 插入序 = 索引序 | `BiMapPalette.java:19-22`、`:39-50` |
| `IdListPalette` | **索引 = `idList.getRawId(object)`（全局 Block.STATE_IDS raw id），构造时 `list` 被忽略** | `IdListPalette.java:19-27`、`:34-42` |

⇒ 机械事实（事实陈述，非评价）：bits ≥ 9 的 section 走 `ID_LIST`，其 storage 索引是**全局 block state id**，不是局部 palette 索引；而 Rust 侧目前只有 **block raw id**（`blocks.rs` `BlockId = i32` = vanilla block 注册表 raw id，`blocks.json` 数据驱动）。且 `IdListPalette.create` 忽略传入的 palette 列表，storage 宽度须取 `dataProvider.bits()`（= `ceilLog2(Block.STATE_IDS.size())`）而非 `getBits()` 返回的 `i`。

### C7. 项目已存在的 accessor/invoker mixin（可照此模式扩展）

| 文件 | 目标 | 形态 |
|---|---|---|
| `mixin/ChunkLightProviderAccessor.java:13-17`（两版相同） | `@Mixin(ChunkLightProvider.class)` | `@Accessor("chunkProvider") ChunkProvider wgGetChunkProvider();` |
| `mixin/LightingProviderAccessor.java:11-15`（两版相同） | `@Mixin(LightingProvider.class)` | `@Accessor("blockLightProvider") ChunkLightProvider<?,?> wgGetBlockLightProvider();` |
| `mixin/ThreadedAnvilChunkStorageAccessor.java:11-15`（仅 1.20.1） | `@Mixin(ThreadedAnvilChunkStorage.class)` | `@Invoker("releaseLightTicket") void wgReleaseLightTicket(ChunkPos pos);` |

- 全部为 **interface + `@Accessor`/`@Invoker`** 形态（无 `@Shadow` 实现类），文件头注明「未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）」+ 依赖 yarn 签名。
- 现存 `@Shadow` 用法示例：`ServerLightingProviderMixin.java:59` `@Shadow private ThreadedAnvilChunkStorage chunkStorage;`。
- **本项目目前没有** `ChunkSection` / `PalettedContainer` / `Heightmap` / `Chunk` 的 accessor（grep `PalettedContainer|HeightmapAccessor|ChunkSectionAccessor` 全 `versions/` 仅命中注释行 `CppBridge.java:532`/`:503` 与 mixin 注释）。

### C8. `Heightmap` 写入接口

| 成员 | 1.20.1 | 1.21.6 |
|---|---|---|
| `storage` | `private final PaletteStorage storage;` `:26`；`new PackedIntegerArray(ceilLog2(chunk.getHeight()+1), 256)` `:33-34` | `:32`、`:39-40` |
| `populateHeightmaps(Chunk, Set<Type>)` | `public static` `:37-71` | `public static` `:43-79`（多一行 `if (!types.isEmpty())` 守卫 `:44`） |
| `trackUpdate(x,y,z,state)` | `public boolean` `:73-100` | `public boolean` `:81-108` |
| `set(x,z,height)` | **`private`** `:114-116` | **`private`** `:122-124` |
| `setTo(Chunk, Type, long[])` | **`public`** `:118-126`（长度相等 → `System.arraycopy`；不等 → warn + 重算） | **`public`** `:126-134` |
| `asLongArray()` | `public long[]` `:128-130`（返回 `storage.getData()` **直接引用，非拷贝**） | `public long[]` `:136-138` |
| `Type` | enum `:142-152`（6 值 + `getBlockPredicate()`） | enum `:150-161`（多 `index` / `PACKET_CODEC`） |
| `Chunk` 侧入口 | `public void setHeightmap(Type, long[])` `Chunk.java:175-177` → `getHeightmap(type).setTo(...)`；`public Heightmap getHeightmap(Type)` `:179-181` | `Chunk.java:183-185`、`:187-189` |
| 已有 accessor | **无** | **无** |

⇒ 事实：**高度图存在现成的「整段写」公开通道**：`Chunk.setHeightmap(type, long[])`（`Chunk.java:175-177` / `:183-185`）→ `Heightmap.setTo` → `arraycopy`。不需要新增 accessor；`asLongArray()` 可直接取底层 `long[]` 做对照。

### C9. 「替换整个 section」的既有路径

| 问题 | 事实 | 锚点 |
|---|---|---|
| `Chunk.setSection(...)`？ | **不存在**（两版 grep 无命中） | `.tmp/.../Chunk.java` grep `setSection` → 0 |
| section 数组字段 | `protected final ChunkSection[] sectionArray;`（1.20.1 `:87`；1.21.6 `:90`） | 同 |
| 取数组 | `public ChunkSection[] getSectionArray() { return this.sectionArray; }` —— **返回活引用，非 clone** | 1.20.1 `:163-165`；1.21.6 `:171-173` |
| ⇒ 能否替换元素 | **可以，无需 mixin**：`chunk.getSectionArray()[i] = newSection;`（数组内容可变，字段 final 只约束引用重绑） | 同上 |
| `@Accessor` 到 `Chunk.sections`？ | 字段实际名 `sectionArray`（yarn）。若要 accessor：`@Accessor("sectionArray")`（`final` 字段仅读无需 `@Mutable`；写字段引用需 `@Mutable`）。**本次未在任何 mod 中见过该 accessor 实例** ⇒ 属「可照 C7 模式新增」，非既有 | — |
| 既有 section 对象替换先例 | vanilla 序列化读路径 `new ChunkSection(palettedContainer, readableContainer)` 后放进 `chunkSections[]`（构造期） | 1.20.1 `ChunkSerializer.java:93/124`；1.21.6 `SerializedChunk.java:170/211` |
| 序列化读 section | 写档取 `chunk.getSectionArray()`（活引用）；1.21.6 还会 `chunkSections[j].copy()` 再编码 | 1.20.1 `ChunkSerializer.java:294`、`:309-310`；1.21.6 `SerializedChunk.java:334`、`:345`、`:426-428` |

---

## 4. Rust 输出契约与改动面（D）

### D10. 填充函数输出契约

| 层 | 事实 | 锚点 |
|---|---|---|
| C ABI 入口 | `pub extern "C" fn wg_fill_blocks_multi(handle, chunk_xs, chunk_zs, outs, count, threads) -> c_int` | `worldgen-core/src/api.rs:134`（函数体至 `:183`） |
| 缓冲区由谁分配 | **调用方分配**：`outs: *mut *mut c_int`（每 chunk 一个 out 指针）；Rust 只写不分配 | `api.rs:134-183`；`struct SendOut(*mut c_int)` + `write()` `api.rs:49-57`（`copy_nonoverlapping`） |
| 是否 per-thread 复用 | **否**：每次调用 `std::thread::scope`（`:158`）+ `s.spawn`（`:164`）重建 `nthreads` 个线程，交错处理 `{t, t+n, …}`（`:157`、`:165-177`），调用结束即回收；`SendOut` 按 chunk 索引写各自 out（`:156`、`:176`） | `api.rs:154-181`；`adaptive_threads` `:24-44`（注释 `:38-43`：原常驻池已不存在） |
| 行为化自证 | `WG_THREADLOG=1` → 首调用一次 `[WG-THREADS] count=… threads_param=… nthreads=…`（`AtomicBool` 门控，热路径零 env 查询） | `api.rs:144-152` |
| 返回长度如何计算 | 返回 `count`（= `chunk_xs` 长度）；单 chunk 数据长度 = `blocks.len()` = `16*16*height`（BlockColumn 尺寸，overworld 98304 / nether 65536） | `api.rs:134-183`；`blocks.rs:155-159` |
| 单 chunk 生产 | `WorldgenHandle::fill_chunk_blocks(&self, cx, cz) -> Vec<BlockId>`：terrain_cache 命中/直算 → surface → carver → features → `col.data().to_vec()` | `worldgen-core/src/worldgen_handle.rs:613-683`（`:682` `col.data().to_vec()`） |
| 索引布局 | `(y - min_y) * 256 + z * 16 + x`（y-major） | `blocks.rs:152-159`、`:170-178`；`CppWorldgen.java:68-69` |
| block id 体系 | `pub type BlockId = i32;` = vanilla block 注册表 **raw id**；`AIR = 0`；`BlockRegistry` 从 `blocks.json` 加载（`name_to_id` / `id_to_name`，容量 16384）+ 运行时 `register` / `register_with_id` | `blocks.rs:9`、`:11`、`:14`、`:16-27`、`:30-54`、`:71-89` |
| 无 palette / 位打包 | 全 `worldgen-core/src` grep `palette|bits_per|packed|bitpack`：**无任何 palette / 位打包实现**（命中均为 `packed_ice` 方块名与无关注释） | — |
| Java 侧 JNI | `Java_wg_CppWorldgen_fillBlocks`：本地 `Vec<Vec<i32>>`（按 Java 数组实际长度）→ `wg_fill_blocks_multi` → 主线程 `set_int_array_region` 拷回 | `versions/1.20.1/rust/src/jni_bridge.rs:179-227`（`:199-206`、`:208-215`、`:217-223`）；1.21.6 同 |
| Java native 声明 | `public static native int fillBlocks(long, int[], int[], int[][], int);` | `wg/CppWorldgen.java:71`（两版相同） |

### D11. 「额外产出 section palette + 位打包 data」的改动面（事实清单，不含评价）

会被牵动的模块/函数：

| 层 | 需触碰点 | 锚点 |
|---|---|---|
| Rust core 输出 | `fill_chunk_blocks` 目前返回**扁平 `Vec<BlockId>`**；新增 section 级产物需在 `fill_chunk_blocks` 或新函数内对 24 个 section 各做 palette 收集 + 位打包（`worldgen-core` 现无此代码） | `worldgen_handle.rs:613-683`；`blocks.rs:155-183` |
| Rust C ABI | 新增导出（新 `wg_fill_sections_*` 或扩展 `wg_fill_blocks_multi` 输出结构）；现有 `outs: *mut *mut c_int` 是扁平 int 流，承载「palette + long[] 位打包」需新 ABI 形态 | `api.rs:134-183`、`SendOut` `:49-57` |
| JNI 层 | 新增 `Java_wg_CppWorldgen_*` 导出 + Java `CppWorldgen` native 声明；现有 fillBlocks 的本地 buffer + 拷回模式可参照 | `jni_bridge.rs:179-227`；`CppWorldgen.java:71` |
| block id 体系 | Rust 只有 **block raw id**（`blocks.json`）；`PalettedContainer` 的 `idList` 是 `Block.STATE_IDS`（**block state id** 域）。两域之间 Rust 侧无映射表 ⇒ 若 Rust 要直接产出「可构造的 palette」，须新增 raw-id→state-id 数据源（新 JSON 或 Java 侧下发） | `blocks.rs:9`/`:30-54`；`ChunkSection.java:34`（`new PalettedContainer<>(Block.STATE_IDS, Blocks.AIR.getDefaultState(), PaletteProvider.BLOCK_STATE)`） |
| 数据驱动边界声明 | `worldgen-core/data-driven-boundary.md` 现有边界表：block id = 数据驱动（`blocks.json` → `BlockRegistry::load_from_json`，`:14`）；「算法/流程与版本无关的部分保持代码（不数据驱动）」（`:42`）；「block id 一律数据驱动（经 `blocks.id` 或从 JSON config 解析），不硬编码数字」（`:40`）。**该文件未提及 palette / 位打包 / 序列化布局任何条目**（全文 55 行已读） | `data-driven-boundary.md:14`、`:40`、`:42`、`:19` |
| 版本共享性 | `worldgen-core` = rlib（两版共享），`versions/<ver>/rust` = 薄壳 cdylib ⇒ Rust 侧新增 ABI 一次覆盖两版；Java 侧 `PalettedContainer` 构造是每版独立工程 | `AGENTS.md` §〇 / 架构计划-260911-05.md:29-31 |

---

## 5. 成本证据（E）

| # | 来源 file:line | 载体 / 条件 | write 分项数值（per-chunk） | 备注 |
|---|---|---|---|---|
| E1 | `.investigations/vivo-stutter-260911-02/f2-phase1-cost-split.md:10` | `[PERFPROF] n=4096 fill=95.77 scan=0.09 write=6.39 height=0.42 sum=102.67 ms/chunk`；dev server + Chunky r500（4225 chunks，seed 7349435828306001495，center -48/-11），dll `dd3b645f`，factory mask=3，async，池宽 23 | **write = 6.39 ms/chunk** | 同期 `[CHUNKTIME] mixin=102.60`（`:11`）与 sum 吻合 |
| E2 | 同上 `:14-21` 表格 | 同上（「逐块写回（98,304 次含空气 `setBlockState`）」） | 6.39 ms/chunk = **6.2%**；`fillBlocks` JNI 95.77 = 93.3%；nz 扫描 0.09 = 0.1%；6 高度图 0.42 = 0.4% | 结论句 `:21`「上限合计仅 ~6.5%。写回与高度图**不是**低帧根因」；`:42`「Java 侧写回/高度图 —— 量级排除」 |
| E3 | `.investigations/000-架构设计/架构计划-260911-05.md:39` | A1a 项表 | 「上限（F2 成本拆分实测）= **≤6.2%**」 | `:45` 判据纪律：上限 ≤6.2% 合计 ⇒ 单 run 大概率淹没在 ±10% 噪声（#103），须用「同批配对交错」或指纹/计数型判据；`:113` 同 |
| E4 | `.investigations/perf-regression-260910-04/record.md:60` | `[CHUNKTIME] n=1024 perChunk(ms): mixin=51.00 [jni=47.15 write=3.50 hmap=0.22 scan=0.08 beard=0.04]` | **write = 3.50 ms/chunk**（1.21.6 口径，`write` 已扣 hmap：`ChunkTiming.java:95` `max(WRITE-HMAP,0)`） | 与 E1 的 6.39 不同载体/构建态（1.21.6 vs 1.20.1、不同池宽/机器负载），**不可互引**（§9.7 可比性） |
| E5 | `.investigations/a1-opt-pool-260911-05/review-260911-05.md:63` | judge 审查意见 | 「f2 成本拆分数值（A1c 0.42ms/chunk、A1a **≤6.2% 上限**）来自更早工作块，本块未复核」 | 数值来源标注为「更早工作块、未复核」 |
| E6 | `.investigations/vivo-stutter-260911-02/f2-phase1-cost-split.md:56` | 候选修法优先级表 P3 | 「Java 写回跳过空气（F2-1）→ 预期 ~6%」 | — |
| E7 | 1.21.6 同步臂日志（**未归档到 `.investigations/`**，仅 `.tmp/perf-reg-260910-04/logs/r3sync-r1.log`） | `[CHUNKTIME] n=4608` 末行 | 本次未读到该文件（`.tmp` grep 仅命中 `ChunkTiming`/`CHUNKTIME` 的文档提及，未含 r3sync 日志正文）⇒ **write 分项未核** | 归档状态见 `workflow-patterns.md:1748-1749`（引用者自标「未归档」） |
| E8 | `.investigations/perf-regression-260910-04/cmd-output/r3-async-r1.log` | 异步臂完整日志 | 本次**未读该文件**（未核具体行）；`[CHUNKTIME] n=4608` 末行 = 第 147 行（转引 `workflow-patterns.md:1748`） | 转引，非一手 |
| E9 | 1.20.1 侧 `[CHUNKTIME]` | 1.20.1 `ChunkTiming` 只报 mixin/gap/inflight（`ChunkTiming.java:9-12` 声明「1.20.1 侧 CppBridge 未挂 JNI/write/hmap/scan/carve/feat 分项计时钩子」）⇒ **1.20.1 无 write 分项** | 1.20.1 分项写回成本只能用 `[PERFPROF]`（临时计时器，已移除）或 E1 | `.investigations/perf-reg-260910-07/cmd-output/run_arms_1201.ps1:6` 同口径说明 |

**1.21.6 归档 `[CHUNKTIME]` 行（一手读到，含 write 分项）**：本次 grep 命中 `.investigations/` 内 r3sync 臂 `[CHUNKTIME] n=512/768 perChunk(ms): mixin=32.83/27.25 [jni=0.00 write=0.00 hmap=0.17/0.13 scan=0.00 beard=0.03]`（write=0.00 疑为 jni 为 0 的同一异常行）——**来源文件与行号本次未定位到一手路径**（grep 结果被截断，落盘于 spill 文件）⇒ 标「未核来源」，不作为数值依据。

---

## 6. 风险面与可用验证载体（F）

### F14. bulk 写回可能破坏的既有行为 / 下游消费者

| # | 面 | 事实 + 锚点 |
|---|---|---|
| R1 | **派生计数器** | 若只替换 `blockStateContainer` 字段（`private final`，无 setter），3 个 short 派生计数（`nonEmptyBlockCount`/`randomTickableBlockCount`/`nonEmptyFluidCount`）**不会**自动更新 → `isEmpty()`/`hasRandomTicks()`/`toPacket` 失真。走 public 构造器则 `calculateCounts()` 自动重算。锚点：`ChunkSection.java:21-23`、`:27-31`、`:56-93`、`:95-109`、`:164-168`（1.21.6 `:21-23`、`:35-39`、`:64-101`、`:119-148`、`:172-176`） |
| R2 | **光照引擎读方块** | 光照读取路径**每次现查 section**（不缓存 ChunkSection 实例）：`ChunkLightProvider.getStateForLighting` → `getChunk(i,j)` → `lightSourceView.getBlockState(pos)`。锚点：`.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/light/ChunkLightProvider.java:72-76`、`:94-103`。CoreSwap 侧另有 section 直采实现（`ServerLightingProviderMixin.java:111-112` 注释复刻 `WorldChunk#getBlockState` 语义；`:155` `ChunkSection[] secs = nc.getSectionArray();`；`:172` `sec.getBlockState(...)`） |
| R3 | **`Chunk.getBlockState` 生成期消费者** | 默认 stageMask=0b011 ⇒ **Java vanilla carver/features 在 Rust 填好的方块上运行**（Rust 跳过 carver/features）；surface 由 Rust 接管（cancel）。锚点：`CppBridge.java:66-74`（1.20.1）/`:72-80`（1.21.6）阶段掩码；`NoiseChunkGeneratorMixin.wgBuildSurface` 1.20.1 `:404-417`、1.21.6 `:239-252`（`ci.cancel()`）；探针型消费者（诊断，非生产）`SurfaceDumpProbeMixin.java:139`、`NoiseDumpProbeMixin.java:86`、`DiagFeatureBiomeMixin.java:72`（`for (ChunkSection sec : c.getSectionArray())`） |
| R4 | **高度图** | 6 张 WG/非 WG 高度图在 `writeChunk` 内 `populateHeightmaps`；vanilla FEATURES 阶段还会再填 4 张（POST_CARVER）；`ProtoChunk.setBlockState` 会按 status 惰性补填。整段替换若改变 section 对象，`populateHeightmaps` 通过 `chunk.getBlockState` 读方块（走活数组）⇒ 语义跟随；但**顺序/时机**若改动会影响 CARVERS 前后高度图一致性。锚点：`CppBridge.java:594-600`（1.20.1）/`:541-547`（1.21.6）；`ChunkStatus.java:143-157`；`ChunkGenerating.java:131-142`；`ProtoChunk.java:135-151` |
| R5 | **block entity（结构件）** | `populateNoise` 不产 block entity；结构件放置与 feature 在**同一方法同循环**（`ChunkGenerator.java:363/369` 结构 `start.place` vs `:401/405` `placedFeature.generate`）——历史先例：1.21.6 `ChunkGeneratorFeaturesMixin` v1 用 HEAD cancel 连坐了结构放置段，v2 改 `@Redirect` 精准拦截（mixin 注释 `:20-39`，一手注入点 `:59-64`/`:136-148`）。bulk 写回若被移动到 FEATURES 之后/之内，会与该放置段交互 |
| R6 | **存档序列化** | 写档取活数组 + `getBlockStateContainer()` 编码：`ChunkSerializer.java:294`、`:309-310`；1.21.6 `SerializedChunk.java:334`、`:345`（`copy()`）、`:426-428`。1.21.6 用 `copy()` 说明存在「编码期与生成期并发」的既有防护 |
| R7 | **1.21.6 独有** | `ChunkGeneratorFeaturesMixin` + `CppBridge.rustFeaturesTakeover()`（`mixin:68`、`:139-147`）；`NoiseChunkGeneratorTimingMixin`（carve 计时，`:16-31`）；`ChunkTiming` 分项钩子（`CppBridge.java:385-412`、`:540-548`）——bulk 改动会影响 `addWrite` 的语义边界（`tw0` 覆盖 writeChunk 全段，`ChunkTiming.java:88-102` 报告口径 `write = max(WRITE-HMAP,0)`） |
| R8 | **`populateNoise` future 后置改写者** | `BelowZeroRetrogen.replaceOldBedrock` / `fillColumnsWithAirIfMissingBedrock` 在 future 完成后逐块 `chunk.setBlockState` 改写（仅 belowZeroRetrogen != null 时）。锚点：`ChunkGenerating.java:87-96`；`BelowZeroRetrogen.java:59-76` |
| R9 | **section 锁语义** | vanilla 在 fill 前逐 section `lock()`、future 完成 `unlock()`；HEAD cancel 跳过该层。CoreSwap 逐块 `setBlockState`（lock=true）自带 lock/unlock；整段替换若完全绕过 `setBlockState`，**该层锁也随之消失**。锚点：`NoiseChunkGenerator.java:340-346`/`:351-355`（1.20.1）、`:336-340`/`:346-349`（1.21.6）；`ChunkSection.java:48-54`；`PalettedContainer.java:44-53`；CoreSwap 自述 `NoiseChunkGeneratorMixin.java:41-44`（1.20.1）、`:220-222`（E1 自锁死现场） |

### F15. 可直接用于本变更验证的既有等价性判据 / 载体

| 载体 | 现状锚点 | 对本变更（bulk 写回）的覆盖面 |
|---|---|---|
| `[WG-CONTENT]` 逐 chunk 指纹门（FNV-1a 64） | `CppBridge.java:381-391`（`wgBufHash`）、`:419-425`（overworld）、`:460-478`（nether）、`:509-529`（end）；判据用法 = 两臂逐 chunk hash 相同、sorted diff = 0（`.investigations/000-架构设计/架构计划-260911-02-p1.md:26`、`.investigations/vivo-stutter-260911-02/exec-mode-result-260911-03.md:17`、`.investigations/a1-opt-pool-260911-05/a2-dim-gate-260911-05.md:24`） | ⚠️ **事实：该门对 `buf`（Rust 输出）计算 hash**，而 bulk 写回只改 Java 侧消费 ⇒ **buf 不变则 hash 必然相同**，该门**不覆盖写回路径**（对 bulk 回归不敏感）。1.21.6 侧该门**缺失**（`.investigations/cross-version-backfill-260911-05/scout-map.md:18`） |
| region 逐块对拍（存档读回、逐 section 全量） | 工具 `.tmp/hang-repro-260910/diff_arms.py`、`diff_1201.py`；三臂归档 `.investigations/perf-regression-260910-04/cmd-output/region-{r3,r3sync,vanilla-260910-04}/`；数字落盘 `diff-r3-vs-vanilla.txt` 等（`workflow-patterns.md:1753`、`:1785`）；判据口径 = 非零容忍，差值 ≤ 既有 run 级非确定基线（`workflow-patterns.md:1780`） | ✅ **唯一直接覆盖写回产物的门**（读存档方块，绕不过写回） |
| `[WG-FILL]` 逐 chunk 读回自证 | 1.21.6 `CppBridge.java:442-443`/`:486-487`（nether/end nzBuf 扫描）+ `:452`/`:496` println（MIXLOG 门控）；`MIXLOG` 定义 `:30-35` | 部分覆盖（nether/end；overworld 无此行） |
| `WBCHECK` per-chunk 自证计数 | 1.20.1 `CppBridge.java:540`（开关）、`:544-547`（两个 AtomicLong）、`:563-570`（逐格自检 `stale_nonair`）、`:607-611`（destroy 汇总） | 覆盖「写回错位」类回归（A1a 配套），1.21.6 缺 |
| golden 逐位不变对照法（#47） | `knowledge/discovered/workflow-patterns.md:821-851`（`:828` 三步协议：golden_pre/post 逐用例 FNV 等值 + diff 为空 = PASS）；互补关系 `:851`（「golden 逐位法适用于可冻结输入的纯函数内核；双采集适用于无法冻结输入的运行时循环」） | ⚠️ 适用于 **Rust 纯函数内核**（可冻结输入）；对 Java 写回（实机世界状态）不直接适用 |
| block_probe 逐位对比 | `AGENTS.md` §五 载体表（`build-msvc/bin`，Full 层）；调用契约坑 `knowledge/discovered/build-tooling.md:356-374`；载具可比性 `workflow-patterns.md:503-514`、`:577-585` | ⚠️ C++/Rust 探针侧（自带全程管线），**不经过 Java 写回** |
| 执行体三元组（#36） | `架构计划-260911-05.md:54`（jar sha / jar 内 dll sha / target dll 一致）；多例 `review-002-final.md:13`、`concern2-regression-verdict-260906-04.md:11`、`a1-opt-pool-260911-05/record-260911-05.md:9` | 前置纪律（防跨执行体迁移结论），任何 A/B 都需 |
| §9.7 验证可比性声明 | `AGENTS.md` §一.7；实例（探针口径 96.06% vs 存档口径 82.16% 不可比） | 本变更量化结论 MUST 同行声明载体/覆盖面/可比性 |

---

## 7. 未核 / 需人工核

1. **E8/E7 一手 `[CHUNKTIME]` 日志正文未读**：`.investigations/perf-regression-260910-04/cmd-output/r3-async-r1.log`（第 147 行 `n=4608`）与 `.tmp/perf-reg-260910-04/logs/r3sync-r1.log`（第 165 行）本次均未直接打开；E4 的 `write=3.50` 来自 `record.md:60` 转引（`n=1024`）。1.21.6 write 分项**区间/中位数未核**（只核到单点 3.50 与 0.00 两条）。
2. **1.21.6 `ProtoChunk.setBlockState` 光照分支行号未核**：本次只读到 `:140-179`（heightmap 段 `:143-163`）；`status.isAtLeast(INITIALIZE_LIGHT)` 光照分支的 1.21.6 行号未读到（1.20.1 已核 = `:123-133`）。
3. **生成管线调用方未核**：谁调用 `ChunkGenerating.populateNoise`（`ChunkStep`/`ChunkTaskScheduler` 接线）未读源码；「cancel 不影响 status 推进」的结论依据是任务层 `context.generator().populateNoise(...)` 返回 future（`ChunkGenerating.java:78-100`）而非管线源码直证。
4. **section 替换的并发/缓存风险未审**：是否有任何消费者在 NOISE→LIGHT 之间缓存 `ChunkSection` 实例引用（`ChunkSkyLight`、`ChunkToLightData`、`ChunkNibbleArray` 持有者、mod/Connector 侧 mixin）本次未审；`Chunk.getSectionArray()` 返回活数组的**线程安全性**（与 1.21.6 序列化 `copy()` 的并发场景）未审。
5. **`IdListPalette`（bits≥9）路径的完整构造可行性未验**：`dataProvider.bits()`（= `ceilLog2(Block.STATE_IDS.size())`）与 `getBits()` 返回值的差异已在源码读到，但「Java 侧把局部 palette 索引展开成全局 state id 并重打包」的具体成本/正确性**未实测**（本次无 shell，无法取 `Block.STATE_IDS.size()` 实测值）。
6. **`ChunkSection` 构造后的 `calculateCounts()` 实测成本未测**：源码路径已核（`ChunkSection.java:27-31` → `:111-140`；`PalettedContainer.count()` 单 palette O(1) 优化 `:334-343`），但 24 section/chunk 的真实开销**未实测**。
7. **`[WG-CONTENT]` 的 1.20.1 打印位置措辞**：`overworld` 在 `writeChunk` 前（`:419-425` vs `:427`）、nether/end 在其后（`:455` vs `:460-478`）——judge I1 已指出记录措辞不符（`a1-opt-pool-260911-05/review-260911-05.md:54`、`a2-dim-gate-260911-05.md:29`）；本次一手核对与该结论一致，但「hash 语义等价」依赖 `buf` 只读性（未逐字验证 writeChunk 不改 buf——**未核**，仅 1.20.1 `:562-572` 循环体内未见写 buf）。
8. **`addWrite` 的包围范围已核一半**：1.21.6 `:406` `long tw0 = ChunkTiming.ON ? System.nanoTime() : 0L;`、`:407-411` try{writeChunk}、`:412` `addWrite` —— 已逐行核到；**未核**的是「`write` 分项是否含 `populateHeightmaps`」（由 `ChunkTiming.java:95` 的 `max(WRITE-HMAP,0)` 反推 = 含，但 `th0`/`addHmap` 在 `writeChunk` 内 ⇒ 报告值已扣除；**该减法口径未在 1.21.6 实测数据上验证**）。
9. **`versions/1.20.1/data/mc_src_extract/` 与 `.tmp/scout-260905-08/mcsrc/` 谁为权威**：两套 1.20.1 源码本次抽样核对（ChunkSection / Heightmap）行号与内容一致，但**未全树核对**；引用时以两套一致的抽样项为准，其余按 `.tmp/scout-260905-08/mcsrc` 单套来源。
10. **`data-driven-boundary.md` 未覆盖项**：该文件对「palette / 位打包 / 序列化布局」零条目（全文 55 行已读）⇒ 若本变更引入新数据源（raw-id→state-id 表），边界文档需新增条目，**当前无既有声明可继承**。
