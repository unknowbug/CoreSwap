// transpiler_slices_ch0.rs — 直接对比 td_slices vs ms_slices 的 ch0 值（slices 数组），理清 ch0b 0.15 残差与 prod_density 0.000000 的矛盾。
use std::sync::Arc;
use WorldgenRust::density::{NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::terrain::{DensitySource, DensityMacroSampler, TranspilerDensity};
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
    // ⚠️ 必须设 blended_noise（old_blended_noise）：漏设则 sample_blended_noise 返回 0.0 → ch0 系统性偏差
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    // 先采集 corner 的 transpiler ch0（需要 noises 引用），再构建 td（move noises）
    let mut out = vec![0.0f64; 5];
    let px0 = -4608; let py0 = 0; let pz0 = -4096;
    fill_cell_corner_densities_final_density(&noises, px0 as f64, py0 as f64, pz0 as f64, &mut out);

    let ms = DensityMacroSampler::new(&tree, min_y, height);
    let td = TranspilerDensity::new(noises, min_y, height);

    let cx = -288; let cz = -256;
    // 时序 A：先 ms 后 td
    let ms_slices = ms.sample_chunk(cx, cz, min_y, height).unwrap();
    let td_slices = td.sample_chunk(cx, cz, min_y, height).unwrap();
    let mut max_diff_a = 0.0f64; let mut diff_a = 0usize;
    for ix in 0..5i32 { for iz in 0..5i32 { for iy in 0..49i32 {
        let pos = NoisePos { x: cx*16+ix*4, y: -64+iy*8, z: cz*16+iz*4 };
        let d = (td_slices.sample(&pos) - ms_slices.sample(&pos)).abs();
        if d > max_diff_a { max_diff_a = d; }
        if d > 1e-9 { diff_a += 1; }
    }}}
    // 时序 B：再各重建一次（缓存已热）
    let ms_slices = ms.sample_chunk(cx, cz, min_y, height).unwrap();
    let td_slices = td.sample_chunk(cx, cz, min_y, height).unwrap();
    let mut max_diff_b = 0.0f64; let mut diff_b = 0usize;
    for ix in 0..5i32 { for iz in 0..5i32 { for iy in 0..49i32 {
        let pos = NoisePos { x: cx*16+ix*4, y: -64+iy*8, z: cz*16+iz*4 };
        let d = (td_slices.sample(&pos) - ms_slices.sample(&pos)).abs();
        if d > max_diff_b { max_diff_b = d; }
        if d > 1e-9 { diff_b += 1; }
    }}}
    println!("时序A（ms先,td后）: max_diff={:.6}, diff_pts={}/1225", max_diff_a, diff_a);
    println!("时序B（重建后）:   max_diff={:.6}, diff_pts={}/1225", max_diff_b, diff_b);
}