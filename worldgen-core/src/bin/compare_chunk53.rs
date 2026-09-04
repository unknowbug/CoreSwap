// compare_chunk53.rs — 三方对照：vanilla 导出 vs Rust fill vs 玩家存档（chunk(-5,-3)）。
// 判定「橡树树叶海洋」出现在哪一层。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be32(b: &[u8], i: &mut usize) -> i32 {
    let v = i32::from_be_bytes([b[*i], b[*i+1], b[*i+2], b[*i+3]]); *i += 4; v
}
fn be16(b: &[u8], i: &mut usize) -> i32 {
    let v = i16::from_be_bytes([b[*i], b[*i+1]]); *i += 2; i32::from(v)
}
fn be64(b: &[u8], i: &mut usize) -> i64 {
    let mut t = [0u8; 8]; t.copy_from_slice(&b[*i..*i+8]); *i += 8; i64::from_be_bytes(t)
}

fn main() {
    let seed: i64 = -2032795982907864146;
    let vf = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\vanilla_-2032795982907864146_1_-80_-48_nether.blocks";
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    // vanilla 导出解析
    let bd = std::fs::read(vf).expect("vanilla file");
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let ox = be32(&bd, &mut i); let oz = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("vanilla: magic={:#X} seed={} size={} origin=({},{}) min_y={} height={}", magic, vseed, size, ox, oz, min_y, height);
    let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
    let bpc = (16 * 16 * height) as usize;
    let mut vanilla = vec![0i32; bpc];
    for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
    let blocks = h.fill_chunk_blocks(cx, cz);
    println!("chunk({},{}) vanilla={} rust={}（rust 长度 {}）", cx, cz, vanilla.len(), blocks.len(), blocks.len());
    // 玩家存档（MCA）解析 chunk(-5,-3) 的 palette 计数——由 parse_mca_chunk.py 单独跑，此处只对 vanilla/rust。
    println!("y      va_air  ru_air   vanilla主要        rust主要");
    for y in (0..height).step_by(8) {
        let base = (y - min_y) as usize * 256;
        let va_air = (base..base+256).filter(|&k| vanilla[k] == 0).count();
        let ru_air = (base..base+256).filter(|&k| blocks[k] == 0).count();
        let top = |f: &dyn Fn(usize) -> i32| {
            let mut m: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
            for k in base..base+256 { *m.entry(f(k)).or_insert(0) += 1; }
            let mut s = String::new();
            for (b, c) in m.iter().rev().take(3) { s.push_str(&format!(" {}x{}", b, c)); }
            s
        };
        let vt = top(&|k| vanilla[k]);
        let rt = top(&|k| blocks[k]);
        println!("y={:<4} {:<6} {:<7} {}   {}", y, va_air, ru_air, vt, rt);
    }
}
