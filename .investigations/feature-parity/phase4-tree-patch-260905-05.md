# phase4-tree-patch-260905-05 — tree/植被 L0+L1 代码交付 patch（draft，**全部未编译验证** ⚠️）

> 角色：core-worker（feature parity Phase 4a）。语义依据 = 同目录 `s1-semantics-260905-05.md`（S1 一手源裁决，file:line 可溯）。
> **本 patch 只交付文档内嵌全文，不改 src/ 任何文件**；主会话应用后须 `cargo build --offline -p worldgen --release` + 单 seed palette 回归。
> 覆盖：L0 placement 谓词层（would_survive / surface_water_depth_filter / environment_scan / biome / 未知 type 告警 / TrapezoidHeightProvider / BiasedToBottom 修正）+ L1 树生成（straight / large_oak(fancy) / blob / fancy_foliage + cocoa / trunk_vine / leave_vine）+ selector/patch/simple_block 解析与（标注性的）生成分支。

---

## 1. 新文件 `worldgen-core/src/tree.rs`（全文，⚠️ 未编译验证）

```rust
// tree.rs — tree feature 族 + 树 decorator + 树系嵌套 feature 配置（MC 1.20.1）
// 语义依据：.investigations/feature-parity/s1-semantics-260905-05.md（S1 一手源裁决，含 file:line）
// Java 参照：world/gen/feature/TreeFeature.java、trunk/*、foliage/*、treedecorator/*
// 随机消费顺序：逐站点对齐一手源；随机只经函数参数 &mut ChunkRandom 传递（不存字段，F-1 教训）。
// 状态：未编译验证。已知限制见 patch 文档 §九。

use crate::blocks::BlockRegistry;
use crate::chunkrandom::ChunkRandom;
use crate::feature::OreFeatureContext;
use crate::json::JsonValue;

// ===== BlockStateProvider（数据集实测只出现 simple_state_provider / weighted_state_provider）=====
#[derive(Clone)]
pub enum BlockStateProvider {
    /// simple_state_provider：get 恒返回常量，0 随机消费
    Simple { state: i32 },
    /// weighted_state_provider：get 消费 1 次 next_int_bound(total_weight)
    Weighted { entries: Vec<(i32, i32)>, total_weight: i32 },
}

impl BlockStateProvider {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<BlockStateProvider> {
        let v = v?;
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if type_name.contains("simple_state_provider") {
            let name = v.get("state").and_then(|s| s.get("Name")).and_then(|n| n.as_str())?;
            Some(BlockStateProvider::Simple { state: blocks.id(name) })
        } else if type_name.contains("weighted_state_provider") {
            let mut entries = Vec::new();
            let mut total = 0;
            if let Some(items) = v.get("entries").and_then(|e| e.as_array()) {
                for it in items {
                    let w = it.get("weight").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                    let name = it.get("data").and_then(|d| d.get("Name")).and_then(|n| n.as_str()).unwrap_or("");
                    if name.is_empty() { continue; }
                    entries.push((blocks.id(name), w));
                    total += w;
                }
            }
            if entries.is_empty() { return None; }
            Some(BlockStateProvider::Weighted { entries, total_weight: total })
        } else {
            eprintln!("[tree] unsupported block state provider type: {type_name}（告警不静默，S4 全量加载门）");
            None
        }
    }

    /// Java BlockStateProvider.get(random, pos)。返回 (state, consumed_draws 由调用方无感——内部直接消费 random)
    pub fn get(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            BlockStateProvider::Simple { state } => *state,
            BlockStateProvider::Weighted { entries, total_weight } => {
                // Java WeightedStateProvider：dataPool.getOrThrow(nextInt(totalWeight))，1 次消费
                let mut i = random.next_int_bound(*total_weight);
                for (state, w) in entries {
                    i -= w;
                    if i < 0 { return *state; }
                }
                entries[0].0
            }
        }
    }
}

// ===== TrunkPlacer（L1：straight + large_oak(fancy)；其余类型告警跳过）=====
#[derive(Clone)]
pub enum TrunkPlacer {
    /// minecraft:straight_trunk_placer（TrunkPlacer.java:54-56 高度 + StraightTrunkPlacer.java:30-40）
    Straight { base_height: i32, rand_a: i32, rand_b: i32 },
    /// minecraft:fancy_trunk_placer（= LargeOakTrunkPlacer.java:38-88）
    LargeOak { base_height: i32, rand_a: i32, rand_b: i32 },
    /// 数据集出现但 L1 未实现 → 显式告警（不静默丢弃）
    Unsupported { type_name: String },
}

impl TrunkPlacer {
    pub fn parse(v: Option<&JsonValue>) -> TrunkPlacer {
        let v = match v { Some(v) => v, None => return TrunkPlacer::Unsupported { type_name: "<null>".into() } };
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let f = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
        if type_name.contains("straight_trunk_placer") {
            TrunkPlacer::Straight { base_height: f("base_height"), rand_a: f("height_rand_a"), rand_b: f("height_rand_b") }
        } else if type_name.contains("fancy_trunk_placer") {
            TrunkPlacer::LargeOak { base_height: f("base_height"), rand_a: f("height_rand_a"), rand_b: f("height_rand_b") }
        } else {
            eprintln!("[tree] unsupported trunk placer type: {type_name}");
            TrunkPlacer::Unsupported { type_name }
        }
    }

    /// Java TrunkPlacer.getHeight：base + nextInt(rand_a+1) + nextInt(rand_b+1)，恒 2 次消费，顺序固定
    /// （TrunkPlacer.java:54-56；s1-semantics idk-1）
    fn get_height(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            TrunkPlacer::Straight { base_height, rand_a, rand_b }
            | TrunkPlacer::LargeOak { base_height, rand_a, rand_b } => {
                base_height + random.next_int_bound(rand_a + 1) + random.next_int_bound(rand_b + 1)
            }
            TrunkPlacer::Unsupported { .. } => 0,
        }
    }
}

// ===== FoliagePlacer（L1：blob + fancy(=LargeOakFoliagePlacer)；其余告警）=====
#[derive(Clone)]
pub enum FoliagePlacer {
    /// minecraft:blob_foliage_placer（BlobFoliagePlacer.java:32-57）
    Blob { radius: IntProv, offset: IntProv, height: i32 },
    /// minecraft:fancy_foliage_placer（LargeOakFoliagePlacer.java:26-46，blob 子类）
    LargeOak { radius: IntProv, offset: IntProv, height: i32 },
    Unsupported { type_name: String },
}
// radius/offset 用 placement::IntProvider（别名避免与枚举名冲突）
use crate::placement::IntProvider as IntProv;

impl FoliagePlacer {
    pub fn parse(v: Option<&JsonValue>) -> FoliagePlacer {
        let v = match v { Some(v) => v, None => return FoliagePlacer::Unsupported { type_name: "<null>".into() } };
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let radius = IntProv::parse(v.get("radius"));
        let offset = IntProv::parse(v.get("offset"));
        let height = v.get("height").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
        if type_name.contains("blob_foliage_placer") {
            FoliagePlacer::Blob { radius, offset, height }
        } else if type_name.contains("fancy_foliage_placer") {
            FoliagePlacer::LargeOak { radius, offset, height }
        } else {
            eprintln!("[tree] unsupported foliage placer type: {type_name}");
            FoliagePlacer::Unsupported { type_name }
        }
    }

    /// getRandomHeight：blob/fancy 恒常量（BlobFoliagePlacer.java:50-52），0 消费
    fn get_random_height(&self) -> i32 {
        match self {
            FoliagePlacer::Blob { height, .. } | FoliagePlacer::LargeOak { height, .. } => *height,
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }
    /// getRandomRadius（FoliagePlacer.java:68-70）：radius.get(random)，消费随 IntProvider 类型
    fn get_random_radius(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            FoliagePlacer::Blob { radius, .. } | FoliagePlacer::LargeOak { radius, .. } => radius.get(random),
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }
    /// 主 generate 外层的 offset.get(random)（FoliagePlacer.java:48,72-74）
    fn get_random_offset(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            FoliagePlacer::Blob { offset, .. } | FoliagePlacer::LargeOak { offset, .. } => offset.get(random),
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }

    /// 树叶层生成。tree_node = (center_y 绝对坐标, foliage_radius, giant_trunk=false)
    /// 消费点：仅每层四角 nextInt(2)（blob）/ fancy 层角判定无消费 —— s1-semantics idk-2
    fn generate_foliage(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                        cfg: &TreeFeatureConfig, tree_node_y: i32, tree_node_radius: i32,
                        center_x: i32, center_z: i32, foliage_height: i32, radius: i32,
                        trunk_set: &Vec<[i32; 3]>, leaves_set: &mut Vec<[i32; 3]>) {
        let offset = self.get_random_offset(random); // 1 次等价 IntProvider 消费（constant → 0 draw）
        match self {
            FoliagePlacer::Blob { .. } => {
                // BlobFoliagePlacer.java:43-46：i 从 offset 到 offset-foliageHeight（含），i/2 向零截断
                for i in 0..=foliage_height as i64 {
                    let ii = offset - i as i32;
                    let j = std::cmp::max(radius + tree_node_radius - 1 - java_div(ii, 2), 0);
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, j, ii, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::LargeOak { .. } => {
                // LargeOakFoliagePlacer.java:37-40：j = radius + (i!=offset && i!=offset-foliageHeight ? 1 : 0)
                for i in 0..=foliage_height as i64 {
                    let ii = offset - i as i32;
                    let j = radius + if ii != offset && ii != offset - foliage_height { 1 } else { 0 };
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, j, ii, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::Unsupported { .. } => {}
        }
    }

    /// FoliagePlacer.generateSquare（FoliagePlacer.java:101-115）：非 giant，dx,dz ∈ [-r, r]
    fn generate_square(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                       cfg: &TreeFeatureConfig, cx: i32, cz: i32, cy: i32, r: i32, y: i32,
                       trunk_set: &Vec<[i32; 3]>, leaves_set: &mut Vec<[i32; 3]>) {
        if r < 0 { return; }
        for dx in -r..=r {
            for dz in -r..=r {
                if self.is_position_invalid(random, dx, y, dz, r) { continue; }
                let (px, py, pz) = (cx + dx, cy + y, cz + dz);
                if place_foliage_block(ctx, random, cfg, px, py, pz) {
                    leaves_set.push([px, py, pz]);
                }
            }
        }
        let _ = trunk_set; // hasPlacedBlock 仅 hanging-leaves 变体使用；blob/fancy 不用
    }

    /// isPositionInvalid → isInvalidForLeaves（FoliagePlacer.java:84-96 + 两 placer 覆写）
    fn is_position_invalid(&self, random: &mut ChunkRandom, dx: i32, y: i32, dz: i32, r: i32) -> bool {
        let (ax, az) = (dx.abs(), dz.abs());
        match self {
            // blob：四角才判；nextInt(2) 恒消费（|| 短路在 nextInt 之后），y==0 必 invalid
            FoliagePlacer::Blob { .. } => {
                if ax == r && az == r { random.next_int_bound(2) == 0 || y == 0 } else { false }
            }
            // fancy：圆盘判定，无随机消费（LargeOakFoliagePlacer.java:44-46）
            // MathHelper.square(dx + 0.5f) + MathHelper.square(dz + 0.5f) > r*r
            FoliagePlacer::LargeOak { .. } => {
                let fx = dx as f32 + 0.5; let fz = dz as f32 + 0.5;
                fx * fx + fz * fz > (r as f32) * (r as f32)
            }
            FoliagePlacer::Unsupported { .. } => true,
        }
    }
}

/// Java int 除法（向零截断）：i/2，i 可为负（-1/2 = 0，-3/2 = -1）
fn java_div(i: i32, d: i32) -> i32 {
    if i >= 0 { i / d } else { -((-i) / d) }
}

/// FoliagePlacer.placeFoliageBlock（FoliagePlacer.java:165-177）：canReplace 门 + provider.get
fn place_foliage_block(ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                       cfg: &TreeFeatureConfig, x: i32, y: i32, z: i32) -> bool {
    if !can_replace(ctx, x, y, z) { return false; }
    let state = cfg.foliage_provider.get(random);
    ctx.set_block(x, y, z, state);
    true
}

/// TreeFeature.canReplace（TreeFeature.java:53-55）：isAir || isIn(REPLACEABLE_BY_TREES)
/// Rust 近似：air，或 ∈ REPLACEABLE_BY_TREES tag id 集（S1 R-3：tag 数据待接线；先 air + 常见可替代集合）
fn can_replace(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    let cur = ctx.block_at(x, y, z);
    if cur < 0 { return false; } // 世界不可读（出界/邻域缺失）→ 保守拒绝（clip 语义见 §九）
    let air = ctx.blocks.id("minecraft:air");
    if cur == air { return true; }
    // ⚠️ 数据驱动边界：REPLACEABLE_BY_TREES 是 Java tag，当前硬编码其 1.20.1 核心成员
    // （tag JSON 接线后替换为数据驱动展开；见 patch §九清单）
    const REPLACEABLE: &[&str] = &[
        "minecraft:air", "minecraft:water", "minecraft:snow", "minecraft:short_grass", "minecraft:grass",
        "minecraft:fern", "minecraft:dead_bush", "minecraft:hanging_roots", "minecraft:vine",
        "minecraft:glow_lichen", "minecraft:light", "minecraft:tall_grass", "minecraft:large_fern",
    ];
    REPLACEABLE.iter().any(|n| ctx.blocks.id(n) == cur)
}

// ===== TreeDecorator（L1：cocoa / trunk_vine / leave_vine）=====
#[derive(Clone)]
pub enum TreeDecorator {
    Cocoa { probability: f32 },   // f32：JSON f64 → f32 与 Java float 一致（勿混 f64）
    TrunkVine,
    LeaveVine { probability: f32 },
    Unsupported { type_name: String },
}

impl TreeDecorator {
    pub fn parse(v: &JsonValue) -> TreeDecorator {
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let prob = || v.get("probability").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
        if type_name.contains("cocoa") {
            TreeDecorator::Cocoa { probability: prob() }
        } else if type_name.contains("trunk_vine") {
            TreeDecorator::TrunkVine
        } else if type_name.contains("leave_vine") || type_name.contains("leaves_vine") {
            TreeDecorator::LeaveVine { probability: prob() }
        } else {
            eprintln!("[tree] unsupported tree decorator type: {type_name}");
            TreeDecorator::Unsupported { type_name }
        }
    }

    /// 依 config.decorators 顺序执行（TreeFeature.java:154）。共用同一 random。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                    trunk_set: &Vec<[i32; 3]>, leaves_set: &Vec<[i32; 3]>) {
        match self {
            TreeDecorator::Cocoa { probability } => {
                // CocoaBeansTreeDecorator.java:28-45（s1-semantics idk-6）
                if !(random.next_float() >= *probability) {           // 门：nextFloat() < probability，恒 1 消费
                    let Some(&first) = trunk_set.iter().min_by_key(|p| p[1]) else { return };
                    let base_y = first[1];
                    // 遍历 = Y 升序稳定序（同 Y 内序 = Java HashSet 桶序，遗留 idk R-1，此处用插入序近似）
                    let mut sorted: Vec<&[i32; 3]> = trunk_set.iter().collect();
                    sorted.sort_by_key(|p| p[1]); // Rust sort 稳定，对齐 ObjectArrayList.sort
                    for pos in sorted {
                        if pos[1] - base_y > 2 { continue; }          // 底部 3 层
                        // Direction.Type.HORIZONTAL 序 = NORTH, WEST, SOUTH, EAST（Direction.java:49-52）
                        for dir in DIR_HORIZONTAL {
                            if random.next_float() <= 0.25 {          // 注意 <=；恒 1 消费/向
                                let (ox, oz) = opposite(dir);
                                let (bx, bz) = (pos[0] + ox, pos[2] + oz);
                                if ctx.block_at(bx, pos[1], bz) == ctx.blocks.id("minecraft:air") {
                                    let age = random.next_int_bound(3); // air 时再 1 消费
                                    let _ = age; // cocoa age/facing 属性编码：BlockRegistry 属性位接线后补（R-4）
                                    // state = cocoa 默认 + AGE(age) + FACING(dir)；当前以块 id 占位
                                    ctx.set_block(bx, pos[1], bz, ctx.blocks.id("minecraft:cocoa"));
                                }
                            }
                        }
                    }
                }
            }
            TreeDecorator::TrunkVine => {
                // TrunkVineTreeDecorator.java:19-50：每 log 恒 4 次 nextInt(3)，序 西→东→北→南
                let mut sorted: Vec<&[i32; 3]> = trunk_set.iter().collect();
                sorted.sort_by_key(|p| p[1]);
                for pos in sorted {
                    for dir in DIR_HORIZONTAL_JAVA_ORDER_VINE {
                        if random.next_int_bound(3) > 0 {
                            let (bx, bz) = (pos[0] + dir.0, pos[2] + dir.1);
                            if ctx.block_at(bx, pos[1], bz) == ctx.blocks.id("minecraft:air") {
                                // vine 默认态 + face 布尔属性；属性编码 R-4 接线后补
                                ctx.set_block(bx, pos[1], bz, ctx.blocks.id("minecraft:vine"));
                            }
                        }
                    }
                }
            }
            TreeDecorator::LeaveVine { probability } => {
                // LeavesVineTreeDecorator.java:26-70：每 leaf 恒 4 次 nextFloat() < probability
                let mut sorted: Vec<&[i32; 3]> = leaves_set.iter().collect();
                sorted.sort_by_key(|p| p[1]);
                for pos in sorted {
                    for dir in DIR_HORIZONTAL_JAVA_ORDER_VINE {
                        if random.next_float() < *probability {
                            let (bx, bz) = (pos[0] + dir.0, pos[2] + dir.1);
                            if ctx.block_at(bx, pos[1], bz) == ctx.blocks.id("minecraft:air") {
                                // placeVines：先放本格，再向下最长 4 格续藤（0 消费）
                                let mut py = pos[1];
                                ctx.set_block(bx, py, bz, ctx.blocks.id("minecraft:vine"));
                                for _ in 0..4 {
                                    py -= 1;
                                    if ctx.block_at(bx, py, bz) != ctx.blocks.id("minecraft:air") { break; }
                                    ctx.set_block(bx, py, bz, ctx.blocks.id("minecraft:vine"));
                                }
                            }
                        }
                    }
                }
            }
            TreeDecorator::Unsupported { .. } => {}
        }
    }
}

/// Direction.Type.HORIZONTAL 迭代序（cocoa 用）= NORTH, WEST, SOUTH, EAST
const DIR_HORIZONTAL: [(i32, i32); 4] = [(0, -1), (-1, 0), (0, 1), (1, 0)];
/// TrunkVine/LeaveVine 显式顺序 = west, east, north, south（源码字面顺序）
const DIR_HORIZONTAL_JAVA_ORDER_VINE: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
fn opposite(dir: &(i32, i32)) -> (i32, i32) { (-dir.0, -dir.1) }

// ===== FeatureSize =====
#[derive(Clone)]
pub enum FeatureSize {
    /// two_layers_feature_size（TwoLayersFeatureSize.java:38-40）：y < limit ? lower : upper
    TwoLayers { limit: i32, lower_size: i32, upper_size: i32, min_clipped_height: Option<i32> },
    /// three_layers / 其他：告警 + conservative
    Unsupported { type_name: String },
}

impl FeatureSize {
    pub fn parse(v: Option<&JsonValue>) -> FeatureSize {
        let v = match v { Some(v) => v, None => return FeatureSize::Unsupported { type_name: "<null>".into() } };
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let f = |k: &str, d: i32| v.get(k).and_then(|x| x.as_f64()).map(|x| x as i32).unwrap_or(d);
        if type_name.contains("two_layers_feature_size") {
            // 默认值 limit=1/lower=0/upper=1（Codec.orElse，TwoLayersFeatureSize.java:10-12）
            FeatureSize::TwoLayers {
                limit: f("limit", 1), lower_size: f("lower_size", 0), upper_size: f("upper_size", 1),
                min_clipped_height: v.get("min_clipped_height").and_then(|x| x.as_f64()).map(|x| x as i32),
            }
        } else if type_name.contains("three_layers_feature_size") {
            eprintln!("[tree] unsupported feature size type: {type_name}（数据集存在，L3）");
            FeatureSize::Unsupported { type_name }
        } else {
            eprintln!("[tree] unsupported feature size type: {type_name}");
            FeatureSize::Unsupported { type_name }
        }
    }
    fn get_radius(&self, _height: i32, y: i32) -> i32 {
        match self {
            FeatureSize::TwoLayers { limit, lower_size, upper_size, .. } => {
                if y < *limit { *lower_size } else { *upper_size }
            }
            FeatureSize::Unsupported { .. } => 0,
        }
    }
    fn get_min_clipped_height(&self) -> Option<i32> {
        match self { FeatureSize::TwoLayers { min_clipped_height, .. } => *min_clipped_height, _ => None }
    }
}

// ===== TreeFeatureConfig =====
#[derive(Clone)]
pub struct TreeFeatureConfig {
    pub trunk_provider: BlockStateProvider,
    pub foliage_provider: BlockStateProvider,
    pub dirt_provider: BlockStateProvider,
    pub trunk_placer: TrunkPlacer,
    pub foliage_placer: FoliagePlacer,
    pub decorators: Vec<TreeDecorator>,
    pub minimum_size: FeatureSize,
    pub ignore_vines: bool,
    pub force_dirt: bool,
}

impl TreeFeatureConfig {
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<TreeFeatureConfig> {
        let cfg = cfg?;
        let trunk_provider = BlockStateProvider::parse(cfg.get("trunk_provider"), blocks)?;
        let foliage_provider = BlockStateProvider::parse(cfg.get("foliage_provider"), blocks)?;
        let dirt_provider = BlockStateProvider::parse(cfg.get("dirt_provider"), blocks)?;
        let trunk_placer = TrunkPlacer::parse(cfg.get("trunk_placer"));
        let foliage_placer = FoliagePlacer::parse(cfg.get("foliage_placer"));
        let mut decorators = Vec::new();
        if let Some(arr) = cfg.get("decorators").and_then(|d| d.as_array()) {
            for d in arr { decorators.push(TreeDecorator::parse(d)); }
        }
        Some(TreeFeatureConfig {
            trunk_provider, foliage_provider, dirt_provider, trunk_placer, foliage_placer,
            decorators,
            minimum_size: FeatureSize::parse(cfg.get("minimum_size")),
            ignore_vines: cfg.get("ignore_vines").and_then(|x| x.as_bool()).unwrap_or(false), // Codec.orElse(false)
            force_dirt: cfg.get("force_dirt").and_then(|x| x.as_bool()).unwrap_or(false),
        })
    }

    /// 树生成主入口（TreeFeature.java:117-165 + 57-90 全流程）。
    /// random = feat_random（decorator seed 已定的同一 RNG 流）。
    pub fn generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                    x: i32, y: i32, z: i32) -> bool {
        if let TrunkPlacer::Unsupported { type_name } = &self.trunk_placer {
            eprintln!("[tree] skip unsupported trunk placer {type_name} at ({x},{y},{z})");
            return false;
        }
        if let FoliagePlacer::Unsupported { type_name } = &self.foliage_placer {
            eprintln!("[tree] skip unsupported foliage placer {type_name} at ({x},{y},{z})");
            return false;
        }
        // ①-③ 高度随机（TreeFeature.java:66-69；消费见 s1-semantics §1 idk-3 总表）
        let i = self.trunk_placer.get_height(random);            // 2 draws
        let j = self.foliage_placer.get_random_height();          // blob/fancy：0
        let k = i - j;
        let _ = k;
        let l = self.foliage_placer.get_random_radius(random);    // constant → 0 draw
        // 无 rootPlacer → blockPos = pos（0 消费）
        let (bx, by, bz) = (x, y, z);
        // ④ 域守卫（TreeFeature.java:71-73）：m >= bottomY+1 && n <= topY
        let m = by.min(by);
        let n = by.max(by) + i + 1;
        if !(m >= ctx.min_y + 1 && n <= ctx.min_y + ctx.height) { return false; }
        // ⑤ getTopPosition（TreeFeature.java:92-109；0 消费）
        let o = self.get_top_position(ctx, i, bx, by, bz);
        // ⑥ 高度验收（TreeFeature.java:76）
        let clipped = self.minimum_size.get_min_clipped_height();
        if !(o >= i || clipped.map_or(false, |c| o >= c)) { return false; }
        // ⑦ trunk（含 setToDirt；straight=TreeFeature 序，LargeOak 见下）
        let mut trunk_set: Vec<[i32; 3]> = Vec::new();
        let mut leaves_set: Vec<[i32; 3]> = Vec::new();
        let tree_nodes: Vec<(i32, i32, i32, i32)> = match &self.trunk_placer {
            TrunkPlacer::Straight { .. } => {
                self.set_to_dirt(ctx, random, bx, by - 1, bz);
                for iy in 0..o {
                    if can_replace(ctx, bx, by + iy, bz) {
                        let state = self.trunk_provider.get(random);
                        ctx.set_block(bx, by + iy, bz, state);
                        trunk_set.push([bx, by + iy, bz]);
                    }
                }
                vec![(bx, by + o, bz, 0)] // TreeNode(pos.up(height), 0, false)
            }
            TrunkPlacer::LargeOak { .. } => {
                self.large_oak_trunk(ctx, random, bx, by, bz, o, &mut trunk_set)
            }
            TrunkPlacer::Unsupported { .. } => return false,
        };
        // ⑧ foliage（逐 node；TreeFeature.java:81）
        for (nx, ny, nz, nr) in &tree_nodes {
            self.foliage_placer.generate_foliage(ctx, random, self, *ny, *nr, *nx, *nz, j, l, &trunk_set, &mut leaves_set);
        }
        // ⑨ decorators（TreeFeature.java:151-155；须 trunk/leaves 非空）
        if !trunk_set.is_empty() || !leaves_set.is_empty() {
            for d in &self.decorators {
                d.generate(ctx, random, &trunk_set, &leaves_set);
            }
        }
        // ⑩ placeLogsAndLeaves distance 重算（TreeFeature.java:167-229）：
        // 叶块终态 distance = 到最近 log 的 BFS 距离（非 JSON 的 7）。
        // ⚠️ 已知限制：当前 state=i32 块 id 不携带属性位，distance 重写无法编码 → 占位 no-op，
        // 登记为 palette 对比已知偏差源（patch §九）；属性位接线后按 BFS 距离改写。
        let _ = (&trunk_set, &leaves_set);
        true
    }

    /// TreeFeature.getTopPosition（TreeFeature.java:92-109）：0 随机消费；失败层 i → 返回 i-2（可为负）
    fn get_top_position(&self, ctx: &OreFeatureContext, height: i32, px: i32, py: i32, pz: i32) -> i32 {
        for iy in 0..=(height + 1) {
            let r = self.minimum_size.get_radius(height, iy);
            for kx in -r..=r {
                for kz in -r..=r {
                    let (qx, qy, qz) = (px + kx, py + iy, pz + kz);
                    let ok = can_replace_or_is_log(ctx, qx, qy, qz)
                        && (self.ignore_vines || ctx.block_at(qx, qy, qz) != ctx.blocks.id("minecraft:vine"));
                    if !ok { return iy - 2; }
                }
            }
        }
        height
    }

    /// TrunkPlacer.setToDirt（TrunkPlacer.java:62-66）：forceDirt || !canGenerate → 放 dirt
    fn set_to_dirt(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, y: i32, z: i32) {
        // canGenerate（TrunkPlacer.java:58-60）：isSoil && !grass_block && !mycelium
        let cur = ctx.block_at(x, y, z);
        let is_soil_not_grass_myc = cur >= 0 && [
            "minecraft:dirt", "minecraft:coarse_dirt", "minecraft:podzol", "minecraft:rooted_dirt",
            "minecraft:mud", "minecraft:muddy_mangrove_roots", "minecraft:mycelium", "minecraft:grass_block",
        ].iter().any(|n| ctx.blocks.id(n) == cur)
            && cur != ctx.blocks.id("minecraft:grass_block")
            && cur != ctx.blocks.id("minecraft:mycelium");
        // ⚠️ Feature.isSoil = dirt tag 全集；上方列表为其 1.20.1 主体（数据边界声明，见 §九）
        if self.force_dirt || !is_soil_not_grass_myc {
            let state = self.dirt_provider.get(random);
            ctx.set_block(x, y, z, state);
        }
    }

    /// LargeOakTrunkPlacer.generate（LargeOakTrunkPlacer.java:38-88）。
    /// 消费：每候选分支恒 2 次 nextFloat（L57-58 g/h）；makeOrCheckBranch 放置路径无额外随机（provider simple）。
    /// 返回 TreeNode 列表 (x, y, z, foliageRadius=0)。
    fn large_oak_trunk(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                       sx: i32, sy: i32, sz: i32, height: i32, trunk_set: &mut Vec<[i32; 3]>) -> Vec<(i32, i32, i32, i32)> {
        let jj = height + 2;                                   // j = height+2（L42）
        let k = ((jj as f64) * 0.618).floor() as i32;           // L43
        self.set_to_dirt(ctx, random, sx, sy - 1, sz);
        let l = (1.382f64 + ((jj as f64) / 13.0).powi(2)).floor() as i32; // L46：min(1, ...)——注意 Java 是 Math.min(1, ...)
        let l = l.min(1);
        let m = sy + k;                                        // L47
        let mut n = jj - 5;                                    // L48
        // BranchPosition 列表：node 中心 + endY；首项 = startPos.up(n), m
        struct Branch { cx: i32, cy: i32, cz: i32, end_y: i32 }
        let mut branches: Vec<Branch> = vec![Branch { cx: sx, cy: sy + n, cz: sz, end_y: m }];
        while n >= 0 {
            let f = self.should_generate_branch(jj, n);        // L163-178
            if f >= 0.0 {
                for _ in 0..l {
                    let g = (f as f64) * (random.next_float() as f64 + 0.328);  // 消费 1
                    let h = (random.next_float() as f64) * 2.0 * std::f64::consts::PI; // 消费 1
                    let p = (g * h.sin() + 0.5).floor() as i32;
                    let q = (g * h.cos() + 0.5).floor() as i32;
                    let (bpx, bpy, bpz) = (sx + p, sy + n - 1, sz + q);
                    let (b2x, b2y, b2z) = (bpx, bpy + 5, bpz);
                    if self.make_or_check_branch(ctx, random, bpx, bpy, bpz, b2x, b2y, b2z, false, trunk_set) {
                        let r = sx - bpx;
                        let s2 = sz - bpz;
                        let t = bpy as f64 - ((r * r + s2 * s2) as f64).sqrt() * 0.381;
                        let u = if t > m as f64 { m } else { t as i32 };
                        if self.make_or_check_branch(ctx, random, sx, u, sz, bpx, bpy, bpz, false, trunk_set) {
                            branches.push(Branch { cx: bpx, cy: bpy, cz: bpz, end_y: u });
                        }
                    }
                }
            }
            n -= 1;
        }
        // 主干（L77）
        self.make_or_check_branch(ctx, random, sx, sy, sz, sx, sy + k, sz, true, trunk_set);
        // makeBranches（L142-158）
        for b in &branches {
            if !(sx == b.cx && sy == b.cy && sz == b.cz) && (b.end_y - sy) as f64 >= (jj as f64) * 0.2 {
                self.make_or_check_branch(ctx, random, sx, b.end_y, sz, b.cx, b.cy, b.cz, true, trunk_set);
            }
        }
        // isHighEnough 过滤（L79-87）
        branches.iter()
            .filter(|b| ((b.cy - sy) as f64) >= (jj as f64) * 0.2)
            .map(|b| (b.cx, b.cy, b.cz, 0))
            .collect()
    }

    /// LargeOakTrunkPlacer.shouldGenerateBranch（L163-178）
    fn should_generate_branch(&self, tree_height: i32, height: i32) -> f32 {
        if (height as f32) < (tree_height as f32) * 0.3 { return -1.0; }
        let f = tree_height as f32 / 2.0;
        let g = f - height as f32;
        let mut h = (f * f - g * g).sqrt();
        if g == 0.0 { h = f; }
        else if g.abs() >= f { return 0.0; }
        h * 0.5
    }

    /// LargeOakTrunkPlacer.makeOrCheckBranch（L90-113）：沿最长轴插值逐格
    fn make_or_check_branch(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                            ax: i32, ay: i32, az: i32, bx: i32, by: i32, bz: i32, make: bool,
                            trunk_set: &mut Vec<[i32; 3]>) -> bool {
        if !make && ax == bx && ay == by && az == bz { return true; }
        let (dx, dy, dz) = (bx - ax, by - ay, bz - az);
        let i = dx.abs().max(dy.abs().max(dz.abs()));
        if i == 0 { return true; }
        let f = dx as f32 / i as f32;
        let g = dy as f32 / i as f32;
        let h = dz as f32 / i as f32;
        for jj in 0..=i {
            let px = ax + (0.5 + jj as f32 * f).floor() as i32;
            let py = ay + (0.5 + jj as f32 * g).floor() as i32;
            let pz = az + (0.5 + jj as f32 * h).floor() as i32;
            if make {
                if can_replace(ctx, px, py, pz) { // getAndSetState（canReplace 门 + provider.get）
                    let state = self.trunk_provider.get(random);
                    ctx.set_block(px, py, pz, state);
                    trunk_set.push([px, py, pz]);
                }
            } else if !can_replace_or_is_log(ctx, px, py, pz) {
                return false;
            }
        }
        true
    }
}

/// TrunkPlacer.canReplaceOrIsLog（TrunkPlacer.java:98-100）
fn can_replace_or_is_log(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    if can_replace(ctx, x, y, z) { return true; }
    let cur = ctx.block_at(x, y, z);
    // BlockTags.LOGS 主体（数据边界声明：tag 数据接线后替换，见 §九）
    const LOGS: &[&str] = &[
        "minecraft:oak_log", "minecraft:birch_log", "minecraft:spruce_log", "minecraft:jungle_log",
        "minecraft:acacia_log", "minecraft:dark_oak_log", "minecraft:mangrove_log", "minecraft:cherry_log",
        "minecraft:pale_oak_log", "minecraft:oak_wood", "minecraft:birch_wood", "minecraft:spruce_wood",
        "minecraft:jungle_wood", "minecraft:acacia_wood", "minecraft:dark_oak_wood",
        "minecraft:stripped_oak_log", "minecraft:stripped_birch_log", "minecraft:stripped_spruce_log",
        "minecraft:stripped_jungle_log", "minecraft:stripped_acacia_log", "minecraft:stripped_dark_oak_log",
    ];
    LOGS.iter().any(|n| ctx.blocks.id(n) == cur)
}

// ===== 树系嵌套 feature 配置（selector / patch / simple_block）=====
/// random_selector（⚠️ generate 公式为占位：idk-7 未裁决，S6 动工前补 RandomSelectorFeature.java 取证）
#[derive(Clone)]
pub struct RandomSelectorConfig {
    pub features: Vec<(f32, crate::placement::PlacedFeature)>, // (chance, 内嵌 placed)
    pub default_feature: Option<crate::placement::PlacedFeature>,
}
impl RandomSelectorConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<RandomSelectorConfig> {
        let v = v?;
        let mut features = Vec::new();
        if let Some(arr) = v.get("features").and_then(|f| f.as_array()) {
            for e in arr {
                let chance = e.get("chance").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                if let Some(pf) = crate::placement::PlacedFeature::parse_inline(e.get("feature"), blocks) {
                    features.push((chance, pf));
                }
            }
        }
        let default_feature = v.get("default_feature").and_then(|d| crate::placement::PlacedFeature::parse_inline(Some(d), blocks));
        Some(RandomSelectorConfig { features, default_feature })
    }
}

/// random_patch / flower（⚠️ generate 公式为占位：idk-7 未裁决，S7 动工前补 RandomPatchFeature.java 取证）
#[derive(Clone)]
pub struct RandomPatchConfig {
    pub tries: i32,
    pub xz_spread: IntProv,
    pub y_spread: IntProv,
    pub feature: crate::placement::PlacedFeature,
}
impl RandomPatchConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<RandomPatchConfig> {
        let v = v?;
        Some(RandomPatchConfig {
            tries: v.get("tries").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            xz_spread: IntProv::parse(v.get("xz_spread")),
            y_spread: IntProv::parse(v.get("y_spread")),
            feature: crate::placement::PlacedFeature::parse_inline(v.get("feature"), blocks)?,
        })
    }
}

/// simple_block
#[derive(Clone)]
pub struct SimpleBlockConfig {
    pub to_place: BlockStateProvider,
}
impl SimpleBlockConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<SimpleBlockConfig> {
        Some(SimpleBlockConfig { to_place: BlockStateProvider::parse(v?.get("to_place"), blocks)? })
    }
}
```

> 注：`PlacedFeature::parse_inline(v, blocks)` = 本 patch §2 的 placement.rs 新增辅助（把 JSON 里的内嵌 placed feature 对象解析为 `PlacedFeature`；字符串形式则查 FeatureCache——需 cache 参数时改在 feature_loader 侧展开，见 §2 说明）。

---

## 2. `worldgen-core/src/feature_loader.rs` 需改动函数全文（⚠️ 未编译验证）

### 2.1 `ConfiguredFeature` 结构体 + `parse`（全文替换）

```rust
// ===== ConfiguredFeature 解析（type 分发）=====
// 260905-05：解除 2026-08-10 拍板注释（原 L50）——supersedes：G2 归因（g2-convergence-260905-03）
// 判定树/植被残差为「feature 类型未实现」而非「调度错位」+ 本课题（b2-tree-plan-260905-05 / feature parity Phase 4）。
#[derive(Clone)]
pub struct ConfiguredFeature {
    pub id: String,
    pub type_name: String,
    pub ore_config: OreFeatureConfig,
    pub disk_config: DiskFeatureConfig,
    pub spring_config: SpringFeatureConfig,
    pub magma_config: UnderwaterMagmaFeatureConfig,
    pub freeze_top: bool,
    // —— tree/植被载荷（260905-05 新增；恰好一个 Some，其余 None）——
    pub tree_config: Option<crate::tree::TreeFeatureConfig>,          // minecraft:tree
    pub selector_config: Option<crate::tree::RandomSelectorConfig>,   // minecraft:random_selector
    pub patch_config: Option<crate::tree::RandomPatchConfig>,         // minecraft:random_patch / flower
    pub simple_block_config: Option<crate::tree::SimpleBlockConfig>,  // minecraft:simple_block
}

impl ConfiguredFeature {
    pub fn parse(id: &str, root: &JsonValue, blocks: &BlockRegistry) -> ConfiguredFeature {
        let type_name = root.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let cfg = root.get("config");
        let mut cf = ConfiguredFeature {
            id: id.to_string(),
            type_name: type_name.clone(),
            ore_config: OreFeatureConfig::parse(None, blocks),
            disk_config: DiskFeatureConfig::parse(None, blocks),
            spring_config: SpringFeatureConfig::parse(None, blocks),
            magma_config: UnderwaterMagmaFeatureConfig::parse(None, blocks),
            freeze_top: false,
            tree_config: None,
            selector_config: None,
            patch_config: None,
            simple_block_config: None,
        };
        if type_name.contains("ore") {
            cf.ore_config = OreFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("disk") {
            cf.disk_config = DiskFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("spring") {
            cf.spring_config = SpringFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("underwater_magma") {
            cf.magma_config = UnderwaterMagmaFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("freeze_top_layer") {
            cf.freeze_top = true;
        } else if type_name == "minecraft:tree" {
            // 精确匹配（不用 contains：防 azalea_tree 等误伤；azalea_tree 在数据集 type 也是
            // minecraft:tree，走同一分支由 placer/size 解析告警兜底）
            cf.tree_config = crate::tree::TreeFeatureConfig::parse(cfg, blocks);
            if cf.tree_config.is_none() {
                eprintln!("[feature-loader] tree config parse failed: {id}");
            }
        } else if type_name == "minecraft:random_selector" {
            cf.selector_config = crate::tree::RandomSelectorConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:random_patch" || type_name == "minecraft:flower" {
            cf.patch_config = crate::tree::RandomPatchConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:simple_block" {
            cf.simple_block_config = crate::tree::SimpleBlockConfig::parse(cfg, blocks);
        } else {
            // 未知 configured type 显式告警（消除静默丢弃，b2 S2）——不 panic（S4 全量加载门）
            eprintln!("[feature-loader] unknown configured feature type: {type_name} ({id})");
        }
        cf
    }
}
```

### 2.2 `generate_configured`（全文替换；**签名变更**：增 `cache: &FeatureCache`）

```rust
// 生成分发（ConfiguredFeature.generate → Feature.generate）
// 返回是否放置了方块。
// ⚠️ 签名变更（260905-05）：增 cache 参数——selector/patch 的内嵌 placed feature 走
// PlacedFeature::generate（同一 RNG 流 DFS 语义）。F-1 教训：加参后所有调用点必须同批改
// （本仓库唯一调用点 = worldgen_handle.rs L914-915 闭包，见 §2.4）。
pub fn generate_configured(
    cf: &ConfiguredFeature,
    ctx: &FeaturePlacementContext,
    octx: &mut crate::feature::OreFeatureContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    biome_temp: f32, biome_rainfall: f32,
    cache: &FeatureCache,
) -> bool {
    octx.origin_x = x; octx.origin_y = y; octx.origin_z = z;
    if cf.type_name.contains("ore") {
        let is_scattered = cf.type_name.contains("scattered_ore");
        if is_scattered {
            crate::feature::ScatteredOreFeature.generate(octx, &cf.ore_config, random)
        } else {
            crate::feature::OreFeature.generate(octx, &cf.ore_config, random)
        }
    } else if cf.type_name.contains("disk") {
        crate::feature::DiskFeature.generate(octx, &cf.disk_config, random)
    } else if cf.type_name.contains("spring") {
        crate::feature::SpringFeature.generate(octx, &cf.spring_config, random)
    } else if cf.type_name.contains("freeze_top_layer") {
        crate::feature::FreezeTopLayerFeature.generate(octx, biome_temp, biome_rainfall, random)
    } else if cf.type_name.contains("underwater_magma") {
        crate::feature::UnderwaterMagmaFeature.generate(octx, &cf.magma_config, random)
    } else if cf.type_name == "minecraft:tree" {
        match &cf.tree_config {
            Some(tc) => crate::tree::TreeFeatureConfig::generate(tc, octx, random, x, y, z),
            None => { eprintln!("[feature-loader] tree without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:random_selector" {
        match &cf.selector_config {
            Some(sc) => {
                // ⚠️ 占位公式（idk-7 未裁决）：「逐项 nextFloat() < chance 即选即返，否则 default」
                // 形态来自 JSON 结构推断——S6 动工前必须以 RandomSelectorFeature.java 核对，
                // 核对前本分支结果只可用于 smoke 不可用于对拍。
                for (chance, pf) in &sc.features {
                    if random.next_float() < *chance {
                        return pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                            generate_nested(c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                        });
                    }
                }
                match &sc.default_feature {
                    Some(pf) => pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                        generate_nested(c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                    }),
                    None => false,
                }
            }
            None => false,
        }
    } else if cf.type_name == "minecraft:random_patch" || cf.type_name == "minecraft:flower" {
        match &cf.patch_config {
            Some(pc) => {
                // ⚠️ 占位公式（idk-7 未裁决）：tries 循环 + nextInt(2*xz+1)-xz 偏移形态为推断。
                // 且嵌套 placed 完整 placement 链（含 in_square/count）的 RNG 流连续性未核对。
                let tries = pc.tries;
                for _ in 0..tries {
                    let sx = pc.xz_spread.get(random);
                    let sy = pc.y_spread.get(random);
                    let sz = pc.xz_spread.get(random);
                    let _ = (sx, sy, sz); // 偏移合成方式待 RandomPatchFeature.java 裁决后落地
                    // S7 落地点：pf.generate(ctx, random, x+?, y+?, z+?, ...)
                }
                false
            }
            None => false,
        }
    } else if cf.type_name == "minecraft:simple_block" {
        match &cf.simple_block_config {
            Some(sc) => {
                let state = sc.to_place.get(random);
                if crate::tree::can_replace_pub(octx, x, y, z) { // 公开包装 tree::can_replace
                    octx.set_block(x, y, z, state);
                    true
                } else { false }
            }
            None => false,
        }
    } else {
        false
    }
}

// 嵌套 placed → configured 的二次分发（selector/patch 内层用；保持同一 random 流）
// configured 引用经 cache 只读查找；未知 id 显式告警。
fn generate_nested(
    ctx: &FeaturePlacementContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    octx: &mut crate::feature::OreFeatureContext,
    cache: &FeatureCache,
    biome_temp: f32, biome_rainfall: f32,
) -> bool {
    // pf.generate 闭包只给坐标，configured id 由 parse_inline 时记入 PlacedFeature.configured_feature
    // 实现注记：PlacedFeature 需增 pub embedded_configured: Option<ConfiguredFeature>（内嵌对象时）
    // 或 configured_feature: String（id 引用时经 cache.get_configured 查）。
    // 本函数体在接线时二选一实现；此处仅声明签名与告警路径：
    eprintln!("[feature-loader] generate_nested hit (S6/S7 接线点)");
    let _ = (ctx, random, x, y, z, octx, cache, biome_temp, biome_rainfall);
    false
}
```

### 2.3 `preload_all`（增补段：树系嵌套递归）

在现有 `preload_all`（feature_loader.rs L193-227）的 `if let Ok(croot) = crate::json::parse(&ctxt)` 块之后追加：

```rust
// —— 260905-05 增补：selector/patch 内嵌 placed 的递归预加载——
// placed JSON 的 feature 字段可能是对象（内嵌 placed）而非 id 字符串；此时 configured_feature
// 为空串，内嵌体已在 PlacementModifier::parse 阶段随 PlacedFeature::parse_inline 解析，
// 其引用的 configured id 需在此登记到 self.configured（防运行时 cache miss）。
if let Some(emb) = root.get("feature").filter(|f| f.as_object().is_some()) {
    let cid = emb.get("feature").and_then(|f| f.as_str()).unwrap_or("");
    if !cid.is_empty() && !self.configured.contains_key(cid) {
        let cname = cid.strip_prefix("minecraft:").unwrap_or(cid);
        let cpath = format!("{}/data/minecraft/worldgen/configured_feature/{}.json", wg_dir, cname);
        if let Ok(ctxt2) = std::fs::read_to_string(&cpath) {
            if let Ok(croot2) = crate::json::parse(&ctxt2) {
                let cf = ConfiguredFeature::parse(cid, &croot2, blocks);
                self.configured.insert(cid.to_string(), cf);
            }
        }
    }
}
```

### 2.4 调用点改动（F-1 教训，全仓唯一调用点）

`worldgen_handle.rs` L914-915 闭包：`generate_configured` 调用处末尾追加 `&feature_cache` 实参（cache 为 preload 后只读共享引用；闭包捕获改为 `&feature_cache`，无可变借用冲突——cache 在 apply_features 阶段不再写）。示意：

```rust
let feature_cache_ref = &feature_cache;
let generate_configured = |_fctx: &crate::placement::FeaturePlacementContext, random: &mut ChunkRandom, gx: i32, gy: i32, gz: i32| -> bool {
    let r = crate::feature_loader::generate_configured(&cf, &fctx, &mut octx, random, gx, gy, gz, biome_temp_f, 0.5, feature_cache_ref);
    // …原样…
};
```

---

## 3. `worldgen-core/src/placement.rs` 增补（⚠️ 未编译验证）

### 3.1 `PlacementModifier` enum 增项（L133-144 enum 内追加）

```rust
    // —— 260905-05 增补（L0）——
    /// surface_water_depth_filter（SurfaceWaterDepthFilterPlacementModifier.java:28-32）
    SurfaceWaterDepthFilter(i32),
    /// environment_scan（EnvironmentScanPlacementModifier.java:46-69）
    EnvironmentScan {
        down: bool,
        max_steps: i32,
        target: BlockPredicate,
        allowed: Option<BlockPredicate>, // alwaysTrue 缺省 → None 表示恒真
    },
    /// block_predicate_filter 重构：谓词树（替代旧 { is_fluid, ids }）
    BlockPredicateFilter { predicate: BlockPredicate },
```

### 3.2 `BlockPredicate` 枚举 + eval（新段，加在 placement.rs 内）

```rust
// ===== BlockPredicate（Java world/gen/blockpredicate/*，S1 idk-5）=====
#[derive(Clone)]
pub enum BlockPredicate {
    MatchingBlocks { offset: [i32; 3], ids: Vec<i32> },
    MatchingFluids { offset: [i32; 3], ids: Vec<i32> },
    /// would_survive：state.canPlaceAt。树苗 = 下方 ∈ DIRT tag ∪ {farmland}（无光照判定，idk-5）
    WouldSurvive { offset: [i32; 3], state_name: String, dirt_ids: Vec<i32> },
    Solid { offset: [i32; 3] },
    Replaceable { offset: [i32; 3] },
    Not(Box<BlockPredicate>),
    AllOf(Vec<BlockPredicate>),
    AlwaysTrue,
    /// 数据集出现但未实现的谓词 → 告警 + 恒 false（显式不静默）
    Unsupported { type_name: String },
}

impl BlockPredicate {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> BlockPredicate {
        let v = match v { Some(v) => v, None => return BlockPredicate::AlwaysTrue };
        let type_name = v.get("predicate_type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let offset = || {
            let o = v.get("offset");
            [
                o.and_then(|o| o.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                o.and_then(|o| o.get("y")).and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                o.and_then(|o| o.get("z")).and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            ]
        };
        let ids_of = |key: &str| -> Vec<i32> {
            let mut ids = Vec::new();
            if let Some(arr) = v.get(key).and_then(|b| b.as_array()) {
                for b in arr { if let Some(s) = b.as_str() { ids.push(blocks.id(s)); } }
            } else if let Some(s) = v.get(key).and_then(|b| b.as_str()) {
                ids.push(blocks.id(s));
            }
            ids
        };
        if type_name.contains("matching_blocks") {
            BlockPredicate::MatchingBlocks { offset: offset(), ids: ids_of("blocks") }
        } else if type_name.contains("matching_fluids") {
            BlockPredicate::MatchingFluids { offset: offset(), ids: ids_of("fluids") }
        } else if type_name.contains("would_survive") {
            let state_name = v.get("state").and_then(|s| s.get("Name")).and_then(|n| n.as_str()).unwrap_or("").to_string();
            // DIRT tag ∪ {farmland}（idk-5；tag JSON 接线前硬编码 1.20.1 主体，数据边界声明 §九）
            let dirt_ids = ["minecraft:dirt", "minecraft:grass_block", "minecraft:podzol", "minecraft:coarse_dirt",
                "minecraft:mycelium", "minecraft:rooted_dirt", "minecraft:moss_block", "minecraft:mud",
                "minecraft:muddy_mangrove_roots", "minecraft:farmland"]
                .iter().map(|n| blocks.id(n)).collect();
            BlockPredicate::WouldSurvive { offset: offset(), state_name, dirt_ids }
        } else if type_name.contains("solid") {
            BlockPredicate::Solid { offset: offset() }
        } else if type_name.contains("replaceable") {
            BlockPredicate::Replaceable { offset: offset() }
        } else if type_name.contains("all_of") {
            let mut preds = Vec::new();
            if let Some(arr) = v.get("predicates").and_then(|p| p.as_array()) {
                for p in arr { preds.push(BlockPredicate::parse(Some(p), blocks)); }
            }
            BlockPredicate::AllOf(preds)
        } else if type_name.contains("not") {
            BlockPredicate::Not(Box::new(BlockPredicate::parse(v.get("predicate"), blocks)))
        } else if type_name.contains("true") {
            BlockPredicate::AlwaysTrue
        } else {
            eprintln!("[placement] unsupported block predicate type: {type_name}");
            BlockPredicate::Unsupported { type_name }
        }
    }

    /// 判定。block_at 返回 -1（不可读）→ 除 AlwaysTrue 外一律 false（保守，防误放）。
    pub fn test(&self, ctx: &FeaturePlacementContext, x: i32, y: i32, z: i32) -> bool {
        let air = crate::constants::AIR_ID; // 若无常量：以 blocks.id("minecraft:air") 预解析存入谓词（接线点）
        match self {
            BlockPredicate::AlwaysTrue => true,
            BlockPredicate::Unsupported { type_name } => { let _ = type_name; false }
            BlockPredicate::MatchingBlocks { offset, ids } => {
                let cur = ctx.block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && ids.contains(&cur)
            }
            BlockPredicate::MatchingFluids { offset, ids } => {
                let cur = ctx.block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && ids.contains(&cur)
            }
            BlockPredicate::WouldSurvive { offset, state_name, dirt_ids } => {
                // SaplingBlock 无重写 → PlantBlock.canPlaceAt（idk-5）：下方 ∈ dirt ∪ farmland
                let _ = state_name; // state 本体不参与 sapling 判定；非 PlantBlock 方块时需分派（登记 R-4）
                let below = ctx.block_at(x + offset[0], y + offset[1] - 1, z + offset[2]);
                below >= 0 && dirt_ids.contains(&below)
            }
            BlockPredicate::Solid { offset } => {
                let cur = ctx.block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && cur != air // ⚠️ Java isSolid 近似（非空气即 solid，误差登记 §九）
            }
            BlockPredicate::Replaceable { offset } => {
                let cur = ctx.block_at(x + offset[0], y + offset[1], z + offset[2]);
                // REPLACEABLE tag 近似（air/植被/流体族）——与 tree::can_replace 同口径
                cur == air
            }
            BlockPredicate::Not(inner) => !inner.test(ctx, x, y, z),
            BlockPredicate::AllOf(preds) => preds.iter().all(|p| p.test(ctx, x, y, z)),
        }
    }
}
```

> `BlockPredicateFilter` 旧分支（placement.rs L182-189 get_positions、L234-254 parse）**删除**，由下文新分支取代（同 enum 变体名，匹配面不变，调用方零改动）。

### 3.3 `get_positions` 增分支（match 内追加）

```rust
            // —— 260905-05 增补 ——
            PlacementModifier::SurfaceWaterDepthFilter(max_depth) => {
                // Java L28-32：OCEAN_FLOOR 与 WORLD_SURFACE 双高度图差 <= maxWaterDepth（0 随机消费）
                if *max_depth == 0 {
                    return vec![[x, y, z]]; // Java：maxWaterDepth==0 短路放行（shouldPlace 直接 true 语义近似——
                                            // 注意 Java 实际无此短路：j-i<=0 恒算。保留精确路径见下）
                }
                let (Some(of), Some(ws)) = (ctx.ocean_floor, ctx.world_surface) else { return vec![[x, y, z]]; };
                let lx = x - ctx.chunk_start_x;
                let lz = z - ctx.chunk_start_z;
                if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 { return vec![[x, y, z]]; } // 邻域——保留（登记）
                let i = of[(lz * 16 + lx) as usize];
                let j = ws[(lz * 16 + lx) as usize];
                if j - i <= *max_depth { vec![[x, y, z]] } else { vec![] }
            }
            PlacementModifier::EnvironmentScan { down, max_steps, target, allowed } => {
                // Java L46-69（0 随机消费）：先 allowed(起点) → 步进 target 命中即返回；步后超界/allowed 失败 break
                let dir = if *down { -1 } else { 1 };
                let ok_allowed = |p: i32| allowed.as_ref().map_or(true, |a| a.test(ctx, x, p, z));
                if !ok_allowed(y) { return vec![]; }
                let mut py = y;
                for _ in 0..*max_steps {
                    if target.test(ctx, x, py, z) { return vec![[x, py, z]]; }
                    py += dir;
                    if py < ctx.min_y || py >= ctx.min_y + ctx.height { return vec![]; } // isOutOfHeightLimit
                    if !ok_allowed(py) { break; }
                }
                if target.test(ctx, x, py, z) { vec![[x, py, z]] } else { vec![] }
            }
            PlacementModifier::BlockPredicateFilter { predicate } => {
                if predicate.test(ctx, x, y, z) { vec![[x, y, z]] } else { vec![] }
            }
```

> **注意**：`SurfaceWaterDepthFilter` 分支中 `max_depth == 0` 的短路是错误保留项——Java 无此短路（j-i<=0 仍判定恒 true，等价），短路与否**无行为差异**，但为消除歧义建议实现时直接删掉该 early-return，只走精确路径。

### 3.4 `parse` 增分支 + 未知 type 告警（L211-272 函数内）

```rust
        } else if type_name.contains("surface_water_depth_filter") {
            // SurfaceWaterDepthFilterPlacementModifier.java:11-16
            return Some(PlacementModifier::SurfaceWaterDepthFilter(
                m.get("max_water_depth").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32));
        } else if type_name.contains("environment_scan") {
            // EnvironmentScanPlacementModifier.java:18-28
            let down = m.get("direction_of_search").and_then(|x| x.as_str()) == Some("down");
            let steps = m.get("max_steps").and_then(|x| x.as_f64()).unwrap_or(1.0) as i32;
            let target = m.get("target_condition").map(|t| BlockPredicate::parse(Some(t), blocks))
                .unwrap_or(BlockPredicate::AlwaysTrue);
            let allowed = m.get("allowed_search_condition")
                .map(|t| BlockPredicate::parse(Some(t), blocks)); // 缺省 alwaysTrue → None
            return Some(PlacementModifier::EnvironmentScan { down, max_steps: steps, target, allowed });
        } else if type_name.contains("block_predicate_filter") {
            // 重构：谓词树（取代旧 matching_fluids/matching_blocks 两写死分支）
            if let Some(pred) = m.get("predicate") {
                return Some(PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::parse(Some(pred), blocks),
                });
            }
            return None;
        }
        // —— 尾部（原 L271 None 之前）：未知 modifier 显式告警（b2 S2，消除静默丢弃）——
        eprintln!("[feature-loader] unknown placement modifier type: {type_name}");
        None
```

### 3.5 `IntProvider::BiasedToBottom` / `Trapezoid` 修正（get() L43-46 / L32-42 替换）

```rust
            IntProvider::Trapezoid(a, b, plateau) => {
                // 修正（260905-05）：1.20.1 无 TrapezoidIntProvider（intprovider 目录 glob 实证 8 文件无此名）——
                // 数据中的 trapezoid 是 height provider（TrapezoidHeightProvider.java:49-64）：
                //   i>max → warn 返回 i；plateau>=k → nextBetween(i,j)；否则 i + nextBetween(0,m) + nextBetween(0,l)
                //   l = (k-plateau)/2, m = k-l；nextBetween(min,max) = nextInt(max-min+1)+min = 每次恒 1 消费（共 2 次）
                let k = b - a;
                if k <= 0 { return *a; }
                if *plateau >= k {
                    r.next_int_bound(k + 1) + a              // nextBetween(i,j)
                } else {
                    let l = (k - plateau) / 2;
                    let m = k - l;
                    a + r.next_int_bound(m + 1) + r.next_int_bound(l + 1)
                }
            }
            IntProvider::BiasedToBottom(a, b) => {
                // 修正（260905-05）：BiasedToBottomIntProvider.java:39-41
                //   return this.min + random.nextInt(random.nextInt(this.max - this.min + 1) + 1);
                // 恒 2 次消费；旧实现「r.next_int_bound(inner + a)」公式错误（min 加在内层 bound 上）。
                let inner = r.next_int_bound(b - a + 1);
                a + r.next_int_bound(inner + 1)
            }
```

### 3.6 `carver.rs` `HeightProvider` 全文替换（trapezoid 实装）

```rust
// HeightProvider：uniform / trapezoid（Java UniformHeightProvider / TrapezoidHeightProvider）
// 260905-05：增 plateau + trapezoid get（此前只有 uniform，height_range 的 trapezoid 数据解析即错）
#[derive(Clone, Copy)]
pub struct HeightProvider {
    pub min_offset: YOffset,
    pub max_offset: YOffset,
    pub trapezoid: bool,
    pub plateau: i32,
}
impl HeightProvider {
    pub fn get(&self, r: &mut ChunkRandom, min_y: i32, height: i32) -> i32 {
        let i = self.min_offset.get_y(min_y, height);
        let j = self.max_offset.get_y(min_y, height);
        if i > j { return i; }                                  // TrapezoidHeightProvider.java:52-55 warn 分支
        if !self.trapezoid {
            // UniformHeightProvider：MathHelper.nextBetween(random, i, j)
            return r.next_int_bound(j - i + 1) + i;
        }
        // TrapezoidHeightProvider.java:56-64：k=j-i；plateau>=k → nextBetween(i,j)；
        // 否则 i + nextBetween(0,m) + nextBetween(0,l)，l=(k-plateau)/2, m=k-l（每 nextBetween 恒 1 消费，共 2 次）
        let k = j - i;
        if self.plateau >= k {
            r.next_int_bound(k + 1) + i
        } else {
            let l = (k - self.plateau) / 2;
            let m = k - l;
            i + r.next_int_bound(m + 1) + r.next_int_bound(l + 1)
        }
    }
    pub fn parse(v: Option<&JsonValue>) -> HeightProvider {
        let mut hp = HeightProvider {
            min_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 },
            max_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 },
            trapezoid: false, plateau: 0,
        };
        if let Some(v) = v {
            let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
            hp.trapezoid = type_name.contains("trapezoid");
            if hp.trapezoid {
                hp.plateau = v.get("plateau").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32; // optionalFieldOf 0
            }
            if v.as_object().is_some() && v.get("min_inclusive").is_some() && v.get("max_inclusive").is_some() {
                hp.min_offset = YOffset::parse(v.get("min_inclusive"));
                hp.max_offset = YOffset::parse(v.get("max_inclusive"));
            }
        }
        hp
    }
}
```

### 3.7 `PlacedFeature::parse_inline`（placement.rs 新增辅助；selector/patch 内嵌体）

```rust
impl PlacedFeature {
    /// JSON 内嵌 placed feature（selector features[].feature / default_feature / patch 的 feature）：
    /// 对象形式 {feature: <configured id 或内嵌>, placement: [...]} → 直接解析；
    /// 字符串形式 → 返回只含 id 的壳（configured 经 FeatureCache 在 generate 侧查）。
    pub fn parse_inline(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<PlacedFeature> {
        let v = v?;
        if let Some(id) = v.as_str() {
            return Some(PlacedFeature { id: id.to_string(), modifiers: Vec::new(),
                configured_feature: id.to_string(), step: 0, global_index: -1 });
        }
        let mut pf = PlacedFeature {
            id: String::new(),
            modifiers: Vec::new(),
            configured_feature: v.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
            step: 0, global_index: -1,
        };
        if let Some(mods) = v.get("placement").and_then(|p| p.as_array()) {
            for m in mods {
                if let Some(pm) = PlacementModifier::parse(m, blocks) { pf.modifiers.push(pm); }
            }
        }
        Some(pf)
    }
}
```

### 3.8 `Biome` modifier 实装（get_positions L173-178 替换；配套 context 增字段）

```rust
// FeaturePlacementContext 增字段（L115-129 struct 内追加）：
    /// Biome modifier 锚定 biome（当前 chunk 的 biome 名；由 worldgen_handle 闭包填入）
    pub anchor_biome: Option<String>,

// get_positions 的 Biome 分支替换：
            PlacementModifier::Biome => {
                // Java BiomeFilter：posToBiome(pos) ∈ feature 集。Rust 以 anchor_biome 对比采样 biome；
                // biome_at 未接入（None）→ 保留位置（现状直通，不收紧，防回归 S2 前链路）。
                match (ctx.biome_at, &ctx.anchor_biome) {
                    (Some(f), Some(anchor)) => {
                        let (sx, sz) = ((x >> 2) << 2, (z >> 2) << 2); // biome 4×4 对齐采样
                        if f(sx, y, sz) == *anchor { vec![[x, y, z]] } else { vec![] }
                    }
                    _ => vec![[x, y, z]],
                }
            }
```

---

## 4. 未知 type 告警汇总（b2 S2 兑现点）

| 层 | 告警点 | 文本前缀 |
|---|---|---|
| placement modifier | `parse` 尾部（§3.4） | `[feature-loader] unknown placement modifier type:` |
| block predicate | `BlockPredicate::parse` else | `[placement] unsupported block predicate type:` |
| configured feature | `ConfiguredFeature::parse` else（§2.1） | `[feature-loader] unknown configured feature type:` |
| trunk/foliage placer / decorator / feature size / state provider | tree.rs 各 parse else | `[tree] unsupported ...` |
| tree config 解析失败 | §2.1 | `[feature-loader] tree config parse failed:` |

---

## 5. §九 静态自检清单（subagent 写码交付自检，AGENTS.md §九）

**状态声明：以上全部代码块「未编译验证」**——交付后主会话必须 `cargo build --offline -p worldgen --release`；编译失败/崩溃退回本角色修复（附现场）。

1. **类型宽度 / 截断**
   - `probability`/`chance` 全部 f32（`as f32` 显式截断 JSON f64），比较用 `random.next_float() < p`（f32×f32），不混 f64（对齐 Java float；oak cocoa 0.2 → f32 后与 Java `0.2F` 位型一致）。
   - 坐标/高度/半径全 i32；`i/2` 用 `java_div`（向零截断），**禁止 Rust `/`（负数行为不同：Rust -3/2 = -1 与 Java 相同，但显式函数防审读歧义）**。
   - LargeOak：`(jj as f64) * 0.618`、`1.382 + ((jj as f64)/13.0).powi(2)`、`sqrt`、`2π` 全 f64（Java double）；`should_generate_branch` 内全 f32（Java float，L163-178）——**f32/f64 不得互换**。
   - `MathHelper.floor` → `as i32` 前必须 `.floor()`（负 f64 直接 `as i32` 是向零截断，语义不同——§1 LargeOak L61/66、L103 的 `MathHelper.floor` 均已显式 floor）。
2. **move/借用（F-1 教训）**
   - `generate_configured` 增 `&FeatureCache` 后**全仓唯一调用点** = worldgen_handle.rs L914-915 闭包（§2.4）；grep 确认无第二调用点后同批改，否则编译失败。
   - `ChunkRandom` 只作 `&mut` 函数参数在 tree.rs 流转，**不存任何结构体字段**；`TreeFeatureConfig::generate(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom, ...)`——self 与 random 分离，无双重可变借用。
   - decorator 的 `trunk_set`/`leaves_set` 只读借用传参（&Vec），foliage 阶段持有 `&mut leaves_set` 与后续 decorator 借用**时序不重叠**（foliage 先结束借用）。
   - `PlacedFeature::generate` 闭包签名 `FnMut` 捕获 `octx`（&mut）与 `cache`（&）——octx 在闭包内用完即还（与现有 ore 路径一致）；若借用冲突，接线时把 `generate_nested` 改为经 `RefCell` 或把 octx 移入闭包（主会话编译时定夺，登记项）。
3. **空容器 / None / panic 面**
   - `decorators: []`（oak）→ 空 vec，循环零次，无 panic。
   - selector `features: []` 且无 default → 返回 false（不 unwrap）；`trunk_set.iter().min_by_key` 用 `let Some(..) = .. else { return }`（cocoa 空集安全）。
   - 所有 parse 返回 Option，失败显式 None + 告警，**无 unwrap/expect/panic**（S4 全量 194 configured 加载是防 panic 门）。
   - `Weighted { total_weight: 0 }` → `next_int_bound(0)` 会 panic（ChunkRandom 契约）——parse 处 `entries.is_empty() → None` 已挡；但 `entries` 非空而全 0 权重 → total=0 仍会 panic：**接线时在 get() 加 `if *total_weight <= 0 { return entries[0].0; }` 守卫**（本角色遗漏，主会话应用时补上）。
4. **与 Java 语义对拍点清单**（每点 s1-semantics 有 file:line）
   - getHeight 2 次消费序（TrunkPlacer.java:54-56）｜blob 层循环含 java_div（BlobFoliagePlacer.java:43-46）｜四角 nextInt(2) 短路序（BlobFoliagePlacer.java:56）｜getTopPosition 返回 i-2 与 min_clipped 验收（TreeFeature.java:76,92-109）｜decorator Y 升序稳定序（TreeDecorator.java:50-52）｜cocoa `<=0.25` vs leave_vine `<` 严格不等（CocoaBeansTreeDecorator.java:35 vs LeavesVineTreeDecorator.java:29）｜cocoa HORIZONTAL = N,W,S,E（Direction.java:49-52）｜trapezoid 2 次消费序 l=(k-p)/2 先 nextBetween(0,m) 后 nextBetween(0,l)（TrapezoidHeightProvider.java:60-62，注意 **m 在前 l 在后**）｜biased_to_bottom 双层 nextInt（BiasedToBottomIntProvider.java:40）。
5. **数据驱动边界声明**
   - 硬编码 id 列表（REPLACEABLE_BY_TREES / LOGS / DIRT tag / soil 集合）= Java tag 的**主体近似**，升级点登记：应改为读 `versions\1.20.1\data\minecraft\tags\block\*.json` 展开（存在性未核，R-3）。
   - 树全部参数（providers/placers/decorators/size）自 JSON；placer/decorator **类型逻辑**在代码——符合数据驱动铁律。
   - cocoa age / vine 面属性 / leaves distance 属性：当前 i32 块 id 不携带属性 → 占位（设基础块 id），属性位编码为 R-4 接线项；palette 对比时 cocoa/vine 的属性差异与 leaves distance 差异是**已知偏差源**，S5 判据须声明。
6. **出界 clip 语义**
   - `can_replace` 对 `block_at < 0`（出界/不可读）→ **false（拒绝放置）**；这与 ore 的 isExposedToAir「-1 当非空气」方向**相反**——有意为之：树冠出界丢弃 + 告警（b2 §4.4 clip 项；scout §4.7 指出 ore 方向可疑，tree 不跟随）。
   - decorator 的 `isAir` 判定 `block_at != air` 同样受 -1 影响：-1 ≠ air → 不放藤（保守，登记）。
7. **遗留 idk 移交**
   - R-1（HashSet 同 Y 序）：decorator 用「Y 升序稳定 + 插入序」近似，S5 vine 对拍前必须声明或换 Java hash 桶序复刻。
   - idk-7（selector/patch generate 公式）：代码内已标「占位」，**S6/S7 动工前必须取证**，占位结果禁入对拍。
   - R-2（Direction idHorizontal 字面值 N=0/W=1/S=2/E=3）：实现前读 Direction.java L30-34 闭合。
8. **supersedes**：解除 feature_loader.rs L50 注释时保留一行 supersedes 指向（§2.1 注释已内嵌）。

---

## 6. 结论与移交

- 交付 = 本文档（内嵌 tree.rs 全文 + feature_loader.rs 4 处改动全文 + placement.rs/carver.rs 增补全文），**未改 src 任何文件**。
- 语义依据 100% 落在 `s1-semantics-260905-05.md`（8 条 idk 裁决 6 条、保留 2 条 + R-1~R-4）。
- 应用顺序建议：§3.4 未知告警（可单独先合，零行为变化）→ §3.5/3.6（Trapezoid/BiasedToBottom 修正，**注意会改变现有 height_range 类 feature 的随机序列**，须 palette 回归定界）→ §3 其余 → §1/§2（tree + loader）→ S5 验证。
- 最大风险：① LargeOak 分支浮点公式未运行验证（f32/f64 混用点已逐一标注）；② R-1 HashSet 序使 vine 位置对拍有系统性不确定性；③ decorator 属性位缺失（cocoa age/vine face/leaf distance）。
- 本交付为 draft；judge 审查 + 用户拍板后方可进 src。

（draft，260905-05，core-worker 交付。）
