// features_probe.rs — 验证 WorldgenHandle 的 FEATURES 阶段（矿石/装饰层）。
// 用 WorldgenHandle::create + fill_chunk_blocks（含 apply_features）生成 chunk，对比 vanilla FULL 参照。
// 验证：FEATURES 阶段开启后 match 率应提升（矿石/disk/spring 等装饰层）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");
    println!("handle created: min_y={} height={}", h.min_y, h.height);

    // 读 vanilla FULL 参照（-8248 种子 4x4 origin -288,-256，含 carver+features）
    let path = "E:\\python\\MC\\data\\vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks";
    let bd = std::fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = 16*16*height as usize;

    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    let mut feature_placed = 0u64; let mut feature_match = 0u64; // FEATURES 放置的方块是否匹配 vanilla
    for _c in 0..(size*size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl>0 { i += bl; } }

        // WorldgenHandle 块级管线（含 FEATURES）
        let blocks = h.fill_chunk_blocks(cx, cz);
        for k in 0..bpc {
            let got = blocks[k];
            total += 1;
            if vanilla[k] != 0 { tnair += 1; }
            if got == vanilla[k] { match_t += 1; if vanilla[k] != 0 { mnair += 1; } }
        }
    }
    println!("WorldgenHandle(+features) vs vanilla FULL: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
}
