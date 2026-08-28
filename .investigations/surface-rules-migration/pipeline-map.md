# ③ vanilla 块管线 — surface rules 移植范围摸底（recode.scout 勘探，2026-08-28）

> 载体：`.investigations/surface-rules-migration/pipeline-map.md`
> 来源：recode.scout subagent（只读勘探，未改代码）
> 任务：为「③ vanilla 块管线」补 surface rules（地表方块），让 fill_chunk 产出具体 block id。

## 结论

- **C++ `versions/1.20.1/cpp/worldgen/src/surface.h`（859 行）已完整移植 vanilla surface rules**，配 block_probe 锚点（SURF#001/002, PILLAR#001），是 Rust 重写的直接移植源（非从 Java 起步）。
- Java 权威在 `mc_src_extract/net/minecraft/world/gen/surfacebuilder/`：`MaterialRules.java`（=SurfaceRules）、`SurfaceBuilder.java`、`VanillaSurfaceRules.java`。

## C++ 条件（SurfaceCond）与规则（SurfaceRule）

条件：BiomeCond / AboveYCond / WaterCond / StoneDepthCond / NoiseThresholdCond / HoleCond / SteepCond / SurfaceCondC(above_preliminary_surface) / TempCond(temperature) / VerticalGradientCond / NotCond。
规则：BlockRule / CondRule / SeqRule / TerracottaBandsRule。

## 规则树 & 引擎

- `buildOverworldRule` = VanillaSurfaceRules.createDefaultRule(true,false,true)，终序列 bedrock_floor + surface(mr9) + deepslate。
- `placeBadlandsPillar`（eroded_badlands）已实现（含 heightmap 写回抬升）。
- `buildSurface` 引擎：在 density/aquifer 产出 default=stone 的 BlockColumn 后逐列扫描应用规则；输入含 surfaceHeights4 4角 estimateSurfaceHeight。

## Rust 输入就绪度

✅ 就绪：block 分类、density+initial_density_without_jaggedness、biome、estimate_surface_height 4角（aquifer.rs L164）、heightmap（surface_height）、steep（可推导）、surface 噪声全集（density_builder.rs L57-71）、splitter。
❌ 缺 4 项工程基础设施：**BlockId 注册表、biome 温度表、sampleRunDepth 函数、buildSurface 引擎本身+红陶带生成**。

## 建议拆分

- **阶段 A**：BlockId 注册表 + 条件/规则枚举 + 规则树 + buildSurface 引擎接入 fill_chunk + biome 温度表 → 达成「fill_chunk 产出具体 block id」。
- **阶段 B**：terracottaBands + placeBadlandsPillar + 可选 placeIceberg（C++ 未实现）。

## ⚠️ 移植前待确认（scout 风险）

1. **HoleCond 语义存疑 → 已核对，C++ 有 bug，Rust 应修正**：
   - Java：`context.runDepth` = `sampleRunDepth(blockX,blockZ)`（SurfaceBuilder.java L172-175 = surfaceNoise + random 噪声值，L459 设置一次列级）；`hole()` = NegativeRunDepthPredicate.test = `context.runDepth <= 0`（MaterialRules.java L537）。
   - C++：`ctx.surfaceDepth = sampleRunDepth(m,n)`（surface.h L749，对应 Java runDepth）；但 `HoleCond::test` 用 `ctx.stoneDepthAbove <= 0`（L251，= q 扫描计数器），且 `ctx.runDepth` 是死字段（L750 设 0 从未更新）。
   - **结论**：C++ HoleCond 用错字段（stoneDepthAbove 扫描计数器 ≠ Java runDepth 噪声）。Rust 移植应改用 `ctx.surfaceDepth <= 0`（对齐 Java runDepth=sampleRunDepth）。C++ 锚点 SURF#001/002 未暴露此 bug（hole() 分支在测试区域未触达）。
2. **surfaceNoiseThreshold /8.25 分歧 → 已核对，C++ 有 bug，Rust 应修正**：
   - Java：`surfaceNoiseThreshold(min)` = `noiseThreshold(SURFACE, min/8.25, MAX)`（VanillaSurfaceRules.java L391-392）。
   - C++：`noiseThresholdNoMax("minecraft:surface", 1.0)` 直接用 1.0（surface.h L519 等），**未除 8.25**。
   - **结论**：C++ surfaceNoiseThreshold 用错值（未除 8.25）。Rust 移植已修正为 `min/8.25`（windswept_* 系列 1.0/1.75/2.0/-1.0/-0.5/-0.95 全部除 8.25）。C++ 锚点未暴露（windswept 系列 biome 在测试区域未触达）。
3. **fill_chunk 接入点二选一**：改 fill_chunk 内部 vs 独立后处理 pass（对齐 C++ 独立 buildSurface 阶段）。
4. **biome 温度表数据来源**待定。
5. **placeIceberg** C++ 未实现，建议延后。
