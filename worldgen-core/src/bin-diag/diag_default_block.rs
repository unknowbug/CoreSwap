// diag_default_block.rs — 260907-09：vivo 路径 default_block 消费诊断。
// 直接调 WorldgenHandle::create_for_dim（与 JNI 同构入口），观察 fill_chunk_blocks 输出的 id 分布。
// env CORESWAP_DEFAULT_BLOCK=minecraft:coarse_dirt 下：Rock 应=10（coarse_dirt），不应=1（stone）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create_for_dim(8576294172403134396, dir, "overworld.json", "biome_params.json", 384)
        .expect("create_for_dim failed");
    let blocks = h.fill_chunk_blocks(0, 0);
    let mut counts: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
    for &b in &blocks { *counts.entry(b as i32).or_insert(0) += 1; }
    println!("total cells: {}", blocks.len());
    for (id, n) in counts { println!("id {}: {}", id, n); }
}
