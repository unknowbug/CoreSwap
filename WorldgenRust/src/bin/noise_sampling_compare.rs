// noise_sampling_compare.rs — 对比 transpiler NoiseSet.sample_noise（数组）vs 运行时直接 noise.sample 吞吐。
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::noise::NoiseSet;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    // 构建 NoiseSet
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    // 运行时直接 noise（jagged）
    let rnd_jagged = db.get_noise_sampler("minecraft:jagged");

    let n = 1_000_000usize;
    // 预热
    for i in 0..10000 { let _ = noises.sample_noise("minecraft:jagged", i as f64*0.001, 0.0, i as f64*0.001); let _ = rnd_jagged.sample(i as f64*0.001, 0.0, i as f64*0.001); }
    // transpiler NoiseSet
    let t0 = Instant::now();
    let mut acc = 0.0;
    for i in 0..n { acc += noises.sample_noise("minecraft:jagged", i as f64*0.001, 0.0, i as f64*0.001); }
    let dt_set = t0.elapsed().as_secs_f64();
    // 运行时直接
    let t1 = Instant::now();
    let mut acc2 = 0.0;
    for i in 0..n { acc2 += rnd_jagged.sample(i as f64*0.001, 0.0, i as f64*0.001); }
    let dt_direct = t1.elapsed().as_secs_f64();
    println!("NoiseSet.sample_noise(数组): {:.1}ns/次", dt_set/n as f64*1e9);
    println!("运行时直接 noise.sample: {:.1}ns/次", dt_direct/n as f64*1e9);
    println!("差异: {:.2}x", dt_set/dt_direct);
}
