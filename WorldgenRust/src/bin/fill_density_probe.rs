// density_probe.rs — 验证 wg_fill_density / fill_density（finalDensity 网格采样）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");

    // 1x2 chunks
    let points = h.fill_density(0, 0, 2);
    let xz = WorldgenRust::api::density_xz_interval();
    let yi = WorldgenRust::api::density_y_interval();
    let sx = 16 / xz;
    let sy = 384 / yi;
    let expected = 2 * 2 * (sx * sy * sx) as usize;
    println!("fill_density: {} points (expected ~{})", points.len(), expected);
    let mut nz = 0;
    for &p in &points { if p.abs() > 1e-9 { nz += 1; } }
    println!("non-zero points: {} / {}", nz, points.len());
    if nz > 0 { println!("[OK] fill_density functional (density values present)"); }
    else { println!("[WARN] fill_density all zero"); }
}
