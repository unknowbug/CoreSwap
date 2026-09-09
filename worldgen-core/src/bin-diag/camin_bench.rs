// camin_bench.rs — WG_CA_MIN 性能判别 bench（260909-04，bin-diag 临时区）。
// 无探针污染：纯 wall + 内容 hash（行为等价哨兵）。串行 region 扫描（#28 串行铁律）。
// 用法：camin_bench.exe <seed> <wg_dir> <N chunks/边> [origin_x origin_z]
// 输出：wall、per-chunk、FNV-1a 64 内容 hash（CA_MIN on/off 同 hash = 行为等价）。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args.get(1).map(|s| s.parse().expect("seed i64")).unwrap_or(-8248318472910187742);
    let wg_dir = args.get(2).cloned()
        .unwrap_or_else(|| "E:\\PYTHON\\CoreSwap\\versions\\1.21.6\\data\\worldgen".to_string());
    let n: usize = args.get(3).map(|s| s.parse().expect("N")).unwrap_or(16);
    let ox: i32 = args.get(4).map(|s| s.parse().expect("ox")).unwrap_or(0);
    let oz: i32 = args.get(5).map(|s| s.parse().expect("oz")).unwrap_or(0);

    let ca_min = std::env::var("WG_CA_MIN").map(|v| v != "0").unwrap_or(true);
    let h = WorldgenHandle::create(seed, &wg_dir).expect("create handle");
    let bpc = (16 * 16 * h.height as usize) as u64;
    println!("handle: min_y={} height={} seed={} wg={} N={}x{} origin=({},{}) WG_CA_MIN={}",
        h.min_y, h.height, seed, wg_dir, n, n, ox, oz, ca_min);

    // 预热 1 chunk（页/分配器）
    let _ = h.fill_chunk_blocks(ox, oz);

    let mut hash: u64 = 0xcbf29ce484222325;
    let mut placed_total: u64 = 0;
    let t0 = Instant::now();
    for c in 0..(n * n) {
        let cx = ox + (c % n) as i32;
        let cz = oz + (c / n) as i32;
        let blocks = h.fill_chunk_blocks(cx, cz);
        std::hint::black_box(&blocks);
        for &b in &blocks {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    let wall = t0.elapsed();
    println!("RESULT: chunks={} wall={:.1}ms per_chunk={:.2}ms hash={:016x} placed_hint={}",
        n * n, wall.as_secs_f64() * 1000.0,
        wall.as_secs_f64() * 1000.0 / (n * n) as f64, hash, placed_total);
    let _ = bpc;
}
