// transpiler_fill_cold.rs — 测 transpiler fill 单次（真正缓存冷：每 corner 不同 (x,z)）。
// 对比 transpiler_fill_noise_share（坐标循环 16 种，缓存命中 = 缓存热）——确认探针污染影响。
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
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
    // 真正缓存冷：每 corner 不同 (x,z)（用 i 直接作为坐标，避免循环命中）
    let n = 100_000usize;
    let t0 = Instant::now();
    for i in 0..n {
        let px = -288*16 + (i as i32 % 1000) * 4;   // 1000 种不同 x
        let py = -64 + (i as i32 % 49) * 8;
        let pz = -256*16 + (i as i32 / 1000 % 1000) * 4;  // 1000 种不同 z
        fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("transpiler fill 单次(真正缓存冷, 每 corner 不同 xz): {:.1}ns", dt/n as f64*1e9);
    println!("(对比 transpiler_fill_noise_share 坐标循环 16 种 = 缓存热)");
}
