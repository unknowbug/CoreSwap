// transpiler_prod_ch0b.rs — 对比 transpiler out[0] vs 运行时 channels[0] 对同一 corner 的采样，分解 spline vs blended_noise。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::fill_cell_corner_densities_final_density;

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
    let tree = Arc::new(db.build_node(router.get("final_density").unwrap()).ok().unwrap());

    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    let (channels, _combine) = macrolize_channels(&tree);
    let mut out = vec![0.0f64; 5];
    let px = -288*16; let py = 0; let pz = -256*16;
    fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
    let pos = NoisePos { x: px, y: py, z: pz };
    println!("corner ({},{},{}):", px, py, pz);
    println!("  transpiler out[0] (ch0) = {:.6}", out[0]);
    println!("  runtime channels[0].sample = {:.6}", channels[0].sample(&pos));
    println!("  diff = {:.6}", (out[0] - channels[0].sample(&pos)).abs());
    // 分解：blended_noise 部分
    println!("  noises.sample_blended_noise = {:.6}", noises.sample_blended_noise(px as f64, py as f64, pz as f64));
    // 运行时 channels[0] 的 InterpolatedNoise 采样（用 sample_ctx 的 InterpolatedNoise 分支）
    // 直接对比 transpiler out[0] 的 spline 部分（out[0] - blended_noise）
    println!("  transpiler out[0] - blended_noise (spline 部分) = {:.6}", out[0] - noises.sample_blended_noise(px as f64, py as f64, pz as f64));
}
