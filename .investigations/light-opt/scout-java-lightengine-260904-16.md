# 勘探报告：Minecraft 1.20.1 光照引擎（LIGHT 阶段）结构地图

- 角色：recode.scout（只读勘探，不修改源码）
- 日期标签：260904-16（用户已核实）
- 状态：draft（静态源码阅读，Degraded 分层——无运行时验证）
- 目的：为「Rust 重写光照引擎（允许有损、不逐位对齐）」课题提供结构地图

## 0. 一手源码位置与缺口声明

- 指定位置 `.tmp/p2full/src-ext/net/minecraft/world/gen/` **只含 worldgen 类**（NoiseChunkGenerator / SurfaceBuilder / ChunkRegion / World / BiomeAccess 等 7 个文件），**不含任何光照类**——按任务约定报告此缺口。
- 完整 1.20.1（Forge + Mojmap 名）源码树实际位于 `.tmp/net/minecraft/`（5469 个 .java，含 `world/level/lighting/` 全部 14 个光照类 + `server/level/` 调度类）。本报告全部证据取自该树（文件已逐个打开核实）。
- 未自行下载/解包任何内容。

光照核心类清单（`.tmp/net/minecraft/world/level/lighting/`）：
BlockLightEngine / BlockLightSectionStorage / ChunkSkyLightSources / DataLayerStorageMap / DynamicGraphMinFixedPoint / LayerLightEventListener / LayerLightSectionStorage / LeveledPriorityQueue / LevelLightEngine / LightEngine / LightEventListener / SkyLightEngine / SkyLightSectionStorage / SpatialLongSet。
注意：**DynamicGraphMinFixedPoint + LeveledPriorityQueue 已不在光照路径上**（仅被 server/level 的 ChunkTracker/SectionTracker 用于 ticket 距离追踪）；1.20.1 光照传播用简单 FIFO（见 §2）。

## 1. 类层次地图

```
LightEventListener (接口: checkBlock/updateSectionStatus/...)
├─ LayerLightEventListener (接口: getLightValue/getDataLayerData + Dummy 实现)
│  └─ LightEngine<M extends DataLayerStorageMap<M>, S extends LayerLightSectionStorage<M>>  [抽象基类]
│     ├─ BlockLightEngine  <BlockLightSectionStorage.BlockDataLayerStorageMap, BlockLightSectionStorage>
│     └─ SkyLightEngine    <SkyLightSectionStorage.SkyDataLayerStorageMap, SkyLightSectionStorage>
└─ LevelLightEngine implements LightEventListener  [门面：block + sky 双引擎聚合]
   └─ ThreadedLevelLightEngine  [服务端线程化包装，加调度/任务队列]

存储线：
LayerLightSectionStorage<M>  [抽象]
├─ BlockLightSectionStorage  (内含 BlockDataLayerStorageMap)
└─ SkyLightSectionStorage    (内含 SkyDataLayerStorageMap: +topSections Long2IntMap +currentLowestY)
DataLayerStorageMap<M>  [DataLayer 的 section→layer 映射 + 2 项缓存, 可 copy 做双缓冲]
辅助：ChunkSkyLightSources（每 chunk 16x16 列的 sky 光源最低高度图, BitStorage 256 项）
```

关键方法签名（文件:行）：

| 类 | 方法 | 位置 |
|---|---|---|
| LightEngine | `int runLightUpdates()` — 主入口：处理 blockNodesToCheck → propagateDecreases → propagateIncreases → markNewInconsistencies → swapSectionMap | LightEngine.java:139-155 |
| LightEngine | `void checkBlock(BlockPos)` — 只是把坐标 add 进 `blockNodesToCheck`（LongOpenHashSet） | LightEngine.java:119 |
| LightEngine | 抽象：`checkNode(long)` / `propagateIncrease(long,long,int)` / `propagateDecrease(long,long)` | LightEngine.java:219-223 |
| LightEngine | `QueueEntry`（静态嵌套）：long 编码 [3:0]=fromLevel、[9:4]=6 方向位、bit10=fromEmptyShape、bit11=increaseFromEmission | LightEngine.java:225-323 |
| LevelLightEngine | 构造 `(LightChunkGetter, boolean hasBlockLight, boolean hasSkyLight)`；`getLayerListener(LightLayer)`；`getRawBrightness(pos, darken)`=max(block, sky−darken) | LevelLightEngine.java:20-24, 91-97, 145-149 |
| LevelLightEngine | 光照 section 范围 = level sections ± 1（`getLightSectionCount()=sectionsCount+2`, `getMinLightSection()=minSection−1`） | LevelLightEngine.java:156-166 |
| SkyLightEngine | `checkNode` / `updateSourcesInColumn` / `removeSourcesBelow` / `addSourcesAbove` / `propagateFromEmptySections` / `countEmptySectionsBelowIfAtBorder` / `propagateLightSources(ChunkPos)` | SkyLightEngine.java:48-352 |
| BlockLightEngine | `checkNode` / `propagateIncrease` / `propagateDecrease` / `getEmission` / `propagateLightSources(ChunkPos)` | BlockLightEngine.java:25-122 |
| ThreadedLevelLightEngine | `addTask(...)` / `tryScheduleUpdate()` / `runUpdate()`（PRE/POST_UPDATE 两段式，批量 1000）/ `initializeLight` / `lightChunk` / `updateChunkStatus` | ThreadedLevelLightEngine.java:116-219 |
| LayerLightSectionStorage | `getStoredLevel/setStoredLevel(long)` / `getDataLayerToWrite` / `markNewInconsistencies` / `queueSectionData` / `updateSectionStatus` / `swapSectionMap` | LayerLightSectionStorage.java:80-97, 123-165, 196-226, 255-274 |
| ChunkSkyLightSources | `fillFrom(ChunkAccess)` / `update(...)`（增量单列）/ `getLowestSourceY(x,z)` / `getHighestLowestSourceY()` | ChunkSkyLightSources.java:33-97, 143-159 |
| DataLayer | `get/set(x,y,z,v)`，索引 `y<<8|z<<4|x`，4-bit nibble；`data==null` 时为「均质 defaultValue」惰性表示（`fill(v)` 直接回到 null+default） | DataLayer.java:33-96 |

## 2. 传播算法

### 2.1 通用框架（LightEngine，双队列两阶段）

- 入口 `runLightUpdates()`（LightEngine.java:139）：先逐个 `checkNode(node)`（差异检测，产生 decrease/increase 入队），然后**先 propagateDecreases 再 propagateIncreases**（删光优先，防止先增后删的中间态错误），最后 `markNewInconsistencies`（吞入 queuedSections 新数据）+ `swapSectionMap`（双缓冲翻面 + 回调 onLightUpdate）。
- 队列：`LongArrayFIFOQueue decreaseQueue / increaseQueue`，**每节点入队两个 long**（位置 posLong + QueueEntry 编码）。**不是优先队列**——这是 1.20 重写点（旧版 DynamicGraphMinFixedPoint/LeveledPriorityQueue 已退役出光照路径）。
- increase 循环（:157-175）：出队 (pos, entry)；若 `isIncreaseFromEmission` 且 stored < fromLevel → 直接写 fromLevel；当 `stored == fromLevel` 才 `propagateIncrease`（已被更高光覆盖则跳过）。
- decrease 循环（:177-186）：直接 `propagateDecrease(pos, entry)`。
- `propagateIncrease`（Block 版 :45-75 / Sky 版 SkyLightEngine.java:135-168）：对 QueueEntry 中每个使能方向：邻居 stored < level−1，且 `level − opacity(neighborState) > stored`，且两方块之间形状不遮光（`shapeOccludes`，VoxelShape 面遮蔽），则写值并入队（level>1 才继续）；Sky 版多一步 `propagateFromEmptySections`（见 2.3）。
- `propagateDecrease`：邻居 stored ≤ fromLevel−1 → 清 0 继续减传播；否则（邻居更高、是被别的源照到的）→ `increaseOnlyOneDirection` 入增队列「回拉」。Block 版还会检查邻居自身 emission 重新点火（BlockLightEngine.java:87-96）。
- 衰减：`getOpacity = max(1, state.getLightBlock())`（LightEngine.java:78-80，**最小不透明度 1**，即每步至少 −1）；边界面遮蔽直接判 16 级全挡（`getLightBlockInto` :51-61）。

### 2.2 Block light（BlockLightEngine）

- `checkNode`（:25-43）：比较 emission 与 stored；emission < stored → 清 0 + decreaseAllDirections(k)；否则 PULL_LIGHT_IN（从邻居拉光入减队列）；emission>0 → increaseLightFromEmission 入增队列。
- `propagateLightSources(ChunkPos)`（:112-122）：setLightEnabled(true) 后遍历 chunk 的所有发光方块（`LightChunk.findBlockLightSources`）逐个入增队列。

### 2.3 Sky light（SkyLightEngine）

- 垂直特判（**不存在显式「darkened」命名**——任务描述的 darkened 对应 `Level#getSkyDarken`（天气/昼夜）在读取侧 `getRawBrightness(pos, skyDarken)` 扣减，不在传播引擎内；⚠️ 此点标注：未在本次勘探中逐行核对 Level.getSkyDarken，draft）。
- **15 级直下不衰减**不是靠方向判断，而是靠**光源模型**：`ChunkSkyLightSources`（16x16 高度图，语义=每列最低 sky 光源 Y）以上柱状全填 15（`setLightEnabled` 时 datalayer.fill(15)，SkyLightEngine.java:280-298；`propagateLightSources` :300-352 按列写 15）。高度图以下的 15 值才作为普通光源入增队列向侧向/向下传播。
- `checkNode`（:48-74）：取该列 `lowestSourceY`；y ≥ sourceY → REMOVE_SKY_SOURCE（15, 跳过 UP 方向的减）+ ADD_SKY_SOURCE（15, 跳过 UP 方向的增）成对入队（源位置变化）；否则若 stored>0 清 0 全向减传播，stored==0 → PULL_LIGHT_IN。
- `removeSourcesBelow` / `addSourcesAbove`（:82-133）：高度图变化时沿整列扫，清除/重建 15 光源；addSourcesAbove 会看 4 邻列的 lowestSourceY 决定哪些层要真正入增队列（被邻列更高源照到的层只写值不重复点火）。
- `countEmptySectionsBelowIfAtBorder` + `propagateFromEmptySections`（:194-256）：**空 section 穿透**——传播到 section 边界且下方连续是无光照数据的空 section 时，整条 16 高柱一次性写同值（跳 15 递减），模拟「直下不衰减」跨空段。这是 skylight 算法里最结构化的特判。
- skylight 存储 `getLightValue`（SkyLightSectionStorage.java:21-43）：高于 topSections 或无数据的空段直接读 15（`currentLowestY` 以上视为满）。

## 3. 调度 / 增量更新

调度链（服务端，1.20.1 无 1.20.2+ 的独立 lightThread 类 ThreadedLightingEngine；light 引擎跑在 ChunkMap 专用的 ProcessorMailbox 线程）：

```
ChunkMap 构造 (ChunkMap.java:163-172)
  ProcessorMailbox<Runnable> lightMailbox（名为 "light" 的专用线程, ProcessorHandle）
  queueSorter = ChunkTaskPriorityQueueSorter([worldgen, light, main], Integer.MAX_VALUE)
  lightEngine = ThreadedLevelLightEngine(chunkGetter, this, hasSkyLight, lightMailbox, sorter.getProcessor(lightMailbox))

写入路径:
  ThreadedLevelLightEngine.addTask(x, z, PRE/POST_UPDATE, run) (ThreadedLevelLightEngine.java:120-128)
    → sorterMailbox.tell(Message(..., chunkPos, priority=chunkMap.getChunkQueueLevel(pos)))
    → ChunkTaskPriorityQueueSorter 按 ticket 级别排序后投递到 light mailbox
    → lightTasks.add(...)；满 1000 或 tryScheduleUpdate() 触发
  tryScheduleUpdate (:185-193)：AtomicBoolean 去重，taskMailbox.tell(runUpdate)
  runUpdate (:195-219)：最多取 1000 条 —— 先跑全部 PRE_UPDATE → super.runLightUpdates()（真正传播）→ POST_UPDATE（如 setLightCorrect(true) + releaseLightTicket）
```

- 触发点：`ServerChunkCache.pollTask` → `lightEngine.tryScheduleUpdate()`（ServerChunkCache.java:548）；ChunkMap 截获 holder 时 `updateChunkStatus + tryScheduleUpdate`（ChunkMap.java:516-517）；`ChunkMap.hasWork()` 计入 `lightEngine.hasLightWork()`（ChunkMap.java:461）。
- **增量更新路径（setBlock 后）**：`LevelChunk.setBlockState`（LevelChunk.java:240-242）→ `skyLightSources.update(...)`（增量修正该列高度图，可能两侧同步修）→ `level.getChunkSource().getLightEngine().checkBlock(pos)`（即 Threaded 版 addTask(PRE_UPDATE, super.checkBlock)）。ProtoChunk 同理（ProtoChunk.java:108-116，且只在 status≥INITIALIZE_LIGHT 时）。之后依赖下一轮 runUpdate 消化。
- **chunk 加载初始化**（ChunkStatus 驱动，见 §5）：INITIALIZE_LIGHT → `ChunkAccess.initializeLightSources()`（填 ChunkSkyLightSources）+ `ThreadedLevelLightEngine.initializeLight`（:140-163：非空 section updateSectionStatus(false) PRE + setLightEnabled/retainData POST）。LIGHT → `lightChunk`（:165-183：PRE=propagateLightSources（若非已点亮重载），POST=setLightCorrect(true)+releaseLightTicket）。
- **chunk 卸载**：`updateChunkStatus`（:59-78）PRE：retainData(false)+setLightEnabled(false)+ 全部 light section queueSectionData(null)+ 全部 section updateSectionStatus(true)。
- 读侧双缓冲：`LayerLightSectionStorage.visibleSectionData`（volatile）/ `updatingSectionData`；每轮 `swapSectionMap`（:255-274）copy 翻面 + 对受影响 section 回调 `chunkSource.onLightUpdate` → ChunkHolder 记 changedSections → `ClientboundLightUpdatePacket`（ChunkHolder.java:171-193）。
- 优先级：所有任务带 `chunkMap.getChunkQueueLevel`（ticket level），ChunkTaskPriorityQueueSorter 按离玩家近优先执行——**同列任务的相对顺序仍由 sorter 保序，但跨 chunk 无全局顺序保证**。

### 全量重算会踩到的顺序依赖（draft 观察，供 Phase 2 评估）

1. **跨 chunk 边界传播**：光照是全局场；单 chunk 的 runUpdate 会把光传进邻 chunk 的 section（storage 按 section 全局寻址，`storingLightForSection` 依赖 `updateSectionStatus` 的 26 邻居计数，LayerLightSectionStorage.java:206-226）。全量重算必须先保证参与 chunk 的**方块数据已冻结**（≥CARVERS/FEATURES 完成），否则边算边改。
2. **skylight 高度图跨界依赖**：`propagateLightSources` 显式读 4 邻 chunk 的 `ChunkSkyLightSources`（SkyLightEngine.java:303-307）——邻 chunk 未到 INITIALIZE_LIGHT 时退化为 emptyChunkSources（按无遮挡处理），结果不同。全量重算需要「chunk 桌面 ≥ 玩家光照半径 + 1」的高度图可用性契约（对应 LIGHT status range=1）。
3. **POST_UPDATE 依赖 PRE 的完成**：setLightCorrect(true) 必须在传播后（lightChunk :176-182），`isLighted()` 用 `status.isOrAfter(LIGHT) && isLightCorrect()` 判断重载时可跳过（ChunkStatus.java:175-177）。若全量重算吞掉状态位，存档 isLightOn/重载语义会变。
4. **ticket 级优先序**：现有系统近玩家 chunk 光照先完成（玩家移动流畅性）；全量重算天然一次算一片，需自行决定批序，不影响正确性、影响响应性。
5. **增删 section 的 26 邻居计数不变量**：updateSectionStatus 维护 sectionStates 的 neighborCount[0,26]，错乱会导致 storingLightForSection 返回错值——全量重算可以整个绕开该机制（不用模拟它），但与遗留增量路径共存时需要兼容。
6. 双缓冲 visible/updating + sectionsAffectedByLightUpdates 回调是**每轮一次**的批粒度，客户端光照包按 section 过滤；重写时发包粒度/时机是行为面的一部分（允许有损时可放宽）。

## 4. 存储

- **DataLayer**：2048 字节 = 4096 nibble（4-bit × 16³），索引 `y<<8|z<<4|x`（DataLayer.java:41-43）；**均质惰性**：data==null 时只存 defaultValue（全 0 或全 15 的 section 零字节占位）。每 section 每层（BLOCK/SKY）各一个。
- **section 布局**：光照 section 比 chunk section 上下各多 1（`LIGHT_SECTION_PADDING=1`，LevelLightEngine.java:13, 156-166）。
- **内存**：LayerLightSectionStorage 持 `Long2ObjectMap<DataLayer>`（updating/visible 双份）+ queuedSections（同步 map，等待吞入的序列化数据）+ sectionStates(Long2Byte: hasData bit + 26 邻计数) + columnsWithSources/columnsToRetainQueuedDataFor。
- **序列化**（ChunkSerializer.java:288-319）：逐 light section，非空 DataLayer → `sections[i].BlockLight` / `SkyLight` byte[2048]；`isLightOn` 标志。读取（:179-184）：status≥INITIALIZE_LIGHT 时 ProtoChunk 挂 lightEngine，setLightCorrect(flag)。
- **ChunkSkyLightSources** 本身是否入库：ChunkAccess 字段，ChunkSerializer 勘探段未见显式 NBT 写出（按 fillFrom 重算恢复；⚠️ 未逐行确认 1.20.1 存档是否有其序列化，draft）。
- **下游读者清单**（调用点，grep 证据）：
  - 生怪：`Monster.checkMonsterSpawnRules/checkAnyLightMonsterSpawnRules`（Monster.java:82-90，sky≤nextInt(32)、block 阈值）、`PatrollingMonster`（:87）、`BaseSpawner`（:109，blockLightLimit/skyLightLimit）、`Slime`（:263）、`Bat`（:203）、`GlowSquid`（:89）、`Animal.isBright`（:108）；BIOME 气候类（Biome.java:145,180，biome feature 生成时 block light<10 判定）；POI/biome 内 spawn placement 走同一 getBrightness 接口。
  - 方块 tick：作物类 `CropBlock/StemBlock/SaplingBlock/Bamboo*/SweetBerryBush/PitcherCrop`（rawBrightness≥9/8 生长条件）、`IceBlock/SnowLayerBlock/FrostedIceBlock`（block light>11 融化）、`MushroomBlock`（<13）、`DaylightDetectorBlock`（sky−skyDarken）、`SpreadingSnowyDirtBlock`。
  - 客户端渲染：`LightTexture`（亮度表）、`EntityRenderer/EntityRenderDispatcher/MobRenderer`（实体光照打包）、`ModelBlockRenderer/LevelRenderer`（AO 与方块光照）、`LevelRenderer` 天光判定 :2976-2978。
  - 其他：`LightPredicate`（advancement 条件）、`Gui`（低亮度提示）、`DebugScreenOverlay`、`BiomeAmbientSoundsHandler`、`ArmorStand`（发光）、`EntityRenderer`（slime 大小/光照类判定）。
  - **寻路（navigation）**：grep 全路径 `world/entity/ai/navigation` 无 light 引用——vanilla 1.20.1 寻路不读光照值（draft，仅静态 grep 证据）。
  - 流体 tick：未见直接读光照（水结冰走 Biome.shouldFreeze 的温度+天光 canSeeSky 路径，属于上方 Biome/方块 tick 类）。

## 5. ChunkStatus 管线中 LIGHT 的位置

完整链（ChunkStatus.java:38-115）：
`EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES(range8) → BIOMES(8) → NOISE(8) → SURFACE(8) → CARVERS(8) → FEATURES(8) → INITIALIZE_LIGHT(0) → LIGHT(1, hasLoadDependencies=true) → SPAWN(0) → FULL`

- **INITIALIZE_LIGHT**：parent=FEATURES，range 0；任务=`initializeLight`（:138-143）：initializeLightSources（填 skylight 高度图）+ setLightEngine + initializeLight（enable/retain）。
- **LIGHT**：parent=INITIALIZE_LIGHT，**range=1（需要 1 chunk 半径邻居就位）+ hasLoadDependencies=true（加载路径也必须先跑依赖 status）**；任务=`lightChunk`（:145-148）：若 `!isLighted`（status<LIGHT 或 !lightCorrect）→ propagateLightSources。
- **LIGHT 之前必须**：FEATURES（含 CARVERS/NOISE/…）——方块世界定形；邻居 chunk 至少到 INITIALIZE_LIGHT（range=1 决定）。
- **LIGHT 完成前不能跑**：SPAWN（直接 parent=LIGHT）、FULL；以及一切把 chunk 交给玩家的路径（FULL 才是 LevelChunk）。
- 拦截/接管点判断（draft 观察）：
  - LIGHT 是**管线里唯一通过专用线程 + sorter 邮箱异步执行的 status**（其余 status 任务在 worldgen executor / main thread mailbox，ChunkMap.java:631-774）；其完成信号是 CompletableFuture（initializeLight/lightChunk 返回）回填 ChunkHolder future 链。
  - 重写接管最自然的两个缝：① ChunkStatus.LIGHT 任务本身（返回自己的 future，保持对上契约不变）；② ThreadedLevelLightEngine API 面（initializeLight/lightChunk/queueSectionData/checkBlock/tryScheduleUpdate）——上游 ChunkMap/ChunkStatus 只看这层接口。
  - 存档读取路径的 `isLightOn + lightCorrect` 快捷跳过语义需要保留（或声明有损变更：每次读档全量重算）。

## 6. 有损机会观察（初步，draft，只列不下结论）

1. **全量重算替代增量队列**：增量 FIFO 双队列 + checkNode 差异检测的全部复杂度（QueueEntry 方向位编码、PULL_LIGHT_IN、回拉 increase）只在「单方块修改」场景有价值；生成期管线本来就是对冻结方块一次性点亮——LIGHT status 路径上已是准全量（propagateLightSources）。生成期整片重算 + 运行期小改动局部重算可能是更简单的分解。
2. **skylight 高度图近似**：15 级直下不衰减本质是「高度图以上=15」的柱状模型；`ChunkSkyLightSources`（16x16 高度图 + 边缘遮蔽判定 isEdgeOccluded，含 VoxelShape）已经是主要成本与精度来源——若接受有损，可用 MOTION_BLOCKING heightmap 或「第一个不透明方块」简化 isEdgeOccluded 的 shape 判定。
3. **空 section 穿透批量写**（propagateFromEmptySections）是精确优化；全量重算可以按列一次下推自然获得同效果，不需复刻该函数。
4. **形状遮蔽（VoxelShape face occlusion）**：静态判定的开销大头之一；有损方向=只按 `getLightBlock>0` 判挡，放弃半砖/楼梯类面遮蔽差异。
5. **均质 DataLayer 惰性表示**（data==null + defaultValue）：Rust 侧可用 enum {Uniform(u4), Nibbles(Box<[u8;2048]>)} 直接对应，节省内存且序列化天然跳过空段。
6. **透明度表扁平化**：opacity=getLightBlock 每次走 BlockState 查询；可预展开为 per-blockstate 查找表（数据驱动铁律兼容）。
7. **双缓冲/visible map**：为线程安全服务；若 Rust 侧以 chunk 粒度并行 + 单写者模型，可简化为 per-chunk 锁内直接读写。
8. **队列粒度**：现有 FIFO 每节点 2 long；全量重算下用按层扫描（类 Skyledger 的 15 层 BFS frontier）可能更缓存友好——仅观察。

## 7. 待深入点（建议 Phase 2 worker）

- `LightChunkGetter.getChunkForLighting` 的实现（ChunkMap/LevelChunk 侧）与 lastChunk 2 项缓存的并发边界。
- ChunkHolder → ClientboundLightUpdatePacket 的 section 位图（skyChangedLightSectionFilter）细节，决定有损后客户端兼容面。
- Level.getSkyDarken / dimensionType hasSkyLight 对 skylight 读值的影响链（本次未逐行核对，见 §2.3 标注）。
- ChunkSerializer 是否序列化 ChunkSkyLightSources（§4 标注）。
- 旧版 DynamicGraphMinFixedPoint/LeveledPriorityQueue/SpatialLongSet 是否为死代码（初步：光照路径无引用）。

## 8. 混淆评估

无混淆：Mojmap 命名 + 完整方法体（Forge sources jar 形态），无需 recode.deobfuscate。

—— 产物完（status: draft；全部结论基于静态源码阅读，Degraded 分层，无运行时证据）
