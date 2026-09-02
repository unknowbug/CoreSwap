// dfc_backend.rs —— P2a DFC CPU 后端（Rust），lossless-accel 260903-03。
// 同源直译：.investigations/perf-rework/dfc_gen.py gen_cpu/gen_cpu_sampling（C++ CpuBackend，
// WG_DFC_CPU 已验证形态）；数据表由 gen_tables_rs.py 产 generated/dfc_cpu_tables.rs，
// split/split_top 生成体 include! 于本 impl 内（generated/dfc_cpu_split.rs）。
// 红线（gen_cpu_sampling docstring）：f32 采样语义、persistence 每 octave /2、interp_noise 双早停独立、
// pn_section_f32 y-fade 用 fadeY（第 7 值）、floorDiv 负坐标、mapPermD v&255、GRADIENTS f32、
// minY=-64、splitCoord 必须 per-thread、COORD_SLOT_TABLE 运行时查表勿 switch。
// 状态：draft（未验证）。

use crate::density::InterpolatedNoiseData;
use crate::legacy_random::RsSplitter;
use crate::noise::{DoublePerlinNoiseSampler, NoiseParameters, OctavePerlinNoiseSampler};
use crate::xoroshiro::XoroshiroRandom;
use std::cell::RefCell;

mod tbl {
    include!("generated/dfc_cpu_tables.rs");
}
use tbl::*;

// node type 常量（与 dfc_gen DF_* 枚举一致）
const T_CONSTANT: i32 = 0;
const T_Y: i32 = 1;
const T_NOISE: i32 = 2;
const T_OLD: i32 = 3;
const T_SPLINE: i32 = 4;
const T_INTERP: i32 = 5;
const T_ADD: i32 = 6;
const T_MUL: i32 = 7;
const T_MIN: i32 = 8;
const T_MAX: i32 = 9;
const T_ABS: i32 = 10;
const T_SQUARE: i32 = 11;
const T_CUBE: i32 = 12;
const T_HALF_NEG: i32 = 13;
const T_QUARTER_NEG: i32 = 14;
const T_SQUEEZE: i32 = 15;
const T_CLAMP: i32 = 16;
const T_RANGE_CHOICE: i32 = 17;
const T_Y_CLAMPED: i32 = 18;
const T_SHIFTED_NOISE: i32 = 19;
const T_BLEND_DENSITY: i32 = 20;
const T_FLAT_CACHE: i32 = 21;
const T_WEIRD: i32 = 22;

const GRADIENTS: [[f32; 3]; 16] = [
    [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0], [0.0, -1.0, 1.0], [-1.0, 1.0, 0.0], [0.0, -1.0, -1.0],
];

// ---- per-thread 缓冲（红线：splitCoord 必须 per-thread；grids 同 C++ thread_local 方案 i）----
thread_local! {
    static SPLIT_COORD: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static GRIDS: RefCell<Vec<GridSlot>> = const { RefCell::new(Vec::new()) };
}
pub static GRID_BUILDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct GridSlot {
    key: i64,
    grid: Vec<f32>,      // 49*5*5，索引 (gy*5+gz)*5+gx
    edge_cx: i32,
    edge_cz: i32,
    edge_col: Vec<f32>,  // 49*5，索引 gy*5+gz
}

impl GridSlot {
    fn new() -> Self {
        GridSlot { key: i64::MIN, grid: vec![0.0; 49 * 5 * 5], edge_cx: i32::MIN, edge_cz: i32::MIN, edge_col: vec![0.0; 49 * 5] }
    }
    #[inline] fn at(&self, gy: usize, gz: usize, gx: usize) -> f32 { self.grid[(gy * 5 + gz) * 5 + gx] }
    #[inline] fn set(&mut self, gy: usize, gz: usize, gx: usize, v: f32) { self.grid[(gy * 5 + gz) * 5 + gx] = v; }
}

pub struct DfcBackend {
    shifts: Vec<DoublePerlinNoiseSampler>,
    normals: Vec<DoublePerlinNoiseSampler>,
    olds: Vec<InterpolatedNoiseData>,
    perm: Vec<u32>,
}

impl DfcBackend {
    pub fn new(world_seed: u64) -> Self {
        // 对齐 C++ init：XoroshiroRandom base(worldSeed) → nextSplitter → rd.split(key)。
        // 采样器构造对齐 production（density_builder）：DoublePerlinNoiseSampler::new（modern）；
        // old_blended：rd.split("minecraft:terrain") → InterpolatedNoiseData（lower→upper→interp，均消费 rnd）。
        let rd = RsSplitter::Xoro(XoroshiroRandom::new(world_seed).next_splitter());
        let mut shifts = Vec::with_capacity(N_SHIFTS);
        for ni in SHIFT_INIT.iter() {
            let mut rnd = rd.split_str(ni.key);
            shifts.push(DoublePerlinNoiseSampler::new(&mut rnd, &NoiseParameters {
                first_octave: ni.first_octave, amplitudes: ni.amps.to_vec(),
            }));
        }
        let mut normals = Vec::with_capacity(N_NORMALS);
        for ni in NORMAL_INIT.iter() {
            let mut rnd = rd.split_str(ni.key);
            normals.push(DoublePerlinNoiseSampler::new(&mut rnd, &NoiseParameters {
                first_octave: ni.first_octave, amplitudes: ni.amps.to_vec(),
            }));
        }
        let mut olds = Vec::with_capacity(N_OLDS);
        for oi in OLD_INIT.iter() {
            let mut rnd = rd.split_str("minecraft:terrain");
            let amp_l = OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
            let lower = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
            let upper = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
            let amp_i = OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
            let interp = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
            olds.push(InterpolatedNoiseData::new(lower, upper, interp, oi.xz_scale, oi.y_scale, oi.xz_factor, oi.y_factor, oi.smear));
        }
        let mut be = DfcBackend { shifts, normals, olds, perm: Vec::new() };
        be.collect_perm();
        be
    }

    fn collect_perm(&mut self) {
        self.perm = vec![0u32; PERM_SIZE];
        // OLD_PACK 按 noise_instances 索引对齐：old 实例处 [octBase, splitBase]，其余 [0,0]
        let mut old_entries: Vec<(usize, usize)> = Vec::with_capacity(N_OLDS);
        for i in 0..NORMAL_INSTANCES {
            let ob = OLD_PACK[i * 2] as usize;
            let sb = OLD_PACK[i * 2 + 1] as usize;
            if ob == 0 && sb == 0 { continue; }
            old_entries.push((ob, sb));
        }
        debug_assert_eq!(old_entries.len(), N_OLDS, "OLD_PACK old 实例数不符");
        for (vi, (ob, _sb)) in old_entries.iter().enumerate() {
            let data = &self.olds[vi];
            for r in 0..16i32 {
                if let Some(pn) = data.lower.get_octave(r) {
                    for j in 0..256usize { self.perm[(ob + r as usize) * 256 + j] = pn.map(j as i32) as u32; }
                }
                if let Some(pn) = data.upper.get_octave(r) {
                    for j in 0..256usize { self.perm[(ob + 16 + r as usize) * 256 + j] = pn.map(j as i32) as u32; }
                }
            }
            for q in 0..8i32 {
                if let Some(pn) = data.interpolation.get_octave(q) {
                    for j in 0..256usize { self.perm[(ob + 32 + q as usize) * 256 + j] = pn.map(j as i32) as u32; }
                }
            }
        }
        // normal 实例 perm：NORMAL_PACK 按 noise_instances 索引 [n, octBase, splitBase]；
        // normals Vec 只含 normal（按 instances 序第 k 个 normal = normals[k]）
        let mut k = 0usize;
        for i in 0..NORMAL_INSTANCES {
            let n = NORMAL_PACK[i * 3] as usize;
            if n == 0 { continue; }
            let oct_base = NORMAL_PACK[i * 3 + 1] as usize;
            let noise = &self.normals[k];
            k += 1;
            for j in 0..n {
                if let Some(pn) = noise.first().octave_at(j) {
                    for jj in 0..256usize { self.perm[(oct_base + j) * 256 + jj] = pn.map(jj as i32) as u32; }
                }
                if let Some(pn) = noise.second().octave_at(j) {
                    for jj in 0..256usize { self.perm[(oct_base + n + j) * 256 + jj] = pn.map(jj as i32) as u32; }
                }
            }
        }
    }

    // ---- 原语 ----
    #[inline] fn floor_div(a: i32, b: i32) -> i32 { let r = a / b; if a % b != 0 && ((a ^ b) < 0) { r - 1 } else { r } }
    #[inline] fn maintain_precision(v: f64) -> f64 { v - ((v / 3.3554432e7 + 0.5) as i64) as f64 * 3.3554432e7 }

    // 双精度 ws_scale（split 侧 rarity 用；C++ CpuBackend::ws_scale double 版）
    fn ws_scale(kind: i32, v: f64) -> f64 {
        if kind == 1 {
            if v < -0.75 { return 0.5; }
            if v < -0.5 { return 0.75; }
            if v < 0.5 { return 1.0; }
            return if v < 0.75 { 2.0 } else { 3.0 };
        }
        if v < -0.5 { return 0.75; }
        if v < 0.0 { return 1.0; }
        if v < 0.5 { 1.5 } else { 2.0 }
    }

    // f32 版（eval 侧 DF_WEIRD 用；C++ ws_scaleF）
    #[inline] fn ws_scale_f(kind: i32, v: f32) -> f32 {
        if kind == 1 {
            if v < -0.75 { return 0.5; }
            if v < -0.5 { return 0.75; }
            if v < 0.5 { return 1.0; }
            return if v < 0.75 { 2.0 } else { 3.0 };
        }
        if v < -0.5 { return 0.75; }
        if v < 0.0 { return 1.0; }
        if v < 0.5 { 1.5 } else { 2.0 }
    }

    #[inline] fn map_perm_d(&self, oct_base: usize, v: i32) -> i32 { self.perm[oct_base * 256 + (v & 255) as usize] as i32 }
    #[inline] fn perlin_fade_f(v: f32) -> f32 { v * v * v * (v * (v * 6.0 - 15.0) + 10.0) }
    #[inline] fn lerp_f(d: f32, s: f32, e: f32) -> f32 { s + d * (e - s) }
    #[inline] fn grad_dot_f(&self, hash: i32, x: f32, y: f32, z: f32) -> f32 {
        let g = GRADIENTS[(hash & 15) as usize];
        g[0] * x + g[1] * y + g[2] * z
    }

    fn pn_sample3_f32(&self, oct_base: usize, sx: i32, sy: i32, sz: i32, lx: f32, ly: f32, lz: f32) -> f32 {
        let i0 = self.map_perm_d(oct_base, sx); let j = self.map_perm_d(oct_base, sx + 1);
        let k = self.map_perm_d(oct_base, i0 + sy); let l = self.map_perm_d(oct_base, i0 + sy + 1);
        let m = self.map_perm_d(oct_base, j + sy); let nn = self.map_perm_d(oct_base, j + sy + 1);
        let d = self.grad_dot_f(self.map_perm_d(oct_base, k + sz), lx, ly, lz);
        let e = self.grad_dot_f(self.map_perm_d(oct_base, m + sz), lx - 1.0, ly, lz);
        let f = self.grad_dot_f(self.map_perm_d(oct_base, l + sz), lx, ly - 1.0, lz);
        let g = self.grad_dot_f(self.map_perm_d(oct_base, nn + sz), lx - 1.0, ly - 1.0, lz);
        let h = self.grad_dot_f(self.map_perm_d(oct_base, k + sz + 1), lx, ly, lz - 1.0);
        let o = self.grad_dot_f(self.map_perm_d(oct_base, m + sz + 1), lx - 1.0, ly, lz - 1.0);
        let p = self.grad_dot_f(self.map_perm_d(oct_base, l + sz + 1), lx, ly - 1.0, lz - 1.0);
        let q = self.grad_dot_f(self.map_perm_d(oct_base, nn + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
        let r = Self::perlin_fade_f(lx); let s = Self::perlin_fade_f(ly); let t = Self::perlin_fade_f(lz);
        let x0 = Self::lerp_f(r, d, e); let x1 = Self::lerp_f(r, f, g);
        let x2 = Self::lerp_f(r, h, o); let x3 = Self::lerp_f(r, p, q);
        let y0 = Self::lerp_f(s, x0, x1); let y1 = Self::lerp_f(s, x2, x3);
        Self::lerp_f(t, y0, y1)
    }

    // old_blended 5 参数 sample：读 7 值拆分 [ix,iy,iz,gx,gy(h-n),gz,fadeY(h)]，y-fade 用 fadeY（红线）
    fn pn_section_f32(&self, oct_base: usize, s_idx: i32, split_offset: i32) -> f32 {
        SPLIT_COORD.with(|sc| {
            let sc = sc.borrow();
            let b = (s_idx * SPLIT_TOTAL as i32 + split_offset) as usize;
            let sx = sc[b] as i32; let sy = sc[b + 1] as i32; let sz = sc[b + 2] as i32;
            let lx = sc[b + 3]; let ly = sc[b + 4]; let lz = sc[b + 5];
            let fade_y = sc[b + 6];
            let i0 = self.map_perm_d(oct_base, sx); let j = self.map_perm_d(oct_base, sx + 1);
            let k = self.map_perm_d(oct_base, i0 + sy); let l = self.map_perm_d(oct_base, i0 + sy + 1);
            let m = self.map_perm_d(oct_base, j + sy); let nn = self.map_perm_d(oct_base, j + sy + 1);
            let d = self.grad_dot_f(self.map_perm_d(oct_base, k + sz), lx, ly, lz);
            let e = self.grad_dot_f(self.map_perm_d(oct_base, m + sz), lx - 1.0, ly, lz);
            let f = self.grad_dot_f(self.map_perm_d(oct_base, l + sz), lx, ly - 1.0, lz);
            let g = self.grad_dot_f(self.map_perm_d(oct_base, nn + sz), lx - 1.0, ly - 1.0, lz);
            let h = self.grad_dot_f(self.map_perm_d(oct_base, k + sz + 1), lx, ly, lz - 1.0);
            let o = self.grad_dot_f(self.map_perm_d(oct_base, m + sz + 1), lx - 1.0, ly, lz - 1.0);
            let p = self.grad_dot_f(self.map_perm_d(oct_base, l + sz + 1), lx, ly - 1.0, lz - 1.0);
            let q = self.grad_dot_f(self.map_perm_d(oct_base, nn + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
            let r = Self::perlin_fade_f(lx); let s = Self::perlin_fade_f(fade_y); let t = Self::perlin_fade_f(lz);
            let x0 = Self::lerp_f(r, d, e); let x1 = Self::lerp_f(r, f, g);
            let x2 = Self::lerp_f(r, h, o); let x3 = Self::lerp_f(r, p, q);
            let y0 = Self::lerp_f(s, x0, x1); let y1 = Self::lerp_f(s, x2, x3);
            Self::lerp_f(t, y0, y1)
        })
    }

    #[inline] fn y_clamped_gradient(y: i32, from_y: f32, to_y: f32, from_v: f32, to_v: f32) -> f32 {
        if to_y == from_y { return 0.0; }
        let t = 1.0f32.min(0.0f32.max((y as f32 - from_y) / (to_y - from_y)));
        from_v + t * (to_v - from_v)
    }

    // ---- 数据驱动噪声（读 A 表 + splitCoord + perm）----
    fn normal_noise(&self, noise_idx: usize, s_idx: i32) -> f32 {
        let b3 = noise_idx * 3;
        let n = NORMAL_PACK[b3] as usize;
        let oct_base = NORMAL_PACK[b3 + 1] as usize;
        let split_base = NORMAL_PACK[b3 + 2] as usize;
        let persistence = NORMAL_PACK_F[noise_idx * 2];
        let amplitude = NORMAL_PACK_F[noise_idx * 2 + 1];
        let amp_off = NORMAL_AMP_OFF[noise_idx] as usize;
        let mut d = 0.0f32; let mut f = persistence;
        SPLIT_COORD.with(|sc| {
        let sc = sc.borrow();
        for i in 0..n {
            let b = s_idx as usize * SPLIT_TOTAL + split_base + i * 6;
            let ix = sc[b] as i32; let iy = sc[b + 1] as i32; let iz = sc[b + 2] as i32;
            let gx = sc[b + 3]; let gy = sc[b + 4]; let gz = sc[b + 5];
            let ns = self.pn_sample3_f32(oct_base + i, ix, iy, iz, gx, gy, gz);
            d += NORMAL_AMPS[amp_off + i] * ns * f;
            f /= 2.0; // persistence 每 octave /2（红线）
        }
        let mut d2 = 0.0f32; let mut f = persistence;
        for i in 0..n {
            let b = s_idx as usize * SPLIT_TOTAL + split_base + 6 * n + i * 6;
            let ix = sc[b] as i32; let iy = sc[b + 1] as i32; let iz = sc[b + 2] as i32;
            let gx = sc[b + 3]; let gy = sc[b + 4]; let gz = sc[b + 5];
            let ns = self.pn_sample3_f32(oct_base + n + i, ix, iy, iz, gx, gy, gz);
            d2 += NORMAL_AMPS[amp_off + i] * ns * f;
            f /= 2.0;
        }
        (d + d2) * amplitude
        })
    }

    fn interp_noise(&self, idx: usize, s_idx: i32) -> f32 {
        let oct_base = OLD_PACK[idx * 2] as usize;
        let split_base = OLD_PACK[idx * 2 + 1] as usize;
        let mut n = 0.0f32; let mut o = 1.0f32;
        for q in 0..8usize {
            n += self.pn_section_f32(oct_base + 32 + q, s_idx, split_base as i32 + ((32 + q) * 7) as i32) / o;
            o /= 2.0;
        }
        let qq = (n / 10.0 + 1.0) / 2.0;
        let bl = qq >= 1.0; let bl2 = qq <= 0.0;
        let mut l = 0.0f32; let mut mm = 0.0f32; let mut o = 1.0f32;
        for r in 0..16usize {
            if !bl { l += self.pn_section_f32(oct_base + r, s_idx, split_base as i32 + (r * 7) as i32) / o; }       // 独立早停 1
            if !bl2 { mm += self.pn_section_f32(oct_base + 16 + r, s_idx, split_base as i32 + ((16 + r) * 7) as i32) / o; } // 独立早停 2
            o /= 2.0; // 除法每圈执行（红线）
        }
        let w = 1.0f32.min(0.0f32.max(qq));
        (l / 512.0 + w * (mm / 512.0 - l / 512.0)) / 128.0
    }

    // ---- split 侧（f64 拆分）----
    fn split_octave(pn: Option<&crate::noise::PerlinNoiseSampler>, cx: f64, cy: f64, cz: f64, out: &mut [f32], off: usize) {
        let (ox, oy, oz) = match pn { Some(p) => p.origin(), None => (0.0, 0.0, 0.0) };
        let ix = cx + ox; let iy = cy + oy; let iz = cz + oz;
        let fix = crate::noise::floor_d(ix); let fiy = crate::noise::floor_d(iy); let fiz = crate::noise::floor_d(iz);
        out[off] = fix as f32; out[off + 1] = fiy as f32; out[off + 2] = fiz as f32;
        out[off + 3] = (ix - fix as f64) as f32; out[off + 4] = (iy - fiy as f64) as f32; out[off + 5] = (iz - fiz as f64) as f32;
    }

    fn split_double(noise: &DoublePerlinNoiseSampler, dx: f64, dy: f64, dz: f64, out: &mut [f32], base: usize, nn: usize) {
        let lacunarity = 2.0f64.powi(noise.first().first_octave());
        let mut e = lacunarity;
        for i in 0..nn {
            Self::split_octave(noise.first().octave_at(i),
                Self::maintain_precision(dx * e), Self::maintain_precision(dy * e), Self::maintain_precision(dz * e),
                out, base + i * 6);
            Self::split_octave(noise.second().octave_at(i),
                Self::maintain_precision(dx * 1.0181268882175227 * e), Self::maintain_precision(dy * 1.0181268882175227 * e), Self::maintain_precision(dz * 1.0181268882175227 * e),
                out, base + 6 * nn + i * 6);
            e *= 2.0;
        }
    }

    // 5 参数 sample 拆分：out = [ix,iy,iz,gx,gy(=h-n),gz,fadeY(=h)]
    fn split7(pn: &crate::noise::PerlinNoiseSampler, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64, out: &mut [f32], off: usize) {
        let (ox, oy, oz) = pn.origin();
        let sx = x + ox; let sy = y + oy; let sz = z + oz;
        let ix = crate::noise::floor_d(sx); let iy = crate::noise::floor_d(sy); let iz = crate::noise::floor_d(sz);
        let gx = sx - ix as f64; let gy_raw = sy - iy as f64; let gz = sz - iz as f64;
        let n: f64;
        if y_scale != 0.0 {
            let m = if y_max >= 0.0 && y_max < gy_raw { y_max } else { gy_raw };
            n = crate::noise::floor_d(m / y_scale + (1.0e-7f32) as f64) as f64 * y_scale; // C++ 1.0E-7F（float 字面量）
        } else { n = 0.0; }
        out[off] = ix as f32; out[off + 1] = iy as f32; out[off + 2] = iz as f32;
        out[off + 3] = gx as f32; out[off + 4] = (gy_raw - n) as f32; out[off + 5] = gz as f32; out[off + 6] = gy_raw as f32;
    }

    fn split_old_blended(ob: &InterpolatedNoiseData, x: i32, y: i32, z: i32, out: &mut [f32], base: usize) {
        let d = x as f64 * ob.scaled_xz_scale;
        let e = y as f64 * ob.scaled_y_scale;
        let f = z as f64 * ob.scaled_xz_scale;
        let g = d / ob.xz_factor;
        let h = e / ob.y_factor;
        let i = f / ob.xz_factor;
        let j = ob.scaled_y_scale * ob.smear_scale_multiplier;
        let k = j / ob.y_factor;
        let mut o = 1.0f64;
        for q in 0..8usize {
            if let Some(pn) = ob.interpolation.get_octave(q as i32) {
                Self::split7(pn, Self::maintain_precision(g * o), Self::maintain_precision(h * o), Self::maintain_precision(i * o), k * o, h * o, out, base + (32 + q) * 7);
            }
            o /= 2.0;
        }
        let mut o = 1.0f64;
        for r in 0..16usize {
            let s2 = Self::maintain_precision(d * o); let t2 = Self::maintain_precision(e * o); let u2 = Self::maintain_precision(f * o);
            if let Some(pn) = ob.lower.get_octave(r as i32) {
                Self::split7(pn, s2, t2, u2, j * o, e * o, out, base + r * 7);
            }
            if let Some(pn) = ob.upper.get_octave(r as i32) {
                Self::split7(pn, s2, t2, u2, j * o, e * o, out, base + (16 + r) * 7);
            }
            o /= 2.0;
        }
    }

    // ---- spline（数据驱动表 + 显式栈 stage 机；D23 边界嵌套递归到 v0/v1 子帧）----
    fn spline_coord(&self, coord_type: usize, corner: i32, s_idx: i32, ix: i32, iy: i32, iz: i32) -> f32 {
        let slot = COORD_SLOT_TABLE[coord_type] as usize;
        let v = self.normal_noise(NOISE_SLOT_BASE[slot] as usize + (corner as usize) * NOISE_SLOT_STRIDE[slot] as usize, s_idx);
        spline_coord_fold(coord_type, v)
    }

    fn spline_find_range(x: f32, loc_begin: usize, n: i32) -> i32 {
        let mut mn = 0i32; let mut i = n;
        while i > 0 {
            let j = i / 2; let k = mn + j;
            if x < SPLINE_LOCS[loc_begin + k as usize] { i = j; } else { mn = k + 1; i -= j + 1; }
        }
        mn - 1
    }

    fn spline_hermite(coord: f32, lo: f32, span: f32, nv: f32, ov: f32, d0: f32, d1: f32) -> f32 {
        let kd = (coord - lo) / span;
        let p = d0 * span - (ov - nv);
        let q = -d1 * span + (ov - nv);
        (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p))
    }

    fn spline_eval(&self, root_node: usize, corner: i32, s_idx: i32, ix: i32, iy: i32, iz: i32) -> f32 {
        let mut st_node = [0i32; 64]; let mut st_i = [0i32; 64]; let mut st_stage = [0i32; 64];
        let mut st_coord = [0f32; 64]; let mut st_v0 = [0f32; 64]; let mut st_v1 = [0f32; 64];
        let mut sp = 0usize;
        st_node[0] = root_node as i32; st_stage[0] = 0; sp = 1;
        let mut out_val = 0.0f32;
        while sp > 0 {
            let fi = sp - 1;
            let node = st_node[fi] as usize;
            let p = node * 5;
            let ct = SPLINE_NODE_PACK[p] as usize;
            let n = SPLINE_NODE_PACK[p + 1] as i32;
            let loc_b = SPLINE_NODE_PACK[p + 2] as usize;
            let der_b = SPLINE_NODE_PACK[p + 3] as usize;
            let val_b = SPLINE_NODE_PACK[p + 4] as usize;
            match st_stage[fi] {
                0 => {
                    let coord = self.spline_coord(ct, corner, s_idx, ix, iy, iz);
                    let i = Self::spline_find_range(coord, loc_b, n);
                    st_coord[fi] = coord; st_i[fi] = i;
                    if i < 0 {
                        // D23：左边界外推遇嵌套 value 必须递归求值（非 0.0）
                        if SPLINE_VAL_KIND[val_b] == 0 {
                            out_val = SPLINE_VAL_F[val_b] + SPLINE_DERS[der_b] * (coord - SPLINE_LOCS[loc_b]);
                            sp -= 1;
                        } else {
                            st_stage[fi] = 4;
                            st_node[sp] = SPLINE_VAL_NODE[val_b]; st_stage[sp] = 0; sp += 1;
                        }
                    } else if i >= n - 1 {
                        // D23：右边界外推
                        if SPLINE_VAL_KIND[val_b + (n - 1) as usize] == 0 {
                            out_val = SPLINE_VAL_F[val_b + (n - 1) as usize]
                                + SPLINE_DERS[der_b + (n - 1) as usize] * (coord - SPLINE_LOCS[loc_b + (n - 1) as usize]);
                            sp -= 1;
                        } else {
                            st_stage[fi] = 5;
                            st_node[sp] = SPLINE_VAL_NODE[val_b + (n - 1) as usize]; st_stage[sp] = 0; sp += 1;
                        }
                    } else {
                        st_stage[fi] = 1;
                        if SPLINE_VAL_KIND[val_b + i as usize] == 0 {
                            st_v0[fi] = SPLINE_VAL_F[val_b + i as usize];
                            st_stage[fi] = 2;
                            if SPLINE_VAL_KIND[val_b + i as usize + 1] == 0 {
                                st_v1[fi] = SPLINE_VAL_F[val_b + i as usize + 1];
                                let lo = SPLINE_LOCS[loc_b + i as usize];
                                out_val = Self::spline_hermite(coord, lo, SPLINE_LOCS[loc_b + i as usize + 1] - lo, st_v0[fi], st_v1[fi], SPLINE_DERS[der_b + i as usize], SPLINE_DERS[der_b + i as usize + 1]);
                                sp -= 1;
                            } else {
                                st_stage[fi] = 3;
                                st_node[sp] = SPLINE_VAL_NODE[val_b + i as usize + 1]; st_stage[sp] = 0; sp += 1;
                            }
                        } else {
                            st_node[sp] = SPLINE_VAL_NODE[val_b + i as usize]; st_stage[sp] = 0; sp += 1;
                        }
                    }
                }
                4 => { // D23：边界 v0 子帧回填 → 左侧外推
                    let coord = st_coord[fi];
                    out_val += SPLINE_DERS[der_b] * (coord - SPLINE_LOCS[loc_b]);
                    sp -= 1;
                }
                5 => { // D23：边界 vn 子帧回填 → 右侧外推
                    let coord = st_coord[fi];
                    out_val += SPLINE_DERS[der_b + (n - 1) as usize] * (coord - SPLINE_LOCS[loc_b + (n - 1) as usize]);
                    sp -= 1;
                }
                1 => { // 等 v0 子帧回填
                    st_v0[fi] = out_val;
                    st_stage[fi] = 2;
                    let i = st_i[fi] as usize;
                    if SPLINE_VAL_KIND[val_b + i + 1] == 0 {
                        st_v1[fi] = SPLINE_VAL_F[val_b + i + 1];
                        let lo = SPLINE_LOCS[loc_b + i];
                        out_val = Self::spline_hermite(st_coord[fi], lo, SPLINE_LOCS[loc_b + i + 1] - lo, st_v0[fi], st_v1[fi], SPLINE_DERS[der_b + i], SPLINE_DERS[der_b + i + 1]);
                        sp -= 1;
                    } else {
                        st_stage[fi] = 3;
                        st_node[sp] = SPLINE_VAL_NODE[val_b + i + 1]; st_stage[sp] = 0; sp += 1;
                    }
                }
                2 => { // 瞬态（v0 回填后 v1 也齐）：完成 Hermite
                    st_v1[fi] = out_val;
                    let i = st_i[fi] as usize;
                    let lo = SPLINE_LOCS[loc_b + i];
                    out_val = Self::spline_hermite(st_coord[fi], lo, SPLINE_LOCS[loc_b + i + 1] - lo, st_v0[fi], st_v1[fi], SPLINE_DERS[der_b + i], SPLINE_DERS[der_b + i + 1]);
                    sp -= 1;
                }
                _ => { // 3：等 v1 子帧回填 → Hermite 完成
                    let v1 = out_val;
                    let i = st_i[fi] as usize;
                    let lo = SPLINE_LOCS[loc_b + i];
                    out_val = Self::spline_hermite(st_coord[fi], lo, SPLINE_LOCS[loc_b + i + 1] - lo, st_v0[fi], v1, SPLINE_DERS[der_b + i], SPLINE_DERS[der_b + i + 1]);
                    sp -= 1;
                }
            }
        }
        out_val
    }

    // ---- interp grid 缓存（path C：每 interp 每 chunk 5×49×5 去重网格 + 三线性）----
    fn build_interp_grid(&self, interp_idx: usize, chunk_x: i32, chunk_z: i32) {
        GRID_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        GRIDS.with(|g| {
            let mut g = g.borrow_mut();
            if g.len() < N_INTERP { g.resize_with(N_INTERP, GridSlot::new); }
        });
        // 保留外层 (block 位置) 的 split（反 interp 路径）——C++ splitCoord 保存/还原语义
        let saved = SPLIT_COORD.with(|sc| sc.borrow().clone());
        let reuse_left = {
            let ok = GRIDS.with(|g| {
                let g = g.borrow();
                let s = &g[interp_idx];
                s.edge_cx == chunk_x - 1 && s.edge_cz == chunk_z
            });
            if ok {
                GRIDS.with(|g| {
                    let mut g = g.borrow_mut();
                    let s = &mut g[interp_idx];
                    for gy in 0..49usize {
                        for gz in 0..5usize {
                            s.set(gy, gz, 0, s.edge_col[gy * 5 + gz]);
                        }
                    }
                });
            }
            ok
        };

        // per-cell split 去重（corner=0）：interior cell (cx,cy,cz) 的 (0,0,0) 角点 = 网格节点
        for gy in 0..48usize {
            for gz in 0..4usize {
                for gx in 0..4usize {
                    if gx == 0 && reuse_left { continue; }
                    let nx = chunk_x * 16 + gx as i32 * 4;
                    let ny = MIN_Y + gy as i32 * 8;
                    let nz = chunk_z * 16 + gz as i32 * 4;
                    SPLIT_COORD.with(|sc| {
                        let mut sc = sc.borrow_mut();
                        if sc.len() < SPLIT_TOTAL { sc.resize(SPLIT_TOTAL, 0.0); }
                        self.split(nx, ny, nz, &mut sc);
                    });
                    let v = self.eval_df_base(interp_idx, 0, 0, nx, ny, nz);
                    GRIDS.with(|g| g.borrow_mut()[interp_idx].set(gy, gz, gx, v));
                }
            }
        }
        // 边界列/行：gx=4、gz=4、gy=48
        for gy in 0..49usize {
            for gz in 0..5usize {
                for gx in 0..5usize {
                    if gx < 4 && gy < 48 && gz < 4 { continue; }
                    if gx == 0 && reuse_left { continue; }
                    let nx = chunk_x * 16 + gx as i32 * 4;
                    let ny = MIN_Y + gy as i32 * 8;
                    let nz = chunk_z * 16 + gz as i32 * 4;
                    SPLIT_COORD.with(|sc| {
                        let mut sc = sc.borrow_mut();
                        if sc.len() < SPLIT_TOTAL { sc.resize(SPLIT_TOTAL, 0.0); }
                        self.split(nx, ny, nz, &mut sc);
                    });
                    let v = self.eval_df_base(interp_idx, 0, 0, nx, ny, nz);
                    GRIDS.with(|g| g.borrow_mut()[interp_idx].set(gy, gz, gx, v));
                }
            }
        }
        SPLIT_COORD.with(|sc| { let mut sc = sc.borrow_mut(); sc.clear(); sc.extend_from_slice(&saved); });
        // 存 edgeCol（gx=4 列）供右邻复用
        GRIDS.with(|g| {
            let mut g = g.borrow_mut();
            let s = &mut g[interp_idx];
            for gy in 0..49usize {
                for gz in 0..5usize {
                    s.edge_col[gy * 5 + gz] = s.at(gy, gz, 4);
                }
            }
            s.edge_cx = chunk_x; s.edge_cz = chunk_z;
            s.key = ((chunk_x as i64) << 32) | ((chunk_z as i64) & 0xFFFFFFFF);
        });
    }

    fn sample_interp_grid(&self, interp_idx: usize, ix: i32, iy: i32, iz: i32) -> f32 {
        let chunk_x = Self::floor_div(ix, 16); let chunk_z = Self::floor_div(iz, 16);
        let key = ((chunk_x as i64) << 32) | ((chunk_z as i64) & 0xFFFFFFFF);
        GRIDS.with(|g| {
            let mut g = g.borrow_mut();
            if g.len() < N_INTERP { g.resize_with(N_INTERP, GridSlot::new); }
        });
        let built = GRIDS.with(|g| g.borrow()[interp_idx].key == key);
        if !built { self.build_interp_grid(interp_idx, chunk_x, chunk_z); }
        let gx = ix - chunk_x * 16; let gy = iy - MIN_Y; let gz = iz - chunk_z * 16;
        let cx = (gx / 4) as usize; let cy = (gy / 8) as usize; let cz = (gz / 4) as usize;
        let fx = (gx % 4) as f32 / 4.0; let fy = (gy % 8) as f32 / 8.0; let fz = (gz % 4) as f32 / 4.0;
        GRIDS.with(|g| {
            let s = &g.borrow()[interp_idx];
            let d000 = s.at(cy, cz, cx); let d100 = s.at(cy, cz, cx + 1);
            let d010 = s.at(cy + 1, cz, cx); let d110 = s.at(cy + 1, cz, cx + 1);
            let d001 = s.at(cy, cz + 1, cx); let d101 = s.at(cy, cz + 1, cx + 1);
            let d011 = s.at(cy + 1, cz + 1, cx); let d111 = s.at(cy + 1, cz + 1, cx + 1);
            let d00 = d000 + (d100 - d000) * fx; let d10 = d010 + (d110 - d010) * fx;
            let d01 = d001 + (d101 - d001) * fx; let d11 = d011 + (d111 - d011) * fx;
            let d0 = d00 + (d10 - d00) * fy; let d1 = d01 + (d11 - d01) * fy;
            d0 + (d1 - d0) * fz
        })
    }

    // ---- 解释器（D25：每 interp 只遍历自身 delegate 闭包）----
    fn eval_df_base(&self, interp_idx: usize, corner: i32, s_idx: i32, ix: i32, iy: i32, iz: i32) -> f32 {
        let mut val = [0f32; 64];
        debug_assert!(CLOSURE_MAX_SLOTS <= 64);
        let base = CLOSURE_OFF[interp_idx] as usize;
        let len = CLOSURE_LEN[interp_idx] as usize;
        for ci in 0..len {
            let gi = base + ci;
            let t = CLOSURE_TYPE[gi];
            let a1 = CLOSURE_A1[gi]; let a2 = CLOSURE_A2[gi]; let a3 = CLOSURE_A3[gi];
            let f0 = CLOSURE_F0[gi]; let f1 = CLOSURE_F1[gi]; let f2 = CLOSURE_F2[gi]; let f3 = CLOSURE_F3[gi];
            let slot0 = |x: i32| -> usize { CLOSURE_SLOT[base + x as usize] as usize };
            let r = match t {
                T_CONSTANT => f0,
                T_Y => iy as f32,
                T_NOISE | T_SHIFTED_NOISE =>
                    self.normal_noise(NOISE_SLOT_BASE[a1 as usize] as usize + (corner as usize) * NOISE_SLOT_STRIDE[a1 as usize] as usize, s_idx),
                T_OLD => self.interp_noise(NOISE_SLOT_BASE[a1 as usize] as usize + (corner as usize) * NOISE_SLOT_STRIDE[a1 as usize] as usize, s_idx),
                T_SPLINE => {
                    if a2 == 1 { self.spline_eval(a1 as usize, corner, s_idx, (ix >> 2) << 2, 0, (iz >> 2) << 2) }
                    else { self.spline_eval(a1 as usize, corner, s_idx, ix, iy, iz) }
                }
                T_Y_CLAMPED => Self::y_clamped_gradient(iy, f0, f1, f2, f3),
                T_ABS => val[slot0(a1)].abs(),
                T_SQUARE => { let v = val[slot0(a1)]; v * v }
                T_CUBE => { let v = val[slot0(a1)]; v * v * v }
                T_HALF_NEG => { let v = val[slot0(a1)]; if v > 0.0 { v } else { v * 0.5 } }
                T_QUARTER_NEG => { let v = val[slot0(a1)]; if v > 0.0 { v } else { v * 0.25 } }
                T_SQUEEZE => {
                    let v = val[slot0(a1)];
                    let c = if v > 1.0 { 1.0 } else if v < -1.0 { -1.0 } else { v };
                    c / 2.0 - c * c * c / 24.0
                }
                T_CLAMP => { let v = val[slot0(a1)]; if v > f1 { f1 } else if v < f0 { f0 } else { v } }
                T_RANGE_CHOICE => {
                    let inp = val[slot0(a1)];
                    if inp >= f0 && inp < f1 { val[slot0(a2)] } else { val[slot0(a3)] }
                }
                T_WEIRD => {
                    let v = val[slot0(a1)];
                    let d = Self::ws_scale_f(f0 as i32, v);
                    d * self.normal_noise(NOISE_SLOT_BASE[a2 as usize] as usize + (corner as usize) * NOISE_SLOT_STRIDE[a2 as usize] as usize, s_idx).abs()
                }
                T_BLEND_DENSITY | T_FLAT_CACHE => val[slot0(a1)],
                T_ADD => val[slot0(a1)] + val[slot0(a2)],
                T_MUL => val[slot0(a1)] * val[slot0(a2)],
                T_MIN => val[slot0(a1)].min(val[slot0(a2)]),
                T_MAX => val[slot0(a1)].max(val[slot0(a2)]),
                _ => 0.0, // DF_INTERP(5)：delegate 树不含，防御 0
            };
            val[CLOSURE_SLOT[base + ci] as usize] = r;
        }
        val[CLOSURE_SLOT[base + CLOSURE_ROOT_POS[interp_idx] as usize] as usize]
    }

    fn interp_n(&self, interp_idx: usize, s_idx: i32, ix: i32, iy: i32, iz: i32) -> f32 {
        if s_idx == 0 { return self.sample_interp_grid(interp_idx, ix, iy, iz); } // path C
        let chunk_x = Self::floor_div(ix, 16); let chunk_z = Self::floor_div(iz, 16);
        let gx = ix - chunk_x * 16; let gy = iy - MIN_Y; let gz = iz - chunk_z * 16;
        let cx = (gx / 4) as i32; let cy = (gy / 8) as i32; let cz = (gz / 4) as i32;
        let fx = (gx % 4) as f32 / 4.0; let fy = (gy % 8) as f32 / 8.0; let fz = (gz % 4) as f32 / 4.0;
        let mut d = [0f32; 8];
        for c in 0..8usize {
            let dx = (c & 1) as i32; let dy = ((c >> 1) & 1) as i32; let dz = ((c >> 2) & 1) as i32;
            let ax = chunk_x * 16 + (cx + dx) * 4;
            let ay = MIN_Y + (cy + dy) * 8;
            let az = chunk_z * 16 + (cz + dz) * 4;
            d[c] = self.eval_df_base(interp_idx, c as i32, s_idx, ax, ay, az);
        }
        let d00 = d[0] + (d[1] - d[0]) * fx; let d10 = d[2] + (d[3] - d[2]) * fx;
        let d01 = d[4] + (d[5] - d[4]) * fx; let d11 = d[6] + (d[7] - d[6]) * fx;
        let d0 = d00 + (d10 - d00) * fy; let d1 = d01 + (d11 - d01) * fy;
        d0 + (d1 - d0) * fz
    }

    fn eval_df(&self, s_idx: i32, ix: i32, iy: i32, iz: i32) -> f32 {
        let mut val = [0f32; 64];
        debug_assert!(VAL_SLOTS_TOP <= 64);
        for ci in 0..TOP_CLOSURE_LEN {
            let t = TOP_TYPE[ci];
            let a1 = TOP_A1[ci]; let a2 = TOP_A2[ci]; let a3 = TOP_A3[ci];
            let f0 = TOP_F0[ci]; let f1 = TOP_F1[ci]; let f2 = TOP_F2[ci]; let f3 = TOP_F3[ci];
            let slotc = TOP_SLOT[ci] as usize;
            if t == T_INTERP {
                val[slotc] = self.interp_n(a1 as usize, s_idx, ix, iy, iz);
                continue;
            }
            let slot0 = |x: i32| -> usize { TOP_SLOT[x as usize] as usize };
            let r = match t {
                T_CONSTANT => f0,
                T_Y => iy as f32,
                T_NOISE | T_SHIFTED_NOISE => self.normal_noise(NOISE_SLOT_BASE[a1 as usize] as usize, s_idx), // corner=0
                T_OLD => self.interp_noise(NOISE_SLOT_BASE[a1 as usize] as usize, s_idx),
                T_SPLINE => {
                    if a2 == 1 { self.spline_eval(a1 as usize, 0, s_idx, (ix >> 2) << 2, 0, (iz >> 2) << 2) }
                    else { self.spline_eval(a1 as usize, 0, s_idx, ix, iy, iz) }
                }
                T_Y_CLAMPED => Self::y_clamped_gradient(iy, f0, f1, f2, f3),
                T_ABS => val[slot0(a1)].abs(),
                T_SQUARE => { let v = val[slot0(a1)]; v * v }
                T_CUBE => { let v = val[slot0(a1)]; v * v * v }
                T_HALF_NEG => { let v = val[slot0(a1)]; if v > 0.0 { v } else { v * 0.5 } }
                T_QUARTER_NEG => { let v = val[slot0(a1)]; if v > 0.0 { v } else { v * 0.25 } }
                T_SQUEEZE => {
                    let v = val[slot0(a1)];
                    let c = if v > 1.0 { 1.0 } else if v < -1.0 { -1.0 } else { v };
                    c / 2.0 - c * c * c / 24.0
                }
                T_CLAMP => { let v = val[slot0(a1)]; if v > f1 { f1 } else if v < f0 { f0 } else { v } }
                T_RANGE_CHOICE => {
                    let inp = val[slot0(a1)];
                    if inp >= f0 && inp < f1 { val[slot0(a2)] } else { val[slot0(a3)] }
                }
                T_WEIRD => {
                    let v = val[slot0(a1)];
                    let d = Self::ws_scale_f(f0 as i32, v);
                    d * self.normal_noise(NOISE_SLOT_BASE[a2 as usize] as usize, s_idx).abs()
                }
                T_BLEND_DENSITY | T_FLAT_CACHE => val[slot0(a1)],
                T_ADD => val[slot0(a1)] + val[slot0(a2)],
                T_MUL => val[slot0(a1)] * val[slot0(a2)],
                T_MIN => val[slot0(a1)].min(val[slot0(a2)]),
                T_MAX => val[slot0(a1)].max(val[slot0(a2)]),
                _ => 0.0,
            };
            val[slotc] = r;
        }
        val[TOP_SLOT[TOP_ROOT_POS] as usize]
    }

    // ---- 公共入口：单点 final_density 采样（split_top 热路径 + eval sIdx=0）----
    pub fn sample_point(&self, x: i32, y: i32, z: i32) -> f32 {
        SPLIT_COORD.with(|sc| {
            let mut sc = sc.borrow_mut();
            if sc.len() < SPLIT_TOTAL { sc.resize(SPLIT_TOTAL, 0.0); }
            self.split_top(x, y, z, &mut sc);
        });
        self.eval_df(0, x, y, z)
    }

    // ---- 调试钩子（doc(hidden)，诊断 bin 专用；勿进生产热路径）----
    #[doc(hidden)]
    pub fn dbg_split_full(&self, x: i32, y: i32, z: i32) {
        SPLIT_COORD.with(|sc| {
            let mut sc = sc.borrow_mut();
            if sc.len() < SPLIT_TOTAL { sc.resize(SPLIT_TOTAL, 0.0); }
            self.split(x, y, z, &mut sc);
        });
    }
    #[doc(hidden)] pub fn dbg_normal_noise(&self, idx: usize, s_idx: i32) -> f32 { self.normal_noise(idx, s_idx) }
    #[doc(hidden)] pub fn dbg_interp_noise(&self, idx: usize, s_idx: i32) -> f32 { self.interp_noise(idx, s_idx) }
    #[doc(hidden)] pub fn dbg_interp_n(&self, k: usize, x: i32, y: i32, z: i32) -> f32 { self.interp_n(k, 0, x, y, z) }
    #[doc(hidden)] pub fn dbg_eval_base(&self, k: usize, x: i32, y: i32, z: i32) -> f32 { self.eval_df_base(k, 0, 0, x, y, z) }
    #[doc(hidden)] pub fn dbg_n_normals(&self) -> usize { self.normals.len() }
    #[doc(hidden)] pub fn dbg_normal_raw(&self, idx: usize, x: f64, y: f64, z: f64) -> f64 { self.normals[idx].sample(x, y, z) }
    #[doc(hidden)]
    pub fn dump_closures(&self) {        for k in 0..N_CLOSURE {
            let base = CLOSURE_OFF[k] as usize;
            let len = CLOSURE_LEN[k] as usize;
            println!("closure[{}] off={} len={} peak={} root_pos={} root_slot={}",
                k, base, len, CLOSURE_VAL_SLOTS[k], CLOSURE_ROOT_POS[k],
                CLOSURE_SLOT[base + CLOSURE_ROOT_POS[k] as usize]);
            for ci in 0..len {
                let gi = base + ci;
                println!("  ci={} slot={} type={} a1={} a2={} a3={} f0={} f1={} f2={} f3={}",
                    ci, CLOSURE_SLOT[gi], CLOSURE_TYPE[gi], CLOSURE_A1[gi], CLOSURE_A2[gi], CLOSURE_A3[gi],
                    CLOSURE_F0[gi], CLOSURE_F1[gi], CLOSURE_F2[gi], CLOSURE_F3[gi]);
            }
        }
        println!("TOP len={} root_pos={} root_slot={}", TOP_CLOSURE_LEN, TOP_ROOT_POS, TOP_SLOT[TOP_ROOT_POS]);
        for ci in 0..TOP_CLOSURE_LEN {
            println!("  top ci={} slot={} type={} a1={} a2={} a3={} f0={} f1={} f2={} f3={}",
                ci, TOP_SLOT[ci], TOP_TYPE[ci], TOP_A1[ci], TOP_A2[ci], TOP_A3[ci],
                TOP_F0[ci], TOP_F1[ci], TOP_F2[ci], TOP_F3[ci]);
        }
    }
}

// split/split_top 生成体（gen_tables_rs.py 产物；自带 impl DfcBackend 包裹，引用 self + Self:: 辅助）
include!("generated/dfc_cpu_split.rs");
