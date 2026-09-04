// ore_vein.rs — OreVeinSampler 复刻（1.20.1，C++ ore_vein.h 71 行移植）。
// 矿脉：veinToggle > 0 → 铜矿脉(y 0..50)；≤ 0 → 铁矿脉(y -60..-8，含 tuff/deepslate_iron_ore/raw_iron_block)。
// 数据驱动：block id 从 BlockRegistry 解析（非硬编码数字），对齐 C++ blocks->id(name)。
use crate::blocks::BlockRegistry;
use crate::density::{DensityFunction, NoisePos};
use crate::xoroshiro::XoroshiroSplitter;

// 矿脉类型：block 名称（构造时用 BlockRegistry 解析成 id）
struct VeinType { ore: i32, raw_ore_block: i32, stone: i32, min_y: i32, max_y: i32 }

pub struct OreVeinSampler {
    vein_toggle: std::sync::Arc<DensityFunction>,
    vein_ridged: std::sync::Arc<DensityFunction>,
    vein_gap: std::sync::Arc<DensityFunction>,
    splitter: XoroshiroSplitter,
    copper: VeinType,
    iron: VeinType,
}
impl OreVeinSampler {
    // blocks: 从 BlockRegistry 解析矿脉 block id（数据驱动，非硬编码）
    pub fn new(
        vein_toggle: std::sync::Arc<DensityFunction>,
        vein_ridged: std::sync::Arc<DensityFunction>,
        vein_gap: std::sync::Arc<DensityFunction>,
        splitter: XoroshiroSplitter,
        blocks: &BlockRegistry,
    ) -> Self {
        let mut vt = |ore: &str, raw: &str, stone: &str, min_y: i32, max_y: i32| VeinType {
            ore: blocks.id(ore), raw_ore_block: blocks.id(raw), stone: blocks.id(stone), min_y, max_y,
        };
        let copper = vt("minecraft:copper_ore", "minecraft:raw_copper_block", "minecraft:granite", 0, 50);
        let iron = vt("minecraft:deepslate_iron_ore", "minecraft:raw_iron_block", "minecraft:tuff", -60, -8);
        Self { vein_toggle, vein_ridged, vein_gap, splitter, copper, iron }
    }

    fn lerp_clamp(value: f64, from_start: f64, from_end: f64, to_start: f64, to_end: f64) -> f64 {
        let mut t = (value - from_start) / (from_end - from_start);
        t = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
        to_start + t * (to_end - to_start)
    }

    // 返回矿脉块 id；不适用返回 -1
    // &self（只读：density 采样 + splitter.split_xyz 均 &self）——无需锁，并发安全
    pub fn apply(&self, x: i32, y: i32, z: i32) -> i32 {
        // 无损预检查：y 范围外（copper[0,50] ∪ iron[-60,-8]）同 Java 返回 -1
        if y < -60 || y > 50 { return -1; }
        let pos = NoisePos { x, y, z };
        let d = self.vein_toggle.sample(&pos);
        let t = if d > 0.0 { &self.copper } else { &self.iron };
        let e = d.abs();
        let j = t.max_y - y;
        let k = y - t.min_y;
        if k >= 0 && j >= 0 {
            let l = j.min(k);
            let f = Self::lerp_clamp(l as f64, 0.0, 20.0, -0.2, 0.0);
            if e + f < 0.4 { return -1; }
            let mut random = self.splitter.split_xyz(x, y, z);
            if random.next_float() > 0.7_f32 { return -1; }
            if self.vein_ridged.sample(&pos) >= 0.0 { return -1; }
            let g = Self::lerp_clamp(e, 0.4, 0.6, 0.1, 0.3);
            if random.next_float() < g as f32 && self.vein_gap.sample(&pos) > -0.3 {
                if random.next_float() < 0.02_f32 { return t.raw_ore_block; }
                return t.ore;
            }
            return t.stone;
        }
        -1
    }
}
