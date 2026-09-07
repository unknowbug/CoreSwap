// biome_cost_bench.rs — biome 查询单价微测（260908-02，GPU 重勘探 todo 4）
// 动机：scout-a 盘点显示 biome 在所有 stage bench 均无计时行（藏在 density 底座）；
//       GPU 重勘探 J8 判据需要 biome 占比数据。
// 方法：WorldgenHandle::biome_sample 直采 quart 网格（4×96×4=1536 点/chunk × 64 chunks），
//       median ms/chunk + per-sample µs。口径（§9.7）：含 String 分配（微，随行声明）；
//       与 qpd1_stage_bench 同 seed/region；「生产 fill 实际查询数」另查，本 bench 只给单价×1536 折算。
// bench 纪律（workflow #28）：严格串行单线程。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);
const SIZE: i32 = 8;

fn median(v: &mut Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

fn main() {
    println!("=== biome_cost_bench (260908-02) seed={} region=({},{}) size={} ===", SEED, ORIGIN.0, ORIGIN.1, SIZE);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    // warmup 区外
    for i in 0..8 { let _ = h.biome_sample(4000 + i * 16, 64, 4000); }

    // 采样布局：每 chunk quart 网格 x∈{0,4,8,12} z∈{0,4,8,12} y∈{-64..320 步 4}（96 层）
    let mut per_chunk: Vec<f64> = Vec::with_capacity((SIZE * SIZE) as usize);
    let mut sink: u64 = 0;
    let mut per_sample_us: Vec<f64> = Vec::new();
    for cz in 0..SIZE {
        for cx in 0..SIZE {
            let bx = (ORIGIN.0 + cx) * 16;
            let bz = (ORIGIN.1 + cz) * 16;
            let t = Instant::now();
            for y in (-64..320).step_by(4) {
                for lx in (0..16).step_by(4) {
                    for lz in (0..16).step_by(4) {
                        let s = h.biome_sample(bx + lx, y, bz + lz);
                        sink = sink.wrapping_add(s.len() as u64);
                    }
                }
            }
            let dt = t.elapsed().as_secs_f64() * 1e3;
            per_chunk.push(dt);
            per_sample_us.push(dt * 1000.0 / 1536.0);
        }
    }
    let m = median(&mut per_chunk);
    let mus = median(&mut per_sample_us);
    println!("[BIOME-COST] per-chunk(1536 quart samples) median={:.3}ms  per-sample median={:.3}us  sink={}", m, mus, sink);
    println!("[CHECK] samples/chunk=4x96x4=1536; 64 chunks; 串行单线程; String 分配含入（口径随行声明）");
    println!("=== done ===");
}
