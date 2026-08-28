// biome_perf.rs — 量化 biome_of 单点成本（确认优化空间）。
// fillprofile 显示 biome 占管线 46%。测 biome_of 每点耗时 + 每点扫描的 biome 数。
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

    // 测 biome_of 每点（含 6 参数采样 + 线性扫描）
    let iters = 100000usize;
    let t0 = Instant::now();
    let mut acc = 0usize;
    for i in 0..iters {
        let x = ((i as i32) * 7) & 0xFFFF;
        let z = ((i as i32) * 13) & 0xFFFF;
        let bp = NoisePos { x: x >> 2 << 2, y: 0, z: z >> 2 << 2 };
        let b = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp);
        acc += b.len();
    }
    let t_biome = t0.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc);

    println!("biome_perf (seed {}):", seed);
    println!("  biome_of 每点 (含6参数采样+线性扫描): {:.3} us/pt", t_biome);
    println!("  => 若 ~0.16us/pt（fillprofile 46%），优化空间在减少扫描 biome 数（SearchTree/预筛）");
}
