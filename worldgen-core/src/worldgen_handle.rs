// worldgen_handle.rs — Rust worldgen 生产句柄（C ABI 的 Rust 侧实现）
// 封装块级管线：fill_chunk（宏观）→ BlockColumn → build_surface（具体 block id）→ carver（洞穴）。
// 对齐 C++ WorldgenHandle（worldgen_api.cpp wg_create / wg_fill_blocks_multi）。
// 数据源：worldgen_dir 指向 vanilla worldgen JSON 数据目录（含 data/minecraft/worldgen/...）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::beardifier::Beardifier;
use crate::biome::BiomeClassifier;
use crate::blocks::{BlockColumn, BlockId, BlockRegistry};
use crate::carver::{CarverContext, CarvingMask, ConfiguredCarver};
use crate::chunkrandom::ChunkRandom;
use crate::density::{DensityFunction, NoisePos};
use crate::density_builder::DensityBuilder;
use crate::json::parse;
use crate::surface_rules::{SurfaceBuilder, SurfaceRule};
use crate::terrain::{fill_chunk, VanillaAquifer, VanillaDensity, BiomeSource};
use crate::legacy_random::RsSplitter;
use crate::xoroshiro::XoroshiroSplitter;

/// 260903-13 翻默认配套：env 开关语义 = 默认启用，显式设 "0" 关闭（其余值视为开）。
/// 用于 WG_EST_SHARED / WG_EST_L2（est 优化，四臂 hash 已证零语义差）。
pub fn env_enabled(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) => v != "0",
        Err(_) => true,
    }
}

// 宏观 biome 源（BiomeClassifier + 6 参数 DF）
struct MacroBiome {
    bc: BiomeClassifier,
    tempf: Arc<DensityFunction>,
    humf: Arc<DensityFunction>,
    contf: Arc<DensityFunction>,
    erof: Arc<DensityFunction>,
    depthf: Arc<DensityFunction>,
    weirdf: Arc<DensityFunction>,
    // end 维度（260906-04）：TheEndBiomeSource 位置判定（非 MultiNoise，02 篇）。
    // true 时 biome() 走 end 分类；bc 仅承载 carvers/features 注册（判定不用）。
    end_mode: bool,
}
impl BiomeSource for MacroBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        if self.end_mode {
            return end_biome_of(&self.erof, pos);
        }
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}

// TheEndBiomeSource.getBiome（TheEndBiomeSource.java L67-87，02 篇 §1 逐行移植）：
// 输入 = 块坐标（Rust 侧 biome_at 闭包传 (x>>2)<<2 对齐块坐标；Java 输入 biome 坐标 <<2 同值）。
// section = 块坐标 >> 4（ChunkSectionPos.getSectionCoord）；biome 格 4 块 ⊂ section 16 块，
// 对齐后同一 section 内判定一致 ⇒ 纯二维 section 粒度（cache_2d → y 无关）。
fn end_biome_of(erof: &DensityFunction, pos: &NoisePos) -> String {
    let sx = (pos.x >> 4) as i64;
    let sz = (pos.z >> 4) as i64;
    if sx * sx + sz * sz <= 4096 {
        return "minecraft:the_end".to_string();
    }
    // 采样点 = section 中心块 (sx*2+1)*8（L76-77）；y 传 pos.y（cache_2d 忽略）
    let n = ((sx * 2 + 1) * 8) as i32;
    let o = ((sz * 2 + 1) * 8) as i32;
    let d = erof.sample(&NoisePos { x: n, y: pos.y, z: o });
    // 阈值边界归属（L79-84）：d==0.25 → midlands；d==-0.0625 → midlands；d==-0.21875 → barrens
    if d > 0.25 { "minecraft:end_highlands".to_string() }
    else if d >= -0.0625 { "minecraft:end_midlands".to_string() }
    else if d < -0.21875 { "minecraft:small_end_islands".to_string() }
    else { "minecraft:end_barrens".to_string() }
}

// 生产句柄：一次 seed 初始化（构建全部 noise samplers + density 树 + biome + surface）。
pub struct WorldgenHandle {
    pub seed: i64,
    pub biome_access_seed: i64, // BiomeAccess.hashSeed(seed)（sha256-asLong；ChunkRegion L102 同构）
    pub min_y: i32,
    pub height: i32,
    pub noise_height: i32, // 噪声高度（settings noise.height；nether 128 < world 256——density 采样有效域，上方留 air）
    pub sea_level: i32, // settings sea_level（下界 32）——aquifer 禁用时的流体面
    pub aquifers_enabled: bool, // from noise_settings <settings>.aquifers_enabled（下界 false 跳过 aquifer）
    // density 树
    tree: Arc<DensityFunction>,       // final_density
    // multi-channel 宏观采样器（对齐 Java NoiseChunk cell grid；fill_chunk 用 cell grid 采样 density）
    macro_sampler: crate::terrain::DensityMacroSampler,
    // transpiler 宏观采样器（build-time 编译 density 树，WG_TRANSPILER env 时启用；None = 用 macro_sampler）
    transpiler_density: Option<crate::terrain::TranspilerDensity>,
    // DFC 宏观采样器（lossless-accel P2a，WG_DFC env 时启用，优先级 > WG_TRANSPILER；None = 回退）
    dfc_density: Option<crate::terrain::DfcDensity>,
    // GPU 密度源（路线② 260903-05，WG_GPU_DENSITY env 时启用，优先级 > WG_DFC；None = 回退）
    gpu_density: Option<crate::gpu_ffi::GpuDensity>,
    // GPU channels 角点源（X2 260903-05，WG_GPU_CHANNELS env 时启用，优先级最高；None = 回退）
    gpu_channels: Option<crate::gpu_ffi::GpuChannelDensity>,
    barrier: Arc<DensityFunction>,
    flooded: Arc<DensityFunction>,
    spread: Arc<DensityFunction>,
    lava: Arc<DensityFunction>,
    erosion: Arc<DensityFunction>,
    depth: Arc<DensityFunction>,
    init: Arc<DensityFunction>,        // initial_density_without_jaggedness
    // biome
    biomesrc: MacroBiome,
    // surface
    sb: SurfaceBuilder<'static>,
    rule: SurfaceRule,
    // blocks（Box::leak 长期存活，SurfaceBuilder 与 carver 共用）
    blocks: &'static BlockRegistry,
    // 260907-05（缺口 2）：settings.default_block 解析后的 id（fill/surface 的默认填充块）
    default_block: BlockId,
    // aquifer splitter
    splitter: XoroshiroSplitter,
    // 矿脉（ore vein）sampler（density 后 aquifer 无 fluid 时决定矿脉块，只读 &self 并发安全）
    ore_vein: crate::ore_vein::OreVeinSampler,
    // beardifier 缓存（per-chunk，CppBridge set_beardifier 写，fill 读——RwLock 读并发无争用）
    beardifiers: std::sync::RwLock<HashMap<(i32, i32), Beardifier>>,
    // carver 缓存（创建时预加载，运行只读无锁）
    carver_cache: HashMap<String, ConfiguredCarver>,
    // SteelMC uniform_carver_biome 优化：若所有 biome carvers 统一，apply_carvers 跳过 289 次 biome 采样
    uniform_carver_list: Option<Vec<String>>,
    // FEATURES 缓存（创建时从所有 biome features 预加载，运行只读无锁）
    feature_cache: crate::feature_loader::FeatureCache,
    // FEATURES indexer（Java PlacedFeatureIndexer，从所有 biome features 构建，构建后只读——&self 共享并发安全，不需锁）
    feature_indexer: crate::feature_loader::PlacedFeatureIndexer,
    // 数据目录
    wg_dir: String,
    // 句柄级阶段开关（2026-09-08 双跑修复，judge CONCERN：env 门是进程全局，需句柄级 flag）。
    // bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE；0 = 未设置 → 回落 env 判定（兼容 bin-diag/probe）。
    // skip 方向 OR：flag 位或 env 任一命中即 skip。存档链路由 CppBridge 设 flag，standalone 工具零改动。
    pub flags: std::sync::atomic::AtomicU32,
    // b1-b 跨 chunk est L2（260903-13 翻默认：默认启用，WG_EST_L2=0 关）：OnceLock 惰性建（首次 fill 时按 env 决定），
    // Arc 跨 chunk 共享；挂 handle → (seed,params) 代际隔离天然成立。blend 闸门见 aquifer::BLEND_ACTIVE。
    est_l2: std::sync::OnceLock<Option<std::sync::Arc<std::sync::Mutex<crate::aquifer::EstL2>>>>,
    // c-A-min（260905-09，方案 260905-08-cA-block-boundary.md §4；门控 WG_CA_MIN）：
    // 260905-10 用户拍板：默认关（显式 is_ok_and 判存在，勿用 env_enabled——其 unset→true 默认开，#53 三查）。
    // c-A 为行为修正（feature 跨 chunk 读写钩子）非诊断探针；性能 +68% 待优化后另议翻默认。
    // 邻 chunk 地形列缓存（noise+surface+carver，无 feature——对齐 Java「FEATURES 时邻 chunk ≥ post-carver
    // 地形态」时序保证，IDK-cA1：邻 chunk feature 时序近似为无）。block_at 越界读改走此缓存。
    terrain_cache: std::sync::Mutex<HashMap<(i32, i32), std::sync::Arc<crate::blocks::BlockColumn>>>,
    // c-A-write（同门控）：跨 chunk 写缓冲——Java center 写邻 chunk 无条件持久（ProtoChunk 落盘）；
    // 目标 chunk features 开始前 overlay。顺序残余差 IDK-cA2：先于 center 生成的邻 chunk 收不到 overlay。
    pending_cross_writes: std::sync::Mutex<HashMap<(i32, i32), Vec<(usize, BlockId)>>>,
}

// flags 位定义（与 wg_set_flags / Java CppBridge 对齐）
pub const FLAG_SKIP_CARVER: u32 = 1 << 0;
pub const FLAG_SKIP_FEATURES: u32 = 1 << 1;
pub const FLAG_SKIP_SURFACE: u32 = 1 << 2;

// #26 判据 1（260903-15）：overworld surface 噪声 key 单一事实源在 surface_rules.rs
// （ENGINE_NOISE_KEYS，紧邻 get_noise 引擎调用点就近维护）——本文件只 use。
// 启动期断言：collect_rule_noise_keys 收集规则树运行时引用 key ⊆ 预加载集合（base 3 之外），
// 缺失即 panic。根因背景：260903-14 panic 505——place_badlands_pillar 运行时查
// badlands_pillar_roof，预加载静态清单漏该 key，仅 eroded_badlands 低频分支触发，小样本全绿掩盖。
use crate::surface_rules::ENGINE_NOISE_KEYS as OVERWORLD_NOISE_KEYS;

impl WorldgenHandle {
    // 便捷入口：overworld 默认维度（保留既有 probe 调用兼容）。
    pub fn create(seed: i64, worldgen_dir: &str) -> Option<WorldgenHandle> {
        Self::create_for_dim(seed, worldgen_dir, "overworld.json", "biome_params.json", 384)
    }

    // 从 worldgen_dir 构建句柄（**多世界参数化**，对齐 C++ wg_create）。
    // worldgen_dir 指向含 data/minecraft/worldgen/... 的目录。
    // 参数：
    //   settings_name   : noise_settings/<settings_name>.json（"overworld.json" / "nether.json" / mod 维度文件名）
    //                     settings 里的 noise_router 决定 density_function/<dfNs>/ 目录（dfNs = settings_name 去 .json）
    //   biome_params_file: biome 参数文件（overworld 用 biome_params.json，nether 用 biome_params_nether.json，mod 维度自定义）
    //   world_height    : 世界高度（维度定义；overworld 384 / nether 256 / mod 维度按定义；0 = 从 noise.height 兜底）
    // 数据文件约定：
    //   <dir>/data/minecraft/worldgen/noise_settings/<settings_name>.json
    //   <dir>/data/minecraft/worldgen/density_function/<dfNs>/*.json
    //   <dir>/../noise_params.json
    //   <dir>/../<biome_params_file>
    //   <dir>/../blocks.json
    pub fn create_for_dim(seed: i64, worldgen_dir: &str,
                          settings_name: &str, biome_params_file: &str,
                          world_height: i32) -> Option<WorldgenHandle> {
        let wg_dir = worldgen_dir.to_string();
        // dfNs = settings_name 去 ".json"（决定 density_function namespace/目录）
        let df_ns_raw = if settings_name.ends_with(".json") {
            &settings_name[..settings_name.len() - 5]
        } else { settings_name };
        // 260907-05（A 组缺口 3）：settings_name 支持 "modid:name[.json]" → 数据命名空间 modid；
        // 无冒号 = minecraft（向后兼容，vanilla 路径逐字节不变）。
        let (data_ns, df_ns) = match df_ns_raw.split_once(':') {
            Some((ns, short)) => (ns.to_string(), short.to_string()),
            None => ("minecraft".to_string(), df_ns_raw.to_string()),
        };

        // 2. noise_settings（先读维度参数：min_y/height/aquifers_enabled）
        let settings_path = format!("{}/data/{}/worldgen/noise_settings/{}.json", wg_dir, data_ns, df_ns);
        let settings_txt = std::fs::read_to_string(&settings_path).ok()?;
        let settings = parse(&settings_txt).ok()?;
        // 260907-05（A 组缺口 2）：default_block 从 settings 读（JSON 优先；字符串或 {Name} 形态），
        // 缺失 fallback minecraft:stone + 一次性日志（对齐 260907-04 tag 先例）
        let mut default_block_name = settings.get("default_block")
            .and_then(|v| v.as_str().map(|s| s.to_string())
                .or_else(|| v.get("Name").and_then(|n| n.as_str()).map(|s| s.to_string())))
            .unwrap_or_else(|| {
                eprintln!("[WGH] settings '{}' no default_block, fallback minecraft:stone", df_ns);
                "minecraft:stone".to_string()
            });
        // 诊断覆盖（260907-05，创建期读一次，非热路径；A/B 对拍默认块用，#20 恒等式自检随用随证）
        // 260907-09：加一次性行为化日志（#37 判据：env 生效证据必须行为化，沉默 override 不可判别）
        if let Ok(ov) = std::env::var("CORESWAP_DEFAULT_BLOCK") {
            eprintln!("[WGH] env override CORESWAP_DEFAULT_BLOCK={} (was {})", ov, default_block_name);
            default_block_name = ov;
        }
        // 维度参数从 settings 读（非硬编码 overworld -64/384）
        let mut min_y = -64;
        let mut noise_height = 384;
        let mut aquifers_enabled = true;
        if let Some(noise) = settings.get("noise") {
            if let Some(m) = noise.get("min_y") { min_y = m.as_f64().unwrap_or(-64.0) as i32; }
            if let Some(h) = noise.get("height") { noise_height = h.as_f64().unwrap_or(384.0) as i32; }
        }
        if let Some(aq) = settings.get("aquifers_enabled") { aquifers_enabled = aq.as_bool().unwrap_or(true); }
        // legacy_random_source=true（下界）→ 噪声种子/surface 概率走 LegacyRandomSource（Java RandomState ctor 同构）
        let mut legacy_random = false;
        if let Some(l) = settings.get("legacy_random_source") { legacy_random = l.as_bool().unwrap_or(false); }
        // 世界高度：Java 传（维度定义）；兜底 = 噪声高度（对齐 C++ worldHeight>0 ? worldHeight : noiseHeight）
        let height = if world_height > 0 { world_height } else { noise_height };
        let router = settings.get("noise_router")?;

        // transpiler/ fallback 共用 NoiseSet 构建（260903-06 P-B 修复）：
        // ⚠️ 必须设 blended_noise（old_blended_noise = base_3d_noise）——漏设则 sample_blended_noise
        // 返回 0.0 → ch0 丢 base_3d 分量（列扫呈 depth*factor 纯线性，transpiler ch0 系统性偏差）。
        // 污染源定位：260903-05 bA「生成物 y=0 闭包压平」为误归因（supersedes，单点探针证生成函数正确）。
        // scale/factor/smear 从 base_3d_noise.json 读（数据驱动）；octave 结构（-15/-7 legacy）为 Java
        // 构造器固定参数（代码固定，同 C++ 侧）。
        fn build_transpiler_noises(db: &DensityBuilder, noise_params_path: &str, wg_dir: &str) -> Option<crate::noise::NoiseSet> {
            let mut noises = crate::noise::NoiseSet::new();
            let params = crate::density_builder::build_noise_params_from_file(noise_params_path).ok()?;
            for (id, p) in &params {
                let mut rnd = db.random_deriver().split_str(id);
                let sampler = crate::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
                noises.insert(id, sampler);
            }
            // base_3d_noise.json：xz_scale/y_scale/xz_factor/y_factor/smear_scale_multiplier
            let b3 = crate::json::parse(&std::fs::read_to_string(
                format!("{}/data/minecraft/worldgen/density_function/overworld/base_3d_noise.json", wg_dir)).ok()?).ok()?;
            let num = |k: &str, d: f64| -> f64 { b3.get(k).and_then(|x| x.as_f64()).unwrap_or(d) };
            let (xz_s, y_s, xz_f, y_f, smear) = (
                num("xz_scale", 0.25), num("y_scale", 0.125),
                num("xz_factor", 80.0), num("y_factor", 160.0), num("smear_scale_multiplier", 8.0));
            let mut rnd = db.random_deriver().split_str("minecraft:terrain");
            let amp_l = crate::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
            let lower = crate::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
            let upper = crate::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
            let amp_i = crate::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
            let interp = crate::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
            let bn = crate::density::InterpolatedNoiseData::new(lower, upper, interp, xz_s, y_s, xz_f, y_f, smear);
            noises.set_blended_noise(bn);
            Some(noises)
        }

        // 1. DensityBuilder（dfNs 参数化：external_loader 读 <dfNs>/ 目录 + resolve_ref 用 dfNs 前缀）
        let mut db = DensityBuilder::new(seed as u64, min_y, noise_height);
        db.set_df_ns(&df_ns);
        if legacy_random { db.set_legacy_random(); }
        let noise_params_path = format!("{}/../noise_params.json", wg_dir);
        if db.load_noise_params_file(&noise_params_path).is_err() {
            eprintln!("wg_create: cannot load {}", noise_params_path);
            return None;
        }
        let df_dir = format!("{}/data/{}/worldgen/density_function/{}", wg_dir, data_ns, df_ns);
        let df_dir2 = df_dir.clone();
        db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
            let p = format!("{}/{}.json", df_dir2, name);
            std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}: {}", p, e))
        }));

        // 3. router DF 树
        let tree = Arc::new(db.build_node(router.get("final_density")?).ok()?);
        // multi-channel 宏观采样器（对齐 Java NoiseChunk cell grid；fill_chunk 用 cell grid 采样 density）
        // cell 尺寸 = settings size_horizontal*4 / size_vertical*4（end 8×4 / overworld 4×8，260906-04）
        let size_h = settings.get("noise").and_then(|n| n.get("size_horizontal")).and_then(|s| s.as_f64()).unwrap_or(1.0) as i32;
        let size_v = settings.get("noise").and_then(|n| n.get("size_vertical")).and_then(|s| s.as_f64()).unwrap_or(2.0) as i32;
        let macro_sampler = crate::terrain::DensityMacroSampler::with_cells(&tree, min_y, noise_height,
            size_h * 4, size_v * 4);
        // transpiler 宏观采样器（WG_TRANSPILER env 时启用）：构建 NoiseSet + TranspilerDensity
        // transpiler 生成代码（generated_density）用 NoiseSet 采样（非 DensityFunction 树），需独立构建 NoiseSet。
        let transpiler_density = if std::env::var("WG_TRANSPILER").is_ok() {
            build_transpiler_noises(&db, &noise_params_path, &wg_dir)
                .map(|noises| crate::terrain::TranspilerDensity::new(noises, min_y, noise_height))
        } else { None };
        // DFC 宏观采样器（WG_DFC env 时启用；优先级 > WG_TRANSPILER；零退化铁律：默认关）
        let dfc_density = if std::env::var("WG_DFC").is_ok() {
            Some(crate::terrain::DfcDensity::new(seed as u64))
        } else { None };
        // GPU 密度源（WG_GPU_DENSITY env 时启用；优先级 > WG_DFC；零退化铁律：默认关）。
        // 构造即 create（~75s 一次付）；失败 graceful fallback = None（回退 dfc/transpiler/macro）。
        let gpu_density = if std::env::var("WG_GPU_DENSITY").is_ok() {
            crate::gpu_ffi::GpuDensity::new(seed, &wg_dir, min_y)
        } else { None };
        // GPU channels 角点源（WG_GPU_CHANNELS env 时启用；X2 260903-05；优先级最高；默认关）。
        // fallback = TranspilerDensity（独立 NoiseSet，同语义 diff0 路径）。
        let gpu_channels = if std::env::var("WG_GPU_CHANNELS").is_ok() {
            build_transpiler_noises(&db, &noise_params_path, &wg_dir)
                .and_then(|fb_noises| {
                    let fallback = crate::terrain::TranspilerDensity::new(fb_noises, min_y, noise_height);
                    crate::gpu_ffi::GpuChannelDensity::new(seed, &wg_dir, min_y, noise_height, fallback)
                })
        } else { None };
        let barrier = Arc::new(db.build_node(router.get("barrier")?).ok()?);
        let flooded = Arc::new(db.build_node(router.get("fluid_level_floodedness")?).ok()?);
        let spread = Arc::new(db.build_node(router.get("fluid_level_spread")?).ok()?);
        let lava = Arc::new(db.build_node(router.get("lava")?).ok()?);
        let erosion = Arc::new(db.build_node(router.get("erosion")?).ok()?);
        let depth = Arc::new(db.build_node(router.get("depth")?).ok()?);
        let init = Arc::new(db.build_node(router.get("initial_density_without_jaggedness")?).ok()?);
        let tempf = Arc::new(db.build_node(router.get("temperature")?).ok()?);
        let humf = Arc::new(db.build_node(router.get("vegetation")?).ok()?);
        let contf = Arc::new(db.build_node(router.get("continents")?).ok()?);
        let erof = Arc::new(db.build_node(router.get("erosion")?).ok()?);
        let depthf = Arc::new(db.build_node(router.get("depth")?).ok()?);
        let weirdf = Arc::new(db.build_node(router.get("ridges")?).ok()?);
        // 矿脉（ore vein）组件
        let vein_toggle = Arc::new(db.build_node(router.get("vein_toggle")?).ok()?);
        let vein_ridged = Arc::new(db.build_node(router.get("vein_ridged")?).ok()?);
        let vein_gap = Arc::new(db.build_node(router.get("vein_gap")?).ok()?);
        let ore_splitter = match db.random_deriver() {
            RsSplitter::Xoro(s) => s.split_str("minecraft:ore").next_splitter(),
            // 下界（legacy）ore_vein 禁用——占位值不影响输出
            RsSplitter::Legacy(_) => crate::xoroshiro::XoroshiroRandom::new(seed as u64).next_splitter().split_str("minecraft:ore").next_splitter(),
        };
        // ore_vein 延后到 blocks 加载后构建（block id 需 BlockRegistry 解析，数据驱动）

        // 4. noise samplers（surface rules 用）
        // C2（2026-09-07，judge CONCERN：数据驱动化）：预加载 key 分两路——
        //   ① 基础 3 key（SurfaceBuilder 引擎无条件用：sample_run_depth/get_terracotta_block/secondary）
        //   ② overworld 代码规则保留静态清单（规则在代码里，无 JSON 数据源可收集；含 .b2 前的历史 key）；
        //      非 overworld 从 settings.surface_rule JSON 构建期动态收集 noise_threshold 引用的 key
        //      （collect_noise_keys，一次性调用非热路径）。静态 nether 清单已删除（由 JSON 收集取代）。
        {
            let base: &[&str] = &["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset"];
            for k in base {
                let _ = db.get_noise_sampler(k);
            }
            if df_ns == "overworld" {
                // #26 判据 1（260903-15）：清单提为单一事实源常量——预加载来源唯一，
                // 运行时引用侧由 collect_rule_noise_keys 在下方启动期断言自动核对，
                // 新增查表调用点只需改本清单，漏项在启动即 panic（不再低频分支运行时崩）。
                for k in OVERWORLD_NOISE_KEYS {
                    let _ = db.get_noise_sampler(k);
                }
            } else if let Some(sr) = settings.get("surface_rule") {
                let mut dyn_keys: Vec<String> = Vec::new();
                crate::surface_rules::collect_noise_keys(sr, &mut dyn_keys);
                for k in &dyn_keys {
                    let _ = db.get_noise_sampler(k);
                }
            }
        }

        // 5. blocks registry
        let blocks_path = format!("{}/../blocks.json", wg_dir);
        let blocks_json = std::fs::read_to_string(&blocks_path).ok()?;
        let blocks = BlockRegistry::load_from_json(&blocks_json)?;
        // 5b. block tag 注册表（260907-04：carver replaceable / feature RuleTest tag 数据驱动；
        //     必须在 load_carvers/load_features 解析前 init，缺失时消费点走硬编码 fallback）
        crate::block_tags::init(&wg_dir);

        // 6. biome classifier + carvers + features（维度参数化：biome_params_file 决定 biome 参数）
        // end（260906-04）：无 biome_params 文件（非 MultiNoise），bc 只承载 carvers/features；
        // 判定走 MacroBiome.end_mode（TheEndBiomeSource 位置判定）。
        let end_mode = df_ns == "end";
        let biome_params_path = format!("{}/../{}", wg_dir, biome_params_file);
        let mut bc = if end_mode { BiomeClassifier::empty() } else { BiomeClassifier::load(&biome_params_path) };
        // 260907-05（A 组缺口 4）：biome 目录按命名空间合成——vanilla 全量 + mod 命名空间目录（id 带 modid: 前缀）；
        // 同名文件跨命名空间冲突时 minecraft 先入（load_* 为首见覆盖语义前先查——这里两次 load 幂等合并）。
        let biome_dir = format!("{}/data/minecraft/worldgen/biome", wg_dir);
        let _n = bc.load_carvers(&biome_dir, "minecraft");
        let _nf = bc.load_features(&biome_dir, "minecraft");
        if data_ns != "minecraft" {
            let mod_biome_dir = format!("{}/data/{}/worldgen/biome", wg_dir, data_ns);
            let _n2 = bc.load_carvers(&mod_biome_dir, &data_ns);
            let _nf2 = bc.load_features(&mod_biome_dir, &data_ns);
        }
        // 260905-08 E-C2：Java registry 枚举序（feature 全局 index 构建序，数据驱动）
        let _no = bc.load_registry_order(&format!("{}/biome_registry_order.json", wg_dir));

        // 7. surface builder（Box::leak 让引用长期存活）
        let samplers = Box::leak(Box::new(db.noise_samplers().clone()));
        let splitter = Box::leak(Box::new(db.random_deriver().clone()));
        let blocks_leaked = Box::leak(Box::new(blocks));
        // ore_vein（block id 从 BlockRegistry 解析，数据驱动——跨版本换 blocks.json 即可）
        let ore_vein = crate::ore_vein::OreVeinSampler::new(vein_toggle, vein_ridged, vein_gap, ore_splitter, blocks_leaked);
        // sea_level 从 settings 读（主世界 63 / 下界 32 / mod 维度按定义）
        let sea_level = settings.get("sea_level").and_then(|s| s.as_f64()).unwrap_or(63.0) as i32;
        // 260907-05（缺口 2）：default_block 注入 SurfaceBuilder（未知名已由 blocks.id 一次性告警）
        let default_block = blocks_leaked.id(&default_block_name);
        let mut sb = SurfaceBuilder::new(samplers, splitter, sea_level, blocks_leaked);
        sb.set_default_block(default_block);
        // 260907-09：注入名字供消费点惰性解析——mod 方块在本 handle 创建后才经 wg_register_block_id
        // 注册，创建期 id() 解析会 miss 成 AIR（realmod e2e 实测：整片地形被填成空气）。
        sb.set_default_block_name(default_block_name.clone());
        // surface_rule：overworld 用已验证的代码规则；其他维度用 settings.surface_rule JSON 数据驱动（对齐 C++）
        let df_ns2 = df_ns.clone();
        let rule = if df_ns2 == "overworld" {
            sb.build_overworld_rule()
        } else {
            // fail-fast（judge 建议，260902-04）：非 overworld 维度 surface_rule 缺失/解析失败
            // 曾静默回退 overworld 代码规则——错误数据被掩盖成「对齐率莫名下降」，
            // 改为直接 panic 暴露配置/解析问题（对齐 C++ worldgen_api.cpp 的报错路径语义）。
            match settings.get("surface_rule") {
                Some(sr) => match sb.parse_surface_rule(&sr, min_y, noise_height) {
                    Some(r) => r,
                    None => panic!(
                        "[surface] settings '{}' surface_rule 解析失败（不支持的节点/缺字段），fail-fast 拒绝回退 overworld 规则",
                        df_ns2
                    ),
                },
                None => panic!("[surface] settings '{}' 缺少 surface_rule 字段，fail-fast", df_ns2),
            }
        };

        // #26 判据 1（260903-15）启动期机械校验：运行时引用 noise key ⊆ 预加载集合。
        // 规则树运行时查 sampler 的 key（collect_rule_noise_keys 自动收集，含未来新增节点）
        // 若未预加载，宁可在 create 时 panic 也不留到低频分支运行时崩（panic 505 教训）。
        {
            let mut runtime_keys: Vec<String> = Vec::new();
            crate::surface_rules::collect_rule_noise_keys(&rule, &mut runtime_keys);
            let preloaded = db.noise_samplers();
            let missing: Vec<&str> = runtime_keys
                .iter()
                .map(|k| k.as_str())
                .filter(|k| !preloaded.contains_key(*k))
                .collect();
            if !missing.is_empty() {
                panic!(
                    "[surface] noise preload 校验失败：规则引用但未预加载的 key {:?}（settings '{}'，check preload table / OVERWORLD_NOISE_KEYS in worldgen_handle.rs）",
                    missing, df_ns2
                );
            }
        }

        let biomesrc = MacroBiome { bc, tempf, humf, contf, erof, depthf, weirdf, end_mode };
        // aquifer splitter：Java NoiseConfig.java:54 aquiferRandomDeriver = randomDeriver.split(Identifier("aquifer")).nextSplitter()
        //（260904 修复：原直传顶层 random_deriver() 漏 aquifer 字符串 split → blob 随机偏移全错，残留 1830 根因；
        //  对照 ore 管线 L261 的 split_str("minecraft:ore") 同构）
        let splitter = match db.random_deriver() {
            RsSplitter::Xoro(s) => s.split_str("minecraft:aquifer").next_splitter(),
            // 下界（legacy）aquifer 禁用——aquifer 字段需 XoroshiroSplitter 类型，占位值不影响输出
            RsSplitter::Legacy(_) => crate::xoroshiro::XoroshiroRandom::new(seed as u64).next_splitter(),
        };

        // FEATURES indexer（Java PlacedFeatureIndexer，从所有 biome features 构建）
        // p 值 = lastIndex（feature 在所有 biome features 中的最后出现索引），不是 featureIndex！
        let all_biome_features = biomesrc.bc.all_features_lists();
        let mut feature_indexer = crate::feature_loader::PlacedFeatureIndexer::new();
        feature_indexer.build(&all_biome_features);

        // 预加载 carver（数量少，创建时一次性加载，运行时只读无锁）
        let mut carver_cache = HashMap::new();
        for cid in biomesrc.bc.all_carver_ids() {
            if let Some(cc) = Self::load_carver(&wg_dir, blocks_leaked, &cid) {
                carver_cache.insert(cid.clone(), cc);
            }
        }

        // 预加载 feature（从所有 biome features 的唯一 placed_feature，创建时一次性加载，运行时只读无锁）
        let feature_ids = biomesrc.bc.all_feature_ids();
        let mut feature_cache = crate::feature_loader::FeatureCache::new();
        feature_cache.preload_all(&wg_dir, &feature_ids, blocks_leaked);

        // SteelMC uniform_carver_biome 优化：若所有 biome carvers 统一，apply_carvers 跳过 289 次 biome 采样
        let uniform_carver_list = biomesrc.bc.uniform_carver_list();

        Some(WorldgenHandle {
            seed, biome_access_seed: crate::biome::biome_hash_seed(seed),
            min_y, height, noise_height, aquifers_enabled, sea_level,
            tree, macro_sampler, transpiler_density, dfc_density, gpu_density, gpu_channels, barrier, flooded, spread, lava, erosion, depth, init,
            biomesrc, sb, rule,
            blocks: blocks_leaked,
            default_block,
            splitter,
            ore_vein,
            beardifiers: std::sync::RwLock::new(HashMap::new()),
            carver_cache,
            uniform_carver_list,
            feature_cache,
            feature_indexer,
            wg_dir,
            flags: std::sync::atomic::AtomicU32::new(0),
            est_l2: std::sync::OnceLock::new(),
            terrain_cache: std::sync::Mutex::new(HashMap::new()),
            pending_cross_writes: std::sync::Mutex::new(HashMap::new()),
        })
    }

    // 设置指定 chunk 的 Beardifier（StructureWeightSampler）输入。
    // pieces 每 8 int：{minX,minY,minZ,maxX,maxY,maxZ,terrain(0-4),groundLevelDelta}
    // （terrain 序数 = Java StructureTerrainAdaptation ordinal；1.21.6 起 4=ENCAPSULATE）
    // junctions 每 3 int：{sourceX,sourceGroundY,sourceZ}
    pub fn set_beardifier(&self, chunk_x: i32, chunk_z: i32,
                          pieces: &[i32], junctions: &[i32]) {
        let mut b = Beardifier::new();
        for i in 0..pieces.len() / 8 {
            let p = &pieces[i * 8..i * 8 + 8];
            b.pieces.push(crate::beardifier::BeardPiece {
                min_x: p[0], min_y: p[1], min_z: p[2],
                max_x: p[3], max_y: p[4], max_z: p[5],
                terrain: match p[6] { 1 => crate::beardifier::TerrainAdaptation::Bury,
                                       2 => crate::beardifier::TerrainAdaptation::BeardThin,
                                       3 => crate::beardifier::TerrainAdaptation::BeardBox,
                                       4 => crate::beardifier::TerrainAdaptation::Encapsulate, // 1.21.6 ENCAPSULATE（trial_chambers 唯一使用者）
                                       _ => crate::beardifier::TerrainAdaptation::None },
                ground_level_delta: p[7],
            });
        }
        for i in 0..junctions.len() / 3 {
            let j = &junctions[i * 3..i * 3 + 3];
            b.junctions.push(crate::beardifier::BeardJunction {
                source_x: j[0], source_ground_y: j[1], source_z: j[2],
            });
        }
        self.beardifiers.write().unwrap().insert((chunk_x, chunk_z), b);
    }

    pub fn clear_beardifier(&self) {
        self.beardifiers.write().unwrap().clear();
    }

    // finalDensity 网格采样（wg_fill_density / fillDensity 用）：
    // size×size chunks，每 chunk POINTS_PER_CHUNK 点（XZ_INTERVAL/Y_INTERVAL 网格），chunk-major。
    /// 精确 density 采样（tree.sample 纯函数，无网格插值）——对齐 Java DensityProbe 的 df.sample 语义
    pub fn sample_density_exact(&self, x: i32, y: i32, z: i32) -> f64 {        self.tree.sample(&crate::density::NoisePos { x, y, z })
    }
    /// biome 采样（对外诊断/对拍用；输入块坐标，内部按 4 块对齐——与 fill 的 biome_at 同源语义）
    pub fn biome_sample(&self, x: i32, y: i32, z: i32) -> String {
        let bp = NoisePos { x: (x >> 2) << 2, y: (y >> 2) << 2, z: (z >> 2) << 2 };
        self.biomesrc.biome(&bp)
    }
    /// block id → name（对外诊断/对拍用；blocks 字段私有）
    pub fn block_name(&self, id: i32) -> String { self.blocks.name(id) }

    // 260907-05（缺口 1）：运行时注册方块（wg_register_block 后端）
    pub fn register_block(&self, name: &str) -> i32 { self.blocks.register(name) }
    // 260907-08（候选 B）：显式 id 注册（wg_register_block_id 后端，java_raw 同域对齐）
    pub fn register_block_with_id(&self, name: &str, id: i32) -> i32 { self.blocks.register_with_id(name, id) }
    pub fn fill_density(&self, min_chunk_x: i32, min_chunk_z: i32, size: i32) -> Vec<f64> {
        let xz = crate::api::density_xz_interval();
        let yi = crate::api::density_y_interval();
        let sy = (self.height / yi); // HEIGHT/y_interval
        let sx = (16 / xz) as usize;
        let sz = sx;
        let mut out = Vec::new();
        for cz in 0..size {
            for cx in 0..size {
                let chunk_x = min_chunk_x + cx;
                let chunk_z = min_chunk_z + cz;
                for y in 0..sy {
                    for z in 0..sz as i32 {
                        for x in 0..sx as i32 {
                            let pos = NoisePos {
                                x: chunk_x * 16 + x * xz,
                                z: chunk_z * 16 + z * xz,
                                y: self.min_y + y * yi,
                            };
                            out.push(self.tree.sample(&pos));
                        }
                    }
                }
            }
        }
        out
    }

    // 诊断/探针只读访问（X2 gpu_channel_probe 用；不改变生产行为）
    pub fn gpu_channels_density(&self) -> Option<&crate::gpu_ffi::GpuChannelDensity> { self.gpu_channels.as_ref() }
    // 诊断：transpiler 密度源只读访问（ch0 y 依赖性复核用）
    pub fn transpiler_density(&self) -> Option<&crate::terrain::TranspilerDensity> { self.transpiler_density.as_ref() }
    // 诊断：macro 采样器只读访问（生产默认路径 ch0 对照用）
    pub fn macro_sampler(&self) -> &crate::terrain::DensityMacroSampler { &self.macro_sampler }

    // b1-b（260903-11）：L2 惰性句柄——首次调用时创建（容量 DEFAULT_CAP），Arc 跨 chunk 共享。
    // 调用方必须已判 WG_EST_L2（fill 路径 chunk 级判过）；进程内 env 判定时机统一在首次 fill。
    fn est_l2_handle(&self) -> std::sync::Arc<std::sync::Mutex<crate::aquifer::EstL2>> {
        let cell = self.est_l2.get_or_init(|| {
            Some(std::sync::Arc::new(std::sync::Mutex::new(crate::aquifer::EstL2::new(crate::aquifer::EstL2::DEFAULT_CAP))))
        });
        cell.clone().expect("est_l2 lazily initialized to Some")
    }
    // 诊断：[hits, misses, inserts, evictions]（未启用/未初始化 → [0,0,0,0]）
    pub fn est_l2_stats(&self) -> [usize; 4] {
        match self.est_l2.get() {
            Some(Some(l2)) => l2.lock().map(|m| m.stats()).unwrap_or([0; 4]),
            _ => [0; 4],
        }
    }

    // 完整区块生成（方块层）：fill_chunk（宏观）→ BlockColumn → build_surface → carver → features。
    // 返回 16*16*height 的 vanilla raw block id（索引 (y-min_y)*256 + z*16 + x）。
    pub fn fill_chunk_blocks(&self, cx: i32, cz: i32) -> Vec<BlockId> {
        let min_y = self.min_y;
        let height = self.height;
        // 步骤 1-4（宏观+orevein+surface+carver）——c-A 起与邻 chunk 地形列生成共用同一实现
        let (mut col, heightmap) = self.fill_terrain_column(cx, cz);
        let flags = self.flags.load(std::sync::atomic::Ordering::Relaxed);
        // surface 规则外的 feature 侧 biome 输入（与 fill_terrain_column 内 carver 用同源语义）
        let biome_at = |x: i32, y: i32, z: i32| -> String {
            let bp = NoisePos { x: (x >> 2) << 2, y: (y >> 2) << 2, z: (z >> 2) << 2 };
            self.biomesrc.biome(&bp)
        };

        // 5. features（装饰层：矿石/disk/spring/freeze_top/underwater_magma）
        let skip_features = flags & FLAG_SKIP_FEATURES != 0 || std::env::var("WG_SKIP_FEATURES").is_ok();
        if !skip_features {
            let n_features = self.apply_features(&mut col, cx, cz, &heightmap, &biome_at);
            if std::env::var("WG_FEATURELOG").is_ok() {
                eprintln!("[FEATURE] chunk({},{}) placed {} blocks", cx, cz, n_features);
            }
        }

        col.data().to_vec()
    }

    // 步骤 1-4 地形列（宏观 density+aquifer+orevein → BlockColumn → build_surface → carver，无 features）
    // + WORLD_SURFACE_WG heightmap。c-A-min（260905-09）：邻 chunk 越界读复用本实现
    //（Java 时序保证：FEATURES 时邻 chunk ≥ post-carver 地形态；feature 时序近似无 IDK-cA1）。
    fn fill_terrain_column(&self, cx: i32, cz: i32) -> (crate::blocks::BlockColumn, Vec<i32>) {
        let min_y = self.min_y;
        let height = self.height;
        let air = self.blocks.id("minecraft:air");
        let stone = self.sb.default_block_id(); // 260907-09：惰性按名解析（mod 方块创建后注册）；260907-05：settings.default_block（原硬编码 stone）
        let water = self.blocks.id("minecraft:water");
        let lava_id = self.blocks.id("minecraft:lava");

        // 1. fill_chunk（宏观：density + aquifer 分类）
        // multi-channel 宏观采样器（cell grid 采样 density，对齐 Java NoiseChunk；thread_local slices 缓存每 chunk 重建一次）
        // WG_TRANSPILER 时用 transpiler 生成代码采样（build-time 编译 density 树），否则用 DensityMacroSampler。
        // WG_AQDUMP：seed 头（OnceLock 只打一次；门控关时一次原子 load）
        crate::aquifer::aqdump_seed(self.seed);
        let mut aq = crate::aquifer::Aquifer::new(
            self.barrier.clone(), self.flooded.clone(), self.spread.clone(), self.lava.clone(),
            self.erosion.clone(), self.depth.clone(), self.init.clone(), self.splitter.clone(),
            cx * 16, cz * 16, min_y, height);
        // aquifers_enabled=false（下界）→ VanillaAquifer.enabled=false，classify 跳过真实 aquifer（无 water/lava）
        // WG_SKIP_AQUIFER（诊断）chunk 级判断一次（避免每点 env 查询污染热路径）
        let skip_aquifer = std::env::var("WG_SKIP_AQUIFER").is_ok();
        let mut va = VanillaAquifer { aq, enabled: self.aquifers_enabled, skip_aquifer, sea_level: self.sea_level };
        // b1-b（260903-13 翻默认，用户 confirmed）：est L2 默认启用，WG_EST_L2=0 反转关闭
        //（chunk 级 env 判一次，热路径零 env 查询；修复后 L2 精确值缓存零语义差，四臂 hash 同一）
        if crate::worldgen_handle::env_enabled("WG_EST_L2") {
            va.aq.set_est_l2(Some(self.est_l2_handle()));
        }
        // Beardifier（结构密度修正）：读当前 chunk 的 beardifier（RwLock 读，clone 避免持锁跨 fill_chunk）
        let beard = self.beardifiers.read().unwrap().get(&(cx, cz)).cloned();
        let cd = if let Some(gc) = &self.gpu_channels {
            fill_chunk(gc, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(gd) = &self.gpu_density {
            fill_chunk(gd, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(dd) = &self.dfc_density {
            fill_chunk(dd, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(td) = &self.transpiler_density {
            fill_chunk(td, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else {
            let dense: &crate::terrain::DensityMacroSampler = &self.macro_sampler;
            fill_chunk(dense, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        };

        // 2. 宏观 → BlockColumn（具体 block id 占位：air/stone/water/lava）
        //    ore_vein：rock 处按矿脉分布替换为铜/铁矿脉块（aquifer 无 fluid 的 rock）
        let mut col = BlockColumn::new(min_y, height);
        // WG_SKIP_OREVEIN（诊断）chunk 级判断一次（避免每点 env 查询污染热路径）
        let skip_orevein = std::env::var("WG_SKIP_OREVEIN").is_ok();
        for lz in 0..16 { for lx in 0..16 { for ly in 0..height {
            let y = min_y + ly;
            let kind = cd.blocks[(lx + lz * 16 + ly * 256) as usize];
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let mut id = match kind {
                crate::terrain::BlockKind::Air => air,
                crate::terrain::BlockKind::Rock => stone,
                crate::terrain::BlockKind::Water => water,
                crate::terrain::BlockKind::Lava => lava_id,
            };
            // 矿脉（只替换 rock 处的深部块，且 ore_vein.apply 返回非 -1）
            if kind == crate::terrain::BlockKind::Rock && !skip_orevein {
                let vein = self.ore_vein.apply(wx, y, wz);
                if vein >= 0 { id = vein; }
            }
            *col.at_mut(lx, y, lz) = id;
        }}}

        // 3. build_surface（具体 block id：grass/sand/terracotta 等）
        // 空列哨兵映射（260906-04 end 接管）：surface_height 无 solid 列 = i32::MIN（terrain.rs:241），
        // 直接进 build_surface 会让 o = MIN+1 在 biome_pick_cell(by-2) 溢出 panic（debug）/回绕（release）。
        // Java 语义：WORLD_SURFACE_WG 空列 get = bottom-1（Heightmap 初始化 bottom-1），+1 = bottom。
        // end 外岛空域大量全空气列（overworld 有基岩永不触发），故在此消费点统一映射 min_y-1。
        let heightmap: Vec<i32> = cd.surface_height.to_vec()
            .into_iter().map(|h| if h == i32::MIN { min_y - 1 } else { h }).collect();
        // b1-a（260903-13 翻默认，用户 confirmed）：est_at 默认复用 va.aq 的 surface_cache（对齐 Java：
        // SURFACE 阶段走 sampler.estimateSurfaceHeight 同一张 map，ChunkNoiseSampler.java:222-226）；
        // WG_EST_SHARED=0 反转关闭（回归旧独立扫描路径，诊断用）。
        // 260903-13 修复后两路径语义完全一致（四臂 hash 同一 f2b1a3932c6e589e）：
        //  D1 角列：两臂角参数均为 Java SURFACE 四角 (i+1)<<4（+16）；shared 路径内部 (x>>2)<<2 量化（+16 不变）
        //  D3 扫描域：两臂扫描均为 min_y+noise_height 起、含 min_y 下界（overworld 320..-64）
        let est_shared = crate::worldgen_handle::env_enabled("WG_EST_SHARED");
        let mut est_at = |x: i32, z: i32| -> i32 {
            if est_shared {
                va.aq.estimate_surface_height(x, z)
            } else {
                // 260903-13 修 −1 偏移（judge A1，260903-12）：对齐 Java NoiseChunk.computePreliminarySurfaceLevel
                //（forge sources NoiseChunk.java:174）：for(l = min_y+height; l >= min_y; l -= cellHeight)
                // → 首采样点 320、含下界 -64。旧写法半开区间 rev 首点 319（319,311,… vs Java 320,312,…）。
                let mut est = i32::MAX;
                let mut y = min_y + self.noise_height;
                while y >= min_y {
                    if self.init.sample(&NoisePos { x, y, z }) > 0.390625 { est = y; break; }
                    y -= 8;
                }
                est
            }
        };
        // 260903-13 角参数 +16（Java MaterialRules.java:496-499 chunkToBlockCoord(i+1) = (i+1)<<4），
        // 取代旧 +15（#25 第三例：+15 量化 +12 ≠ Java +16）
        let surface_heights4 = vec![
            est_at(cx * 16, cz * 16), est_at(cx * 16 + 16, cz * 16),
            est_at(cx * 16, cz * 16 + 16), est_at(cx * 16 + 16, cz * 16 + 16),
        ];
        // 260903-12（默认关）：WG_EST_DUMP=<path> → 4 角 est 值逐条追加 dump（Java est 对比用 P1.2）
        if let Ok(dump_path) = std::env::var("WG_EST_DUMP") {
            let corner_params = [(cx * 16, cz * 16), (cx * 16 + 16, cz * 16), (cx * 16, cz * 16 + 16), (cx * 16 + 16, cz * 16 + 16)];
            let mut line = format!("{},{}", cx, cz);
            for (i, &(px, pz)) in corner_params.iter().enumerate() {
                line.push_str(&format!(",c{}:{}:{}:{}", i, px, pz, surface_heights4[i]));
            }
            line.push('\n');
            use std::sync::atomic::{AtomicBool, Ordering as Ao};
            static TRUNC: AtomicBool = AtomicBool::new(false);
            let trunc = !TRUNC.swap(true, Ao::SeqCst);
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).write(true).truncate(trunc).append(!trunc).open(&dump_path) {
                use std::io::Write;
                let _ = f.write_all(line.as_bytes());
            }
        }
        let biome_at = |x: i32, y: i32, z: i32| -> String {
            let bp = NoisePos { x: (x >> 2) << 2, y: (y >> 2) << 2, z: (z >> 2) << 2 };
            self.biomesrc.biome(&bp)
        };
        // surface 规则 biome 输入（260904-10 残留 76 主族根因修复）：
        // Java 生产 surface 收到的是 region.getBiomeAccess()（ChunkRegion L102，seed 经
        // BiomeAccess.hashSeed=sha256-asLong）→ MaterialRuleContext.initVerticalContext L464 用
        // posToBiome.apply(精确块坐标) → BiomeAccess.getBiome 8 邻域 jitter 选点（非单元直读）。
        // C++ 侧 biomeCellKey（worldgen_api.cpp L1080）同构；此前 Rust surface 用单元直读漏 zoom。
        // 注意：carver/feature 侧 biome_at_jitter 原用裸 self.seed（L722/849），与 Java region 级
        // BiomeAccess（hashSeed）口径不符——260904-12 已统一改 biome_access_seed（裸 seed 疑点收口，
        // decisive 重导验证中）。
        let biome_at_surface = |x: i32, y: i32, z: i32| -> String {
            let (px, py, pz) = crate::biome::biome_pick_cell(self.biome_access_seed, x, y, z);
            let bp = NoisePos { x: px << 2, y: py << 2, z: pz << 2 };
            self.biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { crate::surface_rules::biome_temperature(id) };
        let initial_density_at = |x: i32, y: i32, z: i32| -> f64 { self.init.sample(&NoisePos { x, y, z }) };
        let flags = self.flags.load(std::sync::atomic::Ordering::Relaxed);
        if flags & FLAG_SKIP_SURFACE == 0 && std::env::var("WG_SKIP_SURFACE").is_err() {
            self.sb.build_surface(&mut col, &self.rule, cx * 16, cz * 16, &heightmap, &surface_heights4,
                                  &biome_at_surface,
                                  &|x, y, z| {
                                      let (px, py, pz) = crate::biome::biome_pick_cell(self.biome_access_seed, x, y, z);
                                      // 对齐 C++ biomeCellKey packing（u32 截断防负坐标符号扩展）
                                      ((px as u32 as i64) << 40) | ((py as u32 as i64) << 20) | (pz as u32 as i64)
                                  },
                                  &biome_temp, min_y, height, &initial_density_at);
        }

        // 4. carver（洞穴雕刻，17×17 邻域）——句柄 flag 或 env 任一命中即 skip
        if flags & FLAG_SKIP_CARVER == 0 && std::env::var("WG_SKIP_CARVER").is_err() {
            self.apply_carvers(&mut col, cx, cz, &mut va.aq, &biome_at);
        }

        (col, heightmap)
    }

    // bin-diag 增量 API（2026-09-08，soul_selector_probe 用；不改任何现有行为）：
    // 返回 surface 阶段**之前**的区块方块（fill_chunk 宏观 + aquifer + ore_vein，步骤 1-2，
    // 与 fill_chunk_blocks 逐行同源）+ WORLD_SURFACE_WG heightmap（build_surface 输入）。
    // 动机：build_surface 的 stone_depth_above/below 由 surface 前列状态扫描得出，
    // fill_chunk_blocks 是 surface 后结果，无法用于复算 stone_depth 判定输入。
    pub fn diag_pre_surface_column(&self, cx: i32, cz: i32) -> (Vec<BlockId>, Vec<i32>) {
        let min_y = self.min_y;
        let height = self.height;
        let air = self.blocks.id("minecraft:air");
        let stone = self.sb.default_block_id(); // 260907-09：惰性按名解析；260907-05：settings.default_block（原硬编码 stone）
        let water = self.blocks.id("minecraft:water");
        let lava_id = self.blocks.id("minecraft:lava");
        let mut aq = crate::aquifer::Aquifer::new(
            self.barrier.clone(), self.flooded.clone(), self.spread.clone(), self.lava.clone(),
            self.erosion.clone(), self.depth.clone(), self.init.clone(), self.splitter.clone(),
            cx * 16, cz * 16, min_y, height);
        let skip_aquifer = std::env::var("WG_SKIP_AQUIFER").is_ok();
        let mut va = VanillaAquifer { aq, enabled: self.aquifers_enabled, skip_aquifer, sea_level: self.sea_level };
        let beard = self.beardifiers.read().unwrap().get(&(cx, cz)).cloned();
        let cd = if let Some(gc) = &self.gpu_channels {
            fill_chunk(gc, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(gd) = &self.gpu_density {
            fill_chunk(gd, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(dd) = &self.dfc_density {
            fill_chunk(dd, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else if let Some(td) = &self.transpiler_density {
            fill_chunk(td, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        } else {
            let dense: &crate::terrain::DensityMacroSampler = &self.macro_sampler;
            fill_chunk(dense, &mut va, &self.biomesrc, cx, cz, min_y, height, beard.as_ref(), self.noise_height)
        };
        let mut col = BlockColumn::new(min_y, height);
        let skip_orevein = std::env::var("WG_SKIP_OREVEIN").is_ok();
        for lz in 0..16 { for lx in 0..16 { for ly in 0..height {
            let y = min_y + ly;
            let kind = cd.blocks[(lx + lz * 16 + ly * 256) as usize];
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let mut id = match kind {
                crate::terrain::BlockKind::Air => air,
                crate::terrain::BlockKind::Rock => stone,
                crate::terrain::BlockKind::Water => water,
                crate::terrain::BlockKind::Lava => lava_id,
            };
            if kind == crate::terrain::BlockKind::Rock && !skip_orevein {
                let vein = self.ore_vein.apply(wx, y, wz);
                if vein >= 0 { id = vein; }
            }
            *col.at_mut(lx, y, lz) = id;
        }}}
        (col.data().to_vec(), cd.surface_height.to_vec())
    }

    // CARVERS 阶段：17×17 邻域，per-biome carvers.air，setCarverSeed，shouldCarve，carve
    fn apply_carvers(&self, col: &mut BlockColumn, cx: i32, cz: i32,
                     aquifer: &mut crate::aquifer::Aquifer,
                     biome_at: &dyn Fn(i32, i32, i32) -> String) {
        let min_y = self.min_y;
        let height = self.height;
        // biomeAtNoJitter：chunk 角采样（无 jitter）
        let biome_at_no_jitter = |cx2: i32, cz2: i32| -> String {
            let bp = NoisePos { x: cx2 * 16, y: 0, z: cz2 * 16 };
            self.biomesrc.biome(&bp)
        };
        // biomeAtJitter：8 邻域 jitter（applyMaterialRule 用）
        let biome_at_jitter = |x: i32, y: i32, z: i32| -> String {
            let (px, py, pz) = crate::biome::biome_pick_cell(self.biome_access_seed, x, y, z);
            let bp = NoisePos { x: px << 2, y: py << 2, z: pz << 2 };
            self.biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { crate::surface_rules::biome_temperature(id) };
        let initial_density_at = |x: i32, y: i32, z: i32| -> f64 { self.init.sample(&NoisePos { x, y, z }) };
        let apply_material_rule = |x: i32, y: i32, z: i32, has_fluid: bool| -> Option<i32> {
            self.sb.apply_material_rule_single(&self.rule, &biome_at_jitter, &biome_temp, x, y, z, has_fluid, min_y, height, &initial_density_at)
        };

        let mut ctx = CarverContext {
            min_y, height,
            aquifer,
            blocks: &self.blocks,
            apply_material_rule: Some(&apply_material_rule),
        };
        let mut mask = CarvingMask::new(height, min_y);
        let mut chunk_random = ChunkRandom::checked();
        // SteelMC uniform_carver_biome 优化：若所有 biome carvers 统一，跳过 289 次 biome 采样，
        // 直接用统一 carver 列表（overworld 统一为 [canyon,cave,cave_extra]）。
        if let Some(uniform) = &self.uniform_carver_list {
            for j in -8..=8 {
                for k in -8..=8 {
                    let cx2 = cx + j; let cz2 = cz + k;
                    let mut l = 0;
                    for carver_id in uniform {
                        let cc = self.get_carver(carver_id);
                        if cc.is_none() { l += 1; continue; }
                        let cc = cc.unwrap();
                        chunk_random.set_carver_seed(self.seed + l, cx2, cz2);
                        if cc.should_carve(&mut chunk_random) {
                            cc.carve(&mut ctx, col, &biome_at_jitter, &mut chunk_random, cx2, cz2, cx, cz, &mut mask);
                        }
                        l += 1;
                    }
                }
            }
            return;
        }

        let carver_diag = std::env::var("WG_CARVERDIAG").is_ok();
        let diag_t0 = std::time::Instant::now();
        let mut t_biome = 0.0f64;
        let mut t_carve = 0.0f64;
        let mut n_neighbor = 0;
        let mut n_carve = 0;
        for j in -8..=8 {
            for k in -8..=8 {
                let cx2 = cx + j; let cz2 = cz + k;
                let ab = std::time::Instant::now();
                let biome_id = biome_at_no_jitter(cx2, cz2);
                let carvers = self.biomesrc.bc.carvers_for(&biome_id).to_vec();
                t_biome += ab.elapsed().as_secs_f64();
                let mut l = 0;
                n_neighbor += 1;
                for carver_id in &carvers {
                    let cc = self.get_carver(carver_id);
                    if cc.is_none() { l += 1; continue; }
                    let cc = cc.unwrap();
                    chunk_random.set_carver_seed(self.seed + l, cx2, cz2);
                    if cc.should_carve(&mut chunk_random) {
                        let ac = std::time::Instant::now();
                        cc.carve(&mut ctx, col, &biome_at_jitter, &mut chunk_random, cx2, cz2, cx, cz, &mut mask);
                        t_carve += ac.elapsed().as_secs_f64();
                        n_carve += 1;
                    }
                    l += 1;
                }
            }
        }
        if carver_diag {
            eprintln!("[CARVERDIAG] chunk({},{}) neighbors={} carve_calls={} t_biome={:.1}ms t_carve={:.1}ms total={:.1}ms other={:.1}ms",
                cx, cz, n_neighbor, n_carve, t_biome * 1e3, t_carve * 1e3,
                diag_t0.elapsed().as_secs_f64() * 1e3,
                (diag_t0.elapsed().as_secs_f64() - t_biome - t_carve) * 1e3);
        }
    }

    // 预加载 carver JSON（创建时调用）。260907-05：路径按 id 命名空间解析（缺口 3 同源）
    fn load_carver(wg_dir: &str, blocks: &BlockRegistry, id: &str) -> Option<ConfiguredCarver> {
        let (ns, short) = match id.split_once(':') { Some((n, s)) => (n, s), None => ("minecraft", id) };
        let path = format!("{}/data/{}/worldgen/configured_carver/{}.json", wg_dir, ns, short);
        let txt = std::fs::read_to_string(&path).ok()?;
        let root = parse(&txt).ok()?;
        Some(ConfiguredCarver::parse(&root, blocks))
    }

    // 取 carver（创建时已预加载，运行只读无锁）
    fn get_carver(&self, id: &str) -> Option<ConfiguredCarver> {
        self.carver_cache.get(id).cloned()
    }

    // c-A-min：邻 chunk 地形列（带缓存；region 顺序扫描下摊销后每 chunk 约 +1 次地形成本）。
    // 缓存只存地形列（无 feature），锁临界区极短；fill_terrain_column 自身不触缓存 → 无重入死锁。
    fn neighbor_terrain(&self, cx: i32, cz: i32) -> std::sync::Arc<crate::blocks::BlockColumn> {
        const CAP: usize = 256; // ~25MB 上限（每列 16*16*384*i32 ≈ 98KB），超限整体清空（region 局部性下安全）
        if let Ok(cache) = self.terrain_cache.lock() {
            if let Some(a) = cache.get(&(cx, cz)) {
                return a.clone();
            }
        }
        let (col, _hm) = self.fill_terrain_column(cx, cz);
        let arc = std::sync::Arc::new(col);
        if let Ok(mut cache) = self.terrain_cache.lock() {
            if cache.len() >= CAP {
                cache.clear();
            }
            cache.insert((cx, cz), arc.clone());
        }
        arc
    }

    // FEATURES 阶段：装饰层（矿石/disk/spring/freeze_top/underwater_magma）。
    // 对齐 C++ applyCarversAndFeatures 的 FEATURES 部分（worldgen_api.cpp L1584-1674）。
    // 简化：set = 当前 chunk biome（Java 是 3×3 chunk 所有 biome section）；structure 部分跳过。
    // 返回放置的方块数（诊断用）。
    fn apply_features(&self, col: &mut BlockColumn, cx: i32, cz: i32,
                       heightmap: &[i32],
                       biome_at: &dyn Fn(i32, i32, i32) -> String) -> usize {
        let min_y = self.min_y;
        let height = self.height;
        let mut placed_count = 0;

        // c-A-min/write（260905-09，方案 260905-08-cA-block-boundary.md §4/§6）；
        // 260905-10 用户拍板默认关（c-A 性能代价 +68%，223.7s vs 133.3s dump 计时）。
        // 260909-03 用户拍板翻默认开（语义优先）：生产路径越界读恒 -1 使 Ore isExposedToAir /
        // Geode isAir 的邻 chunk 语义修复 no-op（.investigations/mc-1216-features-takeover/
        // t3-rerun-record-260909-03.md 归因链 2）；WG_CA_MIN=0 显式关闭可回退。
        let ca_min = std::env::var("WG_CA_MIN").map(|v| v != "0").unwrap_or(true);
        if ca_min {
            // c-A-write：features 前 overlay 邻 chunk 先行生成的跨 chunk 写入（Java 写持久语义）
            if let Ok(mut pc) = self.pending_cross_writes.lock() {
                if let Some(v) = pc.remove(&(cx, cz)) {
                    for (idx, state) in v {
                        let ly = (idx / 256) as i32;
                        let lz = ((idx % 256) / 16) as i32;
                        let lx = (idx % 16) as i32;
                        *col.at_mut(lx, min_y + ly, lz) = state;
                    }
                }
            }
        }
        // c-A 诊断计数（WG_CA_LOG=1 chunk 级门控输出；零门控成本）
        static CA_OUT_READS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        static CA_PENDING_WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let ca_log = std::env::var("WG_CA_LOG").is_ok();
        if ca_log { CA_OUT_READS.store(0, std::sync::atomic::Ordering::Relaxed); CA_PENDING_WRITES.store(0, std::sync::atomic::Ordering::Relaxed); }
        // c-A-write 回调：越界写 → pending 缓冲（Mutex 短临界区；本 chunk 自身写不走此路径）
        let pending_cross_cb = |tcx: i32, tcz: i32, idx: i32, state: i32| {
            if ca_log { CA_PENDING_WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            if let Ok(mut pc) = self.pending_cross_writes.lock() {
                pc.entry((tcx, tcz)).or_default().push((idx as usize, state));
            }
        };

        // OCEAN_FLOOR_WG 高度图：每列从顶向下扫，跳过 air/water/lava，取第一个固体 y（海底/地表）。
        // Ore/disk/spring 用 getOceanFloorTopY 判断放置位置（Java OCEAN_FLOOR_WG 构建于 carver 前）。
        let mut ocean_floor = vec![min_y - 1; 256];
        let air_id = self.blocks.id("minecraft:air");
        let water_id = self.blocks.id("minecraft:water");
        let lava_id = self.blocks.id("minecraft:lava");
        for lz in 0..16 {
            for lx in 0..16 {
                for wy in (min_y..min_y + height).rev() {
                    let b = col.at(lx, wy, lz);
                    if b != air_id && b != water_id && b != lava_id {
                        ocean_floor[(lz * 16 + lx) as usize] = wy;
                        break;
                    }
                }
            }
        }
        // biomeAtNoJitter：无 jitter 直采（泛化 y 参数，Fix-1 3×3 全高并集用）
        let biome_at_no_jitter = |wx: i32, wy: i32, wz: i32| -> String {
            let bp = NoisePos { x: wx, y: wy, z: wz };
            self.biomesrc.biome(&bp)
        };
        // biomeAtJitter：8 邻域 jitter（posToBiome 用）
        let biome_at_jitter = |x: i32, y: i32, z: i32| -> String {
            let (px, py, pz) = crate::biome::biome_pick_cell(self.biome_access_seed, x, y, z);
            let bp = NoisePos { x: px << 2, y: py << 2, z: pz << 2 };
            self.biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { crate::surface_rules::biome_temperature(id) };

        // 当前 chunk biome（anchor_biome 保留：Fix-2 落地前 Biome modifier 仍需）
        let cur_biome_id = biome_at_no_jitter(cx * 16, 0, cz * 16);
        // Fix-1（.b2a）：Java ChunkGenerator.java:346-353 = 3×3 邻域 chunk 全高 biome 容器并集；
        // 此前只用当前 chunk biome → 边界 chunk 缺邻域 biome 独有 feature（jungle −22,611 主候选）。
        // Y 全高并集在本实现以 Y 切片近似（IDK-1：洞窟 biome 地表 step feature 可能漏）。
        let mut biome_set: Vec<String> = Vec::new();
        for nx in (cx - 1)..=(cx + 1) {
            for nz in (cz - 1)..=(cz + 1) {
                for gx in 0..4 { for gz in 0..4 {
                    let wx = nx * 16 + gx * 4;
                    let wz = nz * 16 + gz * 4;
                    for wy in [0, 64, 128] {
                        let b = biome_at_no_jitter(wx, wy, wz);
                        if !biome_set.contains(&b) { biome_set.push(b); }
                    }
                }}
            }
        }
        // 并集内各 biome 的 feature entry（Java :381 intSet = ∪ set 中各 entry 的 step k，
        // entry = biome GenerationSettings 的 per-step feature 表，不去重 entry 本身）
        // 260905-08：Fix-1 恢复（p 序修复后重新验证；此前单变量撤回态）
        let mut entries: Vec<Vec<Vec<String>>> = Vec::new();
        for b in &biome_set {
            entries.push(self.biomesrc.bc.features_for(b).to_vec());
        }
        if entries.is_empty() { return 0; }

        // ChunkRandom(Xoroshiro base)（generateFeatures 用，与 carver 的 CHECKED 不同！）
        let mut feat_random = ChunkRandom::xoroshiro();
        // setPopulationSeed(worldSeed, blockX, blockZ)
        let population_seed = feat_random.set_population_seed(self.seed, cx * 16, cz * 16);
        // 用全局 PlacedFeatureIndexer（Java 语义：p = lastIndex 在所有 biome features 中）
        // 构建后只读，&self 共享并发安全（无锁）
        let indexer = &self.feature_indexer;
        let max_step = indexer.step_features.len(); // Fix-1：Java :358 max(values().length, i) ≥ 全局步数，此前按 biome feature 数提前截断

        for k in 0..max_step {
            // Java :381-395：intSet = ∪ 各 entry 的 indexMapping[step]，去重后排序
            let mut int_set: Vec<i32> = Vec::new();
            for e in &entries {
                for p in indexer.int_set_for(e, k as i32) {
                    if !int_set.contains(&p) { int_set.push(p); }
                }
            }
            int_set.sort_unstable(); int_set.dedup(); // Java :394-395 toIntArray+sort
            for p in int_set {
                // p = lastIndex → featureId = stepFeatures[k][p]
                if k >= indexer.step_features.len() { continue; }
                let step_list = &indexer.step_features[k];
                if p < 0 || p as usize >= step_list.len() { continue; }
                let fid = step_list[p as usize].clone();
                feat_random.set_decorator_seed(population_seed, p, k as i32);
                if std::env::var("WG_FEATURELOG").is_ok() {
                    eprintln!("[FEATURE] chunk({},{}) step={} p={} fid={}", cx, cz, k, p, fid);
                }
                // PlacedFeature（创建时已预加载，运行只读无锁）
                let pf = self.feature_cache.placed.get(&fid).cloned();
                let pf = match pf {
                    Some(pf) => pf,
                    None => {
                        if std::env::var("WG_FEATURELOG").is_ok() {
                            eprintln!("[FEATURE] MISS placed cache: {fid}");
                        }
                        continue
                    }
                };
                // FeaturePlacementContext
                // 260905-06：block_at 接入本 chunk 列（idk-7 链：block_predicate_filter/would_survive
                // 需读实况方块；此前恒 None → 谓词全 false → 树全灭）。
                // 安全性（并发不变量，judge C-4）：col 为 apply_features 独占参数，本指针不跨线程共享、
                // 闭包只读且仅在本次调用栈内存活（fctx 同域销毁）；与 octx 的 &mut 写访问交错发生在
                // 不同时点（谓词读 / 放置写），无同时别名访问。
                // 已知语义偏差（judge C-3）：越界（邻 chunk）返回 -1 = 保守拒绝，Java 读邻 chunk 实况。
                let col_ptr: *const crate::blocks::BlockColumn = &*col;
                let self_ref: &Self = self;
                let block_at_col = move |bx: i32, by: i32, bz: i32| -> i32 {
                    let lx = bx - cx * 16;
                    let lz = bz - cz * 16;
                    if by < min_y || by >= min_y + height {
                        return -1;
                    }
                    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 {
                        unsafe { (*col_ptr).at(lx, by, lz) }
                    } else if ca_min {
                        // c-A-min：邻 chunk 越界读 → 地形列缓存（noise+surface+carver，无 feature；
                        // 对齐 Java「FEATURES 时邻 chunk ≥ post-carver 地形态」时序保证）
                        if ca_log { CA_OUT_READS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                        let t = self_ref.neighbor_terrain(bx >> 4, bz >> 4);
                        t.at(bx - (bx >> 4) * 16, by, bz - (bz >> 4) * 16)
                    } else {
                        // 已知语义偏差（judge C-3）：越界返回 -1 = 保守拒绝
                        -1
                    }
                };
                // Fix-2（.b2b）：Biome modifier 允许集闭包——jitter 点 biome 的 feature 集含 fid
                let biome_allows = |bx: i32, by: i32, bz: i32, fid: &str| -> bool {
                    let b = biome_at_jitter(bx, by, bz);
                    self.biomesrc.bc.features_for(&b).iter().any(|step| step.iter().any(|f| f == fid))
                };
                let fctx = crate::placement::FeaturePlacementContext {
                    biome_at: Some(biome_at),
                    // 260905-06：接 ocean_floor（此前 None → heightmap(OCEAN_FLOOR) 静默直通 y=min_y
                    // → selector 内层 would_survive 全 false → 树全灭）
                    ocean_floor: Some(&ocean_floor),
                    world_surface: Some(heightmap),
                    min_y, height,
                    pos_to_biome: Some(&biome_at_jitter),
                    chunk_start_x: cx * 16,
                    chunk_start_z: cz * 16,
                    block_at: Some(&block_at_col),
                    // Fix-2（.b2b）：jitter 采样 + 允许集判定（替代 anchor_biome 近似，已删除）
                    biome_allows: Some(&biome_allows),
                    feature_id: Some(fid.clone()),
                };
                // OreFeatureContext（不持有 random，由 generate_configured 传入）
                let mut octx = crate::feature::OreFeatureContext {
                    col,
                    origin_x: 0, origin_y: 0, origin_z: 0,
                    chunk_start_x: cx * 16,
                    chunk_start_z: cz * 16,
                    min_y, height,
                    blocks: self.blocks,
                    ocean_floor: Some(&ocean_floor),
                    world_surface: Some(heightmap),
                    region_col_at: None,
                    // c-A-min：octx 侧任意点读（树冠 can_replace 等）与 fctx.block_at 同源路由
                    block_at_ext: if ca_min { Some(&block_at_col) } else { None },
                    pending_cross: if ca_min { Some(&pending_cross_cb) } else { None },
                    // batchA（mc-1216）：geode 噪声采样器种子（GeodeFeature.java:41）
                    world_seed: self.seed,
                };
                // ConfiguredFeature（创建时已预加载，运行只读无锁）
                let cf = self.feature_cache.configured.get(&pf.configured_feature).cloned();
                let cf = match cf {
                    Some(cf) => cf,
                    None => {
                        if std::env::var("WG_FEATURELOG").is_ok() {
                            eprintln!("[FEATURE] MISS configured cache: {} (placed {})", pf.configured_feature, fid);
                        }
                        continue
                    }
                };
                let biome_temp_f = biome_temp(&cur_biome_id) as f32;
                // 260905-05（patch §2.4）：generate_configured 增 cache 实参（preload 后只读共享引用，
                // apply_features 阶段 cache 不再写——无可变借用冲突）
                let feature_cache_ref = &self.feature_cache;
                let generate_configured = |_fctx: &crate::placement::FeaturePlacementContext, random: &mut ChunkRandom, gx: i32, gy: i32, gz: i32| -> bool {
                    let r = crate::feature_loader::generate_configured(&cf, &fctx, &mut octx, random, gx, gy, gz, biome_temp_f, 0.5, feature_cache_ref);
                    if r {
                        placed_count += 1;
                        if std::env::var("WG_FEATURELOG").is_ok() {
                            eprintln!("[FEATURE] fid={} placed at ({},{},{})", cf.id, gx, gy, gz);
                        }
                    }
                    r
                };
                pf.generate(&fctx, &mut feat_random, cx * 16, min_y, cz * 16, generate_configured);
            }
        }
        // batch0（mc-1216）：features 阶段结束 unknown-type 汇总（进程级去重集合，
        // 每 chunk 一次调用；env 门控 OnceLock 进程级读一次，热路径近零成本）
        crate::feature_loader::report_unknown_types();
        if ca_log {
            eprintln!("[CA] chunk({},{}) out_reads={} pending_writes={} placed={}", cx, cz,
                CA_OUT_READS.load(std::sync::atomic::Ordering::Relaxed),
                CA_PENDING_WRITES.load(std::sync::atomic::Ordering::Relaxed), placed_count);
        }
        placed_count
    }
}








