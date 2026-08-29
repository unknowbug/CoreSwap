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
use crate::xoroshiro::XoroshiroSplitter;

// 宏观 biome 源（BiomeClassifier + 6 参数 DF）
struct MacroBiome {
    bc: BiomeClassifier,
    tempf: Arc<DensityFunction>,
    humf: Arc<DensityFunction>,
    contf: Arc<DensityFunction>,
    erof: Arc<DensityFunction>,
    depthf: Arc<DensityFunction>,
    weirdf: Arc<DensityFunction>,
}
impl BiomeSource for MacroBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}

// 生产句柄：一次 seed 初始化（构建全部 noise samplers + density 树 + biome + surface）。
pub struct WorldgenHandle {
    pub seed: i64,
    pub min_y: i32,
    pub height: i32,
    // density 树
    tree: Arc<DensityFunction>,       // final_density
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
    // aquifer splitter
    splitter: XoroshiroSplitter,
    // beardifier 缓存（per-chunk）——未接入生成管线，保留（set 时写）
    beardifiers: Mutex<HashMap<(i32, i32), Beardifier>>,
    // carver 缓存（创建时预加载，运行只读无锁）
    carver_cache: HashMap<String, ConfiguredCarver>,
    // FEATURES 缓存（创建时从所有 biome features 预加载，运行只读无锁）
    feature_cache: crate::feature_loader::FeatureCache,
    // FEATURES indexer（Java PlacedFeatureIndexer，从所有 biome features 构建，构建后只读——&self 共享并发安全，不需锁）
    feature_indexer: crate::feature_loader::PlacedFeatureIndexer,
    // 数据目录
    wg_dir: String,
}

impl WorldgenHandle {
    // 从 worldgen_dir 构建句柄。worldgen_dir 指向含 data/minecraft/worldgen/... 的目录。
    // 数据文件约定（对齐 C++ wg_create）：
    //   <dir>/data/minecraft/worldgen/noise_settings/overworld.json
    //   <dir>/data/minecraft/worldgen/density_function/overworld/*.json
    //   <dir>/../noise_params.json
    //   <dir>/../biome_params.json
    //   <dir>/../blocks.json
    pub fn create(seed: i64, worldgen_dir: &str) -> Option<WorldgenHandle> {
        let wg_dir = worldgen_dir.to_string();
        let min_y = -64;
        let height = 384;

        // 1. DensityBuilder（noise_params + density_function 目录）
        let mut db = DensityBuilder::new(seed as u64, min_y, height);
        let noise_params_path = format!("{}/../noise_params.json", wg_dir);
        if db.load_noise_params_file(&noise_params_path).is_err() {
            eprintln!("wg_create: cannot load {}", noise_params_path);
            return None;
        }
        let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
        let df_dir2 = df_dir.clone();
        db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
            let p = format!("{}/{}.json", df_dir2, name);
            std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}: {}", p, e))
        }));

        // 2. noise_settings（router）
        let settings_path = format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir);
        let settings_txt = std::fs::read_to_string(&settings_path).ok()?;
        let settings = parse(&settings_txt).ok()?;
        let router = settings.get("noise_router")?;

        // 3. router DF 树
        let tree = Arc::new(db.build_node(router.get("final_density")?).ok()?);
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

        // 4. noise samplers（surface rules 用）
        for key in ["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset",
                    "minecraft:badlands_surface", "minecraft:badlands_pillar", "minecraft:calcite",
                    "minecraft:gravel", "minecraft:powder_snow", "minecraft:packed_ice", "minecraft:ice",
                    "minecraft:surface_swamp"] {
            let _ = db.get_noise_sampler(key);
        }

        // 5. blocks registry
        let blocks_path = format!("{}/../blocks.json", wg_dir);
        let blocks_json = std::fs::read_to_string(&blocks_path).ok()?;
        let blocks = BlockRegistry::load_from_json(&blocks_json)?;

        // 6. biome classifier + carvers + features
        let biome_params_path = format!("{}/../biome_params.json", wg_dir);
        let mut bc = BiomeClassifier::load(&biome_params_path);
        let biome_dir = format!("{}/data/minecraft/worldgen/biome", wg_dir);
        let _n = bc.load_carvers(&biome_dir);
        let _nf = bc.load_features(&biome_dir);

        // 7. surface builder（Box::leak 让引用长期存活）
        let samplers = Box::leak(Box::new(db.noise_samplers().clone()));
        let splitter = Box::leak(Box::new(db.random_deriver().clone()));
        let blocks_leaked = Box::leak(Box::new(blocks));
        let sb = SurfaceBuilder::new(samplers, splitter, 63, blocks_leaked);
        let rule = sb.build_overworld_rule();

        let biomesrc = MacroBiome { bc, tempf, humf, contf, erof, depthf, weirdf };
        let splitter = db.random_deriver().clone();

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

        Some(WorldgenHandle {
            seed, min_y, height,
            tree, barrier, flooded, spread, lava, erosion, depth, init,
            biomesrc, sb, rule,
            blocks: blocks_leaked,
            splitter,
            beardifiers: Mutex::new(HashMap::new()),
            carver_cache,
            feature_cache,
            feature_indexer,
            wg_dir,
        })
    }

    // 设置指定 chunk 的 Beardifier（StructureWeightSampler）输入。
    // pieces 每 8 int：{minX,minY,minZ,maxX,maxY,maxZ,terrain(0-3),groundLevelDelta}
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
        self.beardifiers.lock().unwrap().insert((chunk_x, chunk_z), b);
    }

    pub fn clear_beardifier(&self) {
        self.beardifiers.lock().unwrap().clear();
    }

    // 完整区块生成（方块层）：fill_chunk（宏观）→ BlockColumn → build_surface → carver。
    // 返回 16*16*height 的 vanilla raw block id（索引 (y-min_y)*256 + z*16 + x）。
    pub fn fill_chunk_blocks(&self, cx: i32, cz: i32) -> Vec<BlockId> {
        let min_y = self.min_y;
        let height = self.height;
        let air = self.blocks.id("minecraft:air");
        let stone = self.blocks.id("minecraft:stone");
        let water = self.blocks.id("minecraft:water");
        let lava_id = self.blocks.id("minecraft:lava");

        // 1. fill_chunk（宏观：density + aquifer 分类）
        let dense = VanillaDensity { df: &self.tree };
        let mut aq = crate::aquifer::Aquifer::new(
            self.barrier.clone(), self.flooded.clone(), self.spread.clone(), self.lava.clone(),
            self.erosion.clone(), self.depth.clone(), self.init.clone(), self.splitter.clone(),
            cx * 16, cz * 16, min_y, height);
        let mut va = VanillaAquifer { aq };
        let cd = fill_chunk(&dense, &mut va, &self.biomesrc, cx, cz, min_y, height, None);

        // 2. 宏观 → BlockColumn（具体 block id 占位：air/stone/water/lava）
        let mut col = BlockColumn::new(min_y, height);
        for lz in 0..16 { for lx in 0..16 { for ly in 0..height {
            let y = min_y + ly;
            let kind = cd.blocks[(lx + lz * 16 + ly * 256) as usize];
            let id = match kind {
                crate::terrain::BlockKind::Air => air,
                crate::terrain::BlockKind::Rock => stone,
                crate::terrain::BlockKind::Water => water,
                crate::terrain::BlockKind::Lava => lava_id,
            };
            *col.at_mut(lx, y, lz) = id;
        }}}

        // 3. build_surface（具体 block id：grass/sand/terracotta 等）
        let heightmap: Vec<i32> = cd.surface_height.to_vec();
        let est_at = |x: i32, z: i32| -> i32 {
            let mut est = i32::MAX;
            for y in (min_y..min_y + height).rev().step_by(8) {
                if self.init.sample(&NoisePos { x, y, z }) > 0.390625 { est = y; break; }
            }
            est
        };
        let surface_heights4 = vec![
            est_at(cx * 16, cz * 16), est_at(cx * 16 + 15, cz * 16),
            est_at(cx * 16, cz * 16 + 15), est_at(cx * 16 + 15, cz * 16 + 15),
        ];
        let biome_at = |x: i32, y: i32, z: i32| -> String {
            let bp = NoisePos { x: (x >> 2) << 2, y: (y >> 2) << 2, z: (z >> 2) << 2 };
            self.biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { crate::surface_rules::biome_temperature(id) };
        let initial_density_at = |x: i32, y: i32, z: i32| -> f64 { self.init.sample(&NoisePos { x, y, z }) };
        self.sb.build_surface(&mut col, &self.rule, cx * 16, cz * 16, &heightmap, &surface_heights4,
                              &biome_at, &|x, y, z| ((x as i64) << 32) ^ (z as i64), &biome_temp, min_y, height, &initial_density_at);

        // 4. carver（洞穴雕刻，17×17 邻域）
        self.apply_carvers(&mut col, cx, cz, &mut va.aq, &biome_at);

        // 5. features（装饰层：矿石/disk/spring/freeze_top/underwater_magma）
        let skip_features = std::env::var("WG_SKIP_FEATURES").is_ok();
        if !skip_features {
            let n_features = self.apply_features(&mut col, cx, cz, &heightmap, &biome_at);
            if std::env::var("WG_FEATURELOG").is_ok() {
                eprintln!("[FEATURE] chunk({},{}) placed {} blocks", cx, cz, n_features);
            }
        }

        col.data().to_vec()
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
            let (px, py, pz) = crate::biome::biome_pick_cell(self.seed, x, y, z);
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
        for j in -8..=8 {
            for k in -8..=8 {
                let cx2 = cx + j; let cz2 = cz + k;
                let biome_id = biome_at_no_jitter(cx2, cz2);
                let carvers = self.biomesrc.bc.carvers_for(&biome_id).to_vec();
                let mut l = 0;
                for carver_id in &carvers {
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
    }

    // 预加载 carver JSON（创建时调用）
    fn load_carver(wg_dir: &str, blocks: &BlockRegistry, id: &str) -> Option<ConfiguredCarver> {
        let name = if let Some(s) = id.strip_prefix("minecraft:") { s } else { id };
        let path = format!("{}/data/minecraft/worldgen/configured_carver/{}.json", wg_dir, name);
        let txt = std::fs::read_to_string(&path).ok()?;
        let root = parse(&txt).ok()?;
        Some(ConfiguredCarver::parse(&root, blocks))
    }

    // 取 carver（创建时已预加载，运行只读无锁）
    fn get_carver(&self, id: &str) -> Option<ConfiguredCarver> {
        self.carver_cache.get(id).cloned()
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
        // biomeAtNoJitter：chunk 角采样（无 jitter）
        let biome_at_no_jitter = |cx2: i32, cz2: i32| -> String {
            let bp = NoisePos { x: cx2 * 16, y: 0, z: cz2 * 16 };
            self.biomesrc.biome(&bp)
        };
        // biomeAtJitter：8 邻域 jitter（posToBiome 用）
        let biome_at_jitter = |x: i32, y: i32, z: i32| -> String {
            let (px, py, pz) = crate::biome::biome_pick_cell(self.seed, x, y, z);
            let bp = NoisePos { x: px << 2, y: py << 2, z: pz << 2 };
            self.biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { crate::surface_rules::biome_temperature(id) };

        // 当前 chunk biome
        let cur_biome_id = biome_at_no_jitter(cx, cz);
        let cur_features = self.biomesrc.bc.features_for(&cur_biome_id).to_vec();
        if cur_features.is_empty() { return 0; }

        // ChunkRandom(Xoroshiro base)（generateFeatures 用，与 carver 的 CHECKED 不同！）
        let mut feat_random = ChunkRandom::xoroshiro();
        // setPopulationSeed(worldSeed, blockX, blockZ)
        let population_seed = feat_random.set_population_seed(self.seed, cx * 16, cz * 16);
        let max_step = cur_features.len();

        // 用全局 PlacedFeatureIndexer（Java 语义：p = lastIndex 在所有 biome features 中）
        // 构建后只读，&self 共享并发安全（无锁）
        let indexer = &self.feature_indexer;

        for k in 0..max_step {
            let int_set = indexer.int_set_for(&cur_features, k as i32);
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
                let pf = match pf { Some(pf) => pf, None => continue };
                // FeaturePlacementContext
                let fctx = crate::placement::FeaturePlacementContext {
                    biome_at: Some(biome_at),
                    ocean_floor: None,
                    world_surface: Some(heightmap),
                    min_y, height,
                    pos_to_biome: Some(&biome_at_jitter),
                    chunk_start_x: cx * 16,
                    chunk_start_z: cz * 16,
                    block_at: None,
                };
                // OreFeatureContext（不持有 random，由 generate_configured 传入）
                let mut octx = crate::feature::OreFeatureContext {
                    col,
                    origin_x: 0, origin_y: 0, origin_z: 0,
                    chunk_start_x: cx * 16,
                    chunk_start_z: cz * 16,
                    min_y, height,
                    blocks: self.blocks,
                    ocean_floor: None,
                    world_surface: Some(heightmap),
                    region_col_at: None,
                    pending_cross: None,
                };
                // ConfiguredFeature（创建时已预加载，运行只读无锁）
                let cf = self.feature_cache.configured.get(&pf.configured_feature).cloned();
                let cf = match cf { Some(cf) => cf, None => continue };
                let biome_temp_f = biome_temp(&cur_biome_id) as f32;
                let generate_configured = |_fctx: &crate::placement::FeaturePlacementContext, random: &mut ChunkRandom, gx: i32, gy: i32, gz: i32| -> bool {
                    let r = crate::feature_loader::generate_configured(&cf, &fctx, &mut octx, random, gx, gy, gz, biome_temp_f, 0.5);
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
        placed_count
    }
}
