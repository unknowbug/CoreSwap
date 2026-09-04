// qaq1_counter_probe.rs — Q-AQ1 归因：aquifer 计数器组合采数（260903-10）
// 口径：seed 8576294172403134396 / region (200,200)（与 qpd1_stage_bench 同口径，§9.7）。
// 统计：barrier.sample 调用 / get_water_level_at calls+miss / get_block_pos calls+miss。
// #20 自变量生效自检：计数必须 >0 且随 chunk 数线性；分组两批各 reset 对比。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::aquifer;
use std::time::Instant;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

fn main() {
    println!("=== qaq1_counter_probe (260903-10) seed={} ===", SEED);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    // 预热（区外）+ 开表 + reset
    for i in 0..4 { let _ = h.fill_chunk_blocks(400 + i, 400); }
    aquifer::aquifer_barrier_watch(true);
    aquifer::aquifer_wl_watch(true);
    aquifer::aquifer_bp_watch(true);

    let n = 16;
    let mut times = Vec::with_capacity(n);
    for cz in 0..4 { for cx in 0..4 {
        let t = Instant::now();
        let _ = h.fill_chunk_blocks(200 + cx, 200 + cz);
        times.push(t.elapsed().as_secs_f64() * 1e3);
    }}
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = times[times.len()/2];

    let barrier = aquifer::aquifer_barrier_count_reset();
    let [wl_calls, wl_miss] = aquifer::aquifer_wl_count_reset();
    let [bp_calls, bp_miss] = aquifer::aquifer_bp_count_reset();
    let points = n * 16 * 16 * 384;

    println!("[timing] {} chunks median={:.2}ms", n, median);
    println!("[barrier] calls={} per_chunk={:.0} rate_of_points={:.1}%", barrier, barrier as f64/n as f64, barrier as f64/points as f64*100.0);
    println!("[wl] calls={} per_chunk={:.0} miss={} miss_rate={:.1}%", wl_calls, wl_calls as f64/n as f64, wl_miss, wl_miss as f64/wl_calls as f64*100.0);
    println!("[bp] calls={} per_chunk={:.0} miss={} miss_rate={:.1}% (miss→split_xyz+random)", bp_calls, bp_calls as f64/n as f64, bp_miss, bp_miss as f64/bp_calls as f64*100.0);

    // 第二批（另外 16 chunks）线性自检
    for cz in 4..8 { for cx in 4..8 {
        let _ = h.fill_chunk_blocks(200 + cx, 200 + cz);
    }}
    let barrier2 = aquifer::aquifer_barrier_count_reset();
    let [wl2, wlm2] = aquifer::aquifer_wl_count_reset();
    let [bp2, bpm2] = aquifer::aquifer_bp_count_reset();
    println!("[check2] batch2 barrier={} wl={}({}) bp={}({}) — 与批1对比看线性/稳定", barrier2, wl2, wlm2, bp2, bpm2);
    aquifer::aquifer_barrier_watch(false);
    aquifer::aquifer_wl_watch(false);
    aquifer::aquifer_bp_watch(false);
}
