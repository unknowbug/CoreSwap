# P1a — CoreSwap（Java mod 侧 + Rust 侧）已接管世界生成阶段执行形态测绘

- 工作块：260915-01（形态审计 P1a，计划 `.investigations/000-架构设计/架构设计-260914-04b-形态审计.md`）
- 角色：recode-scout（只读勘探，不做优劣判断——错配判定是 P2 worker 的事）
- 日期锚：Get-Date 2026-09-15 15:06
- 状态：draft（勘探产物，只写 .investigations/）
- 范围：Java 侧 = `versions/1.20.1/java`（mixin）+ `java-core`（共享 CppBridge/CppWorldgen/BulkWb）；Rust 侧 = `worldgen-core` + `versions/1.20.1/rust`

---

## 1. 接管面全集清单

### 1.1 mixin 注册面（唯一生产 mixin 配置）

`versions/1.20.1/java/src/main/resources/coreswap.mixins.json:6-31` 共注册 24 个 mixin。其中**生产接管面只有 2 个**（下方 A/B）；其余全部为探针/诊断 mixin（property 门控日志，见 §1.4）。另有 4 个 Accessor（LightingProviderAccessor / ChunkLightProviderAccessor / ChunkSectionAccessor / ThreadedAnvilChunkStorageAccessor，:27-30）供光照路径取私有字段。

### 1.2 生产接管段（Java 注入点 × Rust 入口）

| # | 阶段 | Java 注入点 | 注入方式 | 接管/让位条件 | Rust 侧入口 |
|---|------|------------|---------|--------------|------------|
| A | **NOISE/fill**（含 density/aquifer/ore vein/surface rules/heightmap，整块生成） | `NoiseChunkGeneratorMixin.wgPopulateNoise`（`versions/1.20.1/java/.../mixin/NoiseChunkGeneratorMixin.java:273-283`）注入 `NoiseChunkGenerator.populateNoise(Executor, Blender, NoiseConfig, StructureAccessor, Chunk)` | `@Inject(at=@At("HEAD"), cancellable=true)` + `cir.setReturnValue(wgDispatch(work))` | `CppBridge.enabled`（-Dcpp.replace=1，init 成功置位，`java-core/.../CppBridge.java:112-113`）+ 形状匹配 + settings id ∈ {minecraft:overworld(-64/384), minecraft:nether(0/256), minecraft:end(0/256 + endActive)}（Mixin:288-293, 295, 332, 363）；不匹配 → 放行 vanilla（:392-394 一次性日志） | `CppBridge.fillChunk*`（:451/507/558）→ JNI `CppWorldgen.fillBlocks` → `Java_wg_CppWorldgen_fillBlocks`（`versions/1.20.1/rust/src/jni_bridge.rs:179-227`）→ `wg_fill_blocks_multi`（`worldgen-core/src/api.rs:134-183`）→ `WorldgenHandle::fill_chunk_blocks`（`worldgen-core/src/worldgen_handle.rs:613`） |
| A' | **Beardifier 喂入**（A 段前置，非独立阶段） | 同上 populateNoise 拦截点内、fillChunk 之前调用（Mixin:310/343/372） | —（拦截点内的顺序调用） | 同 A | `CppBridge.feedBeardifier*`（反射序列化 piece/junction int[]，CppBridge.java:348-406）→ JNI `setBeardifier`（jni_bridge.rs:232-263）→ `wg_set_beardifier`（api.rs:189-200） |
| B | **SURFACE** | `NoiseChunkGeneratorMixin.wgBuildSurface`（Mixin:399-416）注入 `NoiseChunkGenerator.buildSurface(ChunkRegion, StructureAccessor, NoiseConfig, Chunk)` | `@Inject(at=@At("HEAD"), cancellable=true)` + `ci.cancel()` | 同 A 的三维判定（Mixin:406-413）；surface rules 已在 Rust `wg_fill_blocks` 内部做过 → Java 侧纯跳过（:397 注释） | 无独立入口（surface 在 A 段 Rust 管线内：`worldgen_handle.rs:824-834`，`FLAG_SKIP_SURFACE` 门控，:825） |
| C | **光照（LIGHT）** | `ServerLightingProviderMixin.wgLightRustTakeover`（`versions/1.20.1/java/.../mixin/ServerLightingProviderMixin.java:421-423`）注入 `ServerLightingProvider.light(Chunk, boolean)` | `@Inject(at=@At("HEAD"), cancellable=true, require=1)` + `cir.setReturnValue(completedFuture(chunk))` | `-Dcoreswap.light.rust`（缺省关 = 完全 vanilla，:63）；失败回退链：lightInit 失败 / 邻 chunk 缺失或 status < FEATURES / 高度参数不符 / rc≠0 / native throw → 不 cancel 走 vanilla（:136-143, 449-482） | JNI `lightInit`（jni_bridge.rs:313）→ `LightEngine::from_json_file`；`lightComputePacked`（jni_bridge.rs:421）/ `lightCompute`（jni_bridge.rs:344）→ `light_decode_packed` + `light_compute`（`worldgen-core/src/light/mod.rs:188, 161`） |
| D | **写回（blocks → Chunk sections + heightmap）** | 无独立 mixin——是 A 段 work 内的 Java 侧后处理 | — | 随 A 段 | Java 侧完成：`CppBridge.writeChunk`（CppBridge.java:651-671）→ `BulkWb.writeSections`（默认，:652-653；`java-core/.../BulkWb.java:163`）或逐块路径（A/B 开关 `-Dcoreswap.bulkwb=0`，:642）；heightmap 一次性补 6 种（:660-666） |

### 1.3 让位段（1.20.1 现行默认形态，非接管）

- **carver / features**：1.20.1 **无对应接管 mixin**（mixins.json 全清单核对）。Rust 侧管线虽有 carver/features 实现（`worldgen_handle.rs:836-838` apply_carvers、:675-677 apply_features），但 stageMask 默认 `0b011` = SKIP_CARVER|SKIP_FEATURES（CppBridge.java:99-107 `resolveStageMask`，:104 缺省 0b011；init/initNether/initEnd 均设入，:115/165/185）⇒ **存档链路这两阶段由 Java vanilla 执行**，Rust 段被 flag 关闭。`rustFeaturesTakeover()` 谓词在 1.20.1 无调用者（CppBridge.java:204-206 注记，消费者 ChunkGeneratorFeaturesMixin 是 1.21.6 独有）。`-Dcoreswap.rust.stages=all`（mask=0）= 双跑对照（CppBridge.java:101）。
- **BIOMES 阶段**：无接管 mixin（mixins.json 无相关注入点）——vanilla 生成 biome sections；Rust 侧 biome（MacroBiome，worldgen_handle.rs:31-49）只在 fill/surface/carver/feature 判定内部消费。
- **存盘序列化**：无 mixin——`SerializingRegionBasedStorage`/nio 序列化完全 vanilla。
- **STRUCTURES/STRUCTURE_STARTS 等结构阶段**：无接管 mixin；结构只经 A' Beardifier 喂入影响 density。

### 1.4 其余注册 mixin = 探针/诊断面（非接管）

`ChunkRandomSeedLogMixin / SquarePlacementModifierMixin / CountPlacementModifierMixin / TrunkPlacerMixin / MegaJungleTrunkPlacerMixin / BeehiveDecoratorMixin / TreeDecoratorGeneratorMixin / BlobFoliagePlacerMixin / BushFoliagePlacerMixin / BiomeSourceLogMixin / DiagFeatureBiomeMixin / BlobProbeMixin / ConfiguredFeatureProbeMixin / ColProfProbeMixin / SurfaceDumpProbeMixin / EstDumpProbeMixin / AquiferDumpProbeMixin / NoiseDumpProbeMixin`（mixins.json:8-25）——各文件头均注明 property 门控（如 `wg.seedlog`、probe 开关），行为=日志/dump，不 cancel 目标方法（各文件 @Inject 无 cancellable）。

---

## 2. 各接管段 5 形态维度记录

### 2.1 A 段：NOISE/fill（populateNoise 接管）

| 维度 | 代码事实 |
|------|---------|
| ① 同步性 | 拦截点**立即返回 pending future**，重活（feedBeardifier + fillChunk + writeChunk 全链）在 work lambda 内异步执行（Mixin:298-327）；调用方（ChunkStatus.NOISE 任务）阻塞在 future 上由 MC 状态机调度。A/B 开关 `-Dcoreswap.syncfill=1` = 内联同步（Mixin:229-231） |
| ② 线程放置 | 三级分派（`wgDispatch`，Mixin:227-253）：默认 **EXEC_MODE**（260911-03 起缺省开，Mixin:135-145）投**自有有界池** `wgOwnPool`（Mixin:174-197）：`ThreadPoolExecutor(N, N, 60s keepAlive, LinkedBlockingQueue(128), "CoreSwap-Fill-N" daemon, CallerRunsPolicy)`，N 缺省 = `logical/2 - 2`（Mixin:151-164），显式 `-Dcoreswap.execpool=N`；队列满 128 背压时 CallerRuns 回退到调用线程内联执行（Mixin:128-131）。`-Dcoreswap.exec=0` 回退 vanilla 共享池 `WG_FILL_POOL = Util.getMainWorkerExecutor()`（Mixin:67-68）+ P1 信号量 `WG_MAX_INFLIGHT`（缺省 logical/2-2，Mixin:86-116；许可在调用线程获取，Mixin:243-252）。Rust 内部再并行：`wg_fill_blocks_multi` 用 **per-call `std::thread::scope`**（非常驻池），线程数 `adaptive_threads` = min(物理核-2, count)，count=1 clamp 到 1（api.rs:24-44, 143, 158-181）；`CORESWAP_THREADS` env 覆盖（api.rs:26-28）。Java 侧传 `THREADS`（服务端 -1 全核/客户端 -2，CppBridge.java:73-95）→ 均 ≤0 → 自适应 |
| ③ 批粒度 | **单 chunk / 次**：Java 每 chunk 调一次 `fillBlocks(h, {cx}, {cz}, {buf}, threads)`（CppBridge.java:459-460，单元素数组）；Rust `count=1` → 单线程 scope（api.rs:38-44 注记：消除线程风暴）。多 chunk 批量 ABI 存在（wg_fill_blocks_multi）但生产 Java 侧不用 |
| ④ 增量 vs 全量 | **全量**：整 chunk 16×16×384（nether 256/end 128）从 density 起整块生成，无跨 chunk 增量；跨 chunk 状态 = aquifer 邻居随机偏移（split 种子，纯函数式重导）+ beardifier 显式喂入（A'）+ terrain_cache（见⑤） |
| ⑤ 缓存层 | (a) Java `STATE_BY_ID` AtomicReferenceArray（id→BlockState，进程级幂等，CppBridge.java:66-68/725-734）；(b) Java per-thread 输出 buffer `BUF/BUF_NETHER/BUF_END` ThreadLocal（CppBridge.java:71-72/504-505/555-556）；(c) Rust `terrain_cache`（ca_min 门控的邻 chunk 地形列缓存，noise+surface+carver 无 feature——**仅 features 实际启用时才启用**，stageMask=3 时只写不读纯成本故跳过，worldgen_handle.rs:136-146/620-626）；(d) Rust `est_l2` OnceLock<Arc<Mutex<EstL2>>>()（aquifer est 二级缓存，worldgen_handle.rs:132-134/597-600）；(e) carver_cache/feature_cache/feature_indexer 创建时预载运行只读（worldgen_handle.rs:117-124/472-484）；(f) WG_METHOD_CACHE/WG_BEARD_FAIL_KEYS 反射缓存（CppBridge.java:317-320） |

### 2.2 B 段：SURFACE（buildSurface 跳过）

| 维度 | 代码事实 |
|------|---------|
| ① | 纯 `ci.cancel()`（Mixin:415），无任何异步动作——同步零工作 |
| ② | 无独立线程动作；surface 实际计算在 A 段 Rust 管线内（worldgen_handle.rs:824-834 `sb.build_surface`，`FLAG_SKIP_SURFACE`/`WG_SKIP_SURFACE` 门控 :825） |
| ③ | 单 chunk（随 A 段 fill 粒度） |
| ④ | 全量（surface rules 每列判定，输入含 est/heightmap/biome，无增量） |
| ⑤ | surface 用 `biome_at_surface` = BiomeAccess jitter 采样（worldgen_handle.rs:817-820）；噪声 key 预加载表（surface_rules.rs ENGINE_NOISE_KEYS，worldgen_handle.rs:160-165） |

### 2.3 C 段：光照（gate `-Dcoreswap.light.rust`，缺省关）

| 维度 | 代码事实 |
|------|---------|
| ① 同步性 | **完全同步**：mixin 内联完成 收集→JNI→写回→setLightOn→releaseLightTicket，最后 `cir.setReturnValue(CompletableFuture.completedFuture(chunk))`（ServerLightingProviderMixin.java:423-501）——**不派发到任何 executor，全部工作在调用 `light()` 的线程上**（vanilla light() 内部有 executor 编排，本接管取而代之；调用线程归属 = ChunkStatus.LIGHT 任务车道 ⚠️ 具体线程名未在本轮核实） |
| ② 线程放置 | 无自有池、无 supplyAsync（全文核对 :421-501）。并发模型 = 依赖 MC 对不同 chunk 的 light() 并行调用（ThreadLocal 缓冲注释「light 线程池每线程一份」:155-163/287-309）。⚠️ Rust 侧 `LightEngine.scratch: RefCell<Scratch>`（light/mod.rs:64）**非线程安全**——多线程并发调 `lightCompute`（同 handle 原始指针，jni_bridge.rs:368）时 `borrow_mut` 无同步保护；正确性当前依赖「每线程独立 handle？」——事实是 handle 全局单例（Mixin:66 静态 lightHandle）⇒ **多 light 线程并发进入同一 RefCell 为未定义行为面**（代码事实记录，未做并发验证 ⚠️） |
| ③ 批粒度 | 单 chunk / 次接管调用；但**每次输入 = 3×3 邻域 9 chunk 方块**（blocks9 884736 int，Mixin:146-147/156-157；packed 路同样 9 chunk 216 节，:268-319），**输出仅中心 chunk**（24 section × 2 通道，:486-490） |
| ④ 增量 vs 全量 | **全量重算**：`light_compute` 对 3×3×384 域整场重算（fill opacity → block BFS → sky 柱状直落+种子 BFS → 导出中心，light/mod.rs:273-431）；文件头声明有损边界「全量重算无增量」（mod.rs:5-6）。**无跨 chunk 光照缓存**——同一邻域 chunk 被中心地位重算 1 次 + 作为邻居参与 8 次（结构性事实，代价判定归 P2） |
| ⑤ 缓存层 | (a) `LightEngine.table`（opacity/emission by raw id，light_data.json 载入后只读，mod.rs:58-65/92-105）；(b) `Scratch`（opacity/block_light/sky_light/queue/col_max，RefCell 跨调用复用，mod.rs:67-74/288-302）；(c) Java ThreadLocal：WG_BLOCKS9_TL(884736)/WG_OUT_*_TL/WG_META_TL/WG_PAL_TL/WG_STO_TL/WG_PBUF_TL/WG_SEC_TL/WG_WORDS_TL（Mixin:91-94/104-109/156-163）；(d) Rust JNI thread_local LIGHT_B9/OB/OS/OF/PM/PP/PS（jni_bridge.rs:294-309） |

### 2.4 D 段：写回（blocks → Chunk）

| 维度 | 代码事实 |
|------|---------|
| ① | 随 A 段 work 异步执行（fillBlocks 返回后同线程立即 writeChunk，CppBridge.java:494-500） |
| ② | 与 A 段同线程（CoreSwap-Fill-N / CallerRuns 时 worldgen 车道） |
| ③ | 单 chunk；写回内部**按 section 批量**（BulkWb.writeSections 逐 section 整段替换 PalettedContainer，BulkWb.java:163/175-225）或逐块（旧路 writeChunkPerBlock 98304 次 setBlockState，CppBridge.java:674-710） |
| ④ | 全量整块写（接管点=populateNoise HEAD cancel ⇒ chunk 全新，CppBridge.java:612-615 等价性论证）；A1a 空气不写（-Dcoreswap.skipair，1.20.1 缺省开，:621）；均质空气 section 短路（BulkWb.java:185-190） |
| ⑤ | BulkWb per-thread TL 状态（scratch/stamp epoch 免清零，BulkWb.java:136-139）；`STATE_BY_ID` 共享（CppBridge.java:725-734，BulkWb 复用，:721-723 注记）；并发哨兵 chunk 级 map（BulkWb.java:76-86） |

### 2.5 让位段（carver/features/biome/序列化）

- 由 vanilla 在 vanilla 自己的调度/线程形态下执行（本项目零改动）；Rust 侧对应实现仅 `-Dcoreswap.rust.stages` 打开后用于双跑对照（CppBridge.java:99-107）。P1b 测 vanilla 形态即可，无需 CoreSwap 侧记录。

---

## 3. 特别记录

### 3.1 光照路径调用链全貌（gate `coreswap.light.rust`）

```
ChunkStatus.LIGHT 任务 → ServerLightingProvider.light(Chunk, boolean)
  └ [HEAD cancel] ServerLightingProviderMixin.wgLightRustTakeover (Mixin:421-423)
      ├ gate: LIGHT_RUST (:63)；wgLightEnsureInit → CppWorldgen.lightInit → JNI (jni_bridge.rs:313)
      │    └ LightEngine::from_json_file(light_data.json) (light/mod.rs:78-118)
      ├ 收集（候选 C packed 优先，:439-441）:
      │   wgLightCollectPacked (:254-322)：3×3 邻 × 24 节，每非空节
      │     sec.getBlockStateContainer().writePacket(pbuf) (:286) —— palette 展开**在 Java 侧拆帧**：
      │     bits 字节 + VarInt palette + VarInt n + longs；产出 meta[432]/pal/sto 三数组
      │   邻 chunk 提取 wgLightNeighbor (:231-246)：ChunkProvider.getChunk，status ≥ FEATURES 才用
      ├ JNI packed 直传: CppWorldgen.lightComputePacked(meta, pal, sto, palLen, stoLen, outB, outS, outF)
      │   (Mixin:445-446) → jni_bridge.rs:421-511 → light_decode_packed (light/mod.rs:188-258)
      │   （Rust 侧 LSB-first 解码到 LIGHT_B9，游标闭合校验 :254-256）
      │   → light_compute (mod.rs:161) —— 3×3 邻域全量重算发生于此（mod.rs:273-431），
      │     export_center 只导中心 16..32 (mod.rs:423-424/502-538)
      ├ 回退链: packed rc≠0 → 同 chunk blocks9 ABI 重算 (Mixin:447-454, collect :166-227 +
      │   lightCompute jni_bridge.rs:344-413)；再败 → 不 cancel 走 vanilla (:456-458/480-482)
      │   blocks9 快路径 = Java 侧 PackedIntegerArray 位流解码 (wgLightFillSectionFast :361-419)，
      │   旧路 = 逐格 getBlockState (:326-342)；开关 -Dcoreswap.light.oldcollect / blockabi (:90/103)
      └ 写回: enqueueSectionData(BLOCK/SKY × 24 节, nibble 2048B/节, Mixin:485-490 + wgLightNibble :505-511)
         → chunk.setLightOn(true) + releaseLightTicket (:493-494) → completedFuture (:500)
```

要点：**palette 展开不发生在 Rust**——packed ABI 的 writePacket 帧由 Java 拆成 meta/pal/sto（pal 已是 ABI 编码 rawId|lum<<24，Mixin:295/346-352），Rust 只做位流→blocks9 解码（light_decode_packed）。3×3 重算位置 = `light_compute_inner`（mod.rs:273），输入域 48×48×384。

### 3.2 executor/线程池定义点汇总

| 池/线程 | 定义点 | 池宽 | 队列/策略 |
|---|---|---|---|
| CoreSwap 自有 fill 池 `wgOwnPool`（默认启用） | NoiseChunkGeneratorMixin.java:174-197 | N=N=缺省 `logical/2-2`（:151-164），`-Dcoreswap.execpool` 覆盖 | LinkedBlockingQueue(128)，CallerRunsPolicy，daemon "CoreSwap-Fill-N"，allowCoreThreadTimeOut(true) |
| vanilla 共享池（exec=0 回退臂） | Mixin:67-68 = `Util.getMainWorkerExecutor()`（vanilla Util.java:229） | vanilla 定义（同池还载 ChunkBuilder 网格构建，Mixin:118-121 注记） | vanilla 定义 |
| P1 in-flight 信号量（shared 池臂） | Mixin:102-103/86-116 | `WG_MAX_INFLIGHT` 缺省 logical/2-2，`-Dcoreswap.maxinflight`；≤0 不限 | Semaphore，调用线程取/whenComplete 释（:243-252） |
| Rust fill 并行（per-call scope） | api.rs:158-181 + adaptive_threads api.rs:24-44 | min(物理核-2, count)；`CORESWAP_THREADS` env / Java THREADS 参数（服务端 -1/客户端 -2 → 自适应）；count=1 → 1 | std::thread::scope 每调用重建回收，无队列 |
| 光照 | **无自有池**（同步内联，见 §2.3①） | — | — |

### 3.3 ⚠️ 未验证项清单

1. ChunkStatus.LIGHT 任务的调用线程归属（worldgen worker 车道具体线程名/池）——本轮只核了 CoreSwap 侧代码，vanilla 调度侧归 P1b。
2. `LightEngine.scratch` RefCell 在多 light 线程并发下的安全性（§2.3②）——静态事实成立（单静态 handle + 非同步 RefCell），未做运行时验证；若 MC 实际串行调 light() 则不触发。
3. BulkWb 帧构造细节（Provider/strategy 反射取用）只读了注释与签名，逐行核对归 P2（如需）。
4. nether/end 的 writeChunk 高度参数（256/128）与 Rust `blocks.len()` 匹配性——api.rs:174-176 有防越界注记，本轮未追 fill_chunk_blocks 的返回长度保证。
