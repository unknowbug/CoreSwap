// ch0_column_probe.rs — X2 ch0 分叉复核（260903-05）：Rust transpiler ch0 在 x=4,z=16 列全 49 y 角点值。
// 判读：若出现跨 y 恒定段（非噪声级巧合）→ Rust ch0 y 依赖结构性缺失（bC 假设）；
// 无 GPU 依赖（WG_TRANSPILER 门控），秒级出结果。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "versions/1.20.1/data/worldgen";
    let (cx, cz) = (0, 0);
    let h = WorldgenHandle::create_for_dim(seed, wg_dir, "overworld.json", "biome_params.json", 384).expect("handle");
    let td = h.transpiler_density().expect("WG_TRANSPILER not set");
    let slices = td.build_slices_for(cx, cz);
    let gx = 5usize; let gz = 5usize; let nch = 5usize;
    let (ix, iz) = (1usize, 4usize); // x=4, z=16
    println!("=== ch0 column x=4 z=16 (49 corners) ===");
    let mut prev = f64::NAN;
    for iy in 0..49usize {
        let v = slices[((iy * gz + iz) * gx + ix) * nch];
        let y = -64 + iy as i32 * 8;
        println!("y={:>4}: {:.6}{}", y, v, if v == prev { "  <== same as prev" } else { "" });
        prev = v;
    }
}
