// densityprofile.rs — 分解 density+aquifer 的 614ms：纯树求值 vs aquifer.classify。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
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
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).unwrap());
    let barrier: Arc<DensityFunction> = Arc::new(db.build_node(router.get("barrier").unwrap()).unwrap());
    let flooded: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_floodedness").unwrap()).unwrap());
    let spread: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_spread").unwrap()).unwrap());
    let lava: Arc<DensityFunction> = Arc::new(db.build_node(router.get("lava").unwrap()).unwrap());
    let erosion: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let depth: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let init: Arc<DensityFunction> = Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).unwrap());
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();
    let n_chunks = 8usize;
    // 预热（建 grid）
    for c in 0..2i32 { let _ = tree.sample(&NoisePos{x:c*16, y:0, z:0}); }
    // A: 纯树求值（逐列自顶向下，同 fill 顺序），无 aquifer
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for c in 0..n_chunks {
        let cx = c as i32 * 16;
        for lz in 0..16 { for lx in 0..16 {
            let x = cx+lx; let z = lz;
            for ly in (0..384).rev() {
                let y = -64 + ly;
                acc += tree.sample(&NoisePos{x,y,z});
            }
        }}
    }
    let t_tree = t0.elapsed().as_secs_f64()*1000.0/n_chunks as f64;
    std::hint::black_box(acc);
    // B: 树求值 + aquifer.classify（同 fill）
    let t1 = Instant::now();
    let mut acc2 = 0i64;
    for c in 0..n_chunks {
        let cx = c as i32;
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, 0, -64, 384i32);
        for lz in 0..16 { for lx in 0..16 {
            let x = cx*16+lx; let z = lz;
            for ly in (0..384).rev() {
                let y = -64 + ly;
                let d = tree.sample(&NoisePos{x,y,z});
                let kind = aq.apply(x, y, z, d);
                acc2 += kind as i64;
            }
        }}
    }
    let t_full = t1.elapsed().as_secs_f64()*1000.0/n_chunks as f64;
    std::hint::black_box(acc2);
    println!("per-chunk: 纯树求值(98304 pt) = {:.1}ms ({:.2}us/pt) | +aquifer = {:.1}ms | aquifer cost = {:.1}ms",
        t_tree, t_tree*1000.0/98304.0, t_full, t_full - t_tree);
}