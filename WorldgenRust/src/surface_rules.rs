// surface_rules.rs — MaterialRules 引擎 + VanillaSurfaceRules 翻译 + SurfaceBuilder（Rust 移植）
// 对应 C++: versions/1.20.1/cpp/worldgen/src/surface.h（859 行，已完整移植 vanilla surface rules）
// 对应 Java: MaterialRules.java / VanillaSurfaceRules.java / SurfaceBuilder.java
//
// 移植范围（本模块）：
//   1. SurfaceCond 条件枚举（Biome/AboveY/Water/StoneDepth/NoiseThreshold/Hole/Steep/
//      SurfaceCondC(above_preliminary_surface)/Temp/VerticalGradient/Not）
//   2. SurfaceRule 规则枚举（Block/Cond/Seq/TerracottaBands）
//   3. SurfaceContext（字段 + estimateSurfaceHeight + initVertical + splitterFor）
//   4. buildOverworldRule 规则树（bedrock_floor + surface(mr9) + deepslate）
//   5. buildSurface 引擎（逐列扫描应用规则）
//   6. biome 温度表（biome → 温度，对齐 C++ biomeTemp 用法）
//
// ⚠️ 未编译验证：本文件由 worker 产出，主会话负责 cargo 编译验证。
// 静态自检清单见文件末尾注释。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use crate::blocks::{BlockColumn, BlockId, BlockRegistry};
use crate::noise::DoublePerlinNoiseSampler;
use crate::xoroshiro::XoroshiroRandom;
use crate::legacy_random::{RsRandom, RsSplitter};

// ========== 辅助 ==========
// 对齐 C++ surface.h L20-24 lerpClamp
fn lerp_clamp(value: f64, from_start: f64, from_end: f64, to_start: f64, to_end: f64) -> f64 {
    let mut t = (value - from_start) / (from_end - from_start);
    t = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    to_start + t * (to_end - to_start)
}

// 对齐 C++ SurfaceContext::lerp2 L205-207（Java MathHelper.lerp2）
fn lerp2(fx: f64, fz: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    a + (b - a) * fx + (c - a) * fz + (a - b - c + d) * fx * fz
}

// 打包 (x,z) 为 i64 列缓存 key（对齐 C++ `((int64_t)((uint64_t)(uint32_t)x << 32)) ^ (uint32_t)z`）
fn col_key(x: i32, z: i32) -> i64 {
    ((((x as u32) as u64) << 32) ^ (z as u32 as u64)) as i64
}

// #26 判据 1（260903-15）：overworld 引擎路径 noise key 单一事实源——**紧邻下方 get_noise
// 引擎调用点维护**（place_badlands_pillar 等不在 rule 树内，启动断言的机械收集覆盖不到，
// 新增引擎 get_noise 调用点必须同步在此加一行）。worldgen_handle::create 预加载遍历本清单。
// 注：rule 树内引用（NoiseThreshold 条件）由 collect_rule_noise_keys 启动期机械核对，无需本清单。
pub const ENGINE_NOISE_KEYS: &[&str] = &[
    "minecraft:badlands_surface",
    "minecraft:badlands_pillar",
    "minecraft:badlands_pillar_roof",
    "minecraft:calcite",
    "minecraft:gravel",
    "minecraft:powder_snow",
    "minecraft:packed_ice",
    "minecraft:ice",
    "minecraft:surface_swamp",
];

// ========== 条件枚举（对齐 C++ surface.h L34-124 / L210-296）==========
#[derive(Clone)]
pub enum SurfaceCond {
    Biome { biomes: Vec<String> },
    AboveY { anchor_y: i32, mult: i32, add_stone_depth: bool },
    Water { offset: i32, mult: i32, add_stone_depth: bool },
    StoneDepth { offset: i32, add_surface_depth: bool, secondary_depth_range: i32, ceiling: bool },
    NoiseThreshold { noise_key: String, min_th: f64, max_th: f64 },
    Hole,
    Steep,
    SurfaceCondC, // above_preliminary_surface
    Temp,         // temperature()
    VerticalGradient { name: String, true_y: i32, false_y: i32 },
    Not(Box<SurfaceCond>),
}

// NoiseThreshold 列缓存：thread_local HashMap<noise_key, (col_key, value)>
// 对齐 C++ per-instance thread_local 单槽缓存（值 = 纯函数 (noise_key,x,z)，共享缓存语义等价且更省）
thread_local! {
    static NOISE_THRESH_CACHE: RefCell<HashMap<String, (i64, f64)>> = RefCell::new(HashMap::new());
}

impl SurfaceCond {
    pub fn test(&self, ctx: &SurfaceContext) -> bool {
        match self {
            SurfaceCond::Biome { biomes } => biomes.iter().any(|b| b == &ctx.biome_id),
            // C++ L219-222 AboveYCond
            SurfaceCond::AboveY { anchor_y, mult, add_stone_depth } => {
                let y = ctx.block_y + if *add_stone_depth { ctx.stone_depth_above } else { 0 };
                y >= anchor_y + ctx.surface_depth * mult
            }
            // C++ L223-227 WaterCond
            SurfaceCond::Water { offset, mult, add_stone_depth } => {
                if ctx.fluid_height == i32::MIN {
                    return true;
                }
                let y = ctx.block_y + if *add_stone_depth { ctx.stone_depth_above } else { 0 };
                y >= ctx.fluid_height + offset + ctx.surface_depth * mult
            }
            // C++ L228-236 StoneDepthCond（k 对齐 Java (int)MathHelper.map(sec,-1,1,0,range)，不 clamp + (int) 向零截断）
            SurfaceCond::StoneDepth { offset, add_surface_depth, secondary_depth_range, ceiling } => {
                let i = if *ceiling { ctx.stone_depth_below } else { ctx.stone_depth_above };
                let j = if *add_surface_depth { ctx.surface_depth } else { 0 };
                let k = if *secondary_depth_range == 0 {
                    0
                } else {
                    ((ctx.get_secondary_depth() + 1.0) * 0.5 * (*secondary_depth_range as f64)) as i32
                };
                i <= 1 + offset + j + k
            }
            // C++ L237-250 NoiseThresholdCond
            SurfaceCond::NoiseThreshold { noise_key, min_th, max_th } => {
                let d = noise_threshold_sample(ctx, noise_key);
                d >= *min_th && d <= *max_th
            }
            // M6 附记修正（2026-08-30）：Java HoleCondition = stoneDepthAbove <= 0（worker 逐行核对
            // yarn 源码）。当年注释称「C++ L251 用错字段」系误判——C++ 才是对的，此处曾用 runDepth 噪声。
            SurfaceCond::Hole => ctx.stone_depth_above <= 0,
            // C++ L252-261 SteepCond
            // ⚠️ 关键决策：C++ 读 `hm[i*16+j]`（i=x,j=z）= hm[x*16+z]，但 heightmap 填充为 z*16+x（worldgen_api.cpp L1045），
            // 是转置 bug。Java SteepSlopePredicate 读 sampleHeightmap(x, z±1)/(x±1, z)。此处按 Java 修正。
            SurfaceCond::Steep => steep_test(ctx),
            // C++ L262-280 SurfaceCondC（above_preliminary_surface）
            SurfaceCond::SurfaceCondC => surface_cond_c_test(ctx),
            // C++ L281 TempCond
            SurfaceCond::Temp => ctx.biome_temp < 0.15,
            // C++ L282-293 VerticalGradientCond（先查 false 再查 true，支持反锚序）
            SurfaceCond::VerticalGradient { name, true_y, false_y } => {
                vertical_gradient_test(ctx, name, *true_y, *false_y)
            }
            // C++ L120-124 NotCond
            SurfaceCond::Not(inner) => !inner.test(ctx),
        }
    }
}

fn noise_threshold_sample(ctx: &SurfaceContext, noise_key: &str) -> f64 {
    let key = col_key(ctx.block_x, ctx.block_z);
    NOISE_THRESH_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((k, v)) = c.get(noise_key) {
            if *k == key {
                return *v;
            }
        }
        // E7 教训（judge C1）：查不到 sampler 不能静默回退——每 key 只 warn 一次（全局去重，
        // 不在热路径重复打印），fail-fast 提示 step4 预加载表缺 key（隐式契约显式化）
        let v = match ctx.noise_samplers.get(noise_key) {
            Some(n) => n.sample(ctx.block_x as f64, 0.0, ctx.block_z as f64),
            None => {
                warn_unknown_noise_key(noise_key);
                0.0
            }
        };
        c.insert(noise_key.to_string(), (key, v));
        v
    })
}

// E7/judge C1：未知 noise key 全局每 key 只告警一次（跨线程去重，OnceLock+Mutex）
fn warn_unknown_noise_key(key: &str) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let set = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut s) = set.lock() {
        if s.insert(key.to_string()) {
            eprintln!("[SURFACE-WARN] unknown noise key '{}' in surface rule -> fallback 0.0 (check step4 preload table in worldgen_handle.rs)", key);
        }
    }
}

// C2（2026-09-07，judge CONCERN：step4 预加载表数据驱动化）：遍历 surface_rule JSON，
// 收集所有引用的 noise key（构建期一次性调用，非热路径）。
// 递归结构与 parse_surface_rule/parse_surface_cond 的节点形态一一对应：
// rule = sequence / condition(if_true + then_run)；cond = not(invert) / noise_threshold(noise 字段) / 其他叶子。
// #26 判据 1 泛化（260903-15）：不再只认 noise_threshold 类型——任何带 "noise" 字符串字段的
// 节点都收（noise_threshold / vertical_gradient / 未来新增节点类型自动覆盖），
// 消除「collect_noise_keys 只收单一字段」的缺项盲区（NEXT 260903-14 未闭合课题 1）。
pub fn collect_noise_keys(j: &crate::json::JsonValue, out: &mut Vec<String>) {
    if let Some(n) = j.get("noise").and_then(|n| n.as_str()) {
        if !out.iter().any(|k| k == n) {
            out.push(n.to_string());
        }
    }
    if let Some(seq) = j.get("sequence") {
        if let Some(arr) = seq.as_array() {
            for r in arr {
                collect_noise_keys(r, out);
            }
        }
    }
    if let Some(c) = j.get("if_true") {
        collect_noise_keys(c, out);
    }
    if let Some(t) = j.get("then_run") {
        collect_noise_keys(t, out);
    }
    if let Some(i) = j.get("invert") {
        collect_noise_keys(i, out);
    }
}

// #26 判据 1（260903-15）：对**已构建**的 SurfaceRule 树收集全部运行时会查
// noise_samplers 的 key（目前唯一来源 = NoiseThreshold 条件；VerticalGradient 走
// splitter 不查 sampler，不算）。启动期用：运行时引用 key ⊆ 预加载集合断言的
// 「运行时引用」一侧的事实源——规则在代码里改动时无需同步手工清单，此函数自动覆盖。
pub fn collect_rule_noise_keys(rule: &SurfaceRule, out: &mut Vec<String>) {
    match rule {
        SurfaceRule::Block(_) | SurfaceRule::TerracottaBands => {}
        SurfaceRule::Seq(rules) => {
            for r in rules {
                collect_rule_noise_keys(r, out);
            }
        }
        SurfaceRule::Cond { cond, rule } => {
            let mut c = cond;
            loop {
                match c {
                    SurfaceCond::NoiseThreshold { noise_key, .. } => {
                        if !out.iter().any(|k| k == noise_key) {
                            out.push(noise_key.clone());
                        }
                        break;
                    }
                    SurfaceCond::Not(inner) => c = &**inner,
                    _ => break,
                }
            }
            collect_rule_noise_keys(rule, out);
        }
    }
}

// 对齐 Java SteepSlopePredicate（MaterialRules.java L541-565），heightmap 索引 z*16+x
fn steep_test(ctx: &SurfaceContext) -> bool {
    let i = ctx.block_x & 15; // x
    let j = ctx.block_z & 15; // z
    let hm = ctx.column_heightmap.expect("steep needs column_heightmap");
    let hm = hm.borrow();
    // (x, z-1) 与 (x, z+1)：heightmap[(z±1)*16 + x]
    let m = hm[((j - 1).max(0) * 16 + i) as usize];
    let n = hm[((j + 1).min(15) * 16 + i) as usize];
    if n >= m + 4 {
        return true;
    }
    // (x-1, z) 与 (x+1, z)：heightmap[z*16 + (x±1)]
    let o = (i - 1).max(0);
    let p = (i + 1).min(15);
    let q = hm[(j * 16 + o) as usize];
    let r = hm[(j * 16 + p) as usize];
    q >= r + 4
}

// 对齐 C++ L262-280 SurfaceCondC（above_preliminary_surface）
fn surface_cond_c_test(ctx: &SurfaceContext) -> bool {
    let k: i32;
    if let Some(e) = ctx.surface_heights4 {
        if e.len() == 4 {
            // MathHelper.lerp2(fx, fz, e00, e10, e01, e11)：fx=(blockX&15)/16, fz=(blockZ&15)/16
            let fx = (ctx.block_x & 15) as f64 / 16.0;
            let fz = (ctx.block_z & 15) as f64 / 16.0;
            k = lerp2(fx, fz, e[0] as f64, e[1] as f64, e[2] as f64, e[3] as f64).floor() as i32;
        } else {
            k = ctx.estimate_surface_height();
        }
    } else {
        k = ctx.estimate_surface_height();
    }
    ctx.block_y >= k + ctx.surface_depth - 8
}

// 对齐 C++ L282-293 VerticalGradientCond
fn vertical_gradient_test(ctx: &SurfaceContext, name: &str, true_y: i32, false_y: i32) -> bool {
    let y = ctx.block_y;
    if y >= false_y {
        return false;
    }
    if y <= true_y {
        return true;
    }
    let d = lerp_clamp(y as f64, true_y as f64, false_y as f64, 1.0, 0.0);
    // NoiseConfig.getOrCreateRandomDeriver(name).split(x, y, z)
    let s = ctx.splitter_for(name);
    let mut r = s.split_xyz(ctx.block_x, y, ctx.block_z);
    (r.next_float() as f64) < d
}

// ========== 规则枚举（对齐 C++ surface.h L44-64 / L212-218 / L294-296）==========
#[derive(Clone)]
pub enum SurfaceRule {
    Block(BlockId),
    Cond { cond: SurfaceCond, rule: Box<SurfaceRule> },
    Seq(Vec<SurfaceRule>),
    TerracottaBands,
}

impl SurfaceRule {
    // 返回 BlockId；不适用返回 None（对齐 C++ apply 返回 -1）
    pub fn apply(&self, ctx: &SurfaceContext) -> Option<BlockId> {
        match self {
            SurfaceRule::Block(b) => Some(*b),
            SurfaceRule::Cond { cond, rule } => {
                if cond.test(ctx) {
                    rule.apply(ctx)
                } else {
                    None
                }
            }
            SurfaceRule::Seq(rules) => {
                for r in rules {
                    if let Some(b) = r.apply(ctx) {
                        return Some(b);
                    }
                }
                None
            }
            SurfaceRule::TerracottaBands => {
                ctx.terracotta_bands_getter.map(|f| f(ctx.block_x, ctx.block_y, ctx.block_z))
            }
        }
    }
}

// ========== SurfaceContext（对齐 C++ surface.h L127-208）==========
pub struct SurfaceContext<'a> {
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub world_min_y: i32,
    pub world_height: i32,
    // 列初始（sampleRunDepth 2D 噪声）——y_above/water/stone_depth/hole 用（Java MaterialRuleContext.runDepth）
    pub surface_depth: i32,
    pub stone_depth_above: i32,
    pub stone_depth_below: i32,
    pub fluid_height: i32,
    pub biome_id: String,
    pub biome_temp: f64,
    pub noise_samplers: &'a HashMap<String, Arc<DoublePerlinNoiseSampler>>,
    pub splitter: &'a RsSplitter,
    pub initial_density_at: Option<&'a dyn Fn(i32, i32, i32) -> f64>,
    pub terracotta_bands_getter: Option<&'a dyn Fn(i32, i32, i32) -> i32>,
    pub column_heightmap: Option<&'a RefCell<Vec<i32>>>, // [256] WORLD_SURFACE_WG，索引 z*16+x（RefCell 供 pillar 写回）
    pub surface_heights4: Option<&'a [i32]>, // chunk 4 角 estimateSurfaceHeight
    pub surface_secondary_noise: Option<&'a DoublePerlinNoiseSampler>,
    // 按名字派生的 splitter 缓存（对应 NoiseConfig.getOrCreateRandomDeriver）
    derived_splitters: RefCell<HashMap<String, RsSplitter>>,
    // getSecondaryDepth 列缓存
    secondary_cache_key: Cell<i64>,
    secondary_cache: Cell<f64>,
}

impl<'a> SurfaceContext<'a> {
    // 便捷构造：初始化核心字段 + 缓存（引用字段由调用方 struct literal 填充，见 build_surface）
    pub fn new(
        noise_samplers: &'a HashMap<String, Arc<DoublePerlinNoiseSampler>>,
        splitter: &'a RsSplitter,
        world_min_y: i32,
        world_height: i32,
    ) -> Self {
        SurfaceContext {
            block_x: 0,
            block_y: 0,
            block_z: 0,
            world_min_y,
            world_height,
            surface_depth: 0,
            stone_depth_above: 0,
            stone_depth_below: 0,
            fluid_height: i32::MIN,
            biome_id: String::new(),
            biome_temp: 0.5,
            noise_samplers,
            splitter,
            initial_density_at: None,
            terracotta_bands_getter: None,
            column_heightmap: None,
            surface_heights4: None,
            surface_secondary_noise: None,
            derived_splitters: RefCell::new(HashMap::new()),
            secondary_cache_key: Cell::new(i64::MIN),
            secondary_cache: Cell::new(0.0),
        }
    }

    // NoiseConfig.getOrCreateRandomDeriver(id) = randomDeriver.split(id).nextSplitter()
    fn splitter_for(&self, name: &str) -> RsSplitter {
        let mut map = self.derived_splitters.borrow_mut();
        if let Some(s) = map.get(name) {
            return s.clone();
        }
        let mut r = self.splitter.split_str(name);
        let s = r.next_splitter();
        map.insert(name.to_string(), s.clone());
        s
    }

    // 对齐 C++ L168-175 getSecondaryDepth（surface_secondary 2D 噪声，列缓存）
    fn get_secondary_depth(&self) -> f64 {
        let key = col_key(self.block_x, self.block_z);
        if key != self.secondary_cache_key.get() {
            self.secondary_cache_key.set(key);
            let v = self
                .surface_secondary_noise
                .map(|n| n.sample(self.block_x as f64, 0.0, self.block_z as f64))
                .unwrap_or(0.0);
            self.secondary_cache.set(v);
        }
        self.secondary_cache.get()
    }

    // 对齐 C++ L177-194 estimateSurfaceHeight：从顶向下扫描 initialDensityWithoutJaggedness > 0.390625（间隔 8）
    // 列缓存（thread_local 单槽，纯函数同列同值）
    fn estimate_surface_height(&self) -> i32 {
        thread_local! {
            static CACHE: RefCell<(i64, i32)> = RefCell::new((i64::MIN, 0));
        }
        let key = col_key(self.block_x, self.block_z);
        CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != key {
                c.0 = key;
                let mut est = i32::MAX;
                if let Some(f) = self.initial_density_at {
                    let mut y = self.world_min_y + self.world_height;
                    while y >= self.world_min_y {
                        if f(self.block_x, y, self.block_z) > 0.390625 {
                            est = y;
                            break;
                        }
                        y -= 8;
                    }
                }
                c.1 = est;
            }
            c.1
        })
    }

    // 对齐 C++ L196-203 initVertical（pub：bin-diag soul_selector_probe 复算 stone_depth 用，2026-09-07）
    pub fn init_vertical(
        &mut self,
        stone_depth_above: i32,
        stone_depth_below: i32,
        fluid_height: i32,
        x: i32,
        y: i32,
        z: i32,
        biome: &str,
    ) {
        self.stone_depth_above = stone_depth_above;
        self.stone_depth_below = stone_depth_below;
        self.fluid_height = fluid_height;
        self.block_x = x;
        self.block_y = y;
        self.block_z = z;
        self.biome_id = biome.to_string();
    }
}

// V4 诊断（2026-09-09，soul 签名 B 运行时输入差裁决）：生产链路 soul 分支入口 ctx dump。
// 门控：env WG_SOUL_CTX_DUMP=<点文件路径>（`x y z 标签` 行格式，# 注释）。
// 进程级读一次点集（OnceLock）；build_surface 入口 chunk 级取一次引用——未配置时零热路径成本
// （每 chunk 一次 thread_local/OnceLock 访问，非每点 env 查询，对齐测量污染铁律）。
// 命中点集的 apply 点 dump：生产 ctx 的 biome/sda/sdb/surface_depth/fluid_height/selector +
// soul 分支入口判定 + 规则 apply 结果（stderr，诊断输出）。
fn soul_dump_points() -> &'static Option<std::collections::HashSet<(i32, i32, i32)>> {
    static DUMP: std::sync::OnceLock<Option<std::collections::HashSet<(i32, i32, i32)>>> =
        std::sync::OnceLock::new();
    DUMP.get_or_init(|| {
        let path = std::env::var("WG_SOUL_CTX_DUMP").ok()?;
        let txt = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("[SOUL-CTX] cannot read {}: {}", path, e));
        let mut set = std::collections::HashSet::new();
        for line in txt.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let mut it = line.split_whitespace();
            if let (Some(a), Some(b), Some(c)) = (it.next(), it.next(), it.next()) {
                if let (Ok(x), Ok(y), Ok(z)) = (a.parse(), b.parse(), c.parse()) {
                    set.insert((x, y, z));
                }
            }
        }
        eprintln!("[SOUL-CTX] dump enabled: {} points from {}", set.len(), path);
        Some(set)
    })
}

// ========== SurfaceBuilder（对齐 C++ surface.h L345-483）==========
pub struct SurfaceBuilder<'a> {
    samplers: &'a HashMap<String, Arc<DoublePerlinNoiseSampler>>,
    splitter: &'a RsSplitter,
    #[allow(dead_code)] // 保留 API 对齐（placeIceberg 未移植，暂未使用）
    sea_level: i32,
    blocks: &'a BlockRegistry,
    terracotta_bands: Vec<BlockId>,
}

impl<'a> SurfaceBuilder<'a> {
    pub fn new(
        samplers: &'a HashMap<String, Arc<DoublePerlinNoiseSampler>>,
        splitter: &'a RsSplitter,
        sea_level: i32,
        blocks: &'a BlockRegistry,
    ) -> Self {
        // clay_bands random：预生成 192 长度红陶带数组（对齐 C++ L352-376 / Java createTerracottaBands）
        let mut band_random = splitter.split_str("minecraft:clay_bands");
        let mut terracotta_bands = vec![blocks.id("minecraft:terracotta"); 192];
        let orange = blocks.id("minecraft:orange_terracotta");
        let yellow = blocks.id("minecraft:yellow_terracotta");
        let brown = blocks.id("minecraft:brown_terracotta");
        let red = blocks.id("minecraft:red_terracotta");
        let white = blocks.id("minecraft:white_terracotta");
        let light_gray = blocks.id("minecraft:light_gray_terracotta");
        let mut i: i32 = 0;
        while i < 192 {
            i += band_random.next_int_bound(5) + 1;
            if i < 192 {
                terracotta_bands[i as usize] = orange;
            }
            i += 1; // C++ for 循环增量
        }
        add_terracotta_band(&mut band_random, &mut terracotta_bands, 1, yellow);
        add_terracotta_band(&mut band_random, &mut terracotta_bands, 2, brown);
        add_terracotta_band(&mut band_random, &mut terracotta_bands, 1, red);
        let ix = band_random.next_int_bound(7) + 9; // nextBetween(9,15)
        let mut j = 0;
        let mut k: i32 = 0;
        while j < ix && k < 192 {
            terracotta_bands[k as usize] = white;
            // C++ nextBoolean() = (impl.next() & 1ULL) != 0
            if k - 1 > 0 && (band_random.next() & 1) != 0 {
                terracotta_bands[(k - 1) as usize] = light_gray;
            }
            if k + 1 < 192 && (band_random.next() & 1) != 0 {
                terracotta_bands[(k + 1) as usize] = light_gray;
            }
            j += 1;
            k += band_random.next_int_bound(16) + 4;
        }
        SurfaceBuilder {
            samplers,
            splitter,
            sea_level,
            blocks,
            terracotta_bands,
        }
    }

    fn get_noise(&self, key: &str) -> &DoublePerlinNoiseSampler {
        self.samplers.get(key).expect("missing noise sampler").as_ref()
    }

    // 对齐 C++ L378-391 sampleRunDepth（surfaceDepth 列缓存）
    fn sample_run_depth(&self, block_x: i32, block_z: i32) -> i32 {
        thread_local! {
            static CACHE: RefCell<(i64, i32)> = RefCell::new((i64::MIN, 0));
        }
        let key = col_key(block_x, block_z);
        CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != key {
                c.0 = key;
                let d = self.get_noise("minecraft:surface").sample(block_x as f64, 0.0, block_z as f64);
                let extra = self.splitter.split_xyz(block_x, 0, block_z).next_double();
                c.1 = (d * 2.75 + 3.0 + extra * 0.25) as i32;
            }
            c.1
        })
    }

    #[allow(dead_code)] // 保留 API 对齐（get_secondary_depth 直接走 surface_secondary_noise）
    fn sample_secondary_depth(&self, block_x: i32, block_z: i32) -> f64 {
        self.get_noise("minecraft:surface_secondary").sample(block_x as f64, 0.0, block_z as f64)
    }

    // 对齐 C++ L395-413 getTerracottaBlock：按 y 索引红陶带
    fn get_terracotta_block(&self, x: i32, y: i32, z: i32) -> i32 {
        thread_local! {
            static CACHE: RefCell<(i64, f64)> = RefCell::new((i64::MIN, 0.0));
        }
        let key = col_key(x, z);
        let noise = CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != key {
                c.0 = key;
                c.1 = self.get_noise("minecraft:clay_bands_offset").sample(x as f64, 0.0, z as f64) * 4.0;
            }
            c.1
        });
        let i = noise.round() as i32; // std::lround
        let n = self.terracotta_bands.len() as i32;
        let idx = ((y + i) % n + n) % n;
        self.terracotta_bands[idx as usize]
    }

    // ========== 主世界规则树（对齐 C++ L485-682 / Java VanillaSurfaceRules.createDefaultRule(true,false,true)）==========
    pub fn build_overworld_rule(&self) -> SurfaceRule {
        let b = |name: &str| SurfaceRule::Block(self.blocks.id(&format!("minecraft:{}", name)));
        let air = self.blocks.id("minecraft:air");

        // materialCondition 1..13
        let mc1 = SurfaceCond::AboveY { anchor_y: 97, mult: 2, add_stone_depth: false };
        let mc2 = SurfaceCond::AboveY { anchor_y: 256, mult: 0, add_stone_depth: false };
        let mc3 = SurfaceCond::AboveY { anchor_y: 63, mult: -1, add_stone_depth: true }; // aboveYWithStoneDepth(fixed(63), -1)
        let mc4 = SurfaceCond::AboveY { anchor_y: 74, mult: 1, add_stone_depth: true };
        let mc5 = SurfaceCond::AboveY { anchor_y: 60, mult: 0, add_stone_depth: false };
        let mc6 = SurfaceCond::AboveY { anchor_y: 62, mult: 0, add_stone_depth: false };
        let mc7 = SurfaceCond::AboveY { anchor_y: 63, mult: 0, add_stone_depth: false };
        let mc8 = SurfaceCond::Water { offset: -1, mult: 0, add_stone_depth: false };
        let mc9 = SurfaceCond::Water { offset: 0, mult: 0, add_stone_depth: false };
        let mc10 = SurfaceCond::Water { offset: -6, mult: -1, add_stone_depth: true }; // waterWithStoneDepth(-6, -1)
        let mc11 = SurfaceCond::Hole;
        let mc12 = SurfaceCond::Biome {
            biomes: vec!["minecraft:frozen_ocean".into(), "minecraft:deep_frozen_ocean".into()],
        };
        let mc13 = SurfaceCond::Steep;

        // materialRule
        let mr = SurfaceRule::Seq(vec![
            SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("grass_block")) },
            b("dirt"),
        ]);
        let mr2 = SurfaceRule::Seq(vec![
            SurfaceRule::Cond {
                cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: true },
                rule: Box::new(b("sandstone")),
            },
            b("sand"),
        ]); // STONE_DEPTH_CEILING
        let mr3 = SurfaceRule::Seq(vec![
            SurfaceRule::Cond {
                cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: true },
                rule: Box::new(b("stone")),
            },
            b("gravel"),
        ]); // STONE_DEPTH_CEILING

        let mc14 = SurfaceCond::Biome {
            biomes: vec!["minecraft:warm_ocean".into(), "minecraft:beach".into(), "minecraft:snowy_beach".into()],
        };
        let mc15 = SurfaceCond::Biome { biomes: vec!["minecraft:desert".into()] };

        // materialRule4
        let mr4 = SurfaceRule::Seq(vec![
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:stony_peaks".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:calcite".into(), min_th: -0.0125, max_th: 0.0125 },
                        rule: Box::new(b("calcite")),
                    },
                    b("stone"),
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:stony_shore".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:gravel".into(), min_th: -0.05, max_th: 0.05 },
                        rule: Box::new(mr3.clone()),
                    },
                    b("stone"),
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:windswept_hills".into()] },
                rule: Box::new(SurfaceRule::Cond {
                    cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.0/8.25, max_th: f64::MAX },
                    rule: Box::new(b("stone")),
                }),
            },
            SurfaceRule::Cond { cond: mc14.clone(), rule: Box::new(mr2.clone()) },
            SurfaceRule::Cond { cond: mc15.clone(), rule: Box::new(mr2.clone()) },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:dripstone_caves".into()] },
                rule: Box::new(b("stone")),
            },
        ]);

        let mr5 = SurfaceRule::Cond {
            cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:powder_snow".into(), min_th: 0.45, max_th: 0.58 },
            rule: Box::new(SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("powder_snow")) }),
        };
        let mr6 = SurfaceRule::Cond {
            cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:powder_snow".into(), min_th: 0.35, max_th: 0.6 },
            rule: Box::new(SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("powder_snow")) }),
        };

        let mr7 = SurfaceRule::Seq(vec![
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:frozen_peaks".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond { cond: mc13.clone(), rule: Box::new(b("packed_ice")) },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:packed_ice".into(), min_th: -0.5, max_th: 0.2 },
                        rule: Box::new(b("packed_ice")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:ice".into(), min_th: -0.0625, max_th: 0.025 },
                        rule: Box::new(b("ice")),
                    },
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:snowy_slopes".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond { cond: mc13.clone(), rule: Box::new(b("stone")) },
                    mr5.clone(),
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:jagged_peaks".into()] },
                rule: Box::new(b("stone")),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:grove".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![mr5.clone(), b("dirt")])),
            },
            mr4.clone(),
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:windswept_savanna".into()] },
                rule: Box::new(SurfaceRule::Cond {
                    cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.75/8.25, max_th: f64::MAX },
                    rule: Box::new(b("stone")),
                }),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:windswept_gravelly_hills".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 2.0/8.25, max_th: f64::MAX },
                        rule: Box::new(mr3.clone()),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.0/8.25, max_th: f64::MAX },
                        rule: Box::new(b("stone")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -1.0/8.25, max_th: f64::MAX },
                        rule: Box::new(b("dirt")),
                    },
                    mr3.clone(),
                ])),
            },
            // Java materialRule7 结尾（122-123）：MANGROVE_SWAMP→MUD + DIRT fallback
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:mangrove_swamp".into()] },
                rule: Box::new(b("mud")),
            },
            b("dirt"),
        ]);

        let mc16 = SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -0.909, max_th: -0.5454 };
        let mc17 = SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -0.1818, max_th: 0.1818 };
        let mc18 = SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 0.5454, max_th: 0.909 };

        // materialRule8（海洋段使用，阈值与 mr7 不同）
        let mr8 = SurfaceRule::Seq(vec![
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:frozen_peaks".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond { cond: mc13.clone(), rule: Box::new(b("packed_ice")) },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:packed_ice".into(), min_th: 0.0, max_th: 0.2 },
                        rule: Box::new(b("packed_ice")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:ice".into(), min_th: 0.0, max_th: 0.025 },
                        rule: Box::new(b("ice")),
                    },
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:snowy_slopes".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond { cond: mc13.clone(), rule: Box::new(b("stone")) },
                    mr6.clone(),
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:jagged_peaks".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond { cond: mc13.clone(), rule: Box::new(b("stone")) },
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:grove".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    mr6.clone(),
                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) },
                ])),
            },
            mr4.clone(),
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:windswept_savanna".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.75/8.25, max_th: f64::MAX },
                        rule: Box::new(b("stone")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -0.5/8.25, max_th: f64::MAX },
                        rule: Box::new(b("coarse_dirt")),
                    },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:windswept_gravelly_hills".into()] },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 2.0/8.25, max_th: f64::MAX },
                        rule: Box::new(mr3.clone()),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.0/8.25, max_th: f64::MAX },
                        rule: Box::new(b("stone")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -1.0/8.25, max_th: f64::MAX },
                        rule: Box::new(mr.clone()),
                    },
                    mr3.clone(),
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome {
                    biomes: vec!["minecraft:old_growth_pine_taiga".into(), "minecraft:old_growth_spruce_taiga".into()],
                },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: 1.75/8.25, max_th: f64::MAX },
                        rule: Box::new(b("coarse_dirt")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface".into(), min_th: -0.95/8.25, max_th: f64::MAX },
                        rule: Box::new(b("podzol")),
                    },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:ice_spikes".into()] },
                rule: Box::new(SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(b("snow_block")) }),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:mangrove_swamp".into()] },
                rule: Box::new(b("mud")),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome { biomes: vec!["minecraft:mushroom_fields".into()] },
                rule: Box::new(b("mycelium")),
            },
            mr.clone(),
        ]);

        // 红陶带规则（terracottaBands 需 (x,y,z)）
        let bands_rule = SurfaceRule::TerracottaBands;

        let mr9 = SurfaceRule::Seq(vec![
            // STONE_DEPTH_FLOOR 段
            SurfaceRule::Cond {
                cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: false },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::Biome { biomes: vec!["minecraft:wooded_badlands".into()] },
                        rule: Box::new(SurfaceRule::Cond {
                            cond: mc1.clone(),
                            rule: Box::new(SurfaceRule::Seq(vec![
                                SurfaceRule::Cond { cond: mc16.clone(), rule: Box::new(b("coarse_dirt")) },
                                SurfaceRule::Cond { cond: mc17.clone(), rule: Box::new(b("coarse_dirt")) },
                                SurfaceRule::Cond { cond: mc18.clone(), rule: Box::new(b("coarse_dirt")) },
                                mr.clone(),
                            ])),
                        }),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::Biome { biomes: vec!["minecraft:swamp".into()] },
                        rule: Box::new(SurfaceRule::Cond {
                            cond: mc6.clone(),
                            rule: Box::new(SurfaceRule::Cond {
                                cond: SurfaceCond::Not(Box::new(mc7.clone())),
                                rule: Box::new(SurfaceRule::Cond {
                                    cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface_swamp".into(), min_th: 0.0, max_th: f64::MAX },
                                    rule: Box::new(b("water")),
                                }),
                            }),
                        }),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::Biome { biomes: vec!["minecraft:mangrove_swamp".into()] },
                        rule: Box::new(SurfaceRule::Cond {
                            cond: mc5.clone(),
                            rule: Box::new(SurfaceRule::Cond {
                                cond: SurfaceCond::Not(Box::new(mc7.clone())),
                                rule: Box::new(SurfaceRule::Cond {
                                    cond: SurfaceCond::NoiseThreshold { noise_key: "minecraft:surface_swamp".into(), min_th: 0.0, max_th: f64::MAX },
                                    rule: Box::new(b("water")),
                                }),
                            }),
                        }),
                    },
                ])),
            },
            // badlands 段
            SurfaceRule::Cond {
                cond: SurfaceCond::Biome {
                    biomes: vec!["minecraft:badlands".into(), "minecraft:eroded_badlands".into(), "minecraft:wooded_badlands".into()],
                },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: false },
                        rule: Box::new(SurfaceRule::Seq(vec![
                            SurfaceRule::Cond { cond: mc2.clone(), rule: Box::new(b("orange_terracotta")) },
                            SurfaceRule::Cond {
                                cond: mc4.clone(),
                                rule: Box::new(SurfaceRule::Seq(vec![
                                    SurfaceRule::Cond { cond: mc16.clone(), rule: Box::new(b("terracotta")) },
                                    SurfaceRule::Cond { cond: mc17.clone(), rule: Box::new(b("terracotta")) },
                                    SurfaceRule::Cond { cond: mc18.clone(), rule: Box::new(b("terracotta")) },
                                    bands_rule.clone(),
                                ])),
                            },
                            SurfaceRule::Cond {
                                cond: mc8.clone(),
                                rule: Box::new(SurfaceRule::Seq(vec![
                                    SurfaceRule::Cond {
                                        cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: true },
                                        rule: Box::new(b("red_sandstone")),
                                    },
                                    b("red_sand"),
                                ])),
                            },
                            SurfaceRule::Cond {
                                cond: SurfaceCond::Not(Box::new(mc11.clone())),
                                rule: Box::new(b("orange_terracotta")),
                            },
                            SurfaceRule::Cond { cond: mc10.clone(), rule: Box::new(b("white_terracotta")) },
                            mr3.clone(),
                        ])),
                    },
                    SurfaceRule::Cond {
                        cond: mc3.clone(),
                        rule: Box::new(SurfaceRule::Seq(vec![
                            SurfaceRule::Cond {
                                cond: mc7.clone(),
                                rule: Box::new(SurfaceRule::Cond {
                                    cond: SurfaceCond::Not(Box::new(mc4.clone())),
                                    rule: Box::new(b("orange_terracotta")),
                                }),
                            },
                            bands_rule.clone(),
                        ])),
                    },
                    // STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH
                    SurfaceRule::Cond {
                        cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: true, secondary_depth_range: 0, ceiling: false },
                        rule: Box::new(SurfaceRule::Cond { cond: mc10.clone(), rule: Box::new(b("white_terracotta")) }),
                    },
                ])),
            },
            // 海洋段
            SurfaceRule::Cond {
                cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: false },
                rule: Box::new(SurfaceRule::Cond {
                    cond: mc8.clone(),
                    rule: Box::new(SurfaceRule::Seq(vec![
                        SurfaceRule::Cond {
                            cond: mc12.clone(),
                            rule: Box::new(SurfaceRule::Cond {
                                cond: mc11.clone(),
                                rule: Box::new(SurfaceRule::Seq(vec![
                                    SurfaceRule::Cond { cond: mc9.clone(), rule: Box::new(SurfaceRule::Block(air)) },
                                    SurfaceRule::Cond { cond: SurfaceCond::Temp, rule: Box::new(b("ice")) },
                                    b("water"),
                                ])),
                            }),
                        },
                        mr8.clone(),
                    ])),
                }),
            },
            SurfaceRule::Cond {
                cond: mc10.clone(),
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: false },
                        rule: Box::new(SurfaceRule::Cond {
                            cond: mc12.clone(),
                            rule: Box::new(SurfaceRule::Cond { cond: mc11.clone(), rule: Box::new(b("water")) }),
                        }),
                    },
                    // STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH
                    SurfaceRule::Cond {
                        cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: true, secondary_depth_range: 0, ceiling: false },
                        rule: Box::new(mr7.clone()),
                    },
                    // RANGE_6
                    SurfaceRule::Cond {
                        cond: mc14.clone(),
                        rule: Box::new(SurfaceRule::Cond {
                            cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: true, secondary_depth_range: 6, ceiling: false },
                            rule: Box::new(b("sandstone")),
                        }),
                    },
                    // RANGE_30
                    SurfaceRule::Cond {
                        cond: mc15.clone(),
                        rule: Box::new(SurfaceRule::Cond {
                            cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: true, secondary_depth_range: 30, ceiling: false },
                            rule: Box::new(b("sandstone")),
                        }),
                    },
                ])),
            },
            SurfaceRule::Cond {
                cond: SurfaceCond::StoneDepth { offset: 0, add_surface_depth: false, secondary_depth_range: 0, ceiling: false },
                rule: Box::new(SurfaceRule::Seq(vec![
                    SurfaceRule::Cond {
                        cond: SurfaceCond::Biome {
                            biomes: vec!["minecraft:frozen_peaks".into(), "minecraft:jagged_peaks".into()],
                        },
                        rule: Box::new(b("stone")),
                    },
                    SurfaceRule::Cond {
                        cond: SurfaceCond::Biome {
                            biomes: vec!["minecraft:warm_ocean".into(), "minecraft:lukewarm_ocean".into(), "minecraft:deep_lukewarm_ocean".into()],
                        },
                        rule: Box::new(mr2.clone()),
                    },
                    mr3.clone(),
                ])),
            },
        ]);

        // 最终序列：bedrock_floor + surface(materialRule9) + deepslate
        let mut final_rules: Vec<SurfaceRule> = Vec::new();
        // bedrockFloor
        final_rules.push(SurfaceRule::Cond {
            cond: SurfaceCond::VerticalGradient { name: "minecraft:bedrock_floor".into(), true_y: -64, false_y: -59 },
            rule: Box::new(b("bedrock")),
        });
        // surface → materialRule9（surface=true）
        final_rules.push(SurfaceRule::Cond {
            cond: SurfaceCond::SurfaceCondC,
            rule: Box::new(mr9),
        });
        // deepslate：verticalGradient("deepslate", fixed(0), fixed(8))
        final_rules.push(SurfaceRule::Cond {
            cond: SurfaceCond::VerticalGradient { name: "minecraft:deepslate".into(), true_y: 0, false_y: 8 },
            rule: Box::new(b("deepslate")),
        });
        SurfaceRule::Seq(final_rules)
    }

    // ========== JSON surface_rule 数据驱动解析（多世界/非 overworld，对齐 C++ worldgen_api.cpp L263-343）==========
    // 任意维度的 surface_rule 从 noise_settings/<settings>.json 的 surface_rule 读，映射到 SurfaceRule/SurfaceCond。
    // 支持节点：rule = minecraft:sequence / condition / block；cond = not / biome / y_above / stone_depth /
    //          noise_threshold / vertical_gradient / hole / steep / water / temperature / surface。
    // 未支持节点 → 返回 None（调用方回退 overworld 规则 / 报错）。
    // 布尔感知字段读取（V4 修复，2026-09-09）：旧代码用 as_f64() 读 JSON 布尔字段，
    // JsonValue::Bool 走 as_f64() 返回 None → add_surface_depth/add_stone_depth 恒 false
    //（nether soul/gravel/patch 等全部分支的 stone_depth/y_above/water 修饰位丢失）。
    // 兼容数字写法（0/1）与缺失字段（false）。
    fn parse_bool_field(j: &crate::json::JsonValue, key: &str) -> bool {
        j.get(key)
            .and_then(|x| x.as_bool().or_else(|| x.as_f64().map(|f| f != 0.0)))
            .unwrap_or(false)
    }

    fn parse_anchor_abs_y(a: &crate::json::JsonValue, min_y: i32, height: i32) -> i32 {
        if let Some(v) = a.get("absolute") { return v.as_f64().unwrap_or(0.0) as i32; }
        if let Some(v) = a.get("above_bottom") { return min_y + v.as_f64().unwrap_or(0.0) as i32; }
        if let Some(v) = a.get("below_top") { return min_y + height - 1 - v.as_f64().unwrap_or(0.0) as i32; }
        0
    }
    pub fn parse_surface_rule(&self, j: &crate::json::JsonValue, min_y: i32, height: i32) -> Option<SurfaceRule> {
        let type_name = j.as_str().map(|s| s.to_string())
            .or_else(|| j.get("type").and_then(|t| t.as_str()).map(|s| s.to_string())).unwrap_or_default();
        if type_name.contains("sequence") {
            let mut rules = Vec::new();
            if let Some(seq) = j.get("sequence") {
                if let Some(arr) = seq.as_array() {
                    for r in arr {
                        match self.parse_surface_rule(r, min_y, height) {
                            Some(rr) => rules.push(rr),
                            None => eprintln!("[SURFACE-WARN] sequence 条目解析失败（类型={:?}），已跳过", r.get("type").and_then(|t| t.as_str()).unwrap_or("<cond>"))
                        }
                    }
                }
            }
            return Some(SurfaceRule::Seq(rules));
        }
        if type_name.contains("condition") {
            if let Some(c) = j.get("if_true") {
                let cond = match self.parse_surface_cond(c, min_y, height) { Some(cc) => cc, None => { eprintln!("[SURFACE-WARN] condition if_true 解析失败（类型={:?}），整条分支跳过", c.get("type").and_then(|t| t.as_str()).unwrap_or("<cond>")); return None; } };
                let rule = j.get("then_run").and_then(|r| self.parse_surface_rule(r, min_y, height));
                                if rule.is_none() { eprintln!("[SURFACE-WARN] then_run 解析失败，回退 Block(0)（写 air id）"); }
                return Some(SurfaceRule::Cond { cond, rule: Box::new(rule.unwrap_or(SurfaceRule::Block(0))) });
            }
        }
        if type_name.contains("block") {
            if let Some(rs) = j.get("result_state") {
                if let Some(n) = rs.get("Name") {
                    return Some(SurfaceRule::Block(self.blocks.id(n.as_str().unwrap_or(""))));
                }
            }
        }
        None
    }
    fn parse_surface_cond(&self, j: &crate::json::JsonValue, min_y: i32, height: i32) -> Option<SurfaceCond> {
        let type_name = j.as_str().map(|s| s.to_string())
            .or_else(|| j.get("type").and_then(|t| t.as_str()).map(|s| s.to_string())).unwrap_or_default();
        if type_name.contains("not") {
            if let Some(inv) = j.get("invert") {
                return Some(SurfaceCond::Not(Box::new(self.parse_surface_cond(inv, min_y, height)?)));
            }
        }
        if type_name.contains("biome") {
            let mut biomes = Vec::new();
            if let Some(b) = j.get("biome_is") {
                if let Some(arr) = b.as_array() {
                    for x in arr { if let Some(s) = x.as_str() { biomes.push(s.to_string()); } }
                }
            }
            return Some(SurfaceCond::Biome { biomes });
        }
        if type_name.contains("y_above") {
            if let Some(a) = j.get("anchor") {
                let anchor_y = Self::parse_anchor_abs_y(a, min_y, height);
                let add_stone_depth = Self::parse_bool_field(j, "add_stone_depth");
                // ⚠️ 跨版本风险点（260902-04 标注）：mult 硬编码 0——nether 的 y_above 条目
                // 全部 offset=0，恒等；但 1.20.1 之外若出现 offset≠0 的 y_above，此处会静默
                // 语义偏差。升级 MC 版本时 MUST 从 JSON anchor/offset 派生 mult，不再硬编码。
                return Some(SurfaceCond::AboveY { anchor_y, mult: 0, add_stone_depth });
            }
        }
        if type_name.contains("vertical_gradient") {
            if let Some(name) = j.get("random_name").and_then(|x| x.as_str()) {
                let true_y = j.get("true_at_and_below").map(|a| Self::parse_anchor_abs_y(a, min_y, height)).unwrap_or(i32::MIN);
                let false_y = j.get("false_at_and_above").map(|a| Self::parse_anchor_abs_y(a, min_y, height)).unwrap_or(i32::MAX);
                return Some(SurfaceCond::VerticalGradient { name: name.to_string(), true_y, false_y });
            }
        }
        if type_name.contains("stone_depth") {
            return Some(SurfaceCond::StoneDepth {
                offset: j.get("offset").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                add_surface_depth: Self::parse_bool_field(j, "add_surface_depth"),
                secondary_depth_range: j.get("secondary_depth_range").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                ceiling: j.get("surface_type").and_then(|x| x.as_str()).map(|s| s == "ceiling").unwrap_or(false),
            });
        }
        if type_name.contains("noise_threshold") {
            let min_th = j.get("min_threshold").and_then(|x| x.as_f64()).unwrap_or(-1.7e308);
            let max_th = j.get("max_threshold").and_then(|x| x.as_f64()).unwrap_or(1.7e308);
            let noise_key = j.get("noise").and_then(|n| n.as_str()).unwrap_or("").to_string();
            return Some(SurfaceCond::NoiseThreshold { noise_key, min_th, max_th });
        }
        if type_name.contains("vertical_gradient") {
            let name = j.get("random_name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let true_y = j.get("true_at_and_below").map(|a| Self::parse_anchor_abs_y(a, min_y, height)).unwrap_or(0);
            let false_y = j.get("false_at_and_above").map(|a| Self::parse_anchor_abs_y(a, min_y, height)).unwrap_or(0);
            return Some(SurfaceCond::VerticalGradient { name, true_y, false_y });
        }
        if type_name.contains("hole") { return Some(SurfaceCond::Hole); }
        if type_name.contains("steep") { return Some(SurfaceCond::Steep); }
        if type_name.contains("water") {
            // ⚠️ 跨版本风险点（260902-04 标注）：mult 硬编码 0（overworld 代码规则的
            // WaterCond 才用 mult；nether JSON water(-1,0)/water(0,0) 恒等，当前无影响）。
            // 出现 offset≠0 或 fluid 非水的 water 条目时此处会静默语义偏差，升级时 MUST 派生。
            return Some(SurfaceCond::Water {
                offset: j.get("offset").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32,
                mult: 0,
                add_stone_depth: Self::parse_bool_field(j, "add_stone_depth"),
            });
        }
        if type_name.contains("temperature") { return Some(SurfaceCond::Temp); }
        if type_name.contains("surface") { return Some(SurfaceCond::SurfaceCondC); }
        None
    }

    // ========== buildSurface 引擎（对齐 C++ L685-811 / Java SurfaceBuilder.buildSurface）==========
    pub fn build_surface(
        &self,
        col: &mut BlockColumn,
        rule: &SurfaceRule,
        chunk_start_x: i32,
        chunk_start_z: i32,
        heightmap_in: &[i32],
        surface_heights4: &[i32],
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        biome_cell_key: &dyn Fn(i32, i32, i32) -> i64,
        biome_temp: &dyn Fn(&str) -> f64,
        min_y: i32,
        world_height: i32,
        initial_density_at: &dyn Fn(i32, i32, i32) -> f64,
    ) {
        // Java buildSurface 中 placeBadlandsPillar 会 trackUpdate 实时更新 heightmap，
        // SteepSlopePredicate 等读到的是 pillar 后高度；此处用可变副本（逐列 pillar 写回，对齐 Java 逐列顺序）
        let heightmap = RefCell::new(heightmap_in.to_vec());
        let mut ctx = SurfaceContext {
            block_x: 0,
            block_y: 0,
            block_z: 0,
            world_min_y: min_y,
            world_height,
            surface_depth: 0,
            stone_depth_above: 0,
            stone_depth_below: 0,
            fluid_height: i32::MIN,
            biome_id: String::new(),
            biome_temp: 0.5,
            noise_samplers: self.samplers,
            splitter: self.splitter,
            initial_density_at: Some(initial_density_at),
            terracotta_bands_getter: Some(&|x, y, z| self.get_terracotta_block(x, y, z)),
            column_heightmap: Some(&heightmap),
            surface_heights4: Some(surface_heights4),
            surface_secondary_noise: Some(self.get_noise("minecraft:surface_secondary")),
            derived_splitters: RefCell::new(HashMap::new()),
            secondary_cache_key: Cell::new(i64::MIN),
            secondary_cache: Cell::new(0.0),
        };

        let default_block = self.blocks.id("minecraft:stone");
        let air_block = self.blocks.id("minecraft:air");
        let water_block = self.blocks.id("minecraft:water");
        let lava_block = self.blocks.id("minecraft:lava");
        let world_min_y = min_y;
        let world_top_y = min_y + world_height;

        // biome 缓存：key = 8 邻域选点结果 (px,py,pz) packed（同一选点 cell 共享 find 结果）
        let mut biome_cache: HashMap<i64, (String, f64)> = HashMap::new();
        let mut biome_at_cached = |bx: i32, by: i32, bz: i32| -> (String, f64) {
            let key = biome_cell_key(bx, by, bz);
            if let Some(v) = biome_cache.get(&key) {
                return v.clone();
            }
            let id = biome_at(bx, by, bz);
            let t = biome_temp(&id);
            let r = (id, t);
            biome_cache.insert(key, r.clone());
            r
        };

        // V4 诊断门控：chunk 级取一次（None = 未配置，循环内零成本）
        let dump_points: Option<&std::collections::HashSet<(i32, i32, i32)>> = soul_dump_points().as_ref();

        for k in 0..16 {
            for l in 0..16 {
                let m = chunk_start_x + k;
                let n = chunk_start_z + l;
                let idx = l * 16 + k; // heightmap 索引 z*16+x（fillOneChunk 填充语义）
                // Java L117：o = WORLD_SURFACE_WG + 1（pillar 前表面高度）
                let o = heightmap.borrow()[idx as usize] + 1;
                ctx.block_x = m;
                ctx.block_z = n;
                // Java L119-121：biome 采样在 pillar 前（getBiome(m, o, n)），仅 eroded_badlands 触发 pillar
                let pillar_biome = biome_at_cached(m, o, n);
                let mut column_h = heightmap.borrow()[idx as usize]; // 列表面高度（pillar 可能抬升；heightmap 为可变副本）
                if pillar_biome.0 == "minecraft:eroded_badlands" {
                    self.place_badlands_pillar(col, m, n, k, l, o, &mut column_h, min_y, world_top_y);
                }
                // Java trackUpdate 等效：pillar 抬升写回 heightmap（SteepCond 读 pillar 后高度）
                heightmap.borrow_mut()[idx as usize] = column_h;
                // Java L124：pillar 后重采样 heightmap + 1（有 pillar 且 surfaceY<=j 时 = j+2）
                let p = column_h + 1;
                ctx.surface_depth = self.sample_run_depth(m, n);

                let mut q = 0;
                let mut r = i32::MIN; // 最高流体 y + 1
                let mut s = i32::MAX; // 第一个非 default 块位置

                let mut wy = p;
                while wy >= min_y {
                    let state = if wy >= world_top_y {
                        air_block // 世界高度以上视为空气（vanilla HeightLimitView 越界返回 AIR）
                    } else {
                        col.at(k, wy, l)
                    };
                    let is_air = state == air_block;
                    let is_fluid = state == water_block || state == lava_block;
                    if is_air {
                        q = 0;
                        r = i32::MIN;
                    } else if is_fluid {
                        if r == i32::MIN {
                            r = wy + 1;
                        }
                    } else {
                        if s >= wy {
                            s = i32::MAX;
                            let mut v = wy - 1;
                            while v >= min_y - 1 {
                                let st2 = if v < world_min_y {
                                    air_block // 世界底以下视为空气
                                } else {
                                    col.at(k, v, l)
                                };
                                if st2 != air_block && st2 != water_block && st2 != lava_block {
                                    v -= 1;
                                    continue; // 找到 default 块 → 继续向上找非 default
                                }
                                s = v + 1;
                                break;
                            }
                        }
                        q += 1;
                        let vx = wy - s + 1;
                        let b = biome_at_cached(m, wy, n);
                        ctx.init_vertical(q, vx, r, m, wy, n, &b.0);
                        ctx.biome_temp = b.1;
                        if state == default_block {
                            let new_state = rule.apply(&ctx);
                            if let Some(set) = dump_points {
                                if set.contains(&(m, wy, n)) {
                                    // 生产 ctx 的 soul 分支入口判定（镜像 SurfaceCond::StoneDepth L84-93
                                    // nether 参数：offset=0, add_surface_depth=true, secondary=0）
                                    let selector = noise_threshold_sample(&ctx, "minecraft:nether_state_selector");
                                    let ceiling_ok = ctx.stone_depth_below <= 1 + 0 + ctx.surface_depth + 0;
                                    let floor_ok = ctx.stone_depth_above <= 1 + 0 + ctx.surface_depth + 0;
                                    let applied_s = match new_state {
                                        Some(b) => format!("id={}", b),
                                        None => "none".to_string(),
                                    };
                                    eprintln!("[SOUL-CTX] {},{},{},biome={},sda={},sdb={},surface_depth={},fluid_height={},selector={:.6},ceiling_ok={},floor_ok={},is_default=true,applied={}",
                                        m, wy, n, ctx.biome_id, ctx.stone_depth_above, ctx.stone_depth_below,
                                        ctx.surface_depth, ctx.fluid_height, selector, ceiling_ok, floor_ok, applied_s);
                                }
                            }
                            if let Some(new_state) = new_state {
                                *col.at_mut(k, wy, l) = new_state;
                            }
                        }
                    }
                    wy -= 1;
                }
            }
        }
    }

    // 对齐 C++ L429-451 applyMaterialRuleSingle（carver 挖掉 grass 后 dirt 单点替换）
    // ⚠️ 决策：Java applyMaterialRule 经 initHorizontalContext 设 runDepth=sampleRunDepth，
    // C++ 此处 surfaceDepth 留 0（bug）。此处按 Java 设 surface_depth = sample_run_depth(x,z)。
    pub fn apply_material_rule_single(
        &self,
        rule: &SurfaceRule,
        biome_at: &dyn Fn(i32, i32, i32) -> String,
        biome_temp: &dyn Fn(&str) -> f64,
        x: i32,
        y: i32,
        z: i32,
        has_fluid: bool,
        min_y: i32,
        world_height: i32,
        initial_density_at: &dyn Fn(i32, i32, i32) -> f64,
    ) -> Option<BlockId> {
        let mut ctx = SurfaceContext {
            block_x: 0,
            block_y: 0,
            block_z: 0,
            world_min_y: min_y,
            world_height,
            surface_depth: self.sample_run_depth(x, z),
            stone_depth_above: 0,
            stone_depth_below: 0,
            fluid_height: i32::MIN,
            biome_id: String::new(),
            biome_temp: 0.5,
            noise_samplers: self.samplers,
            splitter: self.splitter,
            initial_density_at: Some(initial_density_at),
            terracotta_bands_getter: Some(&|x, y, z| self.get_terracotta_block(x, y, z)),
            column_heightmap: None,
            surface_heights4: None,
            surface_secondary_noise: Some(self.get_noise("minecraft:surface_secondary")),
            derived_splitters: RefCell::new(HashMap::new()),
            secondary_cache_key: Cell::new(i64::MIN),
            secondary_cache: Cell::new(0.0),
        };
        let biome = biome_at(x, y, z);
        ctx.biome_id = biome.clone();
        ctx.biome_temp = biome_temp(&biome);
        // Java MaterialRuleContext.initVerticalContext(1, 1, hasFluid ? j+1 : MIN, i, j, k)
        ctx.init_vertical(1, 1, if has_fluid { y + 1 } else { i32::MIN }, x, y, z, &biome);
        rule.apply(&ctx)
    }

    // 对齐 C++ L813-850 placeBadlandsPillar（eroded_badlands 支柱填充）
    fn place_badlands_pillar(
        &self,
        col: &mut BlockColumn,
        wx: i32,
        wz: i32,
        cx: i32,
        cz: i32,
        surface_y: i32,
        column_height: &mut i32,
        bottom_y: i32,
        world_top_y: i32,
    ) {
        let default_block = self.blocks.id("minecraft:stone");
        let air_block = self.blocks.id("minecraft:air");
        let water_block = self.blocks.id("minecraft:water");
        // Java L210：e = min(|badlands_surface(x,0,z)*8.25|, badlands_pillar(x*0.2,0,z*0.2)*15.0)
        // 注意 badlands_surface 用原始坐标，badlands_pillar 用 x*0.2/z*0.2；pillar 项无 abs（可为负 → e<=0 跳过）
        let e = (self.get_noise("minecraft:badlands_surface").sample(wx as f64, 0.0, wz as f64) * 8.25).abs().min(
            self.get_noise("minecraft:badlands_pillar").sample(wx as f64 * 0.2, 0.0, wz as f64 * 0.2) * 15.0,
        );
        if e <= 0.0 {
            return; // Java L211：!(e <= 0.0) 才继续
        }
        // Java L214：h = |badlands_pillar_roof(x*0.75,0,z*0.75)*1.5|
        let h = (self.get_noise("minecraft:badlands_pillar_roof").sample(wx as f64 * 0.75, 0.0, wz as f64 * 0.75) * 1.5).abs();
        // Java L215-216：i = 64.0 + min(e*e*2.5, ceil(h*50.0)+24.0)；j = MathHelper.floor(i)（向 -inf）
        let i = 64.0 + (e * e * 2.5).min((h * 50.0).ceil() + 24.0);
        let j = i.floor() as i32;
        if surface_y > j {
            return; // Java L217：surfaceY <= j 才填充
        }
        // Java L218-227 校验循环：从 j 向下，遇 stone break、遇 water 整列 return
        // 越界读（y >= worldTopY）按 Java BlockColumn.getState 返回 AIR 处理
        let mut y = j;
        while y >= bottom_y {
            let state = if y >= world_top_y { air_block } else { col.at(cx, y, cz) };
            if state == default_block {
                break;
            }
            if state == water_block {
                return;
            }
            y -= 1;
        }
        // Java L229-231 填充循环：从 j 向下 while air → defaultState(stone)；越界（y >= worldTopY）不写
        let mut filled = false;
        let mut y = j;
        while y >= bottom_y {
            let state = if y >= world_top_y { air_block } else { col.at(cx, y, cz) };
            if state != air_block {
                break;
            }
            if y < world_top_y {
                *col.at_mut(cx, y, cz) = default_block;
                filled = true;
            }
            y -= 1;
        }
        // Java trackUpdate 等效：填充首个 y=j → heightmap = j+1（后续 y<j 不触发 y<=i-2 更新）
        if filled {
            *column_height = (*column_height).max(j + 1);
        }
    }
}

// 对齐 C++ L453-460 addTerracottaBand
fn add_terracotta_band(r: &mut RsRandom, bands: &mut Vec<BlockId>, min_band_size: i32, state: BlockId) {
    let i = r.next_int_bound(10) + 6; // nextBetween(6,15)
    for _ in 0..i {
        let k = min_band_size + r.next_int_bound(3);
        let l = r.next_int_bound(bands.len() as i32);
        let mut m = 0;
        while l + m < bands.len() as i32 && m < k {
            bands[(l + m) as usize] = state;
            m += 1;
        }
    }
}

// ========== biome 温度表（对齐 C++ biomeTemp 用法；数据源 versions/1.20.1/data/worldgen/.../biome/*.json）==========
// TempCond 判定 `biome_temp < 0.15`（Java Biome.isCold）。默认 0.5（>=0.15 → TempCond false）。
pub fn biome_temperature(biome_id: &str) -> f64 {
    match biome_id {
        "minecraft:frozen_ocean" => 0.0,
        "minecraft:frozen_river" => 0.0,
        "minecraft:ice_spikes" => 0.0,
        "minecraft:snowy_plains" => 0.0,
        "minecraft:snowy_beach" => 0.05,
        "minecraft:old_growth_spruce_taiga" => 0.25,
        "minecraft:taiga" => 0.25,
        "minecraft:snowy_taiga" => -0.5,
        "minecraft:old_growth_pine_taiga" => 0.3,
        "minecraft:snowy_slopes" => -0.3,
        "minecraft:grove" => -0.2,
        "minecraft:frozen_peaks" => -0.7,
        "minecraft:jagged_peaks" => -0.7,
        "minecraft:stony_shore" => 0.2,
        "minecraft:windswept_forest" => 0.2,
        "minecraft:windswept_gravelly_hills" => 0.2,
        "minecraft:windswept_hills" => 0.2,
        "minecraft:stony_peaks" => 1.0,
        "minecraft:beach" => 0.8,
        "minecraft:plains" => 0.8,
        "minecraft:swamp" => 0.8,
        "minecraft:mangrove_swamp" => 0.8,
        "minecraft:desert" => 2.0,
        "minecraft:badlands" => 2.0,
        "minecraft:eroded_badlands" => 2.0,
        "minecraft:wooded_badlands" => 2.0,
        "minecraft:windswept_savanna" => 2.0,
        "minecraft:savanna" => 2.0,
        "minecraft:savanna_plateau" => 2.0,
        "minecraft:mushroom_fields" => 0.9,
        "minecraft:warm_ocean" => 0.5,
        "minecraft:lukewarm_ocean" => 0.5,
        "minecraft:deep_lukewarm_ocean" => 0.5,
        "minecraft:ocean" => 0.5,
        "minecraft:deep_ocean" => 0.5,
        "minecraft:cold_ocean" => 0.5,
        "minecraft:deep_cold_ocean" => 0.5,
        "minecraft:deep_frozen_ocean" => 0.5,
        "minecraft:river" => 0.5,
        "minecraft:meadow" => 0.5,
        "minecraft:cherry_grove" => 0.5,
        "minecraft:birch_forest" => 0.6,
        "minecraft:old_growth_birch_forest" => 0.6,
        "minecraft:dark_forest" => 0.7,
        "minecraft:forest" => 0.7,
        "minecraft:flower_forest" => 0.7,
        "minecraft:jungle" => 0.95,
        "minecraft:bamboo_jungle" => 0.95,
        "minecraft:sparse_jungle" => 0.95,
        "minecraft:dripstone_caves" => 0.8,
        "minecraft:lush_caves" => 0.5,
        "minecraft:deep_dark" => 0.8,
        _ => 0.5,
    }
}

// ========== 静态自检清单（未编译验证）==========
// ① 类型宽度：所有坐标/深度/高度用 i32（对齐 C++ int32_t）；温度/噪声用 f64；next_float 返回 f32（比较时 as f64）。
//    StoneDepth k 用 `as i32`（向零截断，对齐 Java (int)）；SurfaceCondC k 用 `.floor() as i32`（向 -inf）。
// ② move 语义：SurfaceCond/SurfaceRule 均 #[derive(Clone)]，规则树内复用条件/规则用 .clone()，无悬垂引用。
//    SurfaceContext 持有 &'a 引用（samplers/splitter/heightmap/surfaceHeights4/闭包），struct literal 统一生命周期。
// ③ 缓存：NoiseThreshold/estimateSurfaceHeight/sampleRunDepth/getTerracottaBlock 用 thread_local（每线程独立，多线程安全）。
//    getSecondaryDepth 用 Cell 列缓存（ctx 每 chunk 单线程）。
// ④ 与 C++ 逐行对拍点：见下方注释。
//
// 与 C++ 对拍点清单：
//   - SurfaceCond::Biome/AboveY/Water/StoneDepth/NoiseThreshold/Hole/Steep/SurfaceCondC/Temp/VerticalGradient/Not
//     ↔ C++ L210-296（Hole 用 surface_depth 修正；Steep 用 z*16+x 索引修正）
//   - SurfaceRule::Block/Cond/Seq/TerracottaBands ↔ C++ L44-64/L212-218/L294-296
//   - SurfaceContext::estimate_surface_height/init_vertical/splitter_for/get_secondary_depth ↔ C++ L127-208
//   - SurfaceBuilder::build_overworld_rule ↔ C++ L485-682（mr1-mr9 + bedrock_floor + surface + deepslate）
//   - SurfaceBuilder::build_surface ↔ C++ L685-811（逐列扫描 + pillar + 规则应用）
//   - SurfaceBuilder::place_badlands_pillar ↔ C++ L813-850
//   - biome_temperature ↔ C++ biomeTemp 用法（TempCond < 0.15）






