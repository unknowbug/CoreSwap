// biome_hot.rs — 测 biome_of 单点热缓存成本（同 chunk 内，flat_cache 命中）。
// biome_fill 显示 biome 占 45%，但 flat_cache key=chunk 应命中。确认热缓存下 biome_of 成本。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
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

    // 预热（建 flat_cache grid）
    let bp = NoisePos { x: 0, y: 0, z: 0 };
    for _ in 0..100 { let _ = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp); }

    // 热缓存：同 chunk(0,0) 内 256 列（flat_cache 命中），y=0
    let iters = 10000usize;
    let t0 = Instant::now();
    let mut acc = 0usize;
    for i in 0..iters {
        let lx = (i % 16) as i32; let lz = ((i / 16) % 16) as i32;
        let x = lx; let z = lz;
        let bp = NoisePos { x: x >> 2 << 2, y: 0, z: z >> 2 << 2 };
        let b = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp);
        acc += b.len();
    }
    let t_hot = t0.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc);

    // 地表 y（每列不同，模拟 fill_chunk）
    let t1 = Instant::now();
    let mut acc2 = 0usize;
    for i in 0..iters {
        let lx = (i % 16) as i32; let lz = ((i / 16) % 16) as i32;
        let x = lx; let z = lz;
        let y = 60 + (i % 40) as i32; // 地表 y 变化
        let bp = NoisePos { x: x >> 2 << 2, y: y >> 2 << 2, z: z >> 2 << 2 };
        let b = bc.biome_of(&t, &h, &c, &e, &d, &w, &bp);
        acc2 += b.len();
    }
    let t_surf = t1.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc2);

    println!("biome_hot (seed {}):", seed);
    println!("  biome_of 单点 y=0 (flat_cache 命中): {:.3} us/pt", t_hot);
    println!("  biome_of 单点 地表y (每列不同): {:.3} us/pt", t_surf);
    println!("  => 若 surf >> y0，y 变化导致缓存 miss（temperature/vegetation 的 ShiftedNoise 或别的）");
}
