// perf_probe3.rs — 定位 grid 构建里 arg 采样的 348μs/点到底哪个叶子贵。
// 方法：对每个关键 DF/叶子在大量**变化点位**（防 const-fold）采样，测真实每点成本。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn build_builder(seed: u64) -> DensityBuilder {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";
    let mut db = DensityBuilder::new(seed, -64, 384);
    db.load_noise_params_file(noise_params_path).unwrap();
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));
    db
}

fn bench_varied(d: &Arc<DensityFunction>, iters: usize) -> f64 {
    // 用变化点位（一个 chunk 内 16x16 列 × y step4 扫描，避免 const-fold）
    let (cx, cz) = (45, -26);
    let mut idx = 0usize;
    for _ in 0..10 { let _ = d.sample(&NoisePos { x: cx*16+(idx%16) as i32, y: -64 + ((idx/16)%96) as i32 * 4, z: cz*16 + ((idx/(16*96))%16) as i32 }); idx += 1; }
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for _ in 0..iters {
        let x = cx*16 + (idx%16) as i32; let y = -64 + ((idx/16)%96) as i32 * 4; let z = cz*16 + ((idx/(16*96))%16) as i32;
        acc += d.sample(&NoisePos { x, y, z }); idx += 1;
    }
    let _ = acc;
    t0.elapsed().as_secs_f64() * 1e6 / iters as f64
}

fn main() {
    let seed = 8576294172403134396u64;
    let mut db = build_builder(seed);
    let dfs = ["base_3d_noise", "factor", "sloped_cheese", "caves/entrances", "caves/noodle", "depth", "jaggedness"];
    for name in dfs {
        let d = db.resolve_ref(&format!("minecraft:overworld/{}", name));
        let us = bench_varied(&d, 3000);
        println!("  {:<24} {:.1} us/pt", name, us);
    }
}
