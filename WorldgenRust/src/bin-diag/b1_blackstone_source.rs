// b1_blackstone_source.rs — A/B: WG_SKIP_SURFACE 开关下 chunk(200,200) 各 id 计数差
// → 判 blackstone/basalt/netherrack/magma 的写入者是否 surface rule。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn counts(blocks: &[i32]) -> std::collections::HashMap<i32, u64> {
    let mut m = std::collections::HashMap::new();
    for &b in blocks { *m.entry(b).or_insert(0u64) += 1; }
    m
}

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    unsafe { std::env::remove_var("WG_SKIP_SURFACE"); }
    let mut a_all = Vec::new();
    let h = WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256).expect("create");
    for cx in 200..204 { for cz in 200..204 { a_all.extend(h.fill_chunk_blocks(cx, cz)); } }
    let a = a_all;
    // dump for cell-level python compare (chunk order 200..203 x 200..203, y*256+z*16+x)
    let mut out = Vec::with_capacity(a.len() * 4);
    for v in &a { out.extend_from_slice(&v.to_le_bytes()); }
    std::fs::write(r"E:\PYTHON\CoreSwap\.tmp\b1-rlib-blocks.bin", &out).unwrap();
    unsafe { std::env::set_var("WG_SKIP_SURFACE", "1"); }
    let b = h.fill_chunk_blocks(200, 200);
    unsafe { std::env::remove_var("WG_SKIP_SURFACE"); }
    let (ca, cb) = (counts(&a), counts(&b));
    let mut keys: Vec<i32> = ca.keys().chain(cb.keys()).copied().collect();
    keys.sort(); keys.dedup();
    println!("id      with_surface  no_surface  diff");
    for k in keys {
        let x = ca.get(&k).copied().unwrap_or(0); let y = cb.get(&k).copied().unwrap_or(0);
        if x != y { println!("{:<7} {:>12} {:>11} {:>6}", k, x, y, x as i64 - y as i64); }
    }
}
