---
status: draft
task: MC 1.20.1 FEATURE 阶段（CARVERS + FEATURES）完整管线地图
created: 2026-xx-xx
source: mc_src_extract (yarn mappings 1.20.1) + worldgen JSON data + cpp/worldgen 现状
scope: 供 CoreSwap C++ 复刻 CARVERS→FEATURES 阶段使用
---

# MC 1.20.1 worldgen 管线地图：CARVERS + FEATURES

背景（引用，不再深挖）：ChunkStatus 链 `NOISE → SURFACE → CARVERS → FEATURES`。
- CARVERS = `setCarverSeed(seed+l,cx,cz) → shouldCarve → CaveCarver/RavineCarver → CarvingMask → carveRegion 逐点 aquifer.apply + materialRule 补丁`
- FEATURES = `setPopulationSeed → 按 step k 遍历 biome features 列表 → setDecoratorSeed(l,p,k) → PlacedFeature.generate(placementModifiers flatMap) → ConfiguredFeature.generate`

> **⚠ 两个阶段随机数基类不同（复刻最易错点）**
> - CARVERS：`new ChunkRandom(new CheckedRandom(RandomSeed.getSeed()))`（CheckedRandom = Java LCG 48 位，`CheckedRandom.java`）
> - FEATURES：`new ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))`（Xoroshiro128PlusPlus，C++ `random.h` 已有）
> - 两者调用 `setSeed()` 时对 worldSeed 的消化不同（LCG 截断 48 位 / Xoroshiro createXoroshiroSeed），必须分别实现。

---

## 1. CARVERS 数据流

### 1.1 入口链

**Java 位置**：`net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:278-327`（`carve()`）
**机制**：
1. `biomeAccess.withSource(...)`：biome 采样改用 `biomeSource.getBiome(x,y,z, multiNoiseSampler)`（4 倍 block 采样在 BiomeAccess 层）。
2. `ChunkRandom chunkRandom = new ChunkRandom(new CheckedRandom(RandomSeed.getSeed()))` —— **基类是 CheckedRandom（LCG）**。
3. 取 `chunkNoiseSampler`（`chunk.getOrCreateChunkNoiseSampler`）、`aquiferSampler = chunkNoiseSampler.getAquiferSampler()`。
4. 构造 `CarverContext(this, registryManager, chunk.getHeightLimitView(), chunkNoiseSampler, noiseConfig, settings.value().surfaceRule())`。
5. `CarvingMask carvingMask = ((ProtoChunk)chunk).getOrCreateCarvingMask(carverStep)` —— **按 carverStep（AIR/LIQUID）维度各一个 mask**（下见 1.4）。
6. 双循环 `j,k ∈ [-8,8]`（17×17 邻域 chunk）：
   - 对每个邻域 chunk 用 biome 查 `GenerationSettings`（`chunk2.getOrCreateGenerationSettings`，用 `getBiome(startX, 0, startZ)` 选 2×2 邻域中最近群系；实际 Java 取的是 biomeCoords 采样一次）
   - `iterable = generationSettings.getCarversForStep(carverStep)`（`biome/GenerationSettings.java:72-74`，即 `carvers.air` 列表；`carvers` 是 `Map<GenerationStep.Carver, List<ConfiguredCarver>>`）
   - 遍历每个 carver，`l` 从 0 递增：
     - `chunkRandom.setCarverSeed(seed + l, chunkPos2.x, chunkPos2.z)`
     - `if (configuredCarver.shouldCarve(chunkRandom)) configuredCarver.carve(...)`（`ConfiguredCarver.java:24-40`，内部先 `SharedConstants.isOutsideGenerationArea` 检查再转发 `carver.carve`）
7. **种子公式**（`util/math/random/ChunkRandom.java:87-93`）：
   ```
   setSeed(worldSeed);
   l = nextLong(); m = nextLong();
   n = chunkX * l ^ chunkZ * m ^ worldSeed;
   setSeed(n);
   ```
   `nextLong()` = `next(32)<<32 | next(32)`（Java Random 语义；CheckedRandom 下两次 LCG 高 32 位）。

**C++ 复刻要点**：
- 需新增 `CheckedRandom`（48 位 LCG：`seed = (seed * 0x5DEECE66D + 0xB) & ((1<<48)-1)`；`next(bits)` = `seed >>> (48-bits)`；`setSeed` 截断低 48 位）。
- 需新增 `ChunkRandom` 包装类：持有 baseRandom 指针，`next(bits)` 转发（CheckedRandom 走 `next(bits)`；Xoroshiro 走 `(int)(baseRandom.nextLong() >>> 64-bits)`）。
- `setCarverSeed` 精确移植（含 `seed + l` 的 l 为 carver 列表序号）。
- 邻域 17×17 循环 + 每邻域查 biome → GenerationSettings（C++ 需要加载 biome JSON 的 `carvers`，见第 3 节）。

### 1.2 CarverContext

**Java 位置**：`carver/CarverContext.java:18-47`
**字段**：`registryManager`、`chunkNoiseSampler`（密度/含水层采样）、`noiseConfig`、`materialRule`（surfaceRule）。
**关键方法**：`applyMaterialRule(posToBiome, chunk, pos, hasFluid)` → `noiseConfig.getSurfaceBuilder().applyMaterialRule(...)`，用于 carveAtPoint 把被挖掉的草方块下方的 dirt 按 surface rule 补丁（如砂土/菌丝补丁）。

**C++ 复刻要点**：C++ `surface.h` 的 `buildSurface` 已实现 surface rule 全量求值；需暴露单点 `applyMaterialRule(x,y,z,biomeAt)` 入口供 carver 复用（Java 侧同样是把 materialRule 应用到单个 pos）。

### 1.3 CarvingMask

**Java 位置**：`carver/CarvingMask.java`；`ProtoChunk.getOrCreateCarvingMask`（`world/chunk/ProtoChunk.java`，per `GenerationStep.Carver` 维度）
**机制**：
- `BitSet(256 * height)`；`getIndex = (x&15) | (z&15)<<4 | (y-bottomY)<<8`。
- `set/get(offsetX, y, offsetZ)`；`get` 还叠加 `maskPredicate`（FEATURES 阶段 lava 检查用）。
- `streamBlockPos(chunkPos)`：把 mask 中的位还原为世界 BlockPos（被 `CarvingMaskPlacementModifier` 使用）。

**C++ 复刻要点**：`std::vector<uint64_t>`（或 `std::bitset<256*height>`），`std::vector<CarvingMask>` 按 carverStep 索引（AIR=0/LIQUID=1），存于 per-chunk 生成上下文。注意 384 高度时 256*384=98304 bit ≈ 12KB/mask/step。

### 1.4 carveRegion 逐点雕刻

**Java 位置**：`carver/Carver.java:59-117`（`carveRegion`）、`119-162`（`carveAtPoint`）、`164-176`（`getState`）、`206-214`（`canCarveBranch`）
**机制（carveRegion）**：
- 边界检查：`|x - chunkCenterX| > 16 + width*2` 或 z 同理 → 直接 false（该洞穴体不触及本 chunk）。
- 计算局部遍历范围：`k..l`（x）、`m..n`（y，clamp 到 `minY+1 .. minY+height-1-n`，n=0 若 belowZeroRetrogen 否则 7）、`p..q`（z）。
- 归一化坐标 `g=(s+0.5-x)/width`、`h=(u+0.5-z)/width`、`w=(v-0.5-y)/height`；`g²+h² >= 1.0` 跳过（圆形截面）。
- y 从 `o` 递减到 `m`；条件 `!skipPredicate.shouldSkip(...) && (!mask.get || isDebug)` → `mask.set` + `carveAtPoint`。

**机制（carveAtPoint）**：
- 记录 `replacedGrassy`：原方块是 grass_block / mycelium。
- `canAlwaysCarveBlock`（方块 ∈ config.replaceable 标签）否则返回 false（debug 模式例外）。
- `getState`（见 1.5）；null → 不雕刻。
- `chunk.setBlockState(pos, state, false)`；`aquifer.needsFluidTick && 流体非空` → `markBlockForPostProcessing`。
- `replacedGrassy && 下方是 dirt` → `context.applyMaterialRule(...)` 补丁（hasFluid = 新 state 流体非空）。

**机制（canCarveBranch）**（`Carver.java:206-214`）：
```
d = pos.getCenterX(); e = pos.getCenterZ();
f = x - d; g = z - e; h = branchCount - branchIndex;
i = baseWidth + 2.0 + 16.0;
return f*f + g*g - h*h <= i*i;
```
（隧道延续性检查：距 chunk 中心越远允许的横向偏差越小，与剩余分支数耦合）

**C++ 复刻要点**：三重循环 + 归一化判断逐点写 `col`（C++ 的 BlockColumn 等效于 Java Chunk+ChunkSection）；`getState` 直接调 C++ `Aquifer::apply`；materialRule 补丁调 surface 单点求值。

### 1.5 getState / 液面判定

**Java 位置**：`carver/Carver.java:164-176`
**机制**：
```
if (pos.getY() <= config.lavaLevel.getY(context)) return LAVA.getBlockState();
else {
    BlockState st = sampler.apply(new DensityFunction.UnblendedNoisePos(x,y,z), 0.0);
    return st == null ? (isDebug ? barrier : null) : (isDebug ? getDebugState : st);
}
```
- **lavaLevel 边界**：`config.lavaLevel` 是 `YOffset`（cave.json 为 `above_bottom 8` → `minY + 8` = -56）。**y ≤ lavaLevel 时直接放岩浆（不查 aquifer）**；其上才由 aquifer 决定 air/water。
- `sampler.apply` 返回 null 表示该点不是洞穴（density 判据），此时不雕刻。
- `CarverConfig.lavaLevel` 字段来自 `configured_carver/*.json` 的 `lava_level`（`YOffset.OFFSET_CODEC`：`above_bottom` / `below_top` / `absolute`，`YOffset.java:9-93`）。

**C++ 复刻要点**：YOffset 三型解析（`above_bottom` = `minY+offset`；`below_top` = `minY+height-1-offset`；`absolute` = 直接值）；lava 阈值逐点判断在 `Aquifer::apply` 之前。

### 1.6 CaveCarver.carve 洞穴分支树

**Java 位置**：`carver/CaveCarver.java:24-86`（`carve`）、`105-122`（`carveCave`）、`124-219`（`carveTunnels`）、`88-103`（参数 getter）
**机制**：
- `getBranchFactor() = 4`（`Carver.java:55-57`）；`i = ChunkSectionPos.getBlockCoord(4*2-1) = (4*2-1)*16 = 112`。
- **洞穴系统数量**：`j = random.nextInt(random.nextInt(random.nextInt(15)+1)+1)`（`getMaxCaveCount()=15`，`CaveCarver.java:88-90`）。
- 每系统起点：`d = chunkPos.getOffsetX(nextInt(16))`、`e = config.y.get(random, context)`、`f = chunkPos.getOffsetZ(nextInt(16))`、`g=horizontalRadiusMultiplier.get`、`h=verticalRadiusMultiplier.get`、`l=floorLevel.get`。
- `skipPredicate = isPositionExcluded(sx,sy,sz,l) = sy <= floorY || sx²+sy²+sz² >= 1.0`（`CaveCarver.java:221-223`）。
- `m = 1`；**25% 概率（nextInt(4)==0）**：`carveCave`（球形空洞：`i = 1.5 + sin(π/2)*width`，`j = i*h`，`carveRegion(x+1, y, z, i, j)`；`width = 1 + random*6`，`n = config.yScale.get`）+ `m += nextInt(4)`。
- `m` 条隧道：
  - `q = random*2π`（初始 yaw）、`o = (random-0.5)/4`（初始 pitch）、`r = getTunnelSystemWidth(random)`（`f = random*2 + random`；10% 概率 `f *= random²*3+1`）、`s = i - nextInt(i/4)`（分支数）、`t=0`（分支起点索引）。
  - `carveTunnels(..., random.nextLong(), d, e, f, g, h, r, q, o, 0, s, getTunnelSystemHeightWidthRatio()=1.0, mask, skip)`。
- **carveTunnels 递归**（`CaveCarver.java:124-219`）：
  - `Random random = Random.create(seed)` —— **递归子分支用 Xoroshiro128PlusPlus**（`Random.create(seed)` 的默认实现）！
  - `i = random.nextInt(branchCount/2) + branchCount/4`（分支点）；`bl = random.nextInt(6)==0`（pitch 衰减 0.92 vs 0.7）。
  - 每步 `j ∈ [branchStartIndex, branchCount)`：
    - `d = 1.5 + sin(π*j/branchCount)*width`（当前半径）；`e = d * yawPitchRatio`。
    - 位移：`x += cos(yaw)*cos(pitch); y += sin(pitch); z += sin(yaw)*cos(pitch)`。
    - `pitch *= bl?0.92:0.7; pitch += g*0.1; yaw += f*0.1; g *= 0.9; f *= 0.75; g += (r1-r2)*r3*2.0; f += (r1-r2)*r3*4.0`。
    - `j == i && width > 1.0`：**分叉两个子隧道**（`yaw±π/2`、`pitch/3`、`width = random*0.5+0.5`、`branchStart=j`、`branchCount` 不变、`yawPitchRatio=1.0`），`return`。
    - `random.nextInt(4) != 0`：`canCarveBranch` 检查 + `carveRegion(x,y,z, d*horizontalScale, e*verticalScale, mask, skip)`。

**C++ 复刻要点**：
- 严格保持随机数调用顺序（每系统/每隧道/每步的顺序决定结果一致性）。
- 递归子分支 `Random.create(seed)` 用 Xoroshiro128PlusPlus（C++ 已有）；洞穴系统级用 ChunkRandom(CheckedRandom)（来自 setCarverSeed）。两套 RNG 共存，勿混用。

### 1.7 RavineCarver.carve（canyon）

**Java 位置**：`carver/RavineCarver.java:23-45`（`carve`）、`47-110`（`carveRavine`）、`112-126`（`createHorizontalStretchFactors`）、`128-132`（`getVerticalScale`）、`134-140`（`isPositionExcluded`）
**机制**：
- `i = (4*2-1)*16 = 112`；起点随机（同 cave）、`yaw=random*2π`、`pitch=verticalRotation.get`、`h=yScale.get`、`k=shape.thickness.get`（trapezoid 0..6 plateau 2）、`l = (int)(112 * shape.distanceFactor.get)`、`m=0`。
- `carveRavine(..., random.nextLong(), d, j, e, k, f, g, 0, l, h, mask)`（递归结构同 carveTunnels，但**无分支分叉**）。
- `createHorizontalStretchFactors`：`fs[height]`；`j==0 || nextInt(widthSmoothness)==0` 时 `f = 1 + random*random`；`fs[j] = f*f`（逐层宽度平滑）。
- 每步：`d = 1.5 + sin(iπ/branchCount)*width`；`d *= shape.horizontalRadiusFactor.get`；`e = getVerticalScale = (1-|0.5-i/branchCount|*2) → verticalRadiusDefaultFactor + verticalRadiusCenterFactor*fac`，再 `× pitch × nextBetween(0.75, 1.0)`（`MathHelper.nextBetween`）。
- 位移同隧道但：`pitch *= 0.7; pitch += g*0.05; yaw += f*0.05; g *= 0.8; f *= 0.5`。
- `nextInt(4)!=0`：`canCarveBranch` + `carveRegion`，skipPredicate = `isPositionExcluded(context, fs, sx, sy, sz, y)`：
  `(sx²+sz²)*fs[y-1] + sy²/6.0 >= 1.0`（水平拉伸 + 垂直压缩 6 倍）。

**C++ 复刻要点**：`fs` 数组按 `context.getHeight()` 分配；`MathHelper.nextBetween = min + nextInt(max-min+1)`（float 版：`min + random.nextFloat()*(max-min)`）。

**NetherCaveCarver**（`carver/NetherCaveCarver.java:16-61`，`nether_cave`）覆写：`getMaxCaveCount()=10`；`getTunnelSystemWidth=(rand*2+rand)*2.0`；`getTunnelSystemHeightWidthRatio()=5.0`；`carveAtPoint` 覆写为 `y <= minY+31 → LAVA` 否则 `CAVE_AIR`（不查 aquifer）；`carvableFluids={LAVA, WATER}`。

### 1.8 CarverConfig 字段（configured_carver JSON）

**Java 位置**：`carver/CarverConfig.java:15-42`、`CaveCarverConfig.java:11-67`、`RavineCarverConfig.java:12-77`
**基类字段**（`CarverConfig`）：
| JSON 字段 | 类型 | 说明 |
|---|---|---|
| `probability` | float (0..1) | shouldCarve 阈值 |
| `y` | HeightProvider | 洞穴起始 Y（uniform/trapezoid/constant...） |
| `yScale` | FloatProvider | 垂直缩放（cave 大空洞专用；JSON 可写数字=常量） |
| `lava_level` | YOffset | 岩浆边界（above_bottom 8 = -56 主世界） |
| `debug_settings` | CarverDebugConfig | 默认 DEFAULT（非调试） |
| `replaceable` | 方块标签 | `#minecraft:overworld_carver_replaceables` |
**CaveCarverConfig 附加**：`horizontal_radius_multiplier`（FloatProvider）、`vertical_radius_multiplier`、`floor_level`（FloatProvider，-1..1，cave: uniform -1.0..-0.4）。
**RavineCarverConfig 附加**：`vertical_rotation`（FloatProvider，canyon: uniform ±0.125）、`shape{distance_factor, thickness, width_smoothness, horizontal_radius_factor, vertical_radius_default_factor, vertical_radius_center_factor}`。
**Carver 注册名**（`Carver.java:29-31`）：`cave` / `nether_cave` / `canyon`。

**JSON 结构差异（C++ 解析器注意）**：
- FloatProvider 编码：`{"type":"minecraft:uniform","value":{"min_inclusive":..,"max_exclusive":..}}`（**value 包装**），并支持直接数字（= ConstantFloatProvider）；`trapezoid` 为 `{"min":..,"max":..,"plateau":..}`。
- HeightProvider 编码：`{"type":"minecraft:uniform","min_inclusive":{YOffset},"max_inclusive":{YOffset}}`（**平铺，无 value 包装**），支持直接整数简写。
- YOffset 三型：`above_bottom` / `below_top` / `absolute`（数字）。
- 注册名（`heightprovider/HeightProviderType.java:8-13`）：`constant`/`uniform`/`biased_to_bottom`/`very_biased_to_bottom`/`trapezoid`/`weighted_list`。（`floatprovider/FloatProviderType.java:8-11`）：`constant`/`uniform`/`clamped_normal`/`trapezoid`。

---

## 2. FEATURES 数据流

### 2.1 入口链

**Java 位置**：`net/minecraft/world/gen/chunk/ChunkGenerator.java:334-423`（`generateFeatures()`；注意**不在** NoiseChunkGenerator）
**机制**：
1. `!SharedConstants.isOutsideGenerationArea(chunkPos)` 检查。
2. `blockPos = chunkSectionPos.getMinPos()` = `(chunkX*16, bottomY, chunkZ*16)`。
3. `chunkRandom = new ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))` —— **基类是 Xoroshiro128PlusPlus**。
4. `l = chunkRandom.setPopulationSeed(world.getSeed(), blockPos.getX(), blockPos.getZ())`。
5. 收集 3×3 邻域 chunk 所有 section 的 biome 进 `set`，`retainAll(biomeSource.getBiomes())`（只生成当前维度支持的 biome 的 feature）。
6. `i = indexedFeaturesListSupplier.get().size()`（PlacedFeatureIndexer 结果，见 2.4）。
7. 遍历 `k ∈ [0, max(GenerationStep.Feature.values().length, i))`：
   - **结构阶段**（`structureAccessor.shouldGenerateStructures()`）：`map.getOrDefault(k)`（结构按 `getFeatureGenerationStep().ordinal()` 分组），`chunkRandom.setDecoratorSeed(l, m, k)`（m 为结构序号），`start.place(...)`。
   - **feature 阶段**（`k < i`）：
     - `intSet`：遍历 `set` 中每个 biome 的 `features.get(k)`，把每个 placedFeature 经 `indexedFeatures.indexMapping().applyAsInt(pf)` 加入（去重）。
     - 排序 → 遍历：`chunkRandom.setDecoratorSeed(l, p, k)`（**p 是 indexMapping 后的索引**）；`placedFeature.generate(world, this, chunkRandom, blockPos)`。

**种子公式**（`ChunkRandom.java:54-78`）：
```
setPopulationSeed(worldSeed, blockX, blockZ):
    setSeed(worldSeed);
    l = nextLong() | 1L;      // 注意 |1L（保证奇数）
    m = nextLong() | 1L;
    n = blockX * l + blockZ * m ^ worldSeed;
    setSeed(n);
    return n;

setDecoratorSeed(populationSeed, index, step):
    setSeed(populationSeed + index + 10000 * step);
```
- **Xoroshiro 基类下 `nextLong()` 语义**：Java `Random.nextLong()` = `(long)next(32) << 32 | next(32)`；而 `ChunkRandom.next(bits)` 对非 CheckedRandom 基类 = `(int)(baseRandom.nextLong() >>> 64-bits)`。所以 setPopulationSeed 的两轮 `nextLong()` 各消费 **两次** Xoroshiro 输出（每次取该轮 64 位的高 32 位），共 4 轮 Xoroshiro 输出。
- **LCG 基类（carver 阶段）** 则 `next(32)` 是两次 LCG 递推。两套语义完全不同，C++ 必须分别实现。

**C++ 复刻要点**：
- `ChunkRandom(Xoroshiro128PlusPlus)` 包装：`next(bits) = (int)(xoroshiro.next() >>> (64-bits))`；`setSeed` = `createXoroshiroSeed`（C++ `random.h` 已有）。
- `setPopulationSeed` 里 `nextLong()` 用 Java Random 语义（两次取高 32 位拼接）——不要在 Xoroshiro 基类直接调 `next()` 一次拿 64 位。
- `setDecoratorSeed` 中 index = indexMapping 后的索引（不是 biome 列表原始下标）。

### 2.2 PlacedFeature.generate

**Java 位置**：`feature/PlacedFeature.java:44-63`
**机制**：
```
Stream<BlockPos> stream = Stream.of(pos);
for (PlacementModifier pm : placementModifiers)
    stream = stream.flatMap(p -> pm.getPositions(context, random, p));
stream.forEach(p -> { if (configuredFeature.generate(world, generator, random, p)) any = true; });
return any;
```
- `context = FeaturePlacementContext(world, generator, Optional.of(this))`。
- placementModifiers 按 JSON `placement[]` 顺序依次 flatMap（每个位置 → 0..n 个新位置）。
- 传入 `generate` 的随机数是同一个 `chunkRandom`（已 setDecoratorSeed）——**所有 placement modifier 和 feature 内部共用同一 RNG 流**。

**C++ 复刻要点**：用 `std::vector<BlockPos>` 模拟 flatMap（前一个修饰器的输出喂给下一个）；位置顺序必须保持（影响后续 RNG 消费顺序）。

### 2.3 ConfiguredFeature.generate / Feature.generateIfValid

**Java 位置**：`feature/ConfiguredFeature.java:23-25`、`feature/Feature.java:151-153`
**机制**：
```
ConfiguredFeature.generate → feature.generateIfValid(config, world, generator, random, origin)
    = world.isValidForSetBlock(pos) ? feature.generate(new FeatureContext(Optional.empty(), world, generator, random, pos, config)) : false
```
- `FeatureContext`（`feature/util/FeatureContext.java`）：`getOrigin()/getRandom()/getWorld()/getGenerator()/getConfig()`。
- Feature 注册表：`Registries.FEATURE`（`Feature.java:26-118` 全量注册，见第 5 节）。
- ConfiguredFeature 的 CODEC 是 `Registries.FEATURE.getCodec().dispatch(feature, Feature::getCodec)` —— JSON `type` 字段决定 Feature 实例。

**C++ 复刻要点**：feature 分发表（string → generate 函数/类）；`isValidForSetBlock` 即 y ∈ [minY, maxY)。

### 2.4 PlacedFeatureIndexer.indexMapping 语义

**Java 位置**：`feature/util/PlacedFeatureIndexer.java:49-156`
**机制**：
- `collectIndexedFeatures(biomes, biome→featuresList, ...)` 启动时执行一次：
  - 遍历所有 biome 的所有 features step 列表，每个 placedFeature 首次出现时分配全局 `featureIndex`（`mutableInt` 递增）。
  - 记录同 biome 内相邻 feature 的先后关系（`list.get(j) → list.get(j+1)` 边），**拓扑排序**（`TopologicalSorts.sort`）保证：若某 feature 在不同 biome 的 step 不一致 → "feature order cycle" 崩溃（DataFixer 校验）。
  - 输出 `List<IndexedFeatures>`：按 step 分组，每组 `features = List<PlacedFeature>`（拓扑序去重）+ `indexMapping: PlacedFeature→int`（该 feature 在本 step 列表中的索引）。
- **在 generateFeatures 中的用途**：`intSet.add(indexMapping(placedFeature))` 把「各 biome 在该 step 的 placedFeature 集合」映射到**统一的全局索引**，去重后按索引排序生成——保证同一 placed feature 只生成一次、且所有 biomes 间顺序一致。

**C++ 复刻要点**：启动时从全部 biome JSON 的 `features[step]` 构建：每 step 的 placedFeature 集合（去重、稳定排序——Java 用拓扑序，C++ 可按 biome 出现序 + step 升序近似，但**必须与 Java 的 indexMapping 数值一致**才能复现 setDecoratorSeed 的 index）。建议直接导出 `indexMapping`（Java 侧可加 probe 导出）。若无法复刻拓扑序，可用固定 biome 注册表顺序近似并单独验证。

### 2.5 GenerationStep.Feature 枚举顺序（决定 step 序号）

**Java 位置**：`gen/GenerationStep.java:27-38`
**顺序**（ordinal 0..10，与 biome JSON `features[]` 数组下标一一对应）：
```
0 raw_generation
1 lakes
2 local_modifications
3 underground_structures
4 surface_structures
5 strongholds
6 underground_ores
7 underground_decoration
8 fluid_springs
9 vegetal_decoration
10 top_layer_modification
```
**Carver 枚举**（`GenerationStep.java:6-8`）：`AIR("air")` ordinal 0、`LIQUID("liquid")` ordinal 1。

**C++ 复刻要点**：`enum class FeatureStep` 保持 0..10 序号；biome JSON `features` 数组长度 ≤11（空数组保留占位，如 plains 的 `[]`）。

---

## 3. biome JSON → placed/configured feature 引用链

### 3.1 数据结构（GenerationSettings）

**Java 位置**：`world/biome/GenerationSettings.java:35-50, 72-90`
**JSON 结构**（`worldgen/data/minecraft/worldgen/biome/plains.json` 示例）：
```json
{
  "carvers": { "air": ["minecraft:cave", "minecraft:cave_extra_underground", "minecraft:canyon"] },
  "features": [
    [],                                    // 0 raw_generation
    ["minecraft:lake_lava_underground", ...],  // 1 lakes
    ["minecraft:amethyst_geode"],          // 2 local_modifications
    ["minecraft:monster_room", ...],       // 3 underground_structures
    [], [],                                // 4,5
    ["minecraft:ore_dirt", ..., "minecraft:disk_gravel"],  // 6 underground_ores
    [],                                    // 7
    ["minecraft:spring_water", "minecraft:spring_lava"],   // 8 fluid_springs
    ["minecraft:glow_lichen", ..., "minecraft:patch_pumpkin"],  // 9 vegetal_decoration
    ["minecraft:freeze_top_layer"]         // 10 top_layer_modification
  ]
}
```
- `features[i]` = `GenerationStep.Feature` ordinal i 的 placed feature 列表（元素是 `placed_feature/*.json` 的 registry 名）。
- 引用链：`biome.features[step]` → `placed_feature/<name>.json` → `configured_feature/<name>.json` → feature `type` + `config`。
- `getCarversForStep(AIR)` = `carvers.air`（LIQUID 在 vanilla 主世界未用）。

### 3.2 placed_feature JSON 结构

**示例**（`placed_feature/ore_dirt.json`）：
```json
{
  "feature": "minecraft:ore_dirt",
  "placement": [
    { "type": "minecraft:count", "count": 7 },
    { "type": "minecraft:in_square" },
    { "type": "minecraft:height_range", "height": { "type": "minecraft:uniform", "max_inclusive": {"absolute":160}, "min_inclusive": {"absolute":0} } },
    { "type": "minecraft:biome" }
  ]
}
```
- `feature` 引用 configured_feature registry 名；`placement[]` 按顺序应用（见第 4 节）。

### 3.3 configured_feature JSON 结构

**示例**（`configured_feature/ore_dirt.json`）：
```json
{
  "type": "minecraft:ore",
  "config": {
    "discard_chance_on_air_exposure": 0.0,
    "size": 33,
    "targets": [
      { "state": {"Name": "minecraft:dirt"},
        "target": { "predicate_type": "minecraft:tag_match", "tag": "minecraft:base_stone_overworld" } }
    ]
  }
}
```
- `type` → `Registries.FEATURE`（`Feature.java` 注册名）；`config` 按该 Feature 的 config CODEC 解析。

### 3.4 C++ 加载路径

- 参考 `worldgen_api.cpp wg_create` 现有加载模式：`wgDir + "/data/minecraft/worldgen/" + <子目录> + "/" + <name> + ".json"`（如 `noise_settings/`、`density_function/<ns>/`）。
- 需要新增的加载：
  - `worldgen/biome/*.json`：解析 `carvers`（air/liquid 列表）+ `features[11]`（placed feature 名列表）——**biome.h 目前只解析 biome_params.json，需扩展或新增 `generation_settings.h`**。
  - `worldgen/placed_feature/*.json`：`feature` 引用 + `placement[]` 列表。
  - `worldgen/configured_feature/*.json`：`type` + `config`。
  - `worldgen/configured_carver/*.json`：`type`（cave/nether_cave/canyon）+ `config`（1.8 节字段）。
- 注册表三张：`configured_carver`（4 个文件：cave/cave_extra_underground/nether_cave/canyon）、`configured_feature`（~200 个）、`placed_feature`（~400 个）。建议启动时一次性加载建表（id → 结构化对象），而非每 chunk 解析。

---

## 4. PlacementModifier 全集清单

**Java 位置**：`placementmodifier/PlacementModifierType.java:8-30`（注册名）；各实现类。
**基类**：`PlacementModifier.java:17-27`（`getPositions` 抽象）；`AbstractCountPlacementModifier.java:9-15`（`IntStream.range(0, count).mapToObj(i->pos)`）；`AbstractConditionalPlacementModifier.java:8-14`（`shouldPlace ? Stream.of(pos) : Stream.empty()`）。

| type（注册名） | 类 | JSON 字段 | getPositions 语义 |
|---|---|---|---|
| `block_predicate_filter` | BlockFilterPlacementModifier | `predicate`（BlockPredicate） | 条件：`predicate.test(world, pos)` |
| `rarity_filter` | RarityFilterPlacementModifier | `chance`（int>0） | 条件：`random.nextFloat() < 1.0/chance` |
| `surface_relative_threshold_filter` | SurfaceThresholdFilterPlacementModifier | `heightmap`, `min_inclusive`(默认 MIN), `max_inclusive`(默认 MAX) | 条件：`topY(heightmap)+min <= y <= topY+max` |
| `surface_water_depth_filter` | SurfaceWaterDepthFilterPlacementModifier | `max_water_depth` | 条件：`topY(WORLD_SURFACE) - topY(OCEAN_FLOOR) <= max` |
| `biome` | BiomePlacementModifier | （无） | 条件：该 biome 的 GenerationSettings 包含此 placed feature |
| `count` | CountPlacementModifier | `count`（IntProvider 0..256） | 复制 pos `count` 次 |
| `noise_based_count` | NoiseBasedCountPlacementModifier | `noise_to_count_ratio`, `noise_factor`, `noise_offset`(默认0) | `count = ceil((FOLIAGE_NOISE.sample(x/f, z/f, false) + offset) * ratio)` |
| `noise_threshold_count` | NoiseThresholdCountPlacementModifier | `noise_level`, `below_noise`, `above_noise` | `count = FOLIAGE_NOISE.sample(x/200, z/200, false) < level ? below : above` |
| `count_on_every_layer` | CountMultilayerPlacementModifier | `count`（IntProvider） | 从 MOTION_BLOCKING 顶向下逐层找「实心→非实心」分界，每层 count 个 pos（`findPos` 递增 targetY；@Deprecated，但老 JSON 存在） |
| `environment_scan` | EnvironmentScanPlacementModifier | `direction_of_search`, `target_condition`, `allowed_search_condition`(默认 true), `max_steps`(1..32) | 从 pos 沿 direction 扫描至多 max_steps，返回首个满足 target 的 pos |
| `heightmap` | HeightmapPlacementModifier | `heightmap`（Type） | `y = topY(heightmap, x, z)`；topY ≤ bottomY 则空 |
| `height_range` | HeightRangePlacementModifier | `height`（HeightProvider） | `y = height.get(random, context)`（其余 x/z 不变） |
| `in_square` | SquarePlacementModifier | （无，unit codec） | `x += nextInt(16); z += nextInt(16)` |
| `random_offset` | RandomOffsetPlacementModifier | `xz_spread`, `y_spread`（IntProvider -16..16） | `x += xz.get; y += y.get; z += xz.get`（**xz 用同一 provider 两次**） |
| `carving_mask` | CarvingMaskPlacementModifier | `step`（"air"/"liquid"） | 返回该 carver step 的 mask 中所有已雕刻位置（`streamBlockPos`） |

**Heightmap.Type**（`world/Heightmap.java:142-155`）：
- `WORLD_SURFACE_WG`/`WORLD_SURFACE`：`!state.isAir()`。
- `OCEAN_FLOOR_WG`/`OCEAN_FLOOR`：`state.blocksMovement()`。
- `MOTION_BLOCKING`：`blocksMovement() || !fluidState.isEmpty()`。
- `MOTION_BLOCKING_NO_LEAVES`：同上但排除 leaves。
- CODEC 序列化用大写名（`"WORLD_SURFACE_WG"` 等）。

**C++ 复刻要点**：15 个类；`count` 的 IntProvider 支持数字简写（`"count":7` = ConstantIntProvider）；`noise_based_count`/`noise_threshold_count` 依赖 `Biome.FOLIAGE_NOISE`（C++ `noise.h` 已有 Xoroshiro 噪声采样，需注册 `minecraft:foliage` 噪声参数）；`carving_mask` 需要 FEATURES 阶段访问 CARVERS 阶段的 mask（跨阶段状态，ProtoChunk 上持有）。

---

## 5. Feature 类清单（按实现优先级）

**注册表**：`feature/Feature.java:26-118`（`Registries.FEATURE`）。**依赖子系统**：`stateprovider/`（BlockStateProvider）、`foliage/`（FoliagePlacer）、`trunk/`（TrunkPlacer）、`treedecorator/`（TreeDecorator）、`root/`（RootPlacer）、`blockpredicate/`（BlockPredicate）、`size/`（FeatureSize）。

### 5.1 高优先级（主世界地下/地表核心，fillOneChunk 直连）

| type | 类（文件） | 简述 | 依赖 |
|---|---|---|---|
| `ore` | OreFeature | 椭球矿脉：size/8 半轴、size/16 半径、随机角 f、startY/endY=origin±nextInt(3)-2；`generateVeinPart` 沿 size 个球心 lerp 生成，球心半径 `((sin(πf)+1)*h+1)/2`（h=rand*j/16），两两遮挡剔除；逐 targets 匹配替换；`discard_chance_on_air_exposure` 且暴露空气则丢弃 | OreFeatureConfig.targets[].target（BlockPredicate tag_match/state_match...） |
| `scattered_ore` | ScatteredOreFeature | 撒点：`nextInt(size+1)` 个点，每点 `(rand-rand)*min(j,7)` 偏移，targets 替换（MAX_SPREAD=7） | OreFeatureConfig |
| `disk` | DiskFeature | 圆盘：radius 随机、halfHeight 上下各 halfHeight 层；`target().test(world,pos)` 命中 → stateProvider 出方块（`markBlocksAboveForPostProcessing`） | DiskFeatureConfig{state_provider, target, radius, half_height} |
| `simple_block` | SimpleBlockFeature | 单方块：`toPlace().get(random,pos)`，canPlaceAt 检查；TallPlantBlock 双格 | SimpleBlockFeatureConfig{to_place: BlockStateProvider} |
| `spring_feature` | SpringFeature | 泉水：上下/四邻 rockCount 个 valid_blocks 且 holeCount 个 air 才放流体 + `scheduleFluidTick` | SpringFeatureConfig{state, valid_blocks, rock_count, hole_count, requires_block_below} |
| `freeze_top_layer` | FreezeTopLayerFeature | 全 16×16 列：biome.canSetIce → ICE；canSetSnow → SNOW + SNOWY 属性 | Biome 温度（C++ biome.h 已有 temperature） |
| `underwater_magma` | UnderwaterMagmaFeature | 水下岩浆：CaveSurface 找水底，placementRadiusAroundFloor 半径内按概率放岩浆（须被水/空气包围四邻） | CaveSurface（feature/util/CaveSurface.java） |
| `seagrass` | SeagrassFeature | OCEAN_FLOOR 顶：概率双格高 TALL_SEAGRASS | ProbabilityConfig |
| `kelp` | KelpFeature | 1+nextInt(10) 高海带（顶格 KELP age 20..23） | DefaultFeatureConfig |
| `random_patch` | RandomPatchFeature | tries 次在 xzSpread/ySpread 内随机偏移，每次调 `config.feature().value().generateUnregistered(...)`（不查 biome） | RandomPatchFeatureConfig{tries, xz_spread, y_spread, feature: ConfiguredFeature 引用} |
| `flower` / `no_bonemeal_flower` / `random_patch` | RandomPatchFeature 实例 | 同上（flower 是 RandomPatchFeature 注册的别名） | 同上 |

### 5.2 树（TreeFeature）

| type | 类 | 简述 | 依赖 |
|---|---|---|---|
| `tree` | TreeFeature | `trunkPlacer.getHeight` → `foliagePlacer.getRandomHeight/Radius` → rootPlacer.trunkOffset → getTopPosition（minimumSize 半径内可替换检查）→ trunkPlacer.generate 返回 TreeNode 列表 → foliagePlacer.generate 每个 node → treedecorator.generate | TreeFeatureConfig{trunk_placer, foliage_placer, root_placer(optional), decorators[], minimum_size, ignore_vines} |

- **TrunkPlacer 包**（`trunk/`）：straight/`bending`/`forking`/`giant`/`mega_jungle`/`dark_oak`/`fancy`/`cherry`/`mangrove`...（按 `trunk_placer` 的 type 分发）。
- **FoliagePlacer 包**（`foliage/`）：blob/`spruce`/`pine`/`acacia`/`bush`/`fancy`/`jungle`/`mega_pine`/`random_spread`/`cherry`...
- **TreeDecorator 包**（`treedecorator/`）：leave_vine/`cocoa`/`beehive`/`trunk_vine`/`alter_ground`/`attached_to_leaves`...
- **RootPlacer 包**（`root/`）：`mangrove_roots` 等（optional）。
- 依赖 `BlockTags.REPLACEABLE_BY_TREES`、`LEAVES`；`isAirOrLeaves`、`canReplace`。
- **C++ 提示**：TreeFeature 是全管线最复杂的单类（8786 字节 + 3 个 placer 包 + minimum_size 包）。建议先复刻 straight trunk + blob foliage（橡树/桦树），再按需扩展。

### 5.3 中优先级（地物、湖、蘑菇、植被 patch、洞穴地物）

| type | 类 | 简述 |
|---|---|---|
| `lake` | LakeFeature | 湖（1.18+ 多为 `lake_lava_underground/surface`；石质湖盆 + lava/water） |
| `monster_room` | DungeonFeature | 刷怪笼房间（含箱子/刷怪笼方块） |
| `block_column` | BlockColumnFeature | 石柱（`block_column` config） |
| `vegetation_patch` / `waterlogged_vegetation_patch` | VegetationPatchFeature | 地表植被 patch（沿地表生成，BlockPlacer + BlockStateProvider） |
| `multiface_growth` | MultifaceGrowthFeature | 多面生长（glow_lichen） |
| `root_system` | RootSystemFeature | 树根系统（mangrove） |
| `geode` | GeodeFeature | 紫水晶洞（多层 shell） |
| `dripstone_cluster` / `large_dripstone` / `pointed_dripstone` | DripstoneCluster/LargeDripstone/SmallDripstoneFeature | 滴水石 |
| `iceberg` / `forest_rock` / `blue_ice` | Iceberg/ForestRock/BlueIceFeature | 冰山/森林岩石/蓝冰 |
| `huge_red_mushroom` / `huge_brown_mushroom` | HugeMushroomFeature | 大蘑菇 |
| `random_selector` / `simple_random_selector` / `random_boolean_selector` | RandomFeature/SimpleRandomFeature/RandomBooleanFeature | 组合器（按权重/概率选子 feature） |
| `fill_layer` | FillLayerFeature | 填充层 |
| `netherrack_replace_blobs` | ReplaceBlobsFeature | 下界岩替换团 |
| `basalt_columns` / `basalt_pillar` / `delta_feature` | BasaltColumns/BasaltPillar/DeltaFeature | 下界玄武岩/三角洲 |
| `weeping_vines` / `twisting_vines` / `nether_forest_vegetation` / `huge_fungus` | 对应类 | 下界植被 |
| `sea_pickle` / `coral_tree` / `coral_mushroom` / `coral_claw` / `bamboo` | 对应类 | 水下/竹林植被 |
| `chorus_plant` / `end_island` / `end_spike` / `end_gateway` / `void_start_platform` | 对应类 | 末地 |
| `replace_single_block` | EmeraldOreFeature | 绿宝石单点替换 |
| `bonus_chest` | BonusChestFeature | 奖励箱 |
| `fossil` / `desert_well` / `ice_spike` / `glowstone_blob` / `vines` | 对应类 | 杂项 |

### 5.4 依赖子系统（供 5.1-5.3 复用）

- **BlockStateProvider**（`stateprovider/`）：`simple_state_provider`/`weighted_state_provider`/`noisy_threshold_provider`/`rotated_block_provider`/`randomized_int_block_state_provider`/`dual_state_provider`。
- **BlockPlacer**（`feature/util/`）：simple/column/torch 等（VegetationPatch 用）。
- **BlockPredicate**（`blockpredicate/`）：`tag_match`/`state_match`/`matching_blocks`/`matching_fluids`/`would_survive`/`not`/`any_of`/`all_of`/`solid`/`replaceable`/`inside_world_bounds`/`has_sturdy_face`...
- **IntProvider/FloatProvider/HeightProvider**：见 1.8 节编码差异。

**C++ 复刻要点**：以第 2 节分发器（type → generate）+ config JSON 解析为骨架，逐个实现；建议顺序 = 5.1 高优先级 → 5.3 常用 → 5.2 树（单独里程碑）。

---

## 6. C++ 接入点清单

### 6.1 fillOneChunk 插桩位置

**现状**（`versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp:548-834`）：
- `fillOneChunk` 流程：`3. density(densityBuf) → 3b. aquifer+oreVein(col 写入) → 4. buildSurface(col 原地) → 5. memcpy(col → out)`。
- `col` 是 `BlockColumn`（per-chunk 列存储），`heightmap` 已由 3b 维护。

**插桩点**（buildSurface 之后、输出之前）：
```
// --- 现有 4. buildSurface 结束（L816-823）---
// [NEW 5a. CARVERS] 在 col 上原位雕刻（等价 Java ChunkStatus CARVERS）
//     - 若支持 carvers: 遍历邻域 17×17 → setCarverSeed → shouldCarve → CaveCarver/RavineCarver.carve
//     - 需要: per-chunk CarvingMask(step=air/liquid)、aquifer、CarverContext(含 surface 单点补丁回调)
//     - 注意: carveRegion 遍历的是邻域洞穴体，落点可能跨 chunk —— 只写本 chunk 的 (0..15, y, 0..15)
// [NEW 5b. FEATURES] 在 col 上生成 feature
//     - 需要: setPopulationSeed + 按 step 的 placedFeature 列表 + setDecoratorSeed + placement flatMap + feature.generate
//     - 需要: Heightmap(WORLD_SURFACE_WG / OCEAN_FLOOR_WG / MOTION_BLOCKING 等) —— 3b 只维护了简单 heightmap，
//             需按 Java Heightmap.populateHeightmaps 补全各 Type（feature placement 依赖）
// --- 现有 5. 输出 memcpy ---
```
**注意**：Java 的 CARVERS/FEATURES 是独立 ChunkStatus，跨 chunk 邻域读取（17×17 chunk）；C++ fillOneChunk 是单 chunk 内生成。若要做邻域感知（如 lake、geode 跨边界），需维护 chunk 邻接缓冲或按 3×3 生成后取中心（与 Java 的 1-chunk-radius 读取一致：generateFeatures 读 3×3、carve 读 17×17）。**建议 v1 先单 chunk 内部生成（忽略邻域越界部分），与 Java 对比后再补邻域**。

### 6.2 新增 C++ 文件建议

| 文件 | 内容 |
|---|---|
| `chunkrandom.h` | `CheckedRandom`（48 位 LCG）+ `ChunkRandom` 包装（baseRandom 抽象：Checked/Xoroshiro 两种路径）+ `setCarverSeed`/`setPopulationSeed`/`setDecoratorSeed` + `RandomSeed.getSeed`（`new java.util.Random(seed).nextLong()` 的 48 位截断语义） |
| `carver.h` / `carver.cpp` | `CarvingMask`（BitSet 语义 + streamBlockPos）+ `CarverConfig`/`CaveCarverConfig`/`RavineCarverConfig`（含 Shape）+ `Carver` 基类（carveRegion/carveAtPoint/getState/canCarveBranch）+ `CaveCarver` + `RavineCarver` + `NetherCaveCarver`（maxCaveCount=10、tunnelWidth×2、yawPitchRatio=5、carveAtPoint 覆写为 minY+31 以下 LAVA/以上 CAVE_AIR） |
| `feature.h` / `feature.cpp` | `PlacedFeature`/`ConfiguredFeature`/`FeatureContext` + type→generate 分发表 + 5.1/5.3 各 Feature 实现 |
| `placement.h` / `placement.cpp` | `PlacementModifier` 基类 + 15 个实现（第 4 节） |
| `blockstateprovider.h` | BlockStateProvider（simple/weighted/rotated...）+ 也许 BlockPredicate |
| `providers.h` | IntProvider/FloatProvider/HeightProvider/YOffset 统一解析（含 1.8 节编码差异） |
| `generation_settings.h` | biome JSON 的 carvers/features 解析 + PlacedFeatureIndexer（启动时构建，或复用 Java 导出） |

### 6.3 JSON 数据加载

- 沿用 `worldgen_api.cpp wg_create` 模式（`json.h` 解析 + `h->wgDir` 根）。
- 新增目录：
  - `data/minecraft/worldgen/biome/*.json` → `generation_settings.h`（carvers.air/liquid + features[11]）
  - `data/minecraft/worldgen/configured_carver/*.json` → `carver.h`（4 个文件）
  - `data/minecraft/worldgen/placed_feature/*.json` → `placement.h`（~400）
  - `data/minecraft/worldgen/configured_feature/*.json` → `feature.h`（~200）
- 启动时构建三张注册表（configured_carver / placed_feature / configured_feature）+ 每 biome 的 GenerationSettings + PlacedFeatureIndexer。
- `Biome.FOLIAGE_NOISE`（placement 的 noise_based_count / noise_threshold_count 用）需从 noise JSON 注册（`minecraft:foliage`）。

### 6.4 对齐建议（验证）

- 用 Java 侧已有 probe 工具（`block_probe.cpp`/`density_probe.cpp` 模式）新增 `carver_probe.cpp` / `feature_probe.cpp`：对固定 seed + 固定 chunk，dump：
  1. CARVERS 后某列方块序列（对比 Java `getBlockState` 反射 dump）；
  2. FEATURES 后同列（含 ore/disk/spring/freeze 结果）。
- 先验证 RNG 层（CheckedRandom/ChunkRandom 的 setCarverSeed/setPopulationSeed 输出）与 Java 一致，再验证 placement 位置，最后验证方块结果。
- 已知坑：Xoroshiro 基类 `nextLong()` 两次取高 32 位；`Random.create(seed)` 递归分支用 Xoroshiro；`random_offset` 的 xz_spread 用两次同一 provider；`carving_mask` 跨阶段状态。

---

## 附录 A：关键 RNG 语义速查

- `RandomSeed.getSeed()` = `SEED_UNIQUIFIER.updateAndGet(u -> u*1181783497276652981L) ^ System.nanoTime()`（`RandomSeed.java:43-45`，时间+原子计数器）。**仅作 ChunkRandom 构造时的初始种子，随后 setCarverSeed/setPopulationSeed 第一行 `setSeed(worldSeed)` 即覆盖 → 具体值不影响生成结果**，C++ 可任意取常量。
- `CheckedRandom`（LCG）：`setSeed` 截断 48 位；`next(bits) = (seed * 0x5DEECE66D + 0xB) & mask48; return seed >>> (48-bits)`。
- `Xoroshiro128PlusPlusRandom`：`createXoroshiroSeed(seed)`（C++ `random.h` 已有）。
- `ChunkRandom.next(bits)`：CheckedRandom 基类 → `base.next(bits)`；Xoroshiro 基类 → `(int)(base.nextLong() >>> 64-bits)`。
- `Random.create(seed)` 默认 = Xoroshiro128PlusPlus（递归隧道分支、feature 内部 `Random.create` 少见）。

## 附录 B：文件索引（本地图引用的 Java 源）

```
net/minecraft/world/gen/chunk/NoiseChunkGenerator.java        carve() L278-327
net/minecraft/world/gen/chunk/ChunkGenerator.java             generateFeatures() L334-423
net/minecraft/util/math/random/ChunkRandom.java               setPopulationSeed L54-61 / setDecoratorSeed L75-78 / setCarverSeed L87-93
net/minecraft/world/gen/GenerationStep.java                   Feature L27-38 / Carver L6-8
net/minecraft/world/gen/carver/Carver.java                    carveRegion L59-117 / carveAtPoint L119-162 / getState L164-176 / canCarveBranch L206-214
net/minecraft/world/gen/carver/CaveCarver.java                carve L24-86 / carveCave L105-122 / carveTunnels L124-219
net/minecraft/world/gen/carver/RavineCarver.java              carve L23-45 / carveRavine L47-110
net/minecraft/world/gen/carver/CarverConfig.java              L15-42
net/minecraft/world/gen/carver/CaveCarverConfig.java          L11-67
net/minecraft/world/gen/carver/RavineCarverConfig.java        L12-77
net/minecraft/world/gen/carver/CarverContext.java             L18-47
net/minecraft/world/gen/carver/CarvingMask.java               L8-55
net/minecraft/world/gen/carver/ConfiguredCarver.java          L19-40
net/minecraft/world/biome/GenerationSettings.java             L35-50,72-90
net/minecraft/world/gen/feature/PlacedFeature.java            generate L44-63
net/minecraft/world/gen/feature/ConfiguredFeature.java        generate L23-25
net/minecraft/world/gen/feature/Feature.java                  注册表 L26-118 / generateIfValid L151-153
net/minecraft/world/gen/feature/util/PlacedFeatureIndexer.java L49-156
net/minecraft/world/gen/placementmodifier/PlacementModifierType.java L8-30
net/minecraft/world/Heightmap.java                            Type L142-155
net/minecraft/world/gen/YOffset.java                          L9-93
net/minecraft/world/gen/heightprovider/HeightProviderType.java L8-13
net/minecraft/util/math/floatprovider/FloatProviderType.java  L8-11
```
