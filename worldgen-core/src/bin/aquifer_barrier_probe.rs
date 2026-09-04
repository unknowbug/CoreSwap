// aquifer_barrier_probe.rs — 统计 calculate_density 里 barrier.sample 调用次数。
// 用于量化 aquifer 内部 barrier 采样占比（98304 点/chunk 里多少次 barrier.sample）。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::aquifer;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(-8248318472910187742, wg_dir).expect("create");
    aquifer::aquifer_barrier_watch(true);
    // 预热 + reset
    let _ = h.fill_chunk_blocks(-288, -256);
    aquifer::aquifer_barrier_watch(true); // reload to reset
    let chunks = [(-288,-256),(-287,-256),(-286,-256),(-285,-256)];
    for (cx, cz) in &chunks { let _ = h.fill_chunk_blocks(*cx, *cz); }
    let count = aquifer::aquifer_barrier_count_reset();
    let n_chunks = chunks.len(); let n_points = n_chunks * (16*16*384);
    println!("barrier.sample calls: {} over {} chunks ({} points)", count, n_chunks, n_points);
    println!("barrier sample rate: {:.1}% of points (or {:.0} per chunk)", count as f64 / n_points as f64 * 100.0, count as f64 / n_chunks as f64);
    aquifer::aquifer_barrier_watch(false);
}
