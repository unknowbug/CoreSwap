// perf_probe.rs — 基线 profiler：定位 finalDensity 单点 237μs 花在哪。
// 方法：逐 DF 单点采样计时（100次取平均） + 单点 min_value/max_value 计时（spline 递归开销）+ 逐层减法。
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

fn bench(f: impl Fn() -> f64, iters: usize) -> f64 {
    // warmup
    for _ in 0..10 { let _ = f(); }
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for _ in 0..iters { acc += f(); }
    let el = t0.elapsed();
    let _ = acc;
    el.as_secs_f64() * 1e6 / iters as f64  // us/iter
}

fn main() {
    let seed = 8576294172403134396u64;
    let mut db = build_builder(seed);
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    let tree: DensityFunction = db.build_node(fd).unwrap();
    let tree = Arc::new(tree);

    let pos = NoisePos { x: 728, y: 48, z: -408 };
    // 单点采样计时（稳定点：grid 缓存命中后）
    let sample_us = bench(|| tree.sample(&pos), 10000);
    println!("finalDensity sample: {:.1} us/pt", sample_us);
    // min_value/max_value 单次（spline 递归全树开销）
    let min_us = bench(|| tree.min_value(), 2000);
    let max_us = bench(|| tree.max_value(), 2000);
    println!("finalDensity min_value: {:.1} us  max_value: {:.1} us", min_us, max_us);

    // 逐 DF 单点采样（定位哪类节点贵：spline/interpolated/noise）
    let dfs = ["base_3d_noise", "continents", "factor", "sloped_cheese", "caves/entrances", "caves/noodle"];
    for name in dfs {
        let d = db.resolve_ref(&format!("minecraft:overworld/{}", name));
        let us = bench(|| d.sample(&pos), 5000);
        // 若含 interpolated/cache，首次需 grid 构建；用 warmup 后的稳定值
        println!("  {:<28} sample: {:.1} us/pt", name, us);
    }
}
