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
                // Java TrapezoidIntProvider.get = ceil(lerp(nextBetween(0, plateau-1), min, max) + nextFloat())
                let f = if *plateau == 0 { 0 } else { r.next_int_bound(plateau + 1) };
                let g = b - a;
                let h = g - plateau;
                let i = g - 2 * h;
                let _ = i;
                // Java 精确：return this.min + Math.floor(lerp(random.nextInt(plateau+1), min, max) + nextFloat())
                let lerp_v = *a as f64 + (f as f64) / (*plateau as f64) * ((b - a) as f64);
                (lerp_v + r.next_float() as f64).floor() as i32
            }
            IntProvider::BiasedToBottom(a, b) => {
                let inner = r.next_int_bound(b - a + 1);
                r.next_int_bound(inner + a) // 近似（Java 更复杂）
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
    BlockPredicateFilter { is_fluid: bool, ids: Vec<i32> },
    SurfaceRelativeThreshold { heightmap_type: String, has_min: bool, has_max: bool, min_inclusive: i32, max_inclusive: i32 },
    NoiseBasedCount { max_count: i32, noise_name: String, scale: f64, count: IntProvider },
}

impl PlacementModifier {
    pub fn get_positions(&self, ctx: &FeaturePlacementContext, random: &mut ChunkRandom,
                         x: i32, y: i32, z: i32) -> Vec<[i32; 3]> {
        match self {
            PlacementModifier::Count(count) => {
                let n = count.get(random);
                (0..n).map(|_| [x, y, z]).collect()
            }
            PlacementModifier::RarityFilter(chance) => {
                if *chance <= 0 || random.next_int_bound(*chance) == 0 { vec![[x, y, z]] } else { vec![] }
            }
            PlacementModifier::Square => {
                vec![[x + random.next_int_bound(16), y, z + random.next_int_bound(16)]]
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
                if top <= ctx.min_y - 1 { return vec![]; } // Java k > bottomY（高度图无效）
                vec![[x, top, z]]
            }
            PlacementModifier::Biome => {
                // Java BiomePlacementModifier.getPositions：过滤 posToBiome.getBiome(pos) 在 features 集合内
                // C++ 简化：posToBiome 判定位置 biome——Java 内部用 biomeAt（chunk biome）
                // 简化：直接返回（biome 过滤由调用方预判）——Phase 3 先保留位置
                vec![[x, y, z]]
            }
            PlacementModifier::RandomOffset(ox, oy, oz) => {
                vec![[x + ox.get(random), y + oy.get(random), z + oz.get(random)]]
            }
            PlacementModifier::BlockPredicateFilter { is_fluid, ids } => {
                if ctx.block_at.is_none() { return vec![[x, y, z]]; } // 无法读世界——保留
                let cur = ctx.block_at.unwrap()(x, y, z);
                if cur < 0 { return vec![]; }
                for id in ids { if cur == *id { return vec![[x, y, z]]; } }
                let _ = is_fluid;
                vec![]
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
        } else if type_name.contains("block_predicate_filter") {
            if let Some(pred) = m.get("predicate") {
                let ptype = pred.get("predicate_type").and_then(|t| t.as_str()).unwrap_or("");
                if ptype.contains("matching_fluids") {
                    let mut ids = Vec::new();
                    if let Some(fluids) = pred.get("fluids") {
                        if let Some(arr) = fluids.as_array() {
                            for f in arr { if let Some(s) = f.as_str() { ids.push(blocks.id(s)); } }
                        }
                    }
                    return Some(PlacementModifier::BlockPredicateFilter { is_fluid: true, ids });
                } else if ptype.contains("matching_blocks") {
                    let mut ids = Vec::new();
                    if let Some(blocks_node) = pred.get("blocks") {
                        if let Some(arr) = blocks_node.as_array() {
                            for b in arr { if let Some(s) = b.as_str() { ids.push(blocks.id(s)); } }
                        }
                    }
                    return Some(PlacementModifier::BlockPredicateFilter { is_fluid: false, ids });
                }
            }
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
}
