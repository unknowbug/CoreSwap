// dfc_probe.rs — DFC 分层定位探针：interp 层 vs 噪声层（260903-03）。
// 对照基准 = f64 DF 树（density_builder build_node(final_density)，与 macro 同源）。
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::dfc_backend::DfcBackend;
use std::sync::Arc;

const SEED: i64 = -8248318472910187742;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

fn collect_interps<'a>(df: &'a DensityFunction, out: &mut Vec<&'a DensityFunction>) {
    use DensityFunction as D;
    if let D::Interpolated(_) = df {
        if !out.iter().any(|x| std::ptr::eq(*x, df)) { out.push(df); }
    }
    match df {
        D::BinaryOp { a, b, .. } => { collect_interps(a, out); collect_interps(b, out); }
        D::LinearOp { input, .. } | D::UnaryOp { input, .. } | D::Clamp { input, .. }
        | D::BlendDensity { input } | D::Wrapping { input } => collect_interps(input, out),
        D::Interpolated(i) => collect_interps(&i.arg, out),
        D::RangeChoice { input, in_range, out_of_range, .. } => {
            collect_interps(input, out); collect_interps(in_range, out); collect_interps(out_of_range, out);
        }
        D::ShiftedNoise { shift_x, shift_y, shift_z, .. } => {
            collect_interps(shift_x, out); collect_interps(shift_y, out); collect_interps(shift_z, out);
        }
        D::WeirdScaled { input, .. } => { collect_interps(input, out); }
        _ => {}
    }
}

fn main() {
    // f64 树
    let mut db = DensityBuilder::new(SEED as u64, -64, 384);
    db.set_df_ns("overworld");
    let df_dir = format!("{}\\data\\minecraft\\worldgen\\density_function\\overworld", WG_DIR);
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = format!("{}\\{}.json", df_dir, name);
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}: {}", p, e))
    }));
    let settings = WorldgenRust::json::parse(&std::fs::read_to_string(format!("{}\\data\\minecraft\\worldgen\\noise_settings\\overworld.json", WG_DIR)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let fd_json = router.get("final_density").unwrap();
    let tree = db.build_node(fd_json).unwrap();
    let _ = Arc::new(());

    let be = DfcBackend::new(SEED as u64);

    // 找树里的 Interpolated 节点
    let mut interps = Vec::new();
    collect_interps(&tree, &mut interps);
    println!("[tree] interpolated nodes = {}", interps.len());

    // 单点对照：树 interp 值 vs DFC interp_n(k)
    let pts: Vec<(i32, i32, i32)> = vec![(0, 0, 0), (64, -32, 64), (-100, 70, -55), (4, -64, 4), (-16, 100, 20)];
    for &(x, y, z) in &pts {
        be.dbg_split_full(x, y, z);
        println!("--- point ({},{},{})", x, y, z);
        for (k, node) in interps.iter().enumerate() {
            if k >= 5 { break; }
            let pos = WorldgenRust::density::NoisePos { x, y, z };
            let tv = node.sample(&pos);
            let dv = be.dbg_interp_n(k, x, y, z);
            println!("  interp[{}] tree={:.6} dfc={:.6} diff={:.6}", k, tv, dv, (tv - dv as f64).abs());
        }
        let tv = tree.sample(&WorldgenRust::density::NoisePos { x, y, z });
        let dv = be.sample_point(x, y, z);
        println!("  FINAL tree={:.6} dfc={:.6} diff={:.6}", tv, dv, (tv - dv as f64).abs());


        // 决定性单层：interp[k] 的 delegate（f64 树子树）在格点角点上的值 vs Rust eval_df_base
        // 格点角点 = 该点所在 cell 的 8 角点（与 buildInterpGrid 同坐标）
        let chunk_x = x.div_euclid(16); let chunk_z = z.div_euclid(16);
        let gx = x - chunk_x * 16; let gy = y - (-64); let gz = z - chunk_z * 16;
        let cx = gx / 4; let cy = gy / 8; let cz = gz / 4;
        for c in 0..8i32 {
            let dx = c & 1; let dy = (c >> 1) & 1; let dz = (c >> 2) & 1;
            let ax = chunk_x * 16 + (cx + dx) * 4;
            let ay = -64 + (cy + dy) * 8;
            let az = chunk_z * 16 + (cz + dz) * 4;
            be.dbg_split_full(ax, ay, az);
            let pos = WorldgenRust::density::NoisePos { x: ax, y: ay, z: az };
            if let DensityFunction::Interpolated(idata) = interps[0] {
                let tv0 = idata.arg.sample(&pos);
                let dv0 = be.dbg_eval_base(0, ax, ay, az);
                println!("    corner c={} node=({},{},{}) tree_arg={:.6} dfc_base={:.6} diff={:.6}",
                    c, ax, ay, az, tv0, dv0, (tv0 - dv0 as f64).abs());
            }
        }
    }

    // 微观性能：单 chunk 98304 点 sample_point + grid 重建计数
    use std::time::Instant;
    let (cx, cz) = (-288i32, -256i32);
    // 预热（含 grid 构建）
    for lx in 0..4 { let _ = be.sample_point(cx * 16 + lx, 0, cz * 16); }
    let builds_before = WorldgenRust::dfc_backend::GRID_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for lx in 0..16i32 { for lz in 0..16i32 { for ly in 0..384 {
        acc += be.sample_point(cx * 16 + lx, -64 + ly, cz * 16 + lz) as f64;
    } } }
    let dt = t0.elapsed().as_secs_f64() * 1e3;
    let builds_after = WorldgenRust::dfc_backend::GRID_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    println!("[perf-micro] 98304 sample_point: {:.1} ms ({:.3} us/pt) grid_builds_delta={} acc={:.3}",
        dt, dt * 1000.0 / 98304.0, builds_after - builds_before, acc);
}
