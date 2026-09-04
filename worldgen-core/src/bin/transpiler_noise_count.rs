// transpiler_noise_count.rs — 测 transpiler fill_cell_corner_densities 单次调用（5 channels）的 noise 采样次数。
// 用 NoiseSet 计数器（诊断，非热路径）。
use std::sync::atomic::{AtomicUsize, Ordering};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::fill_cell_corner_densities_final_density;

static NOISE_COUNT: AtomicUsize = AtomicUsize::new(0);

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

    // 单次 fill 调用（5 channels）
    let mut out = vec![0.0f64; 5];
    NOISE_COUNT.store(0, Ordering::Relaxed);
    fill_cell_corner_densities_final_density(&noises, -288.0*16.0, 0.0, -256.0*16.0, &mut out);
    let n = NOISE_COUNT.load(Ordering::Relaxed);
    println!("fill_cell_corner_densities 单次调用 noise 采样次数: {}", n);
    println!("(5 channels 完整树——若 > 运行时 Interpolated inner 采样，transpiler 重复采样)");
}
