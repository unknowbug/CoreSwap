// puretree.rs — 决定性实验：纯树(sloped_cheese, 无缓存节点) 在 1 chunk vs 8 chunk 的 per-sample 成本。
// 若纯树 8 chunk 也慢 → 问题在遍历本身；若纯树快 → 问题在 finalDensity 的缓存节点。
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
    let sc = db.resolve_ref("minecraft:overworld/sloped_cheese");
    // 预热
    for _ in 0..100 { let _ = sc.sample(&NoisePos{x:0,y:0,z:0}); }
    // 1 chunk (0,0) 逐列
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for lz in 0..16 { for lx in 0..16 {
        let x = lx; let z = lz;
        for ly in (0..384).rev() { acc += sc.sample(&NoisePos{x,y:-64+ly,z}); }
    }}
    let t1 = t0.elapsed().as_secs_f64()*1e6/98304.0;
    std::hint::black_box(acc);
    // 8 chunk 逐列
    let t2 = Instant::now();
    let mut acc2 = 0.0f64;
    for c in 0..8i32 {
        let cx = c*16;
        for lz in 0..16 { for lx in 0..16 {
            let x = cx+lx; let z = lz;
            for ly in (0..384).rev() { acc2 += sc.sample(&NoisePos{x,y:-64+ly,z}); }
        }}
    }
    let t8 = t2.elapsed().as_secs_f64()*1e6/(8*98304) as f64;
    std::hint::black_box(acc2);
    println!("sloped_cheese(纯树,无缓存): 1chunk={:.2}us/pt  8chunk={:.2}us/pt  ratio={:.1}x", t1, t8, t8/t1);
}