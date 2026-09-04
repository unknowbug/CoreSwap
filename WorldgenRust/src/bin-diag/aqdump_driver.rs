// aqdump_driver.rs — 残留 1830 WG_AQDUMP 探针驱动（bin-diag 隔离区，260904）：
// 全管线 fill_chunk_blocks（不 skip 任何阶段），覆盖点集所在 chunk 邻域；
// dump 输出由 aquifer.rs WG_AQDUMP 门控（点文件经 env 传入），stdout 重定向即结果。
// 用法：rustc 单编或临时挪 src/bin；WG_AQDUMP=<points> 运行。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    unsafe {
        // 诊断确定性：清掉可能影响链路的开关（保留 WG_AQDUMP 本身）
        for k in ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES", "WG_TRANSPILER", "WG_DFC", "WG_GPU_CHANNELS", "WG_GPU_DENSITY"] {
            std::env::remove_var(k);
        }
    }
    let seed: i64 = 8576294172403134396;
    let wg_dir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
    let h = match WorldgenHandle::create(seed, wg_dir) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create failed"); return; }
    };
    // 点域 x196-216 / z236-242 → chunk cx 12-13（x192-223），cz 14-15（z224-255）；
    // 邻域 margin：est/blob 13 邻域跨 ±3 chunk → 填 cx 9-16, cz 11-18 保证缓存状态一致。
    for cz in 11..=18 {
        for cx in 9..=16 {
            let _ = h.fill_chunk_blocks(cx, cz);
        }
        eprintln!("[AQDUMP-DRIVER] cz={} done", cz);
    }
    eprintln!("[AQDUMP-DRIVER] done");
}
