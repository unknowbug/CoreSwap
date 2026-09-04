// biome_breakdown.rs — 测 biome_of 的 6 个 density 各自采样成本（定位热点）。
// biome_hot 显示 biome_of 热缓存 53us/pt。分解 6 参数采样成本。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
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
    let names = ["temperature", "vegetation", "continents", "erosion", "depth", "ridges"];
    let fns: Vec<Arc<DensityFunction>> = names.iter().map(|n| Arc::new(db.build_node(router.get(*n).unwrap()).unwrap())).collect();

    // 预热
    let bp = NoisePos { x: 0, y: 0, z: 0 };
    for f in &fns { for _ in 0..100 { let _ = f.sample(&bp); } }

    // 测每个 density 采样成本（热缓存，同 chunk 内 256 列）
    let iters = 10000usize;
    for (i, f) in fns.iter().enumerate() {
        let t0 = Instant::now();
        let mut acc = 0.0f64;
        for k in 0..iters {
            let lx = (k % 16) as i32; let lz = ((k / 16) % 16) as i32;
            let bp = NoisePos { x: lx >> 2 << 2, y: 0, z: lz >> 2 << 2 };
            acc += f.sample(&bp);
        }
        let t = t0.elapsed().as_secs_f64()*1e6/iters as f64;
        std::hint::black_box(acc);
        println!("  {:<14} {:.3} us/pt", names[i], t);
    }
    println!("biome_breakdown done (hotspot = highest us/pt)");
}
