// pipeline_bench.rs — Rust 完整管线性能基线（WorldgenHandle::fill_chunk_blocks）。
// 完整链路：density + aquifer + ore_vein + build_surface + carver + features + Beardifier。
// 测：单线程每 chunk 耗时 + 多线程扩展（无探针污染：纯 wall + 计数，见测量铁律）。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");
    println!("完整管线基准：density+aquifer+ore_vein+surface+carver+features");

    // 预热（构建缓存）
    let _ = h.fill_chunk_blocks(-18, -16);

    // == 单线程：16 chunks（4x4，-288,-256）==
    let cxs: Vec<i32> = (0..16).map(|i| -288 + (i % 4)).collect();
    let czs: Vec<i32> = (0..16).map(|i| -256 + (i / 4)).collect();
    let t0 = Instant::now();
    for i in 0..16 { let _ = h.fill_chunk_blocks(cxs[i], czs[i]); }
    let dt = t0.elapsed().as_secs_f64();
    println!("单线程 16 chunks: wall={:.1}ms per-chunk={:.2}ms", dt * 1e3, dt * 1e3 / 16.0);

    // == 多线程：64 chunks（8x8），thread::scope 全并行 ==
    let n = 64usize;
    let cxs8: Vec<i32> = (0..n).map(|i| -288 + (i as i32) % 8).collect();
    let czs8: Vec<i32> = (0..n).map(|i| -256 + (i as i32) / 8).collect();
    let h_arc = std::sync::Arc::new(&h);
    let t1 = Instant::now();
    std::thread::scope(|s| {
        for i in 0..n {
            let h = h_arc.clone();
            let cx = cxs8[i];
            let cz = czs8[i];
            s.spawn(move || { let _ = h.fill_chunk_blocks(cx, cz); });
        }
    });
    let dt8 = t1.elapsed().as_secs_f64();
    println!("thread::scope 64 chunks: wall={:.1}ms per-chunk={:.2}ms (全并行)", dt8 * 1e3, dt8 * 1e3 / n as f64);
}
