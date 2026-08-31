use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = std::env::var("WG_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(-2032795982907864146);
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    let blocks = h.fill_chunk_blocks(1, 1);
    let n = blocks.len();
    let height = n / 256;
    println!("chunk(1,1) seed={} fill 长度={} 高度={}", seed, n, height);
    // 每 y 层：非 air 计数 + 主要块
    for y in (0..128).step_by(8) {
        let base = y * 256;
        let mut counts: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
        for k in base..base + 256 {
            *counts.entry(blocks[k]).or_insert(0) += 1;
        }
        let air = *counts.get(&0).unwrap_or(&0);
        let mut top = String::new();
        for (b, cnt) in counts.iter().rev().take(3) {
            top.push_str(&format!(" {}x{}", b, cnt));
        }
        println!("y={:<4} air={:<4} 顶部:{}", y, air, top);
    }
}
