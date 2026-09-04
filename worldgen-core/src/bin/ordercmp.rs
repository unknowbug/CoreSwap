// ordercmp.rs — 决定性实验：同一棵树、同一 chunk，两种遍历顺序的 per-sample 成本对比。
// 解开矛盾：perf_probe5(y-外层) 0.1μs/pt vs fill(逐列) 4.89μs/pt。
// 模式A = perf_probe5 的 bench_varied（y 外层，x 内层，z 最慢）
// 模式B = fill 的逐列（x 外层，y 内层，自顶向下）
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos, GRID_ARG_SAMPLES};
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

    // 模式A：perf_probe5 bench_varied（y 外层，x 内层，z 最慢），chunk(0,0)
    let (cx, cz) = (0i32, 0i32);
    let iters = 98304usize;
    // 预热
    let mut idx = 0usize;
    for _ in 0..100 { let _ = tree.sample(&NoisePos{x:cx*16+(idx%16) as i32, y:-64+((idx/16)%96) as i32*4, z:cz*16+((idx/(16*96))%16) as i32}); idx+=1; }
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for _ in 0..iters {
        let x = cx*16+(idx%16) as i32; let y = -64+((idx/16)%96) as i32*4; let z = cz*16+((idx/(16*96))%16) as i32;
        acc += tree.sample(&NoisePos{x,y,z}); idx+=1;
    }
    let tA = t0.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc);

    // 模式B：逐列（x 外层，y 内层自顶向下），chunk(0,0)
    let mut acc2 = 0.0f64;
    let t1 = Instant::now();
    for lz in 0..16 { for lx in 0..16 {
        let x = cx*16+lx; let z = cz*16+lz;
        for ly in (0..384).rev() {
            let y = -64+ly;
            acc2 += tree.sample(&NoisePos{x,y,z});
        }
    }}
    let tB = t1.elapsed().as_secs_f64()*1e6/iters as f64;
    std::hint::black_box(acc2);

    println!("模式A(y-外层, perf_probe5式): {:.2} us/pt", tA);
    println!("模式B(逐列, fill式):          {:.2} us/pt", tB);
    println!("ratio B/A = {:.1}x", tB/tA);

    // 模式C：8 chunk (0,0)-(3,3) 逐列（同 densityprofile），看多 chunk 是否导致慢
    let s0 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let mut acc3 = 0.0f64;
    let t2 = Instant::now();
    for c in 0..8i32 {
        let cx = c * 16;
        for lz in 0..16 { for lx in 0..16 {
            let x = cx+lx; let z = lz;
            for ly in (0..384).rev() {
                let y = -64+ly;
                acc3 += tree.sample(&NoisePos{x,y,z});
            }
        }}
    }
    let tC = t2.elapsed().as_secs_f64()*1e6/(8*98304) as f64;
    let s1 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    std::hint::black_box(acc3);
    println!("模式C(8 chunk 逐列):          {:.2} us/pt  (grid arg samples delta={})", tC, s1-s0);

    // 单 chunk 网格构建次数（对比）
    let s2 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let mut acc4 = 0.0f64;
    for lz in 0..16 { for lx in 0..16 {
        let x = 0*16+lx; let z = lz;
        for ly in (0..384).rev() {
            let y = -64+ly;
            acc4 += tree.sample(&NoisePos{x,y,z});
        }
    }}
    let s3 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    std::hint::black_box(acc4);
    println!("单 chunk(0,0) grid arg samples delta={} (期望 1225×N)", s3-s2);
}