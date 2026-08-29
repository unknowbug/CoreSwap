// aquifer_wl_probe.rs — 统计 get_water_level_at 调用次数 + miss 率（miss 触发 get_fluid_level）。
// 用 WL_WATCH 门控（单线程诊断），跑 fill_chunk 若干 chunk。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::aquifer;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(-8248318472910187742, wg_dir).expect("create");
    aquifer::aquifer_wl_watch(true);
    // 预热 + reset
    let _ = h.fill_chunk_blocks(-288, -256);
    aquifer::aquifer_wl_watch(true); // reload to reset
    let chunks = [(-288,-256),(-287,-256),(-286,-256),(-285,-256),(-288,-255),(-287,-255)];
    for (cx, cz) in &chunks { let _ = h.fill_chunk_blocks(*cx, *cz); }
    let [calls, miss] = aquifer::aquifer_wl_count_reset();
    let n_chunks = chunks.len();
    let n_points = n_chunks * (16*16*384);
    println!("get_water_level_at calls: {} over {} chunks ({} points)", calls, n_chunks, n_points);
    println!("  per chunk: {:.0} calls, {:.0} miss", calls as f64/n_chunks as f64, miss as f64/n_chunks as f64);
    println!("  miss rate: {:.1}% (miss 触发 get_fluid_level)", miss as f64 / calls as f64 * 100.0);
    aquifer::aquifer_wl_watch(false);
}
