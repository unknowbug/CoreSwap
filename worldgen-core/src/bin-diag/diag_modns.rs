// diag_modns.rs — 260907-09：缺口 3/4 原生验证（8x8 chunks 区域 hash）。
// 用法: diag_modns.exe <settings_name>
// 输出每个 chunk 的 FNV-1a 64 hash + 全区域聚合 hash，供 vanilla vs mod-ns 两臂逐位对比。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use std::collections::BTreeMap;

fn main() {
    let settings = std::env::args().nth(1).expect("usage: diag_modns <settings_name>");
    let dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create_for_dim(8576294172403134396, dir, &settings, "biome_params_nether.json", 256)
        .expect("create_for_dim failed");
    let mut agg: u64 = 0xcbf29ce484222325;
    for cz in 0..8 { for cx in 0..8 {
        let blocks = h.fill_chunk_blocks(cx, cz);
        if cx == 0 && cz == 0 {
            let mut counts: BTreeMap<i32, usize> = BTreeMap::new();
            for &b in &blocks { *counts.entry(b as i32).or_insert(0) += 1; }
            let hist: Vec<String> = counts.iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
            println!("chunk(00,00) hist: {}", hist.join(" "));
        }
        let mut hash: u64 = 0xcbf29ce484222325;
        for &b in &blocks { hash ^= b as u64; hash = hash.wrapping_mul(0x100000001b3); }
        println!("chunk({:02x},{:02x}) hash={:016x}", cx, cz, hash);
        agg ^= hash; agg = agg.wrapping_mul(0x100000001b3);
    } }
    println!("settings={} AGG={:016x}", settings, agg);
}
