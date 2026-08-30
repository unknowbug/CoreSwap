// transpiler_prod_perf.rs — 对比 transpiler 接入生产 vs 基线（DensityMacroSampler）的 fill_chunk_blocks 端到端性能。
// 用两个 WorldgenHandle（WG_TRANSPILER vs 非），跳过 carver/features，测多 chunk 的 wall 时间。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    unsafe { std::env::set_var("WG_SKIP_CARVER", "1"); std::env::set_var("WG_SKIP_FEATURES", "1"); }

    // 基线（DensityMacroSampler）
    unsafe { std::env::remove_var("WG_TRANSPILER"); }
    let h_ms = WorldgenHandle::create(seed, wg_dir).expect("create handle (ms)");
    // transpiler
    unsafe { std::env::set_var("WG_TRANSPILER", "1"); }
    let h_td = WorldgenHandle::create(seed, wg_dir).expect("create handle (td)");

    // 预热
    for cz in -256..-252 { for cx in -288..-284 { let _ = h_ms.fill_chunk_blocks(cx, cz); } }
    for cz in -256..-252 { for cx in -288..-284 { let _ = h_td.fill_chunk_blocks(cx, cz); } }

    // 测 4x4=16 chunk
    let t0 = Instant::now();
    for _ in 0..3 { for cz in -256..-252 { for cx in -288..-284 { let _ = h_ms.fill_chunk_blocks(cx, cz); } } }
    let dt_ms = t0.elapsed().as_secs_f64() / 3.0 * 1e3;

    let t1 = Instant::now();
    for _ in 0..3 { for cz in -256..-252 { for cx in -288..-284 { let _ = h_td.fill_chunk_blocks(cx, cz); } } }
    let dt_td = t1.elapsed().as_secs_f64() / 3.0 * 1e3;

    println!("fill_chunk_blocks 16 chunk (skip carver/features):");
    println!("  DensityMacroSampler: {:.2} ms/chunk", dt_ms / 16.0);
    println!("  TranspilerDensity:  {:.2} ms/chunk", dt_td / 16.0);
    println!("  transpiler/基线: {:.2}x", dt_td / dt_ms);
}
