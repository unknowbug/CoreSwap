// chunk_block_probe.rs — ① Rust 生成 chunk 块（石头/空气/水 + 洞穴），可视化 + 统计，宽松 vanilla 对照。
// 规则（主体地形层）：density>0 石头；density<=0 且 y<63 水；density<=0 且 y>=63 空气。洞穴 = 石头里嵌的空气（地下）。
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

fn main() {
    let seed = 8576294172403134396u64;
    let tree = build_tree(seed);
    let (cx, cz) = (45, -26);
    // 三个洞穴层 + 地表层的 16x16 切片
    let layers = [-64, -40, -10, 40];
    for &ly in &layers {
        let mut line = format!("y={:3} x: ", ly);
        for bx in 0..16 {
            let x = cx*16+bx;
            let mut col = String::new();
            for bz in 0..16 {
                let z = cz*16+bz;
                let d = tree.sample(&NoisePos{x, y: ly, z});
                // 在一行内打 16 个（x 变化）? 改为：每行遍历 bz，打印该 (x,z) 在 y=ly 的类型
                // 这里简化：打印 16x16，行=z，列=x
            }
            let _ = col; let _ = x;
        }
        // 逐行（z）打印 16 列（x）
        println!("--- y={} (z rows, x cols) #=stone .=air ~=water ---", ly);
        for bz in 0..16 {
            let z = cz*16+bz;
            let mut row = String::new();
            for bx in 0..16 {
                let x = cx*16+bx;
                let d = tree.sample(&NoisePos{x, y: ly, z});
                let c = if d > 0.0 { '#' } else if ly < 63 { '~' } else { '.' };
                row.push(c);
            }
            println!("  {}", row);
        }
    }
    // 统计整 chunk（y -64..319 step4）石头/空气/水/洞穴
    let mut stone = 0u64; let mut air = 0u64; let mut water = 0u64;
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16+bx, cz*16+bz);
        for y in (-64..320).step_by(4) {
            let d = tree.sample(&NoisePos{x, y, z});
            if d > 0.0 { stone += 1; } else if y < 63 { water += 1; } else { air += 1; }
        }
    }}
    let total = stone + air + water;
    println!("chunk({},{}) [y -64..320 step4] stone={} air={} water={} (pts={})", cx, cz, stone, air, water, total);
    println!("  cave-ish air fraction (underground air / total) = {:.1}%", air as f64 / total as f64 * 100.0);
}
