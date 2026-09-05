// bin-diag/idk7_region_dump.rs — 260905-06 T5 诊断：确定性 Rust 区域导出（idk-7 新旧代码对拍载体）
// 用法: idk7_region_dump <seed> <cx0> <cx1> <cz0> <cz1> <out.bin>
// 输出格式: header(magic u32=0x4944'K37, seed i64, cx0,cx1,cz0,cz1,min_y,height i32)
//           每chunk: cx i32, cz i32, 16*16*height 个 i16 BlockId（fill_chunk_blocks 原始布局）
// 诊断专用：不参与默认构建（bin-diag 不在 cargo 默认路径），rustc/cargo 临时单编。
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
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");
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
