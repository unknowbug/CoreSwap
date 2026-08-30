// transpiler_ch0_decompose.rs — 判决性分解实验：确定 transpiler ch0 与运行时 ch0 在 cell 内部点谁是「精确值」谁是「量化值」。
// 已知事实（transpiler_exactpoint_verify）：
//   y=64, 内部点(3,3): transpiler ch0=0.133102, runtime ch0=0.068001, corner(0,0) 两者都=0.072304
//   diff 场在 x/z 平滑、corner 处为 0 → 疑似一方在格点做了量化/插值。
// 方法：取 y=64 平面 4 个 corner (x∈{0,4}, z∈{0,4}) 各自的 ch0（分别用 transpiler / runtime 精确采样），
//   对内部点 (3,3)（fx=fz=0.75）做双线性插值，与两边精确点值对比：
//   - 若 runtime(3,3) == bilinear(runtime corners) → 运行时在内部点被某节点量化/插值（FlatCache 格点取值）
//   - 若 transpiler(3,3) == bilinear(transpiler corners) → transpiler 在内部点被同样机制影响
//   - 若都不等 → 两边精确点都不是插值，另有根因
// 另附：同一 (x,z)=(3,3) 列，y 从 -64..320 密扫，定位 diff 的 y 分布（是否真只有 y=64）。
use std::sync::Arc;
use WorldgenRust::density::{NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::noise::NoiseSet;
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
    let mut rnd = db.random_deriver().split_str("minecraft:terrain");
    let amp_l = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
    let lower = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let upper = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
    let amp_i = WorldgenRust::noise::OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
    let interp = WorldgenRust::noise::OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.125, 80.0, 160.0, 8.0);
    noises.set_blended_noise(bn);

    let (channels, _combine) = macrolize_channels(&tree);
    let cx: i32 = -288; let cz: i32 = -256;
    let ch0 = &channels[0];

    let transp_ch0 = |x: i32, y: i32, z: i32| -> f64 {
        let mut out = vec![0.0f64; 5];
        fill_cell_corner_densities_final_density(&noises, x as f64, y as f64, z as f64, &mut out);
        out[0]
    };

    // ===== 实验 1：y=64 平面，4 corner 双线性 vs 精确点 =====
    let y = 64i32;
    let c00_r = ch0.sample(&NoisePos { x: cx*16+0, y, z: cz*16+0 });
    let c10_r = ch0.sample(&NoisePos { x: cx*16+4, y, z: cz*16+0 });
    let c01_r = ch0.sample(&NoisePos { x: cx*16+0, y, z: cz*16+4 });
    let c11_r = ch0.sample(&NoisePos { x: cx*16+4, y, z: cz*16+4 });
    let c00_t = transp_ch0(cx*16+0, y, cz*16+0);
    let c10_t = transp_ch0(cx*16+4, y, cz*16+0);
    let c01_t = transp_ch0(cx*16+0, y, cz*16+4);
    let c11_t = transp_ch0(cx*16+4, y, cz*16+4);
    let fx = 0.75f64; let fz = 0.75f64;
    let bil = |c00: f64, c10: f64, c01: f64, c11: f64| -> f64 {
        (c00*(1.0-fx)+c10*fx)*(1.0-fz) + (c01*(1.0-fx)+c11*fx)*fz
    };
    let exact_t = transp_ch0(cx*16+3, y, cz*16+3);
    let exact_r = ch0.sample(&NoisePos { x: cx*16+3, y, z: cz*16+3 });
    println!("=== 实验1: y=64, 内部点(3,3) fx=fz=0.75 ===");
    println!("transpiler corners: (0,0)={:.6} (4,0)={:.6} (0,4)={:.6} (4,4)={:.6}", c00_t, c10_t, c01_t, c11_t);
    println!("runtime   corners: (0,0)={:.6} (4,0)={:.6} (0,4)={:.6} (4,4)={:.6}", c00_r, c10_r, c01_r, c11_r);
    println!("4-corner 互相 diff (t vs r): {:.6} {:.6} {:.6} {:.6}", (c00_t-c00_r).abs(), (c10_t-c10_r).abs(), (c01_t-c01_r).abs(), (c11_t-c11_r).abs());
    println!("内部点(3,3): transpiler精确={:.6} runtime精确={:.6} diff={:.6}", exact_t, exact_r, (exact_t-exact_r).abs());
    println!("  transpiler bilinear(corner) = {:.6}", bil(c00_t, c10_t, c01_t, c11_t));
    println!("  runtime   bilinear(corner) = {:.6}", bil(c00_r, c10_r, c01_r, c11_r));
    println!("  → runtime精确 vs runtime bilinear diff = {:.6}", (exact_r - bil(c00_r, c10_r, c01_r, c11_r)).abs());
    println!("  → transpiler精确 vs transpiler bilinear diff = {:.6}", (exact_t - bil(c00_t, c10_t, c01_t, c11_t)).abs());

    // ===== 实验 2：runtime(x=3,z=3) 是否等于 runtime(x=0,z=0)（FlatCache 量化假设）=====
    let r00 = ch0.sample(&NoisePos { x: cx*16+0, y, z: cz*16+0 });
    let r03 = ch0.sample(&NoisePos { x: cx*16+3, y, z: cz*16+3 });
    let r01 = ch0.sample(&NoisePos { x: cx*16+1, y, z: cz*16+1 });
    println!("\n=== 实验2: runtime 量化检查 y=64 ===");
    println!("runtime(0,0)={:.6} runtime(1,1)={:.6} runtime(3,3)={:.6}", r00, r01, r03);
    println!("  若 (1,1)==(3,3)==(0,0) → runtime 被 FlatCache 量化到 corner");
    // transpiler 同点
    println!("transpiler(0,0)={:.6} transpiler(1,1)={:.6} transpiler(3,3)={:.6}",
        transp_ch0(cx*16, y, cz*16), transp_ch0(cx*16+1, y, cz*16+1), transp_ch0(cx*16+3, y, cz*16+3));

    // ===== 实验 3：y 密扫（-64..320 step 4），固定 x=3,z=3，diff 的 y 分布 =====
    println!("\n=== 实验3: 固定(3,3), y 密扫 diff 分布 ===");
    let mut diff_ys: Vec<(i32, f64)> = Vec::new();
    let mut max_diff_all = 0.0f64; let mut max_y = 0i32;
    for yy in (-64..320).step_by(4) {
        let d = (transp_ch0(cx*16+3, yy, cz*16+3) - ch0.sample(&NoisePos { x: cx*16+3, y: yy, z: cz*16+3 })).abs();
        if d > 1e-9 { diff_ys.push((yy, d)); }
        if d > max_diff_all { max_diff_all = d; max_y = yy; }
    }
    println!("diff>1e-9 的 y 值数={}，max={:.6} at y={}", diff_ys.len(), max_diff_all, max_y);
    for (i, (yy, d)) in diff_ys.iter().enumerate().take(30) {
        print!("y={}->d={:.6}  ", yy, d);
        if (i+1) % 5 == 0 { println!(); }
    }
    println!();

    // ===== 实验 4：固定 y=64，x 从 -6..6（跨 chunk 边界 in-chunk 视角），看 diff 在 cell 边缘的行为 =====
    println!("\n=== 实验4: y=64, z=3, x 跨 cell/边界 diff 扫描 (x=-6..6) ===");
    for dx in -6..7i32 {
        let wx = cx*16+dx;
        let d = (transp_ch0(wx, y, cz*16+3) - ch0.sample(&NoisePos { x: wx, y, z: cz*16+3 })).abs();
        println!("  x(local {})={} d={:.6}", dx, wx, d);
    }
}