// nether_density_column.rs — Rust final_density 竖切（x=8, z=8 @ chunk 0,0，y 4 步进 0..252）
// 与 Java vanilla_density_nether_c0_0_b8_8.txt 对拍。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    // fill_density(min_chunk_x, min_chunk_z, size=1)：网格 XZ_INTERVAL=4, Y=8
    // 列 (x=8, z=8) → x_idx=2, z_idx=2 → chunk 内 index = y_idx*16 + 10
    let points = h.fill_density(0, 0, 1);
    let n_per_col = points.len();
    // 输出布局：[y_idx][z_idx][x_idx]，y_idx 0..(256/8=32)
    let y_steps = 256 / 8;
    println!("=== Rust nether final_density 列 (8,8) ===");
    for y_idx in 0..y_steps {
        let y = (y_idx * 8) as i32;
        let idx = y_idx * 16 + 2 * 4 + 2;
        if idx < points.len() {
            println!("y={} {:.6}", y, points[idx]);
        }
    }
    let _ = n_per_col;
}
