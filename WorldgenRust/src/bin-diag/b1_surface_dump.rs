// b1_surface_dump.rs — B1 诊断（bin-diag 隔离区，260902-05）：
// Rust surface-only 输出 dump（fill_chunk_blocks + SKIP_CARVER/SKIP_FEATURES），
// 与 vanilla SURFACE 口径参照（BlockProbe 默认口径，ChunkStatus.SURFACE）对拍。
// 输出：WGB2 兼容布局（header 32B + per chunk [wx,wz][u16 LE * 65536][256 biome utf 占位 0]）。
// 用法（主会话执行；bin-diag 不参与默认构建）：
//   挪 src/bin 编译；运行：
//   b1_surface_dump.exe > E:\PYTHON\CoreSwap\.tmp\b1-surface-rust.bin
// ⚠️ 未编译验证：主会话负责编译运行。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use std::io::Write;

fn main() {
    // 诊断确定性：跳过 carver/features（保留 surface）——走 env 门控（fill_chunk_blocks 兼容行为）
    unsafe {
        for k in ["WG_TRANSPILER", "WG_SKIP_SURFACE", "WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN"] {
            std::env::remove_var(k);
        }
        std::env::set_var("WG_SKIP_CARVER", "1");
        std::env::set_var("WG_SKIP_FEATURES", "1");
    }
    let seed: i64 = 8576294172403134396;
    let wg_dir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create_for_dim failed"); return; }
    };
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    let _ = w.write_all(&0x57474232u32.to_be_bytes());
    let _ = w.write_all(&seed.to_be_bytes());
    let _ = w.write_all(&4i32.to_be_bytes());
    let _ = w.write_all(&3200i32.to_be_bytes());
    let _ = w.write_all(&3208i32.to_be_bytes());
    let _ = w.write_all(&0i32.to_be_bytes());
    let _ = w.write_all(&256i32.to_be_bytes());
    for cz in 200i32..204 {
        for cx in 200i32..204 {
            let _ = w.write_all(&cx.to_be_bytes());
            let _ = w.write_all(&cz.to_be_bytes());
            let blocks = h.fill_chunk_blocks(cx, cz);
            let mut buf = Vec::with_capacity(65536 * 2);
            for id in &blocks {
                buf.extend_from_slice(&(*id as u16).to_le_bytes());
            }
            let _ = w.write_all(&buf);
            // biome 段：256 个 u16 len=0 占位（对拍脚本按布局跳过）
            for _ in 0..256 {
                let _ = w.write_all(&0u16.to_be_bytes());
            }
        }
    }
    let _ = w.flush();
    eprintln!("[B1-SURFACE-DUMP] done");
}
