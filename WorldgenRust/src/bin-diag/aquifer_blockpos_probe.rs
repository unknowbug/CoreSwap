// aquifer_blockpos_probe.rs — 无污染测 3×3 邻域 get_block_pos 循环成本（aquifer 内部最大嫌疑）。
// 用 Aquifer::new 构建，diag_blockpos_cost 测纯 3×3 邻域（不做 fluid/calculate_density）。
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::sync::Arc;

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
    // 测 3x3 邻域成本
    let dt = aq.diag_blockpos_cost(cx, cz, 3);
    let per_chunk = dt/3.0*1e3;
    println!("3x3 邻域 get_block_pos: {:.2}ms/chunk", per_chunk);
    // 测 get_fluid_level 成本
    let _ = aq.diag_fluidlevel_cost(cx, cz, 1);
    let dtf = aq.diag_fluidlevel_cost(cx, cz, 3);
    println!("get_fluid_level: {:.2}ms/chunk", dtf/3.0*1e3);
    // 测 calculate_density fluid 逻辑成本
    let _ = aq.diag_caldensity_logic_cost(cx, cz, 1);
    let dtc = aq.diag_caldensity_logic_cost(cx, cz, 3);
    println!("calculate_density(fluid logic 1次/点): {:.2}ms/chunk", dtc/3.0*1e3);
    println!("(对比 aquifer 17.5ms 总)");
}
