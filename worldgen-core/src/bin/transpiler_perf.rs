// transpiler_perf.rs — 验证 build-time 编译（compute_final_density）vs 运行时解释（final_density.sample）性能。
// 对比大量点采样的吞吐。
use std::time::Instant;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::generated_density::{compute_final_density, fill_cell_corner_densities_final_density};

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

    // 构建 NoiseSet（注册所有 noise）
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    // old_blended_noise
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    // 采样点（模拟 fill_chunk 逐点）
    let cx = -288; let cz = -256;
    let n = 16*16*384; // 一个 chunk 的点数
    let mut xs = vec![0i32; n]; let mut ys = vec![0i32; n]; let mut zs = vec![0i32; n];
    let mut k = 0;
    for y in -64..320 { for z in 0..16 { for x in 0..16 { xs[k]=cx*16+x; ys[k]=y; zs[k]=cz*16+z; k+=1; } } }

    // 预热
    for i in 0..1000 { let _ = tree.sample(&NoisePos{x:xs[i],y:ys[i],z:zs[i]}); }

    // 运行时解释（final_density.sample）
    let t0 = Instant::now();
    for _r in 0..3 { for i in 0..n { let _ = tree.sample(&NoisePos{x:xs[i],y:ys[i],z:zs[i]}); } }
    let dt_runtime = t0.elapsed().as_secs_f64()/3.0*1e3;

    // transpiler（cell grid 插值：fill_cell_corner_densities 在 cell corners 采样一次 + 块级插值 + compute）
    let nch = 5;
    let cell_w: i32 = 4; let cell_h: i32 = 8;
    let gx: i32 = 16/4+1; let gy: i32 = 384/8+1; let gz: i32 = 16/4+1;
    let t1 = Instant::now();
    for _r in 0..3 {
        // 构建 cell grid（cell corners 采样 channel inner）
        let mut grid = vec![0.0f64; (gx*gy*gz) as usize * nch];
        for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
            let px = cx*16 + ix*cell_w; let py = -64 + iy*cell_h; let pz = cz*16 + iz*cell_w;
            let mut out = vec![0.0f64; nch];
            fill_cell_corner_densities_final_density(&noises, px as f64, py as f64, pz as f64, &mut out);
            for ch in 0..nch { grid[((iy*gz+iz)*gx+ix) as usize*nch+ch] = out[ch]; }
        }}}
        // 块级插值 + compute
        for i in 0..n {
            let x = xs[i] - cx*16; let z = zs[i] - cz*16; let y = ys[i] - (-64);
            let mut cxx = x / cell_w; let mut cyy = y / cell_h; let mut czz = z / cell_w;
            cxx = cxx.clamp(0, gx-2); cyy = cyy.clamp(0, gy-2); czz = czz.clamp(0, gz-2);
            let fx = (x % cell_w) as f64 / cell_w as f64;
            let fy = (y % cell_h) as f64 / cell_h as f64;
            let fz = (z % cell_w) as f64 / cell_w as f64;
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
            let _ = compute_final_density(&noises, &interp, xs[i] as f64, ys[i] as f64, zs[i] as f64);
        }
    }
    let dt_transpiler = t1.elapsed().as_secs_f64()/3.0*1e3;

    println!("运行时解释 final_density.sample: {:.2}ms/chunk", dt_runtime);
    println!("transpiler compute_final_density(cell grid 插值): {:.2}ms/chunk", dt_transpiler);
    println!("transpiler 提速: {:.2}x", dt_runtime / dt_transpiler);
}
