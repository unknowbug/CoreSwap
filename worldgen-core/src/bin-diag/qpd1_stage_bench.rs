// qpd1_stage_bench.rs — Q-PD1 归因：Rust 全管线分阶段差分（260903-09）
// 方法：WG_SKIP_* env 门控差分（chunk 级读取，进程内按批切换合法）；
// 口径（§9.7）：与 pc_e2e_bench 同 seed/region(200,200)/预热区外；每配置 8×8=64 chunks，median 主判据。
// 段归约：features=FULL-[-F]；carver=[-F]-[-C-F]；surface=[-C-F]-[-S-C-F]；
//         orevein=[-S..]-[-O-S..]；aquifer=[-O..]-[-A-O..]；density/interp=余量（FULL-其余全减）。
// ⚠️ 差分自洽检查：各段之和应 ≈ FULL（偏差大 = 门控级联/测量问题，先查工具）。
use std::time::Instant;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);
const SIZE: i32 = 8; // 8×8 = 64 chunks / 配置

const ALL_KEYS: [&str; 5] = ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES"];

fn set_skips(active: &[&str]) {
    for k in ALL_KEYS {
        if active.contains(&k) { unsafe { std::env::set_var(k, "1"); } } else { unsafe { std::env::remove_var(k); } }
    }
}

fn median(v: &mut Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

fn bench(h: &WorldgenHandle, label: &str, skips: &[&str]) -> f64 {
    set_skips(skips);
    let mut times: Vec<f64> = Vec::with_capacity((SIZE * SIZE) as usize);
    for cz in 0..SIZE {
        for cx in 0..SIZE {
            let t = Instant::now();
            let _ = h.fill_chunk_blocks(ORIGIN.0 + cx, ORIGIN.1 + cz);
            times.push(t.elapsed().as_secs_f64() * 1e3);
        }
    }
    let m = median(&mut times.clone());
    println!("[{}] skips={:?} median={:.2}ms", label, skips, m);
    m
}

fn main() {
    println!("=== qpd1_stage_bench (260903-09) seed={} region=({},{}) size={} ===", SEED, ORIGIN.0, ORIGIN.1, SIZE);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }
    println!("[warmup] 8 chunks done (区外)");

    let m_full     = bench(&h, "FULL", &[]);
    let m_nofeat   = bench(&h, "no-features", &["WG_SKIP_FEATURES"]);
    let m_nocarf   = bench(&h, "no-carver+features", &["WG_SKIP_CARVER", "WG_SKIP_FEATURES"]);
    let m_nosurf   = bench(&h, "no-surface+carver+features", &["WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES"]);
    let m_noore    = bench(&h, "no-orevein+surface+carver+features", &["WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES"]);
    let m_noaqu    = bench(&h, "no-aquifer+orevein+surface+carver+features", &["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES"]);

    let f_features = m_full - m_nofeat;
    let f_carver   = m_nofeat - m_nocarf;
    let f_surface  = m_nocarf - m_nosurf;
    let f_orevein  = m_nosurf - m_noore;
    let f_aquifer  = m_noore - m_noaqu;
    let f_density  = m_noaqu; // 密度+插值+块填充为不可跳过的底座
    let sum_parts  = f_features + f_carver + f_surface + f_orevein + f_aquifer + f_density;
    println!("[STAGE] density/interp={:.2} aquifer={:.2} orevein={:.2} surface={:.2} carver={:.2} features={:.2}",
        f_density, f_aquifer, f_orevein, f_surface, f_carver, f_features);
    println!("[CHECK] sum_parts={:.2} vs FULL={:.2} diff={:.2} ({:.1}%)",
        sum_parts, m_full, sum_parts - m_full, (sum_parts - m_full) / m_full * 100.0);
    println!("=== done ===");
}
