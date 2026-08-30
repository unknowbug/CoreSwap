// multiworld_nether_blocks.rs — Phase A：Rust nether 块级验证（fill_chunk_blocks vs vanilla nether 参照）。
// 参照：E:\PYTHON\MC\data\vanilla_-8248318472910187742_4_0_0_nether.blocks（WGB2 大端，size 4, origin 0,0, min_y 0, height 256）。
// 输出：match% / nonAir match% / 每 32 层带 match% / 首 10 个 mismatch。
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

    let path = "E:\\PYTHON\\MC\\data\\vanilla_-8248318472910187742_4_0_0_nether.blocks";
    let bd = std::fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("ref: magic=0x{:X} seed={} size={} origin=({},{}) min_y={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = (16 * 16 * height) as usize;
    if h.min_y != min_y || h.height != height { println!("[WARN] handle 维度与参照不一致"); }

    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    // 分层统计：32 层一带
    let bands = (height / 32) as usize;
    let mut band_total = vec![0u64; bands]; let mut band_match = vec![0u64; bands];
    let mut mismatches: Vec<(i32, i32, i32, i32, i32)> = Vec::new();
    for _c in 0..(size * size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl > 0 { i += bl; } }

        let blocks = h.fill_chunk_blocks(cx, cz);
        if blocks.len() != bpc { println!("[FAIL] chunk({},{}) blocks.len={} != bpc={}", cx, cz, blocks.len(), bpc); return; }
        for k in 0..bpc {
            let got = blocks[k]; let want = vanilla[k];
            total += 1;
            if want != 0 { tnair += 1; }
            let y = min_y + (k / 256) as i32;
            let band = ((y - min_y) / 32) as usize;
            if got == want {
                match_t += 1; if want != 0 { mnair += 1; }
                band_match[band] += 1;
            } else if mismatches.len() < 10 {
                let yy = (k / 256) as i32;
                let rem = (k % 256) as i32;
                let zz = rem / 16;
                let xx = rem % 16;
                mismatches.push((cx * 16 + xx, min_y + yy, cz * 16 + zz, got, want));
            }
            band_total[band] += 1;
        }
    }
    println!("TOTAL: match={}/{} ({:.4}%)  nonAir match={}/{} ({:.4}%)",
        match_t, total, 100.0 * match_t as f64 / total as f64,
        mnair, tnair, 100.0 * mnair as f64 / tnair.max(1) as f64);
    for b in 0..bands {
        let y0 = min_y + (b * 32) as i32;
        println!("  y={}..{}: {}/{} ({:.2}%)", y0, y0 + 31, band_match[b], band_total[b],
            100.0 * band_match[b] as f64 / band_total[b].max(1) as f64);
    }
    println!("first mismatches (x,y,z, got, want): {:?}", mismatches);
}
