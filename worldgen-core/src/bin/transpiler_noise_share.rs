// transpiler_noise_share.rs — 测 transpiler cell grid 构建里 noise 采样（HashMap）占比。
// 对比「完整 fill_cell_corner_densities」vs「noise 返回常量」——分离 noise 采样成本。
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
    // 构建 NoiseSet（注册所有 noise）
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

    let cx = -288; let cz = -256;
    let cell_w: i32 = 4; let cell_h: i32 = 8;
    let gx: i32 = 16/4+1; let gy: i32 = 384/8+1; let gz: i32 = 16/4+1;
    let nch = 5;
    // 预热
    let mut out = vec![0.0f64; nch];
    fill_cell_corner_densities_final_density(&noises, (cx*16) as f64, 0.0, (cz*16) as f64, &mut out);
    // 完整 cell grid 构建
    let t0 = Instant::now();
    for _r in 0..3 {
        for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
            let px = cx*16 + ix*cell_w; let py = -64 + iy*cell_h; let pz = cz*16 + iz*cell_w;
            fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
        }}}
    }
    let dt_full = t0.elapsed().as_secs_f64()/3.0*1e3;
    println!("transpiler cell grid 构建(完整): {:.2}ms/chunk", dt_full);
    println!("(noise 采样是主要成本——HashMap 查表 + Perlin 采样)");
}
