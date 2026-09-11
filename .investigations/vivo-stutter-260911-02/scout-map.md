# 集成服卡顿可归因面 — scout 勘探地图（vivo-stutter-260911-02）

> 角色：re-code scout（只读，无 shell）。产物 = 勘探地图，**不做结论性判断**（解读归 worker）。
> 载体：Fabric 1.20.1 **集成服（单机客户端）**，CoreSwap 1.0.28（dll `dd3b645f…`，stageMask=3），VD32/模拟距离 32，旁观高速飞行 ~2 min，症状 = **持续低帧**（非周期顿挫）。同机 pure vanilla 同 VD32 不卡（用户实测）。
> 证据口径：全部 file:line；仓库相对路径。**1.20.1 官方反编译源码在库**（`versions/1.20.1/data/mc_src_extract/`）⇒ 本图涉及的 vanilla 侧事实**不是**「需外部资料」。
> 纪律声明：标注 `[未验证]` = 静态阅读得到的假设/估算，未实测；`[需外部资料]` = 库内无证据。量级数字均为**估算**并给出算法，禁当实测。

---

## 0. 一句话地图

`populateNoise` HEAD 拦截（worldgen mailbox 线程）→ `supplyAsync` 到 **`Util.getMainWorkerExecutor()`**（= 客户端区块网格构建**同一个 ForkJoinPool**）→ `feedBeardifier`（反射 + 全局 RwLock 写）→ `fillBlocks` JNI（每 chunk 新建 384 KiB Rust 缓冲 + 拷回）→ Rust 侧 `thread::scope` **每 chunk 新建 (物理核-2) 个 OS 线程**（count=1 不 clamp）→ `fill_chunk_blocks`（含 384 KiB `col.data().to_vec()`）→ Java `writeChunk` **98304 次逐块加锁写**（含空气）→ 6 张高度图全量重扫 → future 返回。

---

## 1. 执行体链路清单（逐站 + 线程）

| # | 站 | 文件:line | 线程 | 证据/说明 |
|---|---|---|---|---|
| 1 | `populateNoise` HEAD `@Inject` cancel | `versions/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:118-128` | **worldgen lane**（`TaskExecutor.create(workerExecutor,"worldgen")` mailbox，`mc_src_extract/…/ThreadedAnvilChunkStorage.java:184`） | 拦截判定 `:133-140`；每 chunk 调 `wgSettingsId()`（`:101-106`，产生 String） |
| 2 | 分派（R3 异步） | `NoiseChunkGeneratorMixin.java:92-98`（`wgDispatch`）、池定义 `:67-68` | 提交侧 = 1 的线程；执行侧 = **共享池 `Worker-Main-N`** | `WG_FILL_POOL = Util.getMainWorkerExecutor()`；回退 `-Dcoreswap.syncfill=1` → 内联（`:71,94-96`） |
| 3 | 共享池身份 | `mc_src_extract/…/util/Util.java:89,182-183,229-230` | `ForkJoinPool(availableProcessors-1)` | `MinecraftServer.java:316` `workerExecutor = getMainWorkerExecutor()` → `ServerChunkManager.java:96` → `ThreadedAnvilChunkStorage.java:184`（worldgen mailbox **也**建在同一池上） |
| 4 | Beardifier 喂入 | `wg/bench/CppBridge.java:251-252 → 296-354`（构造 `:300-301`） | 共享池线程（同 2） | 每 chunk 都调（含无结构 chunk，走 `:336` 清空分支） |
| 5 | JNI 调用（Java 侧） | `wg/bench/CppBridge.java:391-422`（`:398-399` 调用点）；`wg/CppWorldgen.java:71` | 共享池线程（同步 JNI，无线程切换） | `int[]` 传入 / `int[][]{buf}` 传回；`THREADS` 定义 `CppBridge.java:40-62` |
| 6 | JNI 桥（Rust，dll 实际执行体） | `versions/1.20.1/rust/src/jni_bridge.rs:179-227` | 共享池线程 | 本地 buffer 分配 `:203-206`；拷回 Java `:219-223` |
| 7 | Rust 入口 + 并行分块 | `worldgen-core/src/api.rs:131-171`；`adaptive_threads` `:24-41` | **每 chunk 新建 `nthreads` 个 scoped OS 线程**（`std::thread::scope` `:146-152`） | count=1 时 `nthreads = max(1, 物理核-2)`，**不 clamp**（`:38-40`）；`outs[i].write` `:164` |
| 8 | 区块生成主体 | `worldgen-core/src/worldgen_handle.rs:613-683`（`fill_chunk_blocks`）、`:688-842`（`fill_terrain_column`） | 上述 scoped 线程（其中 1 个真正干活） | 宏观 `terrain.rs:256-301`；surface `surface_rules.rs:1202+`；carver/features 被 stageMask=3 跳过（`:837`、`:675`） |
| 9 | 写回 Java | `wg/bench/CppBridge.java:518-559`（`writeChunk`） | 共享池线程（= 调用 5 的线程） | `:546` 逐块 `ChunkSection.setBlockState`；`:552-558` 6 张高度图 |
| 10 | 锁语义落地 | `mc_src_extract/…/world/chunk/ChunkSection.java:56-66` → `PalettedContainer.java:141-152` → `util/thread/LockHelper.java:31-71` | 同 9 | 3 参 `setBlockState` ⇒ `lock=true` ⇒ `swap` ⇒ `LockHelper.lock/unlock`（ReentrantLock + Semaphore 各一次） |
| 11 | future 交回 MC 状态机 | `NoiseChunkGeneratorMixin.java:172`（`cir.setReturnValue(wgDispatch(work))`） | worldgen lane 收 future | 与 vanilla `NoiseChunkGenerator.java:348-355` 同形 |
| 12 | 初始化（非热路径） | `wg/bench/BenchMod.java:15-45`、`CppBridge.java:76-99` | **server thread**（`SERVER_STARTED`） | 句柄/dll 自证在启动期 |

**线程面小结**：vivo 热路径上只有两类线程 —— ① worldgen mailbox（底层仍是共享池）；② 共享池 `Worker-Main-N`；**额外**每 chunk 由 Rust 新建 (物理核-2) 个短命 OS 线程。JNI 是同步调用，不引入第三类常驻线程。

---

## 2. 每 chunk 稳态开销盘点（重点）

> 单位：1 chunk = 16×16×384 = 98304 格。384 KiB = 98304×4 B（`int`）。量级均为**估算**，标注算法。

### 2.1 JNI 边界 / 内存搬运

| # | 项 | 证据 | 量级（估算） |
|---|---|---|---|
| J1 | JNI 传参：`int[]{cx}`、`int[]{cz}`、`int[][]{buf}` 每 chunk 新建 | `CppBridge.java:398-399` | 3 个小对象（可忽略） |
| J2 | **传的是 `int[]` 不是 `byte[]`，也不是直接缓冲**；大小 = 16×16×384 `int` = **393,216 B = 384 KiB**（~393 KB 成立） | `CppWorldgen.java:67-71`（注释与签名）、`CppBridge.java:37-39`（ThreadLocal buffer） | 384 KiB/次，**Java 侧 buffer 复用**（per-thread） |
| J3 | Rust JNI 侧**每 chunk 新建** `Vec<Vec<i32>>` 并全零 | `jni_bridge.rs:203-206` | 384 KiB 分配 + 清零，调用后释放（allocator churn） |
| J4 | Rust → 本地 buffer 拷贝（`SendOut::write` = `copy_nonoverlapping`） | `api.rs:51-53,164` | 384 KiB memcpy |
| J5 | Rust → Java 数组拷回（`set_int_array_region`） | `jni_bridge.rs:219-223` | 384 KiB memcpy |
| J6 | Rust `col.data().to_vec()` 整列拷贝（每次 fill 末尾） | `worldgen_handle.rs:682` | 384 KiB 分配 + memcpy |
| J7 | `BlockColumn::new` = `vec![AIR; 98304]`（`BlockId=i32`） | `blocks.rs:9,162-167` | 384 KiB 分配 + 填值 |
| J8 | `ChunkData.blocks` = `vec![BlockKind::Air; 98304]`（`BlockKind` 4 变体 = 1 B） | `terrain.rs:185,264` | 96 KiB 分配 + 填值 |
| J9 | 宏观 slices（`sample_chunk` → `build_slices`） | `terrain.rs:111-114,328-331` | `5ch × 5×49×5` f64 ≈ **49 KiB**/chunk |
| J10 | biome 字符串：`ChunkData.biome` 256 列 + `biome.biome()` 每列返回 `String` | `terrain.rs:265,297`；`surface_rules.rs:1252-1263`（cache 命中仍 `.clone()`） | 256+ 次 String 分配/chunk（中量级） |
| — | **小计（估）** | — | **≈1.5 MiB memcpy + ≈1.2 MiB 分配/清零，每 chunk** |

### 2.2 Java 侧写回（锁与写放大）

| # | 项 | 证据 | 量级（估算） |
|---|---|---|---|
| W1 | **逐块写 98304 次**，且**含空气**（无 `!= AIR` 短路） | `CppBridge.java:522-548`（三重循环） | 98304 次/ chunk |
| W2 | 3 参 `setBlockState` ⇒ `lock=true` ⇒ `swap` ⇒ `LockHelper.lock()+unlock()`（ReentrantLock.lock/unlock + Semaphore.tryAcquire/release） | `ChunkSection.java:56-66`；`PalettedContainer.java:141-152`；`LockHelper.java:31-71` | **98304 × 2 把锁操作**/chunk；单次数十 ns 级 ⇒ **量级 2–10 ms/chunk 的锁开销（估算，需实测）** |
| W3 | 每块 `STATE_BY_ID.get(id)`（`AtomicReferenceArray` volatile 读）+ 范围检查 | `CppBridge.java:529-531`（缓存数组定义 `:33-35`） | 98304 次 volatile 读/chunk |
| W4 | `setBlockState` 内部计数/谓词：`nonEmptyBlockCount`、`nonEmptyFluidCount`、`isAir()`、`getFluidState()`、`hasRandomTicks()` | `ChunkSection.java:68-92` | 98304 次多分支/chunk |
| W5 | **6 张高度图全量重扫**（每列自 `highestNonEmptySectionYOffset+16` 下扫至首个非空气） | `CppBridge.java:552-558` → `Heightmap.java:37-71`（`getBlockState` 读 `:52`） | 6 类型 × 256 列；最坏 ~300+ 次读/列 ⇒ **~10⁵ 级 getBlockState（估算）** |
| W6 | 诊断用全 buffer 非零扫描（`nz`），恒执行 | `CppBridge.java:409-410` | 98304 次读/chunk（门控只关 println，**扫描不关**——注释 `:362-368` 自陈） |
| W7 | `chunk.getSection(0..24)` 数组化（一次性） | `CppBridge.java:520-521` | 24 次，可忽略 |

### 2.3 Rust 热路径的 env / 诊断

| # | 项 | 证据 | 说明 |
|---|---|---|---|
| R1 | **每 chunk 多次 `std::env::var`**：`WG_CA_MIN`、`WG_SKIP_FEATURES`、`WG_EST_L2`、`WG_SKIP_AQUIFER`、`WG_SKIP_OREVEIN`、`WG_EST_SHARED`、`WG_EST_DUMP`、`WG_SKIP_SURFACE`、`WG_SKIP_CARVER`、`CORESWAP_THREADS`、`WG_DEBUG` | `worldgen_handle.rs:619,625,707,711,733,766,790,825,837`；`api.rs:26,154` | 每次 = getenv + `String` 分配；合计 ~10 次/chunk（`WG_FEATURELOG` `:677` 因 skip_features 不触发） |
| R2 | `CONF_ECHOED` static `AtomicBool::swap` 每 chunk | `worldgen_handle.rs:652-666` | 全局共享原子 RMW（跨线程同 cache line） |
| R3 | 全局 `RwLock` 独占写 + HashMap insert 每 chunk | `worldgen_handle.rs:536`（`set_beardifier`） | **跨全部生成线程的串行点**；且**未见 per-chunk 移除**（消费侧 `:715` 只 `get`）⇒ 条目随探索单调增长 `[未验证：是否别处清理]` |
| R4 | 全局 `Arc<Mutex<EstL2>>` 每 chunk 4 角 est（miss 时 get+put 各一次锁） | `aquifer.rs:538-547,559-563`；句柄 `worldgen_handle.rs:597-602,711-713` | 跨线程全局 mutex；~8 次锁/chunk |
| R5 | `terrain_cache.lock()` 每 chunk | `worldgen_handle.rs:626-644` | **stageMask=3（skip_features=1）时该分支不进入**（门控 `:626` 注释 `:620-623` 明确）⇒ vivo 下**无此开销**（防误判项） |
| R6 | `beardifiers.read()` 每 chunk | `worldgen_handle.rs:715` | 全局读锁（与 R3 写锁同锁） |

### 2.4 线程创建（形态面，非分配面）

| # | 项 | 证据 | 说明 |
|---|---|---|---|
| T1 | `std::thread::scope` + `nthreads` 次 `s.spawn` | `api.rs:146-152` | **每 chunk 新建并 join `nthreads` 个 OS 线程** |
| T2 | `nthreads = adaptive_threads(THREADS, 1)`；count=1 **不 clamp** | `api.rs:24-41`（`:38-40` 注释）、调用点 `CppBridge.java:398-399`（count 恒 1） | 本机 24 逻辑核 ⇒ `physical=12` ⇒ **10 个线程/chunk，其中 9 个空转退出** `[未验证：用户机核数未知]` |
| T3 | `THREADS` 在客户端环境取 `-2`（模式自适应） | `CppBridge.java:40-62`（`resolveThreads`，Fabric `CLIENT` → `-2`） | 集成服 = CLIENT 环境 ⇒ 走自适应分支 |
| T4 | **注释与实现不一致（候选）**：`api.rs:38-39` 称「池按请求线程数建 worker 并保持」，但实现是 scoped 线程（无持久池） | `api.rs:38-40` vs `:146-152`；全库未见 Rust 侧 ThreadPool（`grep thread::scope\|ThreadPool` 仅命中 `api.rs` + bins） | `[未验证]` 是否有别处池实现；若确无 ⇒ 该 clamp 逻辑是 C++ 池模型遗留 |

### 2.5 Java 侧其它每 chunk 开销

| # | 项 | 证据 | 说明 |
|---|---|---|---|
| P1 | `wgSettingsId()` 每 chunk 构造 String（`Identifier.toString`） | `NoiseChunkGeneratorMixin.java:101-106`（调用点 `:138,140,177,253-257`） | populateNoise 1–2 次 + buildSurface 1 次 |
| P2 | `WgDiag.setCurrent` 每 chunk + `Boolean.getBoolean("wg.seedlog")`（`System.getProperty` = 同步 Hashtable 读）每 `setPopulationSeed`/`setDecoratorSeed` | `ChunkRandomSeedLogMixin.java:21-36`；`WgDiag.java:50-54` | `setDecoratorSeed` 每 feature 一次 ⇒ 特征密集 chunk 下高频 `System.getProperty`（跨线程同步点）`[未验证 量级]` |
| P3 | 特征诊断 mixin（TrunkPlacer/BlobFoliage/Bush/Count/Square/Beehive/TreeDecorator/BiomeSource/DiagFeatureBiome）均在 mixin 表内 | `resources/coreswap.mixins.json:6-30` | 打点前有 `WgDiag.curAllowed()/posAllowed()` 门（`WgDiag.java:62-72`，目标未设时立即 `return true`）+ env 门；**具体每调用点成本未逐行核** `[未验证]` |
| P4 | `ChunkTiming`（`-Dcoreswap.chunktime=1`）/`StallWatch`（`-Dcoreswap.stallwatch`）/`MIXLOG` | `ChunkTiming.java:19,37-62`；`StallWatch.java:20-29`；`CppBridge.java:368` | **默认全关**（static final / 一次 prop 读）⇒ vivo 默认零成本 |
| P5 | 光照 Rust 接管 | `mixin/ServerLightingProviderMixin.java:62` | 门控 `-Dcoreswap.light.rust`，**默认关** ⇒ vivo 默认 vanilla 光照 |

---

## 3. 与 vanilla 同阶段（NOISE fill）的形态差

对照源：`mc_src_extract/…/world/gen/chunk/NoiseChunkGenerator.java:329-357`（`populateNoise`）与 `:359-435`（fill 主体）。

| 面 | vanilla 1.20.1 | CoreSwap | 证据 |
|---|---|---|---|
| 池 | `CompletableFuture.supplyAsync(..., Util.getMainWorkerExecutor())` | 同（R3 已对齐） | vanilla `:348-349`；CoreSwap `NoiseChunkGeneratorMixin.java:67-68,97` |
| **外层 section 锁** | fill 前对全部 section `lock()`，完成回调里 `unlock()` | **不加外层锁**（声明差异，理由=自锁死） | vanilla `:342-346,351-355`；CoreSwap `:82-91` + 错误台账 E1 |
| **逐块写锁** | `setBlockState(..., false)` ⇒ `swapUnsafe`，**零锁** | 3 参 ⇒ `swap` ⇒ **逐块 lock/unlock** | vanilla `:416`；CoreSwap `CppBridge.java:546` → `PalettedContainer.java:141-152` |
| **空气短路** | `if (blockState != AIR && !isOutsideGenerationArea)` 才写 + trackUpdate | **全量 98304 写**（含空气） | vanilla `:415-423`；CoreSwap `CppBridge.java:522-548` |
| JNI | 无 | **2 次跨界/chunk**（`setBeardifier` + `fillBlocks`） | `CppBridge.java:342,398` |
| 中间缓冲 | 无（直接写 section，逐 cell 生成即写） | ≥4 处 384 KiB 级分配/拷贝（J3–J7） | §2.1 |
| 高度图 | **增量** `trackUpdate` 2 张（OCEAN_FLOOR_WG / WORLD_SURFACE_WG） | 事后 **6 张全量重扫** | vanilla `:417-418`；CoreSwap `CppBridge.java:552-558` |
| 线程创建 | 无（池内任务，无线程创建） | 每 chunk 新建 `nthreads` 个 scoped 线程 | `api.rs:146-152` |
| 生成体 | 纯 Java（density/aquifer/carver 全在 Java） | Rust（terrain+surface）+ Java（carver/features，stageMask=3） | `worldgen_handle.rs:613-683`；`api.rs:112-117` |
| 全局共享锁 | 无（per-chunk 对象） | `beardifiers` RwLock、`EstL2` Mutex 每 chunk 触碰 | §2.3 R3/R4 |

**要点（供 worker 解读，不作结论）**：池本身**两者同池** ⇒ 「池争用」不是差异；可辨差异集中在 ①**锁面**（逐块 vs 无锁）②**写放大**（98304 含空气 vs 非空气短路）③**分配/拷贝率**（≥1.5 MiB/chunk + Rust allocator churn）④**线程创建率**（每 chunk ~10 个）⑤**高度图重扫**（6 张 vs 增量 2 张）。

---

## 4. 客户端侧消费者清单（是否与 worldgen 共池）

| 消费者 | 执行器/线程 | 证据 | 与 worldgen 共池？ |
|---|---|---|---|
| **客户端区块网格构建 `ChunkBuilder`** | `Util.getMainWorkerExecutor()` | `mc_src_extract/net/minecraft/client/render/WorldRenderer.java:719`（`new ChunkBuilder(..., Util.getMainWorkerExecutor(), ...)`） | **是（同一 ForkJoinPool）** |
| 网格任务提交 | `CompletableFuture.supplyAsync(task, this.executor)` | `client/render/chunk/ChunkBuilder.java:129`（并发受 `threadBuffers` 数量限制 `:123-128`） | 是 |
| 网格上传 / 排序 | **渲染线程**（render loop 内 `upload()`，`scheduleRebuild` 走 mailbox） | `WorldRenderer.java:2225-2232`（`getProfiler().swap("upload")`）；`ChunkBuilder.java:135-145`（`mailbox.send`） | 否（渲染线程不在池内） |
| worldgen mailbox（ChunkStatus 任务） | `TaskExecutor.create(workerExecutor,"worldgen")` | `server/world/ThreadedAnvilChunkStorage.java:184,643`；底层池 = `MinecraftServer.java:316` | 是（mailbox 底层同池） |
| 区块 NBT 读（异步） | `getMainWorkerExecutor()` | `ThreadedAnvilChunkStorage.java:966` | 是 |
| 区块存盘 IO | `getMainWorkerExecutor()` | `world/level/storage/StorageIoWorker.java:119` | 是 |
| 全量光照更新任务 | `getMainWorkerExecutor()` | `WorldRenderer.java:870` | 是 |
| 资源重载 / 皮肤 / 声音等 | `getMainWorkerExecutor()` | `client/MinecraftClient.java:654,946`；`client/texture/AsyncTexture.java:40` 等 | 是（非稳态） |
| 客户端区块数据来源（集成服 packet / ClientChunkManager 反序列化线程） | 未核对 | `[未核对]`：库内有 client 包源码，但本轮未逐行读该路径 | 未定 |

**池规模**：`ForkJoinPool(clamp(availableProcessors-1, 1, max.bg.threads))`，`Util.java:182-183,208-219`；`max.bg.threads` 未设时上限 255。
**结论性判据（供 worker）**：客户端网格构建与 worldgen fill **确为同一池**；vanilla 亦然 ⇒ 「共池」不构成差异，差异只能来自**任务占用形态**（时长/分配率/线程创建/锁面）。

`[需外部资料]` 项（本图内仅此一条）：集成服下客户端区块**反序列化**到 mesh 触发的具体线程链（`ClientChunkManager`/`WorldChunk` 装载路径）——库内源码可查但本轮未读，**不是缺资料**；若需补，路径 = `versions/1.20.1/data/mc_src_extract/net/minecraft/client/world/ClientChunkManager.java`（无需 gradle 缓存）。

---

## 5. 候选归因面（≤5 类）+ 判别手段

> 全部为**未验证候选**，仅列「若成立应观察到什么」+ 最廉价判别。判据排序不构成结论。

### C1 线程创建爆炸（每 chunk ~10 个 scoped 线程）`[未验证]`
- 机制：`api.rs:38-40` count=1 不 clamp + `:146-152` scoped spawn ⇒ 每 chunk 新建 (物理核-2) 个 OS 线程，9 个空转；与渲染线程/池线程抢 CPU。
- 若成立应观察到：进程线程数持续抖动（~10×池大小）；CPU 时间大量花在内核（线程创建/销毁）；`spark` 里 `Worker-Main-*` 空闲而 CPU 占用高；`-Dcoreswap.syncfill=1`（重活回 worldgen 车道）**仍低帧但形态变化**。
- 最廉价判别：**代码阅读已定位**（`api.rs:38-40,146-152`）→ 下一步 = 主会话跑 `WG_DEBUG`/线程计数探针（或 spark thread 视图）；改 `CORESWAP_THREADS=1` 环境变量跑一臂（`api.rs:26-28` 显式覆盖 ⇒ 只 1 个线程/chunk）——**这是最廉价的单变量实验**。

### C2 逐块加锁写回（98304 × lock/unlock）`[未验证]`
- 机制：`CppBridge.java:546` → `ChunkSection.java:56-66` → `PalettedContainer.java:141-152` → `LockHelper.java:31-71`；vanilla 用 `lock=false` + 空气短路。
- 若成立应观察到：`Worker-Main-*` 在 `LockHelper.lock`/`AQS` 上采样占比高；每 chunk 墙钟随「写回段」线性增长；改 `swapUnsafe`（去掉逐块锁）后显著改善。
- 最廉价判别：`-Dcoreswap.chunktime=1` 得 mixin 段墙钟（`ChunkTiming.java`），配合 spark 火焰图看 `LockHelper`/`ChunkSection.setBlockState` 自耗时；或代码侧临时改 4 参 `setBlockState(...,false)` 做 A/B（**改代码，非用户侧**）。

### C3 分配/拷贝率（≥1.5 MiB memcpy + ~1.2 MiB alloc churn/chunk，跨 JNI 两次 384 KiB）`[未验证]`
- 机制：§2.1 J2–J7；Rust `Vec<Vec<i32>>` 每 chunk 新建（`jni_bridge.rs:205`）+ `col.data().to_vec()`（`worldgen_handle.rs:682`）。
- 若成立应观察到：spark 中 allocator/`copy_nonoverlapping`/`set_int_array_region` 占比可观；GC 压力不高（Rust 侧分配不进 Java 堆）但 CPU 内存带宽升高；VD32 下随并发放大。
- 最廉价判别：spark 火焰图看 `CppBridge.fillChunk`/`jni_bridge` 栈占比；或临时把 JNI 本地缓冲改 thread_local 复用（1.21.6 光照侧已有先例 `jni_bridge.rs:290-299`）做 A/B。

### C4 池占用形态（任务时长份额挤占客户端网格构建）`[未验证]`
- 机制：`WorldRenderer.java:719` 与 `NoiseChunkGeneratorMixin.java:67-68` 同池；若 CoreSwap 每 chunk 任务时长显著长于 vanilla 同阶段，则网格构建排队。
- 若成立应观察到：FPS 与「生成负载」正相关；**臂 B（沿已生成路线重飞）FPS 恢复**（`user-ab-protocol.md` 臂 B/E 即此判据）；`[CHUNKTIME]` 的 `inflight max` 高（异步臂预期 20+）。
- 最廉价判别：用户侧臂 B/E（同世界重飞 / VD12）——**已有协议，零成本**；辅以 `-Dcoreswap.syncfill=1`（臂 C）对比形态。

### C5 诊断热路径（env 查询 / 每 chunk String / `System.getProperty` 同步读）`[未验证，量级存疑]`
- 机制：§2.3 R1/R2 + §2.5 P1/P2。
- 若成立应观察到：Rust 侧 `getenv` 在采样中可见；`System.getProperty` 出现跨线程同步等待；但**单次成本低，需先估算总占比再投入**（本 scout 判：**优先级最低**，除非 spark 显示 `getenv`/`Properties.getProperty` 显著）。
- 最廉价判别：spark 火焰图；或把 R1 的 env 读取提到 chunk 级一次（代码改动）做 A/B。

### 备选面（未列前 5，供 worker 取舍）
- **C6 6 张高度图全量重扫**（§2.2 W5）：若成立 = `getBlockState` 在火焰图占比高；判别 = 代码 A/B（只填 2 张 vs 6 张）。
- **C7 `beardifiers` 全局 RwLock 串行 + 无界增长**（§2.3 R3）：判别 = 打印 map 大小随探索增长；锁争用在 spark 中看 `RwLock`。
- **C8 反射喂 Beardifier**（`CppBridge.java:296-354`）：每 chunk `StructureWeightSampler` 构造 + 反射；判别 = 火焰图 `CppBridge.feedBeardifier`。

---

## 6. 未验证 / 需外部资料 / 交接注记

**未验证假设（禁止当事实继承）**
1. 用户机核数未知 ⇒ C1 的「每 chunk 10 线程」是按本机 24 逻辑核外推；真实值 = `max(1, availableProcessors/2 - 2)`。
2. `[未验证]` Rust 侧是否另有持久线程池（我只做了 `thread::scope|ThreadPool` 检索，命中 `api.rs:146-152` 与 bins）；`api.rs:38-39` 注释与实现的矛盾**未消解**。
3. `[未验证]` 各量级估算（锁开销 2–10 ms、memcpy 总量）均**未实测**，方法已在表内给出。
4. `[未核对]` 客户端区块反序列化→mesh 触发的线程链；`[未逐行核]` 特征诊断 mixin 在 vivo（stageMask=3，Java 跑 features）下的每调用点成本。
5. `[未验证]` `beardifiers` 是否有别处 per-chunk 清理（本轮只见 `clear_beardifier` 全清，`worldgen_handle.rs:539-541`）。

**关键「防误判」注记（供 worker 省一轮）**
- `terrain_cache` 路径在 **stageMask=3 下不进入**（`worldgen_handle.rs:620-626`）⇒ 不要把 260910-08 的「ca_min 缓存 ~393 KB col clone/chunk」当作 vivo 现役开销。
- 光照 Rust 接管、`ChunkTiming`、`StallWatch`、`MIXLOG`、各 probe mixin **默认全关**（§2.5 P4/P5）⇒ vivo 默认不构成开销。
- 「池争用」本身不是差异（vanilla 同池，`NoiseChunkGenerator.java:349`）。

**建议的下一步（scout 视角，不构成结论）**
- 最廉价三连（用户侧，已有协议）：臂 B/E（需求对照）→ 臂 C（`-Dcoreswap.syncfill=1` + `chunktime`）→ spark 火焰图。
- 主会话侧最廉价单变量：`CORESWAP_THREADS=1`（`api.rs:26-28` 显式覆盖，无需改代码）验 C1。
- 若需代码侧 A/B：`writeChunk` 改 4 参 `setBlockState(...,false)`（验 C2）、JNI 本地缓冲 thread_local 化（验 C3）——**均为代码改动，须走 worker + 主会话编译闭环**。

---

## 附：证据索引（file:line 速查）

**Java（mod）**
- `versions/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:67-68,92-98,101-106,118-128,133-140,172`
- `versions/1.20.1/java/src/main/java/wg/bench/CppBridge.java:33-39,40-62,296-354,362-368,391-422,518-559`
- `versions/1.20.1/java/src/main/java/wg/CppWorldgen.java:67-71`
- `versions/1.20.1/java/src/main/java/wg/bench/ChunkTiming.java:19,37-62`；`StallWatch.java:20-29`
- `versions/1.20.1/java/src/main/java/wg/bench/mixin/ChunkRandomSeedLogMixin.java:21-36`；`WgDiag.java:50-54,62-72`
- `versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:62`
- `versions/1.20.1/java/src/main/resources/coreswap.mixins.json:6-30`

**Rust（dll 执行体）**
- `versions/1.20.1/rust/src/jni_bridge.rs:179-227,290-299`
- `worldgen-core/src/api.rs:24-41,51-53,131-171`
- `worldgen-core/src/worldgen_handle.rs:514-541,597-602,613-683,688-842`
- `worldgen-core/src/terrain.rs:111-114,185,256-301,328-331`
- `worldgen-core/src/blocks.rs:9,155-183`
- `worldgen-core/src/aquifer.rs:538-563`
- `worldgen-core/src/surface_rules.rs:1202-1263`

**vanilla 1.20.1（库内反编译源码，非外部资料）**
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:329-357,359-435`
- `.../world/chunk/ChunkSection.java:56-92`；`.../world/chunk/PalettedContainer.java:141-152`；`.../util/thread/LockHelper.java:31-71`
- `.../world/Heightmap.java:37-71`；`.../world/chunk/Chunk.java:179-199`
- `.../util/Util.java:89,182-183,208-219,229-230`
- `.../server/MinecraftServer.java:316`；`.../server/world/ServerChunkManager.java:76,91-97`；`.../server/world/ThreadedAnvilChunkStorage.java:184,643,966`
- `.../client/render/WorldRenderer.java:719,870,2225-2232`；`.../client/render/chunk/ChunkBuilder.java:123-150`
