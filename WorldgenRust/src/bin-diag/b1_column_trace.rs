// b1_column_trace.rs — B1 诊断：seed B 参照 vs Rust fill_chunk_blocks 指定列逐层对比。
// 用法: cargo run --release --bin b1_column_trace
// env: WG_B1_COLS="cx,cz,localx,localz;..."（缺省取 biome 分桶发现的代表列）
//      WG_B1_YMAX=裁剪顶部输出（缺省打印全列非空段）
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

struct Col { cx: i32, cz: i32, lx: i32, lz: i32 }

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let ref_path = "E:\\PYTHON\\CoreSwap\\.tmp-coreswap-data\\vanilla_8576294172403134396_4_3200_3208_nether.blocks";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim failed"); return; }
    };
    println!("handle: min_y={} height={}", h.min_y, h.height);
    let bd = std::fs::read(ref_path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let ox = be32(&bd, &mut i); let oz = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("[ref] magic=0x{:X} seed={} size={} origin=({},{}) min_y={} height={}", magic, vseed, size, ox, oz, min_y, height);
    let bpc = (256 * height) as usize;
    let cols: Vec<Col> = match std::env::var("WG_B1_COLS") {
        Ok(s) => s.split(';').filter_map(|p| {
            let v: Vec<i32> = p.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if v.len() == 4 { Some(Col { cx: v[0], cz: v[1], lx: v[2], lz: v[3] }) } else { None }
        }).collect(),
        Err(_) => vec![
            Col { cx: 200, cz: 200, lx: 7, lz: 0 },
            Col { cx: 200, cz: 200, lx: 11, lz: 1 },
            Col { cx: 200, cz: 200, lx: 12, lz: 2 },
            Col { cx: 200, cz: 200, lx: 0, lz: 2 },
        ],
    };
    let ymax_cut: i32 = std::env::var("WG_B1_YMAX").ok().and_then(|s| s.parse().ok()).unwrap_or(height);

    // 读参照到 map: (cx,cz) -> Vec<i32>
    let mut refs: std::collections::HashMap<(i32, i32), Vec<i32>> = std::collections::HashMap::new();
    for _c in 0..(size * size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl > 0 { i += bl; } }
        refs.insert((cx, cz), vanilla);
    }

    // 需要的 chunk 集合
    let mut need: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for c in &cols { need.insert((c.cx, c.cz)); }
    let mut rusts: std::collections::HashMap<(i32, i32), Vec<i32>> = std::collections::HashMap::new();
    for key in &need {
        let blocks = h.fill_chunk_blocks(key.0, key.1);
        rusts.insert(*key, blocks);
    }

    for c in &cols {
        let van = &refs[&(c.cx, c.cz)];
        let rus = &rusts[&(c.cx, c.cz)];
        let key = (c.lx, c.lz);
        println!("\n=== column chunk({},{}) local({},{}) block({},{}) ===",
            c.cx, c.cz, c.lx, c.lz, c.cx * 16 + c.lx, c.cz * 16 + c.lz);
        let mut shown = 0;
        for yy in 0..ymax_cut.min(height) {
            let k = yy as usize * 256 + (c.lz * 16 + c.lx) as usize;
            let v = van[k]; let r = rus[k];
            if v == 0 && r == 0 { continue; } // 双空不打印
            let mark = if v == r { ' ' } else { '<' };
            println!("  y={:>3}  van={:>4}  rust={:>4} {}", min_y + yy, v, r, mark);
            shown += 1;
            if shown > 400 { println!("  ...(truncated)"); break; }
        }
        let _ = key;
    }
}
