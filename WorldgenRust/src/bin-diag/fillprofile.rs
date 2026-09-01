// fillprofile.rs — 分解 fill_chunk 耗时：density+aquifer vs biome 分类器（隔离热点）。
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

struct RealBiome { bc: BiomeClassifier, tempf: Arc<DensityFunction>, humf: Arc<DensityFunction>, contf: Arc<DensityFunction>, erof: Arc<DensityFunction>, depthf: Arc<DensityFunction>, weirdf: Arc<DensityFunction> }
impl BiomeSource for RealBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}
struct NoBiome;
impl BiomeSource for NoBiome { fn biome(&self, _pos: &NoisePos) -> String { "x".to_string() } }

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
    let tt = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let th = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let tc = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let te = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let td = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let tw = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();
    let dense = VanillaDensity { df: &tree };
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let real = RealBiome { bc, tempf: tt, humf: th, contf: tc, erof: te, depthf: td, weirdf: tw };
    let nob = NoBiome;
    let n_chunks = 8usize;
    // 预热
    for c in 0..2i32 {
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), c*16, 0, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let _ = fill_chunk(&dense, &mut va, &nob, c, 0, -64, 384i32, None, 384);
    }
    // A: no-biome（纯 density+aquifer）
    let t0 = Instant::now();
    for c in 0..n_chunks {
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), (c as i32)*16, 0, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &nob, c as i32, 0, -64, 384i32, None, 384);
        std::hint::black_box(&cd);
    }
    let t_no = t0.elapsed().as_secs_f64()*1000.0/n_chunks as f64;
    // B: real biome
    let t1 = Instant::now();
    for c in 0..n_chunks {
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), (c as i32)*16, 0, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &real, c as i32, 0, -64, 384i32, None, 384);
        std::hint::black_box(&cd);
    }
    let t_real = t1.elapsed().as_secs_f64()*1000.0/n_chunks as f64;
    println!("fill per-chunk: density+aquifer only = {:.1}ms | +biome(linear scan) = {:.1}ms | biome cost = {:.1}ms ({:.0}%)",
        t_no, t_real, t_real - t_no, 100.0*(t_real-t_no)/t_real);
}
