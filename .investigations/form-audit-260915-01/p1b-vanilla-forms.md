# P1b — vanilla Minecraft 1.20.1 各世界生成阶段执行形态测绘（一手源）

---
工作块: 260915-01（形态审计 P1b，计划 000-260914-04b）
角色: recode-scout（只读勘探）
日期: 2026-09-15
状态: draft（勘探产物，供 P2 fan-out 对照用）
---

## 0. 版本核验（#5 教训强制项）

**源副本**：`.tmp/scout-260905-08/mcsrc/`（yarn mappings 反编译源）
**核验点**：`net/minecraft/MinecraftVersion.java:30-32` — `this.name = "1.20.1"`、`new SaveVersion(3465, "main")`（DataVersion 3465 = 1.20.1 ✅）。
结论：该副本确认为 1.20.1，历史「.tmp/net 非 1.20.1」疑点不适用于本副本，可放心引用。其余 .tmp 副本（jungle-l-260906/mcsrc、light-yarn*）未逐一核版本，本文件不引用。

以下引用全部相对 `.tmp/scout-260905-08/mcsrc/`，行号为一手源行号。

---

## 1. 阶段管线总览（ChunkStatus 状态机）

管线定义：`net/minecraft/world/chunk/ChunkStatus.java:43-202`

```
EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES → BIOMES → NOISE → SURFACE
      → CARVERS → FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL
```

- 每个 ChunkStatus 挂一个 GenerationTask（`ChunkStatus.java:346-371` runGenerationTask）；任务粒度 = **单 chunk**，依赖邻域由 `taskMargin`（STRUCTURE_REFERENCES/BIOMES/NOISE/SURFACE/CARVERS/FEATURES = 8，INITIALIZE_LIGHT = 0，LIGHT = 1，ChunkStatus.java:46-189）+ ChunkRegion 描述。
- **调度入口**：`server/world/ThreadedAnvilChunkStorage.java:637-680` upgradeChunk —— `Executor executor = task -> this.worldGenExecutor.send(ChunkTaskPrioritySystem.createMessage(holder, task))`（:643）。即**所有 generationTask 首先被投到 worldgen 优先级 executor**，任务内部可再自行 supplyAsync 换池。
- ChunkTaskPrioritySystem 按 chunk ticket level 排序 + 支持取消（`server/world/ChunkTaskPrioritySystem.java`，TACR:189 构造，队列上限 Integer.MAX_VALUE）。

## 2. executor 图谱（世界生成期间）

| executor | 定义点 | 池实现/宽度 | 跑什么 |
|---|---|---|---|
| **Util.getMainWorkerExecutor()** | `util/Util.java:89,229-231` | ForkJoinPool，宽 = `clamp(cores-1, 1, max.bg.threads)`，max.bg.threads 默认 255（Util.java:182-224），asyncMode=true | BIOMES/NOISE 的 supplyAsync 体、getUpdatedChunkNbt 的 NBT 升级（TACR:966） |
| **"worldgen" TaskExecutor** | TACR:184 `TaskExecutor.create(executor, "worldgen")` | 包装同一个 main worker executor（见链：MinecraftServer.java:316 `workerExecutor = Util.getMainWorkerExecutor()` → ServerWorld.java:240 → ServerChunkManager.java:96 → TACR ctor:154/183） | 所有 ChunkStatus generationTask 的默认落点（worldGenExecutor = 优先级包装，TACR:190） |
| **"light" TaskExecutor** | TACR:188 `TaskExecutor.create(executor, "light")` | 同上（同一物理池） | ServerLightingProvider.processor（光照批处理）+ 优先级 executor（TACR:192-193） |
| **mainExecutor / mainThreadExecutor** | TACR:185,191 / TACR:125,183 | server 主线程（ThreadExecutor） | convertToFullChunk（FULL 阶段，TACR:736）、serialize/save（unloadChunks 循环，TACR:488-518）、makeChunkTickable 等 |
| **Util.getIoWorkerExecutor()** | Util.java:236-238 | IO ForkJoinPool（cores/4 或类似，本文件未展开） | region 文件写盘（TACR setNbt 底层） |

要点：**worldgen 与 light 是同一物理 ForkJoinPool 上的两个逻辑队列**（经 ChunkTaskPrioritySystem 按 chunk 优先级统一排序），并非独立池；主线程只做 FULL 转换、存盘、tick。

## 3. 各阶段一手形态（文件:类:行号）

### 3.1 populateBiomes（BIOMES）
- `world/gen/chunk/NoiseChunkGenerator.java:87-92`：`CompletableFuture.supplyAsync(..., Util.getMainWorkerExecutor())` —— **忽略传入的 executor 参数**，硬编码主 worker 池。
- 内部（:94-100）：`chunk.getOrCreateChunkNoiseSampler(...)`（ChunkNoiseSampler **缓存在 chunk 上**，跨阶段复用）→ `chunk.populateBiomes(biomeSupplier, multiNoiseSampler)`。
- ① 异步（单跳 future）② main worker 池 ③ 1 chunk ④ 全量（本 chunk 全部 biome 格）⑤ ChunkNoiseSampler 缓存（写=首次创建，读=后续所有阶段）。

### 3.2 populateNoise / noise（NOISE）
- `NoiseChunkGenerator.java:330-357`：对覆盖 section 先 `lock()`（:342-346），然后 `supplyAsync("wgen_fill_noise", ..., Util.getMainWorkerExecutor())`，`whenCompleteAsync(unlock, executor)`（executor=worldgen 优先级 executor，:348-355）。
- 内部（:359+）：cell 采样→逐 cell 插值→写 ChunkSection + 双 heightmap（OCEAN_FLOOR_WG/WORLD_SURFACE_WG，:363-364）；aquifer 来自 ChunkNoiseSampler。
- ① 异步 ② main worker 池（体）/ worldgen 队列（收尾）③ 1 chunk ④ 全量（无增量噪声；BelowZeroRetrogen 特例在 ChunkStatus.java:100-108）⑤ ChunkNoiseSampler 缓存 + heightmap 顺带产出。

### 3.3 surface（SURFACE）
- `ChunkStatus.java:114-119`（SimpleGenerationTask，**同步 void，直接在 worldgen 任务线程内联执行**）→ `NoiseChunkGenerator.buildSurface` :242-276 → `noiseConfig.getSurfaceBuilder().buildSurface(...)`，复用 chunk 上的 ChunkNoiseSampler（:261-263）。
- ① 同步 ② worldgen 优先级队列线程 ③ 1 chunk ④ 全量 ⑤ ChunkNoiseSampler 缓存（读）。

### 3.4 carver（CARVERS）
- `ChunkStatus.java:120-142`（同步）→ `NoiseChunkGenerator.carve` :279-327。
- **批粒度特殊**：对**本 chunk** 应用 carve，但枚举 **17×17（j,k ∈ [-8,8]）邻 chunk 位**的 carver 配置（:303-326）——即中心 chunk 只被 carve 一次，代价是邻域 biome/carver 配置查询。
- ChunkRandom per (carver, 邻 chunk) setCarverSeed（:318）；CarvingMask 挂在 ProtoChunk（:301，增量记录已 carve 位，跨 AIR/LIQUID 两步复用）。
- ① 同步 ② worldgen 线程 ③ 名义 1 chunk（实际 17×17 种子枚举）④ 增量 mask + 两阶段（AIR 步在 CARVERS；LIQUID 步 1.20.1 已并入/由 GenerationStep.Carver 控制）⑤ ChunkNoiseSampler（aquifer 交互，:297）+ CarvingMask。

### 3.5 feature / generateFeatures（FEATURES）
- `ChunkStatus.java:143-157`（同步）：先 `Heightmap.populateHeightmaps` 四类正式 heightmap，再 `generator.generateFeatures(chunkRegion, chunk, ...)`（ChunkRegion margin=1，:153），尾接 `Blender.tickLeavesAndFluids`（:155）。
- `world/gen/chunk/ChunkGenerator.java:334-423`：3×3 chunk 的 biome 集合收集（:346-352）→ 按 GenerationStep.Feature 逐 step：先 structure place（:362-379）再 PlacedFeature 索引排序去重逐个 `placedFeature.generate`（:398-412）；ChunkRandom.setDecoratorSeed 每 feature 重播种（:402）。feature 写邻 chunk 会被 ChunkRegion 的写边界保护（yarn ReadableChunkCache 语义，未展开）。
- ① 同步 ② worldgen 线程 ③ 1 chunk（3×3 biome 读）④ 全量 ⑤ indexedFeaturesListSupplier 惰性缓存（ChunkGenerator 字段，:342）。

### 3.6 光照（INITIZE_LIGHT → LIGHT）—— 重点
**调度链**（`server/world/ServerLightingProvider.java`）：
- INITIALIZE_LIGHT：`initializeLight`（:151-169）—— PRE_UPDATE 任务把所有非空 section `setSectionStatus(false)`（:153-163）；返回 future 的执行器 = POST_UPDATE enqueue（:164-168）。
- LIGHT：`light(chunk, excludeBlocks)`（:171-184）—— PRE_UPDATE 里 `super.propagateLight(chunkPos)`（:174-178）；POST_UPDATE 里 `chunk.setLightOn(true)` + `releaseLightTicket`（:179-183）。
- **批机制（taskBatchSize=1000，:34）**：所有 API（checkBlock/setSectionStatus/propagateLight/...）只 enqueue 到 pendingTasks（:127-138，经 ChunkTaskPrioritySystem 优先级 executor）；`runTasks()`（:195-218）按批 drain ≤1000 条：**先跑完批内全部 PRE_UPDATE → 一次 `super.doLightUpdates()`（真正的传播）→ 再跑 POST_UPDATE（完成 future、setLightOn）**。触发：入队满 1000（:134）或 `tick()`（:186-193，每 server tick 检查 pendingTasks/hasUpdates 后投 processor="light" TaskExecutor）。
- ① 异步（两段 future），但传播本身**串行单线程**（"light" 队列）② light TaskExecutor（与 worldgen 同物理池、独立优先级队列）③ **批粒度 = ≤1000 个待办任务/chunk 间混批**（不是按 chunk 分界）④ 见下 ⑤ 见下。

**传播模型**（`world/chunk/light/`）：
- `LightingProvider.java:20-24`：blockLightProvider = ChunkBlockLightProvider、skyLightProvider = ChunkChunkSkyLightProvider，两个引擎独立。
- `ChunkLightProvider.java`：增量 BFS —— `blockPositionsToCheck`（LongOpenHashSet，:29）+ 增/减两条 FIFO 传播队列（:30-31）；`doLightUpdates()`（:144-160）= drain 检查点集 → drain 减光队列（method_51570/:182）→ drain 增光队列（method_51567/:162）→ `clearChunkCache` → `lightStorage.updateLight(this)` → `lightStorage.notifyChanges()`。
- **chunk 首亮（全量种子化）**：`ChunkBlockLightProvider.propagateLight`（:112-121）= setColumnEnabled(true) 后把本 chunk **全部发光方块**种子入增光队列；`ChunkSkyLightProvider.propagateLight`（:292-343）= 启用列，读**自身 + 4 邻（N/S/W/E）ChunkSkyLight heightmap**，heightmap 以上列直接填 15（:324），边界处打包方向位种子入队（:327，method_51578）。
- **邻 chunk 交互**：不重算邻 chunk——跨 chunk 传播靠共享 LightStorage + 传播队列自然越界；邻域预读仅 sky 的 4 向 heightmap（非 3×3 全量）。unload 侧 `updateChunkStatus`（ServerLightingProvider:69-83）把 section 数据置 null + 标 notReady。
- **增量触发**（运行时改方块）：`checkBlock`（ServerLightingProvider:59-67）只登记单点 → 下一次 runTasks 的 doLightUpdates 增量收敛；`needsLightUpdate` 门槛（ChunkLightProvider:43-50）过滤 opacity/luminance/面透不变的改动。
- **缓存层**：① ChunkLightProvider 2 槽 chunk 缓存（:33-35,94-118，doLightUpdates 末 clearChunkCache）；② LightStorage 的 ChunkToNibbleArrayMap（带 cache 的 nibble 存储 + `uncachedStorage` 快照，`LightStorage.java:276-294` notifyChanges 时 copy+disableCache 给读者）；③ dirtySections/notifySections 驱动 `chunkProvider.onLightUpdate` 回调（:284-293，TACR 消费→标记发包）。写者 = light 线程（runTasks），读者 = 主线程（serialize/发包）与 worldgen 线程。
- **checkLight 机制**：1.20.1 无独立 "checkLight" 状态；等价物 = LIGHT ticket + `world.getChunkManager().getChunk ... ChunkStatus.LIGHT`（TACR:560-572 addTicketWithLevel(LIGHT) + removeTicketWithLevel，:685-688）与 `LightingProvider.checkBlock` 增量路径。

### 3.7 装饰 / populateEntities（SPAWN）
- `ChunkStatus.java:182-188`（同步，margin 0）：`generator.populateEntities(new ChunkRegion(...))`；NoiseChunkGenerator 实现（ChunkGenerator.java:440 抽象）。1 chunk、全量、无缓存。BelowZeroRetrogen 时跳过。

### 3.8 FULL 转换 + 写回/序列化
- **convertToFullChunk**：`ThreadedAnvilChunkStorage.java:711-737` —— ProtoChunk→WorldChunk 转换任务经 `mainExecutor.send(...)`（:736）落 **server 主线程**；含光照回调 `onLightUpdate` 挂钩（ReadableContainer 语义未展开）。
- **序列化**：`save(Chunk)`（TACR:797-827）在主线程 tick/unload 循环调用（unloadChunks:488-518，**每 tick 限 20 个** :513）；`ChunkSerializer.serialize(world, chunk)`（`world/ChunkSerializer.java:264-343+`）**同步主线程**：逐 section 写 block_states/biomes codec（:310-311），并从 `lightingProvider.get(BLOCK/SKY).getLightSection(...)` 直读 nibble 写 BlockLight/SkyLight（:304-320），`isLightOn` 标志（:330-332）。⚠️ 任务提到的 WritableLevelChunk 在 1.20.1 yarn 该签名下未出现——serialize 接收 `Chunk`（参数即 server 侧 chunk 实例，WorldChunk 实现 WritableLevelChunk 接口家族），未见独立调用点；如需精确接口链留给 P2。
- NBT 构建后 `setNbt` 走异步 IO（worker）；`getUpdatedChunkNbt` 的 DataFixer 升级在 `Util.getMainWorkerExecutor()`（TACR:965-966）。
- ① serialize 同步 ② 主线程 ③ 1 chunk（限 20/tick）④ 全量 NBT 重建（无脏区增量；needsSaving 布尔门控 :799-802）⑤ 读 LightStorage nibble + heightmap。

## 4. 五形态维度速查矩阵

| 阶段 | ① 同步/异步 | ② 线程/池 | ③ 批粒度 | ④ 增量 vs 全量 | ⑤ 缓存层 |
|---|---|---|---|---|---|
| BIOMES | 异步单跳 | **硬编码 main worker 池**（忽略传入 executor） | 1 chunk | 全量 | ChunkNoiseSampler（挂 chunk） |
| NOISE | 异步（section lock + unlock 回调） | main worker 池体 / worldgen 队列收尾 | 1 chunk | 全量 | ChunkNoiseSampler + 双 WG heightmap |
| SURFACE | 同步内联 | worldgen 队列线程 | 1 chunk | 全量 | ChunkNoiseSampler（复用） |
| CARVERS | 同步内联 | worldgen 队列线程 | 1 chunk（17×17 种子枚举） | CarvingMask 增量记录 | ChunkNoiseSampler/aquifer |
| FEATURES | 同步内联 | worldgen 队列线程 | 1 chunk（3×3 biome 读） | 全量 | indexedFeatures 惰性索引 |
| INITIALIZE_LIGHT | 异步两段 | light 队列 | 混批 ≤1000 任务 | section 状态登记 | — |
| LIGHT | 异步两段 | light 队列（传播串行） | 混批 ≤1000 任务 | **增量 BFS**；首亮=种子化（sky 读 4 邻 heightmap，block 全光源入队） | 2 槽 chunk 缓存 + LightStorage nibble（cache/uncached 快照）+ dirtySections→onLightUpdate |
| SPAWN | 同步内联 | worldgen 队列线程 | 1 chunk | 全量 | — |
| FULL/serialize | serialize 同步 | **主线程** | 1 chunk（20/tick 上限） | 全量 NBT（needsSaving 门控） | 读 LightStorage/heightmap |

## 5. 1.21.6 差异标注

⚠️ **无法从一手源标注**：仓库内 1.21.6 侧仅有 gradle/loom 工程（`versions/1.21.6/java/build.gradle`，minecraft 1.21.6 + yarn 1.21.6+build.1），**未解包源码目录**，本机 loom 缓存中也无 1.21.6 sources jar（已检索 `.gradle/caches/fabric-loom`，无命中）；genSources 属构建动作，本勘探禁跑。差异对照留待后续工作块先解包 1.21.6 源（注意复刻 #5 版本核验流程）。

## 6. 待深入点（供 P2 worker）

1. `Util.getMainWorkerExecutor` 池宽公式在 dedicated server 下的实际值（cores-1）——与 CoreSwap #122 池宽死参数直接可比。
2. ChunkTaskPrioritySystem 的优先级/取消细节（本文件只到"TACR:643 包装"一层）。
3. LightStorage.updateLight 的 queued nibble 合并语义（:146-190，部分展开）。
4. WritableLevelChunk 接口链在 serialize 路径的精确角色（见 3.8 ⚠️）。
5. Blender（混生）分支未测绘（本世界首生不走）。
