# batchC worker 交付 —— mc-1216 features 接管批次 C（unknown 残差清空 11/16 + nether_forest_vegetation + fixed_placement；5 项声明缓装）

> 角色：worker（代码交付）。**未编译验证**（沙箱无 shell，静态对拍交付，未运行任何命令）。
> 一手源：`versions/1.21.6/data/mc_src_extract/net/minecraft/`（主）+ `versions/1.20.1/data/mc_src_extract/net/minecraft/`（**存在，已双版交叉核对**——本批 17 个实装对象中 16 个两版逐行一致（Compare-Object 实证），唯一 diff = NetherForestVegetationFeature.java 2 行（`getTopY()` → `getTopYInclusive()` 同义改名，见 §一.11），详见 §四 E-4）。
> 引擎参照：`worldgen-core/src/feature.rs`（batchA/B 追加段：set_block_if_allowed / get_top_y_ocean_floor / kelp_can_place_at 复用）/ `feature_loader.rs`（ConfiguredFeature 扩字段 + parse/generate 分发模式）/ `placement.rs`（PlacementModifier / BlockPredicate / IntProvider 模式）/ `tree.rs`（is_solid_id / BlockStateProvider / RandomSelectorConfig 嵌套 placed 模式）。
> 状态：draft（候选需主会话应用 + 编译 + 实跑 unknown 残差验收后才可升 candidate）。
> 日期标签：本文不写 YYMMDD 标签（260904-06 纪律：主会话落盘/归档前 Get-Date 取真实时间补）。

## 〇、本批覆盖面声明

### §〇.1 实装项（11 configured type + 1 nether 族 type + 1 modifier = 13）

| # | 目标 type | Java 类 | config | 本批做了 | 主要残差（→ idk） |
|---|-----------|---------|--------|----------|-------------------|
| 1 | minecraft:bamboo | BambooFeature（ProbabilityConfig） | probability 1 字段 | **全量**：air 门 → canPlaceAt 近似门 → nextInt(12)+5 → podzol 圈（WORLD_SURFACE 采样 + isSoil）→ 竹秆循环 → 顶 3 格 | AGE/LEAVES/STAGE 属性丢弃；canPlaceAt → {bamboo,bamboo_sapling}∨dirt tag 近似；podzol 圈邻域 getTopY 走 block_at 下扫（idk-2） |
| 2 | minecraft:block_column | BlockColumnFeature + Config | layers/direction/allowed_placement/prioritize_tip | **全量**：逐层 height.get → allowed_placement 前探 + adjustLayerHeights 逐行镜像 → 分层放置 | 需 **IntProvider::WeightedProviders 新变体**（dripleaf/cave_vine 层高 = weighted_list 嵌套 IntProvider，E-3）；randomized_int_state_provider tip 取 source（age/values 丢弃）；属性位全丢 |
| 3 | minecraft:blue_ice | BlueIceFeature | DefaultFeatureConfig | **全量**：海平面/水/packed_ice 邻接门 + 200 轮蔓生 | getSeaLevel 硬编码 63（idk-4） |
| 4 | minecraft:desert_well | DesertWellFeature | DefaultFeatureConfig | **全量**：落底 + sand 门 + 5×5 双层 air 门 + 全结构（井体/水/sand/rim/slab/顶/柱）+ 2 次 suspicious_sand | 方块实体/loot no-op（Util.getRandom nextInt(5)×2 消费保留）；SUSPICIOUS_SAND 仅裸 id |
| 5 | minecraft:forest_rock | ForestRockFeature + SingleStateFeatureConfig | state 固定 BlockState | **全量**：落底（isSoil∨isStone 门）+ 3 轮椭球 | 无属性面 |
| 6 | minecraft:ice_spike | IceSpikeFeature | DefaultFeatureConfig | **全量**：snow_block 门 + 菱形截面主循环（:46 短路求值序逐字对拍）+ 底柱 p=50 下探 | setBlockState 无条件写近似；`by > 50` 字面量同 Java 保留（idk-4） |
| 7 | minecraft:random_boolean_selector | RandomBooleanFeature + Config | feature_true/false（placed Holder） | **全量**：nextBoolean（≡next(1)，以 next_int_bound(2)==1 表达，E-1 求证）选支 → 内嵌 placed 同流下钻 | 内层指向的 huge_brown/huge_red_mushroom 未实装 → 命中时成为**新 unknown**（§〇.3） |
| 8 | minecraft:simple_random_selector | **SimpleRandomFeature**（非 RandomFeature！）+ Config | features（placed 列表，无 chance） | **全量**：Util.getRandom = nextInt(n) 均匀选一（与 random_selector 的 nextFloat<chance 语义**完全不同**，E-1）→ 内嵌 placed 同流下钻 | 内层 pointed_dripstone / coral_tree/claw/mushroom 未实装 → 命中时成为**新 unknown**（§〇.3） |
| 9 | minecraft:vegetation_patch | VegetationPatchFeature + Config（10 字段） | 同左 | **全量**：xz_radius×2 → 边缘列 chance 短路序 → 列扫描（air 下扫 + 非 air 回扫）→ depth/extra_bottom → placeGround（每步 groundState.get）→ vegetation 逐位置 nextFloat + 内嵌 placed | **位置集 Java HashSet 迭代序 ≠ 引擎插入序**（RNG 流分叉点，idk-5，诚实声明）；isSideSolidFullSquare/isAir 近似；moss_replaceable tag 数据驱动（缺失=空=门失效，idk-3） |
| 10 | minecraft:waterlogged_vegetation_patch | WaterloggedVegetationPatchFeature（= #9 子类） | 同 #9 | **全量**（扫描时发现，见 E-2）：#9 复用 + 尾段水填充（五向全实心判定）+ vegetation 位 down(1) + WATERLOGGED 属性丢弃 | 同 #9 |
| 11 | minecraft:root_system | RootSystemFeature + Config（13 字段） | 同左 | **全量**：列上扫 predicate + hasSpaceForTree（air/water 垂直配额）→ lava/非实心底门 → 内嵌 placed（azalea_tree）→ roots 柱 + hanging roots | 需 **BlockPredicate 扩 AnyOf + MatchingBlockTag**（rooted_azalea_tree 实测引用，E-3）；canPlaceAt/sideSolid 近似；azalea_grows_on/replaceable_by_trees tag 数据驱动（idk-3） |
| 12 | minecraft:nether_forest_vegetation | NetherForestVegetationFeature + Config | state_provider/spread_width/spread_height | **全量**：nylium tag 门 + y 界门 + w² 轮（恒 6 次 nextInt + state.get 在 air 检查前）→ canPlaceAt 近似 | batch0 起从 ore 误捕转 catch-all 的类型，本批正规接入；canPlaceAt → is_solid_id 近似（idk-3） |
| 13 | modifier minecraft:fixed_placement | FixedPlacementModifier（1.21.6 新增） | positions（BlockPos 列表） | **全量**：按当前 chunk section（x>>4,z>>4）过滤输出，0 RNG | 当前引擎 1.20.1 数据集 **0 引用**（end_platform 为 1.21.6 placed，1.20.1 placed_feature 无此文件，glob 实证）——纯向前兼容，见 E-2 |

### §〇.2 缓装项（5/16，诚实声明逐项理由，非静默降级）

| type | 一手源体量 | 缓装理由（复杂度评估） | 引擎缺的前置设施 |
|------|-----------|------------------------|------------------|
| minecraft:dripstone_cluster | DripstoneClusterFeature.java **206 行** + DripstoneClusterFeatureConfig（12 字段） | 依赖 CaveSurface（floor/ceiling 配对面搜索工具类）+ DripstoneHelper（canGenerate/generatePointedDripstone——后者是 POINTED_DRIPSTONE 三属性 THICKNESS/FACING/WATERLOGGED 的属性级放置，引擎 state=i32 下 THICKNESS 面积语义完全丢失）+ **FloatProvider 家族（clamped_normal 引擎零实现）**。三件套均需新基础设施，估 ~500 行 Rust + 2 个新基建模块，风险集中且无法在本批编译验证 | CaveSurface / DripstoneHelper / FloatProvider(clamped_normal) |
| minecraft:large_dripstone | LargeDripstoneFeature.java **219 行**（DripstoneGenerator + WindModifier 两个内部类） | 同上 CaveSurface/DripstoneHelper/FloatProvider（uniform+bluntness/heightScale/windSpeed 三个 FloatProvider 字段）+ `MathHelper.nextBetween(f32)` + WORLD_SURFACE 高度图列循环；风场偏移 `wind.multiply(i).floor` 浮点逐列对拍面大。估 ~400 行 + 基建同上 | 同上 |
| minecraft:iceberg | IcebergFeature.java **266 行**（15 个私有方法，椭圆场旋转 + 雪顶 + 底柱） | 自包含无新基建，但**全部几何经 `randomSine`（nextDouble×2π 旋转矩阵）**——每个 distance 判定都是 f64 三角函数对拍面；`method_13419/13421/13427` 内嵌条件 nextInt/nextFloat（消费序对拍表 30+ 项）；f32/f64 混算（`Math.pow(rx,2.0)/(m*8.0F)` float→double 提升）一处形态错=整片错位。估 ~400 行，未编译环境下风险收益比最差，建议独立小批（可复用本批 SEA_LEVEL 常量） | 无（纯工作量大） |
| minecraft:sculk_patch | SculkPatchFeature.java 78 行，但核心在 **SculkSpreadManager（world-gen 传播模拟器）** | feature 本体薄，实际逻辑 = SculkSpreadManager.createWorldGen() 的 charge/cursor/queue 多轮 tick 传播（3+ 文件：SpreadManager/Cursor/SculkSpreadable 方块行为族，含 SCULK 多方块变体放置规则）；引擎无 sculk 行为层零起点。若只实装 feature 骨架不装 spread = 假实装（放置面全空），不做 | SculkSpreadManager 子系统 |
| minecraft:fossil | FossilFeature.java 83 行 | 依赖 **StructureTemplateManager（NBT 结构模板）** + StructurePlacementData（rotation/mirror/bounding box/processor）+ 两个 fossil .nbt 资产——引擎结构层整体跳过（feature_loader.rs L5 既有决策），本 feature 是结构子系统的一角，单点实装不可能 | 结构模板子系统（NBT 资产 + 放置管线） |

**建议**：dripstone_cluster + large_dripstone 合并为「dripstone 基建批」（先 CaveSurface/DripstoneHelper/FloatProvider 三基建，再两 feature）；iceberg 独立小批；sculk_patch/fossil 挂靠各自基建课题（sculk 行为层 / 结构模板层），不宜在 features 接管批内消化。

### §〇.3 分发接入后预期 unknown 集合变化

当前实跑 unknown **n=16**（1.21.6 数据 overworld region 0,0）。本批接入后：

- **消失（11）**：`minecraft:bamboo, block_column, blue_ice, desert_well, forest_rock, ice_spike, random_boolean_selector, root_system, simple_random_selector, vegetation_patch, vines`
- **保留（5，= §〇.2 缓装）**：`minecraft:dripstone_cluster, fossil, iceberg, large_dripstone, sculk_patch`
- **overworld region 0,0 预期 unknown = n=5**，且集合 == 上行保留集（无其他新增）。
- **附带清除**：`minecraft:nether_forest_vegetation`（nether 域，批次 0 起为 catch-all 残差）→ 接入后从 unknown 消失；`mod:minecraft:fixed_placement`（end 域）→ 接入后消失（当前引擎数据集本就 0 触发）。
- **⚠️ 预期可能「新增」的嵌套暴露 unknown（非回归，必须写进验收预期）**：selector 类（#7/#8/#9）接入后，此前因 catch-all false 而从未下钻的内层 configured 首次可获得执行，其 type 若未实装将以 unknown 面暴露：
  - `minecraft:pointed_dripstone`（simple_random_selector `pointed_dripstone.json` 两 entry）
  - `minecraft:coral_tree` / `minecraft:coral_claw` / `minecraft:coral_mushroom`（`warm_ocean_vegetation.json` 三 entry）
  - `minecraft:huge_brown_mushroom` / `minecraft:huge_red_mushroom`（random_boolean_selector `mushroom_island_vegetation.json` 两 entry）
  - 这些是否进入实测 unknown 取决于 region 0,0 是否有对应 biome/selector 命中（pointed_dripstone 的 entry 経 environment_scan 链，dripstone biome 面大，**出现概率不低**）。验收比对时请区分「残差保留集 n=5」与「嵌套暴露集」。
- 验收方式：`WG_FEATURE_UNKNOWN_LOG=1` 实跑比对（同 batchA/B）。

---

## 一、逐项对拍（一手源 file:line，1.20.1 版；两版一致性除 §一.11 外均 Compare-Object 逐行实证）

### §一.1 minecraft:bamboo（BambooFeature.java，77 行；bamboo_no_podzol.json prob=0.0 / bamboo_some_podzol.json prob=0.2）

config 修正：ProbabilityConfig 仅 `probability` 单字段（world/gen/ProbabilityConfig.java）——**无 provider/spread**。

| Java | 语义 | Rust |
|---|---|---|
| :40 `isAir(origin)` 门 | 非 air → **0 消费 false**；air → 恒 i++（:72）→ **true（即使 canPlaceAt 失败）** | `block_at != air → false`，canPlace 失败也 `placed=true` |
| :41 `canPlaceAt`（BambooBlock 近似）| below ∈ {BAMBOO, BAMBOO_SAPLING} ∨ isSoil(below)（dirt tag ∨ farmland）| 同，失败 0 消费 |
| :42 `j = nextInt(12)+5` | 竹秆目标高度 | 同 |
| :43 `nextFloat() < probability` | podzol 圈门 | `(next_float() as f64) < prob as f64` |
| :44 `k = nextInt(4)+1` | 圈半径 | 同 |
| :46-57 圈内 `n²+o²≤k²` → `getTopY(WORLD_SURFACE,l,m)-1` → isSoil → podzol | getTopY：chunk 内 world_surface 桶 +1；邻域 block_at 下扫首个非 air +1（idk-2）；不可读 None → 跳过该列 | `get_top_y_world_surface` 新 helper |
| :60 `for k<j && isAir(mutable)` | 循环条件**先**查 air | while 同构 |
| :61 放 BAMBOO（AGE=1/LEAVES=NONE/STAGE=0 属性丢弃） | 秆 | 放 bamboo 裸 id |
| :65-68 `cy-origin ≥ 3` → 顶 3 格 TOP_1/2/3（LEAVES/STAGE 属性丢弃） | 顶 | 3 格 bamboo 裸 id（cy, cy-1, cy-2） |

RNG 消费序：air 门 0 → canPlace 门 0 → nextInt(12) → nextFloat → [圈 nextInt(4)] → 循环/顶 0。

### §一.2 minecraft:block_column（BlockColumnFeature.java 73 行 + BlockColumnFeatureConfig.java 40 行）

config（codec :13-21 全 required）：`layers[{height: IntProvider, provider: BlockStateProvider}]` / `direction` / `allowed_placement: BlockPredicate` / `prioritize_tip`。
数据集形态（cave_vine/dripleaf/patch_cactus 实测）：direction ∈ {up, down}；height 可为 **weighted_list 嵌套 IntProvider**（→ P1 新变体 WeightedProviders）；provider 可为 randomized_int_state_provider（→ parse 取 source，age/values 丢弃）。

| Java | Rust |
|---|---|
| :20-27 每层 `height.get(random)` 先**全抽**（层序）累加 total | 同（层序 = JSON 序） |
| :29 `total==0 → false`（height 消费已发生） | 同 |
| :32-42 `mutable2 = origin+direction` 前探 total 格；`!allowedPlacement.test` → `adjustLayerHeights(is, total, l, prioritizeTip)`（:60-72 逐行镜像：tip 从 0 起 +1 步进 / 否则从尾起 -1）→ break | 谓词 test 走 `crate::placement::BlockPredicate::test(pctx,…)`（P2 扩 any_of/matching_block_tag）；**block_at 不可读(-1) → 谓词 false → 视为阻挡**（保守） |
| :44-54 分层放置：`m!=0` 层内每格 `layer.state().get(random)` → set → move(direction) | 同；属性丢弃消费保留 |
| :56 `return true` | 同 |

RNG 消费序：height.get ×n（先全抽）→ 谓词/adjust 0 → 放置层内 state.get（仅 m≠0 层、每格 1 次）。

### §一.3 minecraft:blue_ice（BlueIceFeature.java 68 行）

| Java | Rust |
|---|---|
| :23 `origin.y > seaLevel-1 → false` | `y > SEA_LEVEL-1`（SEA_LEVEL=63 硬编码，idk-4） |
| :25 本格水 ∨ 下方水门 | 同 |
| :30-35 Direction.values()（DOWN,UP,N,S,W,E）跳 DOWN，任一 packed_ice | 同序 |
| :40 放 blue_ice | 同 |
| :42-62 200 轮：`j=nextInt(5)-nextInt(6)`；`k=3; if j<2 {k+=j/2}`（i32 截断除法同 Java，j 负时 k 减）；`k>=1` 时 `nextInt(k)×4`（x,z 各 2）→ 目标 ∈ {air,water,packed_ice,ice} → 六向任一邻 blue_ice → 放 | 同（六向 values 序） |

RNG 消费序：门 0 → 每轮恒 2 次（j）；命中 k≥1 再 4 次。**无其他消费。**

### §一.4 minecraft:desert_well（DesertWellFeature.java 114 行）

| Java | Rust |
|---|---|
| :33 `up()` 后 :35 while air && y>bottom+2 下扫 | 同（min_y+2） |
| :39 `CAN_GENERATE`（BlockStatePredicate.forBlock(SAND)——任意 sand 状态）| `id == sand` |
| :42-47 5×5 双层（y-1,y-2）air → false | 同 |
| :50-56 i=-2..=0 三层 sandstone（`blockPos.add(jx, i, k)` 相对 y）| `py+i`，i∈[-2,0] |
| :58-69 中心水 + 4 水平邻水；down sand + 4 邻 sand | HORIZONTAL 序 N,E,S,W（无 RNG，序无关） |
| :71-77 边圈 y+1 sandstone；:79-82 四向 slab；:84-92 y+4 3×3（中心 sandstone 余 slab）；:94-99 四角柱 y+1..+3 | 同逐格 |
| :101-104 `Util.getRandom([origin,E,S,W,N])`（=nextInt(5)）.down(1) 与 .down(2) → suspicious_sand | 2 次 `next_int_bound(5)`；spots 序 [0,0],[1,0],[0,1],[-1,0],[0,-1]；**恒消费**（放置无条件） |

RNG 消费序：门全 0 → nextInt(5) ×2。

### §一.5 minecraft:forest_rock（ForestRockFeature.java 53 行 + SingleStateFeatureConfig）

config：`state` = 固定 BlockState（**非 provider**，0 随机消费；JSON `forest_rock.json` state=mossy_cobblestone）。

| Java | Rust |
|---|---|
| :23-30 for(y>bottom+3)：下方非 air 且 `isSoil∨isStone` → break；否则 y-1 | isSoil = dirt tag ∨ farmland；isStone = base_stone_overworld tag（Feature.java isStone） |
| :32 `y≤bottom+3 → false` | 同 |
| :35-48 3 轮：nextInt(2)×3（j,k,l）→ `f=((j+k+l)×0.333F+0.5F)`（f32）→ BlockPos.iterate（x 内/z 外）`squaredDistance ≤ f*f`（int² 和 f64 vs (f*f) f32→f64 提升）→ 放 state | 同；iterate 序对放置结果无影响（无 RNG）照抄 |
| :47 步进 `(-1+nextInt(2), -nextInt(2), -1+nextInt(2))` | 同 |

RNG 消费序：落底 0 → 每轮恒 6 次。

### §一.6 minecraft:ice_spike（IceSpikeFeature.java 101 行）

| Java | Rust |
|---|---|
| :23 落底同 forest_rock；:27 snow_block 门 | 同 |
| :30 `up(nextInt(4))`；:31 `i=nextInt(4)+7`；:32 `j=i/4+nextInt(2)`；:33 `j>1 && nextInt(60)==0 → up(10+nextInt(30))` | 同（:33 短路：j≤1 不抽 60） |
| :37-61 主循环：`f=(1.0-k/i)*j`（f32）；`l=ceil(f)`；g/h=abs-0.25（f32）；**:46 复合条件 `(m==0&&n==0 ∨ !(g²+h²>f²)) && (ring ∨ !(nextFloat()>0.75F))`——nextFloat 仅在「前半真且非 ring」时消费（`&&`/`||` 短路逐字保留）** | 同构布尔表达式 |
| :48-57 目标 ∈ {air, isSoil, snow_block, ice} → 放 packed_ice（本格 + k≠0∧l>1 时镜像 -k 格） | `this.setBlockState`（Feature.setBlockState 无条件写近似声明） |
| :63-68 `k=j-1` clamp [0,1] | 同 |
| :70-96 底柱：(o,l)∈[-k,k]²，角点（|o|==1∧|l|==1）`p=nextInt(5)` 否则 p=50；`while y>50`（**字面量 50**）：非 {air,isSoil,snow_block,ice,packed_ice} → break；放 packed_ice；`--p≤0 → down(nextInt(5)+1); p=nextInt(5)` | 同 |

RNG 消费序：落底 0 → 门 0 → nextInt(4) → nextInt(4) → nextInt(2) → [nextInt(60) → nextInt(30)] → 主循环逐格条件消费 → 底柱逐柱/逐步。

### §一.7 minecraft:random_boolean_selector（RandomBooleanFeature.java 27 行 + Config.java 28 行）

- config：`feature_true` / `feature_false`（PlacedFeature REGISTRY_CODEC——JSON 形态 `{"feature": <id 或内嵌>, "placement": [...]}`，PlacedFeature::parse_inline 直解）。
- :22 `random.nextBoolean()`：Java = `next(1) != 0`；`nextInt(2)` 走幂二 fast path = `next(1)`——**同 1 bit 消费**。Rust 以 `random.next_int_bound(2) == 1` 表达（≈ 对拍声明 E-1；引擎 next_int_bound 移植自 Java Random.nextInt 同 fast path）。
- :23-25 选中支 `.value().generateUnregistered(...)` = 内嵌 placed 同 RNG 流下钻 → 引擎 = `pf.generate(ctx, random, …, |…| generate_nested(...))`（feature_loader random_selector 同款闭包）。

RNG 消费序：1 bit → 全部消费在内层 placed 链。

### §一.8 minecraft:simple_random_selector（**SimpleRandomFeature.java 26 行**——与 random_selector（RandomFeature）不同类！+ SimpleRandomFeatureConfig）

**E-1 核心**：任务简报把本类与 random_selector 并称「selector 类注意 RNG 差异」——一手源实锤差异比预想更大：

| | random_selector（RandomFeature） | simple_random_selector（SimpleRandomFeature） |
|---|---|---|
| config | features[{chance, feature}] + **default** | features（placed 列表，**无 chance 字段**） |
| 选择 | 逐项 `nextFloat() < chance` 即选即返（落空走 default） | **`nextInt(size)` 均匀选一，恒 1 次消费** |
| 全落空 | default（不抽选择 RNG） | 不存在 default（parse 保证 features 非空） |

Rust（SimpleRandomFeature.java:22-24 逐行）：`idx = next_int_bound(n)` → `features[idx].generate(ctx, random, …, generate_nested 闭包)`。

数据集：forest_flowers（4×random_patch/flower 内嵌）、dripleaf（simple_block + block_column×4）、pointed_dripstone（pointed_dripstone×2，**未实装 → 嵌套暴露 unknown**）、warm_ocean_vegetation（coral×3，**未实装 → 嵌套暴露 unknown**）。

### §一.9 minecraft:vegetation_patch（VegetationPatchFeature.java 121 行 + Config.java 63 行 10 字段）

codec（:14-28 全 required）：`replaceable`(tag) / `ground_state` / `vegetation_feature`(placed Holder) / `surface`(floor|ceiling) / `depth`(IntProvider 1..128) / `extra_bottom_block_chance`(f32) / `vertical_range`(1..256) / `vegetation_chance`(f32) / `xz_radius`(IntProvider) / `extra_edge_column_chance`(f32)。数据集 moss_patch*：surface ∈ {floor, ceiling}。

| Java | Rust |
|---|---|
| :29-30 `i=xz_radius.get+1`（先）、`j=同`（后） | 同序 |
| :45-53 三重布尔 bl/bl2/bl3/bl4/bl5；`!bl4 && (!bl5 ∨ (chance!=0 && !(nextFloat()>chance)))`——**角列(bl4)全跳；非边列(bl5=false)不消费；边列且 chance≠0 恒消费** | 同构（短路序照抄） |
| :56-58 从 origin 沿 direction（floor=DOWN/ceiling=UP）扫 air（≤vertical_range）；:60-62 反向扫非 air | block_at -1（不可读）按非 air 走回扫（保守下移，声明） |
| :64-66 `mutable2 = mutable+direction`；`isAir(mutable) && sideSolidFullSquare(mutable2)` → is_solid_id 近似 | 同 |
| :67 `depth.get(random) + (extra_bottom>0 && nextFloat()<chance ? 1 : 0)`——**extra_bottom>0 时 nextFloat 恒消费**（数据集 0.0 → 不消费，代码保留恒消费位） | 同 |
| :69/:103-120 placeGround：每 depth 步 `groundState.get(random)` **恒消费** → 与现块同 block 则跳（不步进）→ 否则 !replaceable → `return i!=0`；可替换 → 放 + 沿 direction 步进 | 同（id 级等值比较） |
| :81-95 vegetation：逐位置 `nextFloat()<vegetation_chance` → 内嵌 placed 于 `pos.offset(direction2)`（floor→UP） | 同；⚠️ 位置集迭代序 idk-5 |
| :33 `return !set.isEmpty()` | `!positions.is_empty()` |

### §一.10 minecraft:waterlogged_vegetation_patch（WaterloggedVegetationPatchFeature.java 64 行，#9 子类——扫描时发现并补入覆盖面，E-2）

与 #9 唯二差异（其余 RNG 流完全一致）：
1. placeGroundAndGetPositions 尾段（:24-33）：对接受集逐位判「五向（N,E,S,W,DOWN）是否**全** side-solid」（:40-52，`isSolidBlockSide = !isSideSolidFullSquare` 取反语义逐字核对）→ 全实心的位改放 WATER，位置集替换为该子集（vegetation 只在子集上滚）。
2. generateVegetationFeature（:55-64）：内层 placed 于 `pos.down()`（即表面位本身，不再 offset(direction2)）；成功后 WATERLOGGED 属性置 true（**属性丢弃声明**，返回值不受影响）。

Rust：复用 `VegetationPatchFeature::generate` 加 `waterlogged: bool` 形参（水填充内联 + vegetation 位 `vy` 替代 `vy-dir_y`）。数据集 clay_pool_with_dripleaves（lush_caves_clay random_boolean_selector false 支）。

### §一.11 minecraft:root_system（RootSystemFeature.java 122 行 + Config.java 74 行 13 字段）

config 13 字段（:13-29 全 required）逐字段对拍无误；`allowed_tree_position` = BlockPredicate（rooted_azalea_tree.json 实测含 **any_of + matching_block_tag** → P2 扩两变体）。

| Java | Rust |
|---|---|
| :24 origin 非 air → **false**（不走 :36 true——与树未命中路径区分） | 同 |
| :65-78 列上扫 maxRootColumnHeight 轮：predicate.test(pctx) ∧ hasSpaceForTree → :68-71 below 是 LAVA ∨ !isSolid → **false**（中断树生成，不 continues）→ :73 内嵌 placed（同流）命中 → :74 roots 柱 [origin.y, origin.y+i) | hasSpace（:39-51/:53-60）：i∈1..=required_space：air → ok；否则 `(i+1) ≤ allowed_vertical_water ∧ water` |
| :31-33/:108-121 hanging roots：**仅树生成成功后**；attempts 轮 nextInt(radius)×2 + nextInt(span)×2 → isAir → state.get（**isAir 后、canPlaceAt 前消费**）→ canPlaceAt ∧ 上方侧满方（均 is_solid_id 近似）→ 放 | 同 |
| :35-36 恒 return true（树失败也 true，仅跳 hanging） | 同 |

RNG 消费序：origin 门 0 → 列扫 0 → 树=内层 placed 全流 → roots 每 attempt 恒 4 次 nextInt(r)（命中 root_replaceable 才 +1 次 state.get）→ hanging 每 attempt 恒 4 次 nextInt（isAir 后 +1 次 state.get）。

### §一.12 minecraft:nether_forest_vegetation（NetherForestVegetationFeature.java 52 行 + Config.java 25 行）

- :24 below ∈ nylium tag（数据驱动 `tags/blocks/nylium.json` 实存；缺失=空 → 恒 false，idk-3）。
- :28 y 界：`y ≥ bottom+1 && y+1 < top`（1.20.1 `getTopY()`；1.21.6 `getTopYInclusive()` 同义——**两版唯一 diff**，E-4）。引擎 = `min_y+height`。
- :31-44 `spread_width²` 轮，每轮恒 6 次 nextInt（w,h,w 序）+ `state_provider.get`（:37，**在 :38 isAir 检查之前消费**）；air ∧ y>bottom ∧ canPlaceAt（NetherPlantBlock → is_solid_id 近似）→ 放。
- :46 `return j>0`。
- 防御：w/h ≤0 → false（Codecs.POSITIVE_INT 保证 ≥1，防御不 panic）。

RNG 消费序：两门 0 → 每轮 6+1 次。

### §一.13 modifier minecraft:fixed_placement（FixedPlacementModifier.java 51 行，1.21.6 新增）

- CODEC（:13-16）：`positions` = BlockPos 列表。1.21.6 数据 `end_platform.json` 实测形态 `[[100,49,0]]`（**三元组数组**，非对象）——parse 双形态兼容（数组/对象），以三元组为主。
- :28-41：`ChunkSectionPos.getSectionCoord(x/z)`（= `x>>4`，含负数算术右移语义 Rust 同）过滤出**落在当前 chunk section 的坐标**输出；无命中 → 空。**0 RNG**。
- 引擎数据集现状：1.20.1 placed_feature **无 end_platform、0 处 fixed_placement**（glob+grep 实证）——本 patch 为 1.21.6 向前兼容，当前运行**不可达**（E-2）。

---

## 二、Patch 清单（old 串均先读当前工作区现文核对，file:line 以当前 HEAD 为准）

应用顺序：P1→P2→P3（placement.rs 三处无相互依赖）→ P4（tree.rs）→ P5（feature.rs）→ P6→P7→P8→P9（feature_loader.rs；P6 先于 P7/P8；P9 独立）。

### Patch P1 — placement.rs：IntProvider 新变体 WeightedProviders（block_column 层高嵌套 provider）

**P1a** 枚举变体（L20-21 一带）：

old:
```rust
    Clamped(Box<IntProvider>, i32, i32), // source, min, max
}
```
new:
```rust
    Clamped(Box<IntProvider>, i32, i32), // source, min, max
    /// batchC（mc-1216）：WeightedListIntProvider——distribution[].data 为嵌套 IntProvider
    /// （block_column layer height：dripleaf.json / cave_vine.json 实测；E-3）。
    WeightedProviders(Vec<(IntProvider, i32)>, i32), // (provider, weight), totalWeight
}
```

**P1b** get 分支（Clamped arm 之后）：

old:
```rust
            IntProvider::Clamped(source, min, max) => {
                let v = source.get(r);
                if v < *min { *min } else if v > *max { *max } else { v }
            }
        }
    }
```
new:
```rust
            IntProvider::Clamped(source, min, max) => {
                let v = source.get(r);
                if v < *min { *min } else if v > *max { *max } else { v }
            }
            IntProvider::WeightedProviders(entries, total_weight) => {
                if entries.is_empty() { return 0; }
                let mut i = r.next_int_bound(*total_weight);
                for (ip, w) in entries {
                    i -= w;
                    if i < 0 { return ip.get(r); }
                }
                entries[0].0.get(r)
            }
        }
    }
```

**P1c** parse：weighted_list 分支整体替换（嵌套形态优先判定）：

old:
```rust
        } else if type_name.contains("weighted_list") {
            // {"type":"minecraft:weighted_list","distribution":[{"data":6,"weight":9},...]}
            let mut weighted = Vec::new();
            let mut total = 0;
            if let Some(dist) = v.get("distribution") {
                if let Some(arr) = dist.as_array() {
                    for e in arr {
                        let data = e.get("data").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                        let w = e.get("weight").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                        weighted.push((data, w));
                        total += w;
                    }
                }
            }
            IntProvider::WeightedList(weighted, total)
        } else if type_name.contains("clamped") {
```
new:
```rust
        } else if type_name.contains("weighted_list") {
            // {"type":"minecraft:weighted_list","distribution":[{"data":6,"weight":9},...]}
            // batchC（mc-1216）：distribution[].data 为对象时 = 嵌套 IntProvider（WeightedListIntProvider，
            // block_column layer height：dripleaf/cave_vine 实测，E-3）——数值形态走旧 WeightedList。
            let nested = v.get("distribution").and_then(|d| d.as_array())
                .map(|arr| arr.iter().any(|e| e.get("data").map_or(false, |d| d.as_object().is_some())))
                .unwrap_or(false);
            if nested {
                let mut entries: Vec<(IntProvider, i32)> = Vec::new();
                let mut total = 0;
                if let Some(arr) = v.get("distribution").and_then(|d| d.as_array()) {
                    for e in arr {
                        let w = e.get("weight").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                        entries.push((IntProvider::parse(e.get("data")), w));
                        total += w;
                    }
                }
                return IntProvider::WeightedProviders(entries, total);
            }
            let mut weighted = Vec::new();
            let mut total = 0;
            if let Some(dist) = v.get("distribution") {
                if let Some(arr) = dist.as_array() {
                    for e in arr {
                        let data = e.get("data").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                        let w = e.get("weight").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                        weighted.push((data, w));
                        total += w;
                    }
                }
            }
            IntProvider::WeightedList(weighted, total)
        } else if type_name.contains("clamped") {
```

**P1d** max_value（L127-128 一带）：

old:
```rust
            IntProvider::Clamped(_, _, max) => *max,
        }
    }
}
```
new:
```rust
            IntProvider::Clamped(_, _, max) => *max,
            // batchC：嵌套 provider 取各上界最大（本批无 getMax 消费点，防御完备性）
            IntProvider::WeightedProviders(entries, _) => {
                entries.iter().map(|(ip, _)| ip.max_value()).max().unwrap_or(0)
            }
        }
    }
}
```

### Patch P2 — placement.rs：BlockPredicate 扩 AnyOf + MatchingBlockTag（root_system allowed_tree_position）

**P2a** 枚举变体：

old:
```rust
    Not(Box<BlockPredicate>),
    AllOf(Vec<BlockPredicate>),
    AlwaysTrue,
```
new:
```rust
    Not(Box<BlockPredicate>),
    AllOf(Vec<BlockPredicate>),
    /// batchC（mc-1216）：any_of（root_system allowed_tree_position 实测引用）
    AnyOf(Vec<BlockPredicate>),
    /// batchC：matching_block_tag（tag parse 期展开为 id 表；空表 = 门失效，idk-3）
    MatchingBlockTag { offset: [i32; 3], ids: Vec<i32> },
    AlwaysTrue,
```

**P2b** parse：matching_fluids 分支后插 matching_block_tag：

old:
```rust
        } else if type_name.contains("matching_fluids") {
            BlockPredicate::MatchingFluids { offset: offset(), ids: ids_of("fluids") }
        } else if type_name.contains("would_survive") {
```
new:
```rust
        } else if type_name.contains("matching_fluids") {
            BlockPredicate::MatchingFluids { offset: offset(), ids: ids_of("fluids") }
        } else if type_name.contains("matching_block_tag") {
            // batchC（mc-1216）：{"type":"minecraft:matching_block_tag","tag":"...","offset":[..]}
            //（rooted_azalea_tree.json azalea_grows_on / replaceable_by_trees 实测）
            let tag = v.get("tag").and_then(|t| t.as_str()).unwrap_or("");
            let tag = tag.strip_prefix('#').unwrap_or(tag);
            let mut ids = Vec::new();
            crate::feature::expand_tag(blocks, tag, &mut ids);
            if ids.is_empty() {
                eprintln!("[placement] matching_block_tag expanded to empty: {tag}（tag JSON 缺失？idk-3）");
            }
            BlockPredicate::MatchingBlockTag { offset: offset(), ids }
        } else if type_name.contains("would_survive") {
```

**P2c** parse：all_of 分支后插 any_of：

old:
```rust
            BlockPredicate::AllOf(preds)
        } else if type_name.contains("not") {
```
new:
```rust
            BlockPredicate::AllOf(preds)
        } else if type_name.contains("any_of") {
            let mut preds = Vec::new();
            if let Some(arr) = v.get("predicates").and_then(|p| p.as_array()) {
                for p in arr { preds.push(BlockPredicate::parse(Some(p), blocks)); }
            }
            BlockPredicate::AnyOf(preds)
        } else if type_name.contains("not") {
```

**P2d** test 分支：

old:
```rust
            BlockPredicate::Not(inner) => !inner.test(ctx, x, y, z),
            BlockPredicate::AllOf(preds) => preds.iter().all(|p| p.test(ctx, x, y, z)),
        }
    }
}
```
new:
```rust
            BlockPredicate::Not(inner) => !inner.test(ctx, x, y, z),
            BlockPredicate::AllOf(preds) => preds.iter().all(|p| p.test(ctx, x, y, z)),
            BlockPredicate::AnyOf(preds) => preds.iter().any(|p| p.test(ctx, x, y, z)),
            BlockPredicate::MatchingBlockTag { offset, ids } => {
                let cur = block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && ids.contains(&cur)
            }
        }
    }
}
```

### Patch P3 — placement.rs：fixed_placement modifier

**P3a** 枚举变体（NoiseThresholdCount 之后）：

old:
```rust
    NoiseThresholdCount { noise_level: f64, below_noise: i32, above_noise: i32 },
}
```
new:
```rust
    NoiseThresholdCount { noise_level: f64, below_noise: i32, above_noise: i32 },
    /// batchC（mc-1216）：fixed_placement（FixedPlacementModifier.java:12-51，1.21.6 新增）。
    /// positions 全表按当前 chunk section（x>>4,z>>4）过滤输出；0 RNG。
    /// 当前 1.20.1 数据集 0 引用（end_platform 为 1.21.6 placed）——纯向前兼容（E-2）。
    Fixed { positions: Vec<[i32; 3]> },
}
```

**P3b** get_positions 分支（NoiseThresholdCount arm 之后、match 收尾前）：

old:
```rust
                let d = 0.0f64;
                let n = if d < *noise_level { *below_noise } else { *above_noise };
                (0..n).map(|_| [x, y, z]).collect()
            }
        }
    }
```
new:
```rust
                let d = 0.0f64;
                let n = if d < *noise_level { *below_noise } else { *above_noise };
                (0..n).map(|_| [x, y, z]).collect()
            }
            PlacementModifier::Fixed { positions } => {
                // FixedPlacementModifier.java:28-41：getSectionCoord = x>>4；0 随机消费
                let (sx, sz) = (x >> 4, z >> 4);
                positions.iter().copied()
                    .filter(|p| p[0] >> 4 == sx && p[2] >> 4 == sz)
                    .collect()
            }
        }
    }
```

**P3c** parse 分支（noise_threshold_count 之后、rarity_filter 之前插入）：

old:
```rust
        } else if type_name.contains("rarity_filter") {
```
new:
```rust
        } else if type_name == "minecraft:fixed_placement" {
            // FixedPlacementModifier CODEC：positions = BlockPos 列表（1.21.6 end_platform.json 实测
            // 形态 = 三元组数组 [[x,y,z],...]；对象形态防御兼容）。batchC
            let mut positions = Vec::new();
            if let Some(arr) = m.get("positions").and_then(|p| p.as_array()) {
                for p in arr {
                    let pos = if let Some(t) = p.as_array() {
                        let g = |i: usize| t.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
                        [g(0), g(1), g(2)]
                    } else {
                        [
                            p.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                            p.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                            p.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                        ]
                    };
                    positions.push(pos);
                }
            }
            return Some(PlacementModifier::Fixed { positions });
        } else if type_name.contains("rarity_filter") {
```

### Patch P5 — feature.rs：batchC 全部 feature 实装（追加到文件尾，MultifaceGrowthFeature 闭合后）

> 分两段给出（第一段：bamboo/block_column/blue_ice/desert_well/forest_rock/ice_spike/vines/nether_forest_vegetation；第二段：vegetation_patch(+waterlogged)/root_system）。两段连续追加。

**P5 第一段：**

```rust

// ===== batchC（mc-1216）：unknown 残差清空 11/16 + nether_forest_vegetation =====
// 一手源：versions/1.20.1 + versions/1.21.6 双版 mc_src_extract（实装对象两版逐行一致，
// 唯一 diff = NetherForestVegetationFeature getTopY→getTopYInclusive 同义改名，batchC §一.11/E-4）。
// 对拍表 + RNG 消费序声明：.investigations/mc-1216-features-takeover/batchC-worker-delivery.md §一
// 已知限制（本族）：state=i32 无属性位（BAMBOO.LEAVES/STAGE/AGE、VINE faces、WATERLOGGED 等丢弃，
// 对应 RNG 消费全部保留）；isSolid/isSoil/isStone/isSideSolidFullSquare → tag 展开 + is_solid_id 近似；
// 方块实体/流体 tick no-op（同 batchA/B 全族声明）。缓装 5 项（dripstone_cluster/large_dripstone/
// iceberg/sculk_patch/fossil）逐项理由见交付文档 §〇.2。

/// Java getSeaLevel（WorldAccess 默认 63）。本批仅 overworld 数据集引用（blue_ice；iceberg 预留），
/// 硬编码声明（idk-4）；多维度参数化课题落地时改注入。
const SEA_LEVEL: i32 = 63;

/// BlockTags.DIRT 展开（数据驱动 block_tags JSON；缺失 fallback 硬编码表 expand_tag_fallback）。
fn dirt_tag_ids(blocks: &BlockRegistry) -> Vec<i32> {
    let mut ids = Vec::new();
    expand_tag(blocks, "minecraft:dirt", &mut ids);
    ids
}

/// Feature.isSoil（Feature.java）：isIn(BlockTags.DIRT) ∨ isOf(FARMLAND)。
fn is_soil_id(ctx: &OreFeatureContext, id: i32, dirt_ids: &[i32], farmland: i32) -> bool {
    let _ = ctx;
    id == farmland || dirt_ids.contains(&id)
}

/// getTopY(Heightmap.Type.WORLD_SURFACE, x, z)（bamboo podzol 圈用）：
/// chunk 内 = world_surface 桶 +1（引擎口径：数组存最高实体块 y）；邻域 = block_at 下扫首个非 air +1；
/// 不可读 → None（调用点跳过该列，idk-2——下扫含已生成 feature 实况，时序差声明同 batchB idk-2）。
fn get_top_y_world_surface(ctx: &OreFeatureContext, wx: i32, wz: i32) -> Option<i32> {
    let air = crate::blocks::AIR;
    let lx = wx - ctx.chunk_start_x;
    let lz = wz - ctx.chunk_start_z;
    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 {
        let hm = ctx.world_surface?;
        return Some(hm[(lz * 16 + lx) as usize] + 1);
    }
    let mut y = ctx.min_y + ctx.height - 1;
    while y >= ctx.min_y {
        let b = ctx.block_at(wx, y, wz);
        if b < 0 { return None; }
        if b != air { return Some(y + 1); }
        y -= 1;
    }
    None
}

// ===== minecraft:bamboo（BambooFeature.java + ProbabilityConfig.probability）=====
pub struct BambooFeature;
impl BambooFeature {
    /// RNG 消费序（§一.1）：origin 非 air 0 消费 false；air 时 canPlace 失败 0 消费但**返回 true**（:72 i++）
    /// → nextInt(12)+5 → nextFloat<probability → [圈 nextInt(4)] → 循环/顶 0 消费。
    pub fn generate(&self, ctx: &mut OreFeatureContext, probability: f32,
                    random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let bamboo = blocks.id("minecraft:bamboo");
        let bamboo_sapling = blocks.id("minecraft:bamboo_sapling");
        let farmland = blocks.id("minecraft:farmland");
        let podzol = blocks.id("minecraft:podzol");
        let dirt = dirt_tag_ids(blocks);
        if ctx.block_at(x, y, z) != air { return false; }                                  // :40
        // :41 canPlaceAt（BambooBlock 近似）：below ∈ {bamboo, bamboo_sapling} ∨ isSoil(below)
        let below = ctx.block_at(x, y - 1, z);
        let can_place = below == bamboo || below == bamboo_sapling
            || is_soil_id(ctx, below, &dirt, farmland);
        if can_place {                                                                      // :41
            let j = random.next_int_bound(12) + 5;                                          // :42
            if (random.next_float() as f64) < probability as f64 {                          // :43
                let k = random.next_int_bound(4) + 1;                                       // :44
                for l in (x - k)..=(x + k) {                                                // :46
                    for m in (z - k)..=(z + k) {                                            // :47
                        let n = l - x;
                        let o = m - z;
                        if n * n + o * o <= k * k {                                         // :50
                            // :51 getTopY(WORLD_SURFACE)-1；邻域不可读 None → 跳过（idk-2）
                            if let Some(top) = get_top_y_world_surface(ctx, l, m) {
                                let sy = top - 1;
                                if is_soil_id(ctx, ctx.block_at(l, sy, m), &dirt, farmland) {
                                    ctx.set_block(l, sy, m, podzol);                        // :53
                                }
                            }
                        }
                    }
                }
            }
            let mut cy = y;
            let mut kk = 0;
            while kk < j && ctx.block_at(x, cy, z) == air {                                  // :60（先查 air）
                ctx.set_block(x, cy, z, bamboo);                                            // :61（属性丢弃）
                cy += 1;
                kk += 1;
            }
            if cy - y >= 3 {                                                                 // :65
                ctx.set_block(x, cy, z, bamboo);                                            // :66 TOP_1（属性丢弃）
                ctx.set_block(x, cy - 1, z, bamboo);                                        // :67 TOP_2
                ctx.set_block(x, cy - 2, z, bamboo);                                        // :68 TOP_3
            }
        }
        true                                                                                // :72/:75（air 即 true，canPlace 失败也是）
    }
}

// ===== minecraft:block_column（BlockColumnFeature.java + Config）=====
#[derive(Clone)]
pub struct BlockColumnLayer {
    pub height: crate::placement::IntProvider,
    pub state: crate::tree::BlockStateProvider,
}
#[derive(Clone)]
pub struct BlockColumnConfig {
    pub layers: Vec<BlockColumnLayer>,
    pub direction: [i32; 3],
    pub allowed_placement: crate::placement::BlockPredicate,
    pub prioritize_tip: bool,
}
/// layer provider 扩展解析：simple/weighted 之外补 randomized_int_state_provider
///（cave_vine 顶端 age provider——property/values 丢弃，取 source；声明）。
fn parse_state_provider_ext(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<crate::tree::BlockStateProvider> {
    if let Some(p) = crate::tree::BlockStateProvider::parse(v, blocks) { return Some(p); }
    let v = v?;
    let t = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if t.contains("randomized_int_state_provider") {
        return crate::tree::BlockStateProvider::parse(v.get("source"), blocks);
    }
    None
}
impl BlockColumnConfig {
    /// BlockColumnFeatureConfig.java:13-21（全 required）。direction 数据集 ∈ {up, down}，六向防御兼容。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<BlockColumnConfig> {
        let cfg = cfg?;
        let dir = match cfg.get("direction").and_then(|d| d.as_str()).unwrap_or("up") {
            "down" => [0, -1, 0],
            "north" => [0, 0, -1],
            "south" => [0, 0, 1],
            "west" => [-1, 0, 0],
            "east" => [1, 0, 0],
            _ => [0, 1, 0], // "up" 及缺省
        };
        let mut layers = Vec::new();
        if let Some(arr) = cfg.get("layers").and_then(|l| l.as_array()) {
            for l in arr {
                layers.push(BlockColumnLayer {
                    height: crate::placement::IntProvider::parse(l.get("height")),
                    state: parse_state_provider_ext(l.get("provider"), blocks)?,
                });
            }
        }
        if layers.is_empty() { return None; }
        Some(BlockColumnConfig {
            layers,
            direction: dir,
            allowed_placement: crate::placement::BlockPredicate::parse(cfg.get("allowed_placement"), blocks),
            prioritize_tip: cfg.get("prioritize_tip").and_then(|b| b.as_bool()).unwrap_or(false),
        })
    }
}
/// adjustLayerHeights（BlockColumnFeature.java:60-72 逐行镜像）。
fn adjust_layer_heights(heights: &mut [i32], expected: i32, actual: i32, prioritize_tip: bool) {
    let mut i = expected - actual;
    let j = if prioritize_tip { 1 } else { -1 };
    let mut m = if prioritize_tip { 0i32 } else { heights.len() as i32 - 1 };
    let end = if prioritize_tip { heights.len() as i32 } else { -1 };
    while m != end && i > 0 {
        let n = heights[m as usize];
        let o = if n < i { n } else { i };
        i -= o;
        heights[m as usize] -= o;
        m += j;
    }
}
pub struct BlockColumnFeature;
impl BlockColumnFeature {
    /// RNG 消费序（§一.2）：每层 height.get（先全抽）→ 谓词/adjust 0 → 放置层内 state.get。
    /// 谓词经 FeaturePlacementContext（pctx.block_at；-1 不可读 → 谓词 false = 阻挡，保守）。
    pub fn generate(&self, pctx: &crate::placement::FeaturePlacementContext,
                    ctx: &mut OreFeatureContext, config: &BlockColumnConfig,
                    random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let n = config.layers.len() as i32;
        let mut heights: Vec<i32> = Vec::with_capacity(config.layers.len());
        let mut total = 0i32;
        for l in &config.layers {
            let h = l.height.get(random);                                                   // :25
            heights.push(h);
            total += h;
        }
        if total == 0 { return false; }                                                     // :29
        let (dx, dy, dz) = (config.direction[0], config.direction[1], config.direction[2]);
        // :32-42 前探（mutable2 = origin+direction 起步）
        let mut probe = (x + dx, y + dy, z + dz);
        for l in 0..total {
            if !config.allowed_placement.test(pctx, probe.0, probe.1, probe.2) {            // :36
                adjust_layer_heights(&mut heights, total, l, config.prioritize_tip);         // :37
                break;
            }
            probe = (probe.0 + dx, probe.1 + dy, probe.2 + dz);                              // :41
        }
        // :44-54 分层放置
        let (mut px, mut py, mut pz) = (x, y, z);
        for l in 0..n {
            let m = heights[l as usize];
            if m != 0 {
                for _ in 0..m {
                    let st = config.layers[l as usize].state.get(random);                    // :50（属性丢弃消费保留）
                    ctx.set_block(px, py, pz, st);
                    px += dx; py += dy; pz += dz;                                            // :51
                }
            }
        }
        true                                                                                 // :56
    }
}

// ===== minecraft:blue_ice（BlueIceFeature.java，DefaultFeatureConfig 空 config）=====
pub struct BlueIceFeature;
impl BlueIceFeature {
    /// RNG 消费序（§一.3）：门 0 → 每轮恒 nextInt(5)-nextInt(6)，k>=1 命中再 nextInt(k)×4。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let packed_ice = blocks.id("minecraft:packed_ice");
        let blue_ice = blocks.id("minecraft:blue_ice");
        let ice = blocks.id("minecraft:ice");
        let air = blocks.id("minecraft:air");
        if y > SEA_LEVEL - 1 { return false; }                                               // :23（getSeaLevel≈63，idk-4）
        if ctx.block_at(x, y, z) != water && ctx.block_at(x, y - 1, z) != water { return false; } // :25
        const D6: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]]; // Direction.values()
        let mut found = false;
        for d in D6 {                                                                        // :30
            if d[1] == -1 { continue; }                                                      // != DOWN
            if ctx.block_at(x + d[0], y + d[1], z + d[2]) == packed_ice { found = true; break; }
        }
        if !found { return false; }                                                          // :37
        ctx.set_block(x, y, z, blue_ice);                                                    // :40
        for _ in 0..200 {                                                                     // :42
            let j = random.next_int_bound(5) - random.next_int_bound(6);                      // :43
            let mut k = 3i32;
            if j < 2 { k += j / 2; }                                                          // :45-47（i32 截断除法同 Java）
            if k >= 1 {                                                                       // :49
                let px = x + random.next_int_bound(k) - random.next_int_bound(k);             // :50
                let py = y + j;
                let pz = z + random.next_int_bound(k) - random.next_int_bound(k);
                let st = ctx.block_at(px, py, pz);
                if st == air || st == water || st == packed_ice || st == ice {                 // :52
                    for d in D6 {                                                              // :53
                        if ctx.block_at(px + d[0], py + d[1], pz + d[2]) == blue_ice {          // :55
                            ctx.set_block(px, py, pz, blue_ice);                                // :56
                            break;
                        }
                    }
                }
            }
        }
        true                                                                                 // :64
    }
}

// ===== minecraft:desert_well（DesertWellFeature.java，DefaultFeatureConfig）=====
pub struct DesertWellFeature;
impl DesertWellFeature {
    /// RNG 消费序（§一.4）：门全 0 → nextInt(5)×2（suspicious_sand 两次 Util.getRandom，恒消费；
    /// generateSuspiciousSand 本体无 RNG——setLootTable 的 loot seed 抽取发生在方块实体交互期，非生成期）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let sand = blocks.id("minecraft:sand");
        let sandstone = blocks.id("minecraft:sandstone");
        let slab = blocks.id("minecraft:sandstone_slab");
        let water = blocks.id("minecraft:water");
        let sus_sand = blocks.id("minecraft:suspicious_sand");
        let mut py = y + 1;                                                                    // :33 up()
        while ctx.block_at(x, py, z) == air && py > ctx.min_y + 2 { py -= 1; }                 // :35
        if ctx.block_at(x, py, z) != sand { return false; }                                    // :39
        for i in -2..=2 {                                                                      // :42-47
            for j in -2..=2 {
                if ctx.block_at(x + i, py - 1, z + j) == air
                    && ctx.block_at(x + i, py - 2, z + j) == air { return false; }
            }
        }
        for i in -2..=0 {                                                                      // :50-56
            for j in -2..=2 {
                for k in -2..=2 {
                    ctx.set_block(x + j, py + i, z + k, sandstone);
                }
            }
        }
        ctx.set_block(x, py, z, water);                                                        // :58
        const H4: [[i32; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];                           // HORIZONTAL N,E,S,W
        for d in H4 { ctx.set_block(x + d[0], py, z + d[1], water); }                           // :60-62
        ctx.set_block(x, py - 1, z, sand);                                                     // :64-65
        for d in H4 { ctx.set_block(x + d[0], py - 1, z + d[1], sand); }                         // :67-69
        for j in -2..=2 {                                                                      // :71-77
            for k in -2..=2 {
                if j == -2 || j == 2 || k == -2 || k == 2 {
                    ctx.set_block(x + j, py + 1, z + k, sandstone);
                }
            }
        }
        ctx.set_block(x + 2, py + 1, z, slab);                                                 // :79-82
        ctx.set_block(x - 2, py + 1, z, slab);
        ctx.set_block(x, py + 1, z + 2, slab);
        ctx.set_block(x, py + 1, z - 2, slab);
        for j in -1..=1 {                                                                      // :84-92
            for k in -1..=1 {
                if j == 0 && k == 0 { ctx.set_block(x + j, py + 4, z + k, sandstone); }
                else { ctx.set_block(x + j, py + 4, z + k, slab); }
            }
        }
        for j in 1..=3 {                                                                       // :94-99
            ctx.set_block(x - 1, py + j, z - 1, sandstone);
            ctx.set_block(x - 1, py + j, z + 1, sandstone);
            ctx.set_block(x + 1, py + j, z - 1, sandstone);
            ctx.set_block(x + 1, py + j, z + 1, sandstone);
        }
        // :101-104 list=[origin,E,S,W,N]；Util.getRandom = nextInt(5)，down(1)/down(2) 各抽一次
        let spots: [[i32; 2]; 5] = [[0, 0], [1, 0], [0, 1], [-1, 0], [0, -1]];
        for depth in [1, 2] {
            let pick = spots[random.next_int_bound(5) as usize];
            ctx.set_block(x + pick[0], py - depth, z + pick[1], sus_sand);                     // :110
        }
        true                                                                                   // :105
    }
}

// ===== minecraft:forest_rock（ForestRockFeature.java + SingleStateFeatureConfig）=====
#[derive(Clone)]
pub struct SingleStateConfig {
    pub state: i32,
}
impl SingleStateConfig {
    /// SingleStateFeatureConfig：state = 固定 BlockState（非 provider，0 随机消费）。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<SingleStateConfig> {
        Some(SingleStateConfig { state: parse_state_name(cfg, blocks)? })
    }
}
pub struct ForestRockFeature;
impl ForestRockFeature {
    /// RNG 消费序（§一.5）：落底 0 → 3 轮每轮恒 6 次。
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &SingleStateConfig,
                    random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let farmland = blocks.id("minecraft:farmland");
        let dirt = dirt_tag_ids(blocks);
        let mut stone_ids = Vec::new();
        expand_tag(blocks, "minecraft:base_stone_overworld", &mut stone_ids);                   // Feature.isStone
        let (mut px, mut py, mut pz) = (x, y, z);
        while py > ctx.min_y + 3 {                                                              // :23（for 条件）
            let below = ctx.block_at(px, py - 1, pz);
            if below != air && (is_soil_id(ctx, below, &dirt, farmland) || stone_ids.contains(&below)) {
                break;                                                                          // :26-27
            }
            py -= 1;
        }
        if py <= ctx.min_y + 3 { return false; }                                                // :32
        for _ in 0..3 {                                                                          // :35
            let j = random.next_int_bound(2);                                                    // :36
            let k = random.next_int_bound(2);                                                    // :37
            let l = random.next_int_bound(2);                                                    // :38
            let f = (j + k + l) as f32 * 0.333f32 + 0.5f32;                                      // :39（f32 全程）
            for dz in -l..=l {                                                                   // :41 iterate（z 外/x 内，无 RNG 序无关）
                for dy in -k..=k {
                    for dx in -j..=j {
                        let d2 = (dx * dx + dy * dy + dz * dz) as f64;                            // getSquaredDistance（int² 和）
                        if d2 <= (f * f) as f64 {                                                 // :42（f32 平方后提升 f64 比较）
                            ctx.set_block(px + dx, py + dy, pz + dz, config.state);               // :43
                        }
                    }
                }
            }
            px += -1 + random.next_int_bound(2);                                                  // :47
            py -= random.next_int_bound(2);
            pz += -1 + random.next_int_bound(2);
        }
        true                                                                                      // :50
    }
}

// ===== minecraft:ice_spike（IceSpikeFeature.java，DefaultFeatureConfig）=====
pub struct IceSpikeFeature;
impl IceSpikeFeature {
    /// RNG 消费序（§一.6）：落底 0 → 门 0 → nextInt(4) → nextInt(4) → nextInt(2) →
    /// [nextInt(60) → 命中 nextInt(30)] → 主菱形逐格（:46 复合条件短路消费）→ 底柱逐柱/逐步。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let snow_block = blocks.id("minecraft:snow_block");
        let ice = blocks.id("minecraft:ice");
        let packed_ice = blocks.id("minecraft:packed_ice");
        let farmland = blocks.id("minecraft:farmland");
        let dirt = dirt_tag_ids(blocks);
        let placeable = |ctx: &OreFeatureContext, id: i32| -> bool {
            id == air || is_soil_id(ctx, id, &dirt, farmland) || id == snow_block || id == ice      // :48
        };
        let (mut px, mut py, mut pz) = (x, y, z);
        while ctx.block_at(px, py, pz) == air && py > ctx.min_y + 2 { py -= 1; }                     // :23
        if ctx.block_at(px, py, pz) != snow_block { return false; }                                   // :27
        py += random.next_int_bound(4);                                                               // :30
        let i = random.next_int_bound(4) + 7;                                                         // :31
        let j = i / 4 + random.next_int_bound(2);                                                     // :32
        if j > 1 && random.next_int_bound(60) == 0 {                                                  // :33（短路：j≤1 不抽）
            py += 10 + random.next_int_bound(30);                                                     // :34
        }
        for k in 0..i {                                                                                // :37
            let f = (1.0f32 - k as f32 / i as f32) * j as f32;                                         // :38
            let l = f32::ceil(f) as i32;                                                               // :39 MathHelper.ceil
            for m in -l..=l {
                let g = m.abs() as f32 - 0.25f32;                                                      // :42
                for n in -l..=l {
                    let h = n.abs() as f32 - 0.25f32;                                                  // :45
                    // :46 复合条件逐字：nextFloat 仅「前半真 ∧ 非 ring」时消费（短路序）
                    let core = (m == 0 && n == 0) || !(g * g + h * h > f * f);
                    let ring = m != -l && m != l && n != -l && n != l;
                    if core && (ring || !(random.next_float() > 0.75f32)) {
                        let st = ctx.block_at(px + m, py + k, pz + n);
                        if placeable(ctx, st) {
                            ctx.set_block(px + m, py + k, pz + n, packed_ice);                         // :49
                        }
                        if k != 0 && l > 1 {                                                           // :52
                            let st2 = ctx.block_at(px + m, py - k, pz + n);
                            if placeable(ctx, st2) {
                                ctx.set_block(px + m, py - k, pz + n, packed_ice);                     // :55
                            }
                        }
                    }
                }
            }
        }
        let mut k = j - 1;                                                                             // :63
        if k < 0 { k = 0; } else if k > 1 { k = 1; }                                                   // :64-68
        for o in -k..=k {                                                                              // :70-96
            for l2 in -k..=k {
                let mut bx = px + o;
                let mut by = py - 1;
                let mut bz = pz + l2;
                let mut p = 50i32;
                if o.abs() == 1 && l2.abs() == 1 { p = random.next_int_bound(5); }                      // :74-75
                while by > 50 {                                                                         // :78（字面量 50 同 Java，idk-4）
                    let st = ctx.block_at(bx, by, bz);
                    if st != air && !is_soil_id(ctx, st, &dirt, farmland)
                        && st != snow_block && st != ice && st != packed_ice { break; }                  // :80-86
                    ctx.set_block(bx, by, bz, packed_ice);                                               // :88
                    by -= 1;
                    p -= 1;
                    if p <= 0 {                                                                          // :90-93
                        by -= random.next_int_bound(5) + 1;
                        p = random.next_int_bound(5);
                    }
                }
            }
        }
        true                                                                                            // :98
    }
}

// ===== minecraft:vines（VinesFeature.java，DefaultFeatureConfig）=====
pub struct VinesFeature;
impl VinesFeature {
    /// RNG 消费序（§一.8 表外）：**0 消费**（纯检查 + 至多放 1 格）。VINE faces 属性丢弃（裸 id）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, _random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let vine = blocks.id("minecraft:vine");
        if ctx.block_at(x, y, z) != air { return false; }                                               // :22
        // :25-30 Direction.values()（DOWN,UP,N,S,W,E）跳 DOWN；shouldConnectTo（VineBlock.java:128-130 =
        // MultifaceGrowthBlock.canGrowOn 侧满方）→ is_solid_id 近似（idk-3）
        const D6: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];
        for d in D6 {
            if d[1] == -1 { continue; }
            if crate::tree::is_solid_id(ctx, ctx.block_at(x + d[0], y + d[1], z + d[2])) {
                ctx.set_block(x, y, z, vine);                                                            // :27
                return true;
            }
        }
        false                                                                                            // :32
    }
}

// ===== minecraft:nether_forest_vegetation（NetherForestVegetationFeature.java + Config）=====
#[derive(Clone)]
pub struct NetherForestVegetationConfig {
    pub state_provider: crate::tree::BlockStateProvider,
    pub spread_width: i32,
    pub spread_height: i32,
}
impl NetherForestVegetationConfig {
    /// NetherForestVegetationFeatureConfig.java:9-16（extends BlockPileFeatureConfig）。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<NetherForestVegetationConfig> {
        let cfg = cfg?;
        Some(NetherForestVegetationConfig {
            state_provider: crate::tree::BlockStateProvider::parse(cfg.get("state_provider"), blocks)?,
            spread_width: cfg.get("spread_width").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            spread_height: cfg.get("spread_height").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
        })
    }
}
pub struct NetherForestVegetationFeature;
impl NetherForestVegetationFeature {
    /// RNG 消费序（§一.12）：两门 0 → 每轮恒 6 次 nextInt（w,h,w 序）+ state.get（air 检查前消费）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &NetherForestVegetationConfig,
                    random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let mut nylium = Vec::new();
        expand_tag(blocks, "minecraft:nylium", &mut nylium);                                             // BlockTags.NYLIUM（数据驱动）
        if !nylium.contains(&ctx.block_at(x, y - 1, z)) { return false; }                                 // :24
        if !(y >= ctx.min_y + 1 && y + 1 < ctx.min_y + ctx.height) { return false; }                      // :28（1.20.1 getTopY；1.21.6 同义改名 E-4）
        let (w, h) = (config.spread_width, config.spread_height);
        if w <= 0 || h <= 0 { return false; }                                                             // 防御（POSITIVE_INT 保证 ≥1）
        let mut placed = 0;
        for _ in 0..(w * w) {                                                                             // :31
            let px = x + random.next_int_bound(w) - random.next_int_bound(w);                             // :33
            let py = y + random.next_int_bound(h) - random.next_int_bound(h);                             // :34
            let pz = z + random.next_int_bound(w) - random.next_int_bound(w);                             // :35
            let st = config.state_provider.get(random);                                                    // :37（:38 air 检查前消费）
            if ctx.block_at(px, py, pz) == air && py > ctx.min_y                                           // :38-39
                && crate::tree::is_solid_id(ctx, ctx.block_at(px, py - 1, pz)) {                            // :40 canPlaceAt 近似（idk-3）
                ctx.set_block(px, py, pz, st);                                                                // :41
                placed += 1;
            }
        }
        placed > 0                                                                                         // :46
    }
}
```

**P5 第二段（vegetation_patch + waterlogged 变体 + root_system，紧接第一段继续追加）：**

```rust

// ===== minecraft:vegetation_patch（VegetationPatchFeature.java + Config 10 字段）=====
// + waterlogged_vegetation_patch（WaterloggedVegetationPatchFeature.java，子类，§一.10）
#[derive(Clone)]
pub struct VegetationPatchConfig {
    pub replaceable: Vec<i32>,                               // tag 展开缓存（parse 期）
    pub ground_state: crate::tree::BlockStateProvider,
    pub vegetation_feature: crate::placement::PlacedFeature,
    pub surface_floor: bool,                                 // floor=true（direction=DOWN）/ ceiling（UP）
    pub depth: crate::placement::IntProvider,
    pub extra_bottom_block_chance: f32,
    pub vertical_range: i32,
    pub vegetation_chance: f32,
    pub xz_radius: crate::placement::IntProvider,
    pub extra_edge_column_chance: f32,
}
impl VegetationPatchConfig {
    /// VegetationPatchFeatureConfig.java:14-28（全 required）。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<VegetationPatchConfig> {
        let cfg = cfg?;
        let mut replaceable = Vec::new();
        let tag = cfg.get("replaceable").and_then(|t| t.as_str()).unwrap_or("");
        expand_tag(blocks, tag.strip_prefix('#').unwrap_or(tag), &mut replaceable);
        if replaceable.is_empty() {
            eprintln!("[feature] vegetation_patch replaceable tag empty: {tag}（idk-3）");
        }
        Some(VegetationPatchConfig {
            replaceable,
            ground_state: crate::tree::BlockStateProvider::parse(cfg.get("ground_state"), blocks)?,
            vegetation_feature: crate::placement::PlacedFeature::parse_inline(cfg.get("vegetation_feature"), blocks)?,
            surface_floor: cfg.get("surface").and_then(|s| s.as_str()) == Some("floor"),
            depth: crate::placement::IntProvider::parse(cfg.get("depth")),
            extra_bottom_block_chance: cfg.get("extra_bottom_block_chance").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            vertical_range: cfg.get("vertical_range").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            vegetation_chance: cfg.get("vegetation_chance").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            xz_radius: crate::placement::IntProvider::parse(cfg.get("xz_radius")),
            extra_edge_column_chance: cfg.get("extra_edge_column_chance").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
        })
    }
}
pub struct VegetationPatchFeature;
impl VegetationPatchFeature {
    /// RNG 消费序（§一.9）：xz_radius.get ×2（i 先 j 后）→ 边缘列 nextFloat（仅 bl5 ∧ chance!=0）
    /// → 接受列 depth.get + [extra_bottom>0 恒 nextFloat] → placeGround 每步 groundState.get
    /// → vegetation 逐位置 nextFloat<chance + 内层 placed（同流）。
    /// waterlogged=true = WaterloggedVegetationPatchFeature 语义（§一.10）：① 尾段五向全实心位放水
    /// 并收缩位置集 ② vegetation 位 = 表面位本身（pos.down() 语义）③ WATERLOGGED 属性丢弃。
    /// ⚠️ 位置集迭代序：Java HashSet ≠ 本实现插入序（idk-5，RNG 流分叉点，诚实声明）。
    pub fn generate<F>(&self, ctx: &mut OreFeatureContext, config: &VegetationPatchConfig,
                       random: &mut ChunkRandom, x: i32, y: i32, z: i32,
                       waterlogged: bool, gen_vegetation: &mut F) -> bool
    where F: FnMut(&mut OreFeatureContext, &mut ChunkRandom, i32, i32, i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let i = config.xz_radius.get(random) + 1;                                                          // :29
        let j = config.xz_radius.get(random) + 1;                                                          // :30
        let dir_y: i32 = if config.surface_floor { -1 } else { 1 };                                         // surface.getDirection()
        let mut positions: Vec<(i32, i32, i32)> = Vec::new();
        for ix in -i..=i {                                                                                  // :45
            let bl = ix == -i || ix == i;
            for jz in -j..=j {                                                                              // :48
                let bl2 = jz == -j || jz == j;
                let bl3 = bl || bl2;
                let bl4 = bl && bl2;
                let bl5 = bl3 && !bl4;
                // :53 短路序逐字：角列全跳；非边列不消费；边列 ∧ chance!=0 恒消费
                if !bl4 && (!bl5 || (config.extra_edge_column_chance != 0.0f32
                    && !(random.next_float() > config.extra_edge_column_chance))) {
                    let cx = x + ix;
                    let cz = z + jz;
                    let mut cy = y;
                    let mut k = 0;
                    while ctx.block_at(cx, cy, cz) == air && k < config.vertical_range {                     // :56-58
                        cy += dir_y;
                        k += 1;
                    }
                    let opp = -dir_y;
                    let mut k2 = 0;
                    while ctx.block_at(cx, cy, cz) != air && k2 < config.vertical_range {                     // :60-62（-1 按非 air 保守下移）
                        cy += opp;
                        k2 += 1;
                    }
                    let sy = cy + dir_y;                                                                      // :64 mutable2
                    let below = ctx.block_at(cx, sy, cz);
                    if ctx.block_at(cx, cy, cz) == air && crate::tree::is_solid_id(ctx, below) {              // :66（sideSolid 近似 idk-3）
                        let mut depth = config.depth.get(random);                                             // :67
                        if config.extra_bottom_block_chance > 0.0f32
                            && random.next_float() < config.extra_bottom_block_chance { depth += 1; }          // :67（>0 恒消费）
                        // placeGround（:103-120）
                        let mut gy = sy;
                        let mut ok = true;
                        for step in 0..depth {
                            let st = config.ground_state.get(random);                                         // :107 每步恒消费
                            let cur = ctx.block_at(cx, gy, cz);
                            if cur != st {                                                                     // :109（block 级等值）
                                if !config.replaceable.contains(&cur) {                                        // :110（-1 → 不在表 → 失败）
                                    ok = step != 0;                                                             // :111 return i != 0
                                    break;
                                }
                                ctx.set_block(cx, gy, cz, st);                                                  // :114
                                gy += dir_y;                                                                    // :115
                            }
                        }
                        if ok {
                            positions.push((cx, sy, cz));                                                       // :70-71（bl6=false 不入 set）
                        }
                    }
                }
            }
        }
        // WaterloggedVegetationPatchFeature :24-33：五向（N,E,S,W,DOWN）全 side-solid 的位放水、集合收缩
        if waterlogged {
            let water = blocks.id("minecraft:water");
            let sides: [[i32; 3]; 5] = [[0, 0, -1], [1, 0, 0], [0, 0, 1], [-1, 0, 0], [0, -1, 0]];              // N,E,S,W,DOWN
            let mut kept: Vec<(i32, i32, i32)> = Vec::new();
            for (px, py, pz) in &positions {
                let mut all_solid = true;
                for s in sides {
                    if !crate::tree::is_solid_id(ctx, ctx.block_at(px + s[0], py + s[1], pz + s[2])) {
                        all_solid = false; break;                                                               // :51（isSolidBlockSide = !sideSolid 取反语义）
                    }
                }
                if all_solid {                                                                                  // :27 !isSolidBlockAroundPos
                    ctx.set_block(*px, *py, *pz, water);                                                        // :31
                    kept.push((*px, *py, *pz));
                }
            }
            positions = kept;
        }
        // generateVegetation（:81-95）：⚠️ Java HashSet 迭代序 ≠ 插入序（idk-5）
        for (vx, vy, vz) in &positions {
            if config.vegetation_chance > 0.0f32 && random.next_float() < config.vegetation_chance {                    // :91
                // :100 offset(direction2)（floor→UP）；waterlogged 变体 :55-64 pos.down() = 表面位本身
                let gy = if waterlogged { *vy } else { *vy - dir_y };
                let _ = gen_vegetation(ctx, random, *vx, gy, *vz);
            }
        }
        !positions.is_empty()                                                                                           // :33
    }
}

// ===== minecraft:root_system（RootSystemFeature.java + Config 13 字段）=====
#[derive(Clone)]
pub struct RootSystemConfig {
    pub feature: crate::placement::PlacedFeature,
    pub required_vertical_space_for_tree: i32,
    pub root_radius: i32,
    pub root_replaceable: Vec<i32>,
    pub root_state_provider: crate::tree::BlockStateProvider,
    pub root_placement_attempts: i32,
    pub max_root_column_height: i32,
    pub hanging_root_radius: i32,
    pub hanging_root_vertical_span: i32,
    pub hanging_root_state_provider: crate::tree::BlockStateProvider,
    pub hanging_root_placement_attempts: i32,
    pub allowed_vertical_water_for_tree: i32,
    pub allowed_tree_position: crate::placement::BlockPredicate,
}
impl RootSystemConfig {
    /// RootSystemFeatureConfig.java:13-29（全 required）。root_replaceable = tag（数据驱动展开）。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<RootSystemConfig> {
        let cfg = cfg?;
        let gi = |k: &str| cfg.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
        let mut root_replaceable = Vec::new();
        let tag = cfg.get("root_replaceable").and_then(|t| t.as_str()).unwrap_or("");
        expand_tag(blocks, tag.strip_prefix('#').unwrap_or(tag), &mut root_replaceable);
        Some(RootSystemConfig {
            feature: crate::placement::PlacedFeature::parse_inline(cfg.get("feature"), blocks)?,
            required_vertical_space_for_tree: gi("required_vertical_space_for_tree"),
            root_radius: gi("root_radius"),
            root_replaceable,
            root_state_provider: crate::tree::BlockStateProvider::parse(cfg.get("root_state_provider"), blocks)?,
            root_placement_attempts: gi("root_placement_attempts"),
            max_root_column_height: gi("root_column_max_height"),
            hanging_root_radius: gi("hanging_root_radius"),
            hanging_root_vertical_span: gi("hanging_roots_vertical_span"),
            hanging_root_state_provider: crate::tree::BlockStateProvider::parse(cfg.get("hanging_root_state_provider"), blocks)?,
            hanging_root_placement_attempts: gi("hanging_root_placement_attempts"),
            allowed_vertical_water_for_tree: gi("allowed_vertical_water_for_tree"),
            allowed_tree_position: crate::placement::BlockPredicate::parse(cfg.get("allowed_tree_position"), blocks),
        })
    }
}
/// hasSpaceForTree（:39-51 + isAirOrWater :53-60）。
fn root_has_space_for_tree(ctx: &OreFeatureContext, config: &RootSystemConfig,
                           x: i32, y: i32, z: i32, air: i32, water: i32) -> bool {
    for i in 1..=config.required_vertical_space_for_tree {                                                              // :42
        let st = ctx.block_at(x, y + i, z);
        if st != air {                                                                                                   // :54 isAir
            let allowed = i + 1 <= config.allowed_vertical_water_for_tree;                                                // :57（i = height+1）
            if !(allowed && st == water) { return false; }                                                                // :58
        }
    }
    true
}
/// generateRoots（:93-106）：每 attempt 恒 4 次 nextInt(r)（:98 x/z 各 2），命中 rootReplaceable 才 +1 次 state.get。
fn root_generate_roots(ctx: &mut OreFeatureContext, config: &RootSystemConfig,
                       random: &mut ChunkRandom, x: i32, y: i32, z: i32) {
    let r = config.root_radius;                                                                                          // :94
    if r <= 0 { return; }                                                                                                // 防御
    for _ in 0..config.root_placement_attempts {                                                                         // :97
        let rx = x + random.next_int_bound(r) - random.next_int_bound(r);                                                 // :98
        let rz = z + random.next_int_bound(r) - random.next_int_bound(r);
        let cur = ctx.block_at(rx, y, rz);
        if config.root_replaceable.contains(&cur) {                                                                       // :99
            let st = config.root_state_provider.get(random);                                                               // :100
            ctx.set_block(rx, y, rz, st);
        }
    }
}
/// generateHangingRoots（:108-121）：isAir 后、canPlaceAt 前消费 state.get（:115 求值序）。
fn root_generate_hanging_roots(ctx: &mut OreFeatureContext, config: &RootSystemConfig,
                               random: &mut ChunkRandom, x: i32, y: i32, z: i32, air: i32) {
    let i = config.hanging_root_radius;                                                                                  // :109
    let j = config.hanging_root_vertical_span;                                                                           // :110
    if i <= 0 || j <= 0 { return; }                                                                                      // 防御
    for _ in 0..config.hanging_root_placement_attempts {                                                                 // :112
        let hx = x + random.next_int_bound(i) - random.next_int_bound(i);                                                 // :113
        let hy = y + random.next_int_bound(j) - random.next_int_bound(j);
        let hz = z + random.next_int_bound(i) - random.next_int_bound(i);
        if ctx.block_at(hx, hy, hz) == air {                                                                              // :114
            let st = config.hanging_root_state_provider.get(random);                                                       // :115
            let below = ctx.block_at(hx, hy - 1, hz);
            let above = ctx.block_at(hx, hy + 1, hz);
            // :116 canPlaceAt（HangingRootsBlock）∧ 上方侧满方 DOWN——均 is_solid_id 近似（idk-3）
            if crate::tree::is_solid_id(ctx, below) && crate::tree::is_solid_id(ctx, above) {
                ctx.set_block(hx, hy, hz, st);                                                                                // :117
            }
        }
    }
}
pub struct RootSystemFeature;
impl RootSystemFeature {
    /// RNG 消费序（§一.11）：origin 门 0 → 列扫 0 → 树 = 内层 placed 全流（:73）→
    /// roots 柱（仅树命中，:74）→ hanging（仅树命中，:32）→ 恒 return true（:36，树失败也 true）。
    /// 谓词经 FeaturePlacementContext（pctx.block_at，idk-6）。
    pub fn generate<F>(&self, pctx: &crate::placement::FeaturePlacementContext,
                       ctx: &mut OreFeatureContext, config: &RootSystemConfig,
                       random: &mut ChunkRandom, x: i32, y: i32, z: i32,
                       gen_tree: &mut F) -> bool
    where F: FnMut(&mut OreFeatureContext, &mut ChunkRandom, i32, i32, i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let lava = blocks.id("minecraft:lava");
        let water = blocks.id("minecraft:water");
        if ctx.block_at(x, y, z) != air { return false; }                                                                   // :24
        let mut tree_done = false;
        let mut my = y;
        for i in 0..config.max_root_column_height {                                                                          // :65
            my += 1;                                                                                                         // :66 move(UP)
            if config.allowed_tree_position.test(pctx, x, my, z)
                && root_has_space_for_tree(ctx, config, x, my, z, air, water) {                                               // :67
                let below = ctx.block_at(x, my - 1, z);                                                                       // :68 down()
                if below == lava || !crate::tree::is_solid_id(ctx, below) { return false; }                                    // :69（注意：此处 false，非 :36 true）
                if gen_tree(ctx, random, x, my, z) {                                                                           // :73
                    for k in y..(y + i) {                                                                                      // :74/:83-91 generateRootsColumn
                        root_generate_roots(ctx, config, random, x, k, z);
                    }
                    tree_done = true;
                    break;
                }
            }
        }
        if tree_done {
            root_generate_hanging_roots(ctx, config, random, x, y, z, air);                                                    // :32
        }
        true                                                                                                                    // :36
    }
}
```

### Patch P6 — feature_loader.rs：ConfiguredFeature 扩字段 + init None

**P6a** 字段（sea_pickle_config 之后）：

old:
```rust
    // —— batchB（mc-1216）：kelp 走 DefaultFeatureConfig 空配置（无字段，分发直发）——
    pub seagrass_config: Option<crate::feature::SeagrassConfig>,      // minecraft:seagrass（ProbabilityConfig）
    pub sea_pickle_config: Option<crate::feature::SeaPickleConfig>,   // minecraft:sea_pickle（CountConfig=IntProvider）
}
```
new:
```rust
    // —— batchB（mc-1216）：kelp 走 DefaultFeatureConfig 空配置（无字段，分发直发）——
    pub seagrass_config: Option<crate::feature::SeagrassConfig>,      // minecraft:seagrass（ProbabilityConfig）
    pub sea_pickle_config: Option<crate::feature::SeaPickleConfig>,   // minecraft:sea_pickle（CountConfig=IntProvider）
    // —— batchC（mc-1216）：残差清空 11/16 + nether_forest_vegetation + waterlogged 变体 ——
    pub bamboo_probability: Option<f32>,                              // minecraft:bamboo（ProbabilityConfig 1 字段）
    pub block_column_config: Option<Box<crate::feature::BlockColumnConfig>>,   // minecraft:block_column（含谓词树）
    pub single_state_config: Option<crate::feature::SingleStateConfig>,        // minecraft:forest_rock（SingleStateFeatureConfig）
    pub random_boolean_config: Option<crate::tree::RandomBooleanSelectorConfig>,          // minecraft:random_boolean_selector
    pub simple_random_selector_config: Option<crate::tree::SimpleRandomSelectorConfig>,   // minecraft:simple_random_selector
    pub vegetation_patch_config: Option<Box<crate::feature::VegetationPatchConfig>>,      // minecraft:(waterlogged_)vegetation_patch
    pub root_system_config: Option<Box<crate::feature::RootSystemConfig>>,                // minecraft:root_system
    pub nether_forest_vegetation_config: Option<crate::feature::NetherForestVegetationConfig>, // minecraft:nether_forest_vegetation
}
```
（blue_ice / desert_well / ice_spike / vines = DefaultFeatureConfig 空 config，无字段，分发直发——同 monster_room/kelp 先例。）

**P6b** init None：

old:
```rust
            seagrass_config: None,
            sea_pickle_config: None,
        };
```
new:
```rust
            seagrass_config: None,
            sea_pickle_config: None,
            bamboo_probability: None,
            block_column_config: None,
            single_state_config: None,
            random_boolean_config: None,
            simple_random_selector_config: None,
            vegetation_patch_config: None,
            root_system_config: None,
            nether_forest_vegetation_config: None,
        };
```

### Patch P7 — feature_loader.rs：parse 分发（sea_pickle 分支后、catch-all 前）

old:
```rust
        } else if type_name == "minecraft:sea_pickle" {
            // CountConfig：count = IntProvider（CountConfig.java:9-25；E-1「非 IntProvider / 1..25」不符）
            cf.sea_pickle_config = crate::feature::SeaPickleConfig::parse(cfg, blocks);
            if cf.sea_pickle_config.is_none() {
                eprintln!("[feature-loader] sea_pickle config parse failed: {id}");
            }
        } else {
```
new:
```rust
        } else if type_name == "minecraft:sea_pickle" {
            // CountConfig：count = IntProvider（CountConfig.java:9-25；E-1「非 IntProvider / 1..25」不符）
            cf.sea_pickle_config = crate::feature::SeaPickleConfig::parse(cfg, blocks);
            if cf.sea_pickle_config.is_none() {
                eprintln!("[feature-loader] sea_pickle config parse failed: {id}");
            }
        } else if type_name == "minecraft:bamboo" {
            // ProbabilityConfig：probability 单字段（BambooFeature.java:17；batchC §一.1）
            cf.bamboo_probability = Some(
                cfg.and_then(|c| c.get("probability")).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32);
        } else if type_name == "minecraft:block_column" {
            // BlockColumnFeatureConfig.java:13-21（batchC §一.2；层高嵌套 IntProvider → P1 WeightedProviders）
            cf.block_column_config = crate::feature::BlockColumnConfig::parse(cfg, blocks).map(Box::new);
            if cf.block_column_config.is_none() {
                eprintln!("[feature-loader] block_column config parse failed: {id}");
            }
        } else if type_name == "minecraft:blue_ice" || type_name == "minecraft:desert_well"
            || type_name == "minecraft:ice_spike" || type_name == "minecraft:vines" {
            // DefaultFeatureConfig 空 config（各自 Feature.java；4 个 JSON config={} 实证）
        } else if type_name == "minecraft:forest_rock" {
            // SingleStateFeatureConfig：state 固定 BlockState（0 随机消费；batchC §一.5）
            cf.single_state_config = crate::feature::SingleStateConfig::parse(cfg, blocks);
            if cf.single_state_config.is_none() {
                eprintln!("[feature-loader] forest_rock config parse failed: {id}");
            }
        } else if type_name == "minecraft:random_boolean_selector" {
            // RandomBooleanFeatureConfig.java:9-15：feature_true/feature_false（placed Holder；batchC §一.7）
            cf.random_boolean_config = crate::tree::RandomBooleanSelectorConfig::parse(cfg, blocks);
            if cf.random_boolean_config.is_none() {
                eprintln!("[feature-loader] random_boolean_selector config parse failed: {id}");
            }
        } else if type_name == "minecraft:simple_random_selector" {
            // SimpleRandomFeatureConfig：placed 列表无 chance（SimpleRandomFeature.java；batchC §一.8/E-1）
            cf.simple_random_selector_config = crate::tree::SimpleRandomSelectorConfig::parse(cfg, blocks);
            if cf.simple_random_selector_config.is_none() {
                eprintln!("[feature-loader] simple_random_selector config parse failed: {id}");
            }
        } else if type_name == "minecraft:vegetation_patch" || type_name == "minecraft:waterlogged_vegetation_patch" {
            // VegetationPatchFeatureConfig.java:14-28（batchC §一.9/§一.10；两 type 同 config，generate 侧分语义）
            cf.vegetation_patch_config = crate::feature::VegetationPatchConfig::parse(cfg, blocks).map(Box::new);
            if cf.vegetation_patch_config.is_none() {
                eprintln!("[feature-loader] vegetation_patch config parse failed: {id}");
            }
        } else if type_name == "minecraft:root_system" {
            // RootSystemFeatureConfig.java:13-29（13 字段；batchC §一.11；谓词 any_of/matching_block_tag → P2）
            cf.root_system_config = crate::feature::RootSystemConfig::parse(cfg, blocks).map(Box::new);
            if cf.root_system_config.is_none() {
                eprintln!("[feature-loader] root_system config parse failed: {id}");
            }
        } else if type_name == "minecraft:nether_forest_vegetation" {
            // NetherForestVegetationFeatureConfig.java:9-16（batch0 误捕澄清后正规接入；batchC §一.12）
            cf.nether_forest_vegetation_config = crate::feature::NetherForestVegetationConfig::parse(cfg, blocks);
            if cf.nether_forest_vegetation_config.is_none() {
                eprintln!("[feature-loader] nether_forest_vegetation config parse failed: {id}");
            }
        } else {
```

### Patch P8 — feature_loader.rs：generate 分发（sea_pickle 分支后、catch-all 前）

old:
```rust
    } else if cf.type_name == "minecraft:sea_pickle" {
        match &cf.sea_pickle_config {
            Some(pc) => crate::feature::SeaPickleFeature::generate(octx, pc, random, x, y, z),
            None => { eprintln!("[feature-loader] sea_pickle without config: {}", cf.id); false }
        }
    } else {
```
new:
```rust
    } else if cf.type_name == "minecraft:sea_pickle" {
        match &cf.sea_pickle_config {
            Some(pc) => crate::feature::SeaPickleFeature::generate(octx, pc, random, x, y, z),
            None => { eprintln!("[feature-loader] sea_pickle without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:bamboo" {
        match cf.bamboo_probability {
            Some(prob) => crate::feature::BambooFeature.generate(octx, prob, random, x, y, z),
            None => false, // parse 恒 Some；防御
        }
    } else if cf.type_name == "minecraft:block_column" {
        match &cf.block_column_config {
            Some(bc) => crate::feature::BlockColumnFeature.generate(ctx, octx, bc, random, x, y, z),
            None => { eprintln!("[feature-loader] block_column without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:blue_ice" {
        crate::feature::BlueIceFeature.generate(octx, random, x, y, z)
    } else if cf.type_name == "minecraft:desert_well" {
        crate::feature::DesertWellFeature.generate(octx, random, x, y, z)
    } else if cf.type_name == "minecraft:forest_rock" {
        match &cf.single_state_config {
            Some(sc) => crate::feature::ForestRockFeature.generate(octx, sc, random, x, y, z),
            None => { eprintln!("[feature-loader] forest_rock without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:ice_spike" {
        crate::feature::IceSpikeFeature.generate(octx, random, x, y, z)
    } else if cf.type_name == "minecraft:random_boolean_selector" {
        match &cf.random_boolean_config {
            Some(rc) => {
                // RandomBooleanFeature.java:22-25：nextBoolean（=next(1)）选支，同流下钻（§一.7）
                let pick_true = random.next_int_bound(2) == 1;
                let pf = if pick_true { &rc.feature_true } else { &rc.feature_false };
                pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                    generate_nested(&pf.configured_feature, c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                })
            }
            None => { eprintln!("[feature-loader] random_boolean_selector without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:simple_random_selector" {
        match &cf.simple_random_selector_config {
            Some(ssc) => {
                // SimpleRandomFeature.java:22-24：Util.getRandom = nextInt(n) 均匀选一（parse 保证非空）（§一.8）
                let idx = random.next_int_bound(ssc.features.len() as i32) as usize;
                let pf = &ssc.features[idx];
                pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                    generate_nested(&pf.configured_feature, c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                })
            }
            None => { eprintln!("[feature-loader] simple_random_selector without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:vegetation_patch" || cf.type_name == "minecraft:waterlogged_vegetation_patch" {
        match &cf.vegetation_patch_config {
            Some(vc) => {
                // VegetationPatchFeature.java:97-101（内嵌 placed 同流）；waterlogged 变体 §一.10
                let wl = cf.type_name == "minecraft:waterlogged_vegetation_patch";
                crate::feature::VegetationPatchFeature.generate(octx, vc, random, x, y, z, wl, &mut |o2, r2, gx, gy, gz| {
                    generate_nested(&vc.vegetation_feature.configured_feature, ctx, r2, gx, gy, gz, o2, cache, biome_temp, biome_rainfall)
                })
            }
            None => { eprintln!("[feature-loader] vegetation_patch without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:root_system" {
        match &cf.root_system_config {
            Some(rc) => {
                // RootSystemFeature.java:73 config.feature.generateUnregistered（同流下钻）（§一.11）
                crate::feature::RootSystemFeature.generate(ctx, octx, rc, random, x, y, z, &mut |o2, r2, gx, gy, gz| {
                    generate_nested(&rc.feature.configured_feature, ctx, r2, gx, gy, gz, o2, cache, biome_temp, biome_rainfall)
                })
            }
            None => { eprintln!("[feature-loader] root_system without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:nether_forest_vegetation" {
        match &cf.nether_forest_vegetation_config {
            Some(nc) => crate::feature::NetherForestVegetationFeature.generate(octx, nc, random, x, y, z),
            None => { eprintln!("[feature-loader] nether_forest_vegetation without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:vines" {
        crate::feature::VinesFeature.generate(octx, random, x, y, z)
    } else {
```
（签名核对：generate_configured 现有形参 (cf, ctx, octx, random, x, y, z, biome_temp, biome_rainfall, cache)——block_column 用 ctx（谓词）；四个嵌套型用闭包 + cache（random_selector arm 同款模式）；其余只需 octx/random/x/y/z。`FeaturePlacementContext` / `OreFeatureContext` 均未加字段 → 两个构造点（worldgen_handle.rs / bin-diag/b5b6_smoke.rs）零改动。）

### Patch P9 — feature_loader.rs：preload_all 嵌套补载扩容（新容器内层 placed 同口径）

**P9a** 首段收集循环（selector_config 之后补新容器）：

old:
```rust
        for cf in self.configured.values() {
            if let Some(sc) = &cf.selector_config {
                for (_, pf) in &sc.features { queue.push(pf.configured_feature.clone()); }
                if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
            }
        }
```
new:
```rust
        for cf in self.configured.values() {
            if let Some(sc) = &cf.selector_config {
                for (_, pf) in &sc.features { queue.push(pf.configured_feature.clone()); }
                if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
            }
            // —— batchC（mc-1216）：selector 之外的新嵌套容器同口径补载（防运行时 cache miss）——
            if let Some(pc) = &cf.patch_config { queue.push(pc.feature.configured_feature.clone()); }
            if let Some(rc) = &cf.random_boolean_config {
                queue.push(rc.feature_true.configured_feature.clone());
                queue.push(rc.feature_false.configured_feature.clone());
            }
            if let Some(ssc) = &cf.simple_random_selector_config {
                for pf in &ssc.features { queue.push(pf.configured_feature.clone()); }
            }
            if let Some(vc) = &cf.vegetation_patch_config { queue.push(vc.vegetation_feature.configured_feature.clone()); }
            if let Some(rc) = &cf.root_system_config { queue.push(rc.feature.configured_feature.clone()); }
        }
```

**P9b** 不动点循环内的嵌套 requeue（selector requeue 之后补同款）：

old:
```rust
                        // 嵌套 selector（selector 内层 placed → configured 又是 selector）继续入队
                        if let Some(sc) = &cf.selector_config {
                            for (_, ipf) in &sc.features { queue.push(ipf.configured_feature.clone()); }
                            if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
                        }
```
new:
```rust
                        // 嵌套 selector/patch/新容器（内层 placed → configured 又是容器）继续入队
                        if let Some(sc) = &cf.selector_config {
                            for (_, ipf) in &sc.features { queue.push(ipf.configured_feature.clone()); }
                            if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
                        }
                        if let Some(pc) = &cf.patch_config { queue.push(pc.feature.configured_feature.clone()); }
                        if let Some(rc) = &cf.random_boolean_config {
                            queue.push(rc.feature_true.configured_feature.clone());
                            queue.push(rc.feature_false.configured_feature.clone());
                        }
                        if let Some(ssc) = &cf.simple_random_selector_config {
                            for ipf in &ssc.features { queue.push(ipf.configured_feature.clone()); }
                        }
                        if let Some(vc) = &cf.vegetation_patch_config { queue.push(vc.vegetation_feature.configured_feature.clone()); }
                        if let Some(rc) = &cf.root_system_config { queue.push(rc.feature.configured_feature.clone()); }
```
（既有行为保留：补载的壳 placed 的 configured 经查 cache 预载；patch_config 为既有字段，本批一并纳入防 moss_vegetation/cave_vine_in_moss 类仅经 patch/patch-族内层引用的 id miss。）

## 三、静态自检清单（未编译验证声明）

**状态：全部 patch 未编译、未运行**（沙箱无 shell / 本任务约定不跑命令）。静态自检如下：

- [x] **循环变量 usize 推断**：所有 `for l in 0..n` 循环变量均为 i32（`n: i32`）或 usize（`0..config.root_placement_attempts` 等 i32 场景用 `for _ in`）；索引处统一 `as usize`：`heights[l as usize]`（l∈[0,n) 守卫）、`hm[(lz*16+lx) as usize]`（get_top_y_world_surface，[0,15] 守卫同 batchB）、`spots[... as usize]`（next_int_bound(5)∈[0,4]）、`ssc.features[idx]`（idx< len 由 next_int_bound 保证）。`for depth in [1,2]` 数组迭代无索引。
- [x] **`as f64 <` 泛型歧义**（batchA 实踩形态）：全部浮点比较写法 = `(expr as f64) < x` 或 f32/f64 具体类型变量比较——`< probability as f64`（bamboo）、`< config.extra_edge_column_chance`（f32 对 f32）、`< config.vegetation_chance`（f32）、`> 0.75f32`（ice_spike）、`d2 <= (f * f) as f64`（forest_rock，两侧具体类型）；**无 `as f64` 直接后随 `<` 字面量的推断面**。
- [x] **panic 面**：`next_int_bound(bound>0)` 逐一核对——bamboo(12/4)、blue_ice(5/6/k≥1)、desert_well(5)、forest_rock(2)、ice_spike(4/2/60/30/5)、nether(门后 w/h≥1 守卫)、veg_patch（IntProvider 各变体 bound≥1；xz_radius/depth 由 codec validating 保证）、root_system（r/i/j ≥1 codec 保证 + ≤0 早退防御）。切片索引：`heights[m as usize]`（adjust 循环 m∈[0,len) 由 tip 两分支界保证）；`entries[0]`/`weighted[0]`（empty 早退守卫在前）。`ssc.features.len()` >0 由 parse 保证（空返 None）。
- [x] **借用/所有权**：`get_top_y_world_surface`/`is_soil_id`/`dirt_tag_ids` 均 & 或按值；`let blocks: &BlockRegistry = ctx.blocks;` 后续 `ctx.set_block/block_at(&self)` 混用 = batchB SeagrassFeature 同型（NLL：blocks 最后使用点之后无再借用）；两个闭包型 generate（veg_patch/root_system）的 `gen_*: &mut F` 与 `ctx: &mut` 分时使用（调用点无同时存活借用）；feature_loader 四个嵌套 arm 的闭包捕获 `octx(&mut)/cache(&)/ctx(&)` 与 random_selector arm 同构（同函数内既有先例）。
- [x] **区间/端点**：`-i..=i`/`0..total`/`y..(y+i)`（roots 柱半开=Java `k<maxY`）/`1..=required`（含端点=Java `i<=`）/`0..depth`（半开=Java `i<depth`）；`(x-k)..=(x+k)` 含端点 = Java `l<=x+k`。
- [x] **分发两侧一致**（parse ↔ generate）：bamboo/block_column/blue_ice/desert_well/forest_rock/ice_spike/random_boolean_selector/simple_random_selector/vegetation_patch+waterlogged/root_system/nether_forest_vegetation/vines 12 组两侧精确匹配；catch-all 告警 + record_unknown_type 原样保留（缓装 5 项仍走该路径）。
- [x] **构造点完备**：`FeaturePlacementContext` / `OreFeatureContext` **未加字段** → worldgen_handle.rs / bin-diag/b5b6_smoke.rs 两构造点零改动；`ConfiguredFeature` 加的 8 个 Option 字段带 init None（P6b）→ 无构造点连锁。
- [x] **RNG 消费序**：12 组逐行对拍表见 §一；高危短路点（ice_spike :46 复合条件、veg_patch :53 边缘 chance、block_column 先全抽后判 0、nether state.get 在 air 检查前、desert_well nextInt(5) 恒消费、bamboo canPlace 失败仍 true）均已按 Java 求值序落位并注释标注。
- [x] **权重桶**：WeightedProviders get 与 WeightedList 同构（next_int_bound(total) 后线性扣减）；total=0 由 `entries.is_empty()` + 数值权重≥1 防御（JSON 全 0 权重形态数据集无先例，返回 entries[0] 兜底同 BlockStateProvider 口径）。

## 四、错误记录（五段式）

### E-1 simple_random_selector 不是 RandomFeature——简报「selector 类注意 RNG 差异」实锤为不同类不同语义

- **现象**：任务简报将 random_boolean_selector/simple_random_selector 归为「selector 类，注意与 random_selector 的 RNG 语义差异」。初查 RandomFeature.java（chance 逐项）疑似 simple_random_selector 复用同 config（其 config 字段也叫 features），差点按「features 无 default + chance 缺省」实现。
- **根因**：1.20.1 存在**独立的 SimpleRandomFeature / SimpleRandomFeatureConfig**（Feature.java 注册名 `simple_random_selector` 指向它）：entry 无 chance 字段，选择 = `Util.getRandom` = `nextInt(n)` 均匀一次消费；与 random_selector（RandomFeature：逐项 `nextFloat()<chance` 即选即返、落空走 default 不抽）**RNG 消费次数与分布完全不同**。两者误同实现会系统性错位 RNG 流（一个 entry 场景：1 次 nextInt(1) vs 1 次 nextFloat）。
- **定位**：Feature.java 注册表反查类名 → SimpleRandomFeature.java 逐行读（26 行）→ 与 RandomFeature.java 并排对拍 → 数据集 4 个 JSON entry 形态核实（无 chance 字段）。
- **修复**：P4 独立 `SimpleRandomSelectorConfig`（Vec<PlacedFeature>，无 chance）+ P8 generate `next_int_bound(n)` 均匀选一；random_boolean_selector 的 `nextBoolean()` 同轮核实为 `next(1)`（与 `nextInt(2)` 幂二 fast path 同消费），以 `next_int_bound(2)==1` 表达并注释声明。
- **教训**：「同名族类」（selector 类）必须逐一反查 Feature.java 注册表确认实际类，config 字段名相同（features）不代表同一 codec/语义；本批再次验证 batchB E-1 结论——简报分类词不进一手源核对不可用。

### E-2 简报范围与引擎数据集错位两处（fixed_placement / waterlogged_vegetation_patch）

- **现象**：① 简报将 fixed_placement（end_platform 引用 1）列为批次目标；② 按简报 16 项清单实施时，simple_random_selector 的 `clay_pool_with_dripleaves`（lush_caves_clay 经 random_boolean_selector false 支）type 实为 `waterlogged_vegetation_patch`——不在 16 项清单内。
- **根因**：① 引擎数据集是 1.20.1（batchB 已有教训），`end_platform` placed_feature 与 fixed_placement modifier 均为 **1.21.6 新增**——1.20.1 placed_feature glob+grep 实证 0 引用，引擎当前运行该 patch **不可达**；② waterlogged_vegetation_patch 是 VegetationPatchFeature 的直接子类，藏在 random_boolean_selector 内层，此前 catch-all false 使其从未暴露，清单据实测 unknown 反推时被漏。
- **定位**：① `versions/1.20.1/data/worldgen/data/minecraft/worldgen/placed_feature` glob end*.json（无 end_platform）+ 全数据 grep fixed_placement（0）；1.21.6 侧 end_platform.json 读到 `[[100,49,0]]` 三元组形态。② lush_caves_clay.json 顺藤读内层 type。
- **修复**：① fixed_placement 照做（向前兼容，parse 支持三元组/对象双形态）+ 文档头声明当前 0 触发；② waterlogged_vegetation_patch 补入覆盖面（#10，VegetationPatchFeature::generate 加 waterlogged 形参，~20 行增量），不静默留残差。
- **教训**：批次范围以「简报清单」为起点但须以「数据集 grep 全集 + 内层嵌套展开」封闭（selector/patch 内层 type 要递归清点）；「1.21.6 数据有的」≠「引擎会跑到的」（双版数据集都要 grep 一遍定可达性）。

### E-3 IntProvider::WeightedList 无法表达 block_column 层高（嵌套 provider 形态）

- **现象**：cave_vine.json / dripleaf.json 的 block_column `layers[].height` 为 `{"type":"minecraft:weighted_list","distribution":[{"data":{<IntProvider>},"weight":N},...]}`——data 是**对象**（嵌套 IntProvider），现有 `IntProvider::WeightedList(Vec<(i32,i32)>)` 只支持数值 data。
- **根因**：Java 侧是 `WeightedListIntProvider`（distribution of IntProvider）；旧 parse 分支 `e.get("data").and_then(|x| x.as_f64())` 对对象返回 None → data 恒 0 → 若不扩，所有 block_column 层高恒 0（j==0 → 特征全灭），且无告警。
- **定位**：读 BlockColumnFeatureConfig.java codec（height = IntProvider.NON_NEGATIVE_CODEC）→ 数据集 JSON 逐个核对 height 形态（weighted_list 嵌套 2 处 / plain int 1 处 / biased_to_bottom 1 处）。
- **修复**：P1 新变体 `WeightedProviders(Vec<(IntProvider,i32)>, i32)`，parse 在 weighted_list 分支内先探测 data 是否对象再分派（数值形态走旧路，零回归面）；get/max_value 同构补全。
- **教训**：扩 IntProvider/FloatProvider 类多态解析时，先对数据集做「形态普查」（每个 variant 的 JSON 形态穷举），再决定「复用变体 + 分派」还是「新变体」——直接信旧分支的宽松 contains 匹配会把新形态静默吞成 0。

### E-4 双版交叉核对发现 NetherForestVegetationFeature 唯一 diff（边界 API 改名）

- **现象**：Compare-Object 双版对拍 17 个实装对象，16 个逐行一致，唯 `NetherForestVegetationFeature.java` 2 行 diff：`i + 1 < structureWorldAccess.getTopY()`（1.20.1）↔ `i + 1 <= structureWorldAccess.getTopYInclusive()`（1.21.6）。
- **根因**：1.21.6 把 `getTopY()`（开界）改名为 `getTopYInclusive()`（闭界）——两写法边界值相同（top-1 < i+1 ⟺ i+1 ≤ top-1+1），**纯 API 改名非语义变更**；不核对会误判「1.21.6 边界放宽」而错改引擎口径（引擎无此区分）。
- **定位**：pwsh Compare-Object 全对象批量 diff → diff 行人肉核对边界语义。
- **修复**：引擎按 1.20.1 语义实现（`y + 1 < min_y + height`），注释标注两版写法对应关系。
- **教训**：双版 diff 不是「diff=0 才安心」，而是「diff 必须逐条判语义」——改名型 diff（getTopY→getTopYInclusive）与行为型 diff 处置完全不同；批量工具出粗筛、人肉定语义，两步都不可省。

### E-5（自查纠错，未入库）交付稿 vegetation_patch 曾把 `ok=false` 的列也 push 进位置集

- **现象**：第一稿 placeGround 结果处理写为无条件 push + `let accept = ...; let _ = accept;` 占位。
- **根因**：Java :69-72 `bl6 = placeGround(...); if (bl6) set.add(...)` ——`ok=false`（首个非替换块在 step 0）时该列**不入集合**，也就不参与 vegetation 滚点与返回值判定；占位写法会多出假位置 → vegetation 多滚 RNG（流错位 + 多放置）。
- **定位**：静态自检「区间/端点」项复核 Java :69-72 时发现占位残留。
- **修复**：交付稿已改为 `if ok { positions.push(...) }`（P5 第二段现文），无占位代码。
- **教训**：feature 级「返回值/集合成员」与「放置量」是两条独立判定线（本例 ok 只管入集，不管已放方块）；逐行注释 Java 行号时同步核对每条控制流的出口，不留 `let _ =` 占位过夜。

## 五、@anchor.idk 清单（随 patch 注释落码）

- **idk-1**（本族通用）：方块 state 属性位不可表达——BAMBOO.AGE/LEAVES/STAGE、VINE faces、CACTUS age、CAVE_VINES age/berries、BIG_DRIPLEAF facing/tilt、WATERLOGGED、SUSPICIOUS_SAND 方块实体/loot——均裸 id 放置，palette 对比按「存在性」对齐（batchA/B 同族口径）；对应 RNG 消费全部保留。
- **idk-2**（特征级高度图邻域，bamboo podzol 圈）：chunk 内走 world_surface 桶（pre-carver 快照）；邻域 block_at 下扫读到**含本 chunk 已生成 feature 的实况**（Java 高度图不含 feature 产物）→ 早生成 feature 顶高后续列。同 batchB idk-2 家族。
- **idk-3**（判定近似 + tag 依赖族）：`isSolid`/`isSoil`/`isStone`/`isSideSolidFullSquare`/各 `canPlaceAt` → `is_solid_id` + tag 展开（dirt/base_stone_overworld/nylium/moss_replaceable/azalea_grows_on/replaceable_by_trees/root_replaceable）近似；tag JSON 缺失 → 空表 → 门失效（有 eprintln 告警不静默）；谓词 `-1`（不可读）一律保守拒绝/阻挡。
- **idk-4**（硬编码常量）：`SEA_LEVEL=63`（blue_ice 海平面门；iceberg 预留）与 ice_spike 底柱 `while by > 50` 字面量（同 Java 源字面量，非 sea level）——多维度参数化课题落地时一并改注入。
- **idk-5**（vegetation_patch 位置集迭代序，**RNG 流分叉点，最高优先验收观察项**）：Java `HashSet<BlockPos>` 迭代序 = BlockPos hash 桶序，与引擎插入序不同 → vegetationChance 滚点与内层 feature 的 (位置, RNG) 配对在多位置 patch 上与 Java 错位；单位置 patch 无差。方向：如需逐位对齐需移植 Java HashMap 桶序（BlockPos.hashCode + hash spreading），建议随 palette 对比反馈决定是否立项。
- **idk-6**（root_system 谓词读取面）：`allowed_tree_position` 经 FeaturePlacementContext.block_at 闭包求值——假设该闭包含当前 chunk 已生成 feature 实况；与 Java StructureWorldAccess 直读等价性依赖 worldgen_handle 接线（未在本批验证）。
- **idk-7**（嵌套暴露 unknown 面，见 §〇.3）：selector 族接入后 `pointed_dripstone` / `coral_tree|claw|mushroom` / `huge_brown|red_mushroom` 等内层 type 可能首次进入实测 unknown——属「行为解锁」非回归；验收比对时与残差保留集（n=5）分开列。

## 六、主会话后续动作建议

1. 应用顺序 P1→P2→P3（placement.rs）→ P4（tree.rs）→ P5（feature.rs，两段连续追加）→ P6→P7→P8→P9（feature_loader.rs）；`cargo build --offline -p worldgen --release` 编译门。
2. WG_FEATURE_UNKNOWN_LOG=1 实跑验收：**残差保留集 == {minecraft:dripstone_cluster, fossil, iceberg, large_dripstone, sculk_patch}（n=5）**；`minecraft:nether_forest_vegetation` / `mod:minecraft:fixed_placement` 消失；idk-7 嵌套暴露集（pointed_dripstone / coral 族 / huge_*_mushroom / waterlogged 之外的意外项）单列核对，出现即回本清单比对。
3. judge 审查（candidate 前 SHOULD）：重点核 §一.6 ice_spike :46 短路消费序、§一.9 vegetation_patch :53/:67/:107 三处求值序、§一.11 root_system :69 提前 false 与 :36 恒 true 的双出口、P1c weighted_list 分派对既有数值形态的零回归。
4. 缓装 5 项入台账：dripstone_cluster+large_dripstone 合并「dripstone 基建批」（CaveSurface/DripstoneHelper/FloatProvider 三基建先行）；iceberg 独立小批；sculk_patch 挂 sculk 行为层课题；fossil 挂结构模板层课题（§〇.2 已附体量评估）。



### Patch P4 — tree.rs：selector 族两新 config（追加到文件尾 `}` 之后，即 FallenTreeConfig generate 闭合大括号后）

old（tree.rs 文件尾，L1173-1180）:
```rust
    /// FallenTreeFeature.applyDecorators（L116-121）：Generator(positions, ∅, ∅)
    fn apply_decorators(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                        positions: &[[i32; 3]], decorators: &[TreeDecorator]) {
        for d in decorators {
            d.generate(ctx, random, positions, &[], &[]);
        }
    }
}
```
new（原样保留 + 尾部追加）:
```rust
    /// FallenTreeFeature.applyDecorators（L116-121）：Generator(positions, ∅, ∅)
    fn apply_decorators(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                        positions: &[[i32; 3]], decorators: &[TreeDecorator]) {
        for d in decorators {
            d.generate(ctx, random, positions, &[], &[]);
        }
    }
}

// ===== batchC（mc-1216）：selector 族两新 config =====
// 一手源：versions/1.20.1 + 1.21.6 双版 mc_src_extract（两版逐行一致，batchC §一.7/§一.8）：
//   world/gen/feature/RandomBooleanFeature.java(+Config) / SimpleRandomFeature.java(+Config)

/// random_boolean_selector（RandomBooleanFeature.java:22-25 + Config.java:9-15）：
/// nextBoolean（= next(1)）选真/假支，内层 placed 各自完整链共用同一 RNG 流。
/// Rust 表达 = next_int_bound(2) == 1（Java nextInt(2) 幂二 fast path = next(1)，同 1 bit 消费）。
#[derive(Clone)]
pub struct RandomBooleanSelectorConfig {
    pub feature_true: crate::placement::PlacedFeature,
    pub feature_false: crate::placement::PlacedFeature,
}
impl RandomBooleanSelectorConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<RandomBooleanSelectorConfig> {
        let v = v?;
        Some(RandomBooleanSelectorConfig {
            feature_true: crate::placement::PlacedFeature::parse_inline(v.get("feature_true"), blocks)?,
            feature_false: crate::placement::PlacedFeature::parse_inline(v.get("feature_false"), blocks)?,
        })
    }
}

/// simple_random_selector（**SimpleRandomFeature.java:16-25**——非 RandomFeature！+ SimpleRandomFeatureConfig）：
/// entries 无 chance 字段；Util.getRandom = nextInt(n) 均匀选一（恒 1 次消费）。
/// ⚠️ 与 random_selector 的「逐项 nextFloat<chance 即选即返」语义完全不同（batchC E-1）。
#[derive(Clone)]
pub struct SimpleRandomSelectorConfig {
    pub features: Vec<crate::placement::PlacedFeature>,
}
impl SimpleRandomSelectorConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<SimpleRandomSelectorConfig> {
        let v = v?;
        let mut features = Vec::new();
        if let Some(arr) = v.get("features").and_then(|f| f.as_array()) {
            // entry 本体 = placed feature 对象（feature + placement）
            for e in arr {
                if let Some(pf) = crate::placement::PlacedFeature::parse_inline(Some(e), blocks) {
                    features.push(pf);
                }
            }
        }
        if features.is_empty() { return None; }
        Some(SimpleRandomSelectorConfig { features })
    }
}
```


