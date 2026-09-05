# scout1 — FEATURE 放置阶段管线地图（Rust worldgen-core ↔ Java 1.20.1 接线现状）

> recode-scout 只读勘探产物。状态：draft（静态勘探，Degraded 分层声明：未运行任何探针，全部结论来自源码/JSON 静态核对）。
> 上下文：feature-parity-charter-260905-04.md + 架构设计-feature-parity-260905-05.md（已批准）。
> 本文件只给地图与接线点，不解读差异根因。

## 1. NOISE → SURFACE → FEATURE 阶段顺序与边界

### Rust 侧（worldgen-core/src/worldgen_handle.rs，fill_chunk_blocks 主管线）
按执行顺序（fill_chunk_blocks，L497 起）：
1. **NOISE**（密度填充）：`fill_chunk(...)`（L677-686，按 gpu_channels/gpu_density/dfc_density/transpiler_density/macro_sampler 分发）+ aquifer（VanillaAquifer）+ ore_vein + beardifier —— L689 前后。
2. **SURFACE**：`self.sb.build_surface(...)`（worldgen_handle.rs **L630**，SurfaceBuilder，规则树来自 surface_rules.rs；env 门 `WG_SKIP_SURFACE`）。
3. **CARVERS**：`self.apply_carvers(&mut col, cx, cz, &mut va.aq, &biome_at)`（**L642**，定义 L711；env 门 `WG_SKIP_CARVER`）。
   ⚠️ 注意 Rust 执行序是 SURFACE→CARVERS→FEATURE；Java 是 CARVERS→SURFACE→FEATURE（DecorationSteps：CARVERS step 生成于 SURFACE 之前）。此序差异是否影响 feature 输入（heightmap/替换面）**列为疑点 Q1**。
4. **FEATURE**：`self.apply_features(&mut col, cx, cz, &heightmap, &biome_at)`（**L648** 调用，定义 **L819-942**；env 门 `WG_SKIP_FEATURES`，flag `FLAG_SKIP_FEATURES`）。注释（L816）自述对齐 C++ worldgen_api.cpp applyCarversAndFeatures 的 FEATURES 部分（C++ L1584-1674）。

apply_features 内部调度（L819-942）：
- OCEAN_FLOOR_WG 高度图逐列重扫（L828-842，top-down 第一个固体）。
- biome 采样两套：`biome_at_no_jitter`（chunk 角无 jitter，L844）/ `biome_at_jitter`（8 邻域 jitter，L849，供 pos_to_biome）。
- **简化①**：set = 当前 chunk 单一 biome（L857-858 `features_for(cur_biome)`）；注释自认 Java 是 3×3 chunk 所有 biome section。
- RNG：`ChunkRandom::xoroshiro()`（L862）→ `set_population_seed(seed, cx*16, cz*16)`（**L864**）→ 逐 step k 逐 p：`indexer.int_set_for(&cur_features, k)`（L872）→ `feat_random.set_decorator_seed(population_seed, p, k)`（**L879**）→ PlacedFeature → `generate_configured`（L913-918 附近，经 feature_cache 只读）。
- **简化②**：structure 部分（Java generateFeatures 先做 structure 引用的 intSet 合并）跳过（注释 L817）。

### Java 侧（versions/1.20.1/data/mc_src_extract/，yarn 源）
- 入口：`net/minecraft/world/gen/chunk/ChunkGenerator.java`
  - `generateFeatures(StructureWorldAccess, Chunk, StructureAccessor)` **L334**；
  - indexedFeaturesListSupplier = `PlacedFeatureIndexer.collectIndexedFeatures(biomes, GenerationSettings::getFeatures, true)`（**L100-103**）；
  - structure 分支（L341 groupingBy featureGenerationStep → intSet 合并，L385-389）；feature 主循环 L396-401：`indexedFeatures2.features().get(p)` → `placedFeature.generate(world, this, chunkRandom, blockPos)`（**L406**）；
  - decoration seed 在 ChunkStatusTask/ChunkPopulator 侧经 `setDecorationSeed`（ChunkRandom），Rust 等价物 = set_population_seed（chunkrandom.rs **L148-155**）。
- `setDecoratorSeed`：Java `ChunkRandom.java` L81（decorator seed 注释）；Rust：chunkrandom.rs **L158-161**（`l = populationSeed + index + 10000*step`，与 11-features-stage.md 版本敏感点一致）。
- `PlacedFeature.generate`：Java world/gen/feature/PlacedFeature.java（placement modifier 流惰性）；Rust：placement.rs `PlacedFeature::generate` **L447**。

### 阶段边界共享数据
| 数据 | Java | Rust |
|---|---|---|
| WORLD_SURFACE_WG/OCEAN_FLOOR_WG 高度图 | ChunkHeightmap（carver 前建 OCEAN_FLOOR） | Rust OCEAN_FLOOR 在 apply_features 内重建（L828），WORLD_SURFACE = fill_chunk_blocks 的 heightmap 参数 |
| biome 判定 | posToBiome（noise router） | `biome_pick_cell`（biome.rs）+ jitter/no-jitter 双闭包（worldgen_handle L844-853） |

## 2. ore feature 现状（Rust）

- **放置/形状**：`feature.rs` — `OreFeature::generate` **L213** + `generate_vein_part` **L242**（3D 矿脉）；`ScatteredOreFeature::generate` **L370**；`DiskFeature` **L439**；`SpringFeature` **L508**；`FreezeTopLayerFeature` **L547**；`UnderwaterMagmaFeature` **L598**。
- **replaceable/target 集合**：`RuleTest::parse`（feature.rs **L40-59**，tag_match/block_match/random_block_match 三型）+ `expand_tag`（**L65-93**）。
- **数据源现状**（核实 data-driven-boundary.md 标注属实，未过时）：
  - 矿石配置本身（targets/size/discard_chance）**数据驱动**：`configured_feature/*.json` → `OreFeatureConfig::parse`（feature.rs **L109-129**），加载链 feature_loader.rs `FeatureCache::preload_all`（L250-270 附近）。
  - **tag 展开硬编码**：`expand_tag` L65-93 手写 7 个 tag（base_stone_overworld / stone_ore_replaceables / deepslate_ore_replaceables / netherrack / base_stone_nether / sand / dirt）。数据目录确无 `tags/blocks`（L30 注释 + 勘探确认 worldgen-data 下无 tags/ 目录）。
  - 同类硬编码：carver replaceable（carver.rs `build_overworld_replaceable`，data-driven-boundary.md L24）；tree.rs `REPLACEABLE`（**L244-248**，#minecraft:replaceable_by_trees 13 成员手写）、`LOGS`（**L636-643**，BlockTags.LOGS 手写 21 成员）。
- JSON 数据权威位置（本仓库多份拷贝，权威 = `versions/1.20.1/data/worldgen/data/minecraft/worldgen/`）：coal_ore/andesite 等的 placed_feature + configured_feature JSON 齐备。

## 3. 树木 feature 现状（Rust tree.rs，700 行，260905-05 新移植）

- **主入口**：`TreeFeatureConfig::generate`（tree.rs **L440-503**）——对齐 TreeFeature.java:117-165：高度随机（L451-455）→ 域守卫 → getTopPosition（L506-519）→ trunk（straight L471-481 / large_oak L482-486）→ foliage（L488-490）→ decorators（L492-496）。
- **覆盖面（L1 范围，逐类核对）**：
  - TrunkPlacer：**straight + fancy(LargeOak) 仅 2 种**（L68-104；LargeOak 分支逻辑 L541-628）；Java 侧有 straight/large_oak/forking/giant/mega_jungle/dark_oak/bending/upwards_branching/cherry（trunk/*.java 共 9 种）→ **缺 7 种**（unsupported 显式告警跳过，generate L442-445 直接 return false）。
  - FoliagePlacer：**blob + fancy(LargeOak) 仅 2 种**（L106-155）；Java 侧 blob/dark_oak/fancy/jungle/mega_pine/pine/random_spread/spruce/acacia/cherry/bush（foliage/*.java 11 种）→ **缺 9 种**。
  - TreeDecorator：cocoa / trunk_vine / leave_vine 3 种（L258-353），与 jungle_tree.json 需求恰好对齐；vine 下垂逻辑（placeVines 向下 ≤4）在 LeaveVine L337-345；**属性位未编码**（vine face、cocoa age/facing、leaf distance——L304/L322 注释 + L497-501 placeLogsAndLeaves distance BFS 重算为 no-op，登记为 palette 已知偏差源）。
  - FeatureSize：仅 two_layers（L362-400）；three_layers unsupported（L382-384，标注 L3）。
  - BlockStateProvider：simple + weighted 2 种（L12-65）。
  - canReplace：REPLACEABLE_BY_TREES tag 硬编码 13 成员（L240-250）。
- **嵌套 feature**：random_selector / random_patch / simple_block 配置已 parse（tree.rs L647-700），但 generate 公式两处**显式占位**（feature_loader.rs L308-346，idk-7：selector 逐项 nextFloat 形态未与 RandomSelectorFeature.java 核对；patch tries 循环偏移合成待裁决）+ `generate_nested` 未接线（feature_loader.rs **L365-380**，恒 false + 告警）→ **树 via random_selector（trees_jungle/trees_oak 等 placed 标准形态）当前实际走占位公式**。疑点 Q3。
- jungle 侧数据事实：`jungle_tree.json` = straight_trunk + blob_foliage + cocoa/trunk_vine/leave_vine（**L1 全覆盖可走通**）；但 `trees_jungle.json`（placed）为 random_selector 包 fancy/mega → 走占位；`mega_jungle_tree.json` = mega_jungle_trunk_placer + jungle_foliage_placer → Unsupported 跳过。

## 4. 两侧同机制接线点清单（diff 用站点表）

| # | 机制 | Java（mc_src_extract 路径:行） | Rust（worldgen-core/src:行） | 接线状态 |
|---|---|---|---|---|
| W1 | feature 调度入口 | chunk/ChunkGenerator.java generateFeatures L334 | worldgen_handle.rs apply_features L819 | 已接，简化：单 biome（Java 3×3）+ structure 跳过 |
| W2 | 全局 feature 索引 p | feature/util/PlacedFeatureIndexer + ChunkGenerator L100-103, L388 | feature_loader.rs PlacedFeatureIndexer（build 于 worldgen_handle L368-370，biome.rs all_features_lists L496） | 已接（F-3 教训修正后全局构建） |
| W3 | populationSeed | ChunkRandom setPopulationSeed | chunkrandom.rs set_population_seed L148 | 已接 |
| W4 | decoratorSeed(l,p,k) | ChunkRandom.java L81 | chunkrandom.rs set_decorator_seed L158 | 已接（挂起项④：1/13 不匹配疑点，populationSeed 派生层为架构边界停手点） |
| W5 | placement modifier 流 | feature/PlacedFeature.java + placementmodifier/* | placement.rs PlacementModifier::get_positions L276-364 | 已接 13 型；NoiseBasedCount noise=0 占位（L357-363）；BiomeFilter 用 anchor 直通近似（L302-312） |
| W6 | IntProvider | feature/size/IntProvider*（placing 包） | placement.rs IntProvider L70-115 | 已接 uniform/trapezoid/biased_bottom/weighted_list/clamped |
| W7 | ore 矿脉 | feature/OreFeature.java + ScatteredOreFeature.java | feature.rs L213/L242/L370 | 已接（C++ 载具 ore desync 挂起项不在此范围） |
| W8 | RuleTest/tag | OreFeatureConfig.Anchor + BlockTags | feature.rs RuleTest::parse L40 + expand_tag L65 | 已接但 tag 硬编码（无 tags/blocks 数据源） |
| W9 | tree 主流程 | feature/TreeFeature.java L64-229 | tree.rs generate L440-503 | 已接（L1：straight/fancy oak 域） |
| W10 | trunk placer | trunk/*.java（9 类） | tree.rs L68-104（2 类） | 部分；缺 7 类 |
| W11 | foliage placer | foliage/*.java（11 类） | tree.rs L106-155（2 类） | 部分；缺 9 类 |
| W12 | tree decorator | treedecorator/*.java（cocoa/trunk_vine/leaves_vine/beehive/alter_ground 5 类） | tree.rs L258-353（3 类） | 部分；缺 beehive/alter_ground |
| W13 | 方块属性位 | BlockState Properties（distance/facing/age） | 全局缺失（state=i32 block id，无属性位） | **缺**（palette 对比已知偏差源，tree.rs L497-501） |
| W14 | selector/patch 嵌套 | feature/RandomSelectorFeature.java / RandomPatchFeature.java | feature_loader.rs L308-346（占位）+ generate_nested L365-380（未接线） | **占位/未接**（idk-7） |
| W15 | Feature 级分发 | feature/Feature.java register 静态表（~60 型） | feature_loader.rs generate_configured L278-361（7 型 + unknown→false） | 部分（ore/disk/spring/freeze/magma/tree/selector/patch/simple_block） |
| W16 | dispose/fluid/leaf 层判定 | TreeFeature.placeLogsAndLeaves + isAirOrLeaves 等 | tree.rs can_replace 系 L240-256, L631-645 | 已接但 tag 硬编码 |

## 5. 数据源现状汇总（JSON vs 硬编码）

- **JSON 数据驱动**：biome features 列表（biome.rs load_features L456，读 biome/*.json features[step][]）；placed/configured feature 全部参数（feature_loader.rs FeatureCache::preload_all）；ore targets/size；tree 的 provider/placer 参数值/decorators 列表/minimum_size 参数。
- **代码硬编码（无数据源，均已注释标注升级点）**：
  1. feature.rs expand_tag L65-93（7 个 RuleTest tag）
  2. carver.rs build_overworld_replaceable（carver replaceable tag）
  3. tree.rs REPLACEABLE L244-248（replaceable_by_trees）
  4. tree.rs LOGS L636-643（BlockTags.LOGS）
  5. tree.rs set_to_dirt soil 名单 L525-530（isSoil tag 主体）
  6. surface_rules.rs biome_temperature（biome 温度表，11-features-stage 冻结用）
- **占位/近似（非硬编码但有简化公式）**：NoiseBasedCount noise=0（placement.rs L360）、BiomeFilter anchor 近似（placement.rs L302-312）、random_selector/random_patch 公式（feature_loader.rs L308-346，idk-7）。
- 数据目录（versions/1.20.1/data/worldgen/data/minecraft/worldgen/）**确无 tags/blocks**——tag 数据驱动的前提缺口仍在。

## 6. 疑点清单（不解读、不下结论）

- **Q1**：Rust 执行序 SURFACE→CARVERS→FEATURE（worldgen_handle L630/642/648）vs Java GenerationStep 顺序（CARVERS 在 SURFACE 前）——顺序差异是否实际影响 feature 放置输入（高度图/替换面/RNG）？文档 01-architecture.md 口径待比对。
- **Q2**：apply_features 单 biome set（L857）vs Java 3×3 chunk biome section 合并——同 chunk 跨 biome 边界时 feature 集差异。
- **Q3**：trees_oak/trees_jungle 走 random_selector → 占位公式（idk-7）——树差异中多少流经此占位、多少流经形状本体，未分流。
- **Q4**：挂起项④ populationSeed 1/13 不匹配（11-features-stage.md L42 已永久挂起）与本课题「矿石只修规则层」边界（架构 §1）——ore 差异归因停在派生层即停手，派生层现状数值未在本次勘探复测。
- **Q5**：方块属性位全局缺失（W13）——jungle_leaves distance=7、vine face 等属性在 palette 对比中如何计差，对比脚本口径未核。
- **Q6**：Heightmap modifier 仅支持 OCEAN_FLOOR/其他=WORLD_SURFACE 二分（placement.rs L294）——Java HeightmapPlacement 对 heightmap 类型全集的语义核对未做。
- **Q7**：tree.rs 头注释自标「未编译验证」+ phase4-apply-report/phase4-status 已存在——patch 落地后编译/回归状态需 Phase 1 廉价验证确认（本勘探未跑 cargo）。
- **Q8**：structure setDecoratorSeed 独立重置（docs 11 版本敏感点）在 Java 侧消耗 RNG 与否，Rust 跳过 structure 分支是否等价未逐行核对（ChunkGenerator L341-389）。

## 附：关键参照文件路径速查
- Rust：worldgen-core/src/{worldgen_handle,placement,feature,feature_loader,tree,biome,chunkrandom}.rs
- Java：versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/{chunk/ChunkGenerator.java, feature/*.java, trunk/*, foliage/*, treedecorator/*, feature/util/PlacedFeatureIndexer}
- 数据：versions/1.20.1/data/worldgen/data/minecraft/worldgen/{biome,placed_feature,configured_feature}/*.json
- 文档：versions/1.20.1/docs/11-features-stage.md；worldgen-core/data-driven-boundary.md
- 前序勘探：.investigations/feature-parity/scout-feature-pipeline-260905-05.md、scout-random-chain-260905-05.md、s1-semantics-260905-05.md
