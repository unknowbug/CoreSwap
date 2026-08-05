# 1. 架构与文件映射

## 功能目的

1.20.1 主世界区块生成（ChunkStatus.NOISE + SURFACE）的 C++ 复刻，逐位对齐 vanilla。
迭代新版本时，**diff Java 核心类 → 本表定位 C++ 文件 → 按各篇『版本敏感点』改**。

## 数据流（单 chunk）

```
seed
 ├─ randomDeriver(split("minecraft:root"))            → Splitter
 │   ├─ split("minecraft:aquifer").nextSplitter()     → aquifer 随机
 │   ├─ split("minecraft:ore").nextSplitter()         → oreVein 随机
 │   └─ 各 noise 的 split("minecraft:<name>")          → DoublePerlinNoiseSampler
 │
 ├─ DensityBuilder.buildNode(overworld.json noise_router)   → router 分量树
 │   └─ finalDensity / initialDensityWithoutJaggedness / vein_* / barrier / ...
 │
 ├─ fillOneChunk(chunkX, chunkZ)
 │   ├─ finalDensity.sample(pos) 逐块（interpolated 节点 cell 网格插值）
 │   ├─ Aquifer.apply(density) → block / -1
 │   ├─ OreVein.apply() 仅在 aquifer 返回 -1 时（ChainedBlockSource）
 │   └─ SurfaceBuilder.buildSurface（surface rules 覆盖表层）
```

## Java 类 ↔ C++ 文件映射

| Java（1.20.1 fabric/yarn） | C++ | 说明 |
|---|---|---|
| `net.minecraft.util.math.random.XoroshiroRandom` / `RandomSplitter` | `worldgen/src/xoroshiro.h` | 随机派生全链 |
| `net.minecraft.util.math.noise.DoublePerlinNoiseSampler` | `worldgen/src/noise.h` | 噪声采样（OctavePerlin + Perlin） |
| `net.minecraft.world.gen.densityfunction.DensityFunctionTypes` | `worldgen/src/density.h` | 密度函数节点实现 |
| `DensityFunctionTypes.InterpolatedNoiseSampler`（old_blended_noise） | `density.h: InterpolatedNoiseDF` | base_3d_noise，逐块重算 24 octave |
| `DensityFunctionTypes.Interpolated` | `density.h: InterpolatedDF` | 4×4×8 cell 网格 + 三线性插值 |
| `net.minecraft.world.gen.noise.NoiseConfig` | `worldgen/src/density_builder.h` | JSON → 密度树构建、sampler 派生 |
| `net.minecraft.world.gen.densityfunction.DensityFunctions` | `worldgen/src/density_builder.h` | overworld noise_router 动态构造（vein_* 等） |
| `net.minecraft.world.gen.chunk.NoiseChunkGenerator` | `worldgen/src/worldgen_api.cpp` | 块级流水线（fillOneChunk） |
| `net.minecraft.world.gen.chunk.ChunkNoiseSampler` | `density.h`（插值）+ `worldgen_api.cpp` | CellCache / DensityInterpolator / estimateSurfaceHeight |
| `net.minecraft.world.gen.chunk.AquiferSampler` | `worldgen/src/aquifer.h` | 含水层 blob |
| `net.minecraft.world.gen.OreVeinSampler` | `worldgen/src/ore_vein.h` | 矿脉 |
| `net.minecraft.world.gen.surfacebuilder.VanillaSurfaceRules` | `worldgen/src/surface.h` | materialRule1..10 等规则树 |
| `net.minecraft.world.gen.surfacebuilder.MaterialRules` | `surface.h` | condition/sequence/stoneDepth/water/surface |
| `net.minecraft.world.gen.surfacebuilder.SurfaceBuilder` | `surface.h` | buildSurface 列引擎 |
| `net.minecraft.world.biome.source.BiomeSource` | `worldgen/src/biome.h` | biome 判定 |
| `net.minecraft.registry.Registry`（blocks/biomes） | `worldgen/src/blocks.h` / `biome.h` | id ↔ name |

## 数据文件（data/worldgen）

从 1.20.1 数据包复制（`data/minecraft/worldgen/...`）：
- `noise_settings/overworld.json`：noise_router 分量（vein_toggle 等**内联**定义）
- `density_function/overworld/*.json`：final_density 引用链（sloped_cheese、caves/*）
- `noise/*.json`：噪声参数（firstOctave + amplitudes）
- `biome/*.json`：biome 参数（用于 surface rules）
- `../blocks.json`：方块 id ↔ name（**vanilla raw id**，Java 导出用）

## 关键 API（worldgen_api.h）

- `wg_create(seed, worldgenDir)` → handle（一次性构建：密度树、blocks、biome、surfaceBuilder、**overworldRule 预构建**）
- `wg_fill_blocks(handle, cx, cz, out)`：单 chunk（串行，JNI 用）
- `wg_fill_blocks_multi(handle, cxs, czs, outs, count, threads)`：chunk 级并行，结果与串行逐位一致；threads<=0 自适应 `min(hw, count)`
- out 布局：`int32_t[16*16*384]`，索引 `(y+64)*256 + z*16 + x`，值 = vanilla raw BlockId（air=0）
