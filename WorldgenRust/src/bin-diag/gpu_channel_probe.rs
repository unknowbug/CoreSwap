// gpu_channel_probe.rs — X2 验证（260903-05，judge B/C1/D1 判据）：
// ① 逐通道对拍（judge B：禁止只对拍 combine 后值——min 掩盖通道错位）：
//    GPU 角点 channels slices vs 同一 GpuChannelDensity 内 fallback（TranspilerDensity，已 diff0）slices。
// ② combine 后整 chunk density 对拍：GPU sample_chunk 逐块 vs fallback sample 逐点。
// ③ C1 生死门计时：角点 fill 实测（1225 点 + split 上传 ~42MB，勿外推）。
// 跨域覆盖（judge D1）：原点 / 负坐标 / 远坐标多 chunk。
// 用法：WG_GPU_CHANNELS=1（默认关，勿污染其他 probe）+ PATH 含 build-msvc/bin；仓库根运行。
use std::sync::atomic::{AtomicU32, Ordering};
use WorldgenRust::terrain::{ChunkDensitySampler, DensitySource};
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn probe_chunk(gc: &WorldgenRust::gpu_ffi::GpuChannelDensity, seed: i64, cx: i32, cz: i32, min_y: i32, nh: i32) -> bool {
    println!("--- chunk ({},{}) ---", cx, cz);
    let t0 = std::time::Instant::now();
    let gpu = match gc.sample_chunk(cx, cz, min_y, nh) {
        Some(cd) => cd.slices().to_vec(),
        None => { eprintln!("[FAIL] GPU sample_chunk None"); return false; }
    };
    let dt = t0.elapsed().as_secs_f64();
    println!("  [C1] gpu corner fill (1225 pts + ~42MB split): {:.1} ms", dt * 1000.0);
    let cpu = gc.cpu_slices(cx, cz);
    // ① 逐通道 + 5×5 交叉矩阵（通道序错位定位：GPU ch_a vs CPU ch_b 全组合，取最小 major 行列）
    let nch = 5usize;
    let mut ok = true;
    let mut matrix = [[0usize; 5]; 5];
    let mut gmax = [[0.0f64; 5]; 5];
    let n = gpu.len() / nch;
    for ga in 0..nch { for cb in 0..nch {
        let mut major = 0usize; let mut max_diff = 0.0f64;
        for i in 0..n {
            let d = (gpu[i * nch + ga] - cpu[i * nch + cb]).abs();
            if d > 1e-4 { major += 1; }
            if d > max_diff { max_diff = d; }
        }
        matrix[ga][cb] = major; gmax[ga][cb] = max_diff;
    }}
    for ga in 0..nch {
        let (best_cb, best_major) = {
            let mut b = 0; let mut m = usize::MAX;
            for cb in 0..nch { if matrix[ga][cb] < m { m = matrix[ga][cb]; b = cb; } }
            (b, m)
        };
        println!("  gpu_ch{ga} vs cpu: major={:?} → best=cpu_ch{best_cb} (major={best_major}, max={:.3e})",
                 [matrix[ga][0], matrix[ga][1], matrix[ga][2], matrix[ga][3], matrix[ga][4]], gmax[ga][best_cb]);
        if matrix[ga][ga] > 0 { ok = false; }
    }
    // ② combine 后整 chunk：a = GPU slices 经 sample_interp（trilerp+combine）vs b = CPU fallback 逐点`r`n    // ①b 三方决定性对拍（ch0 分叉定位）：GPU ch0 vs CPU ch0 vs DFC 整树直采 @ 最差 5 角点
    // （DfcDensity 已验证 vs C++ production；若 dfc≈cpu_ch0 → GPU ch0 分叉；dfc≈gpu_ch0 → Rust ch0 分叉）
    {
        let dfc = WorldgenRust::terrain::DfcDensity::new(seed as u64);
        let mut worst: Vec<(usize, f64)> = (0..n).map(|i| (i, (gpu[i * nch] - cpu[i * nch]).abs())).collect();
        worst.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let gx = 5usize; let gz = 5usize; let gy = (nh / 8 + 1) as usize;
        println!("  ch0 三方对拍（最差 5 角点）:");
        for wi in worst.iter().take(5) {
            let (i, _d) = *wi;
            // slices 布局 i = (iy*gz+iz)*gx + ix（ix 最低位）——此前误按 ix 最高位分解导致 y 错位乌龙
            let ix = i % gx; let t = i / gx; let iz = t % gz; let iy = t / gz;
            let px = cx * 16 + ix as i32 * 4;
            let py = min_y + iy as i32 * 8;
            let pz = cz * 16 + iz as i32 * 4;
            let dv = dfc.sample(&WorldgenRust::density::NoisePos { x: px, y: py, z: pz });
            println!("    @({},{},{}) gpu={:.6} cpu_ch0={:.6} dfc={:.6} |diff gpu-dfc={:.2e} cpu-dfc={:.2e}",
                     px, py, pz, gpu[i * nch], cpu[i * nch], dv,
                     (gpu[i * nch] - dv).abs(), (cpu[i * nch] - dv).abs());
        }
        let _ = (gx, gz);
    }
    let mut cmajor = 0usize;
    for lx in [0i32, 5, 15] { for lz in [0i32, 7, 15] {
        for by in (0..nh).step_by(3) {
            let pos = WorldgenRust::density::NoisePos { x: cx * 16 + lx, y: min_y + by, z: cz * 16 + lz };
            let a = gc.sample_interp(&gpu, &pos);   // GPU 角点 slices → trilerp + combine
            let b = gc.cpu_sample(&pos);
            if (a - b).abs() > 1e-4 { cmajor += 1; }
        }
    }}
    println!("  combine 后抽样 major_diff={cmajor}");
    if cmajor > 0 { ok = false; }
    ok
}

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "versions/1.20.1/data/worldgen";
    let min_y = -64; let nh = 384;
    println!("=== gpu_channel_probe (X2, 260903-05) seed={} ===", seed);
    // 跨域 chunk 集（judge D1：原点/负/远）
    let chunks: Vec<(i32, i32)> = vec![(0, 0), (-3, -5), (45, -28), (6553, -6554), (200, 200)];

    // 生产入口创建 handle（NoiseSet/db 派生与生产零差异）
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "overworld.json", "biome_params.json", 384) {
        Some(h) => h,
        None => { eprintln!("[FAIL] handle create"); std::process::exit(1); }
    };
    let _ = AtomicU32::new(0); // silence unused import in some cfgs
    let gc = match h.gpu_channels_density() {
        Some(g) => g,
        None => { eprintln!("[FAIL] WG_GPU_CHANNELS not set or engine create failed"); std::process::exit(1); }
    };
    let mut all_ok = true;
    for (cx, cz) in &chunks {
        if !probe_chunk(gc, seed, *cx, *cz, min_y, nh) { all_ok = false; }
    }
    if all_ok { println!("=== [PASS] 逐通道 + combine 对拍全绿（f32 口径）"); }
    else { println!("=== [FAIL] 通道序/语义分叉 — 禁止生产启用"); std::process::exit(1); }
}
