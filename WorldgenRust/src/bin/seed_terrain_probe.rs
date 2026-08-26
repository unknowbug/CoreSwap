// seed_terrain_probe.rs — 用樱花盆地 seed 生成 spawn 附近区域的地表高度/水域图，展示宏观地形（盆地+山+湖）。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn build_tree(seed: u64) -> Arc<DensityFunction> {
    let mut db = DensityBuilder::new(seed, -64, 384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    Arc::new(db.build_node(fd).unwrap())
}

// 每个 chunk 取一格代表 (0,0) 列，扫地表高度（首个密度>0 的 y）。列太多则稀疏抽样。
fn main() {
    let seed: i64 = -2032795982907864146;
    let tree = build_tree(seed as u64);
    println!("seed={} ... surface height map (per chunk, sampled column). Legend: ^=high h=hill x=low-land ~=water .=air", seed);
    // spawn 附近：chunk -3..3 × -3..3 = 7x7 chunks；每 chunk 采样 4x4 = 16 列（稀疏，fast），取平均高度 + 是否水
    let mut rows: Vec<String> = Vec::new();
    for cz in -3..=3 {
        let mut row = String::new();
        for cx in -3..=3 {
            // 该 chunk 4x4 采样列，统计地表高度均值 + 水域比例
            let mut hs = 0.0f64; let mut cnt = 0i32; let mut water = 0i32;
            for bx in [0, 6, 12] { for bz in [0, 6, 12] {
                let (x, z) = (cx*16+bx, cz*16+bz);
                for y in (-64..320).rev() {
                    let d = tree.sample(&NoisePos{x, y, z});
                    if d > 0.0 { hs += y as f64; cnt += 1; if y < 63 { water += 1; } break; }
                    if y == -64 { water += 1; } // 全空/水
                }
            }}
            let c = if cnt == 0 { '~' } else {
                let avg = hs / cnt as f64;
                if water as f64 > (cnt as f64)*0.5 { '~' }      // 多水 = 湖/海
                else if avg > 90.0 { '^' }                      // 高山
                else if avg > 75.0 { 'h' }                      // 丘陵
                else { 'x' }                                    // 平原/盆地
            };
            row.push(c);
        }
        rows.push(row);
    }
    for r in &rows { println!("  {}", r); }
}
