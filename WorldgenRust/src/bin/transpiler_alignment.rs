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

    // 对比多个点：构建 cell grid（fill_cell_corner_densities 采样 channel inner）+ 块级插值 + compute
    let cx: i32 = -288; let cz: i32 = -256;
    // 先测单个 noise（jagged）transpiler vs 运行时
    let n_jagged = noises.sample_noise("minecraft:jagged", (cx*16+4) as f64 * 1500.0, 0.0, (cz*16+4) as f64 * 1500.0);
    let rnd_jagged = db.get_noise_sampler("minecraft:jagged");
    let r_jagged = rnd_jagged.sample((cx*16+4) as f64 * 1500.0, 0.0, (cz*16+4) as f64 * 1500.0);
    println!("jagged noise: transpiler={:.6} runtime={:.6} diff={:.6}", n_jagged, r_jagged, (n_jagged-r_jagged).abs());
    // 构建 cell grid（4x4x8 cell corners 采样 channel inner）
    let cell_w: i32 = 4; let cell_h: i32 = 8;
    let gx: i32 = 16/4+1; let gy: i32 = 384/8+1; let gz: i32 = 16/4+1; // 5, 49, 5
    let nch: usize = 5; // final_density 5 channels
    let mut grid = vec![0.0f64; (gx*gy*gz) as usize * nch];
    for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
        let px = cx*16 + ix*cell_w; let py = -64 + iy*cell_h; let pz = cz*16 + iz*cell_w;
        let mut out = vec![0.0f64; nch];
        WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
        for ch in 0..nch { grid[((iy*gz+iz)*gx+ix) as usize*nch+ch] = out[ch]; }
    }}}
    // 块级插值 + compute
    let mut max_diff = 0.0f64; let mut n = 0; let mut max_pt = (0i32, 0i32, 0i32);
    for y in [-64i32, 0, 64, 128, 200, 300] {
        for z in [4i32, 8, 12] { for x in [4i32, 8, 12] {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = tree.sample(&NoisePos{x:wx,y,z:wz});
            // 块级三线性插值 channel
            let gxx = x; let gzz = z; let gyy = y - (-64);
            let mut cxx = gxx / cell_w; let mut cyy = gyy / cell_h; let mut czz = gzz / cell_w;
            cxx = cxx.clamp(0, gx-2); cyy = cyy.clamp(0, gy-2); czz = czz.clamp(0, gz-2);
            let fx = (gxx % cell_w) as f64 / cell_w as f64;
            let fy = (gyy % cell_h) as f64 / cell_h as f64;
            let fz = (gzz % cell_w) as f64 / cell_w as f64;
            let mut interp = vec![0.0f64; nch];
            for ch in 0..nch {
                let at = |dx: i32, dy: i32, dz: i32| grid[(((cyy+dy)*gz+(czz+dz))*gx+(cxx+dx)) as usize*nch+ch];
                let d000=at(0,0,0); let d100=at(1,0,0); let d010=at(0,1,0); let d110=at(1,1,0);
                let d001=at(0,0,1); let d101=at(1,0,1); let d011=at(0,1,1); let d111=at(1,1,1);
                let d00=d000+(d100-d000)*fx; let d10=d010+(d110-d010)*fx;
                let d01=d001+(d101-d001)*fx; let d11=d011+(d111-d011)*fx;
                let d0=d00+(d10-d00)*fy; let d1=d01+(d11-d01)*fy;
                interp[ch] = d0 + (d1 - d0)*fz;
            }
            let b = WorldgenRust::generated_density::compute_final_density(&noises, &interp, wx as f64, y as f64, wz as f64);
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; max_pt = (wx, y, wz); }
            n += 1;
        }}
    }
    println!("compute_final_density(竖切) vs 运行时 final_density: max_diff={:.6} at {:?} (n={})", max_diff, max_pt, n);
}
