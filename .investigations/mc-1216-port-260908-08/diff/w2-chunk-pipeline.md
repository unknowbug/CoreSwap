# W2 — 1.20.1 vs 1.21.6 chunk 生成管线语义 diff（裁决「1.21.2 重构」实际范围）

置信度：**draft**（静态源码审查，Degraded 分层——无运行时探针）。
对比树：A = versions/1.20.1/data/mc_src_extract，B = versions/1.21.6/data/mc_src_extract。

## 1. ChunkStatus 状态集与语义

**状态集不变**：两版均为 12 个 status，`EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES → BIOMES → NOISE → SURFACE → CARVERS → FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL`，顺序、index、heightmap 类型集（PRE_CARVER→WORLD_GEN_HEIGHTMAP_TYPES、POST_CARVER→NORMAL_HEIGHTMAP_TYPES）一一对应，**仅改名**。

**ChunkStatus 类本身发生质变（接口级/数据级）**：
- A（482 行）：ChunkStatus 内嵌 `GenerationTask` / `LoadTask` lambda、`taskMargin`（单一 int 半径）、`shouldAlwaysUpgrade`、`runGenerationTask` / `runLoadTask` / `byDistanceFromFull` / `DISTANCE_TO_STATUS` / `STATUS_TO_DISTANCE`——**status = 状态 + 任务 + 半径 + 距离映射的全静态单体**。
- B（111 行）：ChunkStatus 是**纯数据记录**（previous/chunkType/heightMapTypes/index + 新增 `CODEC`、`isLaterThan/isAtMost/isEarlierThan/max` 比较器）。任务、半径、距离映射全部外迁。

## 2. 调度模型：从什么变成了什么

**A：全静态模型**。每个 ChunkStatus 自带 GenerationTask（收到 `List<Chunk> chunks`，中心 chunk + taskMargin 半径邻域），调度由 ChunkHolder/ThreadedAnvilChunkStorage 按 status index 逐级 CompletableFuture 链驱动；load 路径用每 status 的 LoadTask（多数为 STATUS_BUMP）。半径 = 每 status 单一 taskMargin（STRUCTURE_REFERENCES/BIOMES/NOISE/SURFACE/CARVERS/FEATURES=8，LIGHT=1，其余 0）。

**B：声明式步骤图 + 显式加载器**（新文件，B 独有）：
- `ChunkGenerationSteps`：两条静态步骤链 `GENERATION`（12 步）与 `LOADING`（12 步，多数为 noop——替代旧 LoadTask 体系）。每步 `ChunkGenerationStep{targetStatus, directDependencies, accumulatedDependencies, blockStateWriteRadius, task}`。
- 依赖从「单一 taskMargin」变为**逐依赖声明** `dependsOn(status, level)`：builder 内按 level+1 展开 directDependencies 数组（每邻域半径一个所需 status），再经 `accumulateDependencies` 逐级累积成 `GenerationDependencies`（status→additionalLevel 查表）。
- 任务体迁至 `ChunkGenerating`（静态方法集，逐条与 A 的 lambda 语义对拍：generateStructures/populateBiomes/populateNoise+BelowZeroRetrogen/buildSurface/carve+createCarvingMasks/generateFeatures+heightmap 预填充+tickLeavesAndFluids/initializeLight/light/generateEntities 全部对应）。
- 调度迁至 `ChunkLoader`（逐 status 推进循环 loadNextStatus，含 `isGenerationUnnecessary` 走 LOADING 链短路）+ `AbstractChunkHolder` + `BoundedRegionArray<AbstractChunkHolder>`（旧 `List<Chunk>` 的结构化替代）。FULL 转换（旧 fullChunkConverter/ChunkHolder future）变成显式步骤 `ChunkGenerating::convertToFullChunk`（mainThreadExecutor supplyAsync + `holder.replaceWith(new WrapperProtoChunk(...))`）。
- `ServerChunkManager`：入口仍是 `getChunk(x,z,leastStatus,create)`，内部改为委托 `ServerChunkLoadingManager`/ChunkLoader（旧版直接管理 ChunkHolder status futures）。

## 3. 对 Rust「按 status 逐阶段生成并拦截」架构的裁决

**定性：接口级重构为主 + 少量语义变化；核心行为（各 status 内做什么、顺序）等价。**

- **算法级**：无。12 个 status 的任务体逐条对拍语义相同（含 Blender、retrogen、heightmap 填充时机）。
- **数据级（实质变化）**：依赖表达从单一半径改为逐依赖表。旧 taskMargin 是「保守最大值」，新 `accumulatedDependencies` 按依赖精确展开。**SPAWN 是明确的语义变化**：旧 taskMargin=0（ChunkRegion radius -1），新 `SPAWN.dependsOn(BIOMES, 1)` → 邻域 ring-1 需要 BIOMES 级 chunk 才能推进 SPAWN——**邻居推进的 barrier 变了**。其余 status 的有效最大半径与旧 taskMargin 相同（8/8/8/8/8/1/0 逐一对上）。
- **boundary 变化清单**：
  1. FULL 转换从「ChunkHolder future 转换器」变为管线上显式最后一步（可在 FULL 前拦截/替换，且经过主线程 executor + WrapperProtoChunk 双重包装路径）；
  2. load 与 generate 分成两条链（LOADING），`isGenerationUnnecessary` 在数据层显式判定；旧版靠 shouldAlwaysUpgrade + LoadTask 隐式处理；
  3. CARVERS 的 carve 调用去掉 `GenerationStep.Carver.AIR` 参数（carver 选择内移到 ChunkGenerator/carver 配置，接口级）；
  4. populateBiomes 去掉 executor 参数（并发治理上移到 ChunkLoader/holder 层）。
- **对 Rust 接管架构的结论**：按 status 拦截的架构**不需要推翻**——status 序列与各阶段语义稳定；需要改的是：(a) 邻居依赖表从「每 status 一个半径」升级为「status×radius 查表」（GenerationDependencies 语义，直接影响预取/依赖推进逻辑）；(b) 若对齐 1.21.6 则 SPAWN 阶段邻居门槛从 0 提到 ring-1@BIOMES；(c) FULL 转换点成为显式可拦截步骤。对「复刻 1.20.1 行为」目标而言**零行为迁移**（此重构不改变 1.20.1 复刻的正确性），仅影响未来升级路径的设计。

## 4. SerializedChunk 与 region 格式载体

- B 新增 `SerializedChunk`（record）替代 A 的 `net.minecraft.world.ChunkSerializer`（A 树无该文件，rg 证实）。NBT 载体**键位兼容**："Status" 字符串键保留（B 读 `nbt.get("Status", ChunkStatus.CODEC)`，写 `putString("Status", id)`），chunkType 判定、heightmap 按 status 类型集写入、`isAtLeast(INITIALIZE_LIGHT)` 光照分支、`setStatus` 回填逻辑逐条对应（SerializedChunk.java L104/110/271-272/414/477 vs ChunkSerializer.java L149/188-190/382-383）。
- **裁决：region 格式对比载体不受影响（数据级内部 API 重排，磁盘格式键位未变）**——但 1.21.6 自身的 NBT 内容差异（biome 4D→容器格式演进等，不在本辖区）另行评估；本结论仅覆盖 status 序列化面。

## 逐处变更分类表

| 变更处 | 分类 | 说明 |
|---|---|---|
| ChunkStatus 瘦身为纯数据（任务/半径/距离表外迁） | 接口级 | 状态集/顺序/heightmap 集不变 |
| ChunkGenerating 任务体 | 装饰级（逐条语义等价）+ 接口级（签名：executor 去、carver 枚举去、ChunkRegion 构造传 step/chunk） | 与 A 的 lambda 对拍通过 |
| ChunkGenerationSteps GENERATION/LOADING 双链 | 接口级 | load 路径显式化，替代旧 LoadTask/shouldAlwaysUpgrade |
| dependsOn(status,level)+GenerationDependencies | 数据级 | 依赖表取代单一 taskMargin |
| SPAWN dependsOn(BIOMES,1) | **数据级（语义变化）** | 旧 margin 0；邻居推进 barrier 变化 |
| ChunkLoader + AbstractChunkHolder + BoundedRegionArray | 接口级 | 调度器重写；ServerChunkManager 委托 ServerChunkLoadingManager |
| FULL 转换显式化（convertToFullChunk + replaceWith(WrapperProtoChunk)） | 接口级（行为边界变化） | 可拦截点位置变化 |
| ChunkStatus 新增 CODEC/比较器（isLaterThan/max 等） | 接口级 | 服务于步骤图声明 |
| SerializedChunk 替代 ChunkSerializer | 接口级（磁盘键位兼容 → 对 region 对比载体无影响） | "Status" 键保留 |
| ProtoChunk（335 vs 322 行） | 装饰级 | status 存取/retrogen/光照分支语义一致 |

## 移植影响裁决

| # | 问题 | 裁决 | 置信度 |
|---|---|---|---|
| 1 | ChunkStatus 状态集/语义变化？ | **状态集与顺序零变化**；类从「状态+任务+半径」单体瘦身为纯数据，任务外迁 ChunkGenerating、依赖外迁 ChunkGenerationSteps | candidate（静态逐行对拍） |
| 2 | 调度模型变化？ | 全静态 ChunkStatus 内嵌任务模型 → 声明式步骤图（GENERATION/LOADING 双链）+ ChunkLoader 显式推进 + GenerationDependencies 依赖查表；FULL 转换入管线 | candidate |
| 3 | 行为等价重排还是语义变化？ | **接口级重排为主**：12 阶段任务体逐条语义等价，各阶段有效半径与旧 taskMargin 一致；**1 处语义变化 = SPAWN 邻居依赖 0→ring-1@BIOMES**；FULL 转换点成为显式可拦截 boundary。对 1.20.1 复刻目标零行为迁移，仅影响升级设计 | candidate |
| 4 | SerializedChunk 影响 region 对比载体？ | **不影响**：NBT "Status" 键与 status 相关读写逻辑键位兼容，属内部 API 重排 | candidate |

**给主线的建议**：Rust 侧 chunk status 状态机保持现设计；若规划 1.21.x 升级，把「邻居依赖表」与「FULL 显式转换步」列为两个移植工作项，SPAWN barrier 差异需运行时验证（本结论 Degraded 分层，未做行为探针）。
