// density_interp_bench.rs — 验证给 finalDensity 包 Interpolated 网格缓存的性能收益。
// 对比「裸 finalDensity 逐点采样」vs「包 Interpolated(4x4x8 网格) 采样」。
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
    let raw: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).ok().unwrap());
    // 包 Interpolated
    let interp = Arc::new(DensityFunction::Interpolated(WorldgenRust::density::InterpolatedData::new(raw.clone(), -64, 384)));

    // 采样 1 chunk (16*16*384 点)，模拟 fill_chunk density 部
    let chunk_x = -288; let chunk_z = -256;
    let mut vals_raw = Vec::with_capacity(16*16*384);
    let mut vals_int = Vec::with_capacity(16*16*384);

    // 预热
    for _ in 0..20 { for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = raw.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } } }

    // 测裸
    let t0 = Instant::now();
    for _r in 0..3 {
        for y in -64..320 { for z in 0..16 { for x in 0..16 { vals_raw.push(raw.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z})); } } }
    }
    let dt_raw = t0.elapsed().as_secs_f64();

    // 测 Interpolated（每 chunk 重建网格）
    let t1 = Instant::now();
    for _r in 0..3 {
        for y in -64..320 { for z in 0..16 { for x in 0..16 { vals_int.push(interp.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z})); } } }
    }
    let dt_int = t1.elapsed().as_secs_f64();

    let points = (16*16*384) as f64;
    println!("裸 finalDensity: {:.1}ms/chunk ({:.3}us/pt)", dt_raw/3.0*1e3, dt_raw/3.0/points*1e6);
    println!("包 Interpolated: {:.1}ms/chunk ({:.3}us/pt)", dt_int/3.0*1e3, dt_int/3.0/points*1e6);
    println!("加速: {:.1}x", dt_raw/dt_int);
    // 对齐检查（两者应一致或接近）
    let mut diff = 0; let mut n = 0;
    for i in 0..vals_raw.len().min(vals_int.len()) {
        let a = vals_raw[i]; let b = vals_int[i];
        let d = (a - b).abs();
        if d > 1e-9 { diff += 1; if n < 5 { println!("  diff@{i}: raw={} interp={} d={}", a, b, d); } n += 1; }
    }
    println!("采样值差异: {}/{} (Interpolated 是 4x4x8 网格插值近似)", diff, vals_raw.len());
}
