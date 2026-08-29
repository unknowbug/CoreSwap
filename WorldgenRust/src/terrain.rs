// terrain.rs — Rust 端到端 fill 管线（宏观：表面高度 / biome / 水-岩-空气分类）。
// 目标：从 seed 出发，走 density→aquifer→surface→[MOD扩展点]，产出能看的地形。
// 通用性：每层 trait 化，MOD 可注入替代实现（不焊死 vanilla）。
// 正确性：宏观对（山/湖/biome/水面高度大体一致）；不追微观 block id（tuff/ore/树/矿物）。
// 性能：density 已有 Interpolated/Cache2D/FlatCache 缓存；fill 增量复用缓存。
// Beardifier：fill 时对每个块密度加结构 Beardifier 修正（add(finalDensity, Beardifier) CellCache 语义）。

use crate::beardifier::Beardifier;
use crate::density::{DensityFunction, NoisePos};

// ---- 宏观 cell 网格采样器（对齐 Java NoiseChunk：cell corners 采样 + 三线性插值，非逐 block 采样）----
// Java 宏观：对每个 Interpolated 在 cell corners（4x4x8 网格点）采样一次 + 块级三线性插值。
// Rust fill_chunk 原本逐 block 采样 final_density（98304 次）——缺顶层宏观网格，density 宏观慢。
// 本结构：对 chunk 的 4x4x8 cell corners 采样 final_density（~1225 次），块级三线性插值。
// 对齐 Java 单层插值语义（非精确逐点）；可通过 WG_NO_MACROGRID 回退逐点采样避免风险。
const MCG_CELL_X: i32 = 4;
const MCG_CELL_Y: i32 = 8;
const MCG_CELL_Z: i32 = 4;

pub struct MacroGrid {
    grid: Vec<f64>,
    gx: usize, gy: usize, gz: usize,
    min_y: i32, height: i32,
}
impl MacroGrid {
    /// 对 chunk(cx,cz) 的 4x4x8 cell corners 采样 final_density（~1225 次），存网格。
    pub fn build<F: Fn(&NoisePos) -> f64>(dense: &F, cx: i32, cz: i32, min_y: i32, height: i32) -> Self {
        let gx = (16 / MCG_CELL_X + 1) as usize;
        let gy = (height / MCG_CELL_Y + 1) as usize;
        let gz = (16 / MCG_CELL_Z + 1) as usize;
        let mut grid = vec![0.0f64; gx * gy * gz];
        for iy in 0..gy {
            for iz in 0..gz {
                for ix in 0..gx {
                    let px = cx * 16 + ix as i32 * MCG_CELL_X;
                    let py = min_y + iy as i32 * MCG_CELL_Y;
                    let pz = cz * 16 + iz as i32 * MCG_CELL_Z;
                    grid[((iy * gz + iz) * gx + ix) as usize] = dense(&NoisePos { x: px, y: py, z: pz });
                }
            }
        }
        MacroGrid { grid, gx, gy, gz, min_y, height }
    }
    /// 块级三线性插值（对齐 InterpolatedData::sample 的插值逻辑）。
    #[inline]
    pub fn sample(&self, x: i32, y: i32, z: i32) -> f64 {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);
        let gx = self.gx as i32; let gy = self.gy as i32; let gz = self.gz as i32;
        let gxx = x - chunk_x * 16;
        let gyy = y - self.min_y;
        let gzz = z - chunk_z * 16;
        let mut cx = gxx / MCG_CELL_X;
        let mut cy = gyy / MCG_CELL_Y;
        let mut cz = gzz / MCG_CELL_Z;
        if cx < 0 || cy < 0 || cz < 0 || cx >= gx - 1 || cy >= gy - 1 || cz >= gz - 1 {
            cx = if cx < 0 { 0 } else if cx > gx - 2 { gx - 2 } else { cx };
            cy = if cy < 0 { 0 } else if cy > gy - 2 { gy - 2 } else { cy };
            cz = if cz < 0 { 0 } else if cz > gz - 2 { gz - 2 } else { cz };
        }
        let fx = (gxx % MCG_CELL_X) as f64 / MCG_CELL_X as f64;
        let fy = (gyy % MCG_CELL_Y) as f64 / MCG_CELL_Y as f64;
        let fz = (gzz % MCG_CELL_Z) as f64 / MCG_CELL_Z as f64;
        let at = |dx: i32, dy: i32, dz: i32| self.grid[(((cy + dy) * gz + (cz + dz)) * gx + (cx + dx)) as usize];
        let d000 = at(0, 0, 0); let d100 = at(1, 0, 0); let d010 = at(0, 1, 0); let d110 = at(1, 1, 0);
        let d001 = at(0, 0, 1); let d101 = at(1, 0, 1); let d011 = at(0, 1, 1); let d111 = at(1, 1, 1);
        let d00 = d000 + (d100 - d000) * fx;
        let d10 = d010 + (d110 - d010) * fx;
        let d01 = d001 + (d101 - d001) * fx;
        let d11 = d011 + (d111 - d011) * fx;
        let d0 = d00 + (d10 - d00) * fy;
        let d1 = d01 + (d11 - d01) * fy;
        d0 + (d1 - d0) * fz
    }
}

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
pub struct VanillaAquifer { pub aq: crate::aquifer::Aquifer, pub enabled: bool, pub skip_aquifer: bool }
impl VanillaAquifer {
    // 便捷构造：默认启用 aquifer（overworld）
    pub fn new(aq: crate::aquifer::Aquifer) -> Self { Self { aq, enabled: true, skip_aquifer: false } }
}
impl AquiferSource for VanillaAquifer {
    fn classify(&mut self, x: i32, y: i32, z: i32, d: f64) -> BlockKind {
        if d > 0.0 { return BlockKind::Rock; }
        // skip_aquifer（诊断，chunk 级判断一次）或 aquifers disabled（下界）：跳过真实 aquifer，直接 Air
        if !self.enabled || self.skip_aquifer { return BlockKind::Air; }
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
    // 宏观 cell 网格采样（对齐 Java NoiseChunk）：cell corners 采样 final_density + 块级三线性插值。
    // ⚠️ 实验性：直接对 final_density 采样 corners 会触发内部 interpolated 雪崩（52x 慢，见 macro_grid 记录）。
    // 默认逐点（WG_MACROGRID 显式启用）；正确方向 = Java multi-channel（每个 interpolated 独立 corners 采样，见记录）。
    let use_macro_grid = std::env::var("WG_MACROGRID").is_ok();
    let grid = if use_macro_grid {
        let dense_ref = |p: &NoisePos| dense.sample(p);
        Some(MacroGrid::build(&dense_ref, cx, cz, min_y, height))
    } else {
        None
    };
    // 单遍逐列：自顶向下，一次树求值（或宏观网格插值）同时完成 surface 高度 + 块分类（省 50% 采样）
    for lz in 0..16 {
        for lx in 0..16 {
            let x = cx*16+lx; let z = cz*16+lz;
            let mut top = i32::MIN;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                // density：宏观网格插值（快）或逐点采样（回退/对齐风险时）
                let d0 = match &grid { Some(g) => g.sample(x, y, z), None => dense.sample(&NoisePos{x,y,z}) };
                // Beardifier：块级加结构密度修正（CellCache add(finalDensity, Beardifier)）
                let mut d = d0;
                if let Some(b) = beard { d += b.sample(x, y, z); }
                let kind = aqua.classify(x, y, z, d);
                cd.blocks[(lx + lz*16 + ly*256) as usize] = kind;
                if top == i32::MIN && d > 0.0 { top = y; }
            }
            cd.surface_height[(lz*16+lx) as usize] = top;
            // biome：该列地表处 floor 对齐采样
            let by = if top != i32::MIN { (top>>2)<<2 } else { min_y };
            let bp = NoisePos{x:(x>>2)<<2,y:by,z:(z>>2)<<2};
            cd.biome[(lz*16+lx) as usize] = biome.biome(&bp);
        }
    }
    cd
}
