// dfc_verify.rs — lossless-accel P2a：DFC 路径 vs transpiler vs macro 三路对比（260903-03）。
// ① 密度级：单 chunk 98304 点 final_density 采样，{:.6} 舍入 max_diff（架构 §3 判据）
// ② 块级：fill_chunk_blocks 输出对比
// ③ 性能：无探针整批 wall（AGENTS 测量铁律；§9.7：单机 Rust 侧 fill 口径，仅与同口径历史值比）
use WorldgenRust::density::NoisePos;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::terrain::{DensitySource, DfcDensity, TranspilerDensity};
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = -8248318472910187742;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

fn build_transpiler(min_y: i32, noise_height: i32) -> Option<TranspilerDensity> {
    let mut db = DensityBuilder::new(SEED as u64, min_y, noise_height);
    let noise_params_path = format!("{}\\..\\noise_params.json", WG_DIR);
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&noise_params_path).ok()?;
    let mut noises = WorldgenRust::noise::NoiseSet::new();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }
    Some(TranspilerDensity::new(noises, min_y, noise_height))
}

fn main() {
    let min_y = -64i32; let height = 384i32;
    let td = build_transpiler(min_y, height).expect("transpiler density");
    let dfc = DfcDensity::new(SEED as u64);

    // ① 密度级：chunk (-288,-256) 全 98304 块点
    let (cx, cz) = (-288i32, -256i32);
    let mut max_diff = 0.0f64; let mut n_diff6 = 0usize; let mut n = 0usize;
    let mut sum_td = 0.0f64; let mut sum_dfc = 0.0f64;
    for lx in 0..16i32 { for lz in 0..16i32 {
        for ly in 0..height {
            let pos = NoisePos { x: cx * 16 + lx, y: min_y + ly, z: cz * 16 + lz };
            let a = td.sample(&pos);
            let b = dfc.sample(&pos);
            let diff = (a - b).abs();
            if diff > max_diff { max_diff = diff; }
            if format!("{:.6}", a) != format!("{:.6}", b) { n_diff6 += 1; }
            sum_td += a; sum_dfc += b;
            n += 1;
        }
    } }
    println!("[density] chunk({},{}) n={} max_diff={:.9} rounded6_mismatch={} sum_td={:.3} sum_dfc={:.3}",
        cx, cz, n, max_diff, n_diff6, sum_td, sum_dfc);

    // ② 块级：4 chunk fill 对比（dfc vs td）
    unsafe {
        std::env::set_var("WG_SKIP_CARVER", "1"); std::env::set_var("WG_SKIP_FEATURES", "1");
        std::env::set_var("WG_TRANSPILER", "1"); std::env::remove_var("WG_DFC");
    }
    let h_td = WorldgenHandle::create(SEED, WG_DIR).expect("handle td");
    unsafe { std::env::remove_var("WG_TRANSPILER"); std::env::set_var("WG_DFC", "1"); }
    let h_dfc = WorldgenHandle::create(SEED, WG_DIR).expect("handle dfc");
    unsafe { std::env::remove_var("WG_DFC"); }
    let h_ms = WorldgenHandle::create(SEED, WG_DIR).expect("handle ms");

    let chunks: Vec<(i32, i32)> = vec![(-288, -256), (-287, -256), (-286, -255), (-288, -255)];
    let mut mismatches = 0usize; let mut total = 0usize;
    for &(x, z) in &chunks {
        let a = h_td.fill_chunk_blocks(x, z);
        let b = h_dfc.fill_chunk_blocks(x, z);
        let c = h_ms.fill_chunk_blocks(x, z);
        let m_ab = a.iter().zip(b.iter()).filter(|(p, q)| p != q).count();
        let m_cb = c.iter().zip(b.iter()).filter(|(p, q)| p != q).count();
        println!("[blocks] chunk({},{}) td_vs_dfc_mismatch={} ms_vs_dfc_mismatch={} (len {})", x, z, m_ab, m_cb, a.len());
        mismatches += m_ab; total += a.len();
    }
    println!("[blocks] total td_vs_dfc mismatch {}/{} = {:.2}%", mismatches, total, mismatches as f64 / total as f64 * 100.0);

    // ③ 性能：无探针整批 wall（16 chunk × 3 轮中位）
    use std::time::Instant;
    let batch: Vec<(i32, i32)> = (-256i32..-252).flat_map(|z| (-288i32..-284).map(move |x| (x, z))).collect();
    for (name, h) in [("macro", &h_ms), ("transpiler", &h_td), ("dfc", &h_dfc)] {
        for &(x, z) in &batch { let _ = h.fill_chunk_blocks(x, z); } // 预热
        let mut times = [0.0f64; 3];
        for t in times.iter_mut() {
            let t0 = Instant::now();
            for &(x, z) in &batch { let _ = h.fill_chunk_blocks(x, z); }
            *t = t0.elapsed().as_secs_f64() / batch.len() as f64 * 1e3;
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("[perf] {} : med {:.2} ms/chunk (16 chunks, skip carver/features)", name, times[1]);
    }
}
