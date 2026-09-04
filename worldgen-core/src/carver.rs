// carver.rs — CARVERS 阶段（洞穴雕刻）MC 1.20.1 移植
// 对应 C++: versions/1.20.1/cpp/worldgen/src/carver.h
// 对应 Java: carver/Carver.java / CaveCarver.java / RavineCarver.java / CarvingMask.java /
//            CarverContext.java / CarverConfig.java / CaveCarverConfig.java / RavineCarverConfig.java
// 语义要点（逐位对齐）：
//   - ChunkRandom（CheckedRandom LCG 基类）setCarverSeed(seed+l, cx, cz)；内部递归用 Random.create(seed)=Xoroshiro
//   - CarvingMask index = (x&15)|(z&15)<<4|(y-bottomY)<<8；get/set 按位
//   - carveRegion：g=(s+0.5-x)/width, h=(u+0.5-z)/width；y 从 o 递减到 m+1，w=(v-0.5-y)/height
//   - getState：y<=lavaLevel.getY(minY+8=-56) → lava；否则 aquifer.apply(pos, 0.0)
//   - replaceable tag：overworld_carver_replaceables（含 water！）
//   - applyMaterialRule（grass 被挖后 dirt 替换）：SurfaceContext.initVertical(1,1,fluidHeight) + rule.apply

use crate::blocks::{BlockColumn, BlockId, BlockRegistry};
use crate::chunkrandom::{CheckedRandom, ChunkRandom};
use crate::json::JsonValue;

// ===== MathHelper.sin/cos（MC 65536 项 SINE_TABLE 查表，非 std::sin！carve 漂移逐位对齐关键）=====
fn sine_table() -> &'static [f32; 65536] {
    use std::sync::OnceLock;
    static TABLE: OnceLock<[f32; 65536]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0.0f32; 65536];
        for i in 0..65536 {
            t[i] = (i as f64 * 3.14159265358979323846 * 2.0 / 65536.0).sin() as f32;
        }
        t
    })
}
pub fn math_sin(value: f32) -> f32 {
    let table = sine_table();
    table[((value * 10430.378) as i32) as usize & 65535]
}
pub fn math_cos(value: f32) -> f32 {
    let table = sine_table();
    table[((value * 10430.378 + 16384.0) as i32) as usize & 65535]
}

// ===== CarvingMask（Java CarvingMask.java：BitSet 256*height）=====
pub struct CarvingMask {
    bottom_y: i32,
    bits: Vec<u64>,
}
impl CarvingMask {
    pub fn new(height: i32, bottom_y: i32) -> Self {
        CarvingMask { bottom_y, bits: vec![0; ((256 * height + 63) / 64) as usize] }
    }
    fn get_index(&self, offset_x: i32, y: i32, offset_z: i32) -> usize {
        ((offset_x & 15) | ((offset_z & 15) << 4) | ((y - self.bottom_y) << 8)) as usize
    }
    pub fn get(&self, offset_x: i32, y: i32, offset_z: i32) -> bool {
        let idx = self.get_index(offset_x, y, offset_z);
        (self.bits[idx / 64] >> (idx % 64)) & 1 == 1
    }
    pub fn set(&mut self, offset_x: i32, y: i32, offset_z: i32) {
        let idx = self.get_index(offset_x, y, offset_z);
        self.bits[idx / 64] |= 1u64 << (idx % 64);
    }
}

// ===== providers（YOffset / HeightProvider / FloatProvider 最小移植）=====
#[derive(Clone, Copy)]
pub enum YOffsetKind { Fixed, AboveBottom, BelowTop }
#[derive(Clone, Copy)]
pub struct YOffset { pub kind: YOffsetKind, pub value: i32 }
impl YOffset {
    pub fn get_y(&self, min_y: i32, height: i32) -> i32 {
        match self.kind {
            YOffsetKind::Fixed => self.value,
            YOffsetKind::AboveBottom => min_y + self.value,
            YOffsetKind::BelowTop => min_y + height - 1 - self.value,
        }
    }
    pub fn parse(v: Option<&JsonValue>) -> YOffset {
        let mut y = YOffset { kind: YOffsetKind::Fixed, value: 0 };
        if let Some(v) = v {
            if let Some(a) = v.get("absolute") { y.kind = YOffsetKind::Fixed; y.value = a.as_f64().unwrap_or(0.0) as i32; }
            else if let Some(a) = v.get("above_bottom") { y.kind = YOffsetKind::AboveBottom; y.value = a.as_f64().unwrap_or(0.0) as i32; }
            else if let Some(a) = v.get("below_top") { y.kind = YOffsetKind::BelowTop; y.value = a.as_f64().unwrap_or(0.0) as i32; }
        }
        y
    }
}

// HeightProvider：uniform（min/max YOffset）——carver 用（Java UniformHeightProvider）
#[derive(Clone, Copy)]
pub struct HeightProvider { pub min_offset: YOffset, pub max_offset: YOffset }
impl HeightProvider {
    pub fn get(&self, r: &mut ChunkRandom, min_y: i32, height: i32) -> i32 {
        let i = self.min_offset.get_y(min_y, height);
        let j = self.max_offset.get_y(min_y, height);
        if i == j { return i; }
        // MathHelper.nextBetween(random, i, j) = random.nextInt(j-i+1) + i（ChunkRandom CHECKED 基类）
        r.next_int_bound(j - i + 1) + i
    }
    pub fn parse(v: Option<&JsonValue>) -> HeightProvider {
        let mut hp = HeightProvider { min_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 }, max_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 } };
        if let Some(v) = v {
            if v.as_object().is_some() && v.get("min_inclusive").is_some() && v.get("max_inclusive").is_some() {
                hp.min_offset = YOffset::parse(v.get("min_inclusive"));
                hp.max_offset = YOffset::parse(v.get("max_inclusive"));
            }
        }
        hp
    }
}

// FloatProvider：uniform / trapezoid / constant（Java FloatProvider 体系）
#[derive(Clone, Copy)]
enum FloatKind { Uniform, Trapezoid, Constant }
#[derive(Clone, Copy)]
struct FloatProvider { kind: FloatKind, a: f32, b: f32, plateau: f32 }
impl FloatProvider {
    fn get(&self, r: &mut ChunkRandom) -> f32 {
        match self.kind {
            FloatKind::Uniform => r.next_float() * (self.b - self.a) + self.a,
            FloatKind::Trapezoid => {
                let f = self.b - self.a;
                let g = (f - self.plateau) / 2.0;
                let h = f - g;
                self.a + r.next_float() * h + r.next_float() * g
            }
            FloatKind::Constant => self.a,
        }
    }
    // 重载：CheckedRandom（carveTunnels/carveRavine 内部 Random.create(seed) = CheckedRandom LCG！）
    fn get_checked(&self, r: &mut CheckedRandom) -> f32 {
        match self.kind {
            FloatKind::Uniform => r.next_float() * (self.b - self.a) + self.a,
            FloatKind::Trapezoid => {
                let f = self.b - self.a;
                let g = (f - self.plateau) / 2.0;
                let h = f - g;
                self.a + r.next_float() * h + r.next_float() * g
            }
            FloatKind::Constant => self.a,
        }
    }
    fn parse(v: Option<&JsonValue>) -> FloatProvider {
        let mut fp = FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 };
        if let Some(v) = v {
            if let Some(n) = v.as_f64() { fp.kind = FloatKind::Constant; fp.a = n as f32; return fp; }
            if v.as_object().is_none() { return fp; }
            let type_name = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let val = v.get("value");
            if type_name.contains("uniform") && val.is_some() {
                fp.kind = FloatKind::Uniform;
                fp.a = val.and_then(|x| x.get("min_inclusive")).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                fp.b = val.and_then(|x| x.get("max_exclusive")).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
            } else if type_name.contains("trapezoid") && val.is_some() {
                fp.kind = FloatKind::Trapezoid;
                fp.a = val.and_then(|x| x.get("min")).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                fp.b = val.and_then(|x| x.get("max")).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                fp.plateau = val.and_then(|x| x.get("plateau")).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
            }
        }
        fp
    }
}

// ===== Config（CarverConfig / CaveCarverConfig / RavineCarverConfig）=====
#[derive(Clone)]
pub struct CarverConfig {
    pub probability: f32,
    pub y: HeightProvider,
    pub y_scale: FloatProvider,
    pub lava_level: YOffset,          // above_bottom(8) → minY+8 = -56
    pub replaceable_ids: Vec<BlockId>, // #minecraft:overworld_carver_replaceables 展开（含 water！）
}

impl CarverConfig {
    // #minecraft:overworld_carver_replaceables tag 的 1.20.1 展开（carver 可挖掉的方块）。
    // 数据驱动边界：该 tag 无独立数据源（不在 worldgen dir），故代码硬编码。跨版本时只改此处：
    // 升到新版本若 tag 内容变，核对 server jar 的 data/minecraft/tags/blocks/overworld_carver_replaceables.json
    // 更新 NAMES（block 名从 blocks.id 解析，blocks.json 已数据驱动）。
    pub fn build_overworld_replaceable(blocks: &BlockRegistry) -> Vec<BlockId> {
        // server jar 权威 tag 展开（1.20.1）：
        // base_stone_overworld + dirt + sand + terracotta + iron_ores + copper_ores + 直接值
        const NAMES: &[&str] = &[
            // base_stone_overworld
            "minecraft:stone","minecraft:granite","minecraft:diorite","minecraft:andesite",
            "minecraft:tuff","minecraft:deepslate",
            // dirt
            "minecraft:dirt","minecraft:grass_block","minecraft:podzol","minecraft:coarse_dirt",
            "minecraft:mycelium","minecraft:rooted_dirt","minecraft:moss_block","minecraft:mud",
            "minecraft:muddy_mangrove_roots",
            // sand
            "minecraft:sand","minecraft:red_sand","minecraft:suspicious_sand",
            // terracotta（17 色 + 无前缀）
            "minecraft:terracotta","minecraft:white_terracotta","minecraft:orange_terracotta",
            "minecraft:magenta_terracotta","minecraft:light_blue_terracotta","minecraft:yellow_terracotta",
            "minecraft:lime_terracotta","minecraft:pink_terracotta","minecraft:gray_terracotta",
            "minecraft:light_gray_terracotta","minecraft:cyan_terracotta","minecraft:purple_terracotta",
            "minecraft:blue_terracotta","minecraft:brown_terracotta","minecraft:green_terracotta",
            "minecraft:red_terracotta","minecraft:black_terracotta",
            // iron_ores / copper_ores
            "minecraft:iron_ore","minecraft:deepslate_iron_ore",
            "minecraft:copper_ore","minecraft:deepslate_copper_ore",
            // 直接值
            "minecraft:water","minecraft:gravel","minecraft:suspicious_gravel","minecraft:sandstone",
            "minecraft:red_sandstone","minecraft:calcite","minecraft:snow","minecraft:packed_ice",
            "minecraft:raw_iron_block","minecraft:raw_copper_block",
        ];
        NAMES.iter().map(|n| blocks.id(n)).collect()
    }

    fn parse_common(&mut self, cfg: Option<&JsonValue>, blocks: &BlockRegistry) {
        if let Some(cfg) = cfg {
            if let Some(p) = cfg.get("probability") { self.probability = p.as_f64().unwrap_or(0.0) as f32; }
            if let Some(yv) = cfg.get("y") { self.y = HeightProvider::parse(Some(yv)); }
            if let Some(ys) = cfg.get("yScale") { self.y_scale = FloatProvider::parse(Some(ys)); }
            if let Some(ll) = cfg.get("lava_level") { self.lava_level = YOffset::parse(Some(ll)); }
        }
        self.replaceable_ids = Self::build_overworld_replaceable(blocks);
    }
}

#[derive(Clone)]
pub struct CaveCarverConfig {
    pub common: CarverConfig,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,   // validated [-1,1]
}
impl CaveCarverConfig {
    fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> CaveCarverConfig {
        let mut c = CaveCarverConfig {
            common: CarverConfig { probability: 0.0, y: HeightProvider { min_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 }, max_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 } }, y_scale: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 }, lava_level: YOffset { kind: YOffsetKind::Fixed, value: 0 }, replaceable_ids: vec![] },
            horizontal_radius_multiplier: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
            vertical_radius_multiplier: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
            floor_level: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
        };
        c.common.parse_common(cfg, blocks);
        if let Some(cfg) = cfg {
            if let Some(v) = cfg.get("horizontal_radius_multiplier") { c.horizontal_radius_multiplier = FloatProvider::parse(Some(v)); }
            if let Some(v) = cfg.get("vertical_radius_multiplier") { c.vertical_radius_multiplier = FloatProvider::parse(Some(v)); }
            if let Some(v) = cfg.get("floor_level") { c.floor_level = FloatProvider::parse(Some(v)); }
        }
        c
    }
}

#[derive(Clone)]
pub struct RavineCarverConfig {
    pub common: CarverConfig,
    pub vertical_rotation: FloatProvider,
    pub shape: RavineShape,
}
#[derive(Clone)]
pub struct RavineShape {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub horizontal_radius_factor: FloatProvider,
    pub width_smoothness: i32,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
}
impl RavineCarverConfig {
    fn parse(cfg: Option<&JsonValue>, blocks: &BlockRegistry) -> RavineCarverConfig {
        let mut c = RavineCarverConfig {
            common: CarverConfig { probability: 0.0, y: HeightProvider { min_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 }, max_offset: YOffset { kind: YOffsetKind::Fixed, value: 0 } }, y_scale: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 }, lava_level: YOffset { kind: YOffsetKind::Fixed, value: 0 }, replaceable_ids: vec![] },
            vertical_rotation: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
            shape: RavineShape {
                distance_factor: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
                thickness: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
                horizontal_radius_factor: FloatProvider { kind: FloatKind::Constant, a: 0.0, b: 0.0, plateau: 0.0 },
                width_smoothness: 0,
                vertical_radius_default_factor: 1.0,
                vertical_radius_center_factor: 0.0,
            },
        };
        c.common.parse_common(cfg, blocks);
        if let Some(cfg) = cfg {
            if let Some(v) = cfg.get("vertical_rotation") { c.vertical_rotation = FloatProvider::parse(Some(v)); }
            if let Some(s) = cfg.get("shape") {
                if let Some(v) = s.get("distance_factor") { c.shape.distance_factor = FloatProvider::parse(Some(v)); }
                if let Some(v) = s.get("thickness") { c.shape.thickness = FloatProvider::parse(Some(v)); }
                if let Some(v) = s.get("horizontal_radius_factor") { c.shape.horizontal_radius_factor = FloatProvider::parse(Some(v)); }
                if let Some(v) = s.get("width_smoothness") { c.shape.width_smoothness = v.as_f64().unwrap_or(0.0) as i32; }
                if let Some(v) = s.get("vertical_radius_default_factor") { c.shape.vertical_radius_default_factor = v.as_f64().unwrap_or(1.0) as f32; }
                if let Some(v) = s.get("vertical_radius_center_factor") { c.shape.vertical_radius_center_factor = v.as_f64().unwrap_or(0.0) as f32; }
            }
        }
        c
    }
}

// ===== CarverContext（Java CarverContext.java：HeightContext + applyMaterialRule）=====
pub struct CarverContext<'a> {
    pub min_y: i32,
    pub height: i32,
    pub aquifer: &'a mut crate::aquifer::Aquifer,
    pub blocks: &'a BlockRegistry,
    // surface 单点应用：构造 SurfaceContext + initVertical(1,1,fluidHeight,x,y,z,biome) + overworldRule->apply
    // hasFluid ? j+1 : INT32_MIN；返回 block id 或 None（Java applyMaterialRule 语义）
    pub apply_material_rule: Option<&'a dyn Fn(i32, i32, i32, bool) -> Option<BlockId>>,
}

// ===== Carver 基类（Java Carver.java）=====
pub struct Carver {
    pub target_chunk_x: i32,
    pub target_chunk_z: i32, // 当前 chunk（carveRegion 写方块目标 = chunk.getPos()）
}

impl Carver {
    pub fn new() -> Self { Carver { target_chunk_x: 0, target_chunk_z: 0 } }

    pub fn should_carve(&self, random: &mut ChunkRandom, probability: f32) -> bool {
        random.next_float() <= probability
    }

    pub fn get_branch_factor() -> i32 { 4 }
    // ChunkSectionPos.getBlockCoord(getBranchFactor()*2-1) = (4*2-1)*16 = 112
    pub fn branch_coord() -> i32 { (Self::get_branch_factor() * 2 - 1) * 16 }

    // Java Carver.carveRegion（x/y/z double 中心，width/height 半径，chunk 坐标由调用方传入）
    fn carve_region_impl(
        &self,
        ctx: &mut CarverContext,
        config: &CarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        x: f64, y: f64, z: f64, width: f64, height: f64,
        mask: &mut CarvingMask,
        skip_predicate: &dyn Fn(f64, f64, f64, i32) -> bool,
    ) -> bool {
        let f = 16.0 + width * 2.0;
        let cx = self.target_chunk_x as f64 * 16.0 + 8.0;   // chunkPos.getCenterX()
        let cz = self.target_chunk_z as f64 * 16.0 + 8.0;
        if (x - cx).abs() > f || (z - cz).abs() > f { return false; }
        // 洞穴中心 x/y/z 用邻域 chunk；carveRegion 写方块用 targetChunkX/Z（当前）
        let i2 = self.target_chunk_x * 16;      // 当前 chunk getStartX
        let j2 = self.target_chunk_z * 16;
        let k = ((x - width).floor() as i32) - i2 - 1;
        let k = k.max(0);
        let l = (((x + width).floor() as i32) - i2).min(15);
        let m = (((y - height).floor() as i32) - 1).max(ctx.min_y + 1);
        let n = 7;                       // hasBelowZeroRetrogen=false
        let o = (((y + height).floor() as i32) + 1).min(ctx.min_y + ctx.height - 1 - n);
        let p = (((z - width).floor() as i32) - j2 - 1).max(0);
        let q = (((z + width).floor() as i32) - j2).min(15);
        let mut bl = false;
        for r in k..=l {
            let s = self.target_chunk_x * 16 + r; // 当前 chunk getOffsetX
            let g = (s as f64 + 0.5 - x) / width;
            for t in p..=q {
                let u = self.target_chunk_z * 16 + t; // 当前 chunk getOffsetZ
                let h = (u as f64 + 0.5 - z) / width;
                if g * g + h * h >= 1.0 { continue; }
                let mut replaced_grassy = false;
                let mut v = o;
                while v > m {
                    let w = (v as f64 - 0.5 - y) / height;
                    if skip_predicate(g, w, h, v) { v -= 1; continue; }
                    if !mask.get(r, v, t) {
                        mask.set(r, v, t);
                        bl |= self.carve_at_point(ctx, config, col, biome_at, r, v, t, s, u, &mut replaced_grassy);
                    }
                    v -= 1;
                }
            }
        }
        bl
    }

    // Java Carver.carveAtPoint（mask 已标记；r/t 局部坐标 0-15，v 世界 y；s/u 世界 x/z）
    fn carve_at_point(
        &self,
        ctx: &mut CarverContext,
        config: &CarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        r: i32, v: i32, t: i32, wx: i32, wz: i32,
        replaced_grassy: &mut bool,
    ) -> bool {
        let wy = v; // world y
        // Java chunk.setBlockState：isOutOfHeightLimit(pos) → 跳过（洞穴漂移可越界世界高度！）
        if wy < ctx.min_y || wy >= ctx.min_y + ctx.height { return false; }
        let block = col.at(r, wy, t);
        let grass_id = ctx.blocks.id("minecraft:grass_block");
        let mycelium_id = ctx.blocks.id("minecraft:mycelium");
        if block == grass_id || block == mycelium_id { *replaced_grassy = true; }
        if !self.can_always_carve_block(config, block) { return false; }
        let state = self.get_state(ctx, config, wx, wy, wz);
        if state < 0 { return false; }
        *col.at_mut(r, wy, t) = state;
        if *replaced_grassy {
            let below = col.at(r, wy - 1, t);
            let dirt_id = ctx.blocks.id("minecraft:dirt");
            if below == dirt_id && ctx.apply_material_rule.is_some() {
                let has_fluid = Self::is_fluid(ctx, state);
                let ns = ctx.apply_material_rule.unwrap()(wx, wy - 1, wz, has_fluid);
                if let Some(ns) = ns { *col.at_mut(r, wy - 1, t) = ns; }
            }
        }
        true
    }

    fn is_fluid(ctx: &CarverContext, block_id: BlockId) -> bool {
        // Java !blockState2.getFluidState().isEmpty()：air/water/lava 的 FluidState 判断
        let water_id = ctx.blocks.id("minecraft:water");
        let lava_id = ctx.blocks.id("minecraft:lava");
        block_id == water_id || block_id == lava_id
    }

    // Java Carver.getState：y <= lavaLevel → lava；否则 aquifer.apply(pos, 0.0)
    fn get_state(&self, ctx: &mut CarverContext, config: &CarverConfig, x: i32, y: i32, z: i32) -> i32 {
        let lava_id = ctx.blocks.id("minecraft:lava");
        if y <= config.lava_level.get_y(ctx.min_y, ctx.height) { return lava_id; }
        ctx.aquifer.apply(x, y, z, 0.0) // density=0.0（Java sampler.apply(pos, 0.0)）
    }

    // Java Carver.canAlwaysCarveBlock = state.isIn(config.replaceable)
    fn can_always_carve_block(&self, config: &CarverConfig, block_id: BlockId) -> bool {
        config.replaceable_ids.iter().any(|&id| id == block_id)
    }

    // Java Carver.canCarveBranch
    fn can_carve_branch(chunk_x: i32, chunk_z: i32, x: f64, z: f64, branch_index: i32, branch_count: i32, base_width: f32) -> bool {
        let d = chunk_x as f64 * 16.0 + 8.0; // getCenterX
        let e = chunk_z as f64 * 16.0 + 8.0;
        let f = x - d;
        let g = z - e;
        let h = (branch_count - branch_index) as f64;
        let i = base_width as f64 + 2.0 + 16.0;
        f * f + g * g - h * h <= i * i
    }
}

// ===== CaveCarver（Java CaveCarver.java）=====
pub struct CaveCarver { pub base: Carver }
impl CaveCarver {
    pub fn new() -> Self { CaveCarver { base: Carver::new() } }
    fn get_max_cave_count(&self) -> i32 { 15 }

    fn get_tunnel_system_width(&self, random: &mut ChunkRandom) -> f32 {
        let mut f = random.next_float() * 2.0 + random.next_float();
        if random.next_int_bound(10) == 0 { f *= random.next_float() * random.next_float() * 3.0 + 1.0; }
        f
    }

    fn get_tunnel_system_height_width_ratio(&self) -> f64 { 1.0 }

    pub fn carve(
        &mut self,
        ctx: &mut CarverContext,
        cfg: &CaveCarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        random: &mut ChunkRandom,
        chunk_x: i32, chunk_z: i32,
        mask: &mut CarvingMask,
    ) -> bool {
        let i = Carver::branch_coord(); // 112
        let a = random.next_int_bound(self.get_max_cave_count());
        let b = random.next_int_bound(a + 1);
        let j = random.next_int_bound(b + 1);
        for _k in 0..j {
            let d = chunk_x as f64 * 16.0 + random.next_int_bound(16) as f64;         // getOffsetX(nextInt(16))
            let e = cfg.common.y.get(random, ctx.min_y, ctx.height) as f64;
            let f = chunk_z as f64 * 16.0 + random.next_int_bound(16) as f64;         // getOffsetZ(nextInt(16))
            let g = cfg.horizontal_radius_multiplier.get(random) as f64;
            let h = cfg.vertical_radius_multiplier.get(random) as f64;
            let l = cfg.floor_level.get(random) as f64;
            let skip_predicate = |rx: f64, ry: f64, rz: f64, _y: i32| -> bool {
                // isPositionExcluded：scaledRelativeY <= floorLevel || x²+y²+z² >= 1
                ry <= l || rx * rx + ry * ry + rz * rz >= 1.0
            };
            let mut m = 1;
            if random.next_int_bound(4) == 0 {
                let n = cfg.common.y_scale.get(random) as f64;
                let o = 1.0 + random.next_float() * 6.0;
                self.carve_cave(ctx, cfg, col, biome_at, d, e, f, o, n, mask, &skip_predicate);
                m += random.next_int_bound(4);
            }
            for _p in 0..m {
                let q = random.next_float() * (3.14159265358979323846 * 2.0) as f32;
                let o = (random.next_float() - 0.5) / 4.0;
                let r = self.get_tunnel_system_width(random);
                let s = i - random.next_int_bound(i / 4);
                self.carve_tunnels(ctx, cfg, col, biome_at, random.next_long(), d, e, f, g, h, r, q, o, 0, s,
                                   self.get_tunnel_system_height_width_ratio(), mask, &skip_predicate);
            }
        }
        true
    }

    fn carve_cave(
        &mut self,
        ctx: &mut CarverContext,
        config: &CaveCarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        d: f64, e: f64, f: f64, g: f32, h: f64,
        mask: &mut CarvingMask,
        skip_predicate: &dyn Fn(f64, f64, f64, i32) -> bool,
    ) {
        let i = 1.5 + math_sin((3.14159265358979323846 / 2.0) as f32) * g;
        let j = i as f64 * h;
        // Java carveCave → carveRegion 内部 chunk.getPos() = 当前 chunk（写方块目标）——用 targetChunkX/Z
        self.base.carve_region_impl(ctx, &config.common, col, biome_at, d + 1.0, e, f, i as f64, j, mask, skip_predicate);
    }

    fn carve_tunnels(
        &mut self,
        ctx: &mut CarverContext,
        config: &CaveCarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        seed: i64, mut x: f64, mut y: f64, mut z: f64,
        horizontal_scale: f64, vertical_scale: f64, mut width: f32, mut yaw: f32, mut pitch: f32,
        branch_start_index: i32, branch_count: i32, yaw_pitch_ratio: f64,
        mask: &mut CarvingMask,
        skip_predicate: &dyn Fn(f64, f64, f64, i32) -> bool,
    ) {
        // Java CaveCarver.carveTunnels L145: Random.create(seed) = CheckedRandom（48 位 LCG，非 Xoroshiro！）
        let mut random = CheckedRandom::new(seed);
        let i = random.next_int(branch_count / 2) + branch_count / 4;
        let bl = random.next_int(6) == 0;
        let mut f = 0.0f32;
        let mut g = 0.0f32;
        for j in branch_start_index..branch_count {
            // Java: MathHelper.sin((float)Math.PI * j / branchCount)——(float)Math.PI=3.1415927F 全程 float！
            let d = 1.5 + math_sin(3.1415927f32 * j as f32 / branch_count as f32) * width;
            let e = d as f64 * yaw_pitch_ratio;
            let h = math_cos(pitch);
            x += math_cos(yaw) as f64 * h as f64;
            y += math_sin(pitch) as f64;
            z += math_sin(yaw) as f64 * h as f64;
            pitch *= if bl { 0.92 } else { 0.7 };
            pitch += g * 0.1;
            yaw += f * 0.1;
            g *= 0.9;
            f *= 0.75;
            g += (random.next_float() - random.next_float()) * random.next_float() * 2.0;
            f += (random.next_float() - random.next_float()) * random.next_float() * 4.0;
            if j == i && width > 1.0 {
                self.carve_tunnels(ctx, config, col, biome_at, random.next_long(), x, y, z, horizontal_scale, vertical_scale,
                                   random.next_float() * 0.5 + 0.5, yaw - (3.14159265358979323846 / 2.0) as f32, pitch / 3.0,
                                   j, branch_count, 1.0, mask, skip_predicate);
                self.carve_tunnels(ctx, config, col, biome_at, random.next_long(), x, y, z, horizontal_scale, vertical_scale,
                                   random.next_float() * 0.5 + 0.5, yaw + (3.14159265358979323846 / 2.0) as f32, pitch / 3.0,
                                   j, branch_count, 1.0, mask, skip_predicate);
                return;
            }
            if random.next_int(4) != 0 {
                if !Carver::can_carve_branch(self.base.target_chunk_x, self.base.target_chunk_z, x, z, j, branch_count, width) { return; }
                self.base.carve_region_impl(ctx, &config.common, col, biome_at, x, y, z, d as f64 * horizontal_scale, e * vertical_scale,
                                            mask, skip_predicate);
            }
        }
    }
}

// ===== RavineCarver（Java RavineCarver.java）=====
pub struct RavineCarver { pub base: Carver }
impl RavineCarver {
    pub fn new() -> Self { RavineCarver { base: Carver::new() } }

    pub fn carve(
        &mut self,
        ctx: &mut CarverContext,
        cfg: &RavineCarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        random: &mut ChunkRandom,
        chunk_x: i32, chunk_z: i32,
        mask: &mut CarvingMask,
    ) -> bool {
        let i = Carver::branch_coord(); // 112
        let d = chunk_x as f64 * 16.0 + random.next_int_bound(16) as f64;
        let j = cfg.common.y.get(random, ctx.min_y, ctx.height);
        let e = chunk_z as f64 * 16.0 + random.next_int_bound(16) as f64;
        let f = random.next_float() * (3.14159265358979323846 * 2.0) as f32;
        let g = cfg.vertical_rotation.get(random);
        let h = cfg.common.y_scale.get(random) as f64;
        let k = cfg.shape.thickness.get(random);
        let l = (i as f64 * cfg.shape.distance_factor.get(random) as f64) as i32;
        self.carve_ravine(ctx, cfg, col, biome_at, random.next_long(), d, j as f64, e, k, f, g, 0, l, h, mask);
        true
    }

    fn carve_ravine(
        &mut self,
        ctx: &mut CarverContext,
        config: &RavineCarverConfig,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        seed: i64, mut x: f64, mut y: f64, mut z: f64, mut width: f32, mut yaw: f32, mut pitch: f32,
        branch_start_index: i32, branch_count: i32, yaw_pitch_ratio: f64,
        mask: &mut CarvingMask,
    ) {
        // Java RavineCarver.carveRavine L65: Random.create(seed) = CheckedRandom（48 位 LCG）
        let mut random = CheckedRandom::new(seed);
        let fs = self.create_horizontal_stretch_factors(ctx, config, &mut random);
        let mut f = 0.0f32;
        let mut g = 0.0f32;
        for i in branch_start_index..branch_count {
            let mut d = 1.5 + math_sin(i as f32 * 3.14159265358979323846f32 / branch_count as f32) * width;
            let mut e = d as f64 * yaw_pitch_ratio;
            d *= config.shape.horizontal_radius_factor.get_checked(&mut random);
            e = self.get_vertical_scale(config, &mut random, e, branch_count, i);
            let h = math_cos(pitch);
            let j = math_sin(pitch);
            x += math_cos(yaw) as f64 * h as f64;
            y += j as f64;
            z += math_sin(yaw) as f64 * h as f64;
            pitch *= 0.7;
            pitch += g * 0.05;
            yaw += f * 0.05;
            g *= 0.8;
            f *= 0.5;
            g += (random.next_float() - random.next_float()) * random.next_float() * 2.0;
            f += (random.next_float() - random.next_float()) * random.next_float() * 4.0;
            if random.next_int(4) != 0 {
                if !Carver::can_carve_branch(self.base.target_chunk_x, self.base.target_chunk_z, x, z, i, branch_count, width) { return; }
                let min_y = ctx.min_y;
                let skip = |rx: f64, ry: f64, rz: f64, yv: i32| -> bool {
                    let idx = yv - min_y;
                    if idx - 1 < 0 || idx - 1 >= fs.len() as i32 { return true; }
                    (rx * rx + rz * rz) * fs[(idx - 1) as usize] as f64 + ry * ry / 6.0 >= 1.0
                };
                self.base.carve_region_impl(ctx, &config.common, col, biome_at, x, y, z, d as f64, e, mask, &skip);
            }
        }
    }

    fn create_horizontal_stretch_factors(&self, ctx: &CarverContext, config: &RavineCarverConfig, random: &mut CheckedRandom) -> Vec<f32> {
        let i = ctx.height;
        let mut fs = vec![1.0f32; i as usize];
        let mut f = 1.0f32;
        for j in 0..i {
            if j == 0 || random.next_int(config.shape.width_smoothness) == 0 {
                f = 1.0 + random.next_float() * random.next_float();
            }
            fs[j as usize] = f * f; // Java RavineCarver.java L122: fs[j] = f * f（缺平方曾导致 ravine 挖更宽）
        }
        fs
    }

    // Java getVerticalScale（RavineCarver.java L128-132）：
    //   f = 1.0 - abs(0.5 - branchIndex/branchCount) * 2.0；g = default + center * f
    fn get_vertical_scale(&self, config: &RavineCarverConfig, random: &mut CheckedRandom, pitch: f64,
                          branch_count: i32, i: i32) -> f64 {
        let f = 1.0 - (0.5 - i as f32 / branch_count as f32).abs() * 2.0;
        let g = config.shape.vertical_radius_default_factor + config.shape.vertical_radius_center_factor * f;
        // MathHelper.nextBetween(random, 0.75F, 1.0F) = nextFloat()*0.25+0.75
        let nb = random.next_float() * 0.25 + 0.75;
        (g * pitch as f32 * nb) as f64
    }
}

// ===== ConfiguredCarver 包装（type 分派）=====
// carverStep.air 的 configured_carver JSON：type = minecraft:cave / cave_extra_underground / canyon
#[derive(Clone)]
pub enum ConfiguredCarver {
    Cave(CaveCarverConfig),
    Ravine(RavineCarverConfig),
}
impl ConfiguredCarver {
    pub fn probability(&self) -> f32 {
        match self {
            ConfiguredCarver::Cave(c) => c.common.probability,
            ConfiguredCarver::Ravine(c) => c.common.probability,
        }
    }

    pub fn should_carve(&self, random: &mut ChunkRandom) -> bool {
        random.next_float() <= self.probability()
    }

    pub fn carve(
        &self,
        ctx: &mut CarverContext,
        col: &mut BlockColumn,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        random: &mut ChunkRandom,
        chunk_x: i32, chunk_z: i32, target_x: i32, target_z: i32,
        mask: &mut CarvingMask,
    ) -> bool {
        match self {
            ConfiguredCarver::Cave(cfg) => {
                let mut c = CaveCarver::new();
                c.base.target_chunk_x = target_x; c.base.target_chunk_z = target_z;
                c.carve(ctx, cfg, col, biome_at, random, chunk_x, chunk_z, mask)
            }
            ConfiguredCarver::Ravine(cfg) => {
                let mut c = RavineCarver::new();
                c.base.target_chunk_x = target_x; c.base.target_chunk_z = target_z;
                c.carve(ctx, cfg, col, biome_at, random, chunk_x, chunk_z, mask)
            }
        }
    }

    // 解析 configured_carver JSON（type + config）
    pub fn parse(root: &JsonValue, blocks: &BlockRegistry) -> ConfiguredCarver {
        let type_name = root.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let cfg = root.get("config");
        if type_name.contains("canyon") {
            ConfiguredCarver::Ravine(RavineCarverConfig::parse(cfg, blocks))
        } else {
            ConfiguredCarver::Cave(CaveCarverConfig::parse(cfg, blocks))
        }
    }
}
