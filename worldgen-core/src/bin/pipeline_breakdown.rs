// pipeline_breakdown.rs — 完整管线组件量化（每阶段增量）。
// 用 env 开关跳过 ore_vein/carver/features，对比各阶段耗时增量。
// 注意：env 开关需在进程启动时设置——用 std::process 多次跑或分多次运行。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn bench(h: &WorldgenHandle, label: &str) {
    let cxs: Vec<i32> = (0..8).map(|i| -288 + i % 4).collect();
    let czs: Vec<i32> = (0..8).map(|i| -256 + i / 4).collect();
    let t0 = Instant::now();
    for i in 0..8 { let _ = h.fill_chunk_blocks(cxs[i], czs[i]); }
    let dt = t0.elapsed().as_secs_f64();
    println!("{}: per-chunk={:.2}ms", label, dt * 1e3 / 8.0);
}

fn main() {
    // 打印当前 env 开关状态
    let skip_orevein = std::env::var("WG_SKIP_OREVEIN").is_ok();
    let skip_carver = std::env::var("WG_SKIP_CARVER").is_ok();
    let skip_features = std::env::var("WG_SKIP_FEATURES").is_ok();
    println!("env: skip_orevein={} skip_carver={} skip_features={}", skip_orevein, skip_carver, skip_features);

    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(-8248318472910187742, wg_dir).expect("create");
    let _ = h.fill_chunk_blocks(-18, -16); // 预热
    bench(&h, "当前配置");
}
