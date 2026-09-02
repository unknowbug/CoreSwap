// terrain.rs — Rust 端到端 fill 管线（宏观：表面高度 / biome / 水-岩-空气分类）。
// 目标：从 seed 出发，走 density→aquifer→surface→[MOD扩展点]，产出能看的地形。
// 通用性：每层 trait 化，MOD 可注入替代实现（不焊死 vanilla）。
// 正确性：宏观对（山/湖/biome/水面高度大体一致）；不追微观 block id（tuff/ore/树/矿物）。
// 性能：density 已有 Interpolated/Cache2D/FlatCache 缓存；fill 增量复用缓存。
// Beardifier：fill 时对每个块密度加结构 Beardifier 修正（add(finalDensity, Beardifier) CellCache 语义）。

use crate::beardifier::Beardifier;
use crate::density::{DensityFunction, NoisePos, macrolize_channels};
use std::cell::RefCell;
use std::sync::Arc;

// ---- 生产版 multi-channel 宏观采样器（对齐 SteelMC/Java NoiseChunk，正确性 diff0 已验证）----
// 竖切：macrolize final_density → channels（每个 Interpolated inner）+ combine（外层操作树，Interpolated→ReadChannel）。
// 对 chunk 的 4x4x8 cell corners 采样所有 channels（~1225×Nch 次），块级三线性插值 + combine。
// 避免「采样整树」→ 内部 Interpolated 雪崩（channels 是纯 inner，无嵌套）。
// thread_local slices 缓存（每 chunk 重建一次，块级 O(1) 插值）。
pub struct DensityMacroSampler {
    channels: Vec<Arc<DensityFunction>>,
    combine: DensityFunction,
    min_y: i32, height: i32,
    cell_w: i32, cell_h: i32,
    gx: usize, gy: usize, gz: usize,
}
thread_local! {
    static MACRO_SLICE_CACHE: RefCell<(i64, Vec<f64>)> = RefCell::new((i64::MIN, Vec::new()));
}
impl DensityMacroSampler {
    pub fn new(tree: &DensityFunction, min_y: i32, height: i32) -> Self {
        let (channels, combine) = macrolize_channels(tree);
        Self { channels, combine, min_y, height, cell_w: 4, cell_h: 8,
            gx: (16/4+1) as usize, gy: (height/8+1) as usize, gz: (16/4+1) as usize }
    }
    fn build_slices(&self, cx: i32, cz: i32) -> Vec<f64> {
        let nch = self.channels.len();
        let mut slices = vec![0.0f64; self.gx * self.gy * self.gz * nch];
        for ix in 0..self.gx {
            for iz in 0..self.gz {
                for iy in 0..self.gy {
                    let px = cx*16 + ix as i32 * self.cell_w;
                    let py = self.min_y + iy as i32 * self.cell_h;
                    let pz = cz*16 + iz as i32 * self.cell_w;
                    let pos = NoisePos { x: px, y: py, z: pz };
                    for ch in 0..nch {
                        slices[((iy*self.gz + iz)*self.gx + ix)*nch + ch] = self.channels[ch].sample(&pos);
                    }
                }
            }
        }
        slices
    }
    #[inline]
    fn sample_interp_impl(&self, slices: &[f64], pos: &NoisePos) -> f64 {
        let gx = self.gx as i32; let gy = self.gy as i32; let gz = self.gz as i32;
        let chunk_x = pos.x.div_euclid(16); let chunk_z = pos.z.div_euclid(16);
        let gxx = pos.x - chunk_x*16; let gzz = pos.z - chunk_z*16; let gyy = pos.y - self.min_y;
        let mut cx = gxx / self.cell_w; let mut cy = gyy / self.cell_h; let mut cz = gzz / self.cell_w;
        cx = cx.clamp(0, gx-2); cy = cy.clamp(0, gy-2); cz = cz.clamp(0, gz-2);
        let fx = (gxx % self.cell_w) as f64 / self.cell_w as f64;
        let fy = (gyy % self.cell_h) as f64 / self.cell_h as f64;
        let fz = (gzz % self.cell_w) as f64 / self.cell_w as f64;
        let nch = self.channels.len();
        let at = |dx: i32, dy: i32, dz: i32, ch: usize| -> f64 {
            let cell_idx = ((cy+dy)*gz + (cz+dz))*gx + (cx+dx);
            slices[cell_idx as usize * nch + ch]
        };
        // 栈数组替代每 block heap Vec 分配（热路径 ~98304 次/chunk，scout §1.5）
        let mut interp = [0.0f64; 8];
        debug_assert!(nch <= 8);
        let nch = nch.min(8);
        for ch in 0..nch {
            let d000=at(0,0,0,ch); let d100=at(1,0,0,ch); let d010=at(0,1,0,ch); let d110=at(1,1,0,ch);
            let d001=at(0,0,1,ch); let d101=at(1,0,1,ch); let d011=at(0,1,1,ch); let d111=at(1,1,1,ch);
            let d00=d000+(d100-d000)*fx; let d10=d010+(d110-d010)*fx;
            let d01=d001+(d101-d001)*fx; let d11=d011+(d111-d011)*fx;
            let d0=d00+(d10-d00)*fy; let d1=d01+(d11-d01)*fy;
            interp[ch] = d0 + (d1 - d0)*fz;
        }
        self.combine.sample_combine(pos, &interp[..nch])
    }
}
impl ChunkDensitySampler for DensityMacroSampler {
    fn sample_interp(&self, slices: &[f64], pos: &NoisePos) -> f64 {
        self.sample_interp_impl(slices, pos)
    }
}
impl DensitySource<DensityMacroSampler> for DensityMacroSampler {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let chunk_x = pos.x.div_euclid(16); let chunk_z = pos.z.div_euclid(16);
        let key = ((chunk_x as i64) << 32) ^ (chunk_z as u32 as i64);
        // thread_local slices 缓存：with 内 borrow + sample_interp（避免 RefMut 跨 with 生命周期）
        MACRO_SLICE_CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != key {
                c.0 = key;
                c.1 = self.build_slices(chunk_x, chunk_z);
            }
            self.sample_interp_impl(&c.1, pos)
        })
    }
    // 每 chunk 构建 slices 一次（避免 thread_local 每点访问）
    fn sample_chunk(&self, cx: i32, cz: i32, _min_y: i32, _height: i32) -> Option<ChunkDensity<'_, DensityMacroSampler>> {
        let slices = self.build_slices(cx, cz);
        Some(ChunkDensity { sampler: self, slices })
    }
}

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
pub trait DensitySource<S: ChunkDensitySampler> {
    fn sample(&self, pos: &NoisePos) -> f64;
    // 每 chunk 构建宏观采样（DensityMacroSampler/TranspilerDensity 实现；默认 None = 逐点）。避免 thread_local 每点访问。
    fn sample_chunk(&self, cx: i32, cz: i32, min_y: i32, height: i32) -> Option<ChunkDensity<'_, S>> { None }
}
// 每 chunk 的宏观采样结果（slices 已构建，块级 O(1) 插值）
pub struct ChunkDensity<'a, S: ChunkDensitySampler> {
    sampler: &'a S,
    slices: Vec<f64>,
}
impl<'a, S: ChunkDensitySampler> ChunkDensity<'a, S> {
    #[inline]
    pub fn sample(&self, pos: &NoisePos) -> f64 { self.sampler.sample_interp(&self.slices, pos) }
}
// 宏观采样器的块级插值接口（DensityMacroSampler / TranspilerDensity 实现）
pub trait ChunkDensitySampler {
    fn sample_interp(&self, slices: &[f64], pos: &NoisePos) -> f64;
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
impl<'a> DensitySource<DensityMacroSampler> for VanillaDensity<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 { self.df.sample(pos) }
}
pub struct VanillaAquifer { pub aq: crate::aquifer::Aquifer, pub enabled: bool, pub skip_aquifer: bool,
    pub sea_level: i32 }  // settings sea_level（下界 32）：aquifer 禁用时的 sea-level 熔岩语义用
impl VanillaAquifer {
    // 便捷构造：默认启用 aquifer（overworld）
    pub fn new(aq: crate::aquifer::Aquifer) -> Self { Self { aq, enabled: true, skip_aquifer: false, sea_level: 63 } }
}
impl AquiferSource for VanillaAquifer {
    fn classify(&mut self, x: i32, y: i32, z: i32, d: f64) -> BlockKind {
        if d > 0.0 { return BlockKind::Rock; }
        // aquifers disabled（下界）→ 对齐 Java AquiferSampler.seaLevel()（ChunkNoiseSampler L160-161）：
        // d ≤ 0 时 FluidLevel(sea_level, default_fluid).getBlockState(y) = y < sea_level ? lava : air（严格 <）。
        // 无噪声参与（floodedness/spread/lava/barrier 均 seaLevel 路径不采样）；y 无下界（min_y 起全 lava）。
        if !self.enabled {
            return if y < self.sea_level { BlockKind::Lava } else { BlockKind::Air };
        }
        // skip_aquifer（诊断，chunk 级判断一次）：跳过真实 aquifer，直接 Air（保留原诊断语义）
        if self.skip_aquifer { return BlockKind::Air; }
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
pub fn fill_chunk<D: DensitySource<S>, S: ChunkDensitySampler, A: AquiferSource, B: BiomeSource>(
    dense: &D, aqua: &mut A, biome: &B, cx: i32, cz: i32, min_y: i32, height: i32,
    beard: Option<&Beardifier>,
    noise_height: i32,
) -> ChunkData {
    let mut cd = ChunkData {
        cx, cz,
        surface_height: [i32::MIN; 256],
        blocks: vec![BlockKind::Air; (16*16*height) as usize],
        biome: vec!["".to_string(); 256],
    };
    // 双高度（对齐 docs/09 下界引擎）：噪声高度（density 采样有效域）≤ 世界高度（buffer）。
    // y ≥ min_y+noise_height 不采样，保持初始 Air（nether：noise 128 / world 256；overworld 两者相等零变化）。
    let noise_top = min_y + noise_height;
    // 每 chunk 构建宏观采样（DensityMacroSampler 支持 multi-channel cell grid；否则 None = 逐点）。
    // 避免 thread_local 每点访问（对齐 Java NoiseChunk cell grid 语义）。
    let chunk_density = dense.sample_chunk(cx, cz, min_y, noise_height);
    // 单遍逐列：自顶向下，一次树求值（或宏观网格插值）同时完成 surface 高度 + 块分类（省 50% 采样）
    for lz in 0..16 {
        for lx in 0..16 {
            let x = cx*16+lx; let z = cz*16+lz;
            let mut top = i32::MIN;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                if y >= noise_top { continue; } // 噪声高度以上留 Air（C++「y 循环上限 noiseHeight」修法）
                // density：宏观网格插值（快）或逐点采样（回退/对齐风险时）
                let d0 = match &chunk_density { Some(cd) => cd.sample(&NoisePos{x,y,z}), None => dense.sample(&NoisePos{x,y,z}) };
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

// ---- transpiler 宏观采样器（用 build-time 编译的 density 树采样，替换 DensityMacroSampler）----
// transpiler 生成代码（generated_density::fill_cell_corner_densities_final_density + compute_final_density）
// 把 final_density 编译成 native 代码（5 channels：1 BlendDensity + 4 RangeChoice noodle），
// 对齐 Java NoiseChunk cell grid：cell corners 采样 channels + 块级三线性插值 + combine。
// 与 DensityMacroSampler 语义一致（macrolize_channels 也是 5 channels），但用 NoiseSet 采样（非 DensityFunction 树）。
pub struct TranspilerDensity {
    noises: crate::noise::NoiseSet,
    min_y: i32, height: i32,
    cell_w: i32, cell_h: i32,
    gx: usize, gy: usize, gz: usize,
    nch: usize,
}
thread_local! {
    static TRANSPILER_SLICE_CACHE: RefCell<(i64, Vec<f64>)> = RefCell::new((i64::MIN, Vec::new()));
}
impl TranspilerDensity {
    pub fn new(noises: crate::noise::NoiseSet, min_y: i32, height: i32) -> Self {
        Self { noises, min_y, height, cell_w: 4, cell_h: 8,
            gx: (16/4+1) as usize, gy: (height/8+1) as usize, gz: (16/4+1) as usize,
            nch: 5 } // final_density 5 channels（1 BlendDensity + 4 RangeChoice noodle）
    }
    fn build_slices(&self, cx: i32, cz: i32) -> Vec<f64> {
        let nch = self.nch;
        let mut slices = vec![0.0f64; self.gx * self.gy * self.gz * nch];
        let mut out = vec![0.0f64; nch];
        for ix in 0..self.gx {
            for iz in 0..self.gz {
                for iy in 0..self.gy {
                    let px = cx*16 + ix as i32 * self.cell_w;
                    let py = self.min_y + iy as i32 * self.cell_h;
                    let pz = cz*16 + iz as i32 * self.cell_w;
                    crate::generated_density::fill_cell_corner_densities_final_density(&self.noises, px as f64, py as f64, pz as f64, &mut out);
                    for ch in 0..nch {
                        slices[((iy*self.gz + iz)*self.gx + ix)*nch + ch] = out[ch];
                    }
                }
            }
        }
        slices
    }
    #[inline]
    fn sample_interp_impl(&self, slices: &[f64], pos: &NoisePos) -> f64 {
        let gx = self.gx as i32; let gy = self.gy as i32; let gz = self.gz as i32;
        let chunk_x = pos.x.div_euclid(16); let chunk_z = pos.z.div_euclid(16);
        let gxx = pos.x - chunk_x*16; let gzz = pos.z - chunk_z*16; let gyy = pos.y - self.min_y;
        let mut cx = gxx / self.cell_w; let mut cy = gyy / self.cell_h; let mut cz = gzz / self.cell_w;
        cx = cx.clamp(0, gx-2); cy = cy.clamp(0, gy-2); cz = cz.clamp(0, gz-2);
        let fx = (gxx % self.cell_w) as f64 / self.cell_w as f64;
        let fy = (gyy % self.cell_h) as f64 / self.cell_h as f64;
        let fz = (gzz % self.cell_w) as f64 / self.cell_w as f64;
        let nch = self.nch;
        let at = |dx: i32, dy: i32, dz: i32, ch: usize| -> f64 {
            let cell_idx = ((cy+dy)*gz + (cz+dz))*gx + (cx+dx);
            slices[cell_idx as usize * nch + ch]
        };
        // 栈数组替代每 block heap Vec 分配（热路径 ~98304 次/chunk，scout §1.5）
        let mut interp = [0.0f64; 8];
        debug_assert!(nch <= 8);
        let nch = nch.min(8);
        for ch in 0..nch {
            let d000=at(0,0,0,ch); let d100=at(1,0,0,ch); let d010=at(0,1,0,ch); let d110=at(1,1,0,ch);
            let d001=at(0,0,1,ch); let d101=at(1,0,1,ch); let d011=at(0,1,1,ch); let d111=at(1,1,1,ch);
            let d00=d000+(d100-d000)*fx; let d10=d010+(d110-d010)*fx;
            let d01=d001+(d101-d001)*fx; let d11=d011+(d111-d011)*fx;
            let d0=d00+(d10-d00)*fy; let d1=d01+(d11-d01)*fy;
            interp[ch] = d0 + (d1 - d0)*fz;
        }
        crate::generated_density::compute_final_density(&self.noises, &interp[..nch], pos.x as f64, pos.y as f64, pos.z as f64)
    }
}
impl ChunkDensitySampler for TranspilerDensity {
    fn sample_interp(&self, slices: &[f64], pos: &NoisePos) -> f64 {
        self.sample_interp_impl(slices, pos)
    }
}
impl DensitySource<TranspilerDensity> for TranspilerDensity {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let chunk_x = pos.x.div_euclid(16); let chunk_z = pos.z.div_euclid(16);
        let key = ((chunk_x as i64) << 32) ^ (chunk_z as u32 as i64);
        TRANSPILER_SLICE_CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != key {
                c.0 = key;
                c.1 = self.build_slices(chunk_x, chunk_z);
            }
            self.sample_interp_impl(&c.1, pos)
        })
    }
    fn sample_chunk(&self, cx: i32, cz: i32, _min_y: i32, _height: i32) -> Option<ChunkDensity<'_, TranspilerDensity>> {
        let slices = self.build_slices(cx, cz);
        Some(ChunkDensity { sampler: self, slices })
    }
}

// ---- DFC 宏观采样器（WG_DFC 门控，lossless-accel P2a；默认关）----
// final_density 整树 DFC 直采（对齐 C++ WG_DFC_CPU 已验证形态）：split_top 热路径 + interp grid 缓存，
// 逐点 sample（sample_chunk=None → fill_chunk 逐点回退路径）。f32 采样语义（同源 GLSL/C++ 红线，
// 精度口径见 .investigations/lossless-accel/p2a-design-260903-03.md §3）。
pub struct DfcDensity {
    backend: crate::dfc_backend::DfcBackend,
}
impl DfcDensity {
    pub fn new(seed: u64) -> Self { DfcDensity { backend: crate::dfc_backend::DfcBackend::new(seed) } }
}
impl DensitySource<DfcDensity> for DfcDensity {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.backend.sample_point(pos.x, pos.y, pos.z) as f64
    }
}
// ChunkDensitySampler 仅 trait bound 需要（DfcDensity 走逐点路径，sample_chunk=None，永不调用）
impl ChunkDensitySampler for DfcDensity {
    fn sample_interp(&self, _slices: &[f64], _pos: &NoisePos) -> f64 {
        unreachable!("DfcDensity is per-point only")
    }
}
