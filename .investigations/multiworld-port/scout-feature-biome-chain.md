# scout 勘探产物：feature 装饰链 × biome 上下文依赖地图（M14，假设非结论）

- 角色：recode.scout（只读勘探，未修改任何工程文件）
- 日期：2026-11
- 证据来源：
  - 项目 mixin：`runtime\1.20.1\java\src\main\java\wg\bench\mixin\NoiseChunkGeneratorMixin.java`（全文 99 行已读）
  - 项目桥：`runtime\1.20.1\java\src\main\java\wg\bench\CppBridge.java`（writeChunk/fillChunkNether/initNether 段）
  - vanilla：gradle loom 缓存 yarn 命名 jar
    `C:\Users\NDark\.gradle\caches\fabric-loom\minecraftMaven\net\minecraft\minecraft-merged\1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2\minecraft-merged-1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2.jar`
    （⚠️ **workspace 内无完整 yarn sources**：loom genSources 只留 vineflower 增量缓存 `decompile\v1.zip`（哈希名，不可读）；本产物 vanilla 证据为 **javap -c 字节码**，行号不可得，一律给「类#方法 + 字节码 offset」。可读 yarn 行号仅 `runtime\1.20.1\java\NoiseChunkGenerator.java`（工作区本地拷贝））
  - 台账：`.investigations\multiworld-port\multiworld-errors.md` L462-546（M14）

---

## 1. 项目 mixin 现状摘要（接管点清单）

文件：`NoiseChunkGeneratorMixin.java`（`@Mixin(NoiseChunkGenerator.class)`，coreswap.mixins.json 注册）

| # | 接管点 | 行号 | 行为 | 后续阶段上下文刷新？ |
|---|--------|------|------|---------------------|
| 1 | `populateNoise(Executor,Blender,NoiseConfig,StructureAccessor,Chunk)` @HEAD cancellable | L41-78 | 主世界形状（bottomY=-64,h=384）→ `CppBridge.feedBeardifier`+`fillChunk`+`cir.setReturnValue(completedFuture(chunk))`（L68）；nether 形状（0/256）且非 End 且 `netherActive()` → `fillChunkNether`（L72-77） | **无任何刷新**——不调 `setBiome`/`populateBiomes`/`BlendingData`/`setStatus`（grep 全源码 0 命中） |
| 2 | `buildSurface(ChunkRegion,StructureAccessor,NoiseConfig,Chunk)` @HEAD cancel | L82-98 | 主世界或（nether 形状+非 End+句柄就绪）→ `ci.cancel()`，表面规则交给 C++ fill 内部 | 同上 |

维度分派机制（任务 5 关键事实）：**mixin 只按 chunk 形状分派，无 RegistryKey/维度判断**（L57-60, L90-93；End 用 `biomeSource instanceof TheEndBiomeSource` 反射排除，L27-38）。但 nether ServerWorld 本身用 vanilla nether NoiseChunkGenerator（own biomeSource=MultiNoiseBiomeSource nether / own NoiseConfig），populateNoise 拦截只替换方块层；**feature 阶段没有任何 mixin/分支**，仍按 vanilla 在 nether ServerWorld 内跑——`BenchMod.java` L32 只调 `CppBridge.initNether(server.getOverworld().getSeed())` 建 Rust 句柄，nether 维度创建是 vanilla 流程。
`CppBridge.writeChunk`（L267-304）只 `sec.setBlockState` 写方块 + 补 heightmap，**不触碰 ChunkSection 的 biome 容器，也不建新 section**（L270 复用现有 section）。

## 2. Status 推进链（任务 3）——cancel 不会卡 Status

- `ChunkStatus.runGenerationTask`（javap：`net.minecraft.world.chunk.ChunkStatus#runGenerationTask`）：`generationTask.doWork(...)` → `thenApply(method_51374)` → `method_52270(Chunk)`：`if (ProtoChunk.getStatus().isAtLeast(this) 为假) → ProtoChunk.setStatus(this)`。
- **setStatus 在 ChunkStatus 包装层做，与 generate 方法内部是否 cancel 无关**。mixin `cir.setReturnValue(completedFuture(chunk))` 只替换 populateNoise 的返回值，`doWork` 返回的 future 照常完成 → status 照常推进 NOISE→SURFACE→CARVERS→FEATURES。
- FEATURES 任务 = `ChunkStatus#method_51375`（`SimpleGenerationTask`，javap 证据）：`Heightmap.populateHeightmaps(MOTION_BLOCKING/MOTION_BLOCKING_NO_LEAVES/OCEAN_FLOOR/WORLD_SURFACE)` → `new ChunkRegion(serverWorld, chunks, ChunkStatus.FEATURES, placementRadius=1)` → `chunkGenerator.generateFeatures(region, chunk, structureAccessor.forRegion(region))` → `Blender.tickLeavesAndFluids`。
- 结论：**「Status 停在 biomes」不成立**（证据：method_52270 字节码 offset 11-25，setStatus 由 isAtLeast 门控 + 状态机在 ChunkStatus 层）。
- 结构放置：`StructurePlacementCalculator` 由 `ChunkGenerator.createStructurePlacementCalculator`（STRUCTURE_STARTS 阶段前）建立，与 NOISE cancel 无关。

## 3. feature 装饰链 biome 上下文每一跳（任务 2 依赖地图）

1. `ChunkStatus.FEATURES` task（`ChunkStatus#method_51375`，3×3 chunk list，radius=1）
2. → `ChunkGenerator.generateFeatures(StructureWorldAccess,Chunk,StructureAccessor)`（javap，yarn 名 1.20.1 **是 generateFeatures，不是 addFeatures/applyBiomeDecoration**——后者是旧版/Mojang 名）：
   - **跳 A（feature 候选集来源）**：bytecode offset 141-177：`ChunkPos.stream(centerPos, 1)` 3×3 → **lambda `ChunkGenerator#method_39787`**：对每个邻 chunk `getChunk(x,z).getSectionArray()` → **`ChunkSection.getBiomeContainer().forEachValue(set::add)`——直接收集邻 chunk biome 容器里的全部 biome 条目** → `set.retainAll(biomeSource.getBiomes())`（offset 163-177，biomeSource = 当前 ServerWorld 的 ChunkGenerator 字段）。
     ⚠️ **feature 集由 3×3 邻 chunk biome 容器的「并集」决定**，不是中心 chunk 单点。
   - **跳 B（每 step 的 feature 索引）**：`indexedFeaturesListSupplier`（构造时由 `method_44215(biomeSource, generationSettingsGetter)` 预展开）+ 每 biome 的 `GenerationSettings.getFeatures().get(step)`（offset 446-484）。
   - **跳 C（放置时逐点 biome 过滤）**：`PlacedFeature.generate(world=ChunkRegion, generator, random, originPos)`（offset 643-653）；放置链内 `BiomeFilter`/feature 内部用 `world.getBiome(pos)`。
3. → `ChunkRegion.getBiome(blockPos)`（`WorldView` 默认）→ `ChunkRegion.biomeAccess`（构造 offset 198-214，`new BiomeAccess(storage=this, hashSeed(world.getSeed()))`）→ `BiomeAccess.getBiome` → **storage = `ChunkRegion#getGeneratorStoredBiome`（javap：`ServerWorld.getGeneratorStoredBiome(bx,by,bz)`）→ 邻 chunk 的 biome 容器（4×4×4 cell）**。
4. → `Chunk.populateBiomes(BiomeSupplier, sampler)` 写入 biome 容器发生在 **BIOMES status**（先于 NOISE，未被 mixin 触碰）：`NoiseChunkGenerator.populateBiomes`（工作区 yarn 拷贝 `runtime\1.20.1\java\NoiseChunkGenerator.java` L87-100：`chunk.populateBiomes(biomeSupplier, sampler.createMultiNoiseSampler(noiseConfig.getNoiseRouter(), ...))`，biomeSupplier = `this.biomeSource`+blender 包装）。
5. NoiseConfig 来源：`ChunkRegion` 构造 offset 158-187：`world.getChunkManager().getNoiseConfig()`（**per-ServerWorld 实例**）；mixin 未触碰。
6. CARVERS 阶段（对比参考，yarn 拷贝 L279-327）：biome 用 `biomeSource.getBiome(..., noiseConfig.getMultiNoiseSampler())` 直接重采样 + `chunk.getOrCreateGenerationSettings(...)` 缓存——与 FEATURES 的「读容器」路径不同。

### 与 M14 现象的对接点
森林 feature 签名（oak_leaves/sapling）进入 nether chunk 的机制通道只有两条：跳 A（3×3 邻 chunk biome 容器并集混入 overworld biome 条目）或跳 C 前的 feature 候选集已在 overworld 语境展开（即该 chunk 实际由 overworld ServerWorld/generator 走流程）。`note_block` 非常规 feature 块，待 worker 解释（可疑：Rust id→state 映射误差或 feature 随机 putSchematic 类）。

## 4. 三候选验证探针（主会话可执行模板；subagent 不执行）

命令基座：`pwsh E:\PYTHON\CoreSwap\runtime\1.20.1\java; .\gradlew runServer`（探针一律加 JVM prop `-Dxxx=1`；先 `Stop-Process -Name java` 清残留 + 备份/删 `run\world*`，遵守探针三查铁律：核对 seed 三处）。

### 候选 a：chunk biome 容器在 Rust fill 后未刷新/被写错（含「3×3 并集」变体）
- 挂点：`CppBridge.fillChunkNether` 末尾 + 新临时 mixin `ChunkGenerator.generateFeatures` HEAD：dump 中心 chunk 与 3×3 邻 chunk 每个非空 section 的 `getBiomeContainer` 唯一值（反射/`forEachValue`），打印 `chunk(cx,cz) step=FEATURES biomes=[...]`。
- 判据：FEATURES 时 nether 3×3 内出现 forest/plains 类 overworld biome 条目 → 候选 a 成立并定位到具体邻 chunk。
- 命令模板：`.\gradlew runServer "-Dcpp.replace=1" "-Dwg.dumpbiome=1"`（dumpbiome 代码由 worker 产出 patch 后主会话应用编译）。

### 候选 b：NoiseConfig/climate 采样跨维度污染
- 挂点：mixin `wgPopulateNoise` 入口 + `populateBiomes`（反射断点）：打印 `System.identityHashCode(noiseConfig)`、`noiseConfig.hashCode()`、`world(=chunk 所属 world).getRegistryKey()`。
- 判据：同一 chunk 的 BIOMES 与 FEATURES 两跳 noiseConfig 同一实例且属于 nether world → 排除 b；若 nether chunk 携带 overworld NoiseConfig → b 成立。
- 命令模板：`.\gradlew runServer "-Dcpp.replace=1" "-Dwg.dumpnoise=1"`。

### 候选 c：ChunkRegion 装饰 accessor 错误（读错 world / 邻 chunk 越区）
- 挂点：mixin `ChunkGenerator.generateFeatures` HEAD（或断点）：打印 `((ChunkRegion)world).getWorld().getRegistryKey()`（应为 the_nether）、`region.getCenterPos()`、跳 A 收集集（`world.getBiome(邻chunk.getStartPos())` 快照）、`chunk.getStatus()`。
- 判据：`getRegistryKey() != the_nether`（chunk 被路由进错 world）→ 根因=维度路由缺失；registryKey 正确但 biome 集混入 overworld → 回候选 a 定位邻 chunk。
- 命令模板：同 a（合并进 `-Dwg.dumpbiome`）。

### 维度分派审计（任务 5 专项）
- mixin 无维度判断只有形状判断：实机验证「nether chunk 的 populateNoise 确实进 nether 分支」——`[Mixin] populateNoise(nether) intercepted` 日志与 chunk 坐标对账（已有日志，采集 `run/logs/latest.log` grep `[Mixin]`，核对 -5,-3 出现在 nether 分支且无主世界分支重复拦截）。
- 若 -5,-3 同时/只出现在 overworld 分支 → mixin 形状分派错位 = 根因候选 #0（重排嫌疑首位）。

## 5. 初步嫌疑排序（假设，非结论）

1. **H1（跳 A）**：`generateFeatures` 用 3×3 邻 chunk biome 容器并集选 feature 集（method_39787 证据）——任何一个邻 chunk 的 biome 容器含 overworld 条目，森林 feature 即被排入本 nether chunk；结合 F3 只看中心点，与「中心 biome 正确但森林块成片」现象自洽。待查：overworld 条目如何进 nether 邻 chunk 容器（候选 a 的下位机制）。
2. **H0（维度路由）**：mixin 形状分派正确性未实证——若 -5,-3 曾被 overworld 分支拦截/或该 chunk 生成期穿过 overworld 语境，feature 候选集整体错位。查日志即可低成本排除/确认（优先做）。
3. **H-a'（section 重建默认值）**：ChunkSection 新构造 biome 容器默认 plains（yarn 语义）——若某路径（retrogen/section 重建）换新 section，biome 变 plains → overworld 普通特征集。未在 mixin 路径发现直接触发点，列观察项。
4. **H-b（NoiseConfig 污染）**：现证据均指向 per-world 实例（ChunkRegion 构造 offset 158-187），最弱嫌疑。

## 待深入点（交 worker）
- note_block 从哪个 feature/pool 来（含 Rust STATE_BY_ID 映射误差复查）。
- BIOMES status 是否可能对 nether chunk 使用 overworld 的 biomeSupplier（检查 ChunkStatus.BIOMES task 的 generator 归属）。
- 跳 A 的 `getChunk(x,z)`（WorldView 默认按 ChunkStatus.FEATURES 门控）对未到 NOISE 的邻 chunk 行为（读旧存档 or 加载现算）。
