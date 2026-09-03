// estopt_ab.rs — b1-a/b1-b 四臂 A/B 探针（260903-11）。
// 臂由 env 控制（WG_EST_SHARED / WG_EST_L2，与 fill_chunk_blocks 生产门控同源）：
//   off | shared | l2 | shared+l2
// 输出：8x8 chunks (200,200) blocks 的 FNV-1a 64 hash + L2 统计。
// 用途：① off 臂 = 默认路径零回归基线（vs HEAD 构建同 hash）② shared 臂 vs off = D1 语义变化量
// ③ l2 臂 vs off = L2 精确性自检（必须同 hash）④ l2 stats = miss 率实测。
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

fn fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn main() {
    let shared = std::env::var("WG_EST_SHARED").is_ok();
    let l2 = std::env::var("WG_EST_L2").is_ok();
    println!("=== estopt_ab arm shared={} l2={} seed={} ===", shared, l2, SEED);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    // 预热（区外，触发可能的 L2 初始化但不计入 hash 区域）
    for i in 0..2 { let _ = h.fill_chunk_blocks(400 + i, 400); }

    let mut agg: u64 = 0xcbf29ce484222325;
    let n = 64;
    for cz in 0..8 { for cx in 0..8 {
        let blocks = h.fill_chunk_blocks(200 + cx, 200 + cz);
        // 每块一个中间 hash 再链式混合，防跨块字节边界伪同
        let chunk_h = fnv(bytemuck_cast(&blocks));
        agg = (agg ^ chunk_h).wrapping_mul(0x100000001b3);
    }}
    println!("[hash] {} chunks agg={:016x}", n, agg);
    let s = h.est_l2_stats();
    println!("[l2] hits={} misses={} inserts={} evictions={} (hit_rate={})", s[0], s[1], s[2], s[3],
        if s[0] + s[1] > 0 { format!("{:.1}%", s[0] as f64 / (s[0] + s[1]) as f64 * 100.0) } else { "n/a".into() });
}

fn bytemuck_cast(v: &Vec<i32>) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}
