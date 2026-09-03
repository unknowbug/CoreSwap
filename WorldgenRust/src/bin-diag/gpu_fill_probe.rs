// gpu_fill_probe.rs — 路线② P2 验证（260903-05）：GpuDensity（FFI 逐块批量）vs DfcDensity（CPU 逐点）
// 整 chunk 98304 点对拍：major_diff(>1e-4) 计数（主判据，f32 口径）+ max_diff。
// 前置：PATH 含 versions/1.20.1/cpp/build-msvc/bin（gpu_ffi.dll 及其依赖）；WG_GPU_SPV 可选。
use WorldgenRust::terrain::{DfcDensity, DensitySource};

fn main() {
    let seed: i64 = std::env::var("GP_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(8576294172403134396);
    let cx: i32 = std::env::var("GP_CX").ok().and_then(|s| s.parse().ok()).unwrap_or(45);
    let cz: i32 = std::env::var("GP_CZ").ok().and_then(|s| s.parse().ok()).unwrap_or(-28);
    let wg_dir = "versions/1.20.1/data/worldgen";
    let min_y = -64;
    let noise_height = 384;
    println!("=== gpu_fill_probe (260903-05) seed={} chunk=({},{}) ===", seed, cx, cz);

    println!("[1] creating GpuDensity (FFI create ~75s 一次付)…");
    let t0 = std::time::Instant::now();
    let gd = match WorldgenRust::gpu_ffi::GpuDensity::new(seed, wg_dir, min_y) {
        Some(g) => g,
        None => { eprintln!("[FAIL] GpuDensity::new returned None (create failed / dll not found)"); std::process::exit(1); }
    };
    println!("[1] create ok in {:.1}s", t0.elapsed().as_secs_f32());

    println!("[2] GPU 批量整 chunk（sample_chunk，24×4096 批）…");
    let t1 = std::time::Instant::now();
    let slices = match gd.sample_chunk(cx, cz, min_y, noise_height) {
        Some(cd) => cd.slices().to_vec(),
        None => { eprintln!("[FAIL] sample_chunk returned None"); std::process::exit(1); }
    };
    let dt_gpu = t1.elapsed().as_secs_f32();
    println!("[2] gpu chunk fill: {:.1} ms ({} pts)", dt_gpu * 1000.0, slices.len());

    println!("[3] DFC-CPU 逐点对拍（98304 点）…");
    let dfc = DfcDensity::new(seed as u64);
    let t2 = std::time::Instant::now();
    let mut major = 0usize;
    let mut max_diff = 0.0f64;
    let mut worst: Option<(i32, i32, i32, f64, f64)> = None;
    for by in 0..noise_height {
        for lz in 0..16 { for lx in 0..16 {
            let pos = WorldgenRust::density::NoisePos { x: cx * 16 + lx, y: min_y + by, z: cz * 16 + lz };
            let a = slices[(lx + lz * 16 + by * 256) as usize];            let b = dfc.sample(&pos);
            let d = (a - b).abs();
            if d > 1e-4 { major += 1; }
            if d > max_diff { max_diff = d; worst = Some((pos.x, pos.y, pos.z, a, b)); }
        }}
    }
    let dt_cpu = t2.elapsed().as_secs_f32();
    println!("[3] dfc chunk sample: {:.1} ms", dt_cpu * 1000.0);
    println!("=== RESULT major_diff(>1e-4)={} / {}  max_diff={:.3e}", major, slices.len(), max_diff);
    if let Some((x, y, z, a, b)) = worst {
        println!("=== worst @({},{},{}) gpu={:.6} dfc={:.6}", x, y, z, a, b);
    }
    if major == 0 { println!("[PASS] GPU 批量 = CPU 逐点（f32 口径，major=0）"); }
    else { println!("[FAIL] major_diff>0 — 语义分叉，需排查"); std::process::exit(1); }
}
