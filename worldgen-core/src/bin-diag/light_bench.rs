// 诊断基准：light_compute 内核单 chunk 批 wall（bin-diag 隔离区，不参与默认构建）
// 用法：rustc 单编或临时挪入 src/bin；合成 3×3 数据（地表+4 光源），输出 per-chunk µs
use WorldgenRust::light::{light_compute, LightEngine};

fn main() {
    let data_path = std::env::var("LIGHT_DATA")
        .unwrap_or_else(|_| "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json".into());
    let engine = LightEngine::from_json_file(&data_path).expect("light data");
    let mut blocks9 = vec![0i32; 884736];
    for c in 0..9usize {
        let base = c * 98304;
        for y in 0..384usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    let idx = base + y * 256 + z * 16 + x;
                    blocks9[idx] = if y < 64 { 1 } else { 0 }; // y=-64..-1 实心（raw id 1=stone）
                }
            }
        }
        for (x, z, y) in [(4usize, 4usize, 80usize), (11, 4, 80), (4, 11, 80), (11, 11, 80)] {
            let idx = base + (y + 64) * 256 + z * 16 + x;
            blocks9[idx] = (15i32 << 24) | 0; // 高 8 位 luminance=15 光源
        }
    }
    let mut out_block = vec![0u8; 24 * 2048];
    let mut out_sky = vec![0u8; 24 * 2048];
    let mut out_flags = vec![0u8; 48];

    for _ in 0..8 {
        let _ = light_compute(&engine, &blocks9, &mut out_block, &mut out_sky, &mut out_flags);
    }
    let n = 256u32;
    let t0 = std::time::Instant::now();
    for _ in 0..n {
        let _ = light_compute(&engine, &blocks9, &mut out_block, &mut out_sky, &mut out_flags);
    }
    let wall = t0.elapsed();
    println!(
        "kernel bench: {} chunks, total {:?}, per-chunk {:?} ({:.1} us)",
        n,
        wall,
        wall / n,
        wall.as_secs_f64() * 1e6 / n as f64
    );
}
