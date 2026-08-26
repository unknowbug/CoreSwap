// terrain_map_probe.rs — vanilla 主体地形对照（宽松判据：不是逐位）。
// 用 Rust finalDensity 对一个区域（几 chunk）算每列地表高度（从顶往下首个 density>0 = 石头顶），
// 输出 16x16 ASCII 高度图 + 石头/空气/水占比，与 C++ 参照对照，展示「主体地形」是否一致。
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

// 对 (cx,cz) chunk：扫每列地表高度（首个 density>0 的 y，自顶向下），返回 16x16 高度表（NaN=全空气）。
fn surface_map(tree: &DensityFunction, cx: i32, cz: i32) -> [[f64; 16]; 16] {
    let mut map = [[f64::NAN; 16]; 16];
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16+bx, cz*16+bz);
        for y in (-64..320).rev() {
            if tree.sample(&NoisePos{x, y, z}) > 0.0 { map[bz as usize][bx as usize] = y as f64; break; }
        }
    }}
    map
}

fn print_map(cx: i32, cz: i32, map: &[[f64;16];16]) {
    println!("chunk({},{}) surface height (top solid Y; .. = no solid):", cx, cz);
    for zrow in map.iter() {
        let mut line = String::new();
        for &v in zrow {
            if v.is_nan() { line.push_str(" .."); } else { line.push_str(&format!(" {:3.0}", v)); }
        }
        println!("{}", line);
    }
}

fn main() {
    let seed = 8576294172403134396u64;
    let tree = build_tree(seed);
    // 两个相邻 chunk（720,-432 附近 = chunk 45,-27..-26，对应 vanilla 参照区域）
    for (cx, cz) in [(45, -27), (45, -26)] {
        let map = surface_map(&tree, cx, cz);
        print_map(cx, cz, &map);
        // 石头/空气/水占比概览（水 = y<63 且该列地表高度 < 63）
        let mut solid = 0; let mut water = 0; let mut air = 0;
        for zrow in &map { for &v in zrow {
            if v.is_nan() { air += 1; } else if v < 63.0 { water += 1; } else { solid += 1; }
        }}
        println!("  -> {}/256 solid-land, {}/256 below-waterline, {}/256 all-air", solid, water, air);
    }
}
