// estopt_mt_bench.rs — b1-b 翻默认前置探针（260903-12）。
// 模式 mt（默认）: T 线程共享 Arc<WorldgenHandle>，16×16=256 chunks 完整管线整批 wall（§9.7:
//   region 200,200 与 pc_e2e_bench 同口径；工作队列 atomic 分发；无阶段探针）。
//   用途 P2.1：est L2 Mutex 争用基线——T=1/2/4/8 各跑 off 与 l2 进程（env 门控），差分 = 争用成本。
// 模式 sweep: 单线程 64×64=4096 chunks 顺序 fill（region 200,200 起），每 256-chunk 块打印
//   l2 stats 增量——超过 FIFO 上限（131072/≈30条每chunk ≈ 4370 chunks）后观察命中率/性能退化。
//   用途 P2.2：大 region 淘汰行为曲线。
// 运行: rustc 单编（bin-diag 隔离区）；臂由 WG_EST_L2 env 控制（与生产门控同源）。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "mt".into());
    let l2 = WorldgenRust::worldgen_handle::env_enabled("WG_EST_L2");
    if mode == "sweep" {
        sweep(l2);
    } else {
        let t: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(1);
        mt(t, l2);
    }
}

fn mt(t: usize, l2: bool) {
    println!("=== estopt_mt_bench mode=mt threads={} l2={} seed={} ===", t, l2, SEED);
    let h = Arc::new(WorldgenHandle::create(SEED, WG_DIR).expect("create handle"));
    // 预热 8 chunks 区外
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }
    let size = 16usize;
    let n = size * size;
    let next = Arc::new(AtomicUsize::new(0));
    // 每线程一份 chunk 列表快照（顺序固定，工作队列分发）
    let t0 = Instant::now();
    let handles: Vec<_> = (0..t).map(|_| {
        let h = h.clone();
        let next = next.clone();
        thread::spawn(move || {
            loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= n { break; }
                let wx = ORIGIN.0 + (i % size) as i32;
                let wz = ORIGIN.1 + (i / size) as i32;
                let _ = h.fill_chunk_blocks(wx, wz);
            }
        })
    }).collect();
    for hd in handles { hd.join().unwrap(); }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    println!("[wall] {} chunks threads={} wall={:.1}ms throughput={:.2}ms/chunk", n, t, wall, wall / n as f64);
    let s = h.est_l2_stats();
    println!("[l2] hits={} misses={} inserts={} evictions={} (hit_rate={})", s[0], s[1], s[2], s[3],
        if s[0] + s[1] > 0 { format!("{:.1}%", s[0] as f64 / (s[0] + s[1]) as f64 * 100.0) } else { "n/a".into() });
}

fn sweep(l2: bool) {
    println!("=== estopt_mt_bench mode=sweep l2={} seed={} 64x64 chunks ===", l2, SEED);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }
    let size = 64usize;
    let t0 = Instant::now();
    let mut prev = [0usize; 4];
    let mut block_t0 = Instant::now();
    for cz in 0..size { for cx in 0..size {
        let _ = h.fill_chunk_blocks(ORIGIN.0 + cx as i32, ORIGIN.1 + cz as i32);
        let done = cz * size + cx + 1;
        if done % 256 == 0 {
            let s = h.est_l2_stats();
            let d = [s[0] - prev[0], s[1] - prev[1], s[2] - prev[2], s[3] - prev[3]];
            println!("[block {:>2}] chunks={:>4} block_wall={:.0}ms hit_rate={} hits={} misses={} inserts={} evictions={} (cum evictions={})",
                done / 256 - 1, done, block_t0.elapsed().as_secs_f64() * 1e3,
                if d[0] + d[1] > 0 { format!("{:.1}%", d[0] as f64 / (d[0] + d[1]) as f64 * 100.0) } else { "n/a".into() },
                d[0], d[1], d[2], d[3], s[3]);
            prev = s;
            block_t0 = Instant::now();
        }
    }}
    println!("[total] wall={:.0}ms", t0.elapsed().as_secs_f64() * 1e3);
}
