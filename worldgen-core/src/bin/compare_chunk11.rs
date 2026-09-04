// compare_chunk11.rs — Rust fill vs vanilla chunk(1,1) @ server seed 逐 y 层对照。
// 判定用户报告的「大范围 air」是否 vanilla 亦然（空腔正常 vs 实现差异）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be32(b: &[u8], i: &mut usize) -> i32 {
    let v = i32::from_be_bytes([b[*i], b[*i+1], b[*i+2], b[*i+3]]); *i += 4; v
}
fn be64(b: &[u8], i: &mut usize) -> i64 {
    let mut t = [0u8; 8]; t.copy_from_slice(&b[*i..*i+8]); *i += 8; i64::from_be_bytes(t)
}
fn be16(b: &[u8], i: &mut usize) -> i32 {
    let v = i16::from_be_bytes([b[*i], b[*i+1]]); *i += 2; i32::from(v)
}

fn main() {
    let seed: i64 = -2032795982907864146;
    let vf = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\vanilla_-2032795982907864146_1_16_16_nether.blocks";
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    let bd = std::fs::read(vf).expect("vanilla file");
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let _size = be32(&bd, &mut i);
    let ox = be32(&bd, &mut i); let oz = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("vanilla: magic={:#X} seed={} origin=({},{}) min_y={} height={}", magic, vseed, ox, oz, min_y, height);
    let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
    let bpc = 16 * 16 * height as usize;
    let mut vanilla = vec![0i32; bpc];
    for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
    let blocks = h.fill_chunk_blocks(cx, cz);
    println!("chunk({},{}) 长度 vanilla={} rust={}（{}）", cx, cz, vanilla.len(), blocks.len(), if vanilla.len() == blocks.len() { "一致" } else { "不一致!" });
    // 每 y 层：vanilla air / rust air / vanilla 主要块 / rust 主要块
    println!("y      va_air  ru_air   vanilla主要   rust主要");
    for y in (0..height as i32).step_by(8) {
        let base = (y - min_y) as usize * 256;
        let va_air = (base..base+256).filter(|&k| vanilla[k] == 0).count();
        let ru_air = (base..base+256).filter(|&k| blocks[k] == 0).count();
        let top = |f: &dyn Fn(usize) -> i32| {
            let mut m: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
            for k in base..base+256 { *m.entry(f(k)).or_insert(0) += 1; }
            let mut s = String::new();
            for (b, c) in m.iter().rev().take(2) { s.push_str(&format!(" {}x{}", b, c)); }
            s
        };
        let vt = top(&|k| vanilla[k]);
        let rt = top(&|k| blocks[k]);
        println!("y={:<4} {:<6} {:<7}  {}   {}", y, va_air, ru_air, vt, rt);
    }
    // 总差异数
    let diff = (0..bpc).filter(|&k| vanilla[k] != blocks[k]).count();
    println!("总差异块数: {}/{} ({:.2}%)", diff, bpc, diff as f64 / bpc as f64 * 100.0);
}


