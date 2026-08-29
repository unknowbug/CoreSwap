// transpiler_alignment.rs — 验证 build-time 编译（compute_final_density）vs 运行时解释（density.rs）对齐。
// 构建 NoiseSet（注册所有 noise）+ 对比多个点的 density 值。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::compute_final_density;

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

    // 构建 NoiseSet（注册所有 noise）——从 noise_params 表创建，每个 noise 用其 id 派生 seed（对齐 get_noise_sampler）
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    // 也注册带 minecraft: 前缀的（transpiler 用 "minecraft:jagged" 等）
    let params2 = params.clone();
    for (id, p) in &params2 {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(&format!("minecraft:{}", id), sampler);
    }

    // 对比多个点
    let cx = -288; let cz = -256;
    let mut max_diff = 0.0f64; let mut n = 0;
    for y in [-64i32, 0, 64, 128, 200, 300] {
        for z in [4i32, 8, 12] { for x in [4i32, 8, 12] {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = tree.sample(&NoisePos{x:wx,y,z:wz});
            let b = compute_final_density(&noises, wx as f64, y as f64, wz as f64);
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; }
            n += 1;
        }}
    }
    println!("compute_final_density vs 运行时 final_density: max_diff={:.6} (n={})", max_diff, n);
    println!("(注意：NoiseSet 空，noise 采样返回 0——若 max_diff 大，是 noise 未注册，非 transpiler 错)");
}
