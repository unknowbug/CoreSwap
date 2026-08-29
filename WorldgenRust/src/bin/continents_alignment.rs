// continents_alignment.rs — 验证 transpiler 核心（noise/spline，无 interpolated）对齐。
// continents.json 是纯 noise + spline（无 interpolated）——若对齐，确认 transpiler 核心正确。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::compute_continents;

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
    // 构建 continents 树（运行时）
    let cont_json = parse(&std::fs::read_to_string(format!("{}/continents.json", df_dir)).unwrap()).unwrap();
    let tree = db.build_node(&cont_json).ok().unwrap();

    // 构建 NoiseSet（注册所有 noise）
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }

    // 对比多个点
    let cx = -288; let cz = -256;
    let mut max_diff = 0.0f64; let mut n = 0;
    for y in [-64i32, 0, 64, 128, 200, 300] {
        for z in [4i32, 8, 12] { for x in [4i32, 8, 12] {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = tree.sample(&NoisePos{x:wx,y,z:wz});
            let b = compute_continents(&noises, &[], wx as f64, y as f64, wz as f64);
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; }
            n += 1;
        }}
    }
    println!("compute_continents vs 运行时 continents: max_diff={:.6} (n={})", max_diff, n);
    println!("(continents 无 interpolated——若对齐，确认 transpiler 核心 noise/spline 正确)");
}
