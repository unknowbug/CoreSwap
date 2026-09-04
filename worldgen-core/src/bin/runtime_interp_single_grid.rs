// runtime_interp_single_grid.rs — 测运行时 Interpolated 单次 grid 构建（采样 inner 在 corners）耗时。
// 对比 transpiler fill_cell_corner_densities 单次调用（7μs）。
use std::time::Instant;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

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
    let tree = db.build_node(router.get("final_density").unwrap()).ok().unwrap();

    // 测单次 sample（首次触发 Interpolated grid 构建）
    let pos = NoisePos { x: -288*16+4, y: 0, z: -256*16+4 };
    let _ = tree.sample(&pos); // 预热（建 grid）
    // 换 chunk 触发新 grid 构建
    let pos2 = NoisePos { x: -287*16+4, y: 0, z: -256*16+4 };
    let n = 100_000usize;
    let t0 = Instant::now();
    for i in 0..n {
        let p = NoisePos { x: -287*16 + (i as i32 % 16), y: 0, z: -256*16 + (i as i32 % 16) };
        let _ = tree.sample(&p);
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("运行时 final_density.sample 单次(热, 缓存命中): {:.1}ns", dt/n as f64*1e9);
    println!("(transpiler fill_cell_corner_densities 单次 7μs——对比)");
}
