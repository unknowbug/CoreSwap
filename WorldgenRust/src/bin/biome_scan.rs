// biome_scan.rs — 便宜的大范围 biome 扫描：只查 biome（不 fill chunk），找 badlands/desert 位置。
// biome 判定 = 6 参数采样 + 盒包含，每点 ~1μs 级；扫 ±128 chunk 只需 ~65536 点。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let seed: i64 = -2032795982907864146;
    let mut db = DensityBuilder::new(seed as u64, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let t: Arc<DensityFunction> = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let h: Arc<DensityFunction> = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let c: Arc<DensityFunction> = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let e: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let d: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let w: Arc<DensityFunction> = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");

    // 每 8 chunk 采一个点（biome 尺度足够），扫 ±256 chunk
    let mut found: Vec<(i32, i32, String)> = Vec::new();
    let mut badlands_found: Vec<(i32, i32, String)> = Vec::new();
    let mut biome_hist: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for cz in (-256..256).step_by(8) {
        for cx in (-256..256).step_by(8) {
            let x = cx * 16; let z = cz * 16;
            let bp = NoisePos { x: x >> 2 << 2, y: 0, z: z >> 2 << 2 };
            let b = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp);
            *biome_hist.entry(b.clone()).or_insert(0) += 1;
            if (b.contains("badlands") || b.contains("desert")) && found.len() < 10 {
                found.push((cx, cz, b.clone()));
            }
            if b.contains("badlands") && badlands_found.len() < 10 {
                badlands_found.push((cx, cz, b.clone()));
            }
        }
    }
    println!("biome_scan ±256 chunk (4096x4096 blocks), step 8 chunk:");
    println!("  badlands/desert found: {}", found.len());
    for (cx, cz, b) in &found {
        println!("  chunk({},{}) biome={}", cx, cz, b);
    }
    println!("  badlands (non-desert) found: {}", badlands_found.len());
    for (cx, cz, b) in &badlands_found {
        println!("  BADLANDS chunk({},{}) biome={}", cx, cz, b);
    }
    println!("  biome histogram (top):");
    let mut items: Vec<_> = biome_hist.iter().collect();
    items.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (b, c) in items.iter().take(10) {
        println!("    {} = {}", b, c);
    }
}
