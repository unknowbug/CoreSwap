// qaq1_initdensity_cost.rs — Q-AQ1：initial_density 单次树采样成本（260903-10）
// 用 surf probe 实测 7342 samples/chunk × 单次成本 → 估计 get_fluid_level 冷缓存贡献。
// 口径：seed 8576294172403134396 / region (200,200)，initial_density_without_jaggedness。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = 8576294172403134396;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
    let df_dir2 = df_dir.clone();
    db.set_df_ns("overworld");
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df_dir2, name)).unwrap()
    }));
    let settings = parse(&std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let init: Arc<DensityFunction> = Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).ok().unwrap());

    // 模拟 estimate_surface_height 采样模式：自顶向下每 8 y 一采，region (200,200) 附近列
    let warmup: usize = 2000;
    for i in 0..warmup {
        let x = 3200 + (i % 16) as i32; let z = 3200 + (i / 16) as i32;
        let _ = init.sample(&WorldgenRust::density::NoisePos { x, y: 64 - ((i as i32 % 40) * 8), z });
    }
    let n = 7342 * 10; // 10 chunk 当量
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for i in 0..n {
        let x = 3200 + (i % 16) as i32; let z = 3200 + ((i / 16) % 16) as i32;
        let y = 320 - ((i as i32 % 40) * 8);
        acc += init.sample(&WorldgenRust::density::NoisePos { x, y, z });
    }
    let el = t0.elapsed().as_secs_f64() * 1e3;
    println!("initial_density: {} samples in {:.2}ms → {:.4}us/sample (acc={:.3})", n, el, el / n as f64 * 1e3, acc);
    println!("→ 7342 samples/chunk ≈ {:.2}ms/chunk", el / n as f64 * 7342.0);
}
