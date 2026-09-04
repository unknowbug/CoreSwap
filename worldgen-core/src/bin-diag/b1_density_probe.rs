// b1_density_probe.rs — B1 下钻：13 个差异单元格的 Rust finalDensity 精确采样（bin-diag，不进默认构建）
// seed 8576294172403134396, chunk 3200..3211 区, 差异单元格来自 .tmp/b1drill_step0_diffdetail.out.txt
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = std::env::var("WG_SEED").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(8576294172403134396);
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    // (x, y, z, 方向: R=rust-only-air, V=van-only-air) —— rust d 应与 vanilla 反号或近零
    let cells: [(i32, i32, i32, char); 13] = [
        (51200, 75, 51339, 'V'),
        (51202, 33, 51336, 'V'),
        (51204, 109, 51361, 'R'),
        (51213, 97, 51381, 'R'),
        (51221, 84, 51337, 'R'),
        (51221, 55, 51339, 'R'),
        (51222, 96, 51354, 'R'),
        (51222, 72, 51365, 'V'),
        (51227, 73, 51338, 'R'),
        (51229, 43, 51334, 'R'),
        (51231, 48, 51329, 'R'),
        (51240, 73, 51348, 'R'),
        (51256, 51, 51364, 'R'),
    ];
    println!("[sanity] seed={} n=13", seed);
    println!("x,y,z,dir,d_exact,abs_d");
    for &(x, y, z, dir) in cells.iter() {
        let d = h.sample_density_exact(x, y, z);
        println!("{},{},{},{},{:.17e},{:.3e}", x, y, z, dir, d, d.abs());
    }
}
