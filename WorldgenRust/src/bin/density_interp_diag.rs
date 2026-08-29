// density_interp_diag.rs — judge 诊断：定位 Interpolated 628ms 的根因。
// 假设：final_density 树内部已含 Interpolated 节点，探针 raw 外层再包一层 Interpolated
//       = 双层 Interpolated，可能因 grid 反复重建导致 100x 慢（而非方向错误）。
// 测量：GRID_ARG_SAMPLES（内层所有 Interpolated build_grid 累计采样次数）+ wall。
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

    // 内层网格采样计数（raw 树内部所有 Interpolated 的 build_grid 累计 arg.sample 次数）
    let g0 = WorldgenRust::density::GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let chunk_x = -288; let chunk_z = -256;

    // 预热原始树（触发内部所有 Interpolated 建 chunk 网格）
    for y in (-64..320).step_by(64) { for z in 0..16 { for x in 0..16 { let _ = raw.sample(&WorldgenRust::density::NoisePos{x:chunk_x*16+x,y,z:chunk_z*16+z}); } } }
    let g_pre = WorldgenRust::density::GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    println!("[prewarm] internal Interpolated grid_arg_samples delta = {}", g_pre - g0);

    // 测裸多 chunk（每 chunk 重新建内部网格）
    let t0 = Instant::now();
    for cy in 0..8 { for cz in 0..8 {
        let cx = -288 + cy; let czz = -256 + cz;
        for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = raw.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:czz*16+z}); } } }
    }}
    let dt_raw = t0.elapsed().as_secs_f64();
    let g_raw = WorldgenRust::density::GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    println!("[raw 64chunks] wall={:.1}ms  grid_arg_samples delta={} ({:.1}%/chunk)", dt_raw*1e3, g_raw-g_pre, (g_raw-g_pre) as f64/64.0);

    // 外层包 Interpolated（双层）
    let interp = Arc::new(DensityFunction::Interpolated(WorldgenRust::density::InterpolatedData::new(raw.clone(), -64, 384)));
    let g_i0 = WorldgenRust::density::GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let t1 = Instant::now();
    for cy in 0..8 { for cz in 0..8 {
        let cx = -288 + cy; let czz = -256 + cz;
        for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = interp.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:czz*16+z}); } } }
    }}
    let dt_int = t1.elapsed().as_secs_f64();
    let g_i1 = WorldgenRust::density::GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    println!("[interp 64chunks] wall={:.1}ms  grid_arg_samples delta={} ({:.1}/chunk)", dt_int*1e3, g_i1-g_i0, (g_i1-g_i0) as f64/64.0);
    println!("ratio raw/interp = {:.2}x", dt_raw/dt_int);

    // 结论指示：
    // - 若 interp 的 grid_arg_samples delta 远超 raw（每 chunk 数万次）→ 外层 Interpolated 反复重建 grid 触发内层重建 → 双层 bug 导致慢
    // - 若 interp 的 grid_arg_samples 与 raw 相近但 wall 仍 100x → Interpolated 三线性插值路径本身巨慢（方向问题）
}
