// transpiler_prod_density.rs — 对比 TranspilerDensity vs DensityMacroSampler 的 density 值分布。
// 验证 transpiler 接入生产后 density 是否系统性偏低（导致 nonAir 暴跌）。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::terrain::{DensitySource, ChunkDensitySampler, DensityMacroSampler, TranspilerDensity};

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

    let ms = DensityMacroSampler::new(&tree, min_y, height);
    let td = TranspilerDensity::new(noises, min_y, height);

    let cx = -288; let cz = -256;
    let ms_slices = ms.sample_chunk(cx, cz, min_y, height).unwrap();
    let td_slices = td.sample_chunk(cx, cz, min_y, height).unwrap();
    // 对比 density 值分布（均值/正负比例）
    let mut sum_ms = 0.0f64; let mut sum_td = 0.0f64; let mut pos_ms = 0u64; let mut pos_td = 0u64;
    let mut n = 0u64; let mut max_diff = 0.0f64;
    for y in -64..320 {
        for z in 0..16 { for x in 0..16 {
            let pos = NoisePos { x: cx*16+x, y, z: cz*16+z };
            let a = td_slices.sample(&pos);
            let b = ms_slices.sample(&pos);
            sum_td += a; sum_ms += b;
            if a > 0.0 { pos_td += 1; }
            if b > 0.0 { pos_ms += 1; }
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; }
            n += 1;
        }}
    }
    println!("density 分布 (n={}):", n);
    println!("  DensityMacroSampler: 均值={:.4} 正(d>0)={}/{} ({:.2}%)", sum_ms/n as f64, pos_ms, n, 100.0*pos_ms as f64/n as f64);
    println!("  TranspilerDensity:    均值={:.4} 正(d>0)={}/{} ({:.2}%)", sum_td/n as f64, pos_td, n, 100.0*pos_td as f64/n as f64);
    println!("  max_diff={:.6}", max_diff);
}
