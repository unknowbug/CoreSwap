// transpiler_fill_cost.rs — 测 transpiler fill_cell_corner_densities 单次调用（采样 5 channels）成本。
// 对比运行时 Interpolated 单次 grid 构建（如果可测）。
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::fill_cell_corner_densities_final_density;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
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

    let nch = 5;
    let mut out = vec![0.0f64; nch];
    // 预热
    fill_cell_corner_densities_final_density(&noises, -288.0*16.0, 0.0, -256.0*16.0, &mut out);
    // 测单次调用（采样 5 channels）
    let n = 100_000usize;
    let t0 = Instant::now();
    for i in 0..n {
        fill_cell_corner_densities_final_density(&noises, (i as f64)*0.001, 0.0, (i as f64)*0.001, &mut out);
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("fill_cell_corner_densities 单次调用(5 channels): {:.1}ns", dt/n as f64*1e9);
    println!("(cell grid 构建 1225 corners × 单次 = 每 chunk 成本)");
}
