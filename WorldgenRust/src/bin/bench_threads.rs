// bench_threads.rs — 实测自适应核心数（物理核 vs 逻辑核）。
// 用法：bench_threads <originX> <originZ> <chunkCount> [threads]
// 用固定 threads 数分块并行生成 N chunks，wall 计时。
// 对比不同线程数（12 物理核 vs 24 逻辑核）的吞吐，实测确定自适应方向。
use std::env;
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = env::args().collect();
    let origin_x: i32 = args.get(1).map(|s| s.parse().unwrap_or(200)).unwrap_or(200);
    let origin_z: i32 = args.get(2).map(|s| s.parse().unwrap_or(200)).unwrap_or(200);
    let n: usize = args.get(3).map(|s| s.parse().unwrap_or(64)).unwrap_or(64);
    let threads: usize = args.get(4).map(|s| s.parse().unwrap_or(24)).unwrap_or(24);

    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = Arc::new(WorldgenHandle::create(-8248318472910187742, wg_dir).expect("create"));
    // 预热
    let _ = h.fill_chunk_blocks(-18, -16);

    let side = (n as f64).sqrt() as i32;
    let cxs: Vec<i32> = (0..n).map(|i| origin_x + (i as i32 % side)).collect();
    let czs: Vec<i32> = (0..n).map(|i| origin_z + (i as i32 / side)).collect();

    // 分块：把 n 个 chunk 分成 threads 组，每组内串行，组间并行
    let t0 = Instant::now();
    let nthreads = threads.min(n);
    std::thread::scope(|s| {
        for t in 0..nthreads {
            let h = h.clone();
            let cxs = &cxs;
            let czs = &czs;
            // 每个线程处理 chunk_indices = {t, t+nthreads, t+2*nthreads, ...}（分块交错）
            s.spawn(move || {
                let mut i = t;
                while i < n {
                    let _ = h.fill_chunk_blocks(cxs[i], czs[i]);
                    i += nthreads;
                }
            });
        }
    });
    let dt = t0.elapsed().as_secs_f64();
    println!("threads={} chunks={}: wall={:.1}ms per-chunk={:.2}ms throughput={:.1}chunk/s",
             nthreads, n, dt * 1e3, dt * 1e3 / n as f64, n as f64 / dt);
}
