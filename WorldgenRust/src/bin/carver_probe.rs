// carver_probe.rs — CARVERS 阶段验证：Rust(fill_chunk + build_surface + carver) vs vanilla FULL 参照。
// 读 vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks（含 carver+features）。
// Rust 用阶段 A 管线产出具体 block id + carver 洞穴雕刻，对比 vanilla。
// 验证目标：carver 开启后 match 率应超过 SURFACE-only（93.45%），洞穴挖洞位置与 vanilla 重合。
use std::collections::HashMap;
use std::sync::Arc;

use WorldgenRust::blocks::{BlockColumn, BlockRegistry};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::surface_rules::SurfaceBuilder;
use WorldgenRust::terrain::{fill_chunk, VanillaDensity, VanillaAquifer, BiomeSource};
use WorldgenRust::biome::{BiomeClassifier, biome_pick_cell};
use WorldgenRust::carver::{CarverContext, ConfiguredCarver, CarvingMask};
use WorldgenRust::chunkrandom::ChunkRandom;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

struct MacroBiome { bc: BiomeClassifier, tempf: Arc<DensityFunction>, humf: Arc<DensityFunction>, contf: Arc<DensityFunction>, erof: Arc<DensityFunction>, depthf: Arc<DensityFunction>, weirdf: Arc<DensityFunction> }
impl BiomeSource for MacroBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).unwrap());
    let barrier: Arc<DensityFunction> = Arc::new(db.build_node(router.get("barrier").unwrap()).unwrap());
    let flooded: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_floodedness").unwrap()).unwrap());
    let spread: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_spread").unwrap()).unwrap());
    let lava: Arc<DensityFunction> = Arc::new(db.build_node(router.get("lava").unwrap()).unwrap());
    let erosion: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let depth: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let init: Arc<DensityFunction> = Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).unwrap());
    let t_temp = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let t_hum = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let t_cont = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let t_ero = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let t_dep = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let t_wei = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();

    for key in ["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset",
                "minecraft:badlands_surface", "minecraft:badlands_pillar", "minecraft:calcite",
                "minecraft:gravel", "minecraft:powder_snow", "minecraft:packed_ice", "minecraft:ice",
                "minecraft:surface_swamp"] {
        let _ = db.get_noise_sampler(key);
    }

    let blocks_json = fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\blocks.json").unwrap();
    let blocks = BlockRegistry::load_from_json(&blocks_json).expect("blocks.json");

    let dense = VanillaDensity { df: &tree };
    let mut bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let n_carvers = bc.load_carvers("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\biome");
    println!("loaded carvers for {} biomes", n_carvers);
    let biomesrc = MacroBiome { bc, tempf: t_temp, humf: t_hum, contf: t_cont, erof: t_ero, depthf: t_dep, weirdf: t_wei };
    let sb = SurfaceBuilder::new(db.noise_samplers(), db.random_deriver(), 63, &blocks);
    let rule = sb.build_overworld_rule();

    let stone = blocks.id("minecraft:stone");
    let air = blocks.id("minecraft:air");
    let water = blocks.id("minecraft:water");
    let lava_id = blocks.id("minecraft:lava");

    // 读 vanilla FULL 参照（-8248 种子 4x4 origin -288,-256，含 carver+features）
    let path = "E:\\python\\MC\\data\\vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks";
    let bd = fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = 16*16*height as usize;

    // 预加载 configured_carver（按 id）
    let carver_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\configured_carver";
    let mut carver_cache: HashMap<String, ConfiguredCarver> = HashMap::new();
    let get_carver = |id: &str, cache: &mut HashMap<String, ConfiguredCarver>| -> Option<ConfiguredCarver> {
        if let Some(c) = cache.get(id) { return Some(c.clone()); }
        let name = if let Some(s) = id.strip_prefix("minecraft:") { s } else { id };
        let path = format!("{}/{}.json", carver_dir, name);
        let txt = fs::read_to_string(&path).ok()?;
        let root = parse(&txt).ok()?;
        let cc = ConfiguredCarver::parse(&root, &blocks);
        cache.insert(id.to_string(), cc.clone());
        Some(cc)
    };

    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    let mut rust_carved = 0u64; // Rust carver 实际挖掉的块数（surface 后 rock → carver 后 air）
    let mut rust_carved_match = 0u64; // Rust 挖的洞中，vanilla 也是 air（挖洞重合）
    let mut vanilla_carved = 0u64; // vanilla 挖的洞（vanilla air 且 surface-only Rust 是 rock）
    let mut rust_carved_above_surface = 0u64; // Rust 挖的洞中，y >= 地表（异常，carver 不应挖地表以上）
    for _c in 0..(size*size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl>0 { i += bl; } } // biome 段

        // Rust 阶段 A 管线：fill_chunk 宏观 → BlockColumn → build_surface 具体 block id
        let mut aq = WorldgenRust::aquifer::Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &biomesrc, cx, cz, min_y, height, None);
        let mut col = BlockColumn::new(min_y, height);
        for lz in 0..16 { for lx in 0..16 { for ly in 0..height {
            let y = min_y + ly;
            let kind = cd.blocks[(lx + lz*16 + ly*256) as usize];
            let id = match kind {
                WorldgenRust::terrain::BlockKind::Air => air,
                WorldgenRust::terrain::BlockKind::Rock => stone,
                WorldgenRust::terrain::BlockKind::Water => water,
                WorldgenRust::terrain::BlockKind::Lava => lava_id,
            };
            *col.at_mut(lx, y, lz) = id;
        }}}
        let heightmap: Vec<i32> = cd.surface_height.to_vec();
        let est_at = |x: i32, z: i32| -> i32 {
            let mut est = i32::MAX;
            for y in (min_y..min_y + height).rev().step_by(8) {
                if init.sample(&NoisePos { x, y, z }) > 0.390625 { est = y; break; }
            }
            est
        };
        let surface_heights4 = vec![
            est_at(cx*16, cz*16), est_at(cx*16+15, cz*16), est_at(cx*16, cz*16+15), est_at(cx*16+15, cz*16+15),
        ];
        let biome_at = |x: i32, y: i32, z: i32| -> String {
            let bp = NoisePos { x: (x>>2)<<2, y: (y>>2)<<2, z: (z>>2)<<2 };
            biomesrc.biome(&bp)
        };
        let biome_temp = |id: &str| -> f64 { WorldgenRust::surface_rules::biome_temperature(id) };
        let initial_density_at = |x: i32, y: i32, z: i32| -> f64 { init.sample(&NoisePos { x, y, z }) };
        sb.build_surface(&mut col, &rule, cx*16, cz*16, &heightmap, &surface_heights4,
                         &biome_at, &|x,y,z| ((x as i64)<<32) ^ (z as i64), &biome_temp, min_y, height, &initial_density_at);

        // 快照 surface 后状态（carver 前），用于统计 Rust 实际挖洞数
        let pre_carve: Vec<i32> = col.data().to_vec();

        // ---- CARVERS 阶段（对齐 C++ applyCarversAndFeatures）----
        let skip_carver = std::env::var("WG_SKIP_CARVER").is_ok();
        if !skip_carver {
        // biomeAtNoJitter：chunk 角采样（无 jitter）
        let biome_at_no_jitter = |cx2: i32, cz2: i32| -> String {
            let bp = NoisePos { x: cx2*16, y: 0, z: cz2*16 };
            biomesrc.biome(&bp)
        };
        // biomeAtJitter：8 邻域 jitter（applyMaterialRule 用）
        let biome_at_jitter = |x: i32, y: i32, z: i32| -> String {
            let (px, py, pz) = biome_pick_cell(seed, x, y, z);
            let bp = NoisePos { x: px<<2, y: py<<2, z: pz<<2 };
            biomesrc.biome(&bp)
        };
        // applyMaterialRule：surface 单点（grass 挖后 dirt 替换）
        let apply_material_rule = |x: i32, y: i32, z: i32, has_fluid: bool| -> Option<i32> {
            sb.apply_material_rule_single(&rule, &biome_at_jitter, &biome_temp, x, y, z, has_fluid, min_y, height, &initial_density_at)
        };

        let mut ctx = CarverContext {
            min_y, height,
            aquifer: &mut va.aq,
            blocks: &blocks,
            apply_material_rule: Some(&apply_material_rule),
        };
        let mut mask = CarvingMask::new(height, min_y);
        let mut chunk_random = ChunkRandom::checked();
        for j in -8..=8 {
            for k in -8..=8 {
                let cx2 = cx + j; let cz2 = cz + k;
                let biome_id = biome_at_no_jitter(cx2, cz2);
                let carvers = biomesrc.bc.carvers_for(&biome_id).to_vec();
                let mut l = 0;
                for carver_id in &carvers {
                    let cc = match get_carver(carver_id, &mut carver_cache) { Some(c) => c, None => { l += 1; continue; } };
                    chunk_random.set_carver_seed(seed + l, cx2, cz2);
                    if cc.should_carve(&mut chunk_random) {
                        cc.carve(&mut ctx, &mut col, &biome_at_jitter, &mut chunk_random, cx2, cz2, cx, cz, &mut mask);
                    }
                    l += 1;
                }
            }
        }
        } // end if !skip_carver

        // 对比
        for k in 0..bpc {
            let lx = (k % 16) as i32; let ly = (k / 256) as i32; let lz = ((k / 16) % 16) as i32;
            let y = min_y + ly;
            let got = col.at(lx, y, lz);
            total += 1;
            if vanilla[k] != 0 { tnair += 1; }
            if got == vanilla[k] { match_t += 1; if vanilla[k] != 0 { mnair += 1; } }
            // Rust carver 实际挖洞：surface 后非 air → carver 后 air
            if pre_carve[k] != 0 && got == 0 {
                rust_carved += 1;
                if vanilla[k] == 0 { rust_carved_match += 1; }
                // 检查是否挖到地表以上（异常）
                if y >= heightmap[(lz*16+lx) as usize] { rust_carved_above_surface += 1; }
            }
            // vanilla 挖的洞：vanilla air 且 surface-only Rust 是 rock（即 carver 应挖的洞）
            if vanilla[k] == 0 && pre_carve[k] != 0 { vanilla_carved += 1; }
        }
    }
    println!("Rust(surface+carver) vs vanilla FULL: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
    println!("Rust carved: {} (match vanilla air {})  vanilla carved: {}  overlap: {:.2}%", rust_carved, rust_carved_match, vanilla_carved, if vanilla_carved>0 {100.0*rust_carved_match as f64/vanilla_carved as f64} else {0.0});
    println!("Rust carved above surface (anomaly): {}", rust_carved_above_surface);
}
