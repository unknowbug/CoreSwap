# 1.21.6 实机每 chunk 通路成本图 + 结构开销清单

> scout 勘探产物（只读、静态推导）。role = recode.scout，产物只落 `.investigations/`。
> 目标：把「一个 chunk 从 Java Worker-Main 进入 → mixin 拦截 → JNI → Rust → 写回 → 后续 Java 阶段」逐环节摊开，标出每项**是否也被串行 bench 支付**、**是否随线程数放大**，作为 5.15× 回归的判据表。

---

## 0. 范围与口径声明

**被分析配置（实机 coreswap 臂）**

| 项 | 值 | 来源 |
|---|---|---|
| MC 版本 | 1.21.6（Fabric，Chunky 驱动专用服） | 任务书 |
| seed / region | `417950215108767439` / center chunk(-48,-11)、chunkradius=56 → 4225 chunks | 任务书（已核实事实） |
| 启动参数 | `-PcppReplace -PcppLib=<worldgen1216.dll> -PcppWorldgenDir=versions\1.21.6\data\worldgen` | 任务书 |
| stageMask | `3` = bit0 SKIP_CARVER + bit1 SKIP_FEATURES → **Rust 只做 NOISE+SURFACE**，carver/features/structure/light 由 Java vanilla 执行 | `[CppBridge] init ... stageMask=3` |
| 实测 | vanilla 52s vs coreswap 268s（4225 chunks）→ 5.15×；23 个 Worker-Main 线程活跃；吞吐降至 15.8 cps（单 chunk 墙钟 ≈1.5s） | 任务书 |

**Rust 侧「未设 env 时的生效默认值」**（由代码推导，非实测——见 ⚠️@idk-13）

| env | 默认 | 定位 |
|---|---|---|
| `WG_CA_MIN`（c-A 邻 chunk 地形缓存 + 全局 Mutex） | **开**（未设 → true） | `worldgen_handle.rs:619` |
| `WG_EST_L2`（跨 chunk est 全局 Mutex 缓存） | **开** | `worldgen_handle.rs:706` + `env_enabled()` `:24-29` |
| `WG_EST_SHARED`（surface est 复用 aquifer surface_cache） | **开** | `worldgen_handle.rs:761` |
| `WG_CA_CAP`（terrain_cache 容量） | 2048 | `worldgen_handle.rs:1028-1030` |
| `WG_SKIP_*` / `WG_*LOG` / `WG_*DUMP` | 关 | 各门控点 |
| `coreswap.mixlog` | 关（本轮已改） | `NoiseChunkGeneratorMixin.java:51` |

**「串行 bench」口径** = `.tmp/camin_bench_26090905/06.exe`（源码 `worldgen-core/src/bin-diag/camin_bench.rs`）。
它 `WorldgenHandle::create` → 循环调 `h.fill_chunk_blocks(cx,cz)`（`camin_bench.rs:32`）。**三处与实机臂不可比，全表判据必须带上这个前提**：

1. **不经过 JNI / `wg_fill_blocks_multi` / Java**：无 Java mixin、无 `writeChunk`、无线程 spawn、无 env-copy；
2. **flags=0**（`WorldgenHandle::create` → `flags: AtomicU32::new(0)`，`worldgen_handle.rs:503`）→ bench 臂 **Rust 侧 carver+features 都开**，实机臂两者被 flag 跳过（改由 Java 执行）→ bench 的 Rust 段包含了实机不付的 R13/R14；
3. **N=16 → 256 chunks < terrain_cache cap 2048** → bench 全程**不触发 `cache.clear()`**，实机 4225 chunks 会触发 2 次（2048 / 4096 处）。

**证据等级**：本图全部为静态推导（`path:line`）。除任务书已核实的条目外，凡「每 chunk 次数/占比/耗时」一律不填数，进 §6 待测清单。验证分层 = **Degraded（静态审查，无运行证据）**。

---

## 1. 逐环节通路成本图

### 1.1 调度层差异（结构性，先于一切计数）

| ID | 环节 | 定位 | 每 chunk 做什么 | 全局共享可变状态 |
|---|---|---|---|---|
| S1 | **vanilla arm：NOISE 卸载到共享 worker 池** | `versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:326-353` | `populateNoise` 立刻返回 `CompletableFuture.supplyAsync(..., Util.getMainWorkerExecutor().named("wgen_fill_noise"))`；真正填充在 `wgen_fill_noise` 池线程内跑 | 池内共享线程（池级） |
| S2 | **vanilla arm：填充期持 24 个 ChunkSection 锁** | 同上 `:336-340`（`chunkSection.lock()`）/`:346-348` unlock | 逐 section 加锁保护 | section 级锁 |
| S3 | **coreswap arm：全部工作同步跑在调用线程** | `runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:76-105`（HEAD `cancellable` + `cir.setReturnValue(completedFuture(chunk))` `:104`） | mixin HEAD 内**同步**完成 `feedBeardifier` + `fillChunk`（JNI→Rust→写回），无卸载、无 section 锁 | — |

> S1/S3 的差别不是 CPU 计数差，而是**并发结构差**：vanilla 的 Worker-Main 线程发完 populateNoise 即可继续取下一个 chunk（填充在另一池并行），coreswap 把整段填充压在 Worker-Main 上（实机日志 23 个 Worker-Main 全在 intercept 内）。这条与「吞吐稳定在 15.8 cps」直接相关，但缺测量（⚠️@idk-9）。

### 1.2 Java 侧每 chunk 固定开销

| ID | 项 | 定位 | 内容 | 全局共享可变状态 |
|---|---|---|---|---|
| J1 | mixin 拦截判定 | `NoiseChunkGeneratorMixin.java:88-128` | `comp.probe` 属性查询（`:88`）、`CppBridge.enabled` volatile 读（`:91`）、`chunk.getBottomY()/getHeight()`（`:92-93`）、`CppBridge.endActive()/netherActive()`（`:96/109`） | — |
| J2 | `wgSettingsId()` | 定义 `:54-59`；调用点 `:98`（populateNoise 主世界分支）、`:140`（buildSurface） | `getSettings().getKey().map(k -> k.getValue().toString())` → **每 chunk 2 次 `Identifier.toString()`（新字符串分配）**。overworld chunk 上 `:96/109/142/144` 因形状短路不触发（`zeroShape=false`） | — |
| J3 | buildSurface 拦截判定 | `:132-150` | 每 chunk 一次形状+settings 判定；取消 Java surface（`:148`） | — |
| J4 | features 接管判定 | `ChunkGeneratorFeaturesMixin.java:56-81` | 每 chunk HEAD：`CppBridge.rustFeaturesTakeover()`（`:65`）→ 内部 `resolveStageMask()` → `System.getProperty("coreswap.rust.stages")`（`CppBridge.java:69-74`，**`System.getProperty` = `Properties`(Hashtable) 同步 get**）；mask=3 → false → `WG_TAKEOVER_ACTIVE=false` | JVM 全局 `Properties` 监视器 |
| J5 | `@Redirect` 转发 | `ChunkGeneratorFeaturesMixin.java:119-138` | 每次 `placedFeature.generate` 多一层 redirect；接管为 false → 原样转发 `:137` | — |
| J6 | `setPopulationSeed/setDecoratorSeed` 钩子 | `ChunkRandomSeedLogMixin.java:21-36`；`WgDiag.java:18-19,50-54` | `setPopulationSeed` 每 chunk 1 次：`WgDiag.setCurrent` ThreadLocal 写 + `Boolean.getBoolean("wg.seedlog")` + `System.getenv("WG_SEEDLOG")`；`setDecoratorSeed` 每 (chunk×step×p) 一次同样两次查询 | JVM `Properties` 监视器 + OS env 系统调用；`WgDiag` ThreadLocal |
| J7 | `fillChunk` 缓冲与诊断 | `CppBridge.java:373-401` | `BUF.get()` ThreadLocal（`:377`，每线程 384KB，23 线程 ≈9MB）；`new int[]{cx}/{cz}/int[][]{buf}`（`:380-381`）；**`:391-395` 无条件全量扫 98304 项 `if (buf[k]!=0)` 纯诊断计数**；异常/异常形态才打印（`:383/387/393/395`） | 无（ThreadLocal） |
| J8 | `writeChunk` 写回 | `CppBridge.java:494-535` | ① `chunk.getSection(i)` ×24（`:497`）② **98304 次 `sec.setBlockState(x,sy,z,st)`**（`:522`），每次前置 `STATE_BY_ID.get(id)`（`AtomicReferenceArray` 读 `:507`）+ 上界检查（`:505`），首见 id 才走 `Registries.BLOCK.get(id)`（`:515-518`）③ **`Heightmap.populateHeightmaps(chunk, 6 种 type)`**（`:528-534`）④ **写全部 98304 格含 air**（vanilla 只写非 air：`NoiseChunkGenerator.java:411`） | `STATE_BY_ID`（原子数组，读为主） |
| J9 | 日志 | mixin `:51,99,110,118,147`（MIXLOG 默认关）；`CppBridge.java:383-399`（异常路径）；`CppBridge.java:440/484` **nether/end 分支每 chunk 一行 `[WG-FILL]` 同步 println** | overworld 正常路径 0 行；若实机同时生成 nether/end 则每 chunk 1 行 log4j 同步写（vanilla 臂同样为 0 行） | log4j 同步 appender |

**与 vanilla 臂的净差（同一阶段）**：
- vanilla NOISE 只增量维护 2 张高度图（`NoiseChunkGenerator.java:359-360` + `:413-414` `trackUpdate`），coreswap 在 NOISE 全量重建 6 张（`CppBridge.java:528-534`）；**两臂都会在 FEATURES 状态再全量重建 4 张**（`ChunkGenerating.java:135-137`）→ coreswap 多付 6 张全量 + 写含 air 的全量。
- vanilla `populateNoise` 在 aquifer 需要 fluid tick 时 `chunk.markBlockForPostProcessing`（`NoiseChunkGenerator.java:415-418`）；Rust 路径无对应动作 → 后续流体 tick 负载两臂可能不同。

### 1.3 JNI 边界每 chunk 的分配与拷贝

| ID | 项 | 定位 | 字节量 / 次数 | 备注 |
|---|---|---|---|---|
| N1 | Java 侧入参打包 | `CppBridge.java:380-381` | 3 个小数组/chunk | — |
| N2 | JNI 长度/元素校验 | `versions/1.21.6/rust/src/jni_bridge.rs:186-204` | `get_array_length`×3、`get_object_array_element`×1、`get_int_array_region`×2（count=1） | — |
| N3 | **本地 buffer 零初始化** | `jni_bridge.rs:205` | `vec![0i32; 98304]` = **393,216 B 零初始化**，每 chunk 一次 | 每次新分配（未复用/未 `MaybeUninit`） |
| N4 | 输出指针数组 | `jni_bridge.rs:206` | 1 元素 Vec | — |
| N5 | 结果拷回 Java 数组 | `jni_bridge.rs:219-223` | `set_int_array_region` **393,216 B 拷贝**/chunk | 错误不吞（`:221` `?`） |
| N6 | Rust→C ABI 侧的第二次拷贝 | `api.rs:51-53,144,164` | `SendOut::write` `copy_nonoverlapping` **393,216 B**/chunk | 见 R1 |

**每 chunk 跨边界净开销**：393KB 零初始化 + 393KB 拷回 + 393KB（N6）≈ **1.2 MB 内存流量 + 2 次堆分配**（不含 JVM 侧 `int[]`）。

### 1.4 Rust 侧每 chunk

| ID | 项 | 定位 | 内容 | 全局共享可变状态 |
|---|---|---|---|---|
| R1 | `wg_fill_blocks_multi` count=1 包装 | `api.rs:131-171` | `adaptive_threads(-1,1)`（`:24-41`，实机 THREADS=-1 → 逻辑核/2-2，本机 10）；`Arc::new(h)` `:142`；`Arc::new(Vec<SendOut>)` `:144`；`std::thread::scope` **spawn 10 线程** `:146-152`（count=1 → 9 个空转即退）；每 spawn 线程体首行 `std::env::var("WG_DEBUG")` `:154`（**10 次/chunk**）；`outs[i].write()` 393KB `:164` | 线程创建/join（OS 级） |
| R2 | `fill_chunk_blocks` 包装 | `worldgen_handle.rs:613-678` | `WG_CA_MIN` env `:619`；`terrain_cache.lock()` 读 `:621-622`；miss → `fill_terrain_column`；**`c.clone()` 深拷贝 393KB** `:628`；`terrain_cache.lock()` 写 + `ca_cap()` env `:631-632`；**`if cache.len()>=cap { cache.clear(); }` `:633`（持锁清空）**；`flags` relaxed load `:642`；`CONF_ECHOED` 一次性 atomic `:649`；`WG_SKIP_FEATURES` 命中 flag → 短路不查 env `:669`；`col.data().to_vec()` **393KB 新分配+拷贝** `:677` | **`terrain_cache: Mutex<HashMap>`**（`:141`），每 chunk 2 次 lock；clear-all 持锁 drop ≈2048 个 `Arc<BlockColumn>`（≈805MB 内存释放）；驻留上限 2048×(393KB+1KB)≈**806MB** |
| R3 | 缓存命中路径（本配置不出现） | `worldgen_handle.rs:620-624` | hit → `((*e.col).clone(), (*e.heightmap).clone())` 也是 **393KB 深拷贝**；本配置 features 被 skip → `neighbor_terrain` 不被调用 → 实机每 chunk 只走 miss 路径（1 次深拷贝） | 同上 |
| R4 | `fill_terrain_column` env 查询 | `worldgen_handle.rs:683-837` | `WG_SKIP_AQUIFER` `:702`、`WG_EST_L2` `:706`、`WG_SKIP_OREVEIN` `:728`、`WG_EST_SHARED` `:761`、`WG_EST_DUMP` `:785`、`WG_SKIP_SURFACE` `:820`（`WG_SKIP_CARVER` 因 flag 短路 `:832`）；`aqdump_seed` `:695` → `AQDUMP_ON` relaxed load（`aquifer.rs:97-101`） | 每 chunk ≈6 次 env 查询（Windows 下 `env::var` = `GetEnvironmentVariableW` 级调用） |
| R5 | `Aquifer::new` | `worldgen_handle.rs:696-699` → `aquifer.rs:310-339` | 每 chunk 3 个小 Vec（`block_positions` 315×8B、`water_levels` 315、`surface_cache` 1024×4B）+ **7 个 `Arc::clone`**（原子自增） | Arc 引用计数（跨线程 cacheline） |
| R6 | est L2 注入 | `worldgen_handle.rs:706-708` → `:597-602` | `OnceLock.get_or_init`（首 chunk 建）+ `Arc::clone` | **`est_l2: OnceLock<Option<Arc<Mutex<EstL2>>>>`（`:133`）** |
| R7 | Beardifier 读 | `worldgen_handle.rs:710` | 每 chunk `beardifiers.read().unwrap().get(&(cx,cz)).cloned()` | **`beardifiers: RwLock<HashMap>`（`:116`）** 全局读锁 + clone |
| R8 | `fill_chunk` 主循环 | `terrain.rs:256-301` | 分配 `ChunkData{ blocks: vec![Air; 98304]`（≈98KB）、`biome: vec!["";256]` `:264-265`；`DensityMacroSampler::build_slices` → `vec![0f64; 5×49×5×5]`=49KB + 6125 次 channel 采样 `:43-60`；**98304 次 `sample_interp`**（三线性插值 + combine）`:274-298`；每点 `aqua.classify` `:286` → `Aquifer::apply`（`aquifer.rs:454+`；d>0 直接 Rock，d≤0 才进 blob 决策） | thread_local slice cache（`terrain.rs:25-27` / `:315-317`，非共享） |
| R9 | **est 全局锁** | `aquifer.rs:531-565`（`estimate_surface_height`），调用链 `:604-627`（`get_fluid_level` 每次 **13 次** est 查询）、`worldgen_handle.rs:780-783`（每 chunk 4 角 `est_at`） | per-chunk `surface_cache`(32×32) miss 时：`l2.lock()` get（`:540`）+ `l2.lock()` put（`:561`） | **`Mutex<EstL2>`（`aquifer.rs:307`，DEFAULT_CAP=131072 `:262`）**；23 线程共享同一把锁 |
| R10 | 列填充环 + 矿脉 | `worldgen_handle.rs:729-746` | 98304 次循环；**每个 Rock 块调 `ore_vein.apply`**（`ore_vein.rs:44-68`，y∈[-60,50] 内 1 次 `vein_toggle` 采样 + 可能 `splitter.split_xyz` + `vein_ridged/vein_gap`） | 只读 &self（无锁，`:43` 注释） |
| R11 | heightmap 映射 | `worldgen_handle.rs:753-754` | `cd.surface_height.to_vec()` 256 → map 新 Vec | — |
| R12 | surface | `worldgen_handle.rs:820-829` | `sb.build_surface`；`biome_at_surface` 每点 `biome_pick_cell` + biome 判定 | — |
| R13 | carver | `worldgen_handle.rs:832` | **flag 置位 → 整段跳过**（实机不付）；若开启：`apply_carvers` 289 邻域 × carver，且 `get_carver` = `self.carver_cache.get(id).cloned()` **深拷贝 carver 配置**（`:993-995`）、`carvers_for(...).to_vec()` 分配（`:956`） | — |
| R14 | features | `worldgen_handle.rs:669-675` | **flag 置位 → 整段跳过**；若开启：`neighbor_terrain` + `terrain_cache`（每 chunk 9 邻列快照 `:1120-1128`）、`pending_cross_writes` Mutex（`:1053/1088`）、越界读 853k 次量级（已核实事实） | — |
| R15 | 分配器压力 | R3/R8/R2 合计 | 每 chunk 堆分配 ≈ 393KB(BlockColumn) + 98KB(ChunkData.blocks) + 393KB(Arc 深拷贝) + 393KB(to_vec) + 393KB(JNI local) + 49KB(slices) ≈ **1.7MB / chunk**，×4225 chunks ≈ **7GB 级 malloc/free 流量** | allocator 全局锁（多线程放大） |

### 1.5 结构（Beardifier）开销清单

| ID | 项 | 定位 | 内容 |
|---|---|---|---|
| B1 | Java 侧每 chunk 构造 | `CppBridge.java:262-269` → `:307-313` | 每 chunk 调 `StructureWeightSampler.createStructureWeightSampler(structures, chunk.getPos())`（`:311-312`）——与 vanilla 在 `createChunkNoiseSampler` 内同源构造，**vanilla 也每 chunk 构造一次**（`NoiseChunkGenerator.java:356-358`） |
| B2 | 反射提取 | `CppBridge.java:275-305` | field 走 static 缓存 `fPieces/fJunctions`（`:281-293`）；method 走 `ConcurrentHashMap` 缓存（`:295-305`）；每个 piece 反射调 6 次 Box getter + terrain ordinal + delta（`:323-337`）、junction 3 次（`:338-345`） |
| B3 | 空结构快路径仍调 JNI | `CppBridge.java:346-354` | pieces/junctions 皆空 → **仍调** `CppWorldgen.setBeardifier(h,cx,cz,null,0,null,0)`（`:347`，注释：清空防残留） |
| B4 | JNI 侧 | `jni_bridge.rs:232-263` | 分配 2 个空 `Vec<i32>`（`:240/246`）+ `wg_set_beardifier` |
| B5 | **Rust 全局写锁 + 无界增长** | `worldgen_handle.rs:514-537` | 每 chunk `beardifiers.write().unwrap().insert((cx,cz), b)`（`:536`）——**全局写锁**；HashMap **无清理路径**（只有外部 `clear_beardifier` `:539-541`），条目随生成 chunk 单调增长（本 region ≈4225 条，内存小，但锁是全局串行点） |
| B6 | Rust 读侧 | `worldgen_handle.rs:710` | 每 chunk 一次全局读锁 + clone；空结构时 clone 廉价 |

> 净差：vanilla 侧 Beardifier 全程在 worker 池单线程内、不出 JNI、无全局 map；coreswap 额外付 B3/B4/B5/B6 = **每 chunk 一次 JNI + 一次全局 RwLock 写 + 一次全局 RwLock 读 + 一次 HashMap insert**。

### 1.6 Java 后续阶段（carver/features/structure/light）等价性核对点

| 阶段 | 实机 coreswap 臂谁在执行 | 定位 | 两臂等价性 |
|---|---|---|---|
| CARVERS | Java vanilla（Rust 侧 flag 跳过） | 无 mixin 拦截；Rust 侧 `worldgen_handle.rs:832` | 语义上两臂都由 Java 做；可核对点：Rust 写回是否让 carver 读到等价方块（carver 读 `chunk` section 内容） |
| FEATURES | Java vanilla（`rustFeaturesTakeover()` = mask 非零且 bit1 清 → mask=3 时 **false**） | `CppBridge.java:167-170`；`ChunkGeneratorFeaturesMixin.java:56-138` | 结构放置段与 feature 迭代都在 Java（v2 mixin 修正即为此）；可核对点：`@Redirect` 转发是否引入额外开销（每 feature 一次） |
| STRUCTURE | Java（`generateFeatures` 内） | 同上 | 可核对点：B1-B6 是否改变了 `StructureAccessor` 的消费时机（Rust 只读一次，不写回 Java 结构状态） |
| LIGHT | Java vanilla（`coreswap.light.rust` 未设 → 首行 return） | `ServerLightingProviderMixin.java:62,185-188` | 完全 vanilla；可核对点：coreswap 预置的 6 张高度图是否改变 light 阶段成本（light 不依赖高度图，`initializeLight` 另算） |
| HEIGHTMAP | coreswap 在 NOISE 预置 6 张；**两臂都会在 FEATURES 再全量 4 张** | `CppBridge.java:528-534` vs `ChunkGenerating.java:135-137` | **不等价**：coreswap 多 6 张全量重建（见 §1.2 净差） |

**另一处结构性可核对点**：vanilla `populateNoise` 会调 `chunk.getOrCreateChunkNoiseSampler(...)`（`NoiseChunkGenerator.java:356-358`）；coreswap 拦截后该采样器不再创建。若 1.21.6 后续阶段（carver/feature/structure）消费该采样器或其缓存，两臂成本结构会不同——**未核**（见 ⚠️@idk-16）。

---

## 2. 全局共享可变状态清单（锁 / 原子 / 分配 / 线程创建 / 同步 IO）

| 类别 | 状态 | 定位 | 每 chunk 访问频次 | 临界区 |
|---|---|---|---|---|
| Mutex | `terrain_cache: Mutex<HashMap<(i32,i32),CaTerrainEntry>>` | `worldgen_handle.rs:141`，用点 `:621/631` | 2 次 lock（读+写）；clear 时持锁 | clear-all 时 drop ≤2048 项（**≈805MB 释放**）持锁 |
| Mutex | `est_l2: OnceLock<Option<Arc<Mutex<EstL2>>>>` | `:133`，用点 `aquifer.rs:540/561`（经 `worldgen_handle.rs:706` 注入） | per est miss 2 次 lock（get/put），次数未定 | HashMap get/put（cap 131072） |
| RwLock | `beardifiers: RwLock<HashMap>` | `:116`，写 `:536`，读 `:710` | 1 写 + 1 读 | 写锁临界区 = 单条 insert；读锁临界区 = get+clone |
| Mutex | `pending_cross_writes: Mutex<HashMap>` | `:144`，用点 `:1053/1088` | **本配置 0 次**（features 被 skip） | — |
| 原子 | `flags: AtomicU32` | `:130`，用点 `:642/819` | 1–2 次 relaxed load | 无争用 |
| 原子 | `CA_NT_*` / `CA_*` / `AQDUMP_ON` / `SURF_WATCH` 等 | `:170-172`；`aquifer.rs:16-71` | 门控关时 = 计数 0（`SURF_WATCH` 每次 est 一次 relaxed load） | 无 |
| 原子 | `STATE_BY_ID: AtomicReferenceArray` | `CppBridge.java:34`，用点 `:507` | **98304 次读**（Java） | 无（幂等 set） |
| 线程创建 | `std::thread::scope` spawn 10 + join | `api.rs:146-152` | **10 spawn + 10 join / chunk**，9 个空转 | OS 调度器/TCB 分配 |
| 同步 IO | `std::env::var` 查询 | Rust 侧 §R1/R2/R4（≈17 次/chunk）+ Java `System.getProperty`/`getenv`（J4/J6，`Properties`(Hashtable) 同步 get + OS env） | ≈17（Rust）+ ≥3（Java，两臂都付） | JVM `Properties` 监视器；Windows env 系统调用 |
| 同步 IO | stdout println | mixin（已门控）；`CppBridge.java:440/484`（nether/end 每 chunk 1 行）；`:383-399`（异常）；Rust `[WG-CONF]` 一次性 `:651` | overworld 正常路径 0 行 | log4j 同步 appender |
| 分配 | 每 chunk ≈1.7MB 堆分配（R3/R8/R2+R15） | 见 §1.4 | 23 线程并发 → allocator 锁 + 内存带宽 | 全局 allocator |

---

## 3. 成本对照表（核心判据表）

> 「串行 bench」= `camin_bench` 直接调 `fill_chunk_blocks`（§0 三处口径差已述）。
> 「并发放大」= 该项在 23 个 Worker-Main × 各自 10 个 Rust 线程下是否随线程数放大（锁争用 / 线程创建 / 同步 IO / 内存带宽）。

| # | 成本项 | 定位 | 串行 bench 也付？ | 并发放大？ | 说明 |
|---|---|---|---|---|---|
| J1 | mixin 拦截判定（属性查询/volatile/形状） | `NoiseChunkGeneratorMixin.java:88-128` | ❌ | — | 纯 Java，bench 无此路径 |
| J2 | `wgSettingsId()` ×2（含 2 次字符串分配） | `:54-59,98,140` | ❌ | ❌ | 每 chunk 固定；无共享状态 |
| J4 | features 接管判定（`System.getProperty` 同步 get） | `ChunkGeneratorFeaturesMixin.java:65`；`CppBridge.java:69-74` | ❌ | ⚠️ 两臂都付 | `Properties` 全局监视器，23 线程同点 |
| J6 | seed hooks（property+getenv/特征调用） | `ChunkRandomSeedLogMixin.java:21-36` | ❌ | ⚠️ 两臂都付 | 频率随 feature 数 |
| J7 | **无条件 98304 项 buf 扫描（诊断）** | `CppBridge.java:391-395` | ❌ | ❌（但线程私有） | 纯浪费项，未门控 |
| J8a | **98304 次 `setBlockState`（含 air）** + `STATE_BY_ID.get` 每次 | `CppBridge.java:494-525` | ❌ | ❌ | vanilla 只写非 air（`NoiseChunkGenerator.java:411`） |
| J8b | **6 张高度图全量重建** | `CppBridge.java:528-534` | ❌ | ❌ | 两臂在 FEATURES 另付 4 张（`ChunkGenerating.java:135-137`） |
| B1-B4 | feedBeardifier（构造+反射+JNI） | `CppBridge.java:307-365`；`jni_bridge.rs:232-263` | ❌ | ❌ | vanilla 同位置也有 sampler 构造 |
| B5 | **Rust `beardifiers` 全局写锁 + HashMap insert** | `worldgen_handle.rs:536` | ❌（bench 不调 set_beardifier） | ✅ 每 chunk 一写，全局 | 23 线程同锁 |
| B6 | Rust `beardifiers` 全局读锁 + clone | `worldgen_handle.rs:710` | ✅（map 恒空，代价极小） | ✅ 全局读锁 | — |
| N3 | **JNI 本地 buffer 393KB 零初始化** | `jni_bridge.rs:205` | ❌ | ✅（分配器/带宽） | 每 chunk 新分配 |
| N5 | JNI 拷回 393KB | `jni_bridge.rs:219-223` | ❌ | ✅（带宽） | — |
| N6 | Rust→C 拷贝 393KB | `api.rs:51-53,164` | ❌ | ✅（带宽） | — |
| R1a | **10 线程 spawn/join（9 空转）** | `api.rs:146-152` | ❌（bench 不经此函数） | ✅ 23×10 次/chunk 组 | count=1 不 clamp（`:38-40`） |
| R1b | spawn 体内 `env::var("WG_DEBUG")` ×10 | `api.rs:154` | ❌ | ✅ | Windows 系统调用级 |
| R1c | `Arc::new(h)` + `Arc::new(Vec<SendOut>)` | `api.rs:142,144` | ❌ | ✅ 原子操作 | — |
| R2a | **`col.data().to_vec()` 393KB** | `worldgen_handle.rs:677` | ✅ | ✅（分配/带宽） | 每 chunk 一次 |
| R2b | **`c.clone()` 深拷贝 393KB 进 cache** | `worldgen_handle.rs:628` | ✅ | ✅ | miss 路径每 chunk |
| R2c | **`terrain_cache` 2 次全局 lock** | `worldgen_handle.rs:621,631` | ✅ | ✅ **重点嫌疑** | 23 线程同锁，每 chunk 2 次 |
| R2d | **`cache.clear()` 持锁释放 ≈805MB** | `worldgen_handle.rs:633` | ❌ **bench 256 chunks < cap 2048 从不触发** | ✅ 2 次/region，触发时其余 22 线程全部阻塞 | cap 2048（`:1028-1030`）；区域 4225 chunks → 2048/4096 处各一次 |
| R3 | hit 路径 393KB 深拷贝 | `worldgen_handle.rs:620-624` | ✅（bench 有 cache 复用） | ✅ | 本配置 features skip → 实机不出现 |
| R4 | env 查询 ≈6 次/chunk（Rust 热路径） | `worldgen_handle.rs:619/632/702/706/728/761/785/820` | ✅ 部分（同代码路径） | ⚠️（若 env 实现含全局锁） | 次数为静态清单，未实测 |
| R5 | `Arc::clone` ×7/chunk | `worldgen_handle.rs:696-699` | ✅ | ✅ 原子 cacheline | — |
| R6 | est L2 注入（OnceLock + Arc clone） | `worldgen_handle.rs:706-708` | ✅ | ✅ | 首 chunk 后仅原子 load |
| R8 | `ChunkData` 分配 98KB + slices 49KB + 98304 次插值 | `terrain.rs:264-265,43-60,274-298` | ✅ | ✅（分配/带宽，CPU 本身可并行） | 串行 bench 的主 CPU 项 |
| R9 | **`est_l2` 全局 Mutex（get/put per miss）** | `aquifer.rs:540,561`（入径 `worldgen_handle.rs:780-783` + `aquifer.rs:604-627`） | ✅ | ✅ **重点嫌疑** | 次数未定（⚠️@idk-5），跨 chunk 累积命中 |
| R10 | 每 Rock 块 `ore_vein.apply` | `worldgen_handle.rs:741-744`；`ore_vein.rs:44-68` | ✅ | ❌（无锁，纯 CPU） | 两种臂都付（实机也付） |
| R12 | surface（`build_surface`） | `worldgen_handle.rs:820-829` | ✅（bench flags=0 也付） | ❌ | — |
| R13 | carver | `worldgen_handle.rs:832` | ✅（bench **付**） | — | **实机不付** → bench 数字含本项目 |
| R14 | features + `neighbor_terrain` + `pending_cross_writes` | `worldgen_handle.rs:669-675,1053,1088` | ✅（bench **付**） | — | **实机不付** → bench 的 853k 读/chunk 不适用于实机 |
| R15 | 分配器压力 ≈1.7MB/chunk | 合计 | 部分（bench 无 JNI 393KB） | ✅ 全局 allocator 锁 + 带宽 | — |
| S1/S3 | 调度结构差（同步 vs `wgen_fill_noise` 卸载） | `NoiseChunkGenerator.java:326-353` vs mixin `:76-105` | ❌ | ✅ 结构性 | 见 ⚠️@idk-9 |

**判据表读法（对 5.15× 的候选排序，勿当结论）**：
1. **只在实机臂付费**的项 = J1/J2/J4/J6/J7/J8/B1-B6/N1-N6/R1a-c + R2d(clear) —— 这些是 bench 完全看不到的部分；其中 **R2d（持锁释放 805MB）、R2c（每 chunk 2 次全局锁）、R9（est 全局锁）、R1a（每 chunk 10 线程）、J8（98304 写 + 6 高度图）** 是规模最大的五项。
2. **两臂都付费**的项（J4/J6/R10/R12 等）不能解释回归，但抬高了两臂基线。
3. bench 数字（177.9 / 118.9–128.5 ms/chunk）**包含实机不付的 carver+features**（R13/R14），且**不含** clear-all 与全部 Java/JNI 项 → **不能**用「bench ca_min on/off 差 49%」外推实机 5.15×。

---

## 4. 结构开销清单（汇总一行读本）

| 结构相关开销 | 定位 | 实机每 chunk | vanilla 对应 |
|---|---|---|---|
| `StructureWeightSampler` 构造 | `CppBridge.java:311-312` | 1 次 | 1 次（`NoiseChunkGenerator.java:356-358` 内） |
| 反射提取 pieces/junctions | `CppBridge.java:317-345` | 频率 = 结构件数（本 region 未测） | 不需要（Java 直接用对象） |
| JNI setBeardifier | `CppBridge.java:347/353` → `jni_bridge.rs:232-263` | 1 次（空结构也调） | 无 |
| Rust 全局写锁 + insert | `worldgen_handle.rs:536` | 1 次 | 无 |
| Rust 全局读锁 + clone | `worldgen_handle.rs:710` | 1 次 | 无 |
| `beardifiers` 表无界增长 | `worldgen_handle.rs:116/536`（无 evict 路径） | +1 条/chunk | 无 |

---

## 5. 不确定项（⚠️ @idk）

> 每条 = 具体未知点 + 需要什么测量才能钉死。**禁止猜数**：下文中一切未实测的次数/占比都不给数值。

- ⚠️ **@idk-1**：`terrain_cache` 每 chunk 2 次 `Mutex` lock 在 23 线程下的真实等待量级（及 `clear()` 持锁时长）。**需要**：实机 `WG_CA_MIN=0` 同 seed/region A/B（一次性归零 R2b/R2c/R2d 三项）+ Windows 上对该 Mutex 的等待时间采样（或 Rust 侧 chunk 级计时打点，门控）。
- ⚠️ **@idk-2**：`est_l2` 全局 Mutex 的每 chunk lock 次数与阻塞时间。**需要**：`WorldgenHandle::est_l2_stats()`（`worldgen_handle.rs:604-609`，已存在但**当前无调用点**）接到 chunk 级输出；配合实机 `WG_EST_L2=0` A/B。
- ⚠️ **@idk-3**：`cache.clear()` 在 4225 chunks 中实际发生次数与单次持锁时长（释放 ≈805MB 的 free() 时长）。**需要**：clear 分支加一次性计时/计数打点（chunk 级、非热路径），或实机 `WG_CA_CAP=65536` A/B 消掉 clear。
- ⚠️ **@idk-4**：Rust 热路径 `std::env::var` 的每 chunk 真实次数与单次成本（Windows `GetEnvironmentVariableW` + std env 锁语义）。**需要**：静态清单已给（≈6 热路径 + 2 包装 + 10 spawn 内），单次成本需 microbench 或将 env 读取缓存为 `OnceLock` 后 A/B。
- ⚠️ **@idk-5**：每 chunk 实际 `estimate_surface_height` 调用次数与 L2 miss 数（即 `Aquifer::apply` 的 d≤0 展开规模）。**需要**：`SURF_WATCH/SURF_COUNT`（`aquifer.rs:54-59`）与 `EstL2` stats 的输出接到 chunk 级日志（当前未查到输出点）。
- ⚠️ **@idk-6**：Rust 臂 6 张 heightmap 全量重建 与 vanilla 增量维护的净成本差。**需要**：Java 侧 `populateHeightmaps` chunk 级计时（门控）或 `writeChunk` 分段计时。
- ⚠️ **@idk-7**：`writeChunk` 98304 次 `setBlockState`（含 air）与 vanilla「仅非 air」的净差。**需要**：`writeChunk` 分段计时 + 两臂非 air 计数（同 chunk 对比）。
- ⚠️ **@idk-8**：每 chunk 10 次线程 spawn/join 的墙钟成本（Windows 线程创建/销毁）。**需要**：microbench，或把 `adaptive_threads` 在 `count==1` 时 clamp 到 1 后 bench/实机 A/B（工程改动）。
- ⚠️ **@idk-9**：同步执行（S3）vs `wgen_fill_noise` 卸载（S1）对吞吐的影响（谁在跑重活、并发度）。**需要**：两臂线程名 census（jstack/`-D` 日志）+ `Util.getMainWorkerExecutor()` 线程数与 CPU 占用对比。
- ⚠️ **@idk-10**：该 region 实际 pieces/junctions 数量（决定 B2 反射迭代成本与 B5 写锁临界区大小）。**需要**：`feedBeardifier` 加一次性/chunk 级计数打点，或对 region 内结构分布做静态统计。
- ⚠️ **@idk-11**：`wgSettingsId()` 每 chunk 2 次 `Identifier.toString()` 分配的开销占比。**需要**：mixin chunk 级计时，或把 settings id 缓存在 generator 实例字段后 A/B。
- ⚠️ **@idk-12**：JNI `vec![0i32; 98304]` 是否每次真正 touch 全部 393KB 页（calloc 惰性零页 vs 立即写）。**需要**：RSS/页错误计数，或改 `thread_local` 复用 buffer 后 A/B（消除 R3）。
- ⚠️ **@idk-13**：实机是否显式设过 `WG_CA_MIN / WG_EST_L2 / WG_CA_CAP / CORESWAP_THREADS`（§0 默认值表是代码推导）。**需要**：直接查实机日志里的一次性行 `[WG-CONF] ca_min=... est_l2=... ca_cap=... flags=... coreswap_threads=...`（`worldgen_handle.rs:647-660`）——1 行即可钉死。
- ⚠️ **@idk-14**：实机是否同时在生成 nether/end（若生成，`CppBridge.java:440/484` 每 chunk 一行同步 println）。**需要**：实机日志中 `[WG-FILL]` 行数与维度分布。
- ⚠️ **@idk-15**：vanilla 臂 52s/4225 chunks 的实际并发度（12.3 ms/chunk 墙钟均值反推的并行结构）。**需要**：vanilla 臂线程/CPU census。
- ⚠️ **@idk-16**：coreswap 拦截后不再创建 `ChunkNoiseSampler`（`NoiseChunkGenerator.java:356-358` 被跳过）是否影响 1.21.6 后续阶段（carver/feature/structure）的成本或行为。**需要**：核 1.21.6 中 `getOrCreateChunkNoiseSampler` 的消费点（静态）＋必要时行为对比。
- ⚠️ **@idk-17**：`writeChunk` 不持 `ChunkSection.lock()`（vanilla `:338` 持）在实机并发下是否有可见影响（是否正确性与吞吐双未知）。**需要**：并发写同一 chunk 的路径审查 + 实机异常/竞态观察。

---

## 6. 待测项清单（按信息量排序）

1. **查实机日志的一次性 `[WG-CONF]` 行**（`worldgen_handle.rs:647-660`：`ca_min / est_l2 / ca_cap / flags / skip_* / coreswap_threads`）——0 成本，直接钉死 @idk-13（默认值假设）与线程数，并判定 §0 口径表是否成立。**同时查 `[WG-FILL]` 行数**（@idk-14）。
2. **实机 A/B：`WG_CA_MIN=0`**（关 c-A 地形缓存）——同 seed/region 比 wall。一次实验同时消掉 R2b（393KB 深拷贝）+ R2c（2 次全局锁）+ R2d（clear-all 805MB）三项（@idk-1/3），是单项排除法的最高杠杆项。
3. **实机 A/B：`WG_EST_L2=0`**（关 est 全局 L2 锁，`worldgen_handle.rs:706`）——单独分离 R9（@idk-2/5）；跑之前先把 `est_l2_stats` 接到 chunk 级输出。
4. **实机 A/B：`WG_CA_CAP=65536`**——只消掉 clear-all（R2d），与第 2 项对照即可把 clear 与常态化锁分开（@idk-3）。
5. **Java 侧 chunk 级分段计时（门控）**：`fillChunk` 内 JNI 前/后/`writeChunk` 三段 + `populateHeightmaps` 单独计时 —— 定位 J8（98304 写 + 6 高度图）与 J7（98304 扫描）占比（@idk-6/7）。
6. **Rust 侧 chunk 级分段计时（门控、非逐点）**：`fill_terrain_column` 内 ①density/aquifer ②列填充+ore_vein ③surface ④terrain_cache 包装 ⑤env 查询 —— 建立「Rust 内各段占比」并与 bench 口径对齐（@idk-4/5）。
7. **R1 修正后 A/B**：`count==1` 时不 spawn（或 clamp 到 1 worker）+ 复用 JNI buffer（`thread_local`）——消掉 R1a/R1b/N3（@idk-8/12）。工程改动，需编译验证。
8. **两臂线程 census**：Worker-Main / `wgen_fill_noise` 活跃线程数与 CPU 占用（S1 vs S3，@idk-9/15）；含 jstack 定点采样。
9. **结构分布核对**（@idk-10）：region 内 pieces/junctions 数量 → 判定 B2/B5 是否需要优化。
10. **`wgSettingsId` 缓存化 A/B**（@idk-11）与 **`ChunkNoiseSampler` 消费点静态核对**（@idk-16）——信息量低于前 9 项，可在前 9 项收敛后按需。

---

**本图未做的事（越界声明）**：未跑任何命令/探针；未写 `knowledge/`、`docs/`；未修改任何源码；所有结论为 draft 级（静态推导、Degraded 验证分层），**不含 confirmed 级判断**。上表「嫌疑排序」是判据组织方式，不是根因结论。
