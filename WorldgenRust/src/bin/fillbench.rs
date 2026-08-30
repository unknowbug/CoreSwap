// fillbench.rs — fill_chunk 端到端性能基准（无探针污染：纯 wall + 计数）。
// 测：每 chunk 墙钟时间（含 density 树遍历 + aquifer + biome），N chunk 总耗时。
// 这是 Spline 直排优化前的基线。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos, GRID_ARG_SAMPLES};
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

    // 基准：16 chunks (4x4)，spawn 区
    let n_chunks = 16usize;
    // 预热 2 chunk（缓存/页）
    for c in 0..2 {
        let cx = 0 + (c % 2); let cz = 0 + c / 2;
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let _ = fill_chunk(&dense, &mut va, &biomesrc, cx, cz, -64, 384i32, None, 384);
    }
    let s0 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let t0 = Instant::now();
    for c in 0..n_chunks {
        let cx = 0 + (c % 4) as i32; let cz = 0 + (c / 4) as i32;
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384i32);
        let mut va = VanillaAquifer::new(aq);
        let cd = fill_chunk(&dense, &mut va, &biomesrc, cx, cz, -64, 384i32, None, 384);
        std::hint::black_box(&cd);
    }
    let wall = t0.elapsed();
    let s1 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let per = wall.as_secs_f64() * 1000.0 / n_chunks as f64;
    println!("fill_chunk bench: {} chunks, wall={:.1}ms, per-chunk={:.2}ms", n_chunks, wall.as_secs_f64()*1000.0, per);
    println!("interpolated grid arg samples: {} (delta {})", s1, s1 - s0);
    // 粗分解：surface 扫描(每列 ~384 次采样) vs blocks 全量(98k 采样)
    println!("理论: surface 扫描 ≈ 256 列 × ~平均160次 = ~41k 采样; blocks 全量 = {} 采样/chunk", 16*16*384i32);
}

