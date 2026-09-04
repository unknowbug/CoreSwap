// vein_probe.rs — 验证 ore_vein 功能（矿脉块是否产生）。
// 用 WorldgenHandle::create + fill_chunk_blocks 生成 chunk，统计铜/铁矿脉块数量。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");

    let copper = 923;           // minecraft:copper_ore (vein)
    let raw_copper = 993;       // minecraft:raw_copper_block
    let deepslate_iron = 42;    // minecraft:deepslate_iron_ore (vein)
    let raw_iron = 992;         // minecraft:raw_iron_block
    let tuff = 909;
    let granite = 2;

    let mut counts = std::collections::HashMap::new();
    // 生成 4x4 chunk（-288,-256）
    for cz in (-256..-252) {
        for cx in (-288..-284) {
            let blocks = h.fill_chunk_blocks(cx, cz);
            for &b in &blocks {
                if b == copper || b == raw_copper || b == deepslate_iron || b == raw_iron {
                    *counts.entry(b).or_insert(0) += 1;
                }
            }
        }
    }
    println!("=== ore vein block counts (-288..-284, -256..-252) ===");
    println!("copper_ore: {}", counts.get(&copper).unwrap_or(&0));
    println!("raw_copper: {}", counts.get(&raw_copper).unwrap_or(&0));
    println!("deepslate_iron: {}", counts.get(&deepslate_iron).unwrap_or(&0));
    println!("raw_iron: {}", counts.get(&raw_iron).unwrap_or(&0));
    let total: i32 = counts.values().sum();
    println!("total vein blocks: {}", total);
    if total > 0 { println!("[OK] ore_vein functional (vein blocks present)"); }
    else { println!("[WARN] ore_vein produced 0 vein blocks in this region"); }
}
