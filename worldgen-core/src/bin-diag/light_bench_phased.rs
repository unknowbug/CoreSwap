// 诊断：phase 级分解（C2）——light_compute_phased 各阶段耗时（单次 + 批均值）
use WorldgenRust::light::{light_compute_phased, LightEngine};

fn main() {
    let data_path = std::env::var("LIGHT_DATA")
        .unwrap_or_else(|_| "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json".into());
    let engine = LightEngine::from_json_file(&data_path).expect("light data");
    for arg in std::env::args().skip(1) {
        let raw = std::fs::read(&arg).expect("bin");
        let blocks9: Vec<i32> = raw[20..]
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        assert_eq!(blocks9.len(), 884736);
        let mut ob = vec![0u8; 49152];
        let mut os = vec![0u8; 49152];
        let mut of = vec![0u8; 48];
        // 预热 8 + 计时批 64，每调用取 phase 累计
        for _ in 0..8 {
            let _ = light_compute_phased(&engine, &blocks9, &mut ob, &mut os, &mut of);
        }
        let n = 64u32;
        let mut acc = [std::time::Duration::ZERO; 5];
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            let t = light_compute_phased(&engine, &blocks9, &mut ob, &mut os, &mut of).unwrap();
            for k in 0..5 {
                acc[k] += t.0[k];
            }
        }
        let wall = t0.elapsed();
        let names = ["fill", "block_bfs", "sky_fall", "sky_seed_bfs", "export"];
        let name = std::path::Path::new(&arg).file_name().unwrap().to_string_lossy();
        println!("PHASED {name}: wall/chunk {:.2} ms", wall.as_secs_f64() * 1e3 / n as f64);
        for k in 0..5 {
            let us = acc[k].as_secs_f64() * 1e6 / n as f64;
            println!("  {:<13} {:8.1} us ({:4.1}%)", names[k], us, us * 100.0 / (wall.as_secs_f64() * 1e6 / n as f64));
        }
    }
}
