# SURFACE 前/后逐列 dump 判别探针 —— 管线地图 + hook 点清单（recode-scout 勘探）

> 角色：recode-scout（只读勘探，未改任何代码）
> 目标：为「SURFACE 前/后逐列 dump」判别探针（mixin）提供 Java 侧（Minecraft 1.20.1，yarn mappings）的调用链、可 hook 注入点、biome 判定输入、顶面 y 获取方式。
> 背景差异：nether basalt_deltas 第一转换面不同——vanilla 黑石底（y=99 恒平）vs cpp 玄武岩底（y=100~104 贴地形）。四候选：(d) 前置地形形状差 / (a) surface rule 材质分支差 / (b) biome 判定输入差 / (c) surface rule 随机序列差。
> 源码权威：`versions/1.20.1/data/mc_src_extract/`（yarn mappings + sources jar 解包），非 javap 反编译。

---

## 1. surface 规则应用调用链（方法名 + 调用顺序 + 阶段）

### 1.1 阶段总览（MC 1.20.1 世界生成管线）

```
NOISE（populateNoise）→ CARVERS（carve）→ SURFACE（buildSurface）→ FEATURE（generateFeatures）
```

surface 规则应用发生在 **SURFACE 阶段**，由 `NoiseChunkGenerator.buildSurface` 触发。NOISE 阶段（`populateNoise`）已经用 density/aquifer/oreVein 填好了方块 + 高度图（`WORLD_SURFACE_WG` / `OCEAN_FLOOR_WG`），SURFACE 阶段在 NOISE 产物之上「贴皮」。

### 1.2 调用链（实名 + 签名 + 顺序）

| 序 | 方法（yarn 实名） | 签名 | 阶段 | 说明 |
|---|---|---|---|---|
| 1 | `NoiseChunkGenerator.populateNoise(Executor, Blender, NoiseConfig, StructureAccessor, Chunk)` | `CompletableFuture<Chunk>` | NOISE | 填方块 + 高度图（`WORLD_SURFACE_WG`/`OCEAN_FLOOR_WG`），**surface 规则应用前** |
| 2 | `NoiseChunkGenerator.buildSurface(ChunkRegion, StructureAccessor, NoiseConfig, Chunk)` | `void`（public override） | SURFACE | 入口，构造 `HeightContext` 后转调私有重载 |
| 3 | `NoiseChunkGenerator.buildSurface(Chunk, HeightContext, NoiseConfig, StructureAccessor, BiomeAccess, Registry<Biome>, Blender)` | `void`（`@VisibleForTesting` public） | SURFACE | 取 `ChunkNoiseSampler`，调 `SurfaceBuilder.buildSurface` |
| 4 | `SurfaceBuilder.buildSurface(NoiseConfig, BiomeAccess, Registry<Biome>, boolean useLegacyRandom, HeightContext, Chunk, ChunkNoiseSampler, MaterialRules.MaterialRule)` | `void` | SURFACE | **surface 规则实际应用处**（逐列循环） |
| 5 | `MaterialRules.MaterialRule.apply(MaterialRuleContext)` | `BlockStateRule` | SURFACE | 规则树编译成 `BlockStateRule`（一次） |
| 6 | `MaterialRules.BlockStateRule.tryApply(int x, int y, int z)` | `@Nullable BlockState` | SURFACE | 逐块应用规则，返回替换材质或 null |

关键源码位置（`NoiseChunkGenerator.java`）：

- `buildSurface` public 入口（L241-249）：构造 `HeightContext`，调私有重载。
- `buildSurface` 私有重载（L251-276）：`chunk.getOrCreateChunkNoiseSampler(...)` → `noiseConfig.getSurfaceBuilder().buildSurface(noiseConfig, biomeAccess, biomeRegistry, usesLegacyRandom, heightContext, chunk, chunkNoiseSampler, surfaceRule)`。

关键源码位置（`SurfaceBuilder.java`）：

- `buildSurface`（L72-170）：核心逐列循环。
  - L107-110：构造 `MaterialRuleContext`，其中 `posToBiome = biomeAccess::getBiome`（**biome 判定输入来源**）。
  - L110：`materialRule.apply(materialRuleContext)` 编译规则树 → `BlockStateRule`。
  - L113-168：`for k in 0..15, for l in 0..15` 逐列。
    - L117：`o = chunk.sampleHeightmap(WORLD_SURFACE_WG, k, l) + 1`（**顶面 y 来源**）。
    - L119：`registryEntry = biomeAccess.getBiome(mutable2.set(m, useLegacyRandom ? 0 : o, n))`（**该列 biome 判定**）。
    - L125：`materialRuleContext.initHorizontalContext(m, n)`（初始化 runDepth 等水平上下文）。
    - L131-163：自顶向下逐块扫描，`initVerticalContext(...)` 后 `blockStateRule.tryApply(m, u, n)` 应用规则。

### 1.3 关键结论：surface 规则应用「前/后」的边界

- **「前」= NOISE 阶段产物**：`populateNoise` 完成后、`buildSurface` 开始前。此时每列已有：方块序列（density/aquifer/oreVein 产物）+ `WORLD_SURFACE_WG` 高度图 + biome 容器。
- **「后」= SURFACE 阶段产物**：`buildSurface` 完成后。此时每列方块序列被 surface rule 改写（贴皮），高度图可能被 `setState` 更新（`SurfaceBuilder` 内 `blockColumn.setState` 会 `chunk.setBlockState`，但**不更新高度图**——高度图在 NOISE 阶段已定，surface 只改方块不改高度图）。

---

## 2. 可 hook 注入点清单（表格）

> 注入点位置：HEAD = 方法入口（参数已就绪，方法体未执行）；RETURN = 方法返回前（方法体已执行完）；TAIL = 方法体末尾（等价 RETURN 但可拿到局部变量，需 `@At("TAIL")` 且方法非 void 时用 `CallbackInfoReturnable`）。

| # | 注入点方法（yarn 实名） | 注入位置 | 可拿到的上下文 | 建议用途 |
|---|---|---|---|---|
| 1 | `NoiseChunkGenerator.populateNoise(Executor, Blender, NoiseConfig, StructureAccessor, Chunk)` | `@At("RETURN")`（或 TAIL） | `chunk`（含方块 + 高度图 + biome 容器）、`noiseConfig`、`structureAccessor` | **「前」dump**：NOISE 后、SURFACE 前逐列 dump 材质序列 + 顶面 y + biome id |
| 2 | `NoiseChunkGenerator.buildSurface(ChunkRegion, StructureAccessor, NoiseConfig, Chunk)` | `@At("HEAD")` | `region`（含 `getBiomeAccess()`）、`chunk`、`noiseConfig` | **「前」dump 备选**：与 #1 等价，但 `region.getBiomeAccess()` 可直接拿 biome 判定输入 |
| 3 | `NoiseChunkGenerator.buildSurface(ChunkRegion, StructureAccessor, NoiseConfig, Chunk)` | `@At("RETURN")` | `region`、`chunk`（surface 已应用） | **「后」dump**：SURFACE 后逐列 dump 材质序列 + 顶面 y + biome id |
| 4 | `SurfaceBuilder.buildSurface(NoiseConfig, BiomeAccess, Registry<Biome>, boolean, HeightContext, Chunk, ChunkNoiseSampler, MaterialRule)` | `@At("HEAD")` | `biomeAccess`（**直接 biome 判定输入**）、`chunk`、`chunkNoiseSampler`、`materialRule`、`heightContext` | **「前」dump 最精确**：能拿到 `biomeAccess` 与 `chunkNoiseSampler`，可复刻 `biomeAccess.getBiome(pos)` 与 `chunkNoiseSampler.estimateSurfaceHeight(x,z)` |
| 5 | `SurfaceBuilder.buildSurface(...)` | `@At("RETURN")` | `chunk`（surface 已应用）、`biomeAccess` | **「后」dump 最精确**：与 #4 对称 |
| 6 | `MaterialRules.BlockStateRule.tryApply(int x, int y, int z)` | `@At("HEAD")` / `@At("RETURN")` | `x,y,z`、`this`（规则树节点）、返回 `BlockState`（RETURN 时） | **逐块材质分支 dump**：直接观察 surface rule 在每块返回什么材质（候选 (a) 材质分支差 / (c) 随机序列差） |
| 7 | `MaterialRules.MaterialRuleContext.initVerticalContext(int stoneDepthAbove, int stoneDepthBelow, int fluidHeight, int blockX, int blockY, int blockZ)` | `@At("HEAD")` | `blockX/blockY/blockZ`、`stoneDepthAbove/Below`、`fluidHeight`、`this.biomeSupplier`（init 后） | **逐块上下文 dump**：观察 stoneDepth/fluidHeight/biome 判定输入（候选 (b) biome 输入差） |
| 8 | `BiomeAccess.getBiome(BlockPos)` | `@At("HEAD")` / `@At("RETURN")` | `pos`、返回 `RegistryEntry<Biome>`（RETURN 时） | **biome 判定输入 dump**：直接观察每列 biome 判定结果（候选 (b)） |

> 注意：`MaterialRules.BlockStateRule` 是 `protected interface`，`tryApply` 是接口方法，mixin 需 `@Mixin` 到具体实现类（如 `MaterialRules$SequenceBlockStateRule`、`MaterialRules$ConditionalBlockStateRule`、`MaterialRules$SimpleBlockStateRule`），或对接口用 `@Mixin` + `@Inject` 到接口方法（需 `@Mixin(MaterialRules.BlockStateRule.class)`，但接口 mixin 在 1.20.1 需确认 loom 支持）。**更稳妥**：hook `SurfaceBuilder.buildSurface` 的 HEAD/RETURN（#4/#5），在回调里自己遍历列 + 调 `biomeAccess.getBiome` + `chunk.sampleHeightmap`，完全复刻 `SurfaceBuilder` 内部逻辑，不依赖接口 mixin。

---

## 3. biome 判定输入获取方式

### 3.1 surface 规则应用时的 biome 判定链

1. `SurfaceBuilder.buildSurface` L119：`registryEntry = biomeAccess.getBiome(mutable2.set(m, useLegacyRandom ? 0 : o, n))`——**每列 biome 判定入口**，`o = sampleHeightmap(WORLD_SURFACE_WG)+1` 是顶面 y。
2. `biomeAccess` 来自 `NoiseChunkGenerator.buildSurface` 传入的 `region.getBiomeAccess()`（`ChunkRegion` 的 `BiomeAccess`）。
3. `BiomeAccess.getBiome(BlockPos)`（`BiomeAccess.java` L30-64）：3D 最近邻（8 邻域）biome 插值，`pos.getX()-2` 等偏移后 `>>2` 到 biome 坐标，`method_38106` 算距离，取最近邻 `storage.getBiomeForNoiseGen(px, w, x)`。
4. `MaterialRuleContext.initVerticalContext` L464：`biomeSupplier = Suppliers.memoize(() -> posToBiome.apply(pos.set(blockX, blockY, blockZ)))`——**surface rule 内部（`biome_is` 条件）用的 biome 判定**，`posToBiome = biomeAccess::getBiome`（L108 传入）。

### 3.2 在探针里拿「该列当前 biome id」的方式

- **方式 A（推荐，hook #4/#5）**：在 `SurfaceBuilder.buildSurface` 回调里，用入参 `biomeAccess` 直接调 `biomeAccess.getBiome(new BlockPos(x, o, z))`，`o = chunk.sampleHeightmap(WORLD_SURFACE_WG, k, l) + 1`。返回 `RegistryEntry<Biome>`，`getKey().map(k -> k.getValue().toString())` 得 biome id（参考 `DiagFeatureBiomeMixin.wgBiomeId`）。
- **方式 B（hook #2/#3）**：`region.getBiomeAccess().getBiome(pos)`，等价。
- **方式 C（hook #8）**：直接 `@Inject` `BiomeAccess.getBiome(BlockPos)` 的 HEAD/RETURN，dump 每次 biome 判定（含 surface 规则内部 `biome_is` 条件的判定）。

> 关键：`useLegacyRandom` 决定 biome 判定的 y 坐标——`useLegacyRandom ? 0 : o`。nether 的 `usesLegacyRandom()` 需确认（`ChunkGeneratorSettings.usesLegacyRandom()`），若为 true 则 biome 判定用 y=0（恒平），否则用顶面 y（贴地形）。**这直接影响候选 (b) biome 判定输入差**——若 vanilla 用 y=0 而 cpp 用顶面 y，biome 判定输入就不同。

---

## 4. 顶面 y 获取方式

### 4.1 surface 规则应用「前」的顶面 y（NOISE/density 阶段产物）

- **主来源：`chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l)`**——`SurfaceBuilder.buildSurface` L117/L124 正是用它拿顶面 y（`o = ... + 1`）。
- 高度图在 NOISE 阶段（`populateNoise` L363-364、L417-418）由 `heightmap.trackUpdate` / `heightmap2.trackUpdate` 填充，是 density/aquifer/oreVein 产物的「最高非空气方块 y」。
- **不是** `ChunkNoiseSampler` 的字段——`ChunkNoiseSampler` 有 `estimateSurfaceHeight(blockX, blockZ)`（L222-240，基于 `initialDensityWithoutJaggedness > 0.390625` 的**估计**值，非精确顶面），但 surface 规则实际用的是 `chunk.sampleHeightmap(WORLD_SURFACE_WG)`。

### 4.2 在探针里拿顶面 y 的方式

- **方式 A（推荐）**：`chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l)`，k/l 是 chunk 内 0..15 列坐标。返回该列最高非空气方块 y（不含 +1）。
- **方式 B（估计值，仅参考）**：`chunkNoiseSampler.estimateSurfaceHeight(x, z)`（hook #4 能拿到 `chunkNoiseSampler`），但这是 density 估计值，与 `WORLD_SURFACE_WG` 高度图可能不同（尤其 aquifer/oreVein 影响时）。
- **方式 C（逐块）**：`chunk.getBlockState(mutable.setY(y))` 自顶向下扫，找第一个非空气方块（`ColProfProbeMixin` 的 `context.getTopY(MOTION_BLOCKING, x, z)` 是 feature 阶段的等价物，surface 阶段用 `WORLD_SURFACE_WG`）。

> 关键：**「前」dump 的顶面 y 必须用 `WORLD_SURFACE_WG` 高度图**（与 `SurfaceBuilder` 内部一致），不能用 `MOTION_BLOCKING`（那是 feature 阶段的高度图，且 surface 规则应用后才会被 feature 更新）。

---

## 5. 现有 mixin 清单（hook 风格参考）

目录：`runtime/1.20.1/java/src/main/java/wg/bench/mixin/`，注册于 `runtime/1.20.1/java/src/main/resources/coreswap.mixins.json`。

| mixin 文件 | @Mixin 目标 | hook 方法 | 注入点 | 用途 |
|---|---|---|---|---|
| `NoiseChunkGeneratorMixin.java` | `NoiseChunkGenerator` | `populateNoise(Executor,Blender,NoiseConfig,StructureAccessor,Chunk)` | HEAD（cancellable） | NOISE 阶段 C++ 接管（overworld/nether） |
| `NoiseChunkGeneratorMixin.java` | `NoiseChunkGenerator` | `buildSurface(ChunkRegion,StructureAccessor,NoiseConfig,Chunk)` | HEAD（cancellable） | SURFACE 阶段 C++ 接管（跳过 Java surface） |
| `DiagFeatureBiomeMixin.java` | `ChunkGenerator` | `generateFeatures(StructureWorldAccess,Chunk,StructureAccessor)` | HEAD | feature 阶段 biome 上下文 dump（M14） |
| `BlobProbeMixin.java` | `PlacedFeature` | `generate(StructureWorldAccess,ChunkGenerator,Random,BlockPos)` | HEAD | blob 放置 origin dump（H1） |
| `ConfiguredFeatureProbeMixin.java` | `ConfiguredFeature` | `generate(StructureWorldAccess,ChunkGenerator,Random,BlockPos)` | HEAD | configured feature 放置点 dump（H1） |
| `ColProfProbeMixin.java` | `CountMultilayerPlacementModifier` | `getPositions(FeaturePlacementContext,Random,BlockPos)` | HEAD | 首分叉邻域列转换面序列 dump（H1 终审 (e)） |

### 5.1 hook 风格要点（从现有 mixin 提炼）

1. **门控**：`@Unique private static final boolean XXX = System.getProperty("xxx.probe") != null;`，回调首行 `if (!XXX) return;`（零开销短路）。
2. **方法匹配**：带完整描述符（`method = "generate(Lnet/minecraft/world/StructureWorldAccess;...)"`），`require = 1`（fail-fast，见 `BlobProbeMixin` 注释「yarn 1.20.1 PlacedFeature 实名方法为 generate 非 place」踩坑）。
3. **只读性**：不碰 Random、不改世界、不消费随机序列——纯旁观 dump（`ColProfProbeMixin` 注释明确）。
4. **输出**：`System.out.println("[TAG] ...")` 或写文件（`BlobProbeMixin` 用 `Files.write` 到 `.tmp/`）。
5. **异常兜底**：`try { ... } catch (Throwable t) { System.out.println("[TAG] failed: " + t); }`。
6. **反射拿父类字段**：`NoiseChunkGeneratorMixin.wgIsEnd()` / `DiagFeatureBiomeMixin.wgDiagBiomeSource()` 用反射读 `ChunkGenerator.biomeSource`（@Shadow 够不到时）。
7. **mixin 限制**：禁止非 private 静态成员（`BlobProbeMixin` 注释「本版本连 @Unique public static 方法都拒」）。

---

## 6. 架构变更建议（交主会话裁决）

1. **探针形态建议**：新建 `SurfaceDumpProbeMixin.java`，`@Mixin(SurfaceBuilder.class)`，hook `buildSurface` 的 HEAD（「前」dump）与 RETURN（「后」dump），在回调里复刻 `SurfaceBuilder` 内部逐列逻辑（`chunk.sampleHeightmap(WORLD_SURFACE_WG)` + `biomeAccess.getBiome` + 自顶向下扫方块），dump 每列：材质序列（raw id 序列）+ biome id + 顶面 y。**不依赖接口 mixin**（`BlockStateRule` 是 protected interface，接口 mixin 有风险）。
2. **「前」dump 的顶面 y 语义**：必须用 `WORLD_SURFACE_WG`（与 `SurfaceBuilder` 一致），不能用 `MOTION_BLOCKING`。
3. **biome 判定 y 坐标**：需先确认 nether `usesLegacyRandom()` 的值——它决定 biome 判定用 y=0 还是顶面 y，是候选 (b) 的关键分叉点。建议探针同时 dump `useLegacyRandom` 标志 + 实际 biome 判定用的 y 坐标。
4. **四候选判别映射**：
   - (d) 前置地形形状差 → 「前」dump 的顶面 y / 材质序列在 vanilla vs cpp 是否已不同（若「前」已不同，则根因在 NOISE/density，surface 只是继承）。
   - (a) 材质分支差 → 「后」dump 的材质序列 vs 「前」dump 的材质序列，看 surface rule 改写了哪些块。
   - (b) biome 判定输入差 → hook `BiomeAccess.getBiome` 或 dump `biomeAccess.getBiome(pos)` 结果，对比 vanilla vs cpp 的 biome id。
   - (c) 随机序列差 → hook `MaterialRules.BlockStateRule.tryApply` 或 dump `sampleRunDepth`/`sampleSecondaryDepth` 的随机值（`SurfaceBuilder.sampleRunDepth` L172-175 用 `randomDeriver.split(blockX,0,blockZ).nextDouble()`）。
5. **注意 C++ 接管干扰**：`NoiseChunkGeneratorMixin` 在 `-Dcpp.replace=1` 时会 cancel `populateNoise` 和 `buildSurface`（C++ 接管），**探针必须在 vanilla 模式（不启用 cpp.replace）下运行**，否则 hook 不到 vanilla 的 surface 流程。探针应加维度过滤（nether）与 chunk 坐标过滤（对齐 `ColProfProbeMixin` 的 `-Dcolprof.x/z/r` 风格）。

---

## 附：关键源码文件路径（勘探依据）

- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java`（buildSurface L241-276、populateNoise L329-435）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/surfacebuilder/SurfaceBuilder.java`（buildSurface L72-170、sampleRunDepth L172-175）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/surfacebuilder/MaterialRules.java`（MaterialRuleContext L404-573、BlockStateRule L243-246、BiomeMaterialCondition L170-217）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/ChunkNoiseSampler.java`（estimateSurfaceHeight L222-240）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/biome/source/BiomeAccess.java`（getBiome L30-64）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/HeightContext.java`
- 现有 mixin：`runtime/1.20.1/java/src/main/java/wg/bench/mixin/*.java` + `runtime/1.20.1/java/src/main/resources/coreswap.mixins.json`
