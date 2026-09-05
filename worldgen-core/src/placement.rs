// placement.rs — FEATURES 阶段调度（MC 1.20.1 移植）
// 对应 C++: versions/1.20.1/cpp/worldgen/src/placement.h
// Java 参照：world/gen/placementmodifier/*.java + world/gen/feature/PlacedFeature.java
// 调度链：generateFeatures → set 3×3 biome → intSet 全局索引 → setDecoratorSeed(l,p,k)
//        → PlacedFeature.generate → placementModifiers flatMap 链 → ConfiguredFeature.generate
// 惰性语义：Java stream 惰性（第一个 pos 走完所有 modifier 再下一个）——Rust 按序展开

use crate::blocks::BlockRegistry;
use crate::chunkrandom::ChunkRandom;
use crate::json::JsonValue;

// ===== IntProvider（Java util/math/intprovider：uniform / constant / trapezoid / biased_to_bottom / weighted_list）=====
#[derive(Clone)]
pub enum IntProvider {
    Constant(i32),
    Uniform(i32, i32),          // [min, max]
    Trapezoid(i32, i32, i32),   // min, max, plateau
    BiasedToBottom(i32, i32),   // min, max
    WeightedList(Vec<(i32, i32)>, i32), // (data, weight), totalWeight
    Clamped(Box<IntProvider>, i32, i32), // source, min, max
}

impl IntProvider {
    pub fn get(&self, r: &mut ChunkRandom) -> i32 {
        match self {
            IntProvider::Constant(a) => *a,
            IntProvider::Uniform(a, b) => {
                if a >= b { return *a; }
                // Java UniformIntProvider.get = random.nextInt(max - min + 1) + min
                r.next_int_bound(b - a + 1) + a
            }
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
            IntProvider::WeightedList(weighted, total_weight) => {
                if weighted.is_empty() { return 0; }
                let mut i = r.next_int_bound(*total_weight);
                for (data, w) in weighted {
                    i -= w;
                    if i < 0 { return *data; }
                }
                weighted[0].0
            }
            IntProvider::Clamped(source, min, max) => {
                let v = source.get(r);
                if v < *min { *min } else if v > *max { *max } else { v }
            }
        }
    }

    pub fn parse(v: Option<&JsonValue>) -> IntProvider {
        let v = match v { Some(v) => v, None => return IntProvider::Constant(0) };
        if let Some(n) = v.as_f64() { return IntProvider::Constant(n as i32); }
        if v.as_object().is_none() { return IntProvider::Constant(0); }
        let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        // MC 1.20.1 的 uniform/trapezoid/biased_to_bottom 的 min/max 在 "value" 子对象里
        let val = v.get("value").unwrap_or(v);
        if type_name.contains("uniform") {
            IntProvider::Uniform(
                val.get("min_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                val.get("max_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            )
        } else if type_name.contains("trapezoid") {
            IntProvider::Trapezoid(
                val.get("min").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                val.get("max").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                val.get("plateau").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            )
        } else if type_name.contains("biased_to_bottom") {
            IntProvider::BiasedToBottom(
                val.get("min_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                val.get("max_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
            )
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
            // {"type":"minecraft:clamped","value":{...},"min_inclusive":X,"max_inclusive":Y}
            let min = v.get("min_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
            let max = v.get("max_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
            let src = v.get("value").map(|s| IntProvider::parse(Some(s))).unwrap_or(IntProvider::Constant(0));
            IntProvider::Clamped(Box::new(src), min, max)
        } else {
            IntProvider::Constant(0)
        }
    }
}

// ===== PlacementModifier 基类 =====
// getPositions(context, random, x, y, z) → 输出位置列表（Java stream 惰性，Rust 展开）
pub struct FeaturePlacementContext<'a> {
    // 回调：位置 biome 判定（Java FeaturePlacementContext.getBiome(BlockPos)——用 chunk biome 采样）
    pub biome_at: Option<&'a dyn Fn(i32, i32, i32) -> String>,
    // OCEAN_FLOOR_WG / WORLD_SURFACE_WG 高度图（[z*16+x]）
    pub ocean_floor: Option<&'a [i32]>,
    pub world_surface: Option<&'a [i32]>,
    pub min_y: i32,
    pub height: i32,
    // 邻域 biome 判定（biome modifier 用）——Java 用 posToBiome（BiomeAccess 8 邻域 jitter）
    pub pos_to_biome: Option<&'a dyn Fn(i32, i32, i32) -> String>,
    pub chunk_start_x: i32,
    pub chunk_start_z: i32,
    // 世界方块读取（block_predicate_filter 等用；null=不可读）
    pub block_at: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
    /// Fix-2（.b2b，Java BiomePlacementModifier.java:24-29）：允许集判定闭包
    /// (x, y, z, feature_id) -> jitter 点 biome 的 features 是否含 fid；None = 未接线直通
    pub biome_allows: Option<&'a dyn Fn(i32, i32, i32, &str) -> bool>,
    /// 当前 feature id（biome_allows 判定用）
    pub feature_id: Option<String>,
}

// ===== BlockPredicate（Java world/gen/blockpredicate/*，S1 idk-5）=====
#[derive(Clone, Debug)]
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
        // 260905-06：block predicate JSON 的类型字段是 "type"（Java BlockPredicateType codec），
        // 旧码误读 "predicate_type" → type_name 恒空 → 全部落 Unsupported → 树/patch 内层全灭
        let type_name = v.get("type").or_else(|| v.get("predicate_type"))
            .and_then(|t| t.as_str()).unwrap_or("").to_string();
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
        // 应用适配（patch 引用 `crate::constants::AIR_ID`——本仓库无 constants 模块，改用
        // blocks.rs 现成常量 AIR=0）；同时 ctx.block_at 为 Option<&dyn Fn> 不可直调，包一层闭包
        let air = crate::blocks::AIR;
        let block_at = |bx: i32, by: i32, bz: i32| -> i32 {
            ctx.block_at.map_or(-1, |f| f(bx, by, bz))
        };
        match self {
            BlockPredicate::AlwaysTrue => true,
            BlockPredicate::Unsupported { type_name } => { let _ = type_name; false }
            BlockPredicate::MatchingBlocks { offset, ids } => {
                let cur = block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && ids.contains(&cur)
            }
            BlockPredicate::MatchingFluids { offset, ids } => {
                let cur = block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && ids.contains(&cur)
            }
            BlockPredicate::WouldSurvive { offset, state_name, dirt_ids } => {
                // SaplingBlock 无重写 → PlantBlock.canPlaceAt（idk-5）：下方 ∈ dirt ∪ farmland
                let _ = state_name; // state 本体不参与 sapling 判定；非 PlantBlock 方块时需分派（登记 R-4）
                let below = block_at(x + offset[0], y + offset[1] - 1, z + offset[2]);
                below >= 0 && dirt_ids.contains(&below)
            }
            BlockPredicate::Solid { offset } => {
                let cur = block_at(x + offset[0], y + offset[1], z + offset[2]);
                cur >= 0 && cur != air // ⚠️ Java isSolid 近似（非空气即 solid，误差登记 §九）
            }
            BlockPredicate::Replaceable { offset } => {
                let cur = block_at(x + offset[0], y + offset[1], z + offset[2]);
                // REPLACEABLE tag 近似（air/植被/流体族）——与 tree::can_replace 同口径
                cur == air
            }
            BlockPredicate::Not(inner) => !inner.test(ctx, x, y, z),
            BlockPredicate::AllOf(preds) => preds.iter().all(|p| p.test(ctx, x, y, z)),
        }
    }
}

// PlacementModifier：返回输出位置（Java Stream<BlockPos>——惰性，Rust 展开为 Vec）
#[derive(Clone)]
pub enum PlacementModifier {
    Count(IntProvider),
    RarityFilter(i32),
    Square,
    HeightRange(crate::carver::HeightProvider),
    Heightmap(String),
    Biome,
    RandomOffset(IntProvider, IntProvider, IntProvider),
    // 260905-05 重构：谓词树（替代旧 { is_fluid, ids }）
    BlockPredicateFilter { predicate: BlockPredicate },
    SurfaceRelativeThreshold { heightmap_type: String, has_min: bool, has_max: bool, min_inclusive: i32, max_inclusive: i32 },
    NoiseBasedCount { max_count: i32, noise_name: String, scale: f64, count: IntProvider },
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
}

/// WG_TREEDIAG（260905-10 P2 逐树 RNG 打点，b1 §4 模板）：进程级读 env 一次，热路径零成本。
/// 默认关：显式判 env 存在（env_enabled 语义=默认开，勿套用——260905-09 纪律）。
pub fn treediag_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WG_TREEDIAG").is_ok())
}

impl PlacementModifier {
    pub fn get_positions(&self, ctx: &FeaturePlacementContext, random: &mut ChunkRandom,
                         x: i32, y: i32, z: i32) -> Vec<[i32; 3]> {
        match self {
            PlacementModifier::Count(count) => {
                let n = count.get(random);
                if treediag_enabled() { eprintln!("[CNT] {n}"); }
                (0..n).map(|_| [x, y, z]).collect()
            }
            PlacementModifier::RarityFilter(chance) => {
                if *chance <= 0 || random.next_int_bound(*chance) == 0 { vec![[x, y, z]] } else { vec![] }
            }
            PlacementModifier::Square => {
                let dx = random.next_int_bound(16);
                let dz = random.next_int_bound(16);
                if treediag_enabled() { eprintln!("[SQ] {},{}", x + dx, z + dz); }
                vec![[x + dx, y, z + dz]]
            }
            PlacementModifier::HeightRange(height) => {
                let ny = height.get(random, ctx.min_y, ctx.height);
                vec![[x, ny, z]]
            }
            PlacementModifier::Heightmap(heightmap_type) => {
                let hm = if heightmap_type.contains("OCEAN_FLOOR") { ctx.ocean_floor } else { ctx.world_surface };
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
                let lx = x - ctx.chunk_start_x;
                let lz = z - ctx.chunk_start_z;
                let top = if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 { hm[(lz * 16 + lx) as usize] } else { ctx.min_y - 1 };
                // 260905-06 off-by-one 修正（ChunkRegion.getTopY L422-425 一手源：sampleHeightmap + 1
                // = 最高实体方块上方第一个空气位；旧码返回实体方块本身 → 树/patch 全部落进地表被拒）
                let k = top + 1;
                if k <= ctx.min_y { return vec![]; } // Java k > bottomY（高度图无效）
                vec![[x, k, z]]
            }
            PlacementModifier::Biome => {
                // Fix-2（.b2b）：Java BiomePlacementModifier.java:24-29 = AbstractConditional——
                // 0 RNG 消费， BiomeAccess 8 邻域 jitter 采样 → 采样点 biome 的
                // GenerationSettings.isFeatureAllowed(placedFeature) 允许集判定。
                // 旧实现（260905-05 patch §3.8）= anchor_biome(中心 chunk biome) 4×4 对齐采样近似——
                // 无位置级 biome 门，边界/混合 chunk 过放（oak_leaves + 残差主体机制假设）。
                match (&ctx.biome_allows, &ctx.feature_id) {
                    (Some(allows), Some(fid)) => {
                        if allows(x, y, z, fid) { vec![[x, y, z]] } else { vec![] }
                    }
                    _ => vec![[x, y, z]], // 未接线时直通（保持防御姿态）
                }
            }
            PlacementModifier::RandomOffset(ox, oy, oz) => {
                vec![[x + ox.get(random), y + oy.get(random), z + oz.get(random)]]
            }
            // —— 260905-05 增补 ——
            PlacementModifier::SurfaceWaterDepthFilter(max_depth) => {
                // Java L28-32：OCEAN_FLOOR 与 WORLD_SURFACE 双高度图差 <= maxWaterDepth（0 随机消费）
                // （应用适配：patch 草稿曾带 max_depth==0 early-return，§3.3 注明为「错误保留项」，
                //  建议删掉只走精确路径——已按注删除）
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
            PlacementModifier::SurfaceRelativeThreshold { heightmap_type, has_min, has_max, min_inclusive, max_inclusive } => {
                let hm = if heightmap_type.contains("OCEAN_FLOOR") { ctx.ocean_floor } else { ctx.world_surface };
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
                let lx = x - ctx.chunk_start_x;
                let lz = z - ctx.chunk_start_z;
                if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 { return vec![[x, y, z]]; } // 邻域高度图缺失——保留
                let top = hm[(lz * 16 + lx) as usize];
                if *has_min && y < top + min_inclusive { return vec![]; }
                if *has_max && y > top + max_inclusive { return vec![]; }
                vec![[x, y, z]]
            }
            PlacementModifier::NoiseBasedCount { max_count, noise_name, scale, count } => {
                // Java：count + floor(noise(x*scale, 0, z*scale) * maxCount)
                let _ = (noise_name, scale);
                let noise = 0.0; // 需要 noise sampler——Phase 3 简化 0
                let n = (count.get(random) + (noise * *max_count as f64).floor() as i32).max(0);
                (0..n).map(|_| [x, y, z]).collect()
            }
        }
    }

    pub fn parse(m: &JsonValue, blocks: &BlockRegistry) -> Option<PlacementModifier> {
        let type_name = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if type_name.contains("count") && !type_name.contains("noise") {
            if let Some(c) = m.get("count") {
                return Some(PlacementModifier::Count(IntProvider::parse(Some(c))));
            }
        } else if type_name.contains("rarity_filter") {
            return Some(PlacementModifier::RarityFilter(m.get("chance").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32));
        } else if type_name.contains("in_square") {
            return Some(PlacementModifier::Square);
        } else if type_name.contains("height_range") {
            if let Some(hp) = m.get("height") {
                return Some(PlacementModifier::HeightRange(crate::carver::HeightProvider::parse(Some(hp))));
            }
        } else if type_name.contains("heightmap") {
            let t = m.get("heightmap").and_then(|x| x.as_str()).unwrap_or("WORLD_SURFACE_WG").to_string();
            return Some(PlacementModifier::Heightmap(t));
        } else if type_name.contains("biome") {
            return Some(PlacementModifier::Biome);
        } else if type_name.contains("random_offset") {
            let ox = m.get("xz_spread").map(|s| IntProvider::parse(Some(s))).unwrap_or(IntProvider::Constant(0));
            let oy = m.get("y_spread").map(|s| IntProvider::parse(Some(s))).unwrap_or(IntProvider::Constant(0));
            return Some(PlacementModifier::RandomOffset(ox, IntProvider::Constant(0), oy));
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
        } else if type_name.contains("surface_relative_threshold_filter") {
            let t = m.get("heightmap").and_then(|x| x.as_str()).unwrap_or("WORLD_SURFACE_WG").to_string();
            let min = m.get("min_inclusive").and_then(|x| x.as_f64());
            let max = m.get("max_inclusive").and_then(|x| x.as_f64());
            return Some(PlacementModifier::SurfaceRelativeThreshold {
                heightmap_type: t,
                has_min: min.is_some(), has_max: max.is_some(),
                min_inclusive: min.unwrap_or(0.0) as i32, max_inclusive: max.unwrap_or(0.0) as i32,
            });
        } else if type_name.contains("noise_based_count") {
            let mc = m.get("noise_to_count_ratio").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
            let n = m.get("noise").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let s = m.get("noise_factor").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let c = m.get("count").map(|x| IntProvider::parse(Some(x))).unwrap_or(IntProvider::Constant(0));
            return Some(PlacementModifier::NoiseBasedCount { max_count: mc, noise_name: n, scale: s, count: c });
        }
        // —— 尾部：未知 modifier 显式告警（b2 S2，消除静默丢弃）——
        eprintln!("[feature-loader] unknown placement modifier type: {type_name}");
        None
    }
}

// ===== PlacedFeature（Java PlacedFeature.java）=====
#[derive(Clone)]
pub struct PlacedFeature {
    pub id: String,                        // "minecraft:ore_granite_upper"
    pub modifiers: Vec<PlacementModifier>,
    pub configured_feature: String,        // 引用的 configured_feature id
    pub step: i32,                         // GenerationStep.Feature ordinal（biome features 列表索引）
    pub global_index: i32,                 // PlacedFeatureIndexer 全局索引（p）
}

impl PlacedFeature {
    // Java PlacedFeature.generate：Stream.of(pos) → 链式 flatMap（惰性、深度优先：位置逐个走完 modifiers）
    // 关键：Java 惰性 flatMap 是「位置1 走完所有 modifier → 位置2 走完所有 modifier」（深度优先）
    // Rust 若「modifier 全展开再下一个」= 广度优先 → 随机消费顺序不同 → height_range y 全错（granite 位置错）
    pub fn generate<F>(&self, ctx: &FeaturePlacementContext, random: &mut ChunkRandom,
                       origin_x: i32, origin_y: i32, origin_z: i32,
                       mut generate_configured: F) -> bool
    where F: FnMut(&FeaturePlacementContext, &mut ChunkRandom, i32, i32, i32) -> bool {
        let mut placed = false;
        // 用闭包递归（可捕获 FnMut），RefCell 存 placed 标志
        let placed_cell = std::cell::Cell::new(false);
        fn visit<F>(mi: usize, x: i32, y: i32, z: i32, pf: &PlacedFeature,
                    ctx: &FeaturePlacementContext, random: &mut ChunkRandom,
                    generate_configured: &mut F, placed: &std::cell::Cell<bool>)
        where F: FnMut(&FeaturePlacementContext, &mut ChunkRandom, i32, i32, i32) -> bool {
            if mi == pf.modifiers.len() {
                if generate_configured(ctx, random, x, y, z) { placed.set(true); }
                return;
            }
            let out = pf.modifiers[mi].get_positions(ctx, random, x, y, z);
            for p in out { visit(mi + 1, p[0], p[1], p[2], pf, ctx, random, generate_configured, placed); }
        }
        visit(0, origin_x, origin_y, origin_z, self, ctx, random, &mut generate_configured, &placed_cell);
        placed_cell.get()
    }

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
