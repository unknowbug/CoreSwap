# batchA worker 交付 —— mc-1216 features 接管批次 A（4 configured feature 类型 + noise_threshold_count modifier）

> 角色：worker（代码交付）。**未编译验证**（沙箱无 shell，静态对拍交付）。
> 一手源：`versions/1.21.6/data/mc_src_extract/net/minecraft/`（只读，逐文件 file:line 引用）。
> 引擎参照：`worldgen-core/src/feature_loader.rs`（HEAD 批次 0 后）/ `placement.rs` / `feature.rs` / `tree.rs`。
> 状态：draft（候选需主会话应用 + 编译 + 实跑 unknown 残差验收后才可升 candidate）。
> 日期标签：本文不写 YYMMDD 标签（260904-06 纪律：主会话落盘/归档前 Get-Date 取真实时间补）。

## 〇、本批覆盖面声明

| # | 目标 | 本批做了 | 本批留了（残差 → 次批/登记） |
|---|------|---------|------------------------------|
| 1 | minecraft:monster_room（DungeonFeature） | **全量**：预检扫描 + r∈[1,5] 门 + 壳体 mossy/cobblestone + 空洞 + 箱子 2×3 attempts + spawner；RNG 消费序逐行对拍 | chest/spawner 仅放置 state id（facing/方块实体/战利品表无实体层——**loot seed 消费 nextLong 保留** 保 RNG 流）；`isSolid` 用 is_solid_ish 近似（已知限制同 B5/B6） |
| 2 | minecraft:lake（LakeFeature.Config fluid+barrier） | **全量**：椭圆团布尔网格 (16×16×8)、双检验 pass、放置 pass（≥4 = cave_air）、barrier pass（v≥4 时 nextInt(2)）、水湖结冰 | `canSetIce` 用当前 chunk biome temp<0 近似（Java 按列 biome）；方块 state 属性等值比较降级为 id 等值；scheduleBlockTick/postProcess/fluidTick 无实体层 no-op；**起点距 chunk 边 <3 格且 ca_min 关闭时 block_at 越界读 -1 → 保守 abort**（声明，见 §五 idk-3） |
| 3 | minecraft:geode（GeodeFeatureConfig 多子配置） | **全量**（core+layers+crack+placements 一步到位，未分步）：分布点/外壁距离/层阈值椭圆场（DoublePerlin 噪声项经 world_seed 接线**精确**复刻）、crack 分支、invalid_blocks_threshold 门、inner_placements 放置 | FACING/WATERLOGGED 属性位丢失（引擎 state=i32，已知限制——朝向/含水不可表达）；`BuddingAmethystBlock.canGrowIn` 近似为 air/water；scheduleFluidTick no-op |
| 4 | minecraft:multiface_growth（1.21.6 类名确认 = MultifaceGrowthFeature） | **全量**：config 7 字段、方向表（ceiling→UP / floor→DOWN / walls→N,E,S,W）、Util.copyShuffled 洗牌、origin 直放失败后逐方向 search_range 扫描、spreadChance nextFloat 消费 | **faces 属性近似/降级**：glow_lichen 的 faces/down/up/north… 多状态在引擎 state=i32 下不可区分——统一放置 `minecraft:glow_lichen` 裸 id（声明，palette 对比时该方块按「存在性」对齐而非状态）；`grow()` 蔓生未实装（nextFloat 消费保留，保 RNG 流） |
| 5 | modifier minecraft:noise_threshold_count | parse（精确匹配）+ get_positions 实装；count 语义 `d<noise_level ? below : above` | **噪声取样保持简化 d=0.0**（同 NoiseBasedCount 先例 placement.rs:387）：placement 上下文无 foliage sampler。证据：① 1.20.1 全 workspace `*.json` grep `noise_threshold` = **0 引用**（本批实测），残差不影响当前数据集 parity；② Java 侧采样器 = `Biome.FOLIAGE_NOISE`（NoiseThresholdCountPlacementModifier.java:35），由 NoiseConfig 以 world seed 派生，接线需跨阶段传 NoiseSet——次批议题 |

**分发接入后预期 new unknown 集合变化**：batch0 实跑 unknown 残差 n=24 中，`minecraft:monster_room` / `minecraft:lake` / `minecraft:geode` / `minecraft:multiface_growth` / `mod:minecraft:noise_threshold_count`（若在残差清单）应**从 unknown 集合消失**；剩余残差应与批次 0 清单的其余项完全一致（无新增）。验收时用 WG_FEATURE_UNKNOWN_LOG=1 比对。

## 一、逐项对拍

### §1.1 minecraft:monster_room = DungeonFeature（Feature.java:70 注册名确认）

一手源：`world/gen/feature/DungeonFeature.java`（全文 134 行，config = DefaultFeatureConfig 空 config，JSON 实证 `monster_room.json` config={}）。

RNG 消费序声明（本 feature 独立 decorator seed，不影响其他 feature 流；序号 = 流内顺序）：
1. `j = nextInt(2)+2`（L39）
2. `o = nextInt(2)+2`（L44）
3. 预检双循环 L49-67：**0 消费**
4. r∉[1,5] → return false（**到此为止的 2 次消费已发生**）
5. 建造循环 L70-90：仅 t==-1 壳体、`blockState.isSolid() && !chest` 分支内 `nextInt(4)`（L79，每格 1 次，mossy 概率 1/4）
6. 箱子循环 L92-116：每 attempt **恒 2 次** `nextInt(j*2+1)` + `nextInt(o*2+1)`（L94/96，先抽后判 isAir——失败也消费）；命中 `x==1`（L107，四水平邻恰 1 实心）→ 放 chest + `setLootTable` 消费 `nextLong()`（LootableInventory.java:83-85 实证）→ `break` **仅跳出内层 t 循环，外层 s=1 轮继续消费**
7. spawner：`getMobSpawnerEntity(random)` = `Util.getRandom(4 元素数组)` = `nextInt(4)`（L131-133 + L25），实体类型引擎无实体层，**消费保留**

### §1.2 minecraft:lake = LakeFeature（Feature.java:75）

一手源：`world/gen/feature/LakeFeature.java`（163 行；config = record Config(fluid, barrier)，codec 字段 `fluid`/`barrier` L154-162）。JSON 用例 `lake_lava.json`（fluid=lava level=0，barrier=stone——level 属性引擎丢弃，声明）。

RNG 消费序：
1. `y <= bottomY+4` → false（**0 消费**，L30）
2. `i = nextInt(4)+4`（L35）
3. 每 blob **恒 6 次 nextDouble**（L38-43，d/e/f/g/h/k）
4. `fluid.get(random, pos)`（L60；simple→0，weighted→1）
5. 检验 pass L62-86：0 消费
6. 放置 pass L88-104：0 消费
7. `barrier.get(random, pos)`（L106）
8. barrier pass L108-131：`bl2 && (v<4 || nextInt(2)!=0)`（L120）——**短路**：bl2=false 或 v<4 时不消费
9. 水湖结冰 L133-144：0 消费
10. return true（L146，**恒 true**，与放置量无关）

### §1.3 minecraft:geode = GeodeFeature（Feature.java:111）

一手源：`GeodeFeature.java`（166 行）+ `GeodeFeatureConfig.java`（codec 13 字段，L10-33，orElse 默认值 L15-30）+ `GeodeLayerThicknessConfig.java`（filling/inner/middle/outer，默认 1.7/2.2/3.2/4.2）+ `GeodeCrackConfig.java`（1.0/2.0/2）+ `GeodeLayerConfig.java`（5 provider + inner_placements + cannot_replace + invalid_blocks）。

构造器命名陷阱核对（GeodeFeatureConfig.java:48-76）：形参名 `maxDistributionPoints/minPointOffset/maxGenOffset` 与实际赋值目标错位（`this.minGenOffset = maxDistributionPoints` L72）——按 **codec group 序**（L27-28：`min_gen_offset` 在前、`max_gen_offset` 在后）追踪：min_gen_offset JSON 值 → minGenOffset，max_gen_offset JSON 值 → maxGenOffset，**无实际换位 bug**，只是形参命名误导。Rust 按字段名直读。

RNG 消费序（主 decorator 流；噪声采样器是**独立流**不占位，见下）：
1. `distributionPoints.get(random)`（L40）
2. `l`（crack 尺寸）中 `random.nextDouble()/2`（L52，**无条件**）
3. `bl = nextFloat() < generateCrackChance`（L53）
4. 分布点循环 L56-69：每点恒 4 次（outerWallDistance.get ×3 L57-59 + pointOffset.get L68）；invalid 计数 > threshold → **return false（中途断流）**
5. crack 方向 `nextInt(4)`（L72，仅 bl 时）
6. 主迭代 L96-139（**BlockPos.iterate 序 = x 最内 / y 中 / z 最外**，BlockPos.java:485-495 `index % i` 实证）：仅 inner 层带（s∈[f,e)）消费——`nextFloat()<useAlternateLayer0Chance`（L123）+ inner provider（若 weighted）；`(!placementsRequireLayer0Alternate || bl2) && nextFloat()<usePotentialPlacementsChance`（L130，短路：前者 false 不消费）；其余层 provider.get（simple→0）
7. placements 循环 L143-162：每 placement 恒 `Util.getRandom(innerBlocks)` = `nextInt(n)`（L144），属性循环 0 消费

噪声采样器（L41-42）：`new ChunkRandom(new CheckedRandom(worldSeed))` + `DoublePerlinNoiseSampler.create(chunkRandom, -4, 1.0)`。CheckedRandom = LCG = 引擎 `RsRandom::Legacy(LegacyRandom::new(seed))` 同构；`-4, [1.0]` → `NoiseParameters{first_octave:-4, amplitudes:[1.0]}`。**需要 world seed → 本批给 OreFeatureContext 增 `world_seed` 字段接线**（全仓构造点仅 2 处：worldgen_handle.rs:1143 + bin-diag/b5b6_smoke.rs:119，grep 实证）。噪声项 r 在每点累加时**逐点 +r**（L102/106 `+ r` 在循环体内）。

层阈值（L48-52）：e=1/√filling，f=1/√(inner+d)，g=1/√(middle+d)，h=1/√(outer+d)，d=k/outerWallDistance.getMax()；`MathHelper.inverseSqrt` = 1/sqrt。else-if 带（外层守卫 `!(s<h)` 后）：crack(bl && t≥l && s<e) → s≥e filling → s≥f inner → s≥g middle → s≥h outer（1.7<2.2<3.2<4.2 ⇒ e>f>g>h，带宽正确，已数值核对）。

crack 点（L71-91）：`o=k*2+1`，四方向 (o,7,0)/(0,7,o)/(o,7,o)/(0,7,0) 各取 (7,5,1) 三点。

### §1.4 minecraft:multiface_growth = MultifaceGrowthFeature（Feature.java:64-66）

一手源：`MultifaceGrowthFeature.java`（85 行）+ `MultifaceGrowthFeatureConfig.java`（89 行；codec 7 字段 L20-36：block/search_range/can_place_on_floor/can_place_on_ceiling/can_place_on_wall/chance_of_spreading/can_be_placed_on，默认 10/false/false/false/0.5）。任务简报所称字段「places_block/…/predicate」与一手源**不符**，以一手源为准（详见 §四 错误记录 E-1）。

方向表构造（Config ctor L69-79）：ceiling→UP、floor→DOWN、walls→HORIZONTAL.forEach（N,E,S,W，同 fallen_tree facingArray 序，tree.rs:1121 已核对同源）。

RNG 消费序：
1. origin 非 air/water → false（0 消费，L25）
2. `shuffleDirections(random)`（L28）= Util.copyShuffled：**n-1 次 nextInt**（j 从 n 降到 2；B5/B6 同构，tree.rs:1060-1067 已对拍 Util.java:1184）
3. `generate(...)`（静态 L56-80）：沿 dirs 找 canPlaceOn 支撑面，命中 → 放置 + `nextFloat()<spreadChance`（L71，**无条件消费后再分支**）→ true
4. 失败 → 逐方向（L34-49）：每方向先 `shuffleDirections(random, opposite)` = **过滤后 m-1 次 nextInt**（L36/82-84），再 search_range 轮内循环
5. ⚠️ 一手源 L38-39 循环体内为 `mutable.set(blockPos, direction)`（**无距离参数 i**）——每轮检查同一位 `origin+dir`。按一手源逐字对拍（见 §五 idk-4 的 1.20.1 差异疑点）

### §1.5 modifier minecraft:noise_threshold_count

一手源：`world/gen/placementmodifier/NoiseThresholdCountPlacementModifier.java`（43 行）。codec = `noise_level`(double) + `below_noise`(int) + `above_noise`(int)（L11-18）；`getCount` = `Biome.FOLIAGE_NOISE.sample(x/200.0, z/200.0, false) < noiseLevel ? belowNoise : aboveNoise`（L34-37）。继承 AbstractCountPlacementModifier → 输出 count 个同点位、**0 额外 RNG**。

任务简报所称「noise_level min/max、half_cost」为一手源不存在的字段名（错误记录 E-1）。引擎接线结论：PlacementModifier::get_positions 只有 (ctx, random, pos)，无 NoiseSet/foliage sampler 访问面（grep placement.rs 全文无 noise 依赖；NoiseBasedCount 已有 d=0 简化先例）→ **保持 d=0.0 简化** + 残差登记（§〇 #5 证据）。

---
（以下为 patch，逐个补齐）

## 二、Patch 清单（old 串均先读 HEAD 现文核对，file:line 以当前工作区为准）

### Patch P1 — placement.rs：IntProvider::max_value（geode 用，Java IntProvider.getMax()）

文件 `worldgen-core/src/placement.rs`，定位 `impl IntProvider` 尾（parse fn 之后，L114-118 一带）。

old:
```rust
        } else {
            IntProvider::Constant(0)
        }
    }
}

// ===== PlacementModifier 基类 =====
```
new:
```rust
        } else {
            IntProvider::Constant(0)
        }
    }

    /// Java IntProvider.getMax()（geode d = k / outerWallDistance.getMax()，GeodeFeature.java:44）。
    /// batchA（mc-1216）：WeightedList 无明确上界 → 取 data 最大值（数据集 geode 只用 uniform，不可达分支）。
    pub fn max_value(&self) -> i32 {
        match self {
            IntProvider::Constant(a) => *a,
            IntProvider::Uniform(_, b) => *b,
            IntProvider::Trapezoid(_, b, _) => *b,
            IntProvider::BiasedToBottom(_, b) => *b,
            IntProvider::WeightedList(weighted, _) => weighted.iter().map(|(d, _)| *d).max().unwrap_or(0),
            IntProvider::Clamped(_, _, max) => *max,
        }
    }
}

// ===== PlacementModifier 基类 =====
```

### Patch P2 — placement.rs：枚举增 NoiseThresholdCount 变体

文件同上，`PlacementModifier` 枚举尾（L283-284）。

old:
```rust
    CountOnEveryLayer { count: IntProvider, water_id: i32, lava_id: i32, bedrock_id: i32 },
}
```
new:
```rust
    CountOnEveryLayer { count: IntProvider, water_id: i32, lava_id: i32, bedrock_id: i32 },
    /// batchA（mc-1216）：noise_threshold_count（NoiseThresholdCountPlacementModifier.java:11-18）
    /// count = (FOLIAGE 噪声 x/200,z/200) < noise_level ? below : above；当前噪声取样简化 0.0（§〇 #5）
    NoiseThresholdCount { noise_level: f64, below_noise: i32, above_noise: i32 },
}
```

### Patch P3 — placement.rs：get_positions 增分支

old（CountOnEveryLayer 分支尾 + parse fn 签名，L423-428）:
```rust
                out
            }
        }
    }

    pub fn parse(m: &JsonValue, blocks: &BlockRegistry) -> Option<PlacementModifier> {
```
new:
```rust
                out
            }
            PlacementModifier::NoiseThresholdCount { noise_level, below_noise, above_noise } => {
                // Java NoiseThresholdCountPlacementModifier.java:34-37：
                //   d = Biome.FOLIAGE_NOISE.sample(x/200.0, z/200.0, false); d < noise_level ? below : above
                // 残差（batchA #5）：placement 上下文无 foliage sampler（NoiseConfig 派生，未接线）→
                //   沿 NoiseBasedCount 先例（本文件 noise=0.0 注释）取 0.0。证据：1.20.1 数据集 0 JSON 引用。
                let d = 0.0f64;
                let n = if d < *noise_level { *below_noise } else { *above_noise };
                (0..n).map(|_| [x, y, z]).collect()
            }
        }
    }

    pub fn parse(m: &JsonValue, blocks: &BlockRegistry) -> Option<PlacementModifier> {
```

### Patch P4 — placement.rs：parse 增精确匹配分支（对齐 batch0 精确匹配风格）

old（L440-447 一带）:
```rust
                bedrock_id: blocks.id("minecraft:bedrock"),
            });
        } else if type_name.contains("rarity_filter") {
```
new:
```rust
                bedrock_id: blocks.id("minecraft:bedrock"),
            });
        } else if type_name == "minecraft:noise_threshold_count" {
            // NoiseThresholdCountPlacementModifier.java:11-18（一手源字段名；
            // 简报所称 "noise_level min/max、half_cost" 与 codec 不符 → 错误记录 E-1）
            return Some(PlacementModifier::NoiseThresholdCount {
                noise_level: m.get("noise_level").and_then(|x| x.as_f64()).unwrap_or(0.0),
                below_noise: m.get("below_noise").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                above_noise: m.get("above_noise").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            });
        } else if type_name.contains("rarity_filter") {
```

### Patch P5 — tree.rs：is_solid_ish 拆 id 版 + pub（dungeon/lake 复用）

文件 `worldgen-core/src/tree.rs`（L975-980）。

old:
```rust
fn is_solid_ish(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    let cur = ctx.block_at(x, y, z);
    if cur < 0 { return false; } // 世界不可读 → 保守拒绝（can_replace 同语义）
    if cur == ctx.blocks.id("minecraft:air") { return false; }
    !APPROX_NON_SOLID.iter().any(|n| ctx.blocks.id(n) == cur)
}
```
new:
```rust
/// batchA（mc-1216）：id 版 isSolid 近似（dungeon isSolid/isOf(CHEST) 判定、lake isSolid 判定复用）。
/// 语义同 is_solid_ish（APPROX_NON_SOLID 排除表 + 非空气）；不可读（-1）→ false 保守拒绝。
pub fn is_solid_id(ctx: &OreFeatureContext, id: i32) -> bool {
    if id < 0 { return false; }
    if id == ctx.blocks.id("minecraft:air") { return false; }
    !APPROX_NON_SOLID.iter().any(|n| ctx.blocks.id(n) == id)
}
pub fn is_solid_ish(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    is_solid_id(ctx, ctx.block_at(x, y, z))
}
```
（行为等价重构：原实现仅多包一层；call-site 不变。）

### Patch P6 — feature.rs：OreFeatureContext 增 world_seed（geode 噪声接线）

文件 `worldgen-core/src/feature.rs`（L155-158 一带）。

old:
```rust
    // c-A-min（260905-09）：任意点读钩子（含邻 chunk 地形列；与 FeaturePlacementContext.block_at 同源闭包）。
    // None = 旧语义（region_col_at / -1）。优先级高于 region_col_at。
    pub block_at_ext: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
}
```
new:
```rust
    // c-A-min（260905-09）：任意点读钩子（含邻 chunk 地形列；与 FeaturePlacementContext.block_at 同源闭包）。
    // None = 旧语义（region_col_at / -1）。优先级高于 region_col_at。
    pub block_at_ext: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
    // batchA（mc-1216）：世界 seed（geode 噪声采样器 = ChunkRandom(CheckedRandom(worldSeed))，
    // GeodeFeature.java:41）。构造点全仓 2 处：worldgen_handle.rs / bin-diag/b5b6_smoke.rs（同批 patch）。
### Patch P8 — feature_loader.rs：ConfiguredFeature 扩字段

文件 `worldgen-core/src/feature_loader.rs`（L64-65）。

old:
```rust
    pub fallen_config: Option<crate::tree::FallenTreeConfig>,         // minecraft:fallen_tree（B5，260908-15）
}
```
new:
```rust
    pub fallen_config: Option<crate::tree::FallenTreeConfig>,         // minecraft:fallen_tree（B5，260908-15）
    // —— batchA（mc-1216）：monster_room 走 DefaultFeatureConfig 空配置（无字段，分发直发）——
    pub lake_config: Option<crate::feature::LakeConfig>,              // minecraft:lake
    pub geode_config: Option<Box<crate::feature::GeodeConfig>>,       // minecraft:geode（大结构，Box 减负）
    pub multiface_config: Option<crate::feature::MultifaceGrowthConfig>, // minecraft:multiface_growth
}
```

### Patch P9 — feature_loader.rs：parse 初始化 None

old（L82-84）:
```rust
            simple_block_config: None,
            fallen_config: None,
        };
```
new:
```rust
            simple_block_config: None,
            fallen_config: None,
            lake_config: None,
            geode_config: None,
            multiface_config: None,
        };
```

### Patch P10 — feature_loader.rs：parse 分发精确匹配

old（fallen_tree 分支尾 → catch-all，L117-120）:
```rust
            if cf.fallen_config.is_none() {
                eprintln!("[feature-loader] fallen_tree config parse failed: {id}");
            }
        } else {
```
new:
```rust
            if cf.fallen_config.is_none() {
                eprintln!("[feature-loader] fallen_tree config parse failed: {id}");
            }
        } else if type_name == "minecraft:monster_room" {
            // DungeonFeature = DefaultFeatureConfig 空 config（Feature.java:70；monster_room.json config={}）
        } else if type_name == "minecraft:lake" {
            cf.lake_config = crate::feature::LakeConfig::parse(cfg, blocks);
            if cf.lake_config.is_none() {
                eprintln!("[feature-loader] lake config parse failed: {id}");
            }
        } else if type_name == "minecraft:geode" {
            cf.geode_config = crate::feature::GeodeConfig::parse(cfg, blocks).map(Box::new);
            if cf.geode_config.is_none() {
                eprintln!("[feature-loader] geode config parse failed: {id}");
            }
        } else if type_name == "minecraft:multiface_growth" {
            cf.multiface_config = crate::feature::MultifaceGrowthConfig::parse(cfg, blocks);
            if cf.multiface_config.is_none() {
                eprintln!("[feature-loader] multiface_growth config parse failed: {id}");
            }
        } else {
```

### Patch P11 — feature_loader.rs：generate 分发精确匹配

old（fallen_tree generate 分支尾 → catch-all，L509-511）:
```rust
            None => { eprintln!("[feature-loader] fallen_tree without config: {}", cf.id); false }
        }
    } else {
```
new:
```rust
            None => { eprintln!("[feature-loader] fallen_tree without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:monster_room" {
        crate::feature::MonsterRoomFeature.generate(octx, random, x, y, z)
    } else if cf.type_name == "minecraft:lake" {
        match &cf.lake_config {
            Some(lc) => crate::feature::LakeFeature.generate(octx, lc, random, biome_temp),
            None => { eprintln!("[feature-loader] lake without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:geode" {
        match &cf.geode_config {
            Some(gc) => crate::feature::GeodeConfig::generate(gc, octx, random, x, y, z),
            None => { eprintln!("[feature-loader] geode without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:multiface_growth" {
        match &cf.multiface_config {
            Some(mc) => crate::feature::MultifaceGrowthFeature.generate(octx, mc, random, x, y, z),
            None => { eprintln!("[feature-loader] multiface_growth without config: {}", cf.id); false }
        }
    } else {
```
（签名核对：generate_configured 现有形参 (cf, ctx, octx, random, x, y, z, biome_temp, biome_rainfall, cache)——lake 用 biome_temp；geode/multiface 用 octx/random/x/y/z，无额外需求。）

### Patch P12 — worldgen_handle.rs：octx world_seed 接线

文件 `worldgen-core/src/worldgen_handle.rs`（L1155 一带）。

old:
```rust
                    pending_cross: if ca_min { Some(&pending_cross_cb) } else { None },
                };
```
new:
```rust
                    pending_cross: if ca_min { Some(&pending_cross_cb) } else { None },
                    // batchA（mc-1216）：geode 噪声采样器种子（GeodeFeature.java:41）
                    world_seed: self.seed,
                };
```

### Patch P13 — bin-diag/b5b6_smoke.rs：构造点补字段（编译面）

文件 `worldgen-core/src/bin-diag/b5b6_smoke.rs`（L126-127）。

old:
```rust
            region_col_at: None, pending_cross: None, block_at_ext: None,
        };
```
new:
```rust
            region_col_at: None, pending_cross: None, block_at_ext: None,
            world_seed: 0, // 诊断桩：b5/b6 不涉 geode 噪声
        };
```

---

## 三、静态自检清单（未编译验证声明）

**状态：全部 patch 未编译、未运行**（沙箱无 shell）。静态自检如下：

- [x] 类型宽度：world_seed=i64（Java long）；网格下标 `(l*16+m)*8+n` 最大 (14*16+14)*8+6=1918 < 2048 usize 安全；`bls` 定长数组无堆分配。
- [x] 区间/rev 陷阱（compiler-idioms 发现 #10）：Java `for(t=3;t>=-1;t--)` → `(-1..=3).rev()`（含 -1，非 `(0..=4)`）；lake blob `1..15`/`1..7` 半开对齐 Java `l<15`/`n<7`；`for j in (2..=n).rev()` 洗牌与 B5/B6 同构；geode iterate `i..=j` 含端点（Java `<=`）。
- [x] wrapping 算术（发现 #18）：本批坐标均为 feature 原点 ± ≤32 的小偏移，无大坐标乘法；`k*2+1` k≤20 无溢出面；未引入新裸 `-`/`+` 于跨界路径（`x + s` 等沿用引擎既有同型写法——与 feature.rs 现状同口径，如需全面 wrapping 化属引擎级议题）。
- [x] 借用/所有权：`directions()`/`copy_shuffled` 值语义；`generate_multiface_at` 与 `GeodeConfig::generate` 均为关联函数/方法，`octx: &mut` 单一可变借用；`cannot`/`can_replace` 闭包捕获只读。P7 无新增 use（feature.rs 文件头已有 ChunkRandom/JsonValue/BlockRegistry/IntProvider——逐项核对 L12-16）。
- [x] panic 面：`next_int_bound(bound>0)`——monster_room `j*2+1≥3`/`o*2+1≥3`/`nextInt(4)`/`nextInt(2)`；lake `nextInt(4)`/`nextInt(2)`；geode `nextInt(4)`/IntProvider Uniform bound≥2；multiface shuffle 只在 len≥2 抽；inner_blocks 空时 `continue` 防御（Java nonEmptyList 保证非空）。
- [x] 构造点完备：OreFeatureContext 新字段全仓构造点 grep = 2 处（worldgen_handle.rs:1143、bin-diag/b5b6_smoke.rs:119），P12/P13 同批补齐。
- [x] 分发两侧清单一致（parse ↔ generate）：monster_room/lake/geode/multiface_growth 四项 + 既有项，均精确匹配；catch-all 告警 + record_unknown_type 保持。
- [x] 行为恒等重构：P5 is_solid_ish 重构为 is_solid_id 包装，原调用点行为不变。

## 四、错误记录（五段式）

### E-1 任务简报字段名与一手源不符（发现于对拍，非代码错误）

- **现象**：任务简报称 multiface_growth config 字段为「places_block/search_range/can_place_on_floor/…/predicate」、noise_threshold_count 参数为「noise_level min/max、half_cost」。
- **根因**：简报字段名系凭记忆/旧版混杂；1.21.6 一手源 codec 实际为 `block/search_range/can_place_on_floor/can_place_on_ceiling/can_place_on_wall/chance_of_spreading/can_be_placed_on`（MultifaceGrowthFeatureConfig.java:20-36）与 `noise_level/below_noise/above_noise`（NoiseThresholdCountPlacementModifier.java:11-18）。若按简报名写 parse，字段恒 miss → config 全默认值且难察觉。
- **定位**：逐行读两个一手源 codec 定义（本文 §1.4/§1.5 引 file:line）。
- **修复**：parse 按 codec 真名字段实现（P4/P7）；本文档字段名全部以一手源为准。
- **教训**：字段名类简报信息不进 parse 前核对不可用；「先读 codec 再写字段」应与「先读文件再写 patch」同级。

### E-2（预防性记录）geode 构造器形参错位假象

- **现象**：GeodeFeatureConfig.java:48-76 形参名 `maxDistributionPoints/minPointOffset/maxGenOffset` 与赋值目标 `minGenOffset/maxGenOffset` 语义错位，初读像「min/max 换位 bug」。
- **根因**：Mojang 复制粘贴形参名未随 codec 顺序更新；实际按 group 序追踪后 min_gen_offset/max_gen_offset 各归各位，无行为 bug。
- **定位**：按 RecordCodecBuilder group 序（L27-28）逐参对位。
- **修复**：Rust 按字段名直读，不复刻形参错位（若复刻反而引入 bug）。
- **教训**：yarn 源形参名≠codec 语义；多字段 config 对拍必须追 group 序而非形参名。

### E-3（预防性记录）multiface search_range 疑点

- **现象**：1.21.6 MultifaceGrowthFeature.java:38-39 循环体内 `mutable.set(blockPos, direction)` 无距离参数，search_range 次循环检查同一位；`i` 未使用。
- **根因**：idk（见 §五 idk-4）——不排除 1.20.1 为 `mutable.set(blockPos, direction, i)` 的三参扫描形态（引擎目标版本是 1.20.1）。
- **定位**：一手源逐行读 + 无法在沙箱内核对 1.20.1 源（mc_src_extract 仅 1.21.6）。
- **修复**：按交付基准（任务指定 1.21.6 一手源）逐字实现，疑点显式登记。
- **教训**：跨版本对拍时「源版本 vs 目标版本」差异点是独立 idk 类别，不许静默择一。

## 五、@anchor.idk 清单（随 patch 注释落码）

- **idk-1**（geode）：`isAir()` 近似为 `id==minecraft:air`；CAVE_AIR/VOID_AIR 计入 air 的差异未逐一验证（Java BlockState.isAir 对两 air 均真）。id 失配时 invalid 计数口径差 → 点位级残差可能。
- **idk-2**（geode/lake）：cannot_replace/invalid_blocks/lava_pool_stone 三 tag 数据驱动展开，fallback 硬编码为空集——tag JSON 缺失时谓词恒过（过放方向）而非恒拒；与 feature.rs expand_tag 现行 fallback 口径一致。
- **idk-3**（lake/dungeon）：`block_at` 越界读 -1 语义——lake 检验 pass 视为「非实心且 ≠fluid」→ 保守 abort（Javalake 读邻 chunk 实况）；dungeon `set_block_if_allowed` -1 → 跳过。ca_min（WG_CA_MIN=1）开启时由邻 chunk 地形列缓存缓解，但缓存无 feature 态（时序差）。验收实跑需覆盖 lake_lava_surface（近地表起点）用例。
- **idk-4**（multiface）：search_range 循环 1.21.6 源为同位重复检查（E-3）；若 1.20.1 实为三参距离扫描，本实现扫描面偏窄（残差方向 = 部分墙面未覆盖），需以 1.20.1 源/探针复核后定夺。
- **idk-5**（noise_threshold_count）：foliage 噪声未接线（d=0.0 简化）；证据 = 1.20.1 数据集 0 JSON 引用（§〇 #5）。

## 六、主会话后续动作建议

1. 应用 P1→P13（顺序无依赖，可任意）；`cargo build --offline -p worldgen --release` 编译门。
2. WG_FEATURE_UNKNOWN_LOG=1 实跑：验收 unknown 集合 == 批次 0 残差 − {本批 4 type + modifier}（见 §〇 预期）。
3. judge 审查（candidate 前 SHOULD）：重点核 §1.3 geode RNG 序与 P7 else-if 带宽。
4. 次批候选残差：foliage 噪声接线（noise_threshold_count）、multiface grow()、方块实体/属性位引擎级课题（既有 R-4 家族）。

### Patch P7 — feature.rs：新增 4 feature 实现（追加到文件尾）

文件 `worldgen-core/src/feature.rs`，追加到 `is_water_or_air` 之后（原文件尾，L666 后）。全部用全限定路径，不动 use 区。

追加内容（整段）：
```rust

// ===== batchA（mc-1216）：monster_room / lake / geode / multiface_growth =====
// 一手源：versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/feature/
//   DungeonFeature.java / LakeFeature.java / GeodeFeature.java(+Config/Layer/Crack/LayerThickness) /
//   MultifaceGrowthFeature.java(+Config)
// 对拍表 + RNG 消费序声明：.investigations/mc-1216-features-takeover/batchA-worker-delivery.md §一
// 已知限制（全族）：state=i32 无属性位（facing/waterlogged/level/faces 不可表达）；
// 方块实体/流体 tick/后处理调度 no-op；isSolid 系判定走 tree::is_solid_id 近似（B5/B6 同族）。
// （ChunkRandom / JsonValue / BlockRegistry / IntProvider 均已在 feature.rs 文件头 use，无需新增 use）

/// Java Feature.setBlockStateIf（Feature.java:144-148）：当前 state 不在 cannot tag 内才放。
/// 不可读（-1）→ 保守跳过（Java 读邻 chunk 实况，残差声明 §五 idk-3）。
fn set_block_if_allowed(ctx: &mut OreFeatureContext, x: i32, y: i32, z: i32, state: i32, cannot: &[i32]) {
    let cur = ctx.block_at(x, y, z);
    if cur >= 0 && !cannot.contains(&cur) {
        ctx.set_block(x, y, z, state);
    }
}

/// #minecraft:features_cannot_replace 展开（数据驱动，block_tags JSON；缺失时 fallback 硬编码为空 →
/// 谓词恒过 = 过放风险，与 feature.rs expand_tag fallback 口径一致，声明）。
fn features_cannot_replace_ids(blocks: &BlockRegistry) -> Vec<i32> {
    let mut ids = Vec::new();
    crate::feature::expand_tag(blocks, "minecraft:features_cannot_replace", &mut ids);
    ids
}

// ===== minecraft:monster_room（DungeonFeature.java，DefaultFeatureConfig 空 config）=====
pub struct MonsterRoomFeature;
impl MonsterRoomFeature {
    /// RNG 消费序（§1.1）：nextInt(2)+2 ×2 → 预检 0 → 壳体 t==-1 实心格 nextInt(4) →
    /// 箱子每 attempt 恒 2 次 nextInt(j*2+1)/nextInt(o*2+1)（命中再 nextLong loot seed，break 仅内层）
    /// → spawner nextInt(4)。r∉[1,5] 时 return false（前 2 次消费已发生）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let air = blocks.id("minecraft:air");
        let chest_id = blocks.id("minecraft:chest");
        let spawner_id = blocks.id("minecraft:spawner");
        let mossy_id = blocks.id("minecraft:mossy_cobblestone");
        let cobble_id = blocks.id("minecraft:cobblestone");
        let cannot = features_cannot_replace_ids(blocks);
        let j = random.next_int_bound(2) + 2;          // DungeonFeature.java:39
        let k = -j - 1;
        let l = j + 1;
        let o = random.next_int_bound(2) + 2;          // :44
        let p = -o - 1;
        let q = o + 1;
        // 预检（:49-67）：底/顶整面实心，否则 false；墙缘 t==0 双层 air 计数 r
        let mut r = 0;
        for s in k..=l {
            for t in -1..=4 {
                for u in p..=q {
                    let (bx, by, bz) = (x + s, y + t, z + u);
                    let bl = crate::tree::is_solid_id(ctx, ctx.block_at(bx, by, bz)); // :53 isSolid 近似
                    if t == -1 && !bl { return false; }                                   // :54-56
                    if t == 4 && !bl { return false; }                                    // :58-60
                    if (s == k || s == l || u == p || u == q) && t == 0
                        && ctx.block_at(bx, by, bz) == air && ctx.block_at(bx, by + 1, bz) == air {
                        r += 1;                                                           // :62-64
                    }
                }
            }
        }
        if !(r >= 1 && r <= 5) { return false; }                                          // :69/126-128
        // 建造（:70-90）：t 从 3 降到 -1（⚠️ 非 4——顶面 t=4 不进建造循环）
        for s in k..=l {
            for t in (-1..=3).rev() {
                for u in p..=q {
                    let (bx, by, bz) = (x + s, y + t, z + u);
                    let st = ctx.block_at(bx, by, bz);
                    if s == k || t == -1 || u == p || s == l || t == 4 || u == q {        // :75
                        if by >= ctx.min_y && !crate::tree::is_solid_id(ctx, ctx.block_at(bx, by - 1, bz)) {
                            set_block_if_allowed(ctx, bx, by, bz, air, &cannot);          // :76-77
                        } else if crate::tree::is_solid_id(ctx, st) && st != chest_id {   // :78
                            if t == -1 && random.next_int_bound(4) != 0 {                 // :79（仅此分支消费）
                                set_block_if_allowed(ctx, bx, by, bz, mossy_id, &cannot);
                            } else {
                                set_block_if_allowed(ctx, bx, by, bz, cobble_id, &cannot);
                            }
                        }
                    } else if st != chest_id && st != spawner_id {                        // :85
                        set_block_if_allowed(ctx, bx, by, bz, air, &cannot);              // :86
                    }
                }
            }
        }
        // 箱子（:92-116）：2×3 attempts；每 attempt 恒 2 次 nextInt；命中 = 位 air 且四水平邻恰 1 实心；
        // break 只跳内层（外层 s=1 轮继续消费）。chest facing/loot 表无实体层（loot seed 消费保留）。
        for _s in 0..2 {
            for _t in 0..3 {
                let ux = x + random.next_int_bound(j * 2 + 1) - j;                        // :94
                let wz = z + random.next_int_bound(o * 2 + 1) - o;                        // :96
                let (bx, by, bz) = (ux, y, wz);
                if ctx.block_at(bx, by, bz) == air {                                      // :98
                    let mut solid_sides = 0;
                    const H4: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
                    for dd in H4 {
                        if crate::tree::is_solid_id(ctx, ctx.block_at(bx + dd[0], by, bz + dd[1])) {
                            solid_sides += 1;                                             // :101-105
                        }
                    }
                    if solid_sides == 1 {                                                 // :107
                        set_block_if_allowed(ctx, bx, by, bz, chest_id, &cannot);         // :108-110（orientateChest facing 丢弃）
                        let _loot_seed = random.next_long();                              // :111 → LootableInventory.java:85 nextLong
                        break;
                    }
                }
            }
        }
        set_block_if_allowed(ctx, x, y, z, spawner_id, &cannot);                          // :118
        let _entity = random.next_int_bound(4);                                           // :131-133 MOB_SPAWNER_ENTITIES
        true                                                                              // :125
    }
}

// ===== minecraft:lake（LakeFeature.java + LakeFeature.Config record）=====
#[derive(Clone)]
pub struct LakeConfig {
    pub fluid: crate::tree::BlockStateProvider,
    pub barrier: crate::tree::BlockStateProvider,
}
impl LakeConfig {
    /// LakeFeature.java:154-162 codec：fluid + barrier 两个 BlockStateProvider（均 required）。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<LakeConfig> {
        let cfg = cfg?;
        Some(LakeConfig {
            fluid: crate::tree::BlockStateProvider::parse(cfg.get("fluid"), blocks)?,
            barrier: crate::tree::BlockStateProvider::parse(cfg.get("barrier"), blocks)?,
        })
    }
}

pub struct LakeFeature;
impl LakeFeature {
    /// biome_temp：当前 chunk biome 温度（canSetIce 近似，Java 按列 biome——声明 §〇 #2）。
    /// RNG 消费序（§1.2）：nextInt(4)+4 → 每 blob 6 nextDouble → fluid.get → barrier.get →
    /// barrier v≥4 且邻接命中时 nextInt(2)（短路）。恒 return true。
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &LakeConfig,
                    random: &mut ChunkRandom, biome_temp: f32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let x = ctx.origin_x; let y = ctx.origin_y; let z = ctx.origin_z;
        if y <= ctx.min_y + 4 { return false; }                                           // :30
        let by = y - 4;                                                                   // :33 down(4)
        let mut bls = [false; 2048];                                                      // :34 16*16*8
        let count = random.next_int_bound(4) + 4;                                         // :35
        for _ in 0..count {
            let d = random.next_double() * 6.0 + 3.0;                                     // :38
            let e = random.next_double() * 4.0 + 2.0;                                     // :39
            let f = random.next_double() * 6.0 + 3.0;                                     // :40
            let g = random.next_double() * (16.0 - d - 2.0) + 1.0 + d / 2.0;              // :41
            let h = random.next_double() * (8.0 - e - 4.0) + 2.0 + e / 2.0;               // :42
            let kk = random.next_double() * (16.0 - f - 2.0) + 1.0 + f / 2.0;             // :43
            for l in 1..15 {
                for m in 1..15 {
                    for n in 1..7 {                                                       // :45-47 半开上界 14/14/6
                        let oo = (l as f64 - g) / (d / 2.0);
                        let pp = (n as f64 - h) / (e / 2.0);
                        let qq = (m as f64 - kk) / (f / 2.0);
                        if oo * oo + pp * pp + qq * qq < 1.0 {
                            bls[(l * 16 + m) * 8 + n] = true;                             // :53
                        }
                    }
                }
            }
        }
        let fluid = config.fluid.get(random);                                             // :60
        let water_id = blocks.id("minecraft:water");
        let lava_id = blocks.id("minecraft:lava");
        let is_liquid = |id: i32| id == water_id || id == lava_id;                        // :76 isLiquid 近似
        let cannot = features_cannot_replace_ids(blocks);
        let can_replace = |ctx: &OreFeatureContext, bx: i32, byy: i32, bz: i32| -> bool { // :150-152
            let cur = ctx.block_at(bx, byy, bz);
            cur >= 0 && !cannot.contains(&cur)
        };
        // 检验 pass（:62-86）：u≥4 邻接位是液体 → false；u<4 邻接位非实心且 ≠fluid → false。
        // ⚠️ block_at 不可读（-1）→ 非实心且 ≠fluid → return false（保守 abort；ca_min 关闭时
        //   起点距边 <4 格会触发——声明 §五 idk-3）
        for s in 0..16 {
            for t in 0..16 {
                for u in 0..8 {
                    let bl = !bls[(s * 16 + t) * 8 + u] && (
                        (s < 15 && bls[((s + 1) * 16 + t) * 8 + u])
                        || (s > 0 && bls[((s - 1) * 16 + t) * 8 + u])
                        || (t < 15 && bls[(s * 16 + t + 1) * 8 + u])
                        || (t > 0 && bls[(s * 16 + (t - 1)) * 8 + u])
                        || (u < 7 && bls[(s * 16 + t) * 8 + u + 1])
                        || (u > 0 && bls[(s * 16 + t) * 8 + (u - 1)]));                   // :65-73
                    if bl {
                        let b2 = ctx.block_at(x + s, by + u, z + t);
                        if u >= 4 && is_liquid(b2) { return false; }                      // :76-78
                        if u < 4 && !crate::tree::is_solid_id(ctx, b2) && b2 != fluid { return false; } // :80-82（state 等值降级 id 等值）
                    }
                }
            }
        }
        // 放置 pass（:88-104）：u≥4 = CAVE_AIR，u<4 = fluid
        let cave_air = blocks.id("minecraft:cave_air");
        for s in 0..16 {
            for t in 0..16 {
                for u in 0..8 {
                    if bls[(s * 16 + t) * 8 + u] && can_replace(ctx, x + s, by + u, z + t) {
                        let st = if u >= 4 { cave_air } else { fluid };                   // :94-95
                        ctx.set_block(x + s, by + u, z + t, st);
                        // :97-98 scheduleBlockTick/markBlocksAboveForPostProcessing no-op（声明）
                    }
                }
            }
        }
        let barrier = config.barrier.get(random);                                         // :106
        if barrier != blocks.id("minecraft:air") {                                        // :107
            for t in 0..16 {
                for u in 0..16 {
                    for v in 0..8 {
                        let bl2 = !bls[(t * 16 + u) * 8 + v] && (
                            (t < 15 && bls[((t + 1) * 16 + u) * 8 + v])
                            || (t > 0 && bls[((t - 1) * 16 + u) * 8 + v])
                            || (u < 15 && bls[(t * 16 + u + 1) * 8 + v])
                            || (u > 0 && bls[(t * 16 + (u - 1)) * 8 + v])
                            || (v < 7 && bls[(t * 16 + u) * 8 + v + 1])
                            || (v > 0 && bls[(t * 16 + u) * 8 + (v - 1)]));               // :111-119
                        if bl2 && (v < 4 || random.next_int_bound(2) != 0) {              // :120（短路消费）
                            let b4 = ctx.block_at(x + t, by + v, z + u);
                            if crate::tree::is_solid_id(ctx, b4)
                                && !lava_pool_stone_cannot_replace(ctx.blocks).contains(&b4) { // :122
                                ctx.set_block(x + t, by + v, z + u, barrier);
                            }
                        }
                    }
                }
            }
        }
        // 水湖结冰（:133-144）：fluid ∈ WATER tag ≈ id==water；canSetIce ≈ biome_temp<0（声明）
        if fluid == water_id {
            for t in 0..16 {
                for u in 0..16 {
                    let (bx, byy, bz) = (x + t, by + 4, z + u);
                    if biome_temp < 0.0 && can_replace(ctx, bx, byy, bz) {
                        ctx.set_block(bx, byy, bz, blocks.id("minecraft:ice"));           // :140
                    }
                }
            }
        }
        true                                                                              // :146
    }
}

/// BlockTags.LAVA_POOL_STONE_CANNOT_REPLACE（数据驱动展开；缺失 fallback 空 → 恒放，声明）
fn lava_pool_stone_cannot_replace(blocks: &BlockRegistry) -> Vec<i32> {
    let mut ids = Vec::new();
    crate::feature::expand_tag(blocks, "minecraft:lava_pool_stone_cannot_replace", &mut ids);
    ids
}

// ===== minecraft:geode（GeodeFeature.java + 3 子配置）=====
#[derive(Clone)]
pub struct GeodeLayerThickness {
    pub filling: f64, pub inner: f64, pub middle: f64, pub outer: f64,
}
#[derive(Clone)]
pub struct GeodeCrackConfig {
    pub generate_crack_chance: f64, pub base_crack_size: f64, pub crack_point_offset: i32,
}
#[derive(Clone)]
pub struct GeodeLayerConfig {
    pub filling_provider: crate::tree::BlockStateProvider,
    pub inner_layer_provider: crate::tree::BlockStateProvider,
    pub alternate_inner_layer_provider: crate::tree::BlockStateProvider,
    pub middle_layer_provider: crate::tree::BlockStateProvider,
    pub outer_layer_provider: crate::tree::BlockStateProvider,
    pub inner_blocks: Vec<i32>,       // inner_placements（Properties 丢弃，声明）
    pub cannot_replace: Vec<i32>,     // #features_cannot_replace（parse 期展开缓存）
    pub invalid_blocks: Vec<i32>,     // #geode_invalid_blocks
}
#[derive(Clone)]
pub struct GeodeConfig {
    pub layer: GeodeLayerConfig,
    pub thickness: GeodeLayerThickness,
    pub crack: GeodeCrackConfig,
    pub use_potential_placements_chance: f64,
    pub use_alternate_layer0_chance: f64,
    pub placements_require_layer0_alternate: bool,
    pub outer_wall_distance: crate::placement::IntProvider,
    pub distribution_points: crate::placement::IntProvider,
    pub point_offset: crate::placement::IntProvider,
    pub min_gen_offset: i32,
    pub max_gen_offset: i32,
    pub noise_multiplier: f64,
    pub invalid_blocks_threshold: i32,
}

fn parse_state_name(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<i32> {
    let name = v?.get("Name").and_then(|n| n.as_str())?;
    Some(blocks.id(name))
}

impl GeodeConfig {
    /// GeodeFeatureConfig.java:10-33（orElse 默认值逐项对齐 L15-30）+ Thickness/Crack/Layer 三子 codec。
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<GeodeConfig> {
        let cfg = cfg?;
        let layers = cfg.get("blocks")?; // GeodeLayerConfig（fieldOf "blocks"，无默认）
        let tag_ids = |v: Option<&JsonValue>| -> Vec<i32> {
            let mut ids = Vec::new();
            if let Some(t) = v.and_then(|x| x.as_str()) {
                let t = t.strip_prefix('#').unwrap_or(t);
                crate::feature::expand_tag(blocks, t, &mut ids);
            }
            ids
        };
        let provider = |k: &str| crate::tree::BlockStateProvider::parse(layers.get(k), blocks);
        let layer_cfg = GeodeLayerConfig {
            filling_provider: provider("filling_provider")?,
            inner_layer_provider: provider("inner_layer_provider")?,
            alternate_inner_layer_provider: provider("alternate_inner_layer_provider")?,
            middle_layer_provider: provider("middle_layer_provider")?,
            outer_layer_provider: provider("outer_layer_provider")?,
            inner_blocks: {
                let mut ids = Vec::new();
                if let Some(arr) = layers.get("inner_placements").and_then(|v| v.as_array()) {
                    for it in arr { if let Some(id) = parse_state_name(Some(it), blocks) { ids.push(id); } }
                }
                ids
            },
            cannot_replace: tag_ids(layers.get("cannot_replace")),
            invalid_blocks: tag_ids(layers.get("invalid_blocks")),
        };
        let num = |v: Option<&JsonValue>, k: &str, dflt: f64| v.and_then(|x| x.get(k)).and_then(|x| x.as_f64()).unwrap_or(dflt);
        let thickness = GeodeLayerThickness {                                              // GeodeLayerThicknessConfig.java:10-13 orElse
            filling: num(cfg.get("layers"), "filling", 1.7),
            inner: num(cfg.get("layers"), "inner_layer", 2.2),
            middle: num(cfg.get("layers"), "middle_layer", 3.2),
            outer: num(cfg.get("layers"), "outer_layer", 4.2),
        };
        let crack_json = cfg.get("crack");
        let crack = GeodeCrackConfig {                                                     // GeodeCrackConfig.java:9-11 orElse
            generate_crack_chance: num(crack_json, "generate_crack_chance", 1.0),
            base_crack_size: num(crack_json, "base_crack_size", 2.0),
            crack_point_offset: num(crack_json, "crack_point_offset", 2.0) as i32,
        };
        let ip = |k: &str, dflt: crate::placement::IntProvider| -> crate::placement::IntProvider {
            cfg.get(k).map(|v| crate::placement::IntProvider::parse(Some(v))).unwrap_or(dflt)
        };
        Some(GeodeConfig {
            layer: layer_cfg,
            thickness,
            crack,
            use_potential_placements_chance: cfg.get("use_potential_placements_chance").and_then(|x| x.as_f64()).unwrap_or(0.35),
            use_alternate_layer0_chance: cfg.get("use_alternate_layer0_chance").and_then(|x| x.as_f64()).unwrap_or(0.0),
            placements_require_layer0_alternate: cfg.get("placements_require_layer0_alternate").and_then(|x| x.as_bool()).unwrap_or(true),
            outer_wall_distance: ip("outer_wall_distance", crate::placement::IntProvider::Uniform(4, 5)),
            distribution_points: ip("distribution_points", crate::placement::IntProvider::Uniform(3, 4)),
            point_offset: ip("point_offset", crate::placement::IntProvider::Uniform(1, 2)),
            min_gen_offset: cfg.get("min_gen_offset").and_then(|x| x.as_f64()).unwrap_or(-16.0) as i32,
            max_gen_offset: cfg.get("max_gen_offset").and_then(|x| x.as_f64()).unwrap_or(16.0) as i32,
            noise_multiplier: cfg.get("noise_multiplier").and_then(|x| x.as_f64()).unwrap_or(0.05),
            invalid_blocks_threshold: cfg.get("invalid_blocks_threshold").and_then(|x| x.as_f64()).unwrap_or(1.0) as i32, // codec required；缺省 1 + 口径声明
        })
    }

    /// RNG 消费序（§1.3）+ 噪声独立流（world_seed 接线，OreFeatureContext.world_seed）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let (i, j) = (self.min_gen_offset, self.max_gen_offset);
        let k = self.distribution_points.get(random);                                     // :40
        // :41-42 噪声采样器 = ChunkRandom(CheckedRandom(worldSeed)) + DoublePerlin(-4, [1.0])
        // 引擎 CheckedRandom ≡ RsRandom::Legacy(LegacyRandom)（同 LCG）；独立流不占 decorator RNG。
        let mut noise_rnd = crate::legacy_random::RsRandom::Legacy(
            crate::legacy_random::LegacyRandom::new(ctx.world_seed));
        let noise = crate::noise::DoublePerlinNoiseSampler::new(&mut noise_rnd,
            &crate::noise::NoiseParameters { first_octave: -4, amplitudes: vec![1.0] });
        let mut crack_points: Vec<[i32; 3]> = Vec::new();                                 // :43
        let d = k as f64 / self.outer_wall_distance.max_value() as f64;                   // :44
        let e = 1.0 / self.thickness.filling.sqrt();                                      // :48
        let f = 1.0 / (self.thickness.inner + d).sqrt();                                  // :49
        let g = 1.0 / (self.thickness.middle + d).sqrt();                                 // :50
        let h = 1.0 / (self.thickness.outer + d).sqrt();                                  // :51
        let l = 1.0 / (self.crack.base_crack_size + random.next_double() / 2.0
            + if k > 3 { d } else { 0.0 }).sqrt();                                        // :52（nextDouble 无条件）
        let bl = random.next_float() as f64 < self.crack.generate_crack_chance;           // :53
        let mut points: Vec<([i32; 3], i32)> = Vec::new();
        let mut invalid = 0;
        for _ in 0..k {
            let o = self.outer_wall_distance.get(random);                                 // :57
            let p = self.outer_wall_distance.get(random);                                 // :58
            let q = self.outer_wall_distance.get(random);                                 // :59
            let pos = [x + o, y + p, z + q];
            let st = ctx.block_at(pos[0], pos[1], pos[2]);
            if st == crate::blocks::AIR || self.layer.invalid_blocks.contains(&st) {      // :62（isAir ≈ id==AIR；cave_air 残差）
                invalid += 1;
                if invalid > self.invalid_blocks_threshold { return false; }              // :63-65
            }
            let off = self.point_offset.get(random);                                      // :68
            points.push((pos, off));
        }
        if bl {
            let n = random.next_int_bound(4);                                             // :72
            let o = k * 2 + 1;                                                            // :73
            let base: [[i32; 3]; 3] = match n {
                0 => [[o, 7, 0], [o, 5, 0], [o, 1, 0]],
                1 => [[0, 7, o], [0, 5, o], [0, 1, o]],
                2 => [[o, 7, o], [o, 5, o], [o, 1, o]],
                _ => [[0, 7, 0], [0, 5, 0], [0, 1, 0]],
            };
            for b in base { crack_points.push([x + b[0], y + b[1], z + b[2]]); }
        }
        let mut placements: Vec<[i32; 3]> = Vec::new();                                   // :93
        // BlockPos.iterate 序（BlockPos.java:485-495）：x 最内 / y 中 / z 最外
        for dz in i..=j {
            for dy in i..=j {
                for dx in i..=j {
                    let (px, py, pz) = (x + dx, y + dy, z + dz);
                    let r = noise.sample(px as f64, py as f64, pz as f64) * self.noise_multiplier; // :97
                    let mut s = 0.0f64;
                    for (pos, off) in &points {
                        let (ax, ay, az) = (px - pos[0], py - pos[1], pz - pos[2]);
                        s += 1.0 / ((ax * ax + ay * ay + az * az + *off) as f64).sqrt() + r;       // :102（+r 在循环内）
                    }
                    let mut t = 0.0f64;
                    for cp in &crack_points {
                        let (ax, ay, az) = (px - cp[0], py - cp[1], pz - cp[2]);
                        t += 1.0 / ((ax * ax + ay * ay + az * az + self.crack.crack_point_offset) as f64).sqrt() + r; // :106
                    }
                    if s < h { continue; }                                                         // :109 外层守卫
                    if bl && t >= l && s < e {                                            // :110 crack → air
                        set_block_if_allowed(ctx, px, py, pz, crate::blocks::AIR, &self.layer.cannot_replace);
                        // :113-119 scheduleFluidTick 6 邻 no-op（声明）
                    } else if s >= e {                                                    // :120 filling
                        let st = self.layer.filling_provider.get(random);
                        set_block_if_allowed(ctx, px, py, pz, st, &self.layer.cannot_replace);
                    } else if s >= f {                                                    // :122 inner（±alternate）
                        let use_alt = random.next_float() as f64 < self.use_alternate_layer0_chance; // :123
                        let st = if use_alt {
                            self.layer.alternate_inner_layer_provider.get(random)         // :125
                        } else {
                            self.layer.inner_layer_provider.get(random)                   // :127
                        };
                        set_block_if_allowed(ctx, px, py, pz, st, &self.layer.cannot_replace);
                        if (!self.placements_require_layer0_alternate || use_alt)
                            && random.next_float() as f64 < self.use_potential_placements_chance { // :130
                            placements.push([px, py, pz]);
                        }
                    } else if s >= g {                                                    // :133 middle
                        let st = self.layer.middle_layer_provider.get(random);
                        set_block_if_allowed(ctx, px, py, pz, st, &self.layer.cannot_replace);
                    } else {                                                              // :135 outer（s>=h 已由守卫保证）
                        let st = self.layer.outer_layer_provider.get(random);
                        set_block_if_allowed(ctx, px, py, pz, st, &self.layer.cannot_replace);
                    }
                }
            }
        }
        // placements（:143-162）：每 placement 恒 1 次 Util.getRandom(innerBlocks)（保 RNG 流，即使全不命中）
        let water_id = ctx.blocks.id("minecraft:water");
        for pos in &placements {
            if self.layer.inner_blocks.is_empty() { continue; }                            // Java innerBlocks nonEmptyList 保证；防御
            let st = self.layer.inner_blocks
                [random.next_int_bound(self.layer.inner_blocks.len() as i32) as usize];    // :144
            // Direction.values() 序（yarn）：DOWN, UP, NORTH, SOUTH, WEST, EAST
            const DIRS6: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];
            for dd in DIRS6 {
                // :147-149 FACING / :153-155 WATERLOGGED 属性丢弃（state=i32，声明）
                let (bx, by, bz) = (pos[0] + dd[0], pos[1] + dd[1], pos[2] + dd[2]);
                let st2 = ctx.block_at(bx, by, bz);
                // :157 BuddingAmethystBlock.canGrowIn ≈ air/water（近似声明）
                if st2 == crate::blocks::AIR || st2 == water_id {
                    set_block_if_allowed(ctx, bx, by, bz, st, &self.layer.cannot_replace);
                    break;                                                                 // :160
                }
            }
        }
        true                                                                              // :164
    }
}

// ===== minecraft:multiface_growth（MultifaceGrowthFeature.java + Config）=====
#[derive(Clone)]
pub struct MultifaceGrowthConfig {
    pub block: i32,                       // config.block（JSON 字符串或 {Name}）
    pub search_range: i32,                // 默认 10（1..=64 校验域）
    pub place_on_floor: bool,             // can_place_on_floor → DOWN
    pub place_on_ceiling: bool,           // can_place_on_ceiling → UP
    pub place_on_walls: bool,             // can_place_on_wall → N,E,S,W
    pub spread_chance: f32,               // chance_of_spreading，默认 0.5
    pub can_place_on: Vec<i32>,           // can_be_placed_on（字符串数组或 #tag）
}
impl MultifaceGrowthConfig {
    /// MultifaceGrowthFeatureConfig.java:20-36（orElse：block=glow_lichen, search_range=10,
    /// floor/ceiling/walls=false, spread=0.5；can_be_placed_on required）
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<MultifaceGrowthConfig> {
        let cfg = cfg?;
        let block_id = match cfg.get("block") {
            Some(v) => {
                if let Some(s) = v.as_str() { blocks.id(s) }
                else { parse_state_name(Some(v), blocks).unwrap_or_else(|| blocks.id("minecraft:glow_lichen")) }
            }
            None => blocks.id("minecraft:glow_lichen"),                                    // :26 orElse
        };
        let mut can_place_on = Vec::new();
        if let Some(arr) = cfg.get("can_be_placed_on").and_then(|v| v.as_array()) {
            for b in arr {
                if let Some(s) = b.as_str() {
                    if let Some(tag) = s.strip_prefix('#') {
                        crate::feature::expand_tag(blocks, tag, &mut can_place_on);
                    } else {
                        can_place_on.push(blocks.id(s));
                    }
                }
            }
        }
        Some(MultifaceGrowthConfig {
            block: block_id,
            search_range: cfg.get("search_range").and_then(|x| x.as_f64()).unwrap_or(10.0) as i32,
            place_on_floor: cfg.get("can_place_on_floor").and_then(|x| x.as_bool()).unwrap_or(false),
            place_on_ceiling: cfg.get("can_place_on_ceiling").and_then(|x| x.as_bool()).unwrap_or(false),
            place_on_walls: cfg.get("can_place_on_wall").and_then(|x| x.as_bool()).unwrap_or(false),
            spread_chance: cfg.get("chance_of_spreading").and_then(|x| x.as_f64()).unwrap_or(0.5) as f32,
            can_place_on,
        })
    }

    /// Config ctor（L69-79）：ceiling→UP、floor→DOWN、walls→HORIZONTAL N,E,S,W（序同 facingArray）
    fn directions(&self) -> Vec<[i32; 3]> {
        let mut v = Vec::new();
        if self.place_on_ceiling { v.push([0, 1, 0]); }
        if self.place_on_floor { v.push([0, -1, 0]); }
        if self.place_on_walls {
            for d in [[0, 0, -1], [1, 0, 0], [0, 0, 1], [-1, 0, 0]] { v.push(d); }
        }
        v
    }
}

/// Util.copyShuffled（Util.java:1184，同 tree.rs:1060-1067 对拍）：j 从 n 降到 2，k=nextInt(j)，swap(k, j-1)。
/// n≤1 零消费。
fn copy_shuffled(mut list: Vec<[i32; 3]>, random: &mut ChunkRandom) -> Vec<[i32; 3]> {
    let n = list.len() as i32;
    if n >= 2 {
        for j in (2..=n).rev() {
            let kk = random.next_int_bound(j) as usize;
            list.swap(kk, (j - 1) as usize);
        }
    }
    list
}

fn opposite_dir(d: [i32; 3]) -> [i32; 3] { [-d[0], -d[1], -d[2]] }

/// MultifaceGrowthFeature.generate（静态，L56-80）：沿 dirs 找 canPlaceOn 支撑 → 放置于 pos 本体
///（:69 setBlockState(pos, ...)——生长在 pos 的对应面上；faces 属性丢失，统一裸 id，声明）。
/// 命中后 nextFloat 恒消费（:71）。grow() 未实装（残差声明）。
fn generate_multiface_at(ctx: &mut OreFeatureContext, cfg: &MultifaceGrowthConfig,
                         random: &mut ChunkRandom, x: i32, y: i32, z: i32, dirs: &[[i32; 3]]) -> bool {
    for d in dirs {
        let support = ctx.block_at(x + d[0], y + d[1], z + d[2]);
        if support >= 0 && cfg.can_place_on.contains(&support) {                          // :63
            ctx.set_block(x, y, z, cfg.block);                                            // :69
            if random.next_float() < cfg.spread_chance {
                // :72 config.block.getGrower().grow(...) 未实装（nextFloat 消费已保留，残差 §〇 #4）
            }
            return true;
        }
    }
    false
}

pub struct MultifaceGrowthFeature;
impl MultifaceGrowthFeature {
    /// RNG 消费序（§1.4）：origin 检查 0 → shuffleDirections(n-1) → 静态 generate（命中 1 nextFloat）
    /// → 逐方向 shuffleDirections(m-1) → search_range 循环（1.21.6 L38-39 每轮检查同一位 origin+dir，
    /// 按一手源逐字对拍；1.20.1 差异疑点见 §五 idk-4）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, cfg: &MultifaceGrowthConfig,
                    random: &mut ChunkRandom, x: i32, y: i32, z: i32) -> bool {
        let (air, water) = (crate::blocks::AIR, ctx.blocks.id("minecraft:water"));
        let cur = ctx.block_at(x, y, z);
        if cur != air && cur != water { return false; }                                   // :25 isAirOrWater
        let all = cfg.directions();
        let list = copy_shuffled(all.clone(), random);                                    // :28
        if generate_multiface_at(ctx, cfg, random, x, y, z, &list) { return true; }       // :29
        for d in &list {                                                                  // :34
            let opp = opposite_dir(*d);                                                   // :36 opposite
            let filtered: Vec<[i32; 3]> = all.iter().copied().filter(|v| *v != opp).collect();
            let list2 = copy_shuffled(filtered, random);
            for _ in 0..cfg.search_range {                                                // :38
                let (bx, by, bz) = (x + d[0], y + d[1], z + d[2]);                        // :39 mutable.set(blockPos, direction)
                let st = ctx.block_at(bx, by, bz);
                if st != air && st != water && st != cfg.block { break; }                 // :41-43
                if generate_multiface_at(ctx, cfg, random, bx, by, bz, &list2) { return true; } // :45
            }
        }
        false                                                                             // :51
    }
}
```

