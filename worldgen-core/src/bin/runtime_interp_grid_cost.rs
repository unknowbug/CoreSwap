// runtime_interp_grid_cost.rs — 测运行时 final_density.sample 的 Interpolated grid 构建成本。
// 冷采样（首次，建 grid）vs 热采样（缓存命中）——分离 grid 构建成本。
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

    let cx = -288; let cz = -256;
    let n = 16*16*384;
    let mut xs = vec![0i32; n]; let mut ys = vec![0i32; n]; let mut zs = vec![0i32; n];
    let mut k = 0;
    for y in -64..320 { for z in 0..16 { for x in 0..16 { xs[k]=cx*16+x; ys[k]=y; zs[k]=cz*16+z; k+=1; } } }
    // 冷采样（首次，Interpolated 建 grid）
    let t0 = Instant::now();
    for i in 0..n { let _ = tree.sample(&NoisePos{x:xs[i],y:ys[i],z:zs[i]}); }
    let cold = t0.elapsed().as_secs_f64()*1e3;
    // 热采样（缓存命中）
    let t1 = Instant::now();
    for _r in 0..3 { for i in 0..n { let _ = tree.sample(&NoisePos{x:xs[i],y:ys[i],z:zs[i]}); } }
    let hot = t1.elapsed().as_secs_f64()/3.0*1e3;
    println!("运行时冷采样(含 Interpolated grid 构建): {:.2}ms/chunk", cold);
    println!("运行时热采样(缓存命中): {:.2}ms/chunk", hot);
    println!("Interpolated grid 构建贡献: {:.2}ms/chunk", cold - hot);
}
