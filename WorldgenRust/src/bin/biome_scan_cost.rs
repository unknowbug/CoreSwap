// biome_scan_cost.rs — 测 biome_of 的线性扫描成本（不含 6 参数采样）。
// biome_of 54us/pt vs 6 参数采样 0.94us/pt，差异应在线性扫描 64 biome。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

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
    let t: Arc<DensityFunction> = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let h: Arc<DensityFunction> = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let c: Arc<DensityFunction> = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let e: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let d: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let w: Arc<DensityFunction> = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");

    // 预热
    let bp = NoisePos { x: 0, y: 0, z: 0 };
    for _ in 0..100 { let _ = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp); }

    // ① 完整 biome_of（采样 + 扫描）
    let iters = 10000usize;
    let t0 = Instant::now();
    let mut acc = 0usize;
    for i in 0..iters {
        let lx = (i % 16) as i32; let lz = ((i / 16) % 16) as i32;
        let bp = NoisePos { x: lx >> 2 << 2, y: 0, z: lz >> 2 << 2 };
        acc += bc.biome_of(&t, &h, &c, &e, &d, &w, &bp).len();
    }
    let t_full = t0.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc);

    // ② 只采样 6 参数（不扫描）
    let t1 = Instant::now();
    let mut acc2 = 0.0f64;
    for i in 0..iters {
        let lx = (i % 16) as i32; let lz = ((i / 16) % 16) as i32;
        let bp = NoisePos { x: lx >> 2 << 2, y: 0, z: lz >> 2 << 2 };
        acc2 += t.sample(&bp) + h.sample(&bp) + c.sample(&bp) + e.sample(&bp) + d.sample(&bp) + w.sample(&bp);
    }
    let t_samp = t1.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc2);

    println!("biome_scan_cost (seed {}):", seed);
    println!("  biome_of 完整 (采样+扫描): {:.3} us/pt", t_full);
    println!("  只采样 6 参数: {:.3} us/pt", t_samp);
    println!("  扫描成本 (full - samp): {:.3} us/pt", t_full - t_samp);
    println!("  => 若扫描成本大，优化 = SearchTree/预筛（跳过全开维度）");
}
