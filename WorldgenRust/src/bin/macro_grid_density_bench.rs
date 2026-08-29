// macro_grid_density_bench.rs — 验证「Rust 逐 block density 采样」vs「Java cell 网格采样」的成本差异。
// 关键：cell 网格点（4x4x8，~1225 点，均在 chunk 内）采样 final_density，对比逐 block 98304 点。
// 判断宏观网格采样能否降 density 成本（为宏观网格优化提供数据）。
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::sync::Arc;
use std::time::Instant;

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
    let df: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).ok().unwrap());
    let min_y = -64; let height = 384;

    // 预热（让内部 interpolated 缓存热）
    for c in [-288i32, -287, -286] {
        for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:c*16+x,y,z:c*16+z}); } } }
    }

    // 逐 block 采样（1 chunk）
    let cx = -288; let cz = -256;
    let t0 = Instant::now();
    for _r in 0..5 { for y in min_y..min_y+height { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z}); } } } }
    let dt_block = t0.elapsed().as_secs_f64()/5.0*1e3;

    // cell 网格采样（4x4x8，gx=5/gy=49/gz=5=1225 点，均在 chunk 内）
    let gx = 5usize; let gy = 49usize; let gz = 5usize;
    let n_grid = gx*gy*gz;
    let t1 = Instant::now();
    for _r in 0..50 {
        for iy in 0..gy { for iz in 0..gz { for ix in 0..gx {
            let px = cx*16 + ix as i32 * 4;
            let py = min_y + iy as i32 * 8;
            let pz = cz*16 + iz as i32 * 4;
            let _ = df.sample(&WorldgenRust::density::NoisePos{x:px,y:py,z:pz});
        } } }
    }
    let dt_grid = t1.elapsed().as_secs_f64()/50.0*1e3;

    println!("逐 block density 采样 (98304 点/chunk): {:.2}ms/chunk", dt_block);
    println!("cell 网格 density 采样 ({} 点/chunk): {:.2}ms/chunk", n_grid, dt_grid);
    println!("网格 vs 逐 block 采样点数比: {:.0}x; 采样成本比: {:.1}x", 98304.0/n_grid as f64, dt_block/dt_grid);
}
