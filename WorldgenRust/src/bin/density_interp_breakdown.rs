// density_interp_breakdown.rs — 分解 fill_chunk density 采样成本：grid 构建 vs 逐点插值+组合。
// 方法：对 final_density 采样单 chunk，分「冷采样(首次 build grid)」vs「预热后(cache hit 纯插值+组合)」，
// 差异 = interpolated grid 构建成本；cache-hit 部分 = 逐点插值 + 根组合成本。
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
    let cx = -288; let cz = -256;
    let (mut xs, mut ys, mut zs) = (vec![0i32; 16*16*384], vec![0i32; 16*16*384], vec![0i32; 16*16*384]);
    let mut k = 0;
    for y in -64..320 { for z in 0..16 { for x in 0..16 { xs[k]=cx*16+x; ys[k]=y; zs[k]=cz*16+z; k+=1; } } }

    // 冷采样（首次：interpolated build grid）
    println!("开始冷采样 chunk({},{})...", cx, cz);
    let mut cold_pts = 0;
    let t0 = Instant::now();
    for i in 0..xs.len() {
        let _ = df.sample(&WorldgenRust::density::NoisePos{x:xs[i],y:ys[i],z:zs[i]});
        cold_pts += 1;
        if cold_pts % 20000 == 0 { println!("  冷采样 {} 点, 累计 {:.0}ms", cold_pts, t0.elapsed().as_secs_f64()*1e3); }
    }
    let cold = t0.elapsed().as_secs_f64()*1e3;
    println!("冷采样完成: {:.2}ms", cold);

    // 再采样（cache hit：纯插值+组合）
    println!("开始再采样(cache hit)...");
    let t1 = Instant::now();
    for _r in 0..5 { for i in 0..xs.len() { let _ = df.sample(&WorldgenRust::density::NoisePos{x:xs[i],y:ys[i],z:zs[i]}); } }
    let hot = t1.elapsed().as_secs_f64()/5.0*1e3;
    println!("再采样完成: {:.2}ms", hot);

    println!("冷采样(含 build grid): {:.2}ms/chunk", cold);
    println!("预热后(cache hit 插值+组合): {:.2}ms/chunk", hot);
    println!("grid 构建贡献(冷-hot): {:.2}ms/chunk", cold - hot);
    println!("逐点插值+组合贡献(hot): {:.2}ms/chunk", hot);
}
