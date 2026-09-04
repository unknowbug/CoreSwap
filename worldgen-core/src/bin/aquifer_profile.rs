// aquifer_profile.rs — 测 classify 内 part 的成本（barrier.sample vs get_block_pos vs 其他）。
// 用 Aquifer 公开方法不好访问，改用直接测 barrier/erosion/depth density 树采样成本。
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::sync::Arc;
use std::time::Instant;

fn bench_sample(df: &Arc<DensityFunction>, label: &str) {
    // 采样 1 chunk (16*16*384 点)，预热后计时
    let cx = -288; let cz = -256;
    for _ in 0..5 {
        for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z}); } } }
    }
    let t0 = Instant::now();
    for _r in 0..3 {
        for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z}); } } }
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("{:24} {:.2}ms/chunk", label, dt/3.0*1e3);
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
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
    for key in ["final_density", "barrier", "fluid_level_floodedness", "fluid_level_spread", "fluid_level_floodedness", "erosion", "depth", "initial_density_without_jaggedness"] {
        if let Some(v) = router.get(key) {
            if let Ok(df) = db.build_node(v) {
                bench_sample(&Arc::new(df), key);
            }
        }
    }
}
