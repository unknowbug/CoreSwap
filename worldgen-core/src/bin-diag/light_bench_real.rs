// 诊断基准（真实数据口径）：light_compute 内核批 wall（bin-diag 隔离区）
// 输入 = blocks9_real.bin（vanilla WGB2 4x4@200 抽 3x3 转换，见 convert_blocks9.py）
// 用法：rustc 单编；env LIGHT_BLOCKS=blocks9_real.bin；输出 per-chunk µs
use WorldgenRust::light::{light_compute, LightEngine};

fn main() {
    let data_path = std::env::var("LIGHT_DATA")
        .unwrap_or_else(|_| "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json".into());
    let blocks_path = std::env::var("LIGHT_BLOCKS")
        .unwrap_or_else(|_| "E:/PYTHON/CoreSwap/.tmp/d3-opt-260905-04/blocks9_real.bin".into());
    let engine = LightEngine::from_json_file(&data_path).expect("light data");

    let raw = std::fs::read(&blocks_path).expect("blocks9_real.bin");
    let seed = i64::from_le_bytes(raw[0..8].try_into().unwrap());
    let ox = i32::from_le_bytes(raw[8..12].try_into().unwrap());
    let oz = i32::from_le_bytes(raw[12..16].try_into().unwrap());
    let nchunks = i32::from_le_bytes(raw[16..20].try_into().unwrap()) as usize;
    assert_eq!(nchunks, 9, "expect 3x3");
    let blocks9: Vec<i32> = raw[20..]
        .chunks_exact(4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    assert_eq!(blocks9.len(), 884736, "blocks9 len");
    eprintln!(
        "real-data bench: seed={seed} origin=({ox},{oz}) chunks={nchunks} len={}",
        blocks9.len()
    );
    // sanity：非全零、有非空气
    let nonzero = blocks9.iter().filter(|&&v| v != 0).count();
    eprintln!("sanity: nonzero cells = {nonzero}/{}", blocks9.len());
    assert!(nonzero > 100_000, "suspicious empty data");

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
        "kernel bench REAL: {n} chunks, total {wall:?}, per-chunk {:?} ({:.1} us)",
        wall / n,
        wall.as_secs_f64() * 1e6 / n as f64
    );
}
