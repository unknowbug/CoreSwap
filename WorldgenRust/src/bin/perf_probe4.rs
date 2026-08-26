// perf_probe4.rs — 诊断 grid 构建的 arg 采样总次数（确认嵌套 interpolated 递归网格构建）
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, GRID_ARG_SAMPLES, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

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

fn main() {
    let seed = 8576294172403134396u64;
    let tree = build_tree(seed);
    let (cx, cz) = (45, -26);
    // reset counter, fresh fill (grid builds)
    GRID_ARG_SAMPLES.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let mut sum = 0.0f64;
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16 + bx, cz*16 + bz);
        for y in (-64..320).step_by(4) { sum += tree.sample(&NoisePos { x, y, z }); }
    }}
    let el = t0.elapsed().as_secs_f64() * 1000.0;
    let arg_samples = GRID_ARG_SAMPLES.load(Ordering::Relaxed);
    let _ = sum;
    println!("fresh chunk fill: {:.1} ms", el);
    println!("build_grid arg.sample total calls = {}", arg_samples);
    println!("expected if only 1 interpolated node (5x49x5=1225 pts): ~{}", (5*49*5));
    println!("=> arg.sample calls / 1225 = {} => {} interpolated grid builds (nested?); avg per arg.sample = {:.2} us",
             (arg_samples as f64)/((5*49*5) as f64), arg_samples as f64/((5*49*5) as f64), el*1000.0/(arg_samples as f64));
}
