// density_interp_single.rs — judge 对照：评估「单层 Interpolated」真实性能（消除双层污染）。
// 基准对比：
//   A) 纯 SplineDF 树（sloped_cheese，无任何 Interpolated/缓存包装）裸采样
//   B) 同树包一层 Interpolated
// 若 B << A：单层 Interpolated 方向有效，原始 bench 的 100x 是双层污染，非方向错误。
// 若 B >> A：Interpolated 实现本身上限差，方向确实错误。
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
    // 纯 sloped_cheese（SplineDF 主路径，无 Interpolated 包装）
    let sc = Arc::new(db.build_node(
        &parse(&std::fs::read_to_string(format!("{}/sloped_cheese.json", df_dir)).unwrap()).unwrap()).ok().unwrap());
    let sc_interp = Arc::new(DensityFunction::Interpolated(
        WorldgenRust::density::InterpolatedData::new(sc.clone(), -64, 384)));

    let chunk_x = -288; let chunk_z = -256;
    // 预热
    for _ in 0..10 { for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = sc.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } } }
    for _ in 0..10 { for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = sc_interp.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } } }

    let points = (16*16*384) as f64;
    // A) 裸 sloped_cheese（SplineDF 递归树）
    let t0 = Instant::now();
    for _r in 0..3 { for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = sc.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } } }
    let dt_sc = t0.elapsed().as_secs_f64();
    // B) 单层 Interpolated
    let t1 = Instant::now();
    for _r in 0..3 { for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = sc_interp.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } } }
    let dt_sci = t1.elapsed().as_secs_f64();

    println!("[A] 纯 SplineDF sloped_cheese 裸采样: {:.2}ms/chunk ({:.3}us/pt)", dt_sc/3.0*1e3, dt_sc/3.0/points*1e6);
    println!("[B] 单层 Interpolated(sloped_cheese): {:.2}ms/chunk ({:.3}us/pt)", dt_sci/3.0*1e3, dt_sci/3.0/points*1e6);
    println!("B/A 加速比: {:.2}x", dt_sc/dt_sci);

    // 对齐检查（单层 Interpolated vs 原树：网格近似必有误差，记录误差幅度）
    let mut diff = 0; let mut maxd = 0.0f64; let mut sum = 0.0f64;
    let mut va = Vec::new(); let mut vb = Vec::new();
    for y in -64..320 { for z in 0..16 { for x in 0..16 {
        let p = WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z};
        va.push(sc.sample(&p)); vb.push(sc_interp.sample(&p));
    }}}
    for i in 0..va.len() {
        let d = (va[i]-vb[i]).abs();
        if d > 1e-9 { diff += 1; if d>maxd{maxd=d;} sum += d; }
    }
    println!("单层 Interpolated 对齐: diff={}/{}, max_err={:.2e}, mean_err={:.2e}", diff, va.len(), maxd, sum/diff.max(1) as f64);
}
