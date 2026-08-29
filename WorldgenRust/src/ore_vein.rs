// ore_vein.rs — OreVeinSampler 复刻（1.20.1，C++ ore_vein.h 71 行移植）。
// 矿脉：veinToggle > 0 → 铜矿脉(y 0..50)；≤ 0 → 铁矿脉(y -60..-8，含 tuff/deepslate_iron_ore/raw_iron_block)。
use crate::density::{DensityFunction, NoisePos};
use crate::xoroshiro::XoroshiroSplitter;

// 块 id（blocks.json）
const COPPER_ORE: i32 = 923;
const RAW_COPPER_BLOCK: i32 = 993;
const GRANITE: i32 = 2;
const DEEPSLATE_IRON_ORE: i32 = 42;
const RAW_IRON_BLOCK: i32 = 992;
const TUFF: i32 = 909;

struct VeinType { ore: i32, raw_ore_block: i32, stone: i32, min_y: i32, max_y: i32 }

pub struct OreVeinSampler {
    vein_toggle: std::sync::Arc<DensityFunction>,
    vein_ridged: std::sync::Arc<DensityFunction>,
    vein_gap: std::sync::Arc<DensityFunction>,
    splitter: XoroshiroSplitter,
}
impl OreVeinSampler {
    pub fn new(
        vein_toggle: std::sync::Arc<DensityFunction>,
        vein_ridged: std::sync::Arc<DensityFunction>,
        vein_gap: std::sync::Arc<DensityFunction>,
        splitter: XoroshiroSplitter,
    ) -> Self {
        Self { vein_toggle, vein_ridged, vein_gap, splitter }
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
        let (ore, raw, stone, min_y, max_y);
        if d > 0.0 { ore = COPPER_ORE; raw = RAW_COPPER_BLOCK; stone = GRANITE; min_y = 0; max_y = 50; }
        else { ore = DEEPSLATE_IRON_ORE; raw = RAW_IRON_BLOCK; stone = TUFF; min_y = -60; max_y = -8; }
        let e = d.abs();
        let j = max_y - y;
        let k = y - min_y;
        if k >= 0 && j >= 0 {
            let l = j.min(k);
            let f = Self::lerp_clamp(l as f64, 0.0, 20.0, -0.2, 0.0);
            if e + f < 0.4 { return -1; }
            let mut random = self.splitter.split_xyz(x, y, z);
            if random.next_float() > 0.7_f32 { return -1; }
            if self.vein_ridged.sample(&pos) >= 0.0 { return -1; }
            let g = Self::lerp_clamp(e, 0.4, 0.6, 0.1, 0.3);
            if random.next_float() < g as f32 && self.vein_gap.sample(&pos) > -0.3 {
                if random.next_float() < 0.02_f32 { return raw; }
                return ore;
            }
            return stone;
        }
        -1
    }
}
