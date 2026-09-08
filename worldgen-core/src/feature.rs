// feature.rs — FEATURES 阶段 Feature 类（MC 1.20.1 移植）
// 对应 C++: versions/1.20.1/cpp/worldgen/src/feature.h
// Java 参照：world/gen/feature/OreFeature.java / ScatteredOreFeature.java / OreFeatureConfig.java
//            structure/rule/RuleTest.java / Feature.java（isExposedToAir）
//            DiskFeature.java / SpringFeature.java / FreezeTopLayerFeature.java / UnderwaterMagmaFeature.java
// 关键语义：
//   - OreFeature.generate 用 Math.sin/Math.cos（标准库，非查表！）；MathHelper.sin（查表，ds 权重）
//   - OCEAN_FLOOR_WG 高度图（NOISE 阶段 SUFFOCATES=blocksMovement 判定）
//   - chunkSectionCache 读方块（邻域 chunk 可能未生成——Java 用 ChunkSectionCache 惰性生成）
//     Rust 简化：只处理当前 chunk 内（邻域未生成无法读）

use crate::blocks::{BlockColumn, BlockId, BlockRegistry};
use crate::carver::math_sin;
use crate::chunkrandom::ChunkRandom;
use crate::json::JsonValue;
use crate::placement::IntProvider;

// ===== RuleTest（Java structure/rule/RuleTest.java）=====
#[derive(Clone)]
pub enum RuleTest {
    AlwaysTrue,
    BlockMatch(BlockId),
    TagMatch(Vec<BlockId>),
    RandomBlockMatch { probability: f32, block_ids: Vec<BlockId> },
}

impl RuleTest {
    pub fn test(&self, _blocks: &BlockRegistry, block_id: BlockId, random: &mut ChunkRandom) -> bool {
        match self {
            RuleTest::AlwaysTrue => true,
            RuleTest::BlockMatch(id) => block_id == *id,
            RuleTest::TagMatch(ids) => ids.iter().any(|&id| id == block_id),
            RuleTest::RandomBlockMatch { probability, block_ids } => {
                if random.next_float() >= *probability { return false; }
                block_ids.iter().any(|&id| id == block_id)
            }
        }
    }

    pub fn parse(v: Option<&JsonValue>, blocks: &BlockRegistry) -> RuleTest {
        let v = match v { Some(v) => v, None => return RuleTest::AlwaysTrue };
        let type_name = v.get("predicate_type").and_then(|t| t.as_str()).unwrap_or("");
        if type_name.contains("tag_match") {
            let tag = v.get("tag").and_then(|t| t.as_str()).unwrap_or("");
            let tag = if tag.starts_with('#') { &tag[1..] } else { tag };
            let mut ids = Vec::new();
            expand_tag(blocks, tag, &mut ids);
            RuleTest::TagMatch(ids)
        } else if type_name.contains("block_match") {
            let name = v.get("block").and_then(|b| b.as_str()).unwrap_or("");
            RuleTest::BlockMatch(blocks.id(name))
        } else if type_name.contains("random_block_match") {
            let probability = v.get("probability").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
            let name = v.get("block").and_then(|b| b.as_str()).unwrap_or("");
            RuleTest::RandomBlockMatch { probability, block_ids: vec![blocks.id(name)] }
        } else {
            RuleTest::AlwaysTrue
        }
    }
}

// tag 展开（260907-04 数据驱动化）：优先 block_tags JSON（<wg_dir>/data/<ns>/tags/blocks/），
// 缺失/畸形 → fallback 硬编码展开（server jar 1.20.1 权威值，golden 测试保证两者等值）。
pub fn expand_tag(blocks: &BlockRegistry, tag: &str, out: &mut Vec<BlockId>) {
    if crate::block_tags::expand_tag(tag, blocks, out) { return; }
    expand_tag_fallback(blocks, tag, out);
}

// 硬编码 fallback（server jar 权威，1.20.1）——按需补充。
// 注意：跨版本升级时 JSON 数据文件为主，本表为兜底 + golden 基准（数据缺失不炸生成）。
pub fn expand_tag_fallback(blocks: &BlockRegistry, tag: &str, out: &mut Vec<BlockId>) {
    let mut add = |n: &str| out.push(blocks.id(n));
    match tag {
        "minecraft:base_stone_overworld" => {
            add("minecraft:stone"); add("minecraft:granite"); add("minecraft:diorite");
            add("minecraft:andesite"); add("minecraft:tuff"); add("minecraft:deepslate");
        }
        "minecraft:stone_ore_replaceables" => {
            add("minecraft:stone"); add("minecraft:granite"); add("minecraft:diorite");
            add("minecraft:andesite");
        }
        "minecraft:deepslate_ore_replaceables" => {
            add("minecraft:deepslate"); add("minecraft:tuff");
        }
        "minecraft:netherrack" => { add("minecraft:netherrack"); }
        "minecraft:base_stone_nether" => {
            add("minecraft:netherrack"); add("minecraft:basalt"); add("minecraft:blackstone");
        }
        "minecraft:sand" => {
            add("minecraft:sand"); add("minecraft:red_sand"); add("minecraft:suspicious_sand");
        }
        "minecraft:dirt" => {
            add("minecraft:dirt"); add("minecraft:grass_block"); add("minecraft:podzol");
            add("minecraft:coarse_dirt"); add("minecraft:mycelium"); add("minecraft:rooted_dirt");
            add("minecraft:moss_block"); add("minecraft:mud"); add("minecraft:muddy_mangrove_roots");
        }
        _ => {}
    }
}

// ===== OreFeatureConfig（Java OreFeatureConfig.java）=====
#[derive(Clone)]
pub struct OreFeatureConfig {
    pub targets: Vec<OreTarget>,
    pub size: i32,
    pub discard_on_air_chance: f32,
}
#[derive(Clone)]
pub struct OreTarget {
    pub target: RuleTest,
    pub state: BlockId,
}

impl OreFeatureConfig {
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> OreFeatureConfig {
        let mut oc = OreFeatureConfig { targets: Vec::new(), size: 0, discard_on_air_chance: 0.0 };
        if let Some(cfg) = cfg {
            if let Some(s) = cfg.get("size") { oc.size = s.as_f64().unwrap_or(0.0) as i32; }
            if let Some(d) = cfg.get("discard_chance_on_air_exposure") { oc.discard_on_air_chance = d.as_f64().unwrap_or(0.0) as f32; }
            if let Some(targets) = cfg.get("targets") {
                if let Some(arr) = targets.as_array() {
                    for t in arr {
                        let mut tg = OreTarget { target: RuleTest::AlwaysTrue, state: 0 };
                        if let Some(state) = t.get("state") {
                            let name = state.get("Name").and_then(|x| x.as_str()).unwrap_or("");
                            tg.state = blocks.id(name);
                        }
                        if let Some(target) = t.get("target") { tg.target = RuleTest::parse(Some(target), blocks); }
                        oc.targets.push(tg);
                    }
                }
            }
        }
        oc
    }
}

// ===== OreFeatureContext（C++ 版 FeatureContext + StructureWorldAccess 简化）=====
// 注意：random 不存这里（避免与 PlacedFeature.generate 的 &mut random 双重借用），
// 由各 generate 函数作为参数传入。
pub struct OreFeatureContext<'a> {
    pub col: &'a mut BlockColumn,
    pub origin_x: i32, pub origin_y: i32, pub origin_z: i32, // 放置起点（placementModifiers 输出，world 坐标）
    pub chunk_start_x: i32, pub chunk_start_z: i32, // 当前 chunk 起点（world 坐标）
    pub min_y: i32, pub height: i32,
    pub blocks: &'a BlockRegistry,
    // OCEAN_FLOOR_WG 高度图 [z*16+x]（NOISE 阶段构建）
    pub ocean_floor: Option<&'a [i32]>,
    // WORLD_SURFACE_WG 高度图 [z*16+x]（FreezeTopLayer 用）
    pub world_surface: Option<&'a [i32]>,
    // 两阶段 FEATURE 跨 chunk：region_col_at(cx,cz) 返回区域 col（None=不在区域）
    pub region_col_at: Option<&'a dyn Fn(i32, i32) -> Option<&'a [i32]>>,
    // pending 跨 chunk 写入（Java 语义：A 后生成覆盖 B）——回调 (chunkX, chunkZ, 块索引, state)
    pub pending_cross: Option<&'a dyn Fn(i32, i32, i32, i32)>,
    // c-A-min（260905-09）：任意点读钩子（含邻 chunk 地形列；与 FeaturePlacementContext.block_at 同源闭包）。
    // None = 旧语义（region_col_at / -1）。优先级高于 region_col_at。
    pub block_at_ext: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
    // batchA（mc-1216）：世界 seed（geode 噪声采样器 = ChunkRandom(CheckedRandom(worldSeed))，
    // GeodeFeature.java:41）。构造点全仓 2 处：worldgen_handle.rs / bin-diag/b5b6_smoke.rs（同批 patch）。
    pub world_seed: i64,
}

impl<'a> OreFeatureContext<'a> {
    // world → col 局部；越界返回 -1（Java world.isOutOfHeightLimit / isValidForSetBlock）
    pub fn local_idx(&self, wx: i32, wy: i32, wz: i32) -> i32 {
        let lx = wx - self.chunk_start_x;
        let lz = wz - self.chunk_start_z;
        if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 { return -1; }
        if wy < self.min_y || wy >= self.min_y + self.height { return -1; }
        (wy - self.min_y) * 256 + lz * 16 + lx
    }
    pub fn block_at(&self, wx: i32, wy: i32, wz: i32) -> i32 {
        let idx = self.local_idx(wx, wy, wz);
        if idx >= 0 { return self.col.at(wx - self.chunk_start_x, wy, wz - self.chunk_start_z); }
        // c-A-min：任意点读钩子（邻 chunk 地形列缓存路由）
        if let Some(f) = self.block_at_ext {
            return f(wx, wy, wz);
        }
        // 跨 chunk 读（两阶段）
        if let Some(region_col_at) = self.region_col_at {
            let cx = wx >> 4;
            let cz = wz >> 4;
            if let Some(rc) = region_col_at(cx, cz) {
                if wy >= self.min_y && wy < self.min_y + self.height {
                    return rc[((wy - self.min_y) * 256 + (wz & 15) * 16 + (wx & 15)) as usize];
                }
            }
        }
        -1
    }
    // getTopY(OCEAN_FLOOR_WG, x, z)——NOISE 阶段高度图
    pub fn get_ocean_floor_top_y(&self, wx: i32, wz: i32) -> i32 {
        let ocean_floor = match self.ocean_floor { Some(o) => o, None => return self.min_y - 1 };
        let lx = wx - self.chunk_start_x;
        let lz = wz - self.chunk_start_z;
        if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 { return self.min_y - 1; }
        ocean_floor[(lz * 16 + lx) as usize]
    }
    // getTopY(WORLD_SURFACE_WG, x, z)——FreezeTopLayer 用（海面冻结）
    pub fn get_world_surface_top_y(&self, wx: i32, wz: i32) -> i32 {
        let ws = match self.world_surface { Some(o) => o, None => return self.min_y - 1 };
        let lx = wx - self.chunk_start_x;
        let lz = wz - self.chunk_start_z;
        if lx < 0 || lx >= 16 || lz < 0 || lz >= 16 { return self.min_y - 1; }
        ws[(lz * 16 + lx) as usize]
    }
    // 放置（当前 chunk 或跨 chunk：记录 pending，阶段 2 末尾统一应用——Java A 后生成覆盖 B）
    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, state: i32) {
        let lx = wx - self.chunk_start_x;
        let lz = wz - self.chunk_start_z;
        if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && wy >= self.min_y && wy < self.min_y + self.height {
            *self.col.at_mut(lx, wy, lz) = state;
            return;
        }
        if let Some(pending_cross) = self.pending_cross {
            if wy >= self.min_y && wy < self.min_y + self.height {
                let cx = wx >> 4;
                let cz = wz >> 4;
                pending_cross(cx, cz, (wy - self.min_y) * 256 + (wz & 15) * 16 + (wx & 15), state);
            }
        }
    }
}

// ===== OreFeature（Java OreFeature.java）=====
pub struct OreFeature;
impl OreFeature {
    // Java generate：random.nextFloat()*π → 端点；if (o <= getTopY(OCEAN_FLOOR_WG, s, t)) generateVeinPart
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &OreFeatureConfig, random: &mut ChunkRandom) -> bool {
        let x = ctx.origin_x; let y = ctx.origin_y; let z = ctx.origin_z;
        let f = random.next_float() * 3.14159265358979323846f32; // Java Math.PI（double→float 参数）
        let g = config.size as f32 / 8.0;
        let i = ((config.size as f32 / 16.0 * 2.0 + 1.0) / 2.0).ceil() as i32;
        let d = x as f64 + (f as f64).sin() * g as f64;      // Java Math.sin（标准库！）
        let e = x as f64 - (f as f64).sin() * g as f64;
        let h = z as f64 + (f as f64).cos() * g as f64;
        let j = z as f64 - (f as f64).cos() * g as f64;
        let k = 2;
        let l = y as f64 + random.next_int_bound(3) as f64 - 2.0;
        let m = y as f64 + random.next_int_bound(3) as f64 - 2.0;
        let n = x - g.ceil() as i32 - i;
        let o = y - 2 - i;
        let p = z - g.ceil() as i32 - i;
        let q = 2 * (g.ceil() as i32 + i);
        let r = 2 * (2 + i);

        for s in n..=n + q {
            for t in p..=p + q {
                if o <= ctx.get_ocean_floor_top_y(s, t) {
                    return self.generate_vein_part(ctx, config, random, d, e, h, j, l, m, n, o, p, q, r);
                }
            }
        }
        false
    }

    // Java generateVeinPart（L55-166）
    fn generate_vein_part(&self, ctx: &mut OreFeatureContext, config: &OreFeatureConfig, random: &mut ChunkRandom,
                          start_x: f64, end_x: f64, start_z: f64, end_z: f64,
                          start_y: f64, end_y: f64, x: i32, y: i32, z: i32,
                          horizontal_size: i32, vertical_size: i32) -> bool {
        let mut i = 0;
        let j = config.size;
        let mut bit_set = vec![0u64; ((horizontal_size * vertical_size * horizontal_size) as usize + 63) / 64];
        let mut ds = vec![0.0f64; (j * 4) as usize];

        for k in 0..j {
            let f = k as f32 / j as f32;
            let d = lerp(f as f64, start_x, end_x);
            let e = lerp(f as f64, start_y, end_y);
            let g = lerp(f as f64, start_z, end_z);
            let h = random.next_double() * j as f64 / 16.0;
            let l = ((math_sin((3.14159265358979323846 * f as f64) as f32) + 1.0f32) as f64 * h + 1.0) / 2.0; // MathHelper.sin 查表
            ds[(k * 4 + 0) as usize] = d;
            ds[(k * 4 + 1) as usize] = e;
            ds[(k * 4 + 2) as usize] = g;
            ds[(k * 4 + 3) as usize] = l;
        }

        for k in 0..j - 1 {
            if !(ds[(k * 4 + 3) as usize] <= 0.0) {
                for m in k + 1..j {
                    if !(ds[(m * 4 + 3) as usize] <= 0.0) {
                        let d = ds[(k * 4 + 0) as usize] - ds[(m * 4 + 0) as usize];
                        let e = ds[(k * 4 + 1) as usize] - ds[(m * 4 + 1) as usize];
                        let g = ds[(k * 4 + 2) as usize] - ds[(m * 4 + 2) as usize];
                        let h = ds[(k * 4 + 3) as usize] - ds[(m * 4 + 3) as usize];
                        if h * h > d * d + e * e + g * g {
                            if h > 0.0 { ds[(m * 4 + 3) as usize] = -1.0; }
                            else { ds[(k * 4 + 3) as usize] = -1.0; }
                        }
                    }
                }
            }
        }
        for mx in 0..j {
            let d = ds[(mx * 4 + 3) as usize];
            if d < 0.0 { continue; }
            let e = ds[(mx * 4 + 0) as usize];
            let g = ds[(mx * 4 + 1) as usize];
            let h = ds[(mx * 4 + 2) as usize];
            let n = (e - d).floor() as i32;
            let n = n.max(x);
            let o = (g - d).floor() as i32;
            let o = o.max(y);
            let p = (h - d).floor() as i32;
            let p = p.max(z);
            let q = (e + d).floor() as i32;
            let q = q.max(n);
            let r = (g + d).floor() as i32;
            let r = r.max(o);
            let s = (h + d).floor() as i32;
            let s = s.max(p);
            for t in n..=q {
                let u = (t as f64 + 0.5 - e) / d;
                if u * u < 1.0 {
                    for v in o..=r {
                        let w = (v as f64 + 0.5 - g) / d;
                        if u * u + w * w < 1.0 {
                            for aa in p..=s {
                                let ab = (aa as f64 + 0.5 - h) / d;
                                if u * u + w * w + ab * ab < 1.0 {
                                    let ac = t - x + (v - y) * horizontal_size + (aa - z) * horizontal_size * vertical_size;
                                    if ac < 0 { continue; }
                                    if !(bit_set[(ac as usize) / 64] >> (ac % 64) & 1 == 1) {
                                        bit_set[(ac as usize) / 64] |= 1u64 << (ac % 64);
                                        // world.isValidForSetBlock + ChunkSection 读写（Rust：col 局部 + 跨 chunk regionCols）
                                        if v >= ctx.min_y && v < ctx.min_y + ctx.height {
                                            let state = ctx.block_at(t, v, aa);
                                            for target in &config.targets {
                                                if should_place(ctx, config, target, state, t, v, aa, random) {
                                                    ctx.set_block(t, v, aa, target.state);
                                                    i += 1;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        i > 0
    }
}

fn should_place(ctx: &mut OreFeatureContext, config: &OreFeatureConfig,
                target: &OreTarget, state: i32, x: i32, y: i32, z: i32, random: &mut ChunkRandom) -> bool {
    if !target.target.test(ctx.blocks, state, random) { return false; }
    if should_not_discard(random, config.discard_on_air_chance) { return true; }
    !is_exposed_to_air(ctx, x, y, z)
}

// Java Feature.isExposedToAir：6 邻居任一 isAir
fn is_exposed_to_air(ctx: &OreFeatureContext, x: i32, y: i32, z: i32) -> bool {
    const DX: [i32; 6] = [1, -1, 0, 0, 0, 0];
    const DY: [i32; 6] = [0, 0, 1, -1, 0, 0];
    const DZ: [i32; 6] = [0, 0, 0, 0, 1, -1];
    for i in 0..6 {
        let nx = x + DX[i]; let ny = y + DY[i]; let nz = z + DZ[i];
        let idx = ctx.local_idx(nx, ny, nz);
        let id = if idx >= 0 { ctx.col.at(nx - ctx.chunk_start_x, ny, nz - ctx.chunk_start_z) } else { -1 };
        if id == 0 { return true; } // air
    }
    false
}

// Java OreFeature.shouldNotDiscard
fn should_not_discard(random: &mut ChunkRandom, chance: f32) -> bool {
    if chance <= 0.0 { return true; }
    if chance >= 1.0 { return false; }
    random.next_float() >= chance
}

// MathHelper.lerp(double delta, double start, double end) = start + delta * (end - start)
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

// ===== ScatteredOreFeature（Java ScatteredOreFeature.java）=====
pub struct ScatteredOreFeature;
impl ScatteredOreFeature {
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &OreFeatureConfig, random: &mut ChunkRandom) -> bool {
        let i = random.next_int_bound(config.size + 1);
        for j in 0..i {
            let lx = get_spread(random, j.min(7));
            let ly = get_spread(random, j.min(7));
            let lz = get_spread(random, j.min(7));
            let wx = ctx.origin_x + lx;
            let wy = ctx.origin_y + ly;
            let wz = ctx.origin_z + lz;
            if wy < ctx.min_y || wy >= ctx.min_y + ctx.height { continue; }
            let state = ctx.block_at(wx, wy, wz);
            for target in &config.targets {
                if should_place(ctx, config, target, state, wx, wy, wz, random) {
                    ctx.set_block(wx, wy, wz, target.state);
                    break;
                }
            }
        }
        true
    }
}

// Java getSpread = Math.round((nextFloat()-nextFloat()) * spread)
fn get_spread(random: &mut ChunkRandom, spread: i32) -> i32 {
    ((random.next_float() - random.next_float()) * spread as f32).round() as i32
}

// ===== DiskFeatureConfig（Java DiskFeatureConfig.java）=====
#[derive(Clone)]
pub struct DiskFeatureConfig {
    pub half_height: i32,
    pub radius: IntProvider,            // uniform(2,6) 等
    pub state: i32,                     // state_provider fallback（简化：simple_state_provider）
    pub targets: Vec<i32>,              // target matching_blocks（多块）或 tag
}

impl DiskFeatureConfig {
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> DiskFeatureConfig {
        let mut dc = DiskFeatureConfig { half_height: 0, radius: IntProvider::Constant(0), state: 0, targets: Vec::new() };
        if let Some(cfg) = cfg {
            if let Some(h) = cfg.get("half_height") { dc.half_height = h.as_f64().unwrap_or(0.0) as i32; }
            if let Some(r) = cfg.get("radius") { dc.radius = IntProvider::parse(Some(r)); }
            // state_provider：取 fallback（simple_state_provider）
            if let Some(sp) = cfg.get("state_provider") {
                if let Some(fb) = sp.get("fallback") {
                    if let Some(st) = fb.get("state") {
                        dc.state = blocks.id(st.get("Name").and_then(|x| x.as_str()).unwrap_or(""));
                    }
                }
            }
            // target：matching_blocks（数组或字符串）
            if let Some(t) = cfg.get("target") {
                if let Some(blk) = t.get("blocks") {
                    if let Some(arr) = blk.as_array() {
                        for b in arr { if let Some(s) = b.as_str() { dc.targets.push(blocks.id(s)); } }
                    } else if let Some(name) = blk.as_str() {
                        if name.starts_with('#') { expand_tag(blocks, &name[1..], &mut dc.targets); }
                        else { dc.targets.push(blocks.id(name)); }
                    }
                }
            }
        }
        dc
    }
}

// ===== DiskFeature（Java DiskFeature.java）=====
pub struct DiskFeature;
impl DiskFeature {
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &DiskFeatureConfig, random: &mut ChunkRandom) -> bool {
        let y = ctx.origin_y;
        let top_y = y + config.half_height;
        let bottom_y = y - config.half_height - 1;
        let radius = config.radius.get(random);
        let mut placed = false;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let mx = dx * dx + dz * dz;
                if mx > radius * radius { continue; }
                let wx = ctx.origin_x + dx;
                let wz = ctx.origin_z + dz;
                for iy in (bottom_y + 1..=top_y).rev() {
                    if target_matches(ctx, config, wx, iy, wz) {
                        ctx.set_block(wx, iy, wz, config.state);
                        placed = true;
                        break;
                    }
                }
            }
        }
        placed
    }
}

fn target_matches(ctx: &OreFeatureContext, config: &DiskFeatureConfig, x: i32, y: i32, z: i32) -> bool {
    let cur = ctx.block_at(x, y, z);
    if cur < 0 { return false; }
    config.targets.iter().any(|&id| cur == id)
}

// ===== SpringFeatureConfig（Java SpringFeatureConfig.java）=====
#[derive(Clone)]
pub struct SpringFeatureConfig {
    pub state: i32,                 // 简化：固定块（water/lava）
    pub valid_blocks: Vec<i32>,      // 数组或 tag
    pub rock_count: i32,
    pub hole_count: i32,
    pub requires_block_below: bool,
}

impl SpringFeatureConfig {
    pub fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> SpringFeatureConfig {
        let mut sc = SpringFeatureConfig { state: 0, valid_blocks: Vec::new(), rock_count: 0, hole_count: 0, requires_block_below: false };
        if let Some(cfg) = cfg {
            if let Some(st) = cfg.get("state") {
                sc.state = blocks.id(st.get("Name").and_then(|x| x.as_str()).unwrap_or(""));
            }
            if let Some(vb) = cfg.get("valid_blocks") {
                if let Some(arr) = vb.as_array() {
                    for b in arr {
                        if let Some(name) = b.as_str() {
                            if name.starts_with('#') { expand_tag(blocks, &name[1..], &mut sc.valid_blocks); }
                            else { sc.valid_blocks.push(blocks.id(name)); }
                        }
                    }
                }
            }
            if let Some(rc) = cfg.get("rock_count") { sc.rock_count = rc.as_f64().unwrap_or(0.0) as i32; }
            if let Some(hc) = cfg.get("hole_count") { sc.hole_count = hc.as_f64().unwrap_or(0.0) as i32; }
            if let Some(rb) = cfg.get("requires_block_below") { sc.requires_block_below = rb.as_bool().unwrap_or(false); }
        }
        sc
    }
}

// ===== SpringFeature（Java SpringFeature.java）=====
pub struct SpringFeature;
impl SpringFeature {
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &SpringFeatureConfig, _random: &mut ChunkRandom) -> bool {
        let x = ctx.origin_x; let y = ctx.origin_y; let z = ctx.origin_z;
        if !is_valid(ctx, config, x, y + 1, z) { return false; }                    // up 必须 valid
        if config.requires_block_below && !is_valid(ctx, config, x, y - 1, z) { return false; } // down 必须 valid
        let cur = ctx.block_at(x, y, z);
        if cur < 0 { return false; }
        let mut cur_ok = cur == ctx.blocks.id("minecraft:air");
        if !cur_ok { for &id in &config.valid_blocks { if cur == id { cur_ok = true; break; } } }
        if !cur_ok { return false; }
        // 统计 5 邻（东西南北下）valid（rockCount）与 air（holeCount）
        let mut rock = 0;
        let mut hole = 0;
        const DX: [i32; 5] = [-1, 1, 0, 0, 0];
        const DZ: [i32; 5] = [0, 0, -1, 1, 0];
        const DY: [i32; 5] = [0, 0, 0, 0, -1];
        for i in 0..5 {
            let nx = x + DX[i]; let ny = y + DY[i]; let nz = z + DZ[i];
            if is_valid(ctx, config, nx, ny, nz) { rock += 1; }
            if ctx.block_at(nx, ny, nz) == ctx.blocks.id("minecraft:air") { hole += 1; }
        }
        if rock == config.rock_count && hole == config.hole_count {
            ctx.set_block(x, y, z, config.state);
            return true;
        }
        false
    }
}

fn is_valid(ctx: &OreFeatureContext, config: &SpringFeatureConfig, x: i32, y: i32, z: i32) -> bool {
    let cur = ctx.block_at(x, y, z);
    if cur < 0 { return false; }
    config.valid_blocks.iter().any(|&id| cur == id)
}

// ===== FreezeTopLayerFeature（Java FreezeTopLayerFeature.java）=====
// 简化：MOTION_BLOCKING 用 C++ 高度图（buildSurface 的 topY）；canSetIce/canSetSnow 用 biome 温度+降水
pub struct FreezeTopLayerFeature;
impl FreezeTopLayerFeature {
    // 需要 biome 温度（<0 且降水 SNOW 才冻结）——C++ BiomeEntry 简化：temperature/rainfall 参数
    pub fn generate(&self, ctx: &mut OreFeatureContext, biome_temp: f32, _biome_rainfall: f32, _random: &mut ChunkRandom) -> bool {
        let snowy = biome_temp < 0.0; // canSetSnow 主条件（precipitation==SNOW 由温度近似）
        if !snowy { return true; }       // -288 冷洋不冻结——直接返回
        let ice_id = ctx.blocks.id("minecraft:ice");
        let snow_id = ctx.blocks.id("minecraft:snow_block");
        for lx in 0..16 {
            for lz in 0..16 {
                let wx = ctx.chunk_start_x + lx;
                let wz = ctx.chunk_start_z + lz;
                // 用 WORLD_SURFACE_WG（海面）而非 OCEAN_FLOOR_WG（海底）——vanilla FreezeTopLayer 冻结海面
                let top_y = ctx.get_world_surface_top_y(wx, wz);
                if top_y < ctx.min_y { continue; }
                // canSetIce(world, top.down())：top.down 是水/空气且可放冰
                ctx.set_block(wx, top_y - 1, wz, ice_id);
                // canSetSnow(world, top)
                ctx.set_block(wx, top_y, wz, snow_id);
            }
        }
        true
    }
}

// ===== UnderwaterMagmaFeatureConfig（Java UnderwaterMagmaFeatureConfig.java）=====
#[derive(Clone)]
pub struct UnderwaterMagmaFeatureConfig {
    pub floor_search_range: i32,
    pub placement_probability_per_valid_position: f32,
    pub placement_radius_around_floor: i32,
}

impl UnderwaterMagmaFeatureConfig {
    pub fn parse(cfg: Option<&JsonValue>, _blocks: &BlockRegistry) -> UnderwaterMagmaFeatureConfig {
        let mut uc = UnderwaterMagmaFeatureConfig { floor_search_range: 0, placement_probability_per_valid_position: 0.0, placement_radius_around_floor: 0 };
        if let Some(cfg) = cfg {
            if let Some(f) = cfg.get("floor_search_range") { uc.floor_search_range = f.as_f64().unwrap_or(0.0) as i32; }
            if let Some(p) = cfg.get("placement_probability_per_valid_position") { uc.placement_probability_per_valid_position = p.as_f64().unwrap_or(0.0) as f32; }
            if let Some(r) = cfg.get("placement_radius_around_floor") { uc.placement_radius_around_floor = r.as_f64().unwrap_or(0.0) as i32; }
        }
        uc
    }
}

// ===== UnderwaterMagmaFeature（Java UnderwaterMagmaFeature.java + CaveSurface.java）=====
// Java 语义（1.20.1 精确）：
//   - CaveSurface.create(world, origin, floorSearchRange, water, !water)：origin 必须 water；
//     沿列向上/下找水柱边界（canGenerate=water 继续，canReplace=!water 停止）→ floor/ceiling
//   - blockPos2 = origin.withY(floor)；box = blockPos2 ± placementRadiusAroundFloor（3×3×3）
//   - Box.stream（x 内层 z 中层 y 外层）→ filter(nextFloat < prob) → filter(isValidPosition)
//   - isValidPosition：pos 与 pos.down 都非 water/air；4 水平邻都非 water/air（全石头包围）
pub struct UnderwaterMagmaFeature;
impl UnderwaterMagmaFeature {
    pub fn generate(&self, ctx: &mut OreFeatureContext, config: &UnderwaterMagmaFeatureConfig, random: &mut ChunkRandom) -> bool {
        let x = ctx.origin_x; let y = ctx.origin_y; let z = ctx.origin_z;
        let water_id = ctx.blocks.id("minecraft:water");
        let air_id = ctx.blocks.id("minecraft:air");
        let magma_id = ctx.blocks.id("minecraft:magma_block");
        // CaveSurface.create：origin 必须 water（canGenerate）
        if ctx.block_at(x, y, z) != water_id { return false; }
        // 向上找 ceiling、向下找 floor（canGenerate=water 继续移动，停在第一个非 water）
        let mut ceiling_y = -1;
        let mut floor_y = -1;
        let mut my = y;
        let mut i = 1;
        while i < config.floor_search_range && ctx.block_at(x, my, z) == water_id { my += 1; i += 1; }
        if ctx.block_at(x, my, z) != water_id { ceiling_y = my; }
        my = y;
        i = 1;
        while i < config.floor_search_range && ctx.block_at(x, my, z) == water_id { my -= 1; i += 1; }
        if ctx.block_at(x, my, z) != water_id { floor_y = my; }
        if ceiling_y < 0 || floor_y < 0 { return false; } // Java create(floor, ceiling) 需两个都 present（Bounded）
        let mut placed = 0;
        // Box.stream 顺序：x 内层、z 中层、y 外层（BlockPos.BlockPosIterator）
        for dy in -config.placement_radius_around_floor..=config.placement_radius_around_floor {
            for dz in -config.placement_radius_around_floor..=config.placement_radius_around_floor {
                for dx in -config.placement_radius_around_floor..=config.placement_radius_around_floor {
                    let px = x + dx;
                    let py = floor_y + dy;
                    let pz = z + dz;
                    if random.next_float() < config.placement_probability_per_valid_position {
                        if is_valid_position(ctx, px, py, pz, water_id, air_id) {
                            ctx.set_block(px, py, pz, magma_id);
                            placed += 1;
                        }
                    }
                }
            }
        }
        placed > 0
    }
}

fn is_valid_position(ctx: &OreFeatureContext, x: i32, y: i32, z: i32, water_id: i32, air_id: i32) -> bool {
    if is_water_or_air(ctx, x, y, z, water_id, air_id) { return false; }
    if is_water_or_air(ctx, x, y - 1, z, water_id, air_id) { return false; }
    const DX: [i32; 4] = [-1, 1, 0, 0];
    const DZ: [i32; 4] = [0, 0, -1, 1];
    for i in 0..4 {
        if is_water_or_air(ctx, x + DX[i], y, z + DZ[i], water_id, air_id) { return false; }
    }
    true
}

fn is_water_or_air(ctx: &OreFeatureContext, x: i32, y: i32, z: i32, water_id: i32, air_id: i32) -> bool {
    let cur = ctx.block_at(x, y, z);
    if cur < 0 { return true; } // 越界视为 water/air（Java chunkSectionCache null → 默认 air）
    cur == water_id || cur == air_id
}

// ===== batchA（mc-1216）：monster_room / lake / geode / multiface_growth =====
// 一手源：versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/feature/
//   DungeonFeature.java / LakeFeature.java / GeodeFeature.java(+Config/Layer/Crack/LayerThickness) /
//   MultifaceGrowthFeature.java(+Config)
// 对拍表 + RNG 消费序声明：.investigations/mc-1216-features-takeover/batchA-worker-delivery.md §一
// 已知限制（全族）：state=i32 无属性位（facing/waterlogged/level/faces 不可表达）；
// 方块实体/流体 tick/后处理调度 no-op；isSolid 系判定走 tree::is_solid_id 近似（B5/B6 同族）。

/// Java Feature.setBlockStateIf（Feature.java:144-148）：当前 state 不在 cannot tag 内才放。
/// 不可读（-1）→ 保守跳过（Java 读邻 chunk 实况，残差声明 idk-3）。
fn set_block_if_allowed(ctx: &mut OreFeatureContext, x: i32, y: i32, z: i32, state: i32, cannot: &[i32]) {
    let cur = ctx.block_at(x, y, z);
    if cur >= 0 && !cannot.contains(&cur) {
        ctx.set_block(x, y, z, state);
    }
}

/// #minecraft:features_cannot_replace 展开（数据驱动，block_tags；缺失时 fallback 硬编码为空 →
/// 谓词恒过 = 过放风险，与 expand_tag_fallback 口径一致，声明）。
fn features_cannot_replace_ids(blocks: &BlockRegistry) -> Vec<i32> {
    let mut ids = Vec::new();
    expand_tag(blocks, "minecraft:features_cannot_replace", &mut ids);
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
        // 建造（:70-90）：t 从 3 降到 -1（顶面 t=4 不进建造循环）
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
    /// biome_temp：当前 chunk biome 温度（canSetIce 近似，Java 按列 biome——声明）。
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
                            bls[((l * 16 + m) * 8 + n) as usize] = true;                             // :53
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
        // block_at 不可读（-1）→ 非实心且 ≠fluid → return false（保守 abort；idk-3）
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
                        let b2 = ctx.block_at(x + s as i32, by + u as i32, z + t as i32);
                        if u >= 4 && is_liquid(b2) { return false; }                      // :76-78
                        if u < 4 && !crate::tree::is_solid_id(ctx, b2) && b2 != fluid { return false; } // :80-82
                    }
                }
            }
        }
        // 放置 pass（:88-104）：u≥4 = CAVE_AIR，u<4 = fluid
        let cave_air = blocks.id("minecraft:cave_air");
        for s in 0..16 {
            for t in 0..16 {
                for u in 0..8 {
                    if bls[(s * 16 + t) * 8 + u] && can_replace(ctx, x + s as i32, by + u as i32, z + t as i32) {
                        let st = if u >= 4 { cave_air } else { fluid };                   // :94-95
                        ctx.set_block(x + s as i32, by + u as i32, z + t as i32, st);
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
                            let b4 = ctx.block_at(x + t as i32, by + v as i32, z + u as i32);
                            if crate::tree::is_solid_id(ctx, b4)
                                && !lava_pool_stone_cannot_replace(ctx.blocks).contains(&b4) { // :122
                                ctx.set_block(x + t as i32, by + v as i32, z + u as i32, barrier);
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
                    let (bx, byy, bz) = (x + t as i32, by + 4, z + u as i32);
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
    expand_tag(blocks, "minecraft:lava_pool_stone_cannot_replace", &mut ids);
    ids
}

// ===== batchB（mc-1216）：kelp / seagrass / sea_pickle =====
// 一手源：versions/1.20.1 + versions/1.21.6 双版 mc_src_extract（两版逐行一致，批次 B §一.1 版本核对）
// 对拍表 + RNG 消费序声明：.investigations/mc-1216-features-takeover/batchB-worker-delivery.md §一
// 已知限制（本族）：state=i32 无属性位（KELP.AGE / TALL_SEAGRASS.HALF / SEA_PICKLE.PICKLES 不可表达，
// 对应 RNG 消费全部保留）；isSideSolidFullSquare/碰撞形状 → tree::is_solid_id 近似；
// 方块实体/流体 tick no-op（同 batchA 全族声明）。

/// 特征级 getTopY(Heightmap.Type.OCEAN_FLOOR, x, z)。chunk 内 = ocean_floor 桶 +1（引擎口径）；
/// 邻域列 → block_at 逐列下扫近似（skip {air,water,lava}）；不可读 → None（调用点「跳过但保 RNG 流」，idk-2）。
fn get_top_y_ocean_floor(ctx: &OreFeatureContext, wx: i32, wz: i32) -> Option<i32> {
    let air = crate::blocks::AIR;
    let water = ctx.blocks.id("minecraft:water");
    let lava = ctx.blocks.id("minecraft:lava");
    let lx = wx - ctx.chunk_start_x;
    let lz = wz - ctx.chunk_start_z;
    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 {
        let hm = ctx.ocean_floor?;
        let top = hm[(lz * 16 + lx) as usize];
        return Some(top + 1); // ChunkRegion.getTopY = sampleHeightmap + 1（placement.rs 同源）
    }
    // 邻域：自顶下扫首个非 {air,water,lava}（≈SUFFOCATES），返回其上一位
    let mut y = ctx.min_y + ctx.height - 1;
    while y >= ctx.min_y {
        let b = ctx.block_at(wx, y, wz);
        if b < 0 { return None; }
        if b != air && b != water && b != lava { return Some(y + 1); }
        y -= 1;
    }
    None
}

/// kelp 族 canPlaceAt（AbstractPlantPartBlock.java:38-43 + KelpBlock.canAttachTo = !magma）。
/// 不可读（<0）→ false 保守拒绝。
fn kelp_can_place_at(ctx: &OreFeatureContext, x: i32, y: i32, z: i32,
                     kelp: i32, kelp_plant: i32, magma: i32) -> bool {
    let below = ctx.block_at(x, y - 1, z);
    if below < 0 || below == magma { return false; }
    below == kelp || below == kelp_plant || crate::tree::is_solid_id(ctx, below)
}

// ===== minecraft:kelp（KelpFeature.java，DefaultFeatureConfig 空配置）=====
pub struct KelpFeature;
impl KelpFeature {
    /// RNG 消费序（§一.1）：首格非水 0 消费 false → nextInt(10) → 循环门 0 消费；
    /// l==k 放置 nextInt(4)（AGE，属性丢弃消费保留）；l>0 断点命中 nextInt(4) 后恒 break。
    pub fn generate(ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let kelp = blocks.id("minecraft:kelp");
        let kelp_plant = blocks.id("minecraft:kelp_plant");
        let magma = blocks.id("minecraft:magma_block");
        let Some(mut py) = get_top_y_ocean_floor(ctx, x, z) else { return false; };
        if ctx.block_at(x, py, z) != water { return false; }                      // :27
        let k = 1 + random.next_int_bound(10);                                    // :30
        let mut placed = false;
        for l in 0..=k {                                                          // :32（含端点）
            if ctx.block_at(x, py, z) == water
                && ctx.block_at(x, py + 1, z) == water
                && kelp_can_place_at(ctx, x, py, z, kelp, kelp_plant, magma) {    // :33-35
                if l == k {
                    let _age = random.next_int_bound(4);                          // :37 AGE=nextInt(4)+20（丢弃）
                    ctx.set_block(x, py, z, kelp);
                    placed = true;
                } else {
                    ctx.set_block(x, py, z, kelp_plant);                          // :40
                }
            } else if l > 0 {
                let by = py - 1;                                                  // :43 down()
                if kelp_can_place_at(ctx, x, by, z, kelp, kelp_plant, magma)      // :44
                    && ctx.block_at(x, by - 1, z) != kelp {                       // :44 下下格非 KELP
                    let _age = random.next_int_bound(4);                          // :45
                    ctx.set_block(x, by, z, kelp);
                    placed = true;
                }
                break;                                                            // :48 恒 break
            }
            py += 1;                                                              // :51（break 分支不达）
        }
        placed                                                                    // :55 i > 0
    }
}

// ===== minecraft:seagrass（SeagrassFeature.java + ProbabilityConfig）=====
#[derive(Clone)]
pub struct SeagrassConfig {
    pub probability: f32,
}
impl SeagrassConfig {
    /// ProbabilityConfig.java:8-16：probability 单字段。E-1：无 provider 字段。
    pub fn parse(cfg: Option<&JsonValue>, _blocks: &BlockRegistry) -> Option<SeagrassConfig> {
        let p = cfg?.get("probability")?.as_f64()?;
        Some(SeagrassConfig { probability: p as f32 })
    }
}
pub struct SeagrassFeature;
impl SeagrassFeature {
    /// RNG 消费序（§一.2）：nextInt(8)×4 → 水（0）→ nextDouble（仅水时）→ 0。
    /// 双高海草 HALF 属性位丢失：两格均放 tall_seagrass 裸 id。
    pub fn generate(ctx: &mut OreFeatureContext, config: &SeagrassConfig,
                    random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let seagrass = blocks.id("minecraft:seagrass");
        let tall_seagrass = blocks.id("minecraft:tall_seagrass");
        let magma = blocks.id("minecraft:magma_block");
        let i = random.next_int_bound(8) - random.next_int_bound(8);              // :28
        let j = random.next_int_bound(8) - random.next_int_bound(8);              // :29
        let (px, pz) = (x + i, z + j);
        let Some(py) = get_top_y_ocean_floor(ctx, px, pz) else { return false; }; // :30
        if ctx.block_at(px, py, pz) != water { return false; }                    // :32
        let tall = random.next_double() < config.probability as f64;              // :33
        // SeagrassBlock.canPlantOnTop：isSideSolidFullSquare(UP) ∧ !magma
        let below = ctx.block_at(px, py - 1, pz);
        if !(below >= 0 && below != magma && crate::tree::is_solid_id(ctx, below)) {
            return false;                                                         // :35
        }
        if tall {                                                                 // :36-42
            if ctx.block_at(px, py + 1, pz) == water {                            // :39 上半水检
                ctx.set_block(px, py, pz, tall_seagrass);                         // :40 HALF=lower（丢弃）
                ctx.set_block(px, py + 1, pz, tall_seagrass);                     // :41 HALF=upper（丢失）
            }
            // 上半非水：两格都不放，但 bl 已置位（Java :47 位置在 canPlaceAt 后）
        } else {
            ctx.set_block(px, py, pz, seagrass);                                  // :44
        }
        true                                                                      // :47/:51
    }
}

// ===== minecraft:sea_pickle（SeaPickleFeature.java + CountConfig=IntProvider）=====
#[derive(Clone)]
pub struct SeaPickleConfig {
    pub count: crate::placement::IntProvider,
}
impl SeaPickleConfig {
    /// CountConfig.java:9-25：count = IntProvider（validating 0..=256）。E-1：非「1..25 非 IntProvider」。
    pub fn parse(cfg: Option<&JsonValue>, _blocks: &BlockRegistry) -> Option<SeaPickleConfig> {
        let c = cfg?.get("count")?;
        Some(SeaPickleConfig { count: crate::placement::IntProvider::parse(Some(c)) })
    }
}
pub struct SeaPickleFeature;
impl SeaPickleFeature {
    /// RNG 消费序（§一.3）：count.get 一次 → 每轮恒 nextInt(8)×2 → nextInt(4)（PICKLES，
    /// 水门之前恒消费）→ 检查 0 消费。邻域高度图不可读：nextInt(4) 已先消费（保流，idk-2）。
    pub fn generate(ctx: &mut OreFeatureContext, config: &SeaPickleConfig,
                    random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let sea_pickle = blocks.id("minecraft:sea_pickle");
        let n = config.count.get(random);                                         // :26
        let mut placed = 0i32;
        for _ in 0..n {                                                           // :28
            let i = random.next_int_bound(8) - random.next_int_bound(8);          // :29
            let j = random.next_int_bound(8) - random.next_int_bound(8);          // :30
            let py_opt = get_top_y_ocean_floor(ctx, x + i, z + j);                // :31
            let _pickles = random.next_int_bound(4);                              // :33 PICKLES（恒消费）
            let Some(py) = py_opt else { continue; };
            if ctx.block_at(x + i, py, z + j) != water { continue; }              // :34 前半
            // SeaPickleBlock.canPlantOnTop：碰撞上面非空 ∨ 侧满方（无 magma 排除）
            let below = ctx.block_at(x + i, py - 1, z + j);
            if !(below >= 0 && crate::tree::is_solid_id(ctx, below)) { continue; } // :34 后半（形状近似声明）
            ctx.set_block(x + i, py, z + j, sea_pickle);                          // :35
            placed += 1;
        }
        placed > 0                                                                // :40
    }
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
                expand_tag(blocks, t, &mut ids);
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
            invalid_blocks_threshold: cfg.get("invalid_blocks_threshold").and_then(|x| x.as_f64()).unwrap_or(1.0) as i32,
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
        let noise = crate::noise::DoublePerlinNoiseSampler::new_legacy(&mut noise_rnd, -4, &[1.0]);
        let mut crack_points: Vec<[i32; 3]> = Vec::new();                                 // :43
        let d = k as f64 / self.outer_wall_distance.max_value() as f64;                   // :44
        let e = 1.0 / self.thickness.filling.sqrt();                                      // :48
        let f = 1.0 / (self.thickness.inner + d).sqrt();                                  // :49
        let g = 1.0 / (self.thickness.middle + d).sqrt();                                 // :50
        let h = 1.0 / (self.thickness.outer + d).sqrt();                                  // :51
        let l = 1.0 / (self.crack.base_crack_size + random.next_double() / 2.0
            + if k > 3 { d } else { 0.0 }).sqrt();                                        // :52（nextDouble 无条件）
        let bl = (random.next_float() as f64) < self.crack.generate_crack_chance;           // :53
        let mut points: Vec<([i32; 3], i32)> = Vec::new();
        let mut invalid = 0;
        for _ in 0..k {
            let o = self.outer_wall_distance.get(random);                                 // :57
            let p = self.outer_wall_distance.get(random);                                 // :58
            let q = self.outer_wall_distance.get(random);                                 // :59
            let pos = [x + o, y + p, z + q];
            let st = ctx.block_at(pos[0], pos[1], pos[2]);
            if st == crate::blocks::AIR || self.layer.invalid_blocks.contains(&st) {      // :62（isAir ≈ id==AIR；cave_air 残差 idk-1）
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
                        let use_alt = (random.next_float() as f64) < self.use_alternate_layer0_chance; // :123
                        let st = if use_alt {
                            self.layer.alternate_inner_layer_provider.get(random)         // :125
                        } else {
                            self.layer.inner_layer_provider.get(random)                   // :127
                        };
                        set_block_if_allowed(ctx, px, py, pz, st, &self.layer.cannot_replace);
                        if (!self.placements_require_layer0_alternate || use_alt)
                            && ((random.next_float() as f64) < self.use_potential_placements_chance) { // :130
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
        // placements（:143-162）：每 placement 恒 1 次 Util.getRandom(innerBlocks)（保 RNG 流）
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
                        expand_tag(blocks, tag, &mut can_place_on);
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

/// Util.copyShuffled（Util.java:1184，同 tree.rs 对拍）：j 从 n 降到 2，k=nextInt(j)，swap(k, j-1)。
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
///（:69 setBlockState(pos, ...)；faces 属性丢失，统一裸 id，声明）。
/// 命中后 nextFloat 恒消费（:71）。grow() 未实装（nextFloat 消费保留，残差声明）。
fn generate_multiface_at(ctx: &mut OreFeatureContext, cfg: &MultifaceGrowthConfig,
                         random: &mut ChunkRandom, x: i32, y: i32, z: i32, dirs: &[[i32; 3]]) -> bool {
    for d in dirs {
        let support = ctx.block_at(x + d[0], y + d[1], z + d[2]);
        if support >= 0 && cfg.can_place_on.contains(&support) {                          // :63
            ctx.set_block(x, y, z, cfg.block);                                            // :69
            if random.next_float() < cfg.spread_chance {
                // :72 config.block.getGrower().grow(...) 未实装（nextFloat 消费已保留，残差）
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
    /// 按一手源逐字对拍；1.20.1 差异疑点 idk-4）。
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


