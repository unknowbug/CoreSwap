// perf_quant.rs — 量化 Rust sample 真实性能：finalDensity 纯树 vs 完整 fill_chunk。
// 目的：确认 DFC 直排是否值得（Rust enum-match 递归是否真慢）。
// 测：① finalDensity 纯树逐点 ② fill_chunk 完整管线（含 aquifer+biome+surface）逐点。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
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

    let dense = VanillaDensity { df: &tree };
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let biomesrc = MacroBiome { bc, tempf: t_temp, humf: t_hum, contf: t_cont, erof: t_ero, depthf: t_dep, weirdf: t_wei };

    // ① finalDensity 纯树逐点（单 chunk 逐列，缓存命中）
    let (cx, cz) = (0i32, 0i32);
    // 预热
    for _ in 0..100 { let _ = tree.sample(&NoisePos{x:0,y:0,z:0}); }
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for lz in 0..16 { for lx in 0..16 {
        let x = cx*16+lx; let z = cz*16+lz;
        for ly in (0..384).rev() { acc += tree.sample(&NoisePos{x,y:-64+ly,z}); }
    }}
    let t_tree = t0.elapsed().as_secs_f64()*1e6/98304.0;
    std::hint::black_box(acc);

    // ② fill_chunk 完整管线（含 aquifer+biome+surface）
    let n_chunks = 8usize;
    for c in 0..2 {
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), 0, 0, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let _ = fill_chunk(&dense, &mut va, &biomesrc, 0, 0, -64, 384i32, None, 384);
    }
    let t1 = Instant::now();
    for c in 0..n_chunks {
        let cx = 0 + (c % 4) as i32; let cz = 0 + (c / 4) as i32;
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &biomesrc, cx, cz, -64, 384i32, None, 384);
        std::hint::black_box(&cd);
    }
    let t_fill = t1.elapsed().as_secs_f64()*1e6/(n_chunks as f64*98304.0);
    println!("perf_quant (seed {}):", seed);
    println!("  finalDensity 纯树逐点: {:.2} us/pt", t_tree);
    println!("  fill_chunk 完整管线:   {:.2} us/pt", t_fill);
    println!("  管线开销 (fill/tree):  {:.1}x", t_fill/t_tree);
    println!("  => 若 tree 已快(<<1us)，DFC 直排收益有限；若 fill 远慢于 tree，瓶颈在 aquifer/biome/surface 非 density");
}
