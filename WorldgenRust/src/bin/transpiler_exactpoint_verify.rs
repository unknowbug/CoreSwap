// transpiler_exactpoint_verify.rs — 验证 transpiler 精确点路径 vs 运行时精确点路径（排除插值语义差异）。
// 背景：transpiler_alignment_expanded 发现「精确点核心对比」在 cell 内部点 max_diff=0.044780（2522/3584 点 >1e-9），
// 但生产路径（td_slices vs ms_slices）在内部点 0.000000。怀疑 0.044780 是「运行时 tree.sample() 插值 vs
// transpiler 精确点直采」的语义差异（InterpolatedDF 在内部点插值，cell corner 处插值=精确值故 n=54 全 0）。
// 本探针用「运行时精确点」参考（macrolize_channels 的 channel inner 在精确点采样 + combine）对比 transpiler 精确点，
// 排除插值差异，判定 transpiler 核心在内部点是否真正确。
use std::sync::Arc;
use WorldgenRust::density::{NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;

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

    // 运行时精确点参考：macrolize_channels → channel inner 在精确点采样 + combine
    let (channels, combine) = macrolize_channels(&tree);
    let nch = channels.len();

    let cx: i32 = -288; let cz: i32 = -256;
    // 覆盖：cell 内部点（非 4 倍数）+ chunk 边界 + 负 Y 极端
    let xz_all: [i32; 16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
    let ys: [i32; 14] = [-64,-63,-62,-61,-60,-59,-58,-57,-56,0,64,128,200,300];

    // 对比 1：transpiler 精确点 vs 运行时精确点（应 0 若 transpiler 核心正确）
    let mut max_diff_tp = 0.0f64; let mut n_tp = 0u64; let mut diff_gt_1e9_tp = 0u64; let mut max_pt_tp = (0i32,0i32,0i32);
    // 对比 2：transpiler 精确点 vs 运行时 tree.sample()（插值）——预期内部点有差异（语义差异）
    let mut max_diff_ti = 0.0f64; let mut n_ti = 0u64; let mut diff_gt_1e9_ti = 0u64; let mut max_pt_ti = (0i32,0i32,0i32);
    for &y in &ys {
        for &z in &xz_all { for &x in &xz_all {
            let wx = cx*16+x; let wz = cz*16+z;
            let pos = NoisePos { x: wx, y, z: wz };
            // transpiler 精确点
            let mut interp_t = vec![0.0f64; 5];
            WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx as f64, y as f64, wz as f64, &mut interp_t);
            let b_t = WorldgenRust::generated_density::compute_final_density(&noises, &interp_t, wx as f64, y as f64, wz as f64);
            // 运行时精确点：channel inner 在精确点采样 + combine
            let mut interp_r = vec![0.0f64; nch];
            for ch in 0..nch {
                interp_r[ch] = channels[ch].sample(&pos);
            }
            let b_r = combine.sample_combine(&pos, &interp_r);
            // 运行时插值（tree.sample）
            let a = tree.sample(&pos);
            // 对比 1：transpiler vs 运行时精确点
            let d1 = (b_t - b_r).abs();
            if d1 > max_diff_tp { max_diff_tp = d1; max_pt_tp = (wx, y, wz); }
            if d1 > 1e-9 { diff_gt_1e9_tp += 1; }
            n_tp += 1;
            // 对比 2：transpiler vs 运行时插值
            let d2 = (b_t - a).abs();
            if d2 > max_diff_ti { max_diff_ti = d2; max_pt_ti = (wx, y, wz); }
            if d2 > 1e-9 { diff_gt_1e9_ti += 1; }
            n_ti += 1;
        }}
    }
    println!("[对比1] transpiler 精确点 vs 运行时精确点（channel inner 精确采样 + combine）:");
    println!("  n={} max_diff={:.6} at {:?} diff>1e-9={}/{}", n_tp, max_diff_tp, max_pt_tp, diff_gt_1e9_tp, n_tp);
    println!("[对比2] transpiler 精确点 vs 运行时 tree.sample()（InterpolatedDF 插值）:");
    println!("  n={} max_diff={:.6} at {:?} diff>1e-9={}/{}", n_ti, max_diff_ti, max_pt_ti, diff_gt_1e9_ti, n_ti);
    println!("  若对比1≈0 且对比2>0 → 0.044780 是插值语义差异（运行时插值 vs transpiler 直采），非 transpiler bug");
    println!("  若对比1>0 → transpiler 精确点核心在内部点有真实差异，需进一步定位");

    // 诊断：dump 对比1 中 diff>1e-9 的点，按 y 分组统计 + 打印前 20 个
    println!("\n[诊断] 对比1 diff>1e-9 的点按 y 分组:");
    use std::collections::BTreeMap;
    let mut by_y: BTreeMap<i32, (u64, f64)> = BTreeMap::new();
    let mut samples: Vec<(i32,i32,i32,f64,f64,f64)> = Vec::new();
    for &y in &ys {
        for &z in &xz_all { for &x in &xz_all {
            let wx = cx*16+x; let wz = cz*16+z;
            let pos = NoisePos { x: wx, y, z: wz };
            let mut interp_t = vec![0.0f64; 5];
            WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx as f64, y as f64, wz as f64, &mut interp_t);
            let b_t = WorldgenRust::generated_density::compute_final_density(&noises, &interp_t, wx as f64, y as f64, wz as f64);
            let mut interp_r = vec![0.0f64; nch];
            for ch in 0..nch { interp_r[ch] = channels[ch].sample(&pos); }
            let b_r = combine.sample_combine(&pos, &interp_r);
            let d1 = (b_t - b_r).abs();
            if d1 > 1e-9 {
                let e = by_y.entry(y).or_insert((0, 0.0));
                e.0 += 1; if d1 > e.1 { e.1 = d1; }
                if samples.len() < 20 { samples.push((wx, y, wz, b_t, b_r, d1)); }
            }
        }}
    }
    for (y, (cnt, mx)) in &by_y {
        println!("  y={}: diff>1e-9 点数={}, max_diff={:.6}", y, cnt, mx);
    }
    println!("  前 20 个 diff 点 (x,y,z, transpiler, runtime, diff):");
    for (x,y,z,bt,br,d) in &samples {
        println!("    ({},{},{}) t={:.6} r={:.6} d={:.6}", x, y, z, bt, br, d);
    }

    // 聚焦诊断：y=64 全 x/z 的 diff 分布（区分 cell corner vs interior）
    println!("\n[聚焦] y=64 全 x/z diff 分布（x/z 相对 chunk 内坐标 0..15）:");
    let y = 64i32;
    for &z in &xz_all {
        let mut row = String::new();
        for &x in &xz_all {
            let wx = cx*16+x; let wz = cz*16+z;
            let pos = NoisePos { x: wx, y, z: wz };
            let mut interp_t = vec![0.0f64; 5];
            WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx as f64, y as f64, wz as f64, &mut interp_t);
            let b_t = WorldgenRust::generated_density::compute_final_density(&noises, &interp_t, wx as f64, y as f64, wz as f64);
            let mut interp_r = vec![0.0f64; nch];
            for ch in 0..nch { interp_r[ch] = channels[ch].sample(&pos); }
            let b_r = combine.sample_combine(&pos, &interp_r);
            let d1 = (b_t - b_r).abs();
            let mark = if d1 > 1e-9 { format!("{:>6.4}", d1) } else { "  .   ".to_string() };
            row.push_str(&mark);
            row.push(' ');
        }
        println!("  z={:>2}: {}", z, row);
    }

    // 逐 channel 诊断：在 y=64 的一个 diff 点，对比 transpiler 与运行时每个 channel 的值
    println!("\n[逐 channel] y=64, x=3, z=3（cell 内部 diff 点）:");
    let y = 64i32; let x = 3i32; let z = 3i32;
    let wx = cx*16+x; let wz = cz*16+z;
    let pos = NoisePos { x: wx, y, z: wz };
    let mut interp_t = vec![0.0f64; 5];
    WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx as f64, y as f64, wz as f64, &mut interp_t);
    let mut interp_r = vec![0.0f64; nch];
    for ch in 0..nch { interp_r[ch] = channels[ch].sample(&pos); }
    for ch in 0..nch {
        println!("  ch{}: transpiler={:.6} runtime={:.6} diff={:.6}", ch, interp_t[ch], interp_r[ch], (interp_t[ch]-interp_r[ch]).abs());
    }
    let b_t = WorldgenRust::generated_density::compute_final_density(&noises, &interp_t, wx as f64, y as f64, wz as f64);
    let b_r = combine.sample_combine(&pos, &interp_r);
    println!("  final: transpiler={:.6} runtime={:.6} diff={:.6}", b_t, b_r, (b_t-b_r).abs());
    // 同点 cell corner 对照（x=0,z=0 应 0）
    let x0 = 0i32; let z0 = 0i32;
    let wx0 = cx*16+x0; let wz0 = cz*16+z0;
    let pos0 = NoisePos { x: wx0, y, z: wz0 };
    let mut interp_t0 = vec![0.0f64; 5];
    WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx0 as f64, y as f64, wz0 as f64, &mut interp_t0);
    let mut interp_r0 = vec![0.0f64; nch];
    for ch in 0..nch { interp_r0[ch] = channels[ch].sample(&pos0); }
    println!("  [cell corner x=0,z=0 对照]");
    for ch in 0..nch {
        println!("    ch{}: transpiler={:.6} runtime={:.6} diff={:.6}", ch, interp_t0[ch], interp_r0[ch], (interp_t0[ch]-interp_r0[ch]).abs());
    }
}
