// transpiler_prod_blocks.rs — 验证 transpiler 接入生产：对比 WG_TRANSPILER vs 非 transpiler 的 fill_chunk_blocks 块输出。
// 用两个 WorldgenHandle（一个 WG_TRANSPILER，一个非），跳过 carver，对比块输出。
// 验证：transpiler 接入生产后块输出与 DensityMacroSampler 一致（对齐）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    // 默认跳过 carver + features（carver/features 随机数溢出是已知问题，与 transpiler 无关）；
    // WG_FULL_MODE=1 = 不跳过（FULL 管线级联归因探针，260903-02）。
    if std::env::var("WG_FULL_MODE").as_deref() != Ok("1") {
        unsafe { std::env::set_var("WG_SKIP_CARVER", "1"); std::env::set_var("WG_SKIP_FEATURES", "1"); }
    }

    // 非 transpiler handle（DensityMacroSampler）
    unsafe { std::env::remove_var("WG_TRANSPILER"); }
    let h_ms = WorldgenHandle::create(seed, wg_dir).expect("create handle (ms)");
    // transpiler handle（TranspilerDensity）
    unsafe { std::env::set_var("WG_TRANSPILER", "1"); }
    let h_td = WorldgenHandle::create(seed, wg_dir).expect("create handle (td)");

    // 对比多个 chunk 的块输出
    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    // FULL 归因分解（260903-02）：以 vanilla FULL 参照为第三方，统计 td 相对 ms「打破/赢得」的匹配
    let full = std::env::var("WG_FULL_MODE").as_deref() == Ok("1");
    let mut broke = 0u64; let mut gained = 0u64; let mut both_wrong = 0u64;
    let van_path = "E:\\python\\MC\\data\\vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks";
    let mut van: Option<Vec<Vec<i32>>> = None;
    let mut van_idx: Option<std::collections::HashMap<(i32, i32), usize>> = None;
    if full {
        let bd = std::fs::read(van_path).expect("read vanilla FULL ref");
        fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
        fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
        fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }
        let mut i2 = 0usize;
        let magic = be32(&bd, &mut i2); let seed_r = be64(&bd, &mut i2); let size_r = be32(&bd, &mut i2);
        let ox = be32(&bd, &mut i2); let oz = be32(&bd, &mut i2); let mny = be32(&bd, &mut i2); let hgt = be32(&bd, &mut i2);
        println!("ref: magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, seed_r, size_r, ox, oz, mny, hgt);
        let bpc = 16*16*hgt as usize;
        let mut chunks = Vec::new();
        let mut order: Vec<(i32, i32)> = Vec::new();
        for _c in 0..(size_r*size_r) {
            let cx = be32(&bd, &mut i2); let cz = be32(&bd, &mut i2);
            let mut blocks = vec![0i32; bpc];
            for k in 0..bpc { blocks[k] = be16(&bd, &mut i2) as i32; }
            for _bi in 0..256 { let bl = be16(&bd, &mut i2) as usize; if bl>0 { i2 += bl; } }
            order.push((cx, cz));
            chunks.push(blocks);
        }
        van = Some(chunks);
        // 按 (cx,cz) 建索引，防文件序与生成循环序不一致错位配对（参照核对纪律）
        let mut vidx = std::collections::HashMap::new();
        for (i, &(cx, cz)) in order.iter().enumerate() { vidx.insert((cx, cz), i); }
        van_idx = Some(vidx);
        println!("ref chunk coords: {:?}", order);
    }
    // ⚠️ 260903-02 实测：.bak.blocks 参照实际坐标 = (-18..-15, -16..-13)，与 header origin (-288,-256) 不符。
    // 对比 chunk 集合以参照文件实际坐标为准（van_order），硬编码 -288/-256 循环会全 miss。
    let coords: Vec<(i32, i32)> = if full {
        van_idx.as_ref().unwrap().keys().copied().collect()
    } else {
        let mut v = Vec::new();
        for cz in -256..-252 { for cx in -288..-284 { v.push((cx, cz)); } }
        v
    };
    for &(cx, cz) in &coords {
        let blocks_ms = h_ms.fill_chunk_blocks(cx, cz);
        let blocks_td = h_td.fill_chunk_blocks(cx, cz);
        let vref = van_idx.as_ref().and_then(|ix| ix.get(&(cx, cz)).map(|&i| i))
            .map(|i| &van.as_ref().unwrap()[i]);
            for k in 0..blocks_ms.len() {
                total += 1;
                if blocks_ms[k] != 0 { tnair += 1; }
                if blocks_td[k] == blocks_ms[k] { match_t += 1; if blocks_ms[k] != 0 { mnair += 1; } }
                if let Some(v) = vref {
                    let ms_ok = blocks_ms[k] == v[k];
                    let td_ok = blocks_td[k] == v[k];
                    if blocks_td[k] != blocks_ms[k] {
                        if ms_ok && !td_ok { broke += 1; }
                        else if !ms_ok && td_ok { gained += 1; }
                        else { both_wrong += 1; }
                    }
                }
            }
    }
    println!("TranspilerDensity vs DensityMacroSampler blocks: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
    if full {
        println!("FULL cascade decomposition: td!=ms diff-blocks total={}", broke + gained + both_wrong);
        println!("  broke(ms_ok,td_bad)={} gained(ms_bad,td_ok)={} both_wrong={}  net=broke-gained={}", broke, gained, both_wrong, broke as i64 - gained as i64);
        println!("  (identity check: net must equal match_ms - match_td = 17725 from handle_probe runs)");
    }
}
