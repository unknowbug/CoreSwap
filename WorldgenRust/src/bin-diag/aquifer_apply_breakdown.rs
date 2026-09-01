// aquifer_apply_breakdown.rs — 无污染分解 Aquifer.apply 内部成本（用 diag 方法，不 instrument 热路径）。
// 目标：锁定 apply 每点 98304 次调用的固定开销真实构成（judge 建议优化前先锁定）。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::density::DensityFunction;
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
    let b = |db: &mut DensityBuilder, k: &str| -> Arc<DensityFunction> { Arc::new(db.build_node(router.get(k).unwrap()).ok().unwrap()) };
    let barrier = b(&mut db, "barrier"); let flooded = b(&mut db, "fluid_level_floodedness"); let spread = b(&mut db, "fluid_level_spread");
    let lava = b(&mut db, "lava"); let erosion = b(&mut db, "erosion"); let depth = b(&mut db, "depth"); let init = b(&mut db, "initial_density_without_jaggedness");
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();
    let cx = -288; let cz = -256;
    let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava, erosion, depth, init, splitter.clone(), cx*16, cz*16, -64, 384);

    // 预热
    let _ = aq.diag_blockpos_cost(cx, cz, 1);
    let _ = aq.diag_fluidlevel_cost(cx, cz, 1);
    let _ = aq.diag_caldensity_logic_cost(cx, cz, 1);

    // 测各部分（无污染 diag）
    let t_bp = aq.diag_blockpos_cost(cx, cz, 3)/3.0*1e3;
    let t_fl = aq.diag_fluidlevel_cost(cx, cz, 3)/3.0*1e3;
    let t_cd = aq.diag_caldensity_logic_cost(cx, cz, 3)/3.0*1e3;
    let t_wl = aq.diag_waterlevel_cost(cx, cz, 3)/3.0*1e3;

    // 测 apply 完整（模拟 fill_chunk 逐点，d<=0 才调 apply）
    // 用 density 采样决定 d，d<=0 时调 apply
    let tree = b(&mut db, "final_density");
    let t0 = Instant::now();
    for _r in 0..3 {
        for y in -64..320 { for z in 0..16 { for x in 0..16 {
            let d = tree.sample(&WorldgenRust::density::NoisePos{x:cx*16+x,y,z:cz*16+z});
            if d <= 0.0 { let _ = aq.apply(cx*16+x, y, cz*16+z, d); }
        }}}
    }
    let t_apply = t0.elapsed().as_secs_f64()/3.0*1e3;

    println!("Aquifer.apply 内部无污染分解（per chunk）:");
    println!("  3x3 邻域 get_block_pos+距离: {:.2}ms", t_bp);
    println!("  get_fluid_level(全调用上界): {:.2}ms", t_fl);
    println!("  calculate_density(fluid logic): {:.2}ms", t_cd);
    println!("  get_water_level_at(1次/点): {:.2}ms", t_wl);
    println!("  apply 完整(逐点 d<=0): {:.2}ms", t_apply);
    println!("  可解释部分: {:.2}ms; 剩余(apply 固定开销+分支+get_water_level_at 3次): {:.2}ms",
        t_bp+t_fl+t_cd+t_wl, t_apply - (t_bp+t_fl+t_cd+t_wl));
}
