// seed_pick_dump.rs — #20 死参数自检：biome_pick_cell(self.seed) vs (biome_access_seed)
// 在导出区（块坐标 3200..3264 x 3200..3264，y 采样层）两 seed 选点是否真不同。
// 用法: cargo run --release --bin seed_pick_dump
use WorldgenRust::biome::{biome_pick_cell, biome_hash_seed};

fn main() {
    let seed: i64 = 8576294172403134396;
    let hseed = biome_hash_seed(seed);
    println!("seed={seed} hashSeed={hseed}");
    let mut diff = 0u32; let mut tot = 0u32;
    for x in (3200..3264).step_by(4) {
        for z in (3200..3264).step_by(4) {
            for y in [-32i32, 40, 64] {
                let a = biome_pick_cell(seed, x, y, z);
                let b = biome_pick_cell(hseed, x, y, z);
                tot += 1;
                if a != b { diff += 1; if diff <= 5 { println!("DIFF ({x},{y},{z}): raw={a:?} hashed={b:?}"); } }
            }
        }
    }
    println!("points={tot} pick_diff={diff}");
}
