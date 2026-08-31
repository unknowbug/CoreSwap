// nether_density_column.rs — Rust final_density 竖切（x=8, z=8 @ chunk 0,0，y 4 步进 0..252）
// 与 Java vanilla_density_nether_c0_0_b8_8.txt 对拍。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = std::env::var("WG_SEED").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-8248318472910187742);
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    // fill_density(min_chunk_x, min_chunk_z, size=1)：网格 XZ_INTERVAL=4, Y=8
    // 列 (x=8, z=8) → x_idx=2, z_idx=2 → chunk 内 index = y_idx*16 + 10
    let _points = 0;

    // 输出布局：[y_idx][z_idx][x_idx]，y_idx 0..(256/8=32)
    println!("=== Rust nether final_density 列 (8,8) 纯函数 ===");
    for y in (0..256).step_by(4) {
        let v = h.sample_density_exact(8, y, 8);
        println!("y={} {:.6}", y, v);
    }

}



