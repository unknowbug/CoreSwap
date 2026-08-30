// transpiler_alignment_expanded.rs — 扩大对齐样本：覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端。
// 目的：judge 建议项 7——原 n=54 测试点全在 cell 边界平面（x/z∈{4,8,12}，即 cell corner 4 的倍数），
// 未覆盖 cell 内部任意点 / chunk 边界（x/z=0,15）/ 负 Y 极端。本探针把「transpiler 核心正确」从局部
// 充分提升为更全局证明。
//
// 对比方式（对齐 transpiler_alignment.rs）：
//   a = tree.sample(&NoisePos)  —— 运行时解释 final_density（权威）
//   b = compute_final_density(每点精确 channel) —— transpiler 生成代码（fill_cell_corner_densities 每点调用）
// 覆盖：
//   1. cell 内部任意点：x/z ∈ {1,2,3,5,6,7,9,10,11,13,14,15}（非 4 的倍数，cell 内部）
//   2. chunk 边界 clamp：x/z = 0 和 15（chunk 边缘）
//   3. 负 Y 极端：y ∈ {-64,-63,-62,-61,-60,-59,-58,-57,-56}（含 min_y=-64 及以下）
// 另加生产路径对比（td_slices vs ms_slices）在 cell 内部点，验证插值路径。
use std::sync::Arc;
use WorldgenRust::density::NoisePos;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
use WorldgenRust::terrain::{DensitySource, DensityMacroSampler, TranspilerDensity};

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

    // 构建 NoiseSet（注册所有 noise）——noise_params 表 key 已带 minecraft: 前缀，seed 派生用完整 id
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    // 必须设 blended_noise（old_blended_noise）：漏设则 sample_blended_noise 返回 0.0 → ch0 系统性偏差
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    let cx: i32 = -288; let cz: i32 = -256;
    let nch: usize = 5;

    // ============ 1. 精确点核心对比（transpiler compute_final_density vs 运行时 tree.sample）============
    // 覆盖：cell 内部任意点 + chunk 边界 + 负 Y 极端
    // x/z 全 0..16（含 cell 内部 1,2,3,5,6,7,9,10,11,13,14,15 + chunk 边界 0,15 + cell corner 4,8,12）
    let xz_all: [i32; 16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
    // y 覆盖：负 Y 极端（-64..-56）+ 常规（0,64,128,200,300）
    let ys: [i32; 14] = [-64,-63,-62,-61,-60,-59,-58,-57,-56,0,64,128,200,300];
    let mut max_diff = 0.0f64; let mut n = 0u64; let mut diff_gt_1e9 = 0u64; let mut max_pt = (0i32, 0i32, 0i32);
    for &y in &ys {
        for &z in &xz_all { for &x in &xz_all {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = tree.sample(&NoisePos{x:wx,y,z:wz});
            let mut interp = vec![0.0f64; nch];
            WorldgenRust::generated_density::fill_cell_corner_densities_final_density(&noises, wx as f64, y as f64, wz as f64, &mut interp);
            let b = WorldgenRust::generated_density::compute_final_density(&noises, &interp, wx as f64, y as f64, wz as f64);
            let d = (a-b).abs();
            if d > max_diff { max_diff = d; max_pt = (wx, y, wz); }
            if d > 1e-9 { diff_gt_1e9 += 1; }
            n += 1;
        }}
    }
    println!("[精确点核心对比] transpiler compute_final_density vs 运行时 tree.sample:");
    println!("  n={} (x/z 全 0..16 × y 14 值含负 Y 极端 -64..-56)", n);
    println!("  max_diff={:.6} at {:?}", max_diff, max_pt);
    println!("  diff>1e-9 点数={}/{}", diff_gt_1e9, n);

    // ============ 2. 生产路径对比（td_slices vs ms_slices）在 cell 内部点 ============
    // 验证插值路径在 cell 内部（非 corner）也一致
    let ms = DensityMacroSampler::new(&tree, min_y, height);
    let td = TranspilerDensity::new(noises, min_y, height);
    let ms_slices = ms.sample_chunk(cx, cz, min_y, height).unwrap();
    let td_slices = td.sample_chunk(cx, cz, min_y, height).unwrap();
    // cell 内部点：x/z ∈ {1,2,3,5,6,7,9,10,11,13,14,15}（非 4 倍数）+ chunk 边界 0,15
    let interior: [i32; 12] = [1,2,3,5,6,7,9,10,11,13,14,15];
    let boundary: [i32; 2] = [0,15];
    let mut max_diff_p = 0.0f64; let mut n_p = 0u64; let mut diff_gt_1e9_p = 0u64; let mut max_pt_p = (0i32,0i32,0i32);
    // 负 Y 极端 + 常规 y
    let ys_p: [i32; 10] = [-64,-63,-62,-61,-60,-59,-58,-57,0,64];
    for &y in &ys_p {
        for &z in &interior { for &x in &interior {
            let pos = NoisePos { x: cx*16+x, y, z: cz*16+z };
            let d = (td_slices.sample(&pos) - ms_slices.sample(&pos)).abs();
            if d > max_diff_p { max_diff_p = d; max_pt_p = (pos.x, y, pos.z); }
            if d > 1e-9 { diff_gt_1e9_p += 1; }
            n_p += 1;
        }}
        for &z in &boundary { for &x in &boundary {
            let pos = NoisePos { x: cx*16+x, y, z: cz*16+z };
            let d = (td_slices.sample(&pos) - ms_slices.sample(&pos)).abs();
            if d > max_diff_p { max_diff_p = d; max_pt_p = (pos.x, y, pos.z); }
            if d > 1e-9 { diff_gt_1e9_p += 1; }
            n_p += 1;
        }}
    }
    println!("[生产路径对比] td_slices vs ms_slices（cell 内部 + chunk 边界 + 负 Y）:");
    println!("  n={} (interior 12×12 + boundary 2×2 × y 10 值)", n_p);
    println!("  max_diff={:.6} at {:?}", max_diff_p, max_pt_p);
    println!("  diff>1e-9 点数={}/{}", diff_gt_1e9_p, n_p);
}
