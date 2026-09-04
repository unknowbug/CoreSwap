// qaq1_surf_probe.rs — Q-AQ1 决定性探针：生产路径 estimate_surface_height 调用数/迭代数（260903-10）
// 口径：seed 8576294172403134396 / region (200,200) / 16 chunks，与 qaq1_counter_probe 同。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::aquifer;
use std::time::Instant;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

fn main() {
    println!("=== qaq1_surf_probe (260903-10) seed={} ===", SEED);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    for i in 0..4 { let _ = h.fill_chunk_blocks(400 + i, 400); }
    aquifer::aquifer_surf_watch(true);
    aquifer::aquifer_wl_watch(true);

    let n = 16;
    let mut times = Vec::with_capacity(n);
    for cz in 0..4 { for cx in 0..4 {
        let t = Instant::now();
        let _ = h.fill_chunk_blocks(200 + cx, 200 + cz);
        times.push(t.elapsed().as_secs_f64() * 1e3);
    }}
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("[timing] {} chunks median={:.2}ms", n, times[times.len()/2]);

    let [sc, si] = aquifer::aquifer_surf_count_reset();
    let [wl_calls, wl_miss] = aquifer::aquifer_wl_count_reset();
    println!("[surf] estimate_surface_height calls={} per_chunk={:.0} iterations={} per_chunk={:.0} avg_iter/call={:.2}",
        sc, sc as f64/n as f64, si, si as f64/n as f64, si as f64/sc as f64);
    println!("[wl] calls={} per_chunk={:.0} miss={} (miss→get_fluid_level→surf est)", wl_calls, wl_calls as f64/n as f64, wl_miss);
    println!("[注] surf est 的调用者还有 get_fluid_level 内 13 偏移循环（列缓存去重后 calls≈去重列数）");
    aquifer::aquifer_surf_watch(false);
    aquifer::aquifer_wl_watch(false);
}
