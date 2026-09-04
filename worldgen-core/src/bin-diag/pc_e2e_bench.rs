// pc_e2e_bench.rs — P-C1：端到端 256 chunks 无探针整批 wall（260903-08）
// 链路：WorldgenHandle::fill_chunk_blocks 完整管线（density+aquifer+ore_vein+surface+carver+features）。
// 口径（§9.7）：region 200,200 16×16=256 chunks（与 Java WorldGenBench 同 region/size），单线程顺序，
// 预热 chunk 在测量区外；报告 avg/median/min/max + 逐 chunk 行（排除冷启动由判读侧做）。
// 运行：仓库根；WG_GPU_CHANNELS 环境变量门控 A/B（默认关）。诊断 bin，rustc 单编（bin-diag 隔离区）。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396; // 与 runtime server.properties level-seed / benchSeed 一致；可用 WG_E2E_SEED 覆盖（判别用）
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);
const SIZE: i32 = 16; // 16×16 = 256 chunks

fn median(v: &mut Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

fn main() {
    let gpu = std::env::var("WG_GPU_CHANNELS").is_ok();
    let seed: i64 = std::env::var("WG_E2E_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(SEED);
    println!("=== pc_e2e_bench (260903-08) seed={} region=({},{}) size={} WG_GPU_CHANNELS={} ===",
        seed, ORIGIN.0, ORIGIN.1, SIZE, if gpu { "ON" } else { "OFF" });
    let t0 = Instant::now();
    let h = WorldgenHandle::create(seed, WG_DIR).expect("create handle"); // 260903-10 修复：原恒用常量 SEED，WG_E2E_SEED 死参数（workflow-patterns #20）
    println!("[create] {:.1} ms", t0.elapsed().as_secs_f64() * 1e3);

    // 预热：8 chunks 在测量区外（region 400,400），兼触发懒加载缓存
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }
    println!("[warmup] 8 chunks done (region 400,400, 区外)");

    let mut times: Vec<f64> = Vec::with_capacity((SIZE * SIZE) as usize);
    for cz in 0..SIZE {
        for cx in 0..SIZE {
            let wx = ORIGIN.0 + cx;
            let wz = ORIGIN.1 + cz;
            let t = Instant::now();
            let _ = h.fill_chunk_blocks(wx, wz);
            times.push(t.elapsed().as_secs_f64() * 1e3);
        }
    }
    let n = times.len() as f64;
    let total: f64 = times.iter().sum();
    let mut sorted = times.clone();
    let med = median(&mut sorted);
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    for (i, t) in times.iter().enumerate() {
        println!("[chunk {:>3}] ({},{}) {:.2} ms", i, ORIGIN.0 + (i as i32) % SIZE, ORIGIN.1 + (i as i32) / SIZE, t);
    }
    println!("[RESULT] chunks={} total={:.1}ms avg={:.2}ms median={:.2}ms min={:.2}ms max={:.2}ms",
        times.len(), total, total / n, med, min, max);
    println!("=== done ===");
}
