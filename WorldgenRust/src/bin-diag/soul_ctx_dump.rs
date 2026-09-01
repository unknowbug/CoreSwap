// soul_ctx_dump.rs — V4 裁决性探针驱动（bin-diag 隔离区，2026-09-09）：
// 直接调用生产链 fill_chunk_blocks（fill_chunk → surface → carver → features，
// 同 wg_fill_blocks_multi 路径），让 surface_rules.rs 内的 WG_SOUL_CTX_DUMP 门控
// 在生产 ctx 上 dump soul 分支入口输入（biome/sda/sdb/surface_depth/selector/判定/apply）。
//
// 与 soul_selector_probe 的区别：probe 在外部重组 ctx（复算输入），本 bin 不复算任何输入——
// build_surface 就是生产函数，dump 的就是生产 ctx。两份输出对差 = V4 裁决（运行时输入差候选）。
//
// 用法（主会话执行；bin-diag 不参与默认构建）：
//   set WG_SOUL_CTX_DUMP=E:/PYTHON/CoreSwap/.tmp/soul-mismatch-points.txt
//   cargo run --release --bin soul_ctx_dump > E:\PYTHON\CoreSwap\.tmp\soul-ctx-dump.stdout.txt
//   （[SOUL-CTX] 行在 stderr）
//
// ⚠️ 未编译验证：本文件由主会话产出，cargo 编译验证紧随。
// 诊断确定性：明确清掉 WG_SKIP_*/WG_TRANSPILER（诊断不受开关影响，与 soul_selector_probe 同纪律）。

use std::collections::{HashMap, HashSet};

use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
const POINTS_PATH: &str = "E:/PYTHON/CoreSwap/.tmp/soul-mismatch-points.txt";
const SETTINGS: &str = "nether.json";
const BIOME_PARAMS: &str = "biome_params_nether.json";
const WORLD_HEIGHT: i32 = 256;

fn main() {
    // 确定性：诊断 bin 不允许受 WG_SKIP_*/WG_TRANSPILER 开关影响
    unsafe {
        for k in ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES", "WG_TRANSPILER"] {
            std::env::remove_var(k);
        }
    }

    // 0. 读点 → 需要生成的 chunk 集
    let txt = std::fs::read_to_string(POINTS_PATH)
        .unwrap_or_else(|e| panic!("[FAIL] cannot read {}: {}", POINTS_PATH, e));
    let mut chunks: HashSet<(i32, i32)> = HashSet::new();
    let mut n_points = 0usize;
    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut it = line.split_whitespace();
        if let (Some(a), Some(b), Some(c)) = (it.next(), it.next(), it.next()) {
            if let (Ok(x), Ok(y), Ok(z)) = (a.parse::<i32>(), b.parse::<i32>(), c.parse::<i32>()) {
                let _ = y; // y 只进 dump 门控点集（surface_rules 侧读同一文件）
                chunks.insert((x >> 4, z >> 4));
                n_points += 1;
            }
        }
    }
    let mut chunks: Vec<(i32, i32)> = chunks.into_iter().collect();
    chunks.sort();
    eprintln!("[SOUL-CTX-DRIVER] points={} chunks={}", n_points, chunks.len());
    for (cx, cz) in &chunks {
        eprintln!("[SOUL-CTX-DRIVER] chunk {},{}", cx, cz);
    }

    // 1. 生产句柄（与 soul_selector_probe 同参：create_for_dim nether）
    let h = match WorldgenHandle::create_for_dim(SEED, WG_DIR, SETTINGS, BIOME_PARAMS, WORLD_HEIGHT) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create_for_dim failed"); return; }
    };

    // 2. 逐 chunk 走生产 fill_chunk_blocks（dump 由 surface_rules.rs 门控输出到 stderr）
    let mut done: HashMap<(i32, i32), usize> = HashMap::new();
    for &(cx, cz) in &chunks {
        let blocks = h.fill_chunk_blocks(cx, cz);
        let _ = blocks; // 丢弃输出（本探针只看 stderr dump）
        done.insert((cx, cz), 0);
    }
    eprintln!("[SOUL-CTX-DRIVER] done chunks={}", done.len());
}
