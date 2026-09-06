// end_fill_diag.rs — D3 端到端自检：create_for_dim("end") → biome sanity → fill chunk
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(12345);
    let wg_dir = args.get(2).map(|s| s.clone().replace('/', "\\"))
        .unwrap_or_else(|| "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen".to_string());
    println!("[wg_dir] {}", wg_dir);
    let h = WorldgenHandle::create_for_dim(seed, &wg_dir, "end.json", "", 128)
        .expect("create_for_dim end failed");
    println!("[OK] handle: min_y={} height={} noise_height={} sea_level={} aquifers={}",
        h.min_y, h.height, h.noise_height, h.sea_level, h.aquifers_enabled);

    // biome sanity：中心 → the_end；外围按 erosion 阈值
    for (x, z) in [(0, 0), (0, 8192), (2000, 2000), (-16384, 4096), (100, 100)] {
        println!("biome(x={},z={})={}", x, 64, h.biome_sample(x, 64, z));
    }

    // fill 两个 chunk：中心岛 (0,0) + 外围 (100,100)
    for (cx, cz) in [(0i32, 0i32), (100i32, 100i32)] {
        let blocks = h.fill_chunk_blocks(cx, cz);
        let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for &b in &blocks {
            *counts.entry(h.block_name(b)).or_insert(0) += 1;
        }
        println!("chunk({},{}) biome={} total={} counts={:?}", cx, cz, h.biome_sample(cx * 16, 64, cz * 16), blocks.len(), counts);
    }
}
