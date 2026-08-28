// perf_fresh.rs — fresh-vs-cached 分离重测纯树（judge 建议：区分 Interpolated grid 构建 vs 稳态）。
// 目的：确认 perf_quant 的 0.05us/pt 是否被 grid 构建摊薄（4635170 已文档化的坑）。
// 测：① fresh（每 chunk 首次，grid 构建）② cached（同 chunk 重复，稳态）。
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
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    let tree = db.build_node(fd).unwrap();

    // 预热（建 grid）
    for _ in 0..100 { let _ = tree.sample(&NoisePos{x:0,y:0,z:0}); }

    // ① fresh：8 个不同 chunk 各扫一次（每 chunk 首次，grid 构建）
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for c in 0..8i32 {
        let cx = c * 16;
        for lz in 0..16 { for lx in 0..16 {
            let x = cx+lx; let z = lz;
            for ly in (0..384).rev() { acc += tree.sample(&NoisePos{x,y:-64+ly,z}); }
        }}
    }
    let t_fresh = t0.elapsed().as_secs_f64()*1e6/(8.0*98304.0);
    std::hint::black_box(acc);

    // ② cached：同 chunk(0,0) 重复 8 次（稳态，grid 已建）
    let t1 = Instant::now();
    let mut acc2 = 0.0f64;
    for _ in 0..8 {
        for lz in 0..16 { for lx in 0..16 {
            let x = lx; let z = lz;
            for ly in (0..384).rev() { acc2 += tree.sample(&NoisePos{x,y:-64+ly,z}); }
        }}
    }
    let t_cached = t1.elapsed().as_secs_f64()*1e6/(8.0*98304.0);
    std::hint::black_box(acc2);

    println!("perf_fresh (seed {}):", seed);
    println!("  fresh (8 chunk, grid build): {:.2} us/pt", t_fresh);
    println!("  cached (same chunk, steady): {:.2} us/pt", t_cached);
    println!("  fresh/cached ratio: {:.1}x", t_fresh/t_cached);
    println!("  => 若 fresh >> cached，grid 构建是热点（perf_quant 0.05 被摊薄）；若接近，稳态即快");
}
