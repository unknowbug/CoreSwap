// qaq1_b2_ab_bench.rs — Q-AQ1 b2 判别：aquifer × carver 2×2 组合（260903-10，v2 交错版）
// v1 教训：四臂顺序执行出现物理上不可能的负交互（air 列 carver 比实地形 carver 贵 26ms），
//          根因 = 臂间机器漂移（round2 全臂 +8~13%）。v2 改为 chunk 粒度交错：每个 chunk
//          依次按固定顺序测四臂配置，漂移对所有臂同等作用，差分干净。
// 固定 WG_SKIP_OREVEIN/SURFACE/FEATURES 四臂全开，只切 A(skip_aquifer)×C(skip_carver)：
//   m00 = A on,  C on（≈ qpd1 m_noore）  m01 = A on, C off
//   m10 = A off, C on（≈ qpd1 m_noaqu）  m11 = A off, C off
// 判读：
//   A|Con = m00−m10（qpd1 口径的 aquifer 段，含级联） A|Coff = m01−m11（纯 classify aquifer 成本）
//   I = (m00−m10)−(m01−m11)（carver×aquifer 交互 = 误归因量）
//   C|Aon = m00−m01（实地形 carver 成本） C|Aoff = m10−m11（Air 列 carver 残差，应远小于 C|Aon）
// 口径（§9.7）：同 qpd1_stage_bench seed/region(200,200)/8×8=64 chunks/median，3 轮交错。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);
const SIZE: i32 = 8;
const ROUNDS: i32 = 3;

const ALL_KEYS: [&str; 5] = ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES"];

fn set_skips(a: bool, c: bool) {
    unsafe {
        if a { std::env::set_var("WG_SKIP_AQUIFER", "1"); } else { std::env::remove_var("WG_SKIP_AQUIFER"); }
        if c { std::env::set_var("WG_SKIP_CARVER", "1"); } else { std::env::remove_var("WG_SKIP_CARVER"); }
        for k in ["WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_FEATURES"] { std::env::set_var(k, "1"); }
    }
}

fn median(v: &mut Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

fn main() {
    println!("=== qaq1_b2_ab_bench v2-interleaved (260903-10) seed={} region=({},{}) size={} rounds={} ===", SEED, ORIGIN.0, ORIGIN.1, SIZE, ROUNDS);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }
    println!("[warmup] 8 chunks done (区外)");

    // 臂配置固定顺序：(A_on,C_on) (A_on,C_off) (A_off,C_on) (A_off,C_off)
    // ALL_KEYS 引用防 unused 警告（set_skips 显式列出全部键）
    let _ = ALL_KEYS;
    let arms: [(bool, bool, &str); 4] = [(false, false, "m00_AonCon"), (false, true, "m01_AonCoff"), (true, false, "m10_AoffCon"), (true, true, "m11_AoffCoff")];
    let mut samples: [Vec<f64>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let total = (SIZE * SIZE) as usize;

    for round in 1..=ROUNDS {
        for cz in 0..SIZE {
            for cx in 0..SIZE {
                for (arm, &(a, c, _)) in arms.iter().enumerate() {
                    set_skips(a, c);
                    let t = Instant::now();
                    let _ = h.fill_chunk_blocks(ORIGIN.0 + cx, ORIGIN.1 + cz);
                    samples[arm].push(t.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        let meds: Vec<f64> = samples.iter().map(|v| median(&mut v.clone())).collect();
        println!("[ROUND{}] interleaved medians: m00={:.2} m01={:.2} m10={:.2} m11={:.2} (n={})",
            round, meds[0], meds[1], meds[2], meds[3], total * round as usize);
    }

    let m: Vec<f64> = samples.iter().map(|v| median(&mut v.clone())).collect();
    let a_con = m[0] - m[2];
    let a_coff = m[1] - m[3];
    let inter = a_con - a_coff;
    let c_aon = m[0] - m[1];
    let c_aoff = m[2] - m[3];
    println!("[FINAL] m00={:.2} m01={:.2} m10={:.2} m11={:.2} (n={} per arm)", m[0], m[1], m[2], m[3], total * ROUNDS as usize);
    println!("[FINAL] A|Con={:.2}  A|Coff={:.2}  I(carver*aquifer)={:.2}", a_con, a_coff, inter);
    println!("[FINAL] C|Aon={:.2}  C|Aoff={:.2}  (物理约束: C|Aon 应 >= C|Aoff, 违反=仍有污染)", c_aon, c_aoff);
    println!("[FINAL] qpd1 口径核对: A|Con 应 ≈ 35.07 (qpd1 aquifer 段)");
    println!("=== done ===");
}
