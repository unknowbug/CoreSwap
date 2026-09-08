# W4 语义级 diff：biome / feature / structure（1.20.1 → 1.21.6 yarn）

- 日期块：mc-1216-port-260908-08
- 树 A：versions\1.20.1\data\mc_src_extract；树 B：versions\1.21.6\data\mc_src_extract
- 分工：3 个并行 worker（biome / structure / feature），主会话独立抽查 MultiNoiseUtil + VanillaBiomeParameters + Biome.isCold 交叉核对
- **置信度：draft**（静态审查 = Degraded 分层；无运行时验证）
- 分区详录：
  - `w4-section-biome.md`（biome + biome\source + source\util）
  - `w4-section-structure.md`（gen\structure + DimensionPadding）
  - `w4-section-feature.md`（gen\feature 注册表 + 算法类，原始 diff 在 `_raw/`）

---

## 一、★ 核心结论（主会话独立复核确认）

1. **MultiNoise 1e-4 定点 long 域分类器零变化**：`MultiNoiseUtil.toLong`（`(long)(value*10000.0F)`）、`ParameterRange.getSquaredDistance`、`SearchTree`/TreeNode 分割与距离、square-distance 全部逐行相同 → **Rust 1.20.1 biome 路由不需改**。
2. MultiNoiseUtil 全文件唯一算法级变化 = `FittestPositionFinder.calculateFitness`（仅 `/locate biome` 适应度公式：旧 `10000²·(r²/2500²)² + minDist` → 新 `minDist·2048² + r²`），不进 worldgen 热路径。
3. biome\source 全部 5 个重点文件仅 Codec→MapCodec 接口级重构；`VanillaBiomeParameters` 参数范围表无数值变化，仅 nearMountainBiomes 一格 `DARK_FOREST→PALE_GARDEN`（1.21 pale garden 数据项）。
4. structure/feature 辖区**无任何要求 1.20.1 Rust 复刻跟进的既有语义变化**；所有差异归属「1.21 新增内容」「接口/装饰级重构」「仅升 1.21.6 时才需处理的数据/算法点」。
5. 交叉核对（structure→biome 遗留项）：`Biome.isCold/getTemperature/computeTemperature` 新增 `seaLevel` 形参 = 接口级重构；高温衰减阈值 `80 → seaLevel+17`，主世界 seaLevel=63 时等价，公式逐行相同。RuinedPortal 判冷行为两版一致。

## 二、逐辖区变更清单（摘要）

### biome（详见 w4-section-biome.md）
- BiomeKeys/BuiltinBiomes/OverworldBiomeCreator/TheEndBiomeCreator：**数据级**，1.21 内容新增（pale_garden、armadillo/wolves 变体、END_PLATFORM、dry foliage）+ spawn/carver API 机械重排，1.20.1 数值逐一对应不变。
- SpawnSettings：SpawnEntry record 化 + weight 移入 Pool（**接口级**，序列化格式不变）。
- GenerationSettings：**接口级**——GenerationStep.Carver 枚举删除、carvers 单列化（升 1.21.6 数据时 carver 解析需改；1.20.1 不需改）。
- BiomeEffects：新增 dry_foliage_color / music_volume / music Pool（数据级新增）。
- Biome.java：除 seaLevel 重构外，仅上述等价改写。
- 新增 4 色彩类（BiomeColors/GrassColors/FoliageColors/DryFoliageColors）：旧客户端色值逻辑搬包 + 抽取共享索引公式，**算法逐位等价**；DryFoliageColors 为新通道。

### biome\source + util（详见 w4-section-biome.md）
- MultiNoiseBiomeSource / ParameterList(s) / TheEndBiomeSource：MapCodec 重构，语义不变。
- MultiNoiseUtil：见核心结论 1/2。VanillaBiomeParameters：见核心结论 3。

### gen\structure（详见 w4-section-structure.md）
- **DimensionPadding（新增）**：`record(bottom,top)`，jigsaw 专用维度边界保护区。消费链唯一：JigsawStructure → StructurePoolBasedGenerator.method_65173——中心 piece 包围盒侵入 `[worldBottom+bottom, worldTop-top]` 即整结构拒生；NoiseChunkGenerator/ChunkStatus 零引用，纯放置期过滤。vanilla 仅 TrialChambers 用 padding=10，其余默认 NONE（=1.20.1 行为）。**1.20.1 无此概念 → 不实现即对齐**（算法级，但仅影响 1.21+）。
- Structure.java：接口级（MapCodec、JFR profiling 形参）；新增 getAverageCornerHeights（仅 Shipwreck 用）；placement 随机流未变。
- Structures/StructureKeys：数据级（Config.Builder 写法重构，值逐一等价）；新增 TrialChambers（1.21 新结构）。
- **ShipwreckStructure = 本辖区唯一真算法变化**：1.21 对过大残骸做 Y 重定位（isTooLargeForNormalGeneration → beached ? findGroundedY(minCornerHeight) : 四角均值 setY）。1.20.1 无此逻辑。
- 其余 15 个具体结构类：装饰级/接口级（MapCodec）；NetherFortress spawn pool 数据级等价重写。

### gen\feature（详见 w4-section-feature.md；diff +1357/-492）
- **1.21 新增（可忽略）**：FallenTreeFeature(+Config) 与 5 个 fallen_* 键、pale oak/creaking 树系、*_leaf_litter 系、dry grass/firefly bush 等、END_PLATFORM、ORE_DIAMOND_MEDIUM（size8/count2/Y -64..-4）。
- **既有键数据级变化**：全部既有树群系 RANDOM_SELECTOR 键混入 fallen tree(0.0025~0.0125) 与 leaf litter 变体；id 改名（oak_bees_0002→oak_leaf_litter 等）；GRASS→SHORT_GRASS 全量改名（行为等价）；Ocean SEAGRASS_SIMPLE 整体删除。
- **既有算法级差异（复刻 1.20.1 应保持旧行为）**：HugeMushroom 系可放置判定 isOpaqueFullCube→air/REPLACEABLE_BY_MUSHROOMS；DiskFeature post-processing 标记逐层→每连续段一次；EndSpike 水晶下加火；SimpleBlock 新增 scheduleTick 字段（默认 false，旧数据不受影响）；GeodeFeature invalid-blocks 硬编码 tag→per-layer config。
- **接口级（不影响）**：Feature/FeatureSizeType MapCodec、getOrCreateCarvingMask 去 carver 步骤参数、getTopY→getTopYInclusive、DataPool→Pool、CocoaBeansTreeDecorator→CocoaTreeDecorator。
- **TreeFeature 核心放置流程/canReplace/叶更新 logistic 无变化**（仅边界改写 + 新静态工具）。
- 待核（worker @anchor.idk）：GeodeFeature per-layer invalidBlocks 的 1.21 默认值 vs 1.20.1 tag；FossilFeature place flags 常量值对应。

## 三、移植影响裁决（对维持 1.20.1 行为的 Rust 工程）

| # | 变更点 | 分类 | 1.20.1 复刻裁决 | 升 1.21.6 时 |
|---|--------|------|----------------|--------------|
| 1 | MultiNoiseUtil 定点量化/距离/SearchTree | 无变化 | **不需改** | 不需改 |
| 2 | FittestPositionFinder fitness 公式 | 算法级（仅 /locate） | 不需改 | 仅做 locate 时需改 |
| 3 | VanillaBiomeParameters.nearMountainBiomes DARK_FOREST→PALE_GARDEN | 数据级 | 不需改 | 需同步一格 |
| 4 | biome 注册表新增内容（pale_garden/armadillo/dry foliage/END_PLATFORM…） | 数据级（新增） | 不需改 | 新增数据 |
| 5 | GenerationSettings carver 单列化（Carver 枚举删除） | 接口级 | 不需改 | carver 解析需改 |
| 6 | Biome.computeTemperature seaLevel 化（80→seaLevel+17） | 接口级（主世界等价） | 不需改 | 参数化 seaLevel |
| 7 | SpawnEntry record 化 / MapCodec 系重构 | 接口级 | 不需改 | 随 codec 层重写 |
| 8 | 新增 4 色彩类 | 接口级（等价搬包） | 不需改 | 新增 dry foliage 通道 |
| 9 | **DimensionPadding + method_65173 边界拒生** | 算法级（仅 1.21+） | **不实现即对齐** | TrialChambers 等需实现 |
| 10 | **Shipwreck 过大残骸 Y 重定位** | 算法级（仅 1.21+） | 不需改（已知跨版本行为差） | 需实现 |
| 11 | Structures/StructureKeys：Config.Builder 重构 + TrialChambers 新增 | 数据级 | 不需改 | 新增数据 |
| 12 | 15 具体结构类 | 装饰级 | 不需改 | — |
| 13 | 树 RANDOM_SELECTOR 键混入 fallen/leaf litter、id 改名、GRASS→SHORT_GRASS、SEAGRASS_SIMPLE 删除 | 数据级 | 不需改（保持 1.20.1 数据） | 全量数据迁移 |
| 14 | HugeMushroom/Disk/EndSpike/SimpleBlock/Geode 既有算法差异 | 算法级（仅 1.21+ 行为） | **保持 1.20.1 旧行为** | 逐项跟进 |
| 15 | TreeFeature 核心放置算法 | 无变化 | **不需改** | 不需改 |

**总裁决**：1.21.6 相对 1.20.1 在本辖区**没有动摇 1.20.1 已对齐语义的既有变化**——所有差异要么是新增 1.21 内容，要么是仅 1.21+ 生效的行为变化，要么是接口/装饰级重构。维持 1.20.1 行为的 Rust 工程**零必修项**；升级路径上的必修/跟进项共 7 处（#3/#4/#5/#6/#9/#10/#13/#14 中标「升 1.21.6 需处理」者）。

## 附：验证分层与局限声明

- 分层：Degraded（纯静态源码审查 + git diff --no-index），无运行时探针验证。
- 主会话独立抽查覆盖：MultiNoiseUtil 全量 diff、VanillaBiomeParameters 全量 diff、Biome.isCold/computeTemperature 交叉核对（与分区 worker 结论一致）。
- 遗留 idk（feature worker）：GeodeFeature per-layer invalidBlocks 默认值、FossilFeature place flags 常量值——升级 1.21.6 前需补查，维持 1.20.1 不受影响。
