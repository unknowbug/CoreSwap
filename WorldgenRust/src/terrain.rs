// terrain.rs — Rust 端到端 fill 管线（宏观：表面高度 / biome / 水-岩-空气分类）。
// 目标：从 seed 出发，走 density→aquifer→surface→[MOD扩展点]，产出能看的地形。
// 通用性：每层 trait 化，MOD 可注入替代实现（不焊死 vanilla）。
// 正确性：宏观对（山/湖/biome/水面高度大体一致）；不追微观 block id（tuff/ore/树/矿物）。
// 性能：density 已有 Interpolated/Cache2D/FlatCache 缓存；fill 增量复用缓存。
// Beardifier：fill 时对每个块密度加结构 Beardifier 修正（add(finalDensity, Beardifier) CellCache 语义）。

use crate::beardifier::Beardifier;
use crate::density::{DensityFunction, NoisePos};

// ---- 宏观块分类（不追具体 block id；水/岩/空气即可，符合"别严重 BUG + 宏观对"）----
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind { Air, Rock, Water, Lava }

// ---- 抽象源（MOD 扩展点：可注入替代实现）----
pub trait DensitySource {
    fn sample(&self, pos: &NoisePos) -> f64;
}
pub trait AquiferSource {
    // d = density 值；返回该块的水/岩/空气/岩浆分类（宏观）
    fn classify(&mut self, x: i32, y: i32, z: i32, d: f64) -> BlockKind;
}
pub trait BiomeSource {
    // biome id（宏观）；pos 已 floor 对齐
    fn biome(&self, pos: &NoisePos) -> String;
}

// ---- 默认 vanilla 实现（复用已验证的 density/aquifer）----
pub struct VanillaDensity<'a> { pub df: &'a DensityFunction }
impl<'a> DensitySource for VanillaDensity<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 { self.df.sample(pos) }
}
pub struct VanillaAquifer { pub aq: crate::aquifer::Aquifer, pub enabled: bool }
impl VanillaAquifer {
    // 便捷构造：默认启用 aquifer（overworld）
    pub fn new(aq: crate::aquifer::Aquifer) -> Self { Self { aq, enabled: true } }
}
impl AquiferSource for VanillaAquifer {
    fn classify(&mut self, x: i32, y: i32, z: i32, d: f64) -> BlockKind {
        if d > 0.0 { return BlockKind::Rock; }
        // WG_SKIP_AQUIFER（诊断）或 aquifers disabled（下界）：跳过真实 aquifer，直接 Air
        if !self.enabled || std::env::var("WG_SKIP_AQUIFER").is_ok() { return BlockKind::Air; }
        match self.aq.apply(x, y, z, d) { 1 => BlockKind::Water, 2 => BlockKind::Lava, _ => BlockKind::Air }
    }
}

// ---- ChunkData（宏观产物）----
pub struct ChunkData {
    pub cx: i32, pub cz: i32,
    pub surface_height: [i32; 256],   // 每列地表高度（首个 solid 的 y；无 solid = i32::MIN）
    pub blocks: Vec<BlockKind>,        // [16*16*height] 水/岩/空气/岩浆 分类（index: lx + lz*16 + ly*256）
    pub biome: Vec<String>,            // 每列 biome（宏观）[256]
}

// fill_chunk：从 seed + chunk 坐标生成 ChunkData（宏观管线）。
// dense: 密度源；aqua: 含水层源；biome: biome 源；min_y/height: 维度参数。
// beard: 该 chunk 的 Beardifier 输入（结构密度修正）；None = 无结构（Beardifier=0）。
pub fn fill_chunk<D: DensitySource, A: AquiferSource, B: BiomeSource>(
    dense: &D, aqua: &mut A, biome: &B, cx: i32, cz: i32, min_y: i32, height: i32,
    beard: Option<&Beardifier>,
) -> ChunkData {
    let mut cd = ChunkData {
        cx, cz,
        surface_height: [i32::MIN; 256],
        blocks: vec![BlockKind::Air; (16*16*height) as usize],
        biome: vec!["".to_string(); 256],
    };
    // 单遍逐列：自顶向下，一次树求值同时完成 surface 高度 + 块分类（省 50% 采样）
    // WG_FILLCHUNK_PROFILE（单线程诊断，规避测量污染铁律）：分段计时 dense.sample / beard / classify / biome
    let profile = std::env::var("WG_FILLCHUNK_PROFILE").is_ok();
    let mut t_dense = 0.0f64; let mut t_beard = 0.0f64; let mut t_classify = 0.0f64; let mut t_biome = 0.0f64;
    let t0 = std::time::Instant::now();
    for lz in 0..16 {
        for lx in 0..16 {
            let x = cx*16+lx; let z = cz*16+lz;
            let mut top = i32::MIN;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                let b1 = std::time::Instant::now();
                let mut d = dense.sample(&NoisePos{x,y,z});
                if profile { t_dense += b1.elapsed().as_secs_f64(); }
                // Beardifier：块级加结构密度修正（CellCache add(finalDensity, Beardifier)）
                if let Some(b) = beard {
                    let b2 = std::time::Instant::now();
                    d += b.sample(x, y, z);
                    if profile { t_beard += b2.elapsed().as_secs_f64(); }
                }
                let b3 = std::time::Instant::now();
                let kind = aqua.classify(x, y, z, d);
                if profile { t_classify += b3.elapsed().as_secs_f64(); }
                cd.blocks[(lx + lz*16 + ly*256) as usize] = kind;
                if top == i32::MIN && d > 0.0 { top = y; }
            }
            cd.surface_height[(lz*16+lx) as usize] = top;
            // biome：该列地表处 floor 对齐采样
            let by = if top != i32::MIN { (top>>2)<<2 } else { min_y };
            let bp = NoisePos{x:(x>>2)<<2,y:by,z:(z>>2)<<2};
            let b4 = std::time::Instant::now();
            cd.biome[(lz*16+lx) as usize] = biome.biome(&bp);
            if profile { t_biome += b4.elapsed().as_secs_f64(); }
        }
    }
    if profile {
        let total = t0.elapsed().as_secs_f64();
        eprintln!("[FILLCHUNK_PROFILE] chunk({},{}) total={:.1}ms dense={:.1}ms({:.0}%) beard={:.1}ms classify={:.1}ms({:.0}%) biome={:.1}ms({:.0}%) loop_overhead={:.1}ms({:.0}%)",
            cx, cz, total*1e3,
            t_dense*1e3, t_dense/total*100.0,
            t_beard*1e3,
            t_classify*1e3, t_classify/total*100.0,
            t_biome*1e3, t_biome/total*100.0,
            (total - t_dense - t_beard - t_classify - t_biome)*1e3,
            (total - t_dense - t_beard - t_classify - t_biome)/total*100.0);
    }
    cd
}
