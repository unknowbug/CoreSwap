// b1_noiseonly_dump.rs — B1 候选 (d) 干净判别 dump（bin-diag 隔离区，260902-09）：
// Rust noise-only 逐列材质 dump：fill_chunk_blocks + SKIP_SURFACE/CARVER/FEATURES 全开
// = steps 1-2（noise + aquifer/ore_vein），与 vanilla PRE（buildSurface HEAD = NOISE 产物）同阶段。
// 输出 CSV：`NOISEONLY,wx,wz,mat=<y255..0 自顶向下 raw id 逗号串>`，供 compare_air_pockets_noiseread.py 对拍。
// 用法（主会话执行；bin-diag 不参与默认构建）：挪 src/bin 编译运行，或 rustc 单编。
// ⚠️ chunk 坐标用 3200..3204 / 3208..3212（E-B1-12 教训：勿用 block/错误基准 200）。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use std::io::Write;

fn main() {
    unsafe {
        // noise-only：三阶段全跳（env 门控，fill_chunk_blocks 兼容行为）
        std::env::set_var("WG_SKIP_SURFACE", "1");
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
    let out_path = "E:/PYTHON/CoreSwap/.tmp/noiseonly-rust-c3200-3211.csv";
    let mut w = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    for cz in 3208i32..3212 {
        for cx in 3200i32..3204 {
            let blocks = h.fill_chunk_blocks(cx, cz);
            // blocks 布局：index = y*256 + z_local*16 + x_local（与 WGB2 参照一致，y 从 0 起）
            for z_local in 0..16i32 {
                for x_local in 0..16i32 {
                    let wx = cx * 16 + x_local;
                    let wz = cz * 16 + z_local;
                    let mut line = format!("NOISEONLY,{},{},mat=", wx, wz);
                    // 自顶向下 y=255..0
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
        eprintln!("[NOISEONLY-DUMP] cz={} done", cz);
    }
    eprintln!("[NOISEONLY-DUMP] done -> {}", out_path);
}
