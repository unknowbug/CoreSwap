// blocks_cmp.rs — ④ 交叉验证：Rust(fill_chunk + build_surface 具体 block id) vs Java 原版 blocks。
// 读 vanilla_-2032795982907864146_4_0_0.blocks（Java big-endian：块 id 2 字节 + biome 段）。
// Rust 用阶段 A 管线产出具体 block id（grass/sand/stone/dirt 等），对比 vanilla。
use std::collections::HashMap;
use std::sync::Arc;

use WorldgenRust::blocks::{BlockColumn, BlockRegistry};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::surface_rules::SurfaceBuilder;
use WorldgenRust::terrain::{fill_chunk, VanillaDensity, VanillaAquifer, BiomeSource};
use WorldgenRust::biome::BiomeClassifier;
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
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let biomesrc = MacroBiome { bc, tempf: t_temp, humf: t_hum, contf: t_cont, erof: t_ero, depthf: t_dep, weirdf: t_wei };
    let sb = SurfaceBuilder::new(db.noise_samplers(), db.random_deriver(), 63, &blocks);
    let rule = sb.build_overworld_rule();

    let stone = blocks.id("minecraft:stone");
    let air = blocks.id("minecraft:air");
    let water = blocks.id("minecraft:water");
    let lava_id = blocks.id("minecraft:lava");

    // 读 vanilla .blocks（badlands 区，-8248 种子 8x8 origin 2688,-3072）
    let path = "E:\\python\\MC\\data\\vanilla_-8248318472910187742_8_2688_-3072.blocks";
    let bd = fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = 16*16*height as usize;
    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
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
        // 真实 estimateSurfaceHeight 4 角：从顶向下扫描 initial_density > 0.390625（间隔 8）
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

        // 对比
        for k in 0..bpc {
            let lx = (k % 16) as i32; let ly = (k / 256) as i32; let lz = ((k / 16) % 16) as i32;
            let y = min_y + ly;
            let got = col.at(lx, y, lz);
            total += 1;
            if vanilla[k] != 0 { tnair += 1; }
            if got == vanilla[k] { match_t += 1; if vanilla[k] != 0 { mnair += 1; } }
        }
    }
    println!("Rust(surface rules) vs vanilla: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
}
