// 诊断：golden 冻结（C4）——对多 region + 合成边界用例 dump 光照输出 hash。
// 优化前用 HEAD rlib 跑出基线；优化后用新 rlib 复跑，hash 必须逐位一致，任何不等即 FAIL。
// 用法：light_golden_dump.exe [blocks9.bin ...]（无参 = 只跑内置合成用例）
use WorldgenRust::light::{light_compute, LightEngine};

fn fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn run_case(engine: &LightEngine, name: &str, blocks9: &[i32]) {
    let mut out_block = vec![0u8; 24 * 2048];
    let mut out_sky = vec![0u8; 24 * 2048];
    let mut out_flags = vec![0u8; 48];
    light_compute(engine, blocks9, &mut out_block, &mut out_sky, &mut out_flags).expect("compute");
    println!(
        "GOLDEN {} block={:016x} sky={:016x} flags={:016x}",
        name,
        fnv(&out_block),
        fnv(&out_sky),
        fnv(&out_flags)
    );
}

fn main() {
    let data_path = std::env::var("LIGHT_DATA")
        .unwrap_or_else(|_| "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json".into());
    let engine = LightEngine::from_json_file(&data_path).expect("light data");

    // 合成边界用例（沿用 light_bench 合成场景 = 地表 + 4 光源）
    let mut syn = vec![0i32; 884736];
    for c in 0..9usize {
        let base = c * 98304;
        for y in 0..64usize {
            for i in 0..256usize {
                syn[base + y * 256 + i] = 1;
            }
        }
        for (x, z, y) in [(4usize, 4usize, 80usize), (11, 4, 80), (4, 11, 80), (11, 11, 80)] {
            syn[base + (y + 64) * 256 + z * 16 + x] = 15i32 << 24;
        }
    }
    run_case(&engine, "synthetic", &syn);

    for arg in std::env::args().skip(1) {
        let raw = std::fs::read(&arg).expect("region bin");
        let blocks9: Vec<i32> = raw[20..]
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        assert_eq!(blocks9.len(), 884736, "{} len", arg);
        let name = std::path::Path::new(&arg)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        run_case(&engine, &name, &blocks9);
    }
}
