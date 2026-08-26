// perf_probe2.rs — 正确基线 profiler：区分 Interpolated grid 构建成本 vs 稳态每点采样成本。
// 方法：fresh fill 一整 chunk（grid 构建 + 采样） vs cached fill 同 chunk（grid 已缓存，纯采样） => 二者之差 = grid 构建成本。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn build_tree(seed: u64) -> Arc<DensityFunction> {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";
    let mut db = DensityBuilder::new(seed, -64, 384);
    db.load_noise_params_file(noise_params_path).unwrap();
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));
    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    Arc::new(db.build_node(fd).unwrap())
}

fn fill_chunk(tree: &DensityFunction, cx: i32, cz: i32) -> f64 {
    let mut sum = 0.0f64;
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16 + bx, cz*16 + bz);
        for y in (-64..320).step_by(4) {     // 4-step: 96 y/列 × 256 列 = 24576 点/chunk
            sum += tree.sample(&NoisePos { x, y, z });
        }
    }}
    sum
}

fn main() {
    let seed = 8576294172403134396u64;
    let tree = build_tree(seed);
    let (cx, cz) = (45, -26);
    const N: usize = 16 * 16 * 96; // 24576 点/chunk
    // fresh fill（首次：构建该 chunk 的 interpolated grid + 采样）
    let t0 = Instant::now();
    let _ = fill_chunk(&tree, cx, cz);
    let fresh = t0.elapsed().as_secs_f64() * 1000.0;
    // cached fill（grid 已缓存：纯采样）
    let t1 = Instant::now();
    let _ = fill_chunk(&tree, cx, cz);
    let cached = t1.elapsed().as_secs_f64() * 1000.0;
    println!("chunk({},{}) points={}", cx, cz, N);
    println!("fresh fill = {:.1} ms   cached fill (pure sampling) = {:.1} ms", fresh, cached);
    println!("=> grid build cost = {:.1} ms   steady-state per-sample = {:.3} us/point",
             fresh - cached, cached * 1000.0 / N as f64);
}
