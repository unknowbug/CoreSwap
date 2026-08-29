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

// 常见 tag 展开（server jar 权威，1.20.1）——按需补充。
// 数据驱动边界：tag 无独立数据源（不在 worldgen dir），故代码硬编码。跨版本时：
// 核对 server jar data/minecraft/tags/blocks/<tag>.json 更新展开（block 名经 blocks.id 解析，blocks.json 已数据驱动）。
pub fn expand_tag(blocks: &BlockRegistry, tag: &str, out: &mut Vec<BlockId>) {
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
            if let Some(rb) = cfg.get("requires_block_below") { sc.requires_block_below = rb.as_f64().map(|x| x != 0.0).unwrap_or(false); }
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
