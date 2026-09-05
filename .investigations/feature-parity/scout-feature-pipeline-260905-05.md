# scout-feature-pipeline-260905-05 — Rust 侧 worldgen feature 放置管线现状地图

> 角色：recode-scout（只读勘探，不分析定论）。置信度：全部 **draft**。验证分层：Degraded（纯静态代码审读，无运行时 probe）。
> 日期标签：260905-05（沿用任务指派命名；宿主日期未独立重取——按 260904-06 纪律此为流程瑕疵标注，正文内容不含日期推算）。
> 参照权威：Java 1.20.1 vanilla（本勘探未读 Java 源码工程本体，仅读仓库内文档 versions/1.20.1/docs/11-features-stage.md + Rust 代码内注释的 Java 行号引用）。

## 1. 模块路径地图（文件:行号级）

### 1.1 worldgen-core（算法引擎，包名 WorldgenRust rlib）

| 文件 | 职责 | 关键位置 |
|---|---|---|
| `worldgen-core\src\worldgen_handle.rs` | 阶段调度总控（fill_chunk_blocks） | FEATURE 入口 `apply_features` **L819-928**；调度调用点 **L645-652**；CARVERS `apply_carvers` **L710-799**；SURFACE 调用 **L629-638**；carver JSON 预加载 `load_carver` **L802-808**；feature 缓存/indexer 构建 **L366-383** |
| `worldgen-core\src\placement.rs` | PlacementModifier + PlacedFeature | `IntProvider` **L14-111**（uniform/trapezoid/biased_to_bottom/weighted_list/clamped/constant）；`FeaturePlacementContext` **L115-129**；`PlacementModifier` 枚举 **L133-144**，`get_positions` **L147-209**，`parse` **L211-273**；`PlacedFeature::generate`（深度优先惰性复刻）**L289-309** |
| `worldgen-core\src\feature.rs` | Feature 实现 | `RuleTest` **L20-60**；tag 硬编码展开 `expand_tag` **L65-93**；`OreFeatureConfig` **L97-130**；`OreFeatureContext`（含跨 chunk pending 写）**L135-207**；`OreFeature`（含 generateVeinPart）**L210-332**；`ScatteredOreFeature` **L368-390**；`DiskFeature` **L437-462**；`SpringFeature` **L506-534**；`FreezeTopLayerFeature` **L544-567**（温度近似，`biome_temp < 0.0` 简化 **L548**）；`UnderwaterMagmaFeature` **L596-653** |
| `worldgen-core\src\feature_loader.rs` | 数据加载 + 调度 | `ConfiguredFeature::parse`（type 分发）**L27-53**；`PlacedFeatureIndexer`（lastIndex p 值语义）**L62-139**；`FeatureCache` 懒/预加载 **L142-228**（`preload_all` L193-227）；生成分发 `generate_configured` **L232-260** |
| `worldgen-core\src\biome.rs` | biome features/carvers 数据源 | `features: BTreeMap` **L233-235**（BTreeMap 为修 nether 不确定序，2026-08-30）；`load_features` **L456-488**（读 `biome/*.json` 的 `features[step][]`）；`load_carvers` **L395-432**；`features_for` **L491-493**；`all_features_lists` **L496-498** |
| `worldgen-core\src\ore_vein.rs` | NOISE 阶段矿脉（非 feature 管线） | `OreVeinSampler` **L11-69**；copper/iron VeinType 硬编码块名+范围 **L31-32**（copper granite 0..50，iron tuff/deepslate -60..-8）；调用点 worldgen_handle.rs **L701-704** |

### 1.2 versions\1.20.1\rust（薄壳）

| 文件 | 职责 |
|---|---|
| `src\lib.rs`（3 行） | 仅 `pub mod jni_bridge`——算法全在 worldgen-core |
| `src\jni_bridge.rs` | JNI ABI 适配；句柄级阶段开关 flag（bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE，见 worldgen_handle.rs L99/L109-110）；L71 注释：双跑模式下 Java CppBridge 可关 Rust carver/features |

### 1.3 数据目录（wg_dir = `versions\1.20.1\data\worldgen\data\minecraft\worldgen\`）

- placed_feature: **231** 个 JSON、configured_feature: **194**、biome: **64**、configured_carver: **4**——数据齐全（含 oak.json / patch_tall_grass.json 等植被 feature JSON 均存在，但见 §3 未实现项）。

## 2. 阶段顺序图（Rust fill_chunk_blocks，worldgen_handle.rs L560-655）

```
1. NOISE 宏观填充（fill_chunk：density + aquifer + ore_vein 采样）
   └─ ore_vein.apply（L701-704，Rock 判定内逐块；copper/iron 矿脉替换 stone→granite/tuff 系）
2. SURFACE（build_surface，L629-638；surface_rules.rs，overworld 代码规则）
3. CARVERS（apply_carvers，L640-643 → L710-799；17×17 邻域，cave/canyon/cave_extra）
4. FEATURES（apply_features，L645-652 → L819-928）
   ├─ OCEAN_FLOOR_WG 高度图逐列重建（L828-842）
   ├─ cur_biome = chunk 角 biome（L857）→ features[step][]（L858）
   ├─ ChunkRandom::xoroshiro + set_population_seed(seed, cx*16, cz*16)（L862-864）
   ├─ for step k in 0..max_step：int_set（lastIndex p 集合，L871-873）
   │    for p in int_set：fid = step_features[k][p]（L878）
   │      set_decorator_seed(population_seed, p, k)（L879）
   │      PlacedFeature.generate（modifiers 链深度优先，L924）
   │        → generate_configured 分发（feature_loader.rs L232-260）
   └─ 返回放置块数（诊断）
```

- **ore feature（ore/scattered_ore 配置型）入口** = `generate_configured`（feature_loader.rs L241-247）→ `OreFeature::generate` / `ScatteredOreFeature::generate`（feature.rs）。
- **tree feature 入口 = 不存在**：`generate_configured` L257-258 对 tree/flower/random_patch/simple_block/random_selector 显式返回 false（「2026-08-10 用户拍板范围外」，feature_loader.rs L50 注释同）。
- 调度简化（L817 注释自认）：Java 用 3×3 chunk 所有 biome section 的 feature set，Rust 只用当前 chunk 单 biome；structure 部分跳过。
- 顺序 NOISE→SURFACE→CARVERS→FEATURES 与 Java 一致（docs/11 篇同口径）；Java 侧 SURFACE 与 FEATURES 的实际先后（Java 先 FEATURES 后 SURFACE？——不，1.20.1 为 noise→surface→carvers→features→...）**@anchor.idk：本勘探未对 Java ChunkStatus 顺序做独立核对，仅引 docs/11 篇口径，draft**。

## 3. 数据驱动 vs 硬编码清单

### 3.1 数据驱动（从 JSON 读）
| 项 | 源 | 加载点 |
|---|---|---|
| biome features[step][] 列表 | `biome/*.json` | biome.rs load_features L456 |
| carvers.air 列表 | `biome/*.json` | biome.rs load_carvers L395 |
| placed_feature 的 placement modifiers 链 + configured 引用 | `placed_feature/*.json`（231 个） | feature_loader.rs preload_all L193-227 |
| configured_feature 的 ore/disk/spring/magma 配置（targets/size/discard_chance/radius 等） | `configured_feature/*.json`（194 个） | feature.rs parse L109/407/481/578 |
| **ore 配置整体（coal_ore/iron_ore… 的 targets RuleTest、size、y 分布 height_range）** | 同上 JSON | —— ore 配置本身不硬编码 |
| block id | blocks.json | blocks.id |

### 3.2 代码硬编码（无数据源）
| 项 | 位置 | 说明 |
|---|---|---|
| RuleTest tag 展开（base_stone_overworld / stone_ore_replaceables / deepslate_ore_replaceables / nether 系 / sand / dirt 共 8 tag） | feature.rs expand_tag L65-93 | 已注释标注跨版本核对点；**stone_ore_replaceables 含 stone/granite/diorite/andesite——即 stone↔ore、granite↔ore 替换判定的核心数据硬编码在此** |
| carver replaceable | carver.rs build_overworld_replaceable（data-driven-boundary.md L24 登记，本勘探未逐行核） | 同类升级点 |
| ore_vein 的 VeinType（copper=granite/y0..50，iron=tuff,y-60..-8） | ore_vein.rs L31-32 | 块名字符串 + y 范围硬编码 |
| FreezeTopLayer 降水近似（precipitation==SNOW 用 temp<0 近似） | feature.rs L548 | 语义简化 |
| NoiseBasedCount 的 noise 采样 = 常量 0.0 | placement.rs L201-207 | Phase 3 简化，未接 noise sampler |

## 4. 缺失 / 可疑点清单（-feature parity 课题关注面）

1. **【最大缺口】tree/植被 feature 完全未实现**（feature_loader.rs L50/L257-258 显式范围外，2026-08-10 拍板）：tree/random_selector/flower/random_patch/simple_block 全部 generate=false。→ **Java 侧树叶/藤蔓/树干在 Rust 输出中整体缺失**（不是「有分歧」而是「无此阶段产物」）。若光照课题残差含树类方块差异，这是第一候选根因。注意 260904-15 挂起决策仅挂起 ore/populationSeed 疑点，未涉及 tree 实现决策——是否解禁属主会话/用户裁决。
2. **PlacementModifier 覆盖不全（10/15+）**：已实现 count/rarity_filter/in_square/height_range/heightmap/biome/random_offset/block_predicate_filter/surface_relative_threshold_filter/noise_based_count。Java 尚有 count_on_every_layer / environment_scan / heightmap_spread_double / carved_mask 等——`parse`（placement.rs L211-273）对未识别 type 返回 **None → modifier 被静默丢弃**（preload_all L210 同），植被类 placed_feature 大量使用这些 modifier，即使实现 tree 也会位置错。静默丢弃无告警 = 可疑点。
3. **Biome modifier 是 no-op 直通**（placement.rs L173-178，「简化：直接返回」）——biome 过滤未做，cross-biome 植被会越界放置。
4. **NoiseBasedCount noise 恒 0**（placement.rs L204）——`minecraft:ore_…` 不用此 modifier，但部分植被计数用，当前恒为 count 下限。
5. **ore 放置位置与 vanilla 仅 1/13 匹配**（docs/11 篇已知限制，populationSeed/setDecoratorSeed 序列疑点）——**已由 260904-15 用户拍板永久挂起**（含 populationSeed 疑点 + ore_vein 域 ①vein pre 态 + aquifer 域 ②两项真实方块级残差）。光照课题引用残差源时注意：这两项是**已登记的真实差异源**，勿当新发现重查。
6. **features set 3×3 简化**（worldgen_handle.rs L817）：Java 用 3×3 chunk 全部 biome section 并集，Rust 只用当前 chunk 角 biome——chunk 边界 feature 集不同 → step/indexer 语义在此场景偏离。
7. **跨 chunk 读写半成品**：OreFeatureContext 支持 region_col_at/pending_cross 回调（feature.rs L146-148），但 apply_features L907-908 传 **None**——树冠/矿脉越 chunk 部分直接丢弃（isExposedToAir 越界按 -1≈非空气处理，feature.rs L349-350，方向与 Java ChunkSectionCache 默认 air 相反，可疑）。
8. **FreezeTopLayer 硬编码先 ice 后 snow 无条件覆盖**（feature.rs L552-563，只判 biome_temp<0）——无 canSetIce 的水判定，海洋/河流结冰语义简化，与「-288 冷洋不冻结」注释存在张力（draft 存疑）。
9. **block_predicate_filter 只支持 matching_fluids/matching_blocks 两种谓词**（placement.rs L234-254），Java 尚有 would_survive/solid/replaceable/inside_world_bounds 等——植被生存判定缺口。
10. IntProvider::BiasedToBottom 为**自认近似**（placement.rs L45 注释「Java 更复杂」）——ore 高度分布用 biased_to_bottom 的（如部分 ore placed_feature）会有 y 偏差。
11. **granite↔andesite 替换层疑点**：ore_vein.rs 只做 copper(granite)/iron(tuff) 两族，未见 andesite 写入；andesite 出现于 expand_tag 的 stone_ore_replaceables（作为被替换方，非产物）与 surface rule。光照课题所称「granite↔andesite 替换层」分歧在 feature 管线内的具体落点 **@anchor.idk：需 worker 定位（候选：ore feature targets JSON vs ore_vein vs surface rule），本勘探不下结论**。

## 5. Java 参照类 → Rust 对位物（据 docs/11 + 代码注释，未读 Java 本体）

| Java（yarn） | Rust 对位 | 状态 |
|---|---|---|
| Feature / OreFeature / ScatteredOreFeature | feature.rs OreFeature/ScatteredOreFeature | 已实现（1/13 序列疑点挂起） |
| DiskFeature / SpringFeature / FreezeTopLayerFeature / UnderwaterMagmaFeature | feature.rs 同名 | 已实现（含简化，见 §4.8） |
| TreeFeature / RandomPatch / SimpleBlock / RandomSelector / Flower（VegetationPlaced 等） | **无** | 未实现（范围外） |
| PlacedFeature | placement.rs PlacedFeature | 已实现（深度优先惰性已对齐） |
| PlacementModifier 族 | placement.rs PlacementModifier | 10 种；缺 count_on_every_layer/environment_scan/heightmap_spread_double/carved_mask 等 |
| PlacementContext / FeaturePlacementContext | placement.rs FeaturePlacementContext | 已实现（block_at 恒 None，worldgen_handle.rs L895） |
| PlacedFeatureIndexer / IndexedFeatures | feature_loader.rs PlacedFeatureIndexer | 已实现（lastIndex 语义 + BTreeMap 确定序） |
| RuleTest 族 | feature.rs RuleTest | 3 种 + tag 硬编码展开 |
| ChunkGenerator.generateFeatures / setDecoratorSeed | worldgen_handle.rs apply_features | 已实现（3×3 简化） |
| OreVeinSampler（NOISE 阶段，非 feature） | ore_vein.rs | 已实现 |

## 6. 待深入点（交 worker 的建议入口）

- W1：照明残差中方块分类统计——先量化残差里 树叶/原木/藤蔓 占比 vs ore/替换层占比，决定 tree 实现 vs ore 序列对齐哪个先做（fan-out 候选）。
- W2：granite↔andesite 分歧落点定位（§4.11）。
- W3：若解禁 tree：需先补 placement modifier 缺失族（§4.2/§4.9）+ Biome modifier 实装（§4.3）+ 跨 chunk 写（§4.7），否则 tree 位置必错。
- W4：`WG_FEATURELOG`（worldgen_handle.rs L650/880/918）为现成诊断钩子，可做 Java 端 feature 步骤对照的第一数据源。

## 7. 混淆/边界声明

- 本勘探为 Degraded（纯静态）；所有「Java 语义」引述转引自 docs/11-features-stage.md 与代码注释，未对 Java 源独立核对——凡标 Java 行号处均为二手，worker 使用时需回源验证。
- 未修改任何代码；仅写本产物文件。
