// debug_density.rs — 快速看樱花生子 chunk(0,0) 某列的密度分布（是否全水/空气），排除探针 bug。
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
    let seed: i64 = -2032795982907864146;
    let tree = build_tree(seed as u64);
    for &(x, z) in &[(0i32, 0i32), (8, 8), (728, -408), (100, 100)] {
        println!("col ({} , {}):", x, z);
        for y in (-64..200).step_by(16) {
            let d = tree.sample(&NoisePos{x, y, z});
            println!("   y={:4} d={:.4} {}", y, d, if d > 0.0 {"SOLID"} else {"air/water"});
        }
    }
}
