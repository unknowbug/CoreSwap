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

    // 构建 NoiseSet（注册所有 noise）——noise_params 表 key 已带 minecraft: 前缀，seed 派生用完整 id（对齐 get_noise_sampler）
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    // 创建 old_blended_noise（InterpolatedNoiseData，对齐 density_builder L339-352）
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    // 对比多个点
    let cx = -288; let cz = -256;
    // 先测单个 noise（jagged）transpiler vs 运行时
    let n_jagged = noises.sample_noise("minecraft:jagged", (cx*16+4) as f64 * 1500.0, 0.0, (cz*16+4) as f64 * 1500.0);
    let rnd_jagged = db.get_noise_sampler("minecraft:jagged");
    let r_jagged = rnd_jagged.sample((cx*16+4) as f64 * 1500.0, 0.0, (cz*16+4) as f64 * 1500.0);
    println!("jagged noise: transpiler={:.6} runtime={:.6} diff={:.6}", n_jagged, r_jagged, (n_jagged-r_jagged).abs());
    let mut max_diff = 0.0f64; let mut n = 0; let mut max_pt = (0i32, 0i32, 0i32);
    for y in [-64i32, 0, 64, 128, 200, 300] {
        for z in [4i32, 8, 12] { for x in [4i32, 8, 12] {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = tree.sample(&NoisePos{x:wx,y,z:wz});
            let b = compute_final_density(&noises, wx as f64, y as f64, wz as f64);
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; max_pt = (wx, y, wz); }
            if d > 0.01 { println!("  diff={:.4} at ({},{},{}) transpiler={:.4} runtime={:.4}", d, wx, y, wz, b, a); }
            n += 1;
        }}
    }
    println!("compute_final_density vs 运行时 final_density: max_diff={:.6} at {:?} (n={})", max_diff, max_pt, n);
}
