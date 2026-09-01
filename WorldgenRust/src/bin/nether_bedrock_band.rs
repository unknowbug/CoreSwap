// nether_bedrock_band.rs — bedrock 随机带残差诊断：per-y vanilla/rust bedrock 计数 + 失配方向（seed -8248, 4x4 @0,0）
// 用法: cargo run --release --bin nether_bedrock_band
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] nether create_for_dim failed"); return; }
    };
    println!("handle: min_y={} height={}", h.min_y, h.height);

    let path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\vanilla_-8248318472910187742_4_0_0_nether.blocks";
    let bd = std::fs::read(path).unwrap();
    let mut i = 0usize;
    let _magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let _ox = be32(&bd, &mut i); let _oz = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("ref: seed={} size={} min_y={} height={}", vseed, size, min_y, height);
    let bpc = (16 * 16 * height) as usize;

    const BEDROCK: i32 = 31; // blocks.json raw id（下方自校验）
    // 每 y：van bedrock 数 / rust bedrock 数 / van有rust无 / rust有van无
    let y_lo = 115i32; let y_hi = 128i32;
    let n = (y_hi - y_lo + 1) as usize;
    let mut van_b = vec![0u64; n]; let mut rust_b = vec![0u64; n];
    let mut van_only = vec![0u64; n]; let mut rust_only = vec![0u64; n];
    for _c in 0..(size * size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl > 0 { i += bl; } }
        let blocks = h.fill_chunk_blocks(cx, cz);
        for k in 0..bpc {
            let y = min_y + (k / 256) as i32;
            if y < y_lo || y > y_hi { continue; }
            let idx = (y - y_lo) as usize;
            let g = blocks[k]; let w = vanilla[k];
            let gb = g == BEDROCK; let wb = w == BEDROCK;
            if wb { van_b[idx] += 1; }
            if gb { rust_b[idx] += 1; }
            if wb && !gb { van_only[idx] += 1; }
            if gb && !wb { rust_only[idx] += 1; }
        }
    }
    println!("BEDROCK=31 自校验：请对照 blocks.json（minecraft:bedrock 的 raw id）");
    println!("y | van_bedrock | rust_bedrock | van_only | rust_only");
    for (idx, y) in (y_lo..=y_hi).enumerate() {
        println!("{} | {} | {} | {} | {}", y, van_b[idx], rust_b[idx], van_only[idx], rust_only[idx]);
    }
}
