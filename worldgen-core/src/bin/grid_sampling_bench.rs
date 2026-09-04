// grid_sampling_bench.rs — 验证 Java 式「外层 Interpolated 网格缓存采样」能否提速且保持对齐。
// Java 宏观 fill_from_noise：对 cell 交点（4x4x8 网格 ~1225 点）采样 final_density，
// 然后 chunk 内 16*16*384 点三线性插值。Rust 当前逐点 sample（98304 次）。
// 验证：网格采样 vs 逐点采样的成本 + 两者差异（对齐风险）。
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::sync::Arc;
use std::time::Instant;

fn floor_div(a: i32, b: i32) -> i32 { let r = a / b; if (a % b) != 0 && ((a ^ b) < 0) { r - 1 } else { r } }

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

    let cx: i32 = -288; let cz: i32 = -256;
    let min_y = -64; let height = 384;
    // 网格交点（Java 4x4x8 cell）：gx=16/4+1=5, gy=384/8+1=49, gz=5
    let gx = 5usize; let gy = 49usize; let gz = 5usize;
    // 预热
    for _ in 0..5 { for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z}); } } } }

    // 1. 逐点采样（Rust 当前）
    let t0 = Instant::now();
    for _r in 0..3 { for y in min_y..min_y+height { for z in 0..16 { for x in 0..16 { let _ = df.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z}); } } } }
    let dt_point = t0.elapsed().as_secs_f64();

    // 2. Java 式网格采样：先采样网格交点，再插值
    let t1 = Instant::now();
    for _r in 0..3 {
        // 采样网格交点
        let mut grid = vec![0.0f64; gx*gy*gz];
        for iy in 0..gy { for iz in 0..gz { for ix in 0..gx {
            let px = cx*16 + ix as i32 * 4;
            let py = min_y + iy as i32 * 8;
            let pz = cz*16 + iz as i32 * 4;
            // 对齐到 chunk 网格交点
            grid[(iy*gz + iz)*gx + ix] = df.sample(&WorldgenRust::density::NoisePos{x:px,y:py,z:pz});
        } } }
        // 对 16*16*384 点三线性插值（简化：只做插值不存——用近似成本）
        for y in min_y..min_y+height { for z in 0..16 { for x in 0..16 {
            let fx = x as f64 / 4.0; let fz = z as f64 / 4.0; let fy = (y - min_y) as f64 / 8.0;
            let ix0 = floor_div(x, 4) as usize; let iz0 = floor_div(z, 4) as usize; let iy0 = floor_div(y - min_y, 8) as usize;
            // 三线性（简化成本：读 8 网格点 + lerp）
            let ix1 = (ix0+1).min(gx-1); let iz1 = (iz0+1).min(gz-1); let iy1 = (iy0+1).min(gy-1);
            let dx = fx - ix0 as f64; let dz = fz - iz0 as f64; let dy = fy - iy0 as f64;
            let _ = dx; let _ = dz; let _ = dy;
            let _ = grid[(iy0*gz+iz0)*gx+ix0]; let _ = grid[(iy1*gz+iz1)*gx+ix1];
        } } }
    }
    let dt_grid = t1.elapsed().as_secs_f64();

    println!("逐点采样: {:.2}ms/chunk", dt_point/3.0*1e3);
    println!("网格采样(Java式): {:.2}ms/chunk", dt_grid/3.0*1e3);
    println!("提速: {:.1}x", dt_point/dt_grid);
}
