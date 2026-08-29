// bench_single.rs — Rust 完整管线单线程基准（公平对比 Java WorldGenBench）。
// 用法：bench_single <originX> <originZ> [seed]
// 测：单线程顺序生成 N chunks，排除前 2 个冷启动，报告稳定 avg ms/chunk。
// 公平对比协议：同区域 + 单线程 + 排除冷启动 + 取中位数。
use std::env;
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = env::args().collect();
    let origin_x: i32 = args.get(1).map(|s| s.parse().unwrap_or(200)).unwrap_or(200);
    let origin_z: i32 = args.get(2).map(|s| s.parse().unwrap_or(200)).unwrap_or(200);
    let seed: i64 = args.get(3).map(|s| s.parse().unwrap_or(-8248318472910187742)).unwrap_or(-8248318472910187742);

    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");

    // 4x4 = 16 chunks，region origin
    let n = 16usize;
    let cxs: Vec<i32> = (0..n).map(|i| origin_x + (i as i32 % 4)).collect();
    let czs: Vec<i32> = (0..n).map(|i| origin_z + (i as i32 / 4)).collect();

    // 排除前 2 个冷启动（构建缓存/JIT）
    let mut times_ms: Vec<f64> = Vec::new();
    for i in 0..n {
        let t0 = Instant::now();
        let _ = h.fill_chunk_blocks(cxs[i], czs[i]);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        if i >= 2 { times_ms.push(ms); } // 排除冷启动
    }
    // 中位数稳定值
    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = times_ms[times_ms.len() / 2];
    let avg = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
    println!("Rust single-thread region({},{}) seed={} chunks={}: min={:.1}ms median={:.1}ms avg={:.1}ms",
             origin_x, origin_z, seed, n, times_ms[0], median, avg);
    // 逐个打印（对齐 Java 诊断）
    print!("per-chunk: ");
    for t in &times_ms { print!("{:.0} ", t); }
    println!();
}
