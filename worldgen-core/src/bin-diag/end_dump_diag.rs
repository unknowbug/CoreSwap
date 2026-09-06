// end_dump_diag.rs — Rust 侧确定性区域 dump（IDK7 格式，对齐历史 dump 载体）
// 用法：end_dump_diag <seed> <wg_dir> <cx0> <cx1> <cz0> <cz1> <out.bin>
use WorldgenRust::worldgen_handle::WorldgenHandle;
use std::io::Write;

fn be32(v: i32, out: &mut Vec<u8>) { out.extend_from_slice(&v.to_be_bytes()); }
fn be64(v: i64, out: &mut Vec<u8>) { out.extend_from_slice(&v.to_be_bytes()); }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args[1].parse().unwrap();
    let wg_dir = &args[2];
    let (cx0, cx1, cz0, cz1): (i32, i32, i32, i32) =
        (args[3].parse().unwrap(), args[4].parse().unwrap(), args[5].parse().unwrap(), args[6].parse().unwrap());
    let out_path = &args[7];
    let height: i32 = args.get(8).map(|s| s.parse().unwrap()).unwrap_or(128);

    let h = WorldgenHandle::create_for_dim(seed, wg_dir, "end.json", "", height)
        .expect("create_for_dim end failed");
    let mut out = Vec::new();
    be32(0x49444B37, &mut out); be64(seed, &mut out);
    be32(cx0, &mut out); be32(cx1, &mut out); be32(cz0, &mut out); be32(cz1, &mut out);
    be32(h.min_y, &mut out); be32(height, &mut out);
    let bpc = (16 * 16 * height) as usize;
    for cx in cx0..=cx1 {
        for cz in cz0..=cz1 {
            let blocks = h.fill_chunk_blocks(cx, cz);
            assert_eq!(blocks.len(), bpc, "chunk size");
            be32(cx, &mut out); be32(cz, &mut out);
            for b in &blocks { out.extend_from_slice(&(*b as i16).to_be_bytes()); }
        }
    }
    let mut f = std::fs::File::create(out_path).unwrap();
    f.write_all(&out).unwrap();
    println!("[OK] dumped {} chunks -> {}", ((cx1 - cx0 + 1) * (cz1 - cz0 + 1)), out_path);
}
