// b1_surfaceonly_dump.rs — B1 (a/b/c) surface 层对拍 dump（bin-diag 隔离区，260902-09）：
// Rust surface-only：fill_chunk_blocks + SKIP_CARVER/SKIP_FEATURES（surface 保留），
// 与 vanilla POST（buildSurface RETURN = ChunkStatus.SURFACE 产物）同阶段对拍。
// 输出 CSV：`SURFACEONLY,wx,wz,mat=<y255..0 自顶向下 raw id 逗号串>`（Rust 侧 id 空间，对拍需 id 映射）。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use std::io::Write;

fn main() {
    unsafe {
        std::env::remove_var("WG_SKIP_SURFACE"); // surface 保留
        std::env::set_var("WG_SKIP_CARVER", "1");
        std::env::set_var("WG_SKIP_FEATURES", "1");
        std::env::remove_var("WG_SKIP_AQUIFER");
        std::env::remove_var("WG_SKIP_OREVEIN");
        std::env::remove_var("WG_TRANSPILER");
    }
    let seed: i64 = 8576294172403134396;
    let wg_dir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create_for_dim failed"); return; }
    };
    let out_path = "E:/PYTHON/CoreSwap/.tmp/surfaceonly-rust-c3200-3211.csv";
    let mut w = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    for cz in 3208i32..3212 {
        for cx in 3200i32..3204 {
            let blocks = h.fill_chunk_blocks(cx, cz);
            for z_local in 0..16i32 {
                for x_local in 0..16i32 {
                    let wx = cx * 16 + x_local;
                    let wz = cz * 16 + z_local;
                    let mut line = format!("SURFACEONLY,{},{},mat=", wx, wz);
                    for y in (0..256).rev() {
                        let id = blocks[(y as usize) * 256 + (z_local as usize) * 16 + (x_local as usize)];
                        if y < 255 { line.push(','); }
                        line.push_str(&id.to_string());
                    }
                    line.push('\n');
                    let _ = w.write_all(line.as_bytes());
                }
            }
        }
        let _ = w.flush();
        eprintln!("[SURFACEONLY-DUMP] cz={} done", cz);
    }
    eprintln!("[SURFACEONLY-DUMP] done -> {}", out_path);
}
