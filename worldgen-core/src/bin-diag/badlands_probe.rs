// badlands_probe.rs — 阶段 B 验证：扫描找 badlands/desert biome 区域，验证 terracottaBands + badlands pillar。
// terracotta 红陶带在 badlands/desert 等 biome 的特定 y 带生成；badlands pillar 在 eroded_badlands。
// 验证：SurfaceBuilder 的红陶带 + place_badlands_pillar 在对应 biome 生效。
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

fn main() {
    let seed: i64 = -2032795982907864146;
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
                "minecraft:badlands_surface", "minecraft:badlands_pillar", "minecraft:badlands_pillar_roof",
                "minecraft:calcite", "minecraft:gravel", "minecraft:powder_snow", "minecraft:packed_ice",
                "minecraft:ice", "minecraft:surface_swamp"] {
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

    // 扫描 eroded_badlands 所在区域（eroded_diag 定位 chunk(-480,-1024) 附近）
    let mut terracotta_count = 0;
    let mut badlands_chunks = 0;
    let mut terracotta_examples: Vec<(i32, i32, i32, i32, String)> = Vec::new();
    for cz in -1030..-1010 { for cx in -490..-470 {
        // 先看 biome（快速判断是否 badlands/desert）
        let bp = NoisePos { x: (cx*16)>>2<<2, y: 0, z: (cz*16)>>2<<2 };
        let b = biomesrc.biome(&bp);
        let is_badlands = b.contains("badlands") || b.contains("desert");
        if !is_badlands { continue; }
        badlands_chunks += 1;

        let mut aq = WorldgenRust::aquifer::Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &biomesrc, cx, cz, -64, 384i32, None, 384);

        let mut col = BlockColumn::new(-64, 384);
        for lz in 0..16 { for lx in 0..16 { for ly in 0..384 {
            let y = -64 + ly;
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
            for y in (-64..320).rev().step_by(8) {
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
                         &biome_at, &|x,y,z| ((x as i64)<<32) ^ (z as i64), &biome_temp, -64, 384, &initial_density_at);

        // 统计 terracotta 块
        for lz in 0..16 { for lx in 0..16 { for ly in 0..384 {
            let y = -64 + ly;
            let id = col.at(lx, y, lz);
            let name = blocks.name(id);
            if name.contains("terracotta") {
                terracotta_count += 1;
                if terracotta_examples.len() < 5 {
                    terracotta_examples.push((cx, cz, lx, y, name.to_string()));
                }
            }
        }}}
        if badlands_chunks >= 8 { break; }
    }}
    println!("badlands_probe: found {} badlands/desert chunks, {} terracotta blocks", badlands_chunks, terracotta_count);
    for (cx, cz, lx, y, name) in &terracotta_examples {
        println!("  chunk({},{}) col({},?) y={} block={}", cx, cz, lx, y, name);
    }
    println!("badlands_probe done (terracotta bands should appear in badlands/desert y bands)");
}

