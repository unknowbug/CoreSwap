# W4 辖区：net/minecraft/world/gen/feature 语义级 diff（1.20.1 → 1.21.6）

- 置信度：**draft**（静态 diff 审查，Degraded 分层；未运行时验证）
- 方法：`git diff --no-index` 逐文件；原始 diff 留档于本目录 `_raw/*.diff`
- 分类口径：算法级（放置逻辑变化）/ 数据级（注册键/参数变化）/ 装饰级（命名/风格/API 签名，无语义）/ 接口级（类型/codec 体系变化）

## 一、注册表类逐个结论

### ConfiguredFeatures.java / PlacedFeatures.java
- 新增键：无。
- 结构性变化（装饰级）：`new Identifier(id)` → `Identifier.ofVanilla(id)`。
- 接口级（PlacedFeatures）：新增常量 `MOTION_BLOCKING_NO_LEAVES_HEIGHTMAP`（供新 feature 使用）；`createCountExtraModifier` 内 `DataPool`→`Pool`（重命名，无语义）。

### OreConfiguredFeatures.java / OrePlacedFeatures.java
- 新增键：**ORE_DIAMOND_MEDIUM**（configured：size=8, discard 0.5；placed：count 2, uniform Y -64..-4）——1.21 钻石分层改动，纯新增。另 DefaultBiomeFeatures 的矿石步骤在 ORE_DIAMOND 与 ORE_DIAMOND_LARGE 之间插入了它（见下）。
- 既有键参数：全部无变化（OrePlacedFeatures 逐条核对，仅注册变量序号位移）。

### TreeConfiguredFeatures.java
- 新增键（1.21 新内容）：
  - **PALE_OAK / PALE_OAK_BONEMEAL / PALE_OAK_CREAKING**（pale garden；DarkOak 树形参数复制 + PaleMossTreeDecorator / CreakingHeartTreeDecorator）
  - **OAK/DARK_OAK/BIRCH/FANCY_OAK_LEAF_LITTER**（树下 leaf litter，PlaceOnGroundTreeDecorator×2）
  - **FALLEN_OAK/BIRCH/SUPER_BIRCH/JUNGLE/SPRUCE_TREE**（FallenTreeFeature，见算法类节）
- 既有键参数变化（数据级，**影响 1.20.1 复刻**）：
  - `OAK_BEES_0002`：注册 id 改为 `oak_bees_0002_leaf_litter`，decorators 追加两个 PlaceOnGroundTreeDecorator（leaf litter）——**既有橡树/桦树/ fancy oak 的 bee 变体在 1.21 起附带地面 leaf litter**。
  - `BIRCH_BEES_0002`：新增平行键 `BIRCH_BEES_0002_LEAF_LITTER`（原键不变）。
  - `FANCY_OAK_BEES_0002`：同 OAK_BEES_0002（id 加 `_leaf_litter` + decorator）。
  - DARK_OAK：抽取 `darkOak()` helper，参数不变（纯重构）。
  - SAPLING_IS_VALID_GROUND / 可种树地面列表：加入 PALE_OAK_SAPLING、WILDFLOWERS（新增条目，不影响既有条目）。
  - `CocoaBeansTreeDecorator` → `CocoaTreeDecorator`（改名，语义不变）。

### TreePlacedFeatures.java
- 新增键：**PALE_OAK_CHECKED / PALE_OAK_CREAKING_CHECKED**、**OAK/DARK_OAK/BIRCH/FANCY_OAK_LEAF_LITTER**、**FALLEN_OAK/BIRCH/SUPER_BIRCH/SPRUCE/JUNGLE_TREE**（全部 wouldSurvive(sapling) 模式，1.21 新内容）。
- 既有键：注册逻辑无参数变化；仅 id 变化：`oak_bees_0002`→`oak_bees_0002_leaf_litter`、`fancy_oak_bees_0002`→`..._leaf_litter`、`birch_bees_0002` 键名保留但 1.21 新增 `birch_bees_0002_leaf_litter`（注意 1.21 的 `BIRCH_BEES_002_LEAF_LITTER` 字段指向 id `birch_bees_0002_leaf_litter`——命名与 0002 有出入，按 id 为准）。

### VegetationConfiguredFeatures.java
- 新增键（1.21 新内容）：PATCH_GRASS_MEADOW、PATCH_DRY_GRASS、PATCH_BUSH、PATCH_LEAF_LITTER、PATCH_FIREFLY_BUSH、WILDFLOWERS_BIRCH_FOREST/MEADOW、FLOWER_PALE_GARDEN、PALE_FOREST_FLOWERS、PALE_GARDEN_VEGETATION、PALE_MOSS_VEGETATION/PALE_MOSS_PATCH/PALE_MOSS_PATCH_BONEMEAL、TREES_BADLANDS、TREES_SNOWY、TREES_BIRCH。
- 既有键参数变化（数据级，**影响 1.20.1 复刻**）：
  - **`Blocks.GRASS` → `Blocks.SHORT_GRASS` 全量改名**（PATCH_TAIGA_GRASS、PATCH_GRASS、PATCH_GRASS_JUNGLE、SINGLE_PIECE_OF_GRASS、FLOWER_PLAIN、苔地下原始层等）——1.20.5 起草方块改名；1.20.1 复刻对应 `grass`，行为等价，纯重命名，但对照 1.21 数据文件时注意 id 不同。
  - **PATCH_CACTUS**：BlockColumnFeatureConfig 由单层改为两层——上层 cactus（1-3）+ 顶层 **CACTUS_FLOWER**（WeightedListIntProvider 0(w3)/1(w1)）——**1.21 仙人掌顶部会开花，数据级差异**。
  - **TREES_BIRCH_AND_OAK**：注册 id 改 `trees_birch_and_oak_leaf_litter`，且 RANDOM_SELECTOR 列表加入 fallen_oak(0.0125)/fallen_birch(0.0025)，尾选项改 oak_leaf_litter 变体——**既有键的树构成变化**。
  - **DARK_FOREST_VEGETATION**：dark_oak 槽位改用 DARK_OAK_LEAF_LITTER 变体（0.6666667），birch 槽位改 BIRCH_LEAF_LITTER 变体（0.2），fancy_oak 改 FANCY_OAK_LEAF_LITTER（0.1），并插入 fallen_oak(0.0125)/fallen_birch(0.0025)；尾选项 OAK→OAK_LEAF_LITTER 变体——**既有键变化**。
  - **TREES_TAIGA / TREES_GROVE / TREES_SAVANNA / TREES_PLAINS / TREES_SPARSE_JUNGLE / TREES_OLD_GROWTH_SPRUCE_TAIGA / TREES_OLD_GROWTH_PINE_TAIGA / TREES_JUNGLE / BIRCH_TALL / TREES_WINDSWEPT_HILLS / TREES_WATER / TREES_FLOWER_FOREST / MEADOW_TREES**：全部在 RANDOM_SELECTOR 中追加 fallen_* 树条目（rarity 0.0025~0.0125 不等）或槽位换成 *_LEAF_LITTER 变体——**几乎每个既有树群系键都有数据级变化**。
  - PATCH_SUGAR_CANE 系列：`wouldSurviveNearWaterModifier` 抽取为公共方法（纯重构，谓词不变）。
  - FLOWER_CHERRY：内联 builder → `flowerbed()` helper（纯重构）。
- 结构性（装饰级）：`DataPool`→`Pool` 全量重命名；新增 helper `leafLitter()/segmentedBlock()/flowerbed()/wouldSurviveNearWaterModifier()`（public，供 placed 类复用）。

### VegetationPlacedFeatures.java
- 新增键：PATCH_GRASS_MEADOW、PATCH_DRY_GRASS_BADLANDS/DESERT、PATCH_BUSH、PATCH_LEAF_LITTER、PATCH_FIREFLY_BUSH(_NEAR_WATER_SWAMP/_NEAR_WATER)、FLOWER_PALE_GARDEN、WILDFLOWERS_BIRCH_FOREST/MEADOW、PALE_GARDEN_VEGETATION、PALE_GARDEN_FLOWERS、PALE_MOSS_PATCH。
- 既有键：全部放置参数（count/rarity/heightmap）逐条核对**无变化**；唯一差异是引用的 configured 键换成 *_leaf_litter 变体（见上）与 `TREES_BIRCH_AND_OAK` id 改名。PATCH_DEAD_BUSH_BADLANDS 引用不变。
- 注意：**MEADOW 的草从 PATCH_GRASS_PLAIN 换为新 PATCH_GRASS_MEADOW**（DefaultBiomeFeatures.addMeadowFlowers，见下）。

### UndergroundConfiguredFeatures.java
- 新增键：无。
- 既有键参数：无变化。`Blocks.GRASS`→`SHORT_GRASS`（MOSS 层）；`new Identifier(...)`→`Identifier.ofVanilla(...)`（fossil 模板 id）；`DataPool`→`Pool`。装饰级为主。

### EndConfiguredFeatures.java / EndPlacedFeatures.java
- 新增键：**END_PLATFORM**（configured + placed；placed 用 `FixedPlacementModifier(ServerWorld.END_SPAWN_POS.down())`）——1.21 把末地出生平台从特殊逻辑改为常规 feature。
- 既有键：END_SPIKE/GATEWAY_RETURN/CHORUS_PLANT/END_ISLAND_DECORATED 放置参数不变。

### NetherConfiguredFeatures.java
- 仅 `DataPool`→`Pool`（装饰级）。无键变化。

### OceanConfiguredFeatures.java / OceanPlacedFeatures.java
- **删除键：SEAGRASS_SIMPLE**（configured + placed 均删）——1.20.1 中由 `DefaultBiomeFeatures.addSeagrassOnStone` 用于石岸/冷水海底carver液体掩膜放置；**1.21 里该 feature 整体移除，`addSeagrassOnStone` 方法也删除**。这是「既有条目消失」类差异。
- 其余键（sea_pickle/kelp/warm_ocean_vegetation 等）参数不变。

### PileConfiguredFeatures.java
- 仅 `DataPool`→`Pool`。无键变化。

### DefaultBiomeFeatures.java（生物群系装配层）
- 新增方法：addBushes、addBirchForestWildflowers、addLeafLitter、addMangroveSwampAquaticFeatures、addBatsAndMonsters(builder, skeletonWeight) 重载；addDesertDryVegetation（替代 addDesertDeadBushes，加入 PATCH_DRY_GRASS_DESERT）。
- 既有方法语义变化（数据级）：
  - **addLandCarvers**：carver 注册不再带 GenerationStep.Carver 步骤参数（1.21 carver step 由 carver 自带；接口级，复刻 1.20.1 不受影响）。
  - **矿石步骤**：在 ORE_DIAMOND 与 ORE_DIAMOND_LARGE 之间插入 ORE_DIAMOND_MEDIUM（影响 1.21 钻石分布；对 1.20.1 复刻=新增内容）。
  - **addMeadowFlowers**：PATCH_GRASS_PLAIN → PATCH_GRASS_MEADOW（新键，tries 16 且带噪声阈值 placed）。
  - **addDefaultVegetation**：签名改为 `(builder, boolean includeNearWater)`，**PATCH_SUGAR_CANE 移入条件分支**，并新增 PATCH_FIREFLY_BUSH_NEAR_WATER——甘蔗放置不再是默认无条件。
  - **addBadlandsGrass**：追加 PATCH_DRY_GRASS_BADLANDS；addBadlandsVegetation 追加 PATCH_FIREFLY_BUSH_NEAR_WATER；addSwampVegetation 追加 PATCH_FIREFLY_BUSH + _NEAR_WATER_SWAMP。
  - **删除 addSeagrassOnStone**（配合 SEAGRASS_SIMPLE 删除）。
  - **Spawn API 全面变化**：`builder.spawn(group, new SpawnEntry(type, weight, min, max))` → `builder.spawn(group, weight, new SpawnEntry(type, min, max))`（weight 外提；camel 加入沙漠）。语义等价，接口级。

## 二、算法类逐个分类

| 类 | 分类 | 说明 |
|---|---|---|
| **TreeFeature** | **算法级（轻）** | ① 边界：`n <= world.getTopY()` → `n <= world.getTopYInclusive() + 1`（等价+1 边界语义，1.20.5 起 getTopY 定义变化；对 1.20.1 复刻=保持原式即可，数值等价）。② `StructureTemplate.updateCorner` 参数 `3` → `Block.NOTIFY_ALL`（同值常量化，无语义）。③ isVine 改 public；新增静态 `getLeafLitterPositions`（供 leaf-litter 装饰器）。**canReplace、日志/叶放置主流程、post-processing 无变化**。 |
| **FallenTreeFeature（新增）** | 新增类（1.21） | 桩+随机水平倒木：stump 必放；log 从 origin 偏移 2~3 格起、`logLength-2` 段；moveToGroundPos 向下找最多 6 格；canPlaceLog 允许悬空 ≤2 段；axis 对齐方向；decorator 走 TreeDecorator.Generator（logDecorators/stumpDecorators）。复刻 1.20.1 **不需要**。 |
| **FallenTreeFeatureConfig（新增）** | 新增类 | trunkProvider + UniformInt logLength + logDecorators/stumpDecorators。 |
| **Feature** | **接口级** | `codec` 类型 `Codec<ConfiguredFeature>` → **`MapCodec`**（`fieldOf("config").xmap(...).codec()` → 直接 xmap 得 MapCodec）；注册新增 `FALLEN_TREE`、`END_PLATFORM` 两个 feature 实例。 |
| **FeaturePlacementContext** | 接口级 | `getOrCreateCarvingMask(chunkPos, carver)` → `getOrCreateCarvingMask(chunkPos)`（1.21 ProtoChunk 单一 carving mask；1.20.1 按 carver step 分 mask。**对复刻 1.20.1：保持 per-step mask**）。 |
| **SimpleBlockFeature / Config** | **算法级（条件性）** | 新增 `scheduleTick` 字段（默认 false，codec optional）与 PaleMossCarpetBlock placeAt 特例。1.20.1 语义 = scheduleTick(false) 分支，**既有数据不受影响**；仅新增键（eyeblossom 等）用 true。 |
| **RandomPatchFeatureConfig** | 装饰级 | `Codecs.NONNEGATIVE_INT` → `Codecs.NON_NEGATIVE_INT`（常量改名）。 |
| **DiskFeature** | **算法级** | `markBlocksAboveForPostProcessing` 从「每个命中层」改为「每段连续命中只标记一次」（bl2 连续段跟踪）。1.20.1 复刻应保持逐层标记（与 1.20.1 一致）；此差异仅当 post-processing 语义敏感时可见。 |
| **MultifaceGrowthFeature / Config** | 装饰级 | 字段改名 `lichen`→`block`、错误消息变化；逻辑逐行等价。 |
| **HugeMushroomFeature** | **算法级** | `generateStem` 可放置判定从 `!isOpaqueFullCube` 改为 `isAir() \|\| isIn(REPLACEABLE_BY_MUSHROOMS)`；`canGenerate` 上界 `getTopY()` → `getTopYInclusive()`（边界+1 等价改写）。 |
| **HugeBrownMushroomFeature / HugeRedMushroomFeature** | **算法级** | 菌盖方块放置由「跳过不透明完整方块」改为统一走 `generateStem`（可替换标签判定）——**菌盖可替换的方块集合变化**（例如水/草类方块 1.20.1 下不透明判定与 1.21 replaceable 判定可能不同）。复刻 1.20.1 保持 isOpaqueFullCube。 |
| **EndPlatformFeature（新增）** | 新增类 | 5×5×4 obsidian 底 + air 清空，`breakBlocks` 开关；END_PLATFORM 注册默认不 break。 |
| **EndPortalFeature** | 算法级（轻微） | `open` 模式下改用 `place()` 幂等放置（已存在则不 break/重设）——结构重复生成的幂等性变化，首次生成语义不变。 |
| **EndSpikeFeature** | 算法级（轻微） | 水晶基岩/火：由「spike 中心固定坐标 bedrock」改为「crystal 所在方块下 bedrock + 该方块放 FireBlock.getState（火）」——**末地水晶下会生成火**（1.21 行为）。 |
| **EndGatewayFeature** | 装饰级 | BlockEntity 局部变量内联、去 markDirty 调用；语义等价。 |
| **CoralFeature** | 装饰级 | `getEntryList().flatMap(getRandom)` → `getRandomEntry(tag, random)`（等价 API）。 |
| **BasaltColumnsFeature** | 算法级（边界等价） | `getY() < getTopY()` → `getY() <= getTopYInclusive()`（数值等价）。 |
| **BlockPileFeature** | 装饰级 | 仅 DataPool→Pool（diff 2 行）。 |
| **BonusChestFeature** | 接口级 | LootableContainerBlockEntity.setLootTable → LootableInventory.setLootTable。 |
| **DripstoneClusterFeature / DungeonFeature / FossilFeature / GeodeFeature / HugeFungusFeature / LargeDripstoneFeature / NetherForestVegetationFeature / ReplaceBlobsFeature / UnderwaterMagmaFeature** | 混合 | Dungeon：LootableInventory API（接口级）。Fossil：`place(...,4)` → `place(...,Block.SKIP_REDRAW_AND_BLOCK_ENTITY_REPLACED_CALLBACK)`（常量化，值语义待核，倾向等价）+ getTopYInclusive 边界。LargeDripstone/UnderwaterMagma：`isPresent()`→`!isEmpty()`、`Box`→`BlockBox`（等价）。**GeodeFeature：算法级例外——invalid blocks 判定从硬编码 `BlockTags.GEODE_INVALID_BLOCKS` 改为读 `geodeLayerConfig.invalidBlocks`（per-layer 配置）**；1.21 数据若与 1.20.1 tag 相同则行为一致，但这是数据源变化。HugeFungus/NetherForestVegetation/ReplaceBlobs/DripstoneCluster：仅重命名/风格。 |
| **size/FeatureSizeType** | 接口级 | `Codec`→`MapCodec`（同 Feature）。ThreeLayers/TwoLayersFeatureSize：仅风格。 |
| **util/FeatureDebugLogger / PlacedFeatureIndexer** | 装饰级 | 风格/重命名。 |

## 三、共性装饰级变化（全辖区）
1. `DataPool` → `Pool`（+ `WeightedListIntProvider(Pool)`），纯重命名。
2. `new Identifier(id)` → `Identifier.ofVanilla(id)`。
3. `Codec` → `MapCodec`（Feature / FeatureSizeType 序列化体系）。
4. `world.getTopY()` → `world.getTopYInclusive()`（边界语义重定义，多数改写数值等价）。
5. `optional.isPresent()` → `!optional.isEmpty()`。
6. `Block.NOTIFY_*` 常量替换魔法数字（3→NOTIFY_ALL 等）。

## 四、小结：既有语义变化中对 1.20.1 Rust 复刻的影响

**复刻不受影响（1.21 新增/仅 1.21 世界出现）**：pale oak/creaking、fallen tree、leaf litter、dry grass、firefly bush、bush、wildflowers、cactus flower、END_PLATFORM、ORE_DIAMOND_MEDIUM、pale moss patch 全部为新增键。

**需要留意的「既有键/算法」差异（若将来参照 1.21 数据文件或行为对照时）**：
1. **几乎全部既有树群系 RANDOM_SELECTOR 键（trees_plains/taiga/savanna/jungle/birch_tall/dark_forest_vegetation/trees_birch_and_oak 等）在 1.21 混入了 fallen tree 与 *_leaf_litter 变体**——对比 1.20.1 与 1.21 的实际树密度/构成时这不是算法差异而是数据差异；1.20.1 复刻数据保持原样即可。
2. **OAK_BEES_0002 / FANCY_OAK_BEES_0002 的 id 加了 `_leaf_litter` 后缀**——跨版本数据管线注意 id 映射。
3. **Blocks.GRASS → SHORT_GRASS 改名**（1.20.5+）：与 1.21 数据对照时 id 不同、行为相同。
4. **Ocean SEAGRASS_SIMPLE 键删除**（连同放置与 biome 装配）——既有键消失类。
5. **HugeMushroom 系可放置判定 isOpaqueFullCube → air/REPLACEABLE_BY_MUSHROOMS**：真算法级差异，1.20.1 复刻保持旧行为。
6. **DiskFeature post-processing 标记从逐层改为每连续段一次**：真算法级差异，1.20.1 复刻保持逐层。
7. **GeodeFeature invalid-blocks 判定从硬编码 tag 改为 per-layer config**：数据源变化（默认值若同 tag 则行为一致，未核 1.21 数据文件值，见 @anchor.idk）。
8. **getTopY→getTopYInclusive / getOrCreateCarvingMask 合并**：世界高度/掩膜边界 API 语义重定义；1.20.1 复刻按 1.20.1 语义（getTopY = 排他上界、per-carver-step mask）保持。
9. EndSpike 水晶下加火、EndPortal 幂等放置、SimpleBlock scheduleTick：1.21 行为变化，均不影响 1.20.1 复刻。

@anchor.idk("GeodeFeature per-layer invalidBlocks 的 1.21 数据文件默认值是否与 1.20.1 GEODE_INVALID_BLOCKS tag 完全一致——本次仅核对了源码，未比对 data/minecraft/worldgen 配置", source="memory:w4-section-feature-diff-260908")
@anchor.idk("FossilFeature StructureTemplate.place 第 6 参由 4 改为 SKIP_REDRAW_AND_BLOCK_ENTITY_REPLACED_CALLBACK，两常量数值是否相等未核（影响仅限 update 结构更新标志）", source="memory:w4-section-feature-diff-260908")
