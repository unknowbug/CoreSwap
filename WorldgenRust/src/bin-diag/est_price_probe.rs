// est_price_probe.rs — P2.4 决定性微探针（260903-12）：est 扫描单价 hot vs cold（#21 调用形态复刻）。
// 自持 init DF 树（对齐 mt_fill.rs build_ctx 形态），对任意 (x,z) 做 off 臂同款扫描
//（min_y..min_y+384 step -8，sample > 0.390625，计迭代数）。
// hot 模式: 单列重复 2000 次（微测旧形态——缓存全热）
// cold 模式: 大 region 顺序不同列各扫一次（生产形态——每列新 key）
// 输出: ns/iter（wall / 总迭代数）。§9.7: 载体=本探针自身扫描循环；只比较形态差异，不直接外推生产。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};

const SEED: u64 = 8576294172403134396u64;

fn build_init(seed: u64) -> Arc<DensityFunction> {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let mut db = DensityBuilder::new(seed, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = std::path::PathBuf::from(format!("{}\\{}.json", df_dir, name));
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = WorldgenRust::json::parse(&std::fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).unwrap())
}

fn scan(init: &DensityFunction, x: i32, z: i32) -> (i32, u32) {
    let mut est = i32::MAX;
    let mut iters = 0u32;
    let mut y = -64 + 384;
    while y >= -64 {
        iters += 1;
        if init.sample(&NoisePos { x, y, z }) > 0.390625 { est = y; break; }
        y -= 8;
    }
    (est, iters)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("hot");
    let init = build_init(SEED);
    println!("=== est_price_probe mode={} seed={} ===", mode, SEED);
    let _ = scan(&init, 3200, 3200);
    let t0 = Instant::now();
    let mut total_iters = 0u64;
    match mode {
        "hot" => {
            for _ in 0..2000 { let (_, it) = scan(&init, 3200, 3200); total_iters += it as u64; }
        }
        "cold" => {
            for cz in 0..64 { for cx in 0..64 {
                let bx = 200 * 16 + cx * 16;
                let bz = 200 * 16 + cz * 16;
                for &(dx, dz) in &[(0i32, 0i32), (15, 0), (0, 15), (15, 15)] {
                    let (_, it) = scan(&init, bx + dx, bz + dz);
                    total_iters += it as u64;
                }
            }}
        }
        _ => panic!("mode: hot|cold"),
    }
    let wall = t0.elapsed().as_secs_f64();
    println!("[price] total_iters={} wall={:.3}s ns/iter={:.0}", total_iters, wall, wall * 1e9 / total_iters as f64);
}
