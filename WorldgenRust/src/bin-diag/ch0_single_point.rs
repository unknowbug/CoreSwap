// ch0_single_point.rs — 260903-06 P-B 判别探针：单点调用 transpiler 生成函数（隔离评估顺序）。
// 判别 (4,80,16) out[0]：==-1.386253（探针列值）→ 生成结构性 y 压平；==-1.216371（macro/C++）→ 运行时列缓存污染。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::generated_density::fill_cell_corner_densities_final_density;
use WorldgenRust::noise::NoiseSet;

fn make_noises(db: &DensityBuilder) -> NoiseSet {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
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
    noises
}

fn main() {
    let seed: u64 = 8576294172403134396u64;
    let min_y = -64; let height = 384;
    let mut db = DensityBuilder::new(seed, min_y, height);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let noises = Arc::new(make_noises(&db));
    println!("=== ch0_single_point seed={}（build_slices 顺序复现）===", seed);
    let mode = std::env::var("CH0_CLEAR").is_ok();
    println!("mode = {}", if mode { "CLEAR-EACH-POINT" } else { "PLAIN (build_slices order: ix>iz>iy)" });
    // 完整复现 build_slices：ix 外层、iz、iy 内层；dump (x=4,z=16) 列
    let gx = 5usize; let gz = 5usize; let gy = 49usize;
    let mut col = Vec::new();
    for ix in 0..gx {
        for iz in 0..gz {
            for iy in 0..gy {
                let px = 0 + ix as i32 * 4;
                let py = min_y + iy as i32 * 8;
                let pz = 0 + iz as i32 * 4;
                if mode { WorldgenRust::density::transpiler_cache_clear_all(); }
                let mut out = [0.0f64; 5];
                fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
                if px == 4 && pz == 16 { col.push((py, out[0])); }
            }
        }
    }
    for (y, v) in &col { println!("y={:>4} out0={:.9}", y, v); }
}
