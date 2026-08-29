// density.rs — DF 树（enum DensityFunction 数据驱动，替代 C++ 虚调用树——Rust 优势）
// Phase 3 第 1 波：基础 op；第 2 波：SplineDF（Hermite）+ InterpolatedDF（grid 懒建缓存）。
// 逐位对齐 C++ density.h（Hermite BK-001 / InterpolatedDF 4x4x8 cell 插值）。
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::noise::DoublePerlinNoiseSampler;
use crate::noise::OctavePerlinNoiseSampler;

// 缓存节点 id 分配（生产化：缓存改 thread_local，节点需稳定 id）
static NEXT_CACHE_ID: AtomicU32 = AtomicU32::new(0);
// 诊断：build_grid 的 arg 采样总次数（定位交替插值嵌套递归网格构建膨胀）
#[doc(hidden)]
pub static GRID_ARG_SAMPLES: AtomicU32 = AtomicU32::new(0);

/// ShiftDF（offset noise shift）构造：分配缓存 id（y 无关 Cache2D 缓存）。
pub fn shift_df(noise: Arc<DoublePerlinNoiseSampler>, mode: ShiftMode) -> DensityFunction {
    let id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
    DensityFunction::ShiftDF { noise, mode, id }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Mul, Min, Max }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Abs, Square, Cube, HalfNegative, QuarterNegative, Squeeze }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShiftMode { Shift, ShiftA, ShiftB }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WeirdRarity { Tunnels, Caves }

pub struct NoisePos { pub x: i32, pub y: i32, pub z: i32 }

fn floor_div(a: i32, b: i32) -> i32 { let r = a / b; if (a % b) != 0 && ((a ^ b) < 0) { r - 1 } else { r } }
pub fn clamp_d(v: f64, lo: f64, hi: f64) -> f64 { if v < lo { lo } else if v > hi { hi } else { v } }
pub fn apply_unary(op: UnaryOp, x: f64) -> f64 {
    match op {
        UnaryOp::Abs => x.abs(),
        UnaryOp::Square => x * x,
        UnaryOp::Cube => x * x * x,
        UnaryOp::HalfNegative => if x > 0.0 { x } else { 0.5 * x },
        UnaryOp::QuarterNegative => if x > 0.0 { x } else { 0.25 * x },
        UnaryOp::Squeeze => { let d = clamp_d(x, -1.0, 1.0); d / 2.0 - d * d * d / 24.0 }
    }
}

// ---- helper（对齐 C++ YClampedGradient::clampedMap L324-330 / WeirdScaledSampler::scaleValue L352-363）----
pub struct YClampedGradient;
impl YClampedGradient {
    pub fn clamped_map(v: f64, a: i32, b: i32, c: f64, d: f64) -> f64 {
        if a == b { return (c + d) / 2.0; }
        if v < a as f64 { return c; }
        if v > b as f64 { return d; }
        c + (v - a as f64) / (b - a) as f64 * (d - c)
    }
}
pub struct WeirdScaled;
impl WeirdScaled {
    pub fn scale_value(r: WeirdRarity, v: f64) -> f64 {
        if r == WeirdRarity::Caves {
            if v < -0.75 { return 0.5; }
            if v < -0.5 { return 0.75; }
            if v < 0.5 { return 1.0; }
            return if v < 0.75 { 2.0 } else { 3.0 };
        } else {
            if v < -0.5 { return 0.75; }
            if v < 0.0 { return 1.0; }
            return if v < 0.5 { 1.5 } else { 2.0 };
        }
    }
}

// ---- SplineDF 数据（C++ density.h SplineDF 扁平表）----
#[derive(Clone)]
pub struct SplineNode {
    pub loc_fn: i32,
    pub loc_begin: i32,
    pub sub_begin: i32,
    pub n: i32,
    pub fixed_value: f32,
}
#[derive(Clone)]
pub struct SplineData {
    pub nodes: Vec<SplineNode>,
    pub locations: Vec<f32>,
    pub derivatives: Vec<f32>,
    pub sub_idx: Vec<i32>,
    pub loc_fns: Vec<Arc<DensityFunction>>,
    pub root: i32,
    pub min_val: f64,
    pub max_val: f64,
}
impl SplineData {
    fn sample_node(&self, node_id: i32, pos: &NoisePos) -> f64 {
        let nd = &self.nodes[node_id as usize];
        if nd.n == 0 { return nd.fixed_value as f64; }
        let f = self.loc_fns[nd.loc_fn as usize].sample(pos);
        let locs = &self.locations[nd.loc_begin as usize..];
        let ders = &self.derivatives[nd.loc_begin as usize..];
        let subs = &self.sub_idx[nd.sub_begin as usize..];
        let n = nd.n;
        let mut lo = 0i32; let mut hi = n;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if f < locs[mid as usize] as f64 { hi = mid; } else { lo = mid + 1; }
        }
        let i = lo - 1;
        let r: f64;
        if i < 0 {
            let d = ders[0] as f64;
            let base = self.sample_node(subs[0], pos);
            r = base + d * (f - locs[0] as f64);
        } else if i == n - 1 {
            let idx = (n - 1) as usize;
            let d = ders[idx] as f64;
            let base = self.sample_node(subs[idx], pos);
            r = base + d * (f - locs[idx] as f64);
        } else {
            let k = i as usize;
            let g = locs[k] as f64; let h = locs[k + 1] as f64;
            let kd = (f - g) / (h - g);
            let nv = self.sample_node(subs[k], pos);
            let ov = self.sample_node(subs[k + 1], pos);
            let l = ders[k] as f64; let m = ders[k + 1] as f64;
            let p = l * (h - g) - (ov - nv);
            let q = -m * (h - g) + (ov - nv);
            r = lerp64(kd, nv, ov) + kd * (1.0 - kd) * lerp64(kd, p, q);
        }
        r
    }
    pub fn node_min(&self, node_id: i32) -> f64 {
        let nd = &self.nodes[node_id as usize];
        if nd.n == 0 { return nd.fixed_value as f64; }
        let subs = &self.sub_idx[nd.sub_begin as usize..];
        let mut m = f64::INFINITY;
        for k in 0..nd.n as usize { m = m.min(self.node_min(subs[k])); }
        m
    }
    pub fn node_max(&self, node_id: i32) -> f64 {
        let nd = &self.nodes[node_id as usize];
        if nd.n == 0 { return nd.fixed_value as f64; }
        let subs = &self.sub_idx[nd.sub_begin as usize..];
        let mut m = f64::NEG_INFINITY;
        for k in 0..nd.n as usize { m = m.max(self.node_max(subs[k])); }
        m
    }
}
fn lerp64(d: f64, s: f64, e: f64) -> f64 { s + d * (e - s) }

// ---- InterpolatedNoiseData（minecraft:old_blended_noise，C++ InterpolatedNoiseDF L383-476）----
// 三个 OctavePerlinNoiseSampler（lower/upper/interpolation），均藏于 Rc（DensityFunction 需 Clone）。
#[derive(Clone)]
pub struct InterpolatedNoiseData {
    pub lower: Arc<OctavePerlinNoiseSampler>,
    pub upper: Arc<OctavePerlinNoiseSampler>,
    pub interpolation: Arc<OctavePerlinNoiseSampler>,
    pub xz_scale: f64,
    pub y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
    pub scaled_xz_scale: f64,
    pub scaled_y_scale: f64,
    pub max_val: f64,
}
impl InterpolatedNoiseData {
    pub fn new(lower: OctavePerlinNoiseSampler, upper: OctavePerlinNoiseSampler, interpolation: OctavePerlinNoiseSampler,
               xz_scale: f64, y_scale: f64, xz_factor: f64, y_factor: f64, smear_scale_multiplier: f64) -> Self {
        let scaled_xz_scale = (684.412 as f32) as f64 * xz_scale; // Java: 684.412F
        let scaled_y_scale = (684.412 as f32) as f64 * y_scale;
        let max_val = lower.method_40556(scaled_y_scale);
        InterpolatedNoiseData { lower: Arc::new(lower), upper: Arc::new(upper), interpolation: Arc::new(interpolation),
                               xz_scale, y_scale, xz_factor, y_factor, smear_scale_multiplier, scaled_xz_scale, scaled_y_scale, max_val }
    }
    // 对齐 C++ sampleImpl L411-473
    pub fn sample(&self, pos: &NoisePos) -> f64 {
        let d = pos.x as f64 * self.scaled_xz_scale;
        let e = pos.y as f64 * self.scaled_y_scale;
        let f = pos.z as f64 * self.scaled_xz_scale;
        let g = d / self.xz_factor;
        let h = e / self.y_factor;
        let i = f / self.xz_factor;
        let j = self.scaled_y_scale * self.smear_scale_multiplier;
        let k = j / self.y_factor;
        let mut l = 0.0f64; let mut m = 0.0f64; let mut n = 0.0f64;
        let mut o = 1.0f64;
        for p in 0..8 {
            if let Some(pn) = self.interpolation.get_octave(p) {
                let go = OctavePerlinNoiseSampler::maintain_precision(g * o);
                let ho = OctavePerlinNoiseSampler::maintain_precision(h * o);
                let io = OctavePerlinNoiseSampler::maintain_precision(i * o);
                let r0 = pn.sample_ys(go, ho, io, k * o, h * o);
                n += r0 / o;
            }
            o /= 2.0;
        }
        let q = (n / 10.0 + 1.0) / 2.0;
        let bl2 = q >= 1.0;
        let bl3 = q <= 0.0;
        o = 1.0;
        for r in 0..16 {
            let s = OctavePerlinNoiseSampler::maintain_precision(d * o);
            let t = OctavePerlinNoiseSampler::maintain_precision(e * o);
            let u = OctavePerlinNoiseSampler::maintain_precision(f * o);
            let v = j * o;
            if !bl2 {
                if let Some(pn) = self.lower.get_octave(r) {
                    let r0 = pn.sample_ys(s, t, u, v, e * o);
                    l += r0 / o;
                }
            }
            if !bl3 {
                if let Some(pn) = self.upper.get_octave(r) {
                    let r0 = pn.sample_ys(s, t, u, v, e * o);
                    m += r0 / o;
                }
            }
            o /= 2.0;
        }
        let qq = clamp_d(q, 0.0, 1.0);
        (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0
    }
}

// ---- InterpolatedDF 数据（grid 懒建缓存——生产化：thread_local 每线程缓存，跨线程共享树无 cache-line 争用）----
const CELL_X: i32 = 4; const CELL_Y: i32 = 8; const CELL_Z: i32 = 4;
#[derive(Clone)]
pub struct InterpSlot { pub key: i64, pub grid: Vec<f64> }
thread_local! {
    // 性能（2026-08-27 profile：5.68μs/pt vs C++ 0.5μs 的主因 = 每 sample 的 HashMap entry 查找 + TLS 重入）：
    // HashMap<u32, Box<RefCell<Slot>>> → Vec<Box<RefCell<Slot>>> 按 cacheId 直接下标（id 递增唯一），
    // slot_ptr 一次取出后循环内复用，消除每次 sample 的 hash/entry 开销。
    static INTERP_CACHE: RefCell<Vec<Option<Box<RefCell<InterpSlot>>>>> = RefCell::new(Vec::new());
}
#[derive(Clone)]
pub struct InterpolatedData {
    pub arg: Arc<DensityFunction>,
    pub min_y: i32,
    pub height: i32,
    pub id: u32,
    pub mn: f64,
    pub mx: f64,
}
impl InterpolatedData {
    pub fn new(arg: Arc<DensityFunction>, min_y: i32, height: i32) -> Self {
        let id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
        let mn = arg.min_value(); let mx = arg.max_value();
        InterpolatedData { arg, min_y, height, id, mn, mx }
    }
    // 每线程缓存槽（Box 保证节点 id 的 slot 地址稳定；slot_ptr 释放 Vec 借用，仅持 slot RefCell 借用）
    #[inline]
    fn slot_ptr(&self) -> *const RefCell<InterpSlot> {
        INTERP_CACHE.with(|m| {
            let mut m = m.borrow_mut();
            if m.len() <= self.id as usize { m.resize(self.id as usize + 1, None); }
            let e = m[self.id as usize].get_or_insert_with(|| Box::new(RefCell::new(InterpSlot { key: i64::MIN, grid: Vec::new() })));
            &**e as *const RefCell<InterpSlot>
        })
    }
    fn build_grid(&self, chunk_x: i32, chunk_z: i32) -> Vec<f64> {
        let gx = 16 / CELL_X + 1;
        let gy = self.height / CELL_Y + 1;
        let gz = 16 / CELL_Z + 1;
        let mut grid = vec![0.0f64; (gx * gy * gz) as usize];
        for iy in 0..gy {
            for iz in 0..gz {
                for ix in 0..gx {
                    let px = chunk_x * 16 + ix * CELL_X;
                    let py = self.min_y + iy * CELL_Y;
                    let pz = chunk_z * 16 + iz * CELL_Z;
                    let pos = NoisePos { x: px, y: py, z: pz };
                    grid[((iy * gz + iz) * gx + ix) as usize] = self.arg.sample(&pos);
                    GRID_ARG_SAMPLES.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        grid
    }
    fn sample(&self, pos: &NoisePos) -> f64 {
        let chunk_x = floor_div(pos.x, 16);
        let chunk_z = floor_div(pos.z, 16);
        let key = (((chunk_x as u64) << 32) ^ (chunk_z as u32 as u64)) as i64;
        let ptr = self.slot_ptr();
        let slot = unsafe { &*ptr };
        let mut slot = slot.borrow_mut();
        if slot.key != key {
            slot.key = key;
            slot.grid = self.build_grid(chunk_x, chunk_z);
        }
        let gx = 16 / CELL_X + 1;
        let gy = self.height / CELL_Y + 1;
        let gz = 16 / CELL_Z + 1;
        let gxx = pos.x - chunk_x * 16;
        let gyy = pos.y - self.min_y;
        let gzz = pos.z - chunk_z * 16;
        let mut cx = gxx / CELL_X;
        let mut cy = gyy / CELL_Y;
        let mut cz = gzz / CELL_Z;
        if cx < 0 || cy < 0 || cz < 0 || cx >= gx - 1 || cy >= gy - 1 || cz >= gz - 1 {
            cx = if cx < 0 { 0 } else if cx > gx - 2 { gx - 2 } else { cx };
            cy = if cy < 0 { 0 } else if cy > gy - 2 { gy - 2 } else { cy };
            cz = if cz < 0 { 0 } else if cz > gz - 2 { gz - 2 } else { cz };
        }
        let fx = (gxx % CELL_X) as f64 / CELL_X as f64;
        let fy = (gyy % CELL_Y) as f64 / CELL_Y as f64;
        let fz = (gzz % CELL_Z) as f64 / CELL_Z as f64;
        let at = |dx: i32, dy: i32, dz: i32| -> f64 {
            slot.grid[(((cy + dy) * gz + (cz + dz)) * gx + (cx + dx)) as usize]
        };
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

// ---- Cache2DDF（16 槽 LRU，y 无关 2D 缓存——生产化 thread_local 每线程）----
const CACHE2D_CAP: usize = 256;
#[derive(Clone)]
pub struct Cache2DSlot { keys: [i64; CACHE2D_CAP], values: [f64; CACHE2D_CAP], stamps: [u64; CACHE2D_CAP], tick: u64 }
thread_local! {
    // 性能：HashMap → Vec 直接下标（同 INTERP_CACHE，消除每 sample 的 hash/entry 开销）
    static C2D_CACHE: RefCell<Vec<Option<Box<RefCell<Cache2DSlot>>>>> = RefCell::new(Vec::new());
}
#[derive(Clone)]
pub struct Cache2DData { pub arg: Arc<DensityFunction>, pub id: u32, pub mn: f64, pub mx: f64 }
impl Cache2DData {
    pub fn new(arg: Arc<DensityFunction>) -> Self {
        let id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
        let mn = arg.min_value(); let mx = arg.max_value();
        Cache2DData { arg, id, mn, mx }
    }
    #[inline]
    fn slot_ptr(&self) -> *const RefCell<Cache2DSlot> {
        C2D_CACHE.with(|m| {
            let mut m = m.borrow_mut();
            if m.len() <= self.id as usize { m.resize(self.id as usize + 1, None); }
            let e = m[self.id as usize].get_or_insert_with(|| Box::new(RefCell::new(Cache2DSlot { keys: [i64::MIN; CACHE2D_CAP], values: [0.0; CACHE2D_CAP], stamps: [0; CACHE2D_CAP], tick: 0 })));
            &**e as *const RefCell<Cache2DSlot>
        })
    }
    pub fn sample(&self, pos: &NoisePos) -> f64 {
        let key = ((((pos.x as u32) as u64) << 32) ^ (pos.z as u32 as u64)) as i64;
        let ptr = self.slot_ptr();
        let slot = unsafe { &*ptr };
        let mut slot = slot.borrow_mut();
        for i in 0..CACHE2D_CAP {
            if slot.keys[i] == key { slot.stamps[i] = slot.tick; slot.tick += 1; return slot.values[i]; }
        }
        let v = self.arg.sample(pos);
        let mut idx = 0;
        for i in 1..CACHE2D_CAP { if slot.stamps[i] < slot.stamps[idx] { idx = i; } }
        slot.keys[idx] = key; slot.values[idx] = v; slot.stamps[idx] = slot.tick; slot.tick += 1;
        v
    }
}
// ---- FlatCacheDF（5x5 网格，chunk 级，y 无关——生产化 thread_local 每线程）----
const FLAT_GRID: usize = 5;
#[derive(Clone)]
pub struct FlatSlot { key: i64, cx: i32, cz: i32, grid: [f64; 25] }
thread_local! {
    // 性能：HashMap → Vec 直接下标（同 INTERP_CACHE）
    static FLAT_CACHE: RefCell<Vec<Option<Box<RefCell<FlatSlot>>>>> = RefCell::new(Vec::new());
}
// ShiftDF（offset noise，实测 y 无关）：按 (cx,cz) 缓存单值（y 无关 → Cache2D 语义）
#[derive(Clone)]
pub struct ShiftSlot { cx: i32, cz: i32, value: f64 }
thread_local! {
    static SHIFT_CACHE: RefCell<Vec<Option<Box<RefCell<ShiftSlot>>>>> = RefCell::new(Vec::new());
}
#[derive(Clone)]
pub struct FlatCacheData { pub arg: Arc<DensityFunction>, pub id: u32, pub mn: f64, pub mx: f64 }
impl FlatCacheData {
    pub fn new(arg: Arc<DensityFunction>) -> Self {
        let id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
        let mn = arg.min_value(); let mx = arg.max_value();
        FlatCacheData { arg, id, mn, mx }
    }
    #[inline]
    fn slot_ptr(&self) -> *const RefCell<FlatSlot> {
        FLAT_CACHE.with(|m| {
            let mut m = m.borrow_mut();
            if m.len() <= self.id as usize { m.resize(self.id as usize + 1, None); }
            let e = m[self.id as usize].get_or_insert_with(|| Box::new(RefCell::new(FlatSlot { key: i64::MIN, cx: 0, cz: 0, grid: [0.0; 25] })));
            &**e as *const RefCell<FlatSlot>
        })
    }
    pub fn sample(&self, pos: &NoisePos) -> f64 {
        // key = chunk（POC 用 pos>>4；生产 fill 设 g_cur_chunk thread_local 对齐 Java startBiomeX）
        let cx = pos.x >> 4; let cz = pos.z >> 4;
        let key = ((((cx as u32) as u64) << 32) ^ (cz as u32 as u64)) as i64;
        let ptr = self.slot_ptr();
        let slot = unsafe { &*ptr };
        let mut slot = slot.borrow_mut();
        if slot.key != key {
            slot.key = key; slot.cx = cx; slot.cz = cz;
            Self::build_grid(&self.arg, slot.cx, slot.cz, &mut slot.grid);
        }
        let k = (pos.x >> 2) - slot.cx * 4;
        let l = (pos.z >> 2) - slot.cz * 4;
        if k >= 0 && l >= 0 && (k as usize) < FLAT_GRID && (l as usize) < FLAT_GRID {
            return slot.grid[l as usize * FLAT_GRID + k as usize];
        }
        self.arg.sample(pos)
    }
    fn build_grid(arg: &Arc<DensityFunction>, chunk_x: i32, chunk_z: i32, grid: &mut [f64; 25]) {
        grid.fill(0.0);
        for i in 0..FLAT_GRID {
            let px = (chunk_x * 4 + i as i32) * 4;
            for j in 0..FLAT_GRID {
                let pz = (chunk_z * 4 + j as i32) * 4;
                grid[j * FLAT_GRID + i] = arg.sample(&NoisePos { x: px, y: 0, z: pz });
            }
        }
    }
}

#[derive(Clone)]
pub enum DensityFunction {
    Constant { value: f64 },
    Noise { noise: Arc<DoublePerlinNoiseSampler>, xz_scale: f64, y_scale: f64, mn: f64, mx: f64 },
    LinearOp { op: BinOp, input: Box<DensityFunction>, c: f64, mn: f64, mx: f64 },
    BinaryOp { op: BinOp, a: Box<DensityFunction>, b: Box<DensityFunction>, mn: f64, mx: f64 },
    UnaryOp { op: UnaryOp, input: Box<DensityFunction>, mn: f64, mx: f64 },
    Clamp { input: Box<DensityFunction>, mn: f64, mx: f64 },
    Spline(SplineData),
    Interpolated(InterpolatedData),
    Cache2D(Cache2DData),
    FlatCache(FlatCacheData),
    ShiftDF { noise: Arc<DoublePerlinNoiseSampler>, mode: ShiftMode, id: u32 },
    ShiftedNoise { shift_x: Box<DensityFunction>, shift_y: Box<DensityFunction>, shift_z: Box<DensityFunction>, xz_scale: f64, y_scale: f64, noise: Arc<DoublePerlinNoiseSampler> },
    RangeChoice { input: Box<DensityFunction>, min_inclusive: f64, max_exclusive: f64, in_range: Box<DensityFunction>, out_of_range: Box<DensityFunction> },
    YClampedGradient { from_y: i32, to_y: i32, from_value: f64, to_value: f64 },
    WeirdScaled { input: Box<DensityFunction>, noise: Arc<DoublePerlinNoiseSampler>, rarity: WeirdRarity },
    BlendAlpha,
    BlendOffset,
    BlendDensity { input: Box<DensityFunction> },
    Wrapping { input: Box<DensityFunction> },
    InterpolatedNoise(InterpolatedNoiseData),
    Lazy { target: Arc<Mutex<Option<Arc<DensityFunction>>>> },
    // multi-channel 竖切：combine 模式读已插值 channel 值（对齐 SteelMC `interpolated[idx]`）。
    // mn/mx = 对应 channel inner 的边界（块级 combine 外层操作短路判断用）。
    ReadChannel { ch: usize, mn: f64, mx: f64 },
}

impl DensityFunction {
    pub fn sample(&self, pos: &NoisePos) -> f64 {
        self.sample_ctx(pos, &[])
    }
    // combine 模式（multi-channel 竖切）：ReadChannel 读 interp[ch]（对已插值 channel 值），其他节点正常采样。
    // 外层操作树在块级对已插值 channel 应用（对齐 SteelMC `combine_interpolated` + `interpolated[idx]`）。
    pub fn sample_combine(&self, pos: &NoisePos, interp: &[f64]) -> f64 {
        self.sample_ctx(pos, interp)
    }
    #[inline]
    fn sample_ctx(&self, pos: &NoisePos, interp: &[f64]) -> f64 {
        match self {
            DensityFunction::ReadChannel { ch, .. } => interp[*ch],
            DensityFunction::Constant { value } => *value,
            DensityFunction::Noise { noise, xz_scale, y_scale, .. } => {
                noise.sample(pos.x as f64 * xz_scale, pos.y as f64 * y_scale, pos.z as f64 * xz_scale)
            }
            DensityFunction::LinearOp { op, input, c, .. } => {
                let x = input.sample_ctx(pos, interp);
                match op { BinOp::Add => x + c, BinOp::Mul => x * c, _ => x }
            }
            DensityFunction::BinaryOp { op, a, b, .. } => {
                let da = a.sample_ctx(pos, interp);
                match op {
                    BinOp::Add => da + b.sample_ctx(pos, interp),
                    BinOp::Mul => if da == 0.0 { 0.0 } else { da * b.sample_ctx(pos, interp) },
                    BinOp::Min => if da < b.min_value() { da } else { da.min(b.sample_ctx(pos, interp)) },
                    BinOp::Max => { let bmax = b.max_value(); let bv = b.sample_ctx(pos, interp); if da > bmax { da } else { da.max(bv) } }
                }
            }
            DensityFunction::UnaryOp { op, input, .. } => apply_unary(*op, input.sample_ctx(pos, interp)),
            DensityFunction::Clamp { input, mn, mx } => clamp_d(input.sample_ctx(pos, interp), *mn, *mx),
            DensityFunction::Spline(s) => s.sample_node(s.root, pos),
            DensityFunction::Interpolated(id) => id.sample(pos),
            DensityFunction::Cache2D(c) => c.sample(pos),
            DensityFunction::FlatCache(f) => f.sample(pos),
            DensityFunction::ShiftDF { noise, mode, id } => {
                // ShiftDF 实测 y 无关（offset noise 只依赖 xz）→ 按 (x, z) 缓存（Cache2D 语义）
                // key 用实际采样坐标：Shift/ShiftA 用 (x,z)；ShiftB 用 (z,x)。同 xz 不同 y 的 corners 命中。
                let (key_x, key_z, mut x, mut y, mut z) = match mode {
                    ShiftMode::ShiftB => (pos.z, pos.x, pos.z as f64, pos.x as f64, 0.0),
                    _ => (pos.x, pos.z, pos.x as f64, { 0.0 }, pos.z as f64),
                };
                let key = ((key_x as i64) << 32) ^ (key_z as u32 as i64);
                SHIFT_CACHE.with(|m| {
                    let mut m = m.borrow_mut();
                    if m.len() <= *id as usize { m.resize(*id as usize + 1, None); }
                    let slot = m[*id as usize].get_or_insert_with(|| Box::new(RefCell::new(ShiftSlot { cx: 0, cz: 0, value: 0.0 })));
                    let mut slot = slot.borrow_mut();
                    if slot.cx != key_x || slot.cz != key_z {
                        slot.cx = key_x; slot.cz = key_z;
                        // ShiftB 已在 match 设置 x/y/z；Shift/ShiftA 走 y=0（实测 y 无关）
                        slot.value = noise.sample(x * 0.25, y * 0.25, z * 0.25) * 4.0;
                    }
                    slot.value
                })
            }
            DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, xz_scale, y_scale, noise } => {
                let d = pos.x as f64 * xz_scale + shift_x.sample_ctx(pos, interp);
                let e = pos.y as f64 * y_scale + shift_y.sample_ctx(pos, interp);
                let f = pos.z as f64 * xz_scale + shift_z.sample_ctx(pos, interp);
                noise.sample(d, e, f)
            }
            DensityFunction::RangeChoice { input, min_inclusive, max_exclusive, in_range, out_of_range } => {
                let d = input.sample_ctx(pos, interp);
                if *min_inclusive <= d && d < *max_exclusive { in_range.sample_ctx(pos, interp) } else { out_of_range.sample_ctx(pos, interp) }
            }
            DensityFunction::YClampedGradient { from_y, to_y, from_value, to_value } => {
                YClampedGradient::clamped_map(pos.y as f64, *from_y, *to_y, *from_value, *to_value)
            }
            DensityFunction::WeirdScaled { input, noise, rarity } => {
                let d = WeirdScaled::scale_value(*rarity, input.sample_ctx(pos, interp));
                d * noise.sample(pos.x as f64 / d, pos.y as f64 / d, pos.z as f64 / d).abs()
            }
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::BlendDensity { input } => input.sample_ctx(pos, interp),
            DensityFunction::Wrapping { input } => input.sample_ctx(pos, interp),
            DensityFunction::InterpolatedNoise(nd) => nd.sample(pos),
            DensityFunction::Lazy { target } => {
                let t = target.lock().unwrap();
                if let Some(t) = t.as_ref() { t.sample_ctx(pos, interp) } else { 0.0 }
            }
        }
    }
    pub fn min_value(&self) -> f64 {
        match self {
            DensityFunction::Constant { value } => *value,
            DensityFunction::Noise { mn, .. } => *mn,
            DensityFunction::LinearOp { mn, .. } => *mn,
            DensityFunction::BinaryOp { mn, .. } => *mn,
            DensityFunction::UnaryOp { mn, .. } => *mn,
            DensityFunction::Clamp { mn, .. } => *mn,
            DensityFunction::Spline(s) => s.min_val,
            DensityFunction::Interpolated(id) => id.mn,
            DensityFunction::Cache2D(c) => c.mn,
            DensityFunction::FlatCache(f) => f.mn,
            DensityFunction::ShiftDF { noise, .. } => -noise.get_max_value() * 4.0,
            DensityFunction::ShiftedNoise { noise, .. } => -noise.get_max_value(),
            DensityFunction::RangeChoice { in_range, out_of_range, .. } => in_range.min_value().min(out_of_range.min_value()),
            DensityFunction::YClampedGradient { from_value, to_value, .. } => from_value.min(*to_value),
            DensityFunction::WeirdScaled { .. } => 0.0,
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::BlendDensity { input } => input.min_value(),
            DensityFunction::Wrapping { input } => input.min_value(),
            DensityFunction::InterpolatedNoise(nd) => -nd.max_val,
            DensityFunction::Lazy { target } => {
                let t = target.lock().unwrap();
                if let Some(t) = t.as_ref() { t.min_value() } else { f64::NEG_INFINITY }
            }
            DensityFunction::ReadChannel { mn, .. } => *mn,
        }
    }
    pub fn max_value(&self) -> f64 {
        match self {
            DensityFunction::Constant { value } => *value,
            DensityFunction::Noise { mx, .. } => *mx,
            DensityFunction::LinearOp { mx, .. } => *mx,
            DensityFunction::BinaryOp { mx, .. } => *mx,
            DensityFunction::UnaryOp { mx, .. } => *mx,
            DensityFunction::Clamp { mx, .. } => *mx,
            DensityFunction::Spline(s) => s.max_val,
            DensityFunction::Interpolated(id) => id.mx,
            DensityFunction::Cache2D(c) => c.mx,
            DensityFunction::FlatCache(f) => f.mx,
            DensityFunction::ShiftDF { noise, .. } => noise.get_max_value() * 4.0,
            DensityFunction::ShiftedNoise { noise, .. } => noise.get_max_value(),
            DensityFunction::RangeChoice { in_range, out_of_range, .. } => in_range.max_value().max(out_of_range.max_value()),
            DensityFunction::YClampedGradient { from_value, to_value, .. } => from_value.max(*to_value),
            DensityFunction::WeirdScaled { noise, rarity, .. } => {
                let mult = if *rarity == WeirdRarity::Caves { 3.0 } else { 2.0 };
                mult * noise.get_max_value()
            }
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::BlendDensity { input } => input.max_value(),
            DensityFunction::Wrapping { input } => input.max_value(),
            DensityFunction::InterpolatedNoise(nd) => nd.max_val,
            DensityFunction::Lazy { target } => {
                let t = target.lock().unwrap();
                if let Some(t) = t.as_ref() { t.max_value() } else { f64::INFINITY }
            }
            DensityFunction::ReadChannel { mx, .. } => *mx,
        }
    }
}

/// multi-channel 竖切（对齐 SteelMC/Java NoiseChunk）：
/// 遍历 density 树（DFS），收集所有 `Interpolated` 的内层 arg 为独立 channel，
/// 并把树中 `Interpolated` 节点替换为 `ReadChannel{ch}`（combine 模式读已插值 channel 值）。
/// 返回 `(channels, combine_root)`：channels 在 cell corners 采样一次；combine_root 块级应用外层操作。
pub fn macrolize_channels(root: &DensityFunction) -> (Vec<Arc<DensityFunction>>, DensityFunction) {
    let mut channels: Vec<Arc<DensityFunction>> = Vec::new();
    let combine = macrolize_into(root, &mut channels);
    (channels, combine)
}

// 递归转换（DFS 分配 channel；Interpolated → ReadChannel；其余 clone + 递归）
fn macrolize_into(df: &DensityFunction, channels: &mut Vec<Arc<DensityFunction>>) -> DensityFunction {
    match df {
        DensityFunction::Interpolated(id) => {
            // 分配 channel：收集 inner arg
            let ch = channels.len();
            channels.push(id.arg.clone());
            DensityFunction::ReadChannel { ch, mn: id.mn, mx: id.mx }
        }
        DensityFunction::Constant { value } => DensityFunction::Constant { value: *value },
        DensityFunction::Noise { noise, xz_scale, y_scale, mn, mx } =>
            DensityFunction::Noise { noise: noise.clone(), xz_scale: *xz_scale, y_scale: *y_scale, mn: *mn, mx: *mx },
        DensityFunction::LinearOp { op, input, c, mn, mx } =>
            DensityFunction::LinearOp { op: *op, input: Box::new(macrolize_into(input, channels)), c: *c, mn: *mn, mx: *mx },
        DensityFunction::BinaryOp { op, a, b, mn, mx } =>
            DensityFunction::BinaryOp { op: *op, a: Box::new(macrolize_into(a, channels)), b: Box::new(macrolize_into(b, channels)), mn: *mn, mx: *mx },
        DensityFunction::UnaryOp { op, input, mn, mx } =>
            DensityFunction::UnaryOp { op: *op, input: Box::new(macrolize_into(input, channels)), mn: *mn, mx: *mx },
        DensityFunction::Clamp { input, mn, mx } =>
            DensityFunction::Clamp { input: Box::new(macrolize_into(input, channels)), mn: *mn, mx: *mx },
        DensityFunction::Spline(s) => {
            // Spline 的 loc_fns 递归（loc_fns 可能含 Interpolated/其他）
            let loc_fns = s.loc_fns.iter().map(|f| Arc::new(macrolize_into(f, channels))).collect::<Vec<_>>();
            let mut ndata = s.clone();
            ndata.loc_fns = loc_fns;
            DensityFunction::Spline(ndata)
        }
        DensityFunction::Cache2D(c) =>
            DensityFunction::Cache2D(Cache2DData::new(Arc::new(macrolize_into(&c.arg, channels)))),
        DensityFunction::FlatCache(f) =>
            DensityFunction::FlatCache(FlatCacheData::new(Arc::new(macrolize_into(&f.arg, channels)))),
        DensityFunction::ShiftDF { noise, mode, id } => DensityFunction::ShiftDF { noise: noise.clone(), mode: *mode, id: *id },
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, xz_scale, y_scale, noise } =>
            DensityFunction::ShiftedNoise {
                shift_x: Box::new(macrolize_into(shift_x, channels)),
                shift_y: Box::new(macrolize_into(shift_y, channels)),
                shift_z: Box::new(macrolize_into(shift_z, channels)),
                xz_scale: *xz_scale, y_scale: *y_scale,
                noise: noise.clone(),
            },
        DensityFunction::RangeChoice { input, min_inclusive, max_exclusive, in_range, out_of_range } =>
            DensityFunction::RangeChoice {
                input: Box::new(macrolize_into(input, channels)),
                min_inclusive: *min_inclusive, max_exclusive: *max_exclusive,
                in_range: Box::new(macrolize_into(in_range, channels)),
                out_of_range: Box::new(macrolize_into(out_of_range, channels)),
            },
        DensityFunction::YClampedGradient { from_y, to_y, from_value, to_value } =>
            DensityFunction::YClampedGradient { from_y: *from_y, to_y: *to_y, from_value: *from_value, to_value: *to_value },
        DensityFunction::WeirdScaled { input, noise, rarity } =>
            DensityFunction::WeirdScaled { input: Box::new(macrolize_into(input, channels)), noise: noise.clone(), rarity: *rarity },
        DensityFunction::BlendAlpha => DensityFunction::BlendAlpha,
        DensityFunction::BlendOffset => DensityFunction::BlendOffset,
        DensityFunction::BlendDensity { input } =>
            DensityFunction::BlendDensity { input: Box::new(macrolize_into(input, channels)) },
        DensityFunction::Wrapping { input } =>
            DensityFunction::Wrapping { input: Box::new(macrolize_into(input, channels)) },
        DensityFunction::InterpolatedNoise(nd) => DensityFunction::InterpolatedNoise(nd.clone()),
        DensityFunction::Lazy { target } => {
            // Lazy 内部可能含 Interpolated；递归 target（若已解析）
            let t = target.lock().unwrap();
            match t.as_ref() {
                Some(inner) => {
                    let conv = macrolize_into(inner, channels);
                    DensityFunction::Lazy { target: Arc::new(Mutex::new(Some(Arc::new(conv)))) }
                }
                None => DensityFunction::Lazy { target: target.clone() },
            }
        }
        DensityFunction::ReadChannel { ch, mn, mx } => DensityFunction::ReadChannel { ch: *ch, mn: *mn, mx: *mx },
    }
}
