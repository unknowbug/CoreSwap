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
                // 应用时需补守卫（patch §5.3）：entries 非空而全 0 权重 → total=0 时 next_int_bound 会 panic
                if *total_weight <= 0 { return entries[0].0; }
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

// ===== TrunkPlacer（L1：straight + large_oak(fancy) + mega_jungle(giant)；其余告警跳过）=====
#[derive(Clone)]
pub enum TrunkPlacer {
    /// minecraft:straight_trunk_placer（TrunkPlacer.java:54-56 高度 + StraightTrunkPlacer.java:30-40）
    Straight { base_height: i32, rand_a: i32, rand_b: i32 },
    /// minecraft:fancy_trunk_placer（= LargeOakTrunkPlacer.java:38-88）
    LargeOak { base_height: i32, rand_a: i32, rand_b: i32 },
    /// minecraft:mega_jungle_trunk_placer（260905-06：GiantTrunkPlacer.java:31-53 + MegaJungleTrunkPlacer.java:31-52）
    MegaJungle { base_height: i32, rand_a: i32, rand_b: i32 },
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
        } else if type_name.contains("mega_jungle_trunk_placer") {
            TrunkPlacer::MegaJungle { base_height: f("base_height"), rand_a: f("height_rand_a"), rand_b: f("height_rand_b") }
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
            | TrunkPlacer::LargeOak { base_height, rand_a, rand_b }
            | TrunkPlacer::MegaJungle { base_height, rand_a, rand_b } => {
                base_height + random.next_int_bound(rand_a + 1) + random.next_int_bound(rand_b + 1)
            }
            TrunkPlacer::Unsupported { .. } => 0,
        }
    }
}

// ===== FoliagePlacer（L1：blob + fancy(=LargeOakFoliagePlacer) + bush + jungle；其余告警）=====
#[derive(Clone)]
pub enum FoliagePlacer {
    /// minecraft:blob_foliage_placer（BlobFoliagePlacer.java:32-57）
    Blob { radius: IntProv, offset: IntProv, height: i32 },
    /// minecraft:fancy_foliage_placer（LargeOakFoliagePlacer.java:26-46，blob 子类）
    LargeOak { radius: IntProv, offset: IntProv, height: i32 },
    /// minecraft:bush_foliage_placer（260905-06：BushFoliagePlacer.java:31-49，blob 子类）
    Bush { radius: IntProv, offset: IntProv, height: i32 },
    /// minecraft:jungle_foliage_placer（260905-06：JungleFoliagePlacer.java:27-66）
    Jungle { radius: IntProv, offset: IntProv, height: i32 },
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
        } else if type_name.contains("bush_foliage_placer") {
            FoliagePlacer::Bush { radius, offset, height }
        } else if type_name.contains("jungle_foliage_placer") {
            FoliagePlacer::Jungle { radius, offset, height }
        } else {
            eprintln!("[tree] unsupported foliage placer type: {type_name}");
            FoliagePlacer::Unsupported { type_name }
        }
    }

    /// getRandomHeight：blob/fancy/bush/jungle 恒常量，0 消费
    fn get_random_height(&self) -> i32 {
        match self {
            FoliagePlacer::Blob { height, .. } | FoliagePlacer::LargeOak { height, .. }
            | FoliagePlacer::Bush { height, .. } | FoliagePlacer::Jungle { height, .. } => *height,
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }
    /// getRandomRadius（FoliagePlacer.java:68-70）：radius.get(random)，消费随 IntProvider 类型
    fn get_random_radius(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            FoliagePlacer::Blob { radius, .. } | FoliagePlacer::LargeOak { radius, .. }
            | FoliagePlacer::Bush { radius, .. } | FoliagePlacer::Jungle { radius, .. } => radius.get(random),
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }
    /// 主 generate 外层的 offset.get(random)（FoliagePlacer.java:48,72-74）
    fn get_random_offset(&self, random: &mut ChunkRandom) -> i32 {
        match self {
            FoliagePlacer::Blob { offset, .. } | FoliagePlacer::LargeOak { offset, .. }
            | FoliagePlacer::Bush { offset, .. } | FoliagePlacer::Jungle { offset, .. } => offset.get(random),
            FoliagePlacer::Unsupported { .. } => 0,
        }
    }

    /// 树叶层生成。tree_node = (center_x, center_y, center_z, foliage_radius, giant_trunk)
    /// 消费点：仅每层四角 nextInt(2)（blob/bush）；jungle 非 giant 首层 1 次 nextInt(2)；fancy 无
    fn generate_foliage(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                        cfg: &TreeFeatureConfig, tree_node_y: i32, tree_node_radius: i32,
                        center_x: i32, center_z: i32, giant: bool, foliage_height: i32, radius: i32,
                        trunk_set: &Vec<[i32; 3]>, leaves_set: &mut Vec<[i32; 3]>) {
        let offset = self.get_random_offset(random); // 1 次等价 IntProvider 消费（constant → 0 draw）
        match self {
            FoliagePlacer::Blob { .. } => {
                // BlobFoliagePlacer.java:43-46：i 从 offset 到 offset-foliageHeight（含），i/2 向零截断
                for i in 0..=foliage_height as i64 {
                    let ii = offset - i as i32;
                    let j = std::cmp::max(radius + tree_node_radius - 1 - java_div(ii, 2), 0);
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, j, ii, giant, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::LargeOak { .. } => {
                // LargeOakFoliagePlacer.java:37-40：j = radius + (i!=offset && i!=offset-foliageHeight ? 1 : 0)
                for i in 0..=foliage_height as i64 {
                    let ii = offset - i as i32;
                    let j = radius + if ii != offset && ii != offset - foliage_height { 1 } else { 0 };
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, j, ii, giant, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::Bush { .. } => {
                // BushFoliagePlacer.java:38-45：j = radius + nodeRadius - 1 - i（无 /2、无 max0）
                for i in 0..=foliage_height as i64 {
                    let ii = offset - i as i32;
                    let j = radius + tree_node_radius - 1 - ii;
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, j, ii, giant, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::Jungle { .. } => {
                // JungleFoliagePlacer.java:39-49：i = giant ? foliageHeight : 1 + nextInt(2)（1 消费）
                // k = radius + nodeRadius + 1 - j；j 从 offset 到 offset-i（含）
                let i = if giant { foliage_height } else { 1 + random.next_int_bound(2) };
                for j in 0..=i as i64 {
                    let jj = offset - j as i32;
                    let k = radius + tree_node_radius + 1 - jj;
                    self.generate_square(ctx, random, cfg, center_x, center_z, tree_node_y, k, jj, giant, trunk_set, leaves_set);
                }
            }
            FoliagePlacer::Unsupported { .. } => {}
        }
    }

    /// FoliagePlacer.generateSquare（FoliagePlacer.java:101-115）：giant 时范围扩到 r+1（i=1）
    fn generate_square(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                       cfg: &TreeFeatureConfig, cx: i32, cz: i32, cy: i32, r: i32, y: i32, giant: bool,
                       trunk_set: &Vec<[i32; 3]>, leaves_set: &mut Vec<[i32; 3]>) {
        if r < 0 { return; }
        let ext = if giant { 1 } else { 0 };
        for dx in -r..=(r + ext) {
            for dz in -r..=(r + ext) {
                if self.is_position_invalid(random, dx, y, dz, r, giant) { continue; }
                let (px, py, pz) = (cx + dx, cy + y, cz + dz);
                if place_foliage_block(ctx, random, cfg, px, py, pz) {
                    leaves_set.push([px, py, pz]);
                }
            }
        }
        let _ = trunk_set; // hasPlacedBlock 仅 hanging-leaves 变体使用；blob/fancy 不用
    }

    /// isPositionInvalid → isInvalidForLeaves（FoliagePlacer.java:84-96 + 各 placer 覆写）
    fn is_position_invalid(&self, random: &mut ChunkRandom, dx: i32, y: i32, dz: i32, r: i32, giant: bool) -> bool {
        // FoliagePlacer.java:87-93：giant 归一化 min(|dx|,|dx-1|)
        let (ax, az) = if giant {
            (std::cmp::min(dx.abs(), (dx - 1).abs()), std::cmp::min(dz.abs(), (dz - 1).abs()))
        } else {
            (dx.abs(), dz.abs())
        };
        match self {
            // blob：四角才判；nextInt(2) 恒消费（|| 短路在 nextInt 之后），y==0 必 invalid
            FoliagePlacer::Blob { .. } => {
                if ax == r && az == r { random.next_int_bound(2) == 0 || y == 0 } else { false }
            }
            // fancy：圆盘判定，无随机消费（LargeOakFoliagePlacer.java:44-46）
            FoliagePlacer::LargeOak { .. } => {
                let fx = dx as f32 + 0.5; let fz = dz as f32 + 0.5;
                fx * fx + fz * fz > (r as f32) * (r as f32)
            }
            // bush：BushFoliagePlacer.java:47-49：仅角判 nextInt(2)，无 y==0 子句
            FoliagePlacer::Bush { .. } => {
                ax == r && az == r && random.next_int_bound(2) == 0
            }
            // jungle：JungleFoliagePlacer.java:62-64：无随机；dx+dz>=7 或圆盘外
            FoliagePlacer::Jungle { .. } => {
                ax + az >= 7 || ax * ax + az * az > r * r
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

/// 公开包装 tree::can_replace（应用适配：patch §2.2 simple_block 分支引用 `crate::tree::can_replace_pub`，
/// patch 文档未交付该包装函数体——编译必需项，语义 = 原样转发）
pub fn can_replace_pub(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    can_replace(ctx, x, y, z)
}

// ===== TreeDecorator（L1：cocoa / trunk_vine / leave_vine）=====
#[derive(Clone)]
pub enum TreeDecorator {
    Cocoa { probability: f32 },   // f32：JSON f64 → f32 与 Java float 一致（勿混 f64）
    TrunkVine,
    LeaveVine { probability: f32 },
    Beehive { probability: f32 }, // BeehiveTreeDecorator.java:43-75（260905-10 P2 缺抽修复）
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
        } else if type_name.contains("beehive") {
            TreeDecorator::Beehive { probability: prob() }
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
                                let (ox, oz) = opposite(&dir);
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
            TreeDecorator::Beehive { probability } => {
                // BeehiveTreeDecorator.java:43-75：恒 1 次 nextFloat 门（0.002 配置）——RNG 消费对齐是主目的
                if !(random.next_float() >= *probability) {
                    // i = leaves 非空 ? max(leaves[0].y-1, logs[0].y+1)
                    //              : min(logs[0].y+1+nextInt(3), logs[last].y)（leaves 空时多 1 消费）
                    let i = if !leaves_set.is_empty() {
                        let a = leaves_set.iter().map(|p| p[1]).max().unwrap_or(0);
                        let b = trunk_set.iter().map(|p| p[1]).min().unwrap_or(0);
                        // Java leaves[0]/logs[0] = 生成序首元素（非 y 极值）；此处用极值近似（idk-bee1）
                        i32::max(a - 1, b + 1)
                    } else {
                        if trunk_set.is_empty() { return; }
                        let lo = trunk_set.iter().map(|p| p[1]).min().unwrap_or(0);
                        let hi = trunk_set.iter().map(|p| p[1]).max().unwrap_or(0);
                        i32::min(lo + 1 + random.next_int_bound(3), hi)
                    };
                    // 候选 = y==i 的 log × 3 水平向（java GENERATE_DIRECTIONS = HORIZONTAL 流序 N,E,S,W 去 NORTH → E,S,W）
                    // java Collections.shuffle 用自有 Random（非世界流，不消费；顺序不确定）→
                    // rust 取确定序首候选（放置点可能偶差，RNG 流不受影响；idk-bee2）
                    let nest = ctx.blocks.id("minecraft:bee_nest");
                    let air = ctx.blocks.id("minecraft:air");
                    // java GENERATE_DIRECTIONS = HORIZONTAL（N,E,S,W，Direction.java:499）去 NORTH → {E,S,W} 3 向
                    'outer: for pos in trunk_set.iter().filter(|p| p[1] == i) {
                        for (dx, dz) in [(1, 0), (0, 1), (-1, 0)] {
                            let (bx, bz) = (pos[0] + dx, pos[2] + dz);
                            if ctx.block_at(bx, i, bz) == air && ctx.block_at(bx, i, bz + 1) == air {
                                ctx.set_block(bx, i, bz, nest);
                                // java: ix = 2 + nextInt(2)；随后 ix 次 nextInt(599)（蜂实体 tick offset）
                                let ix = 2 + random.next_int_bound(2);
                                for _ in 0..ix { let _ = random.next_int_bound(599); }
                                break 'outer; // java findFirst 只放一个蜂巢
                            }
                        }
                    }
                }
            }
            TreeDecorator::Unsupported { .. } => {
                // WG_TREEDIAG（260905-10 P2）：漏实现 decorator 的缺失消费点标记（如 beehive 恒 1 次 nextFloat）
                if crate::placement::treediag_enabled() { eprintln!("[BEE-MISS] unsupported decorator consumed nothing"); }
            }
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
        // WG_TREEDIAG（260905-10 P2，b1 §4 模板）：getHeight 结果打点（与 placement [CNT]/[SQ] 同 env 门控）
        if crate::placement::treediag_enabled() { eprintln!("[TH] {i} @ ({x},{y},{z})"); }
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
        // ⑦ trunk（含 setToDirt；straight=TreeFeature 序，LargeOak/MegaJungle 见各 fn）
        let mut trunk_set: Vec<[i32; 3]> = Vec::new();
        let mut leaves_set: Vec<[i32; 3]> = Vec::new();
        let tree_nodes: Vec<(i32, i32, i32, i32, bool)> = match &self.trunk_placer {
            TrunkPlacer::Straight { .. } => {
                self.set_to_dirt(ctx, random, bx, by - 1, bz);
                for iy in 0..o {
                    if can_replace(ctx, bx, by + iy, bz) {
                        let state = self.trunk_provider.get(random);
                        ctx.set_block(bx, by + iy, bz, state);
                        trunk_set.push([bx, by + iy, bz]);
                    }
                }
                vec![(bx, by + o, bz, 0, false)] // TreeNode(pos.up(height), 0, false)
            }
            TrunkPlacer::LargeOak { .. } => {
                self.large_oak_trunk(ctx, random, bx, by, bz, o, &mut trunk_set)
                    .into_iter().map(|(x, y, z, r)| (x, y, z, r, false)).collect()
            }
            TrunkPlacer::MegaJungle { .. } => {
                self.mega_jungle_trunk(ctx, random, bx, by, bz, o, &mut trunk_set)
            }
            TrunkPlacer::Unsupported { .. } => return false,
        };
        // ⑧ foliage（逐 node；TreeFeature.java:81）
        for (nx, ny, nz, nr, ng) in &tree_nodes {
            self.foliage_placer.generate_foliage(ctx, random, self, *ny, *nr, *nx, *nz, *ng, j, l, &trunk_set, &mut leaves_set);
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

    /// GiantTrunkPlacer.generate（GiantTrunkPlacer.java:31-53）+ MegaJungleTrunkPlacer 分支（:31-52）。
    /// 2×2 巨干：dirt 四角 base，柱体 (0,0)(1,0)(1,1)(0,1) 顶列缺 (i=height-1 无 +1)。
    /// MegaJungle 附加横向分支：i 从 height-2-nextInt(4) 起、步长 2+nextInt(4)，每支 1 次 nextFloat
    /// + 5 个 getAndSetState（l/2 整除）+ TreeNode(j,i,k, radius=-2, 非 giant)。
    fn mega_jungle_trunk(&self, ctx: &mut OreFeatureContext, random: &mut ChunkRandom,
                         sx: i32, sy: i32, sz: i32, height: i32, trunk_set: &mut Vec<[i32; 3]>) -> Vec<(i32, i32, i32, i32, bool)> {
        // GiantTrunkPlacer.java:35-39：base dirt 四角
        self.set_to_dirt(ctx, random, sx, sy - 1, sz);
        self.set_to_dirt(ctx, random, sx + 1, sy - 1, sz);
        self.set_to_dirt(ctx, random, sx, sy - 1, sz + 1);
        self.set_to_dirt(ctx, random, sx + 1, sy - 1, sz + 1);
        // GiantTrunkPlacer.java:41-50：2×2 柱
        let mut place_log = |tx: i32, ty: i32, tz: i32, trunk_set: &mut Vec<[i32; 3]>| {
            if can_replace(ctx, tx, ty, tz) {
                let state = self.trunk_provider.get(random);
                ctx.set_block(tx, ty, tz, state);
                trunk_set.push([tx, ty, tz]);
            }
        };
        for iy in 0..height {
            place_log(sx, sy + iy, sz, trunk_set);            // (0,i,0)
            if iy < height - 1 {
                place_log(sx + 1, sy + iy, sz, trunk_set);    // (1,i,0)
                place_log(sx + 1, sy + iy, sz + 1, trunk_set);// (1,i,1)
                place_log(sx, sy + iy, sz + 1, trunk_set);    // (0,i,1)
            }
        }
        let mut nodes = vec![(sx, sy + height, sz, 0, true)]; // TreeNode(pos.up(height), 0, true)
        // MegaJungleTrunkPlacer.java:39-51：横向枝干
        let two_pi = std::f32::consts::PI * 2.0;
        let mut i = height - 2 - random.next_int_bound(4);
        while i > height / 2 {
            let f = random.next_float() as f32 * two_pi;
            let mut j = 0i32;
            let mut k = 0i32;
            for l in 0..5i32 {
                // MegaJungleTrunkPlacer.java:43-44：MathHelper.cos/sin = 65536 项查表，非 libm
                j = (1.5f32 + crate::carver::math_cos(f) * l as f32) as i32;
                k = (1.5f32 + crate::carver::math_sin(f) * l as f32) as i32;
                let (px, py, pz) = (sx + j, sy + i - 3 + l / 2, sz + k);
                if can_replace(ctx, px, py, pz) {
                    let state = self.trunk_provider.get(random);
                    ctx.set_block(px, py, pz, state);
                }
            }
            nodes.push((sx + j, sy + i, sz + k, -2, false));
            i -= 2 + random.next_int_bound(4);
        }
        nodes
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
/// random_selector（260905-06 idk-7 取证落地：RandomSelectorFeature.java L22-28，mojmap 一手源：
/// 逐项 nextFloat()<chance 即选即返；全落空走 default 不抽选择 RNG）
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
        // 260905-06：JSON codec 字段是 "default"（RandomFeatureConfiguration），旧码误读 "default_feature" 恒 None；
        // 两者都试（防历史数据双写形态）
        let default_feature = v.get("default").or_else(|| v.get("default_feature"))
            .and_then(|d| crate::placement::PlacedFeature::parse_inline(Some(d), blocks));
        Some(RandomSelectorConfig { features, default_feature })
    }
}

/// random_patch / flower（260905-06 idk-7 取证落地：RandomPatchFeature.java L15-33，yarn 一手源）
/// tries 来自 feature JSON 字段（非 placement Count）；xz/y spread 为 plain int（非 IntProvider）
#[derive(Clone)]
pub struct RandomPatchConfig {
    pub tries: i32,
    pub xz_spread: i32,
    pub y_spread: i32,
    pub feature: crate::placement::PlacedFeature,
}
impl RandomPatchConfig {
    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> Option<RandomPatchConfig> {
        let v = v?;
        Some(RandomPatchConfig {
            tries: v.get("tries").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            xz_spread: v.get("xz_spread").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            y_spread: v.get("y_spread").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
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
