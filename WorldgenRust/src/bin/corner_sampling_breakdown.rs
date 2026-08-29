// corner_sampling_breakdown.rs — 分解 slices 构建（corners 采样）的成本：哪个 channel 贵、FlatCache 命中率。
// 关键：ch#0 BlendDensity(3677 节点) 是大头；测它的 corners 采样成本 + 是否可以 ColumnCache 加速。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
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
    let (channels, _combine) = macrolize_channels(&tree);

    let cx = -288; let cz = -256;
    let cell_w = 4; let cell_h = 8;
    let gx = 16/4+1; let gy = 384/8+1; let gz = 16/4+1; // 5, 49, 5

    // 预热 channels（FlatCache/内部缓存热）
    for ch in &channels { for y in [-64i32, 0, 100, 200, 300] { for z in [0i32,8,15] { for x in [0i32,3,15] {
        let _ = ch.sample(&NoisePos{x:cx*16+x,y,z:cz*16+z});
    }}}}
    // 测每个 channel 的 corners 采样（slices 构建成本分解）
    for (i, ch) in channels.iter().enumerate() {
        let t0 = Instant::now();
        for _r in 0..10 {
            for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
                let px = cx*16 + ix*cell_w; let py = -64 + iy*cell_h; let pz = cz*16 + iz*cell_w;
                let _ = ch.sample(&NoisePos{x:px,y:py,z:pz});
            }}}
        }
        let dt = t0.elapsed().as_secs_f64()/10.0*1e3;
        println!("ch#{} corners 采样: {:.2}ms/chunk (1225 corners)", i, dt);
    }

    // 对比：所有 channels 一起（slices 构建总）
    let t1 = Instant::now();
    for _r in 0..10 {
        for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
            let px = cx*16 + ix*cell_w; let py = -64 + iy*cell_h; let pz = cz*16 + iz*cell_w;
            for ch in &channels { let _ = ch.sample(&NoisePos{x:px,y:py,z:pz}); }
        }}}
    }
    let dt_all = t1.elapsed().as_secs_f64()/10.0*1e3;
    println!("所有 channels corners 总计: {:.2}ms/chunk", dt_all);
}
