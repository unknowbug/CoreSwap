// 探针轮 260914-04（光照 round3 判据定线）：内核数据形态计数器——零改生产代码，独立复算。
// 回答 b1 开放问题 1/3：非 air 格占比（lookup 内联收益面）、15-区间格数（sky_fall 融合收益面）、
// memset 实际耗时（scratch 清零 2.65MB 占 fill 份额）。
// 用法（rustc 单编，链 deps 最新 rlib——#30 陈旧缓存判据）：
//   LIGHT_BLOCKS=<blocks9_real.bin 路径> light_probe_260914.exe
use WorldgenRust::light::{light_compute, LightEngine, DOM, WORLD_H, CHUNK_CELLS};

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

    // ── 计数 1：非 air 格（v != 0，走 lookup 的格）与 lum>0 光源格 ──
    // ABI 同 fill（mod.rs:222-231）：v==0 = air 快路径；v!=0 = 落查表。
    let mut non_air = 0usize;
    let mut lum_only = 0usize; // id==0 但 lum>0（air 快路径漏掉的光源格，#15 家族语义面）
    for &v in &blocks9 {
        let u = v as u32;
        if u == 0 {
            continue;
        }
        non_air += 1;
        if (u & 0x00FF_FFFF) == 0 && (u >> 24) > 0 {
            lum_only += 1;
        }
    }
    let total = blocks9.len();
    println!("[LIGHTPROBE-K] seed={seed} origin=({ox},{oz})");
    println!(
        "[LIGHTPROBE-K] nonAir={non_air}/{total} = {:.1}%",
        non_air as f64 * 100.0 / total as f64
    );
    println!("[LIGHTPROBE-K] lumOnlyNonAirId={lum_only}");

    // ── 计数 2：15-区间格数（sky_fall 写量）——独立复算 col_max（同 fill 语义 mod.rs:241-242）──
    let mut col_max = vec![-1i32; DOM * DOM];
    for y in 0..WORLD_H {
        let yb = y * 256;
        for z in 0..DOM {
            let cz = z >> 4;
            let lz = z & 15;
            let colrow = z * DOM;
            for cx in 0..3usize {
                let c = cz * 3 + cx;
                let base = c * CHUNK_CELLS + yb + lz * 16;
                for lx in 0..16usize {
                    let u = blocks9[base + lx] as u32;
                    if u == 0 && engine.air_fast_probe() {
                        continue;
                    }
                    let id = (u & 0x00FF_FFFF) as i32;
                    let (op, _) = engine.lookup_probe(id);
                    if op > 0 {
                        col_max[colrow + cx * 16 + lx] = y as i32;
                    }
                }
            }
        }
    }
    let mut interval_cells = 0usize;
    for &m in &col_max {
        interval_cells += (WORLD_H as i32 - (m + 1)).max(0) as usize;
    }
    println!(
        "[LIGHTPROBE-K] sky15IntervalCells={interval_cells}/{total} = {:.1}%",
        interval_cells as f64 * 100.0 / total as f64
    );

    // ── 计数 3：memset 实际耗时（scratch 三块 clear+resize 语义，2.65MB）──
    // 复刻 mod.rs:195-203 的 clear+resize 形态（Vec 跨调用复用，非首次 alloc）。
    let mut m1: Vec<u8> = Vec::with_capacity(total);
    let mut m2: Vec<u8> = Vec::with_capacity(total);
    let mut m3: Vec<u8> = Vec::with_capacity(total);
    // 预热
    for _ in 0..8 {
        m1.clear(); m1.resize(total, 0);
        m2.clear(); m2.resize(total, 0);
        m3.clear(); m3.resize(total, 0);
    }
    let n = 256u32;
    let t0 = std::time::Instant::now();
    for _ in 0..n {
        m1.clear(); m1.resize(total, 0);
        m2.clear(); m2.resize(total, 0);
        m3.clear(); m3.resize(total, 0);
    }
    let wall = t0.elapsed();
    println!(
        "[LIGHTPROBE-K] memset3x884736B: {n} iters, per-iter {:.1} us ({} B)",
        wall.as_secs_f64() * 1e6 / n as f64,
        3 * total
    );

    // ── 附带：同载体内核 wall 复核（对照 12 篇 1.587ms，#30 声明可比性）──
    let mut out_block = vec![0u8; 24 * 2048];
    let mut out_sky = vec![0u8; 24 * 2048];
    let mut out_flags = vec![0u8; 48];
    for _ in 0..8 {
        let _ = light_compute(&engine, &blocks9, &mut out_block, &mut out_sky, &mut out_flags);
    }
    let t1 = std::time::Instant::now();
    for _ in 0..n {
        let _ = light_compute(&engine, &blocks9, &mut out_block, &mut out_sky, &mut out_flags);
    }
    println!(
        "[LIGHTPROBE-K] kernelWall: {n} chunks, per-chunk {:.1} us",
        t1.elapsed().as_secs_f64() * 1e6 / n as f64
    );
}
