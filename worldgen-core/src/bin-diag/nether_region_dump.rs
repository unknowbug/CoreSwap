// nether_region_dump.rs — nether 版确定性区域 dump（idk7_region_dump 的 create_for_dim nether 变体）
// 收编自 .tmp/concern2-260906/concern2_nether_dump.rs（260906-05，CONCERN-2 回归载体，知识库 #61 归位声明）
// anchor.source: .investigations/end-takeover/concern2-regression-verdict-260906-04.md 证据链 4（dump SHA 对拍）
// 用法: nether_region_dump <seed> <cx0> <cx1> <cz0> <cz1> <out.bin> [wg_dir]
//   wg_dir 缺省为 1.20.1 数据目录；nether 用 settings=nether.json / biome_params_nether.json / height=256
// 注意: bin-diag 不参与默认构建；用前 rustc 单编（AGENTS.md §八.13），本收编版未重编验证。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 { eprintln!("args: <seed> <cx0> <cx1> <cz0> <cz1> <out.bin>"); std::process::exit(2); }
    let seed: i64 = args[1].parse().expect("seed");
    let cx0: i32 = args[2].parse().expect("cx0");
    let cx1: i32 = args[3].parse().expect("cx1");
    let cz0: i32 = args[4].parse().expect("cz0");
    let cz1: i32 = args[5].parse().expect("cz1");
    let out = &args[6];
    let wg_dir = args.get(7).map(|s| s.as_str())
        .unwrap_or("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen");
    let h = WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256).expect("create_for_dim nether");
    let bpc = (16 * 16 * h.height) as usize;
    let mut f = std::io::BufWriter::new(std::fs::File::create(out).expect("create out"));
    use std::io::Write;
    f.write_all(&0x4944_4B37u32.to_be_bytes()).unwrap();
    f.write_all(&seed.to_be_bytes()).unwrap();
    for v in [cx0, cx1, cz0, cz1, h.min_y, h.height] { f.write_all(&v.to_be_bytes()).unwrap(); }
    let n = ((cx1 - cx0 + 1) * (cz1 - cz0 + 1)) as usize;
    let mut done = 0usize; let mut nonair = 0u64;
    let t0 = std::time::Instant::now();
    for cz in cz0..=cz1 {
        for cx in cx0..=cx1 {
            let blocks = h.fill_chunk_blocks(cx, cz);
            assert_eq!(blocks.len(), bpc, "chunk size");
            f.write_all(&cx.to_be_bytes()).unwrap();
            f.write_all(&cz.to_be_bytes()).unwrap();
            for b in &blocks { f.write_all(&(*b as i16).to_be_bytes()).unwrap(); nonair += (*b != 0) as u64; }
            done += 1;
            if done % 128 == 0 { eprintln!("[dump] {done}/{n} chunks {:.1}s", t0.elapsed().as_secs_f32()); }
        }
    }
    drop(f);
    println!("[dump] done {done} chunks nonair={nonair} out={out} elapsed={:.1}s", t0.elapsed().as_secs_f32());
}
