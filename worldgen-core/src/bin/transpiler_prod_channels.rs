// transpiler_prod_channels.rs — 对比 transpiler fill_cell_corner_densities vs macrolize_channels 的 channels 值。
// 验证 transpiler 接入生产后 channels 采样是否与 macro_sampler 一致（对齐）。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::fill_cell_corner_densities_final_density;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let min_y = -64; let height = 384;
    let mut db = DensityBuilder::new(seed as u64, min_y, height);
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

    // macrolize_channels 提取 channels（生产 DensityMacroSampler 用）
    let (channels, _combine) = macrolize_channels(&tree);
    println!("macrolize_channels channels 数: {}", channels.len());

    // 对比 transpiler fill_cell_corner_densities vs channels[ch].sample 的 channels 值
    let mut out = vec![0.0f64; 5];
    let px = -288*16; let py = min_y; let pz = -256*16;
    fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
    println!("cell corner ({},{},{}):", px, py, pz);
    for ch in 0..channels.len() {
        let ms_val = channels[ch].sample(&NoisePos { x: px, y: py, z: pz });
        println!("  ch{}: transpiler={:.6} channels[{}].sample={:.6} diff={:.6}", ch, out[ch], ch, ms_val, (out[ch]-ms_val).abs());
    }
}
