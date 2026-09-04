// aquifer_bp_probe.rs — 统计 get_block_pos 调用次数 + miss 率（miss 触发 split_xyz+random，贵）。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::aquifer;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(-8248318472910187742, wg_dir).expect("create");
    aquifer::aquifer_bp_watch(true);
    let _ = h.fill_chunk_blocks(-288, -256);
    aquifer::aquifer_bp_watch(true);
    let chunks = [(-288,-256),(-287,-256),(-286,-256),(-285,-256),(-288,-255),(-287,-255)];
    for (cx, cz) in &chunks { let _ = h.fill_chunk_blocks(*cx, *cz); }
    let [calls, miss] = aquifer::aquifer_bp_count_reset();
    let n = chunks.len();
    println!("get_block_pos calls: {} over {} chunks", calls, n);
    println!("  per chunk: {:.0} calls, {:.0} miss", calls as f64/n as f64, miss as f64/n as f64);
    println!("  miss rate: {:.1}% (miss 触发 split_xyz+random)", miss as f64 / calls as f64 * 100.0);
    aquifer::aquifer_bp_watch(false);
}
