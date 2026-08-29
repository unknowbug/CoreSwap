// multiworld_probe.rs — 验证多世界参数化（create_for_dim）。
// 测试：nether（下界）维度能否加载 + 生成 chunk。
// create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256)
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";

    // 尝试加载 nether 维度
    match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => {
            println!("[OK] nether dimension created: min_y={} height={}", h.min_y, h.height);
            // 生成一个 chunk 验证
            let blocks = h.fill_chunk_blocks(0, 0);
            let mut nz = 0;
            for &b in &blocks { if b != 0 { nz += 1; } }
            println!("[OK] nether chunk(0,0) generated: non-air blocks = {} / {} (min_y={} height={})", nz, blocks.len(), h.min_y, h.height);
        }
        None => println!("[FAIL] nether create_for_dim failed"),
    }
}
