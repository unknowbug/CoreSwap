// transpiler_prod_blocks.rs — 验证 transpiler 接入生产：对比 WG_TRANSPILER vs 非 transpiler 的 fill_chunk_blocks 块输出。
// 用两个 WorldgenHandle（一个 WG_TRANSPILER，一个非），跳过 carver，对比块输出。
// 验证：transpiler 接入生产后块输出与 DensityMacroSampler 一致（对齐）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    // 跳过 carver + features（carver/features 随机数溢出是已知问题，与 transpiler 无关）
    unsafe { std::env::set_var("WG_SKIP_CARVER", "1"); std::env::set_var("WG_SKIP_FEATURES", "1"); }

    // 非 transpiler handle（DensityMacroSampler）
    unsafe { std::env::remove_var("WG_TRANSPILER"); }
    let h_ms = WorldgenHandle::create(seed, wg_dir).expect("create handle (ms)");
    // transpiler handle（TranspilerDensity）
    unsafe { std::env::set_var("WG_TRANSPILER", "1"); }
    let h_td = WorldgenHandle::create(seed, wg_dir).expect("create handle (td)");

    // 对比多个 chunk 的块输出
    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    for cz in -256..-252 {
        for cx in -288..-284 {
            let blocks_ms = h_ms.fill_chunk_blocks(cx, cz);
            let blocks_td = h_td.fill_chunk_blocks(cx, cz);
            for k in 0..blocks_ms.len() {
                total += 1;
                if blocks_ms[k] != 0 { tnair += 1; }
                if blocks_td[k] == blocks_ms[k] { match_t += 1; if blocks_ms[k] != 0 { mnair += 1; } }
            }
        }
    }
    println!("TranspilerDensity vs DensityMacroSampler blocks: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
}
