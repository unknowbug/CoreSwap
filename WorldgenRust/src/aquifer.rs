// aquifer.rs — C++ 1.20.1 AquiferSampler.Impl 移植（块级含水层：决定密度<0 区的 lava/水/空洞）。
// 独立于 finalDensity 树（块级），不加重结构性复杂。Port of versions/1.20.1/cpp/worldgen/src/aquifer.h。
// 块 id：AIR=0 WATER=1 LAVA=2；apply 返回 -1 表示 null（保持默认方块/石头）。
use std::sync::Arc;
use crate::density::{DensityFunction, NoisePos};
use crate::xoroshiro::XoroshiroSplitter;
use crate::xoroshiro::XoroshiroRandom;

pub const AIR: i32 = 0;
pub const WATER: i32 = 1;
pub const LAVA: i32 = 2;

fn floor_div(a: i32, b: i32) -> i32 { let r = a / b; if (a % b) != 0 && ((a ^ b) < 0) { r - 1 } else { r } }

// WG_AQUIFERCOUNT（单线程诊断，门控关时零开销）：统计 calculate_density 里 barrier.sample 调用次数
static BARRIER_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static BARRIER_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn aquifer_barrier_watch(on: bool) {
    BARRIER_WATCH.store(on, std::sync::atomic::Ordering::Relaxed);
    if on { BARRIER_COUNT.store(0, std::sync::atomic::Ordering::Relaxed); }
}
pub fn aquifer_barrier_count_reset() -> usize {
    BARRIER_COUNT.swap(0, std::sync::atomic::Ordering::Relaxed)
}

// WG_AQUIFERWL（单线程诊断）：统计 get_water_level_at 调用次数 + miss 次数（miss 触发 get_fluid_level）
static WL_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
thread_local! {
    static WL_COUNT: std::cell::RefCell<[usize; 2]> = std::cell::RefCell::new([0, 0]); // [calls, miss]
}
pub fn aquifer_wl_watch(on: bool) {
    WL_WATCH.store(on, std::sync::atomic::Ordering::Relaxed);
    if on { WL_COUNT.with(|c| *c.borrow_mut() = [0, 0]); }
}
pub fn aquifer_wl_count_reset() -> [usize; 2] {
    WL_COUNT.with(|c| { let mut c = c.borrow_mut(); let r = *c; *c = [0, 0]; r })
}

// WG_AQUIFERBP（单线程诊断）：统计 get_block_pos 调用次数 + miss 次数（miss 触发 split_xyz+random）
static BP_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
thread_local! {
    static BP_COUNT: std::cell::RefCell<[usize; 2]> = std::cell::RefCell::new([0, 0]); // [calls, miss]
}
pub fn aquifer_bp_watch(on: bool) {
    BP_WATCH.store(on, std::sync::atomic::Ordering::Relaxed);
    if on { BP_COUNT.with(|c| *c.borrow_mut() = [0, 0]); }
}
pub fn aquifer_bp_count_reset() -> [usize; 2] {
    BP_COUNT.with(|c| { let mut c = c.borrow_mut(); let r = *c; *c = [0, 0]; r })
}

// WG_AQUIFERSURF（单线程诊断）：统计 estimate_surface_height 调用次数 + initial_density 迭代总次数
//（Q-AQ1 260903-10：验证生产冷 surface_cache 下 get_fluid_level 成本假设）
static SURF_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
thread_local! {
    static SURF_COUNT: std::cell::RefCell<[usize; 2]> = std::cell::RefCell::new([0, 0]); // [calls, iterations]
}
pub fn aquifer_surf_watch(on: bool) {
    SURF_WATCH.store(on, std::sync::atomic::Ordering::Relaxed);
    if on { SURF_COUNT.with(|c| *c.borrow_mut() = [0, 0]); }
}
pub fn aquifer_surf_count_reset() -> [usize; 2] {
    SURF_COUNT.with(|c| { let mut c = c.borrow_mut(); let r = *c; *c = [0, 0]; r })
}

#[derive(Clone, Copy)]
pub struct FluidLevel { pub y: i32, pub block: i32 }
impl FluidLevel {
    fn default_level(block_y: i32) -> FluidLevel { if block_y < -54 { FluidLevel { y: -54, block: LAVA } } else { FluidLevel { y: 63, block: WATER } } }
    fn get_block_state(&self, block_y: i32) -> i32 { if block_y >= self.y { AIR } else { self.block } }
}

struct MutableDouble { v: f64, has: bool }
impl MutableDouble { fn new() -> Self { MutableDouble { v: f64::NAN, has: false } } }

// 列缓存（estimateSurfaceHeight）
const CACHE_DIM: i32 = 32;
const CACHE_OFF_X: i32 = 12;
const CACHE_OFF_Z: i32 = 4;

// b1-b blend 旁路闸门（260903-11）：Rust 未实现 blend（density.rs:626-628 blend 类 DF 均为
// no-blending 语义常数），est 为纯函数；未来实现 blend 时置 true → L2 全量旁路（防 blend
// per-chunk density 污染跨 chunk 缓存，ChunkNoiseSampler.java:142-154 对应语义）。
pub const BLEND_ACTIVE: bool = false;

// b1-b：跨 chunk est L2 精确值缓存（Java 语义外的纯性能优化；260903-13 翻默认：默认启用，WG_EST_L2=0 关）。
// est = f(seed, noise_params, 量化列) 纯函数 → 跨 chunk 缓存逐位安全（blend 闸门见上）。
// 淘汰：FIFO 环（容量上限硬界，语义同 clock——淘汰只影响命中率不影响正确性，重算同值）。
// epoch：实例挂 WorldgenHandle（每 (seed,params) 一个 handle）→ 代际隔离天然成立。
pub struct EstL2 {
    map: std::collections::HashMap<u64, i32>,
    order: std::collections::VecDeque<u64>,
    cap: usize,
    pub hits: usize,
    pub misses: usize,
    pub inserts: usize,
    pub evictions: usize,
}

impl EstL2 {
    // 131072 条 × (8B key + 4B val + 开销) ≈ 2-4MB 硬上限（b1 设计 §2.3）
    pub const DEFAULT_CAP: usize = 131072;
    pub fn new(cap: usize) -> EstL2 {
        EstL2 { map: std::collections::HashMap::with_capacity(cap / 2), order: std::collections::VecDeque::with_capacity(cap), cap, hits: 0, misses: 0, inserts: 0, evictions: 0 }
    }
    fn key(bx: i32, bz: i32) -> u64 {
        ((bx as u64 & 0xFFFF_FFFF) << 32) | (bz as u64 & 0xFFFF_FFFF)
    }
    pub fn get(&mut self, bx: i32, bz: i32) -> Option<i32> {
        let r = self.map.get(&Self::key(bx, bz)).copied();
        if r.is_some() { self.hits += 1; } else { self.misses += 1; }
        r
    }
    pub fn put(&mut self, bx: i32, bz: i32, v: i32) {
        let k = Self::key(bx, bz);
        if self.map.contains_key(&k) { return; }
        if self.map.len() >= self.cap {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
                self.evictions += 1;
            }
        }
        self.map.insert(k, v);
        self.order.push_back(k);
        self.inserts += 1;
    }
    pub fn stats(&self) -> [usize; 4] { [self.hits, self.misses, self.inserts, self.evictions] }
}

pub struct Aquifer {
    barrier: Arc<DensityFunction>,
    fluid_floodedness: Arc<DensityFunction>,
    fluid_spread: Arc<DensityFunction>,
    fluid_type: Arc<DensityFunction>,
    erosion: Arc<DensityFunction>,
    depth: Arc<DensityFunction>,
    initial_density: Arc<DensityFunction>,
    splitter: XoroshiroSplitter,
    min_y: i32, height: i32,
    start_x: i32, start_y: i32, start_z: i32,
    size_x: i32, size_y: i32, size_z: i32,
    block_positions: Vec<i64>,
    water_levels: Vec<FluidLevel>,
    surface_cache: Vec<i32>,
    cache_cx: i32, cache_cz: i32,
    // b1-b 跨 chunk est L2（默认 None=关；WorldgenHandle 按 env 注入，Arc 跨 chunk 共享）
    est_l2: Option<Arc<std::sync::Mutex<EstL2>>>,
}

impl Aquifer {
    pub fn new(
        barrier: Arc<DensityFunction>, fluid_floodedness: Arc<DensityFunction>, fluid_spread: Arc<DensityFunction>,
        fluid_type: Arc<DensityFunction>, erosion: Arc<DensityFunction>, depth: Arc<DensityFunction>, initial_density: Arc<DensityFunction>,
        splitter: XoroshiroSplitter, chunk_start_x: i32, chunk_start_z: i32, min_y: i32, height: i32,
    ) -> Aquifer {
        let start_x_l = floor_div(chunk_start_x, 16) - 1;
        let end_x_l = floor_div(chunk_start_x + 15, 16) + 1;
        let start_y_l = floor_div(min_y, 12) - 1;
        let end_y_l = floor_div(min_y + height, 12) + 1;
        let start_z_l = floor_div(chunk_start_z, 16) - 1;
        let end_z_l = floor_div(chunk_start_z + 15, 16) + 1;
        let sx = end_x_l - start_x_l + 1;
        let sy = end_y_l - start_y_l + 1;
        let sz = end_z_l - start_z_l + 1;
        let m = (sx * sy * sz) as usize;
        Aquifer {
            barrier, fluid_floodedness, fluid_spread, fluid_type, erosion, depth, initial_density,
            splitter,
            min_y, height,
            start_x: start_x_l, start_y: start_y_l, start_z: start_z_l,
            size_x: sx, size_y: sy, size_z: sz,
            block_positions: vec![i64::MAX; m],
            water_levels: vec![FluidLevel { y: i32::MAX, block: AIR }; m],
            surface_cache: vec![i32::MIN; (CACHE_DIM * CACHE_DIM) as usize],
            cache_cx: floor_div(chunk_start_x, 16),
            cache_cz: floor_div(chunk_start_z, 16),
            est_l2: None,
        }
    }

    // b1-b：注入跨 chunk est L2（None=关）。Aquifer::new 签名保持不变（30+ 调用点零改动）。
    pub fn set_est_l2(&mut self, l2: Option<Arc<std::sync::Mutex<EstL2>>>) { self.est_l2 = l2; }
    pub fn est_l2_stats(&self) -> [usize; 4] {
        match &self.est_l2 {
            Some(l2) => l2.lock().map(|m| m.stats()).unwrap_or([0; 4]),
            None => [0; 4],
        }
    }

    fn index(&self, x: i32, y: i32, z: i32) -> usize {
        let i = x - self.start_x; let j = y - self.start_y; let k = z - self.start_z;
        ((j * self.size_z + k) * self.size_x + i) as usize
    }
    fn pack(x: i32, y: i32, z: i32) -> i64 {
        let l = (((x as u32 & 0x3FFFFFF) as u64) << 38)
              | (((y as u32 & 0xFFF) as u64) << 26)
              | ((z as u32 & 0x3FFFFFF) as u64);
        l as i64
    }
    fn unpack_x(l: i64) -> i32 { (l >> 38) as i32 }
    fn unpack_y(l: i64) -> i32 {
        let y12 = (l >> 26) & 0xFFF;
        let y12 = if y12 & 0x800 != 0 { y12 | !0xFFF } else { y12 };
        y12 as i32
    }
    fn unpack_z(l: i64) -> i32 {
        let z26 = l & 0x3FFFFFF;
        let z26 = if z26 & 0x2000000 != 0 { z26 | !0x3FFFFFF } else { z26 };
        z26 as i32
    }

    // 诊断（无热路径污染）：测 3×3 邻域 get_block_pos 循环的成本。rounds 次，每次模拟 apply 的 3×3 邻域。
    pub fn diag_blockpos_cost(&mut self, cx: i32, cz: i32, rounds: usize) -> f64 {
        let t0 = std::time::Instant::now();
        for _r in 0..rounds {
            for y in self.min_y..self.min_y + self.height {
                for z in 0..16 { for x in 0..16 {
                    let l = floor_div(cx*16 + x - 5, 16);
                    let m = floor_div(y + 1, 12);
                    let n = floor_div(cz*16 + z - 5, 16);
                    for u in 0..=1 { for v in -1..=1 { for w in 0..=1 {
                        let _ = self.get_block_pos(l + u, m + v, n + w);
                    }}}
                }}
            }
        }
        t0.elapsed().as_secs_f64()
    }

    // 诊断（无热路径污染）：测 get_fluid_level 循环成本（含 estimate_surface_height + fluid_type 采样）。
    pub fn diag_fluidlevel_cost(&mut self, cx: i32, cz: i32, rounds: usize) -> f64 {
        let t0 = std::time::Instant::now();
        for _r in 0..rounds {
            for y in self.min_y..self.min_y + self.height {
                for z in 0..16 { for x in 0..16 {
                    let _ = self.get_fluid_level(cx*16 + x, y, cz*16 + z);
                }}
            }
        }
        t0.elapsed().as_secs_f64()
    }

    // 诊断（无热路径污染）：测 get_water_level_at 成本（apply 每点 r/s/t 3 次调用）。
    pub fn diag_waterlevel_cost(&mut self, cx: i32, cz: i32, rounds: usize) -> f64 {
        let t0 = std::time::Instant::now();
        for _r in 0..rounds {
            for y in self.min_y..self.min_y + self.height {
                for z in 0..16 { for x in 0..16 {
                    // 模拟 apply 的 r/s/t 3 次 get_water_level_at（用 pack 的 cell pos）
                    let l = floor_div(cx*16 + x - 5, 16);
                    let m = floor_div(y + 1, 12);
                    let n = floor_div(cz*16 + z - 5, 16);
                    let ab = self.get_block_pos(l, m, n);
                    let _ = self.get_water_level_at(ab);
                }}
            }
        }
        t0.elapsed().as_secs_f64()
    }

    // 诊断（无热路径污染）：测 calculate_density 的 fluid 逻辑成本（模拟 1 次/点，barrier 采样已证明 ~0）。
    pub fn diag_caldensity_logic_cost(&mut self, cx: i32, cz: i32, rounds: usize) -> f64 {
        use crate::aquifer::MutableDouble;
        let t0 = std::time::Instant::now();
        for _r in 0..rounds {
            for y in self.min_y..self.min_y + self.height {
                for z in 0..16 { for x in 0..16 {
                    let mut md = MutableDouble::new();
                    let fl1 = FluidLevel { y: 63, block: WATER };
                    let _ = self.calculate_density(cx*16 + x, y, cz*16 + z, &mut md, fl1, fl1);
                }}
            }
        }
        t0.elapsed().as_secs_f64()
    }

    fn get_block_pos(&mut self, x: i32, y: i32, z: i32) -> i64 {
        if BP_WATCH.load(std::sync::atomic::Ordering::Relaxed) { BP_COUNT.with(|c| c.borrow_mut()[0] += 1); }
        let aa = self.index(x, y, z);
        let ab = self.block_positions[aa];
        if ab != i64::MAX { return ab; }
        if BP_WATCH.load(std::sync::atomic::Ordering::Relaxed) { BP_COUNT.with(|c| c.borrow_mut()[1] += 1); }
        let mut random: XoroshiroRandom = self.splitter.split_xyz(x, y, z);
        let rx = random.next_int_bound(10);
        let ry = random.next_int_bound(9);
        let rz = random.next_int_bound(10);
        let nb = Self::pack(x * 16 + rx, y * 12 + ry, z * 16 + rz);
        self.block_positions[aa] = nb;
        nb
    }

    fn max_distance(i: i32, a: i32) -> f64 { 1.0 - (a - i).abs() as f64 / 25.0 }

    pub fn apply(&mut self, block_x: i32, block_y: i32, block_z: i32, density: f64) -> i32 {
        if density > 0.0 { return -1; }
        let mut fluid_block;
        let mut fluid_y;
        if block_y < -54 { fluid_block = LAVA; fluid_y = -54; } else { fluid_block = WATER; fluid_y = 63; }
        if fluid_block == LAVA { return fluid_block; }

        let l = floor_div(block_x - 5, 16);
        let m = floor_div(block_y + 1, 12);
        let n = floor_div(block_z - 5, 16);
        let mut o = i32::MAX; let mut p = i32::MAX; let mut q = i32::MAX;
        let mut r: i64 = 0; let mut s: i64 = 0; let mut t: i64 = 0;
        for u in 0..=1 { for v in -1..=1 { for w in 0..=1 {
            let x = l + u; let y = m + v; let z = n + w;
            let ab = self.get_block_pos(x, y, z);
            let ad = Self::unpack_x(ab) - block_x;
            let ae = Self::unpack_y(ab) - block_y;
            let af = Self::unpack_z(ab) - block_z;
            let ag = ad*ad + ae*ae + af*af;
            if o >= ag { t = s; s = r; r = ab; q = p; p = o; o = ag; }
            else if p >= ag { t = s; s = ab; q = p; p = ag; }
            else if q >= ag { t = ab; q = ag; }
        }}}

        let fl2 = self.get_water_level_at(r);
        let d = Self::max_distance(o, p);
        let bs = fl2.get_block_state(block_y);
        if d <= 0.0 { return bs; }
        if bs == WATER && self.get_fluid_level(block_x, block_y - 1, block_z).get_block_state(block_y - 1) == LAVA { return bs; }

        let fl3 = self.get_water_level_at(s);
        let mut md = MutableDouble::new();
        let e = d * self.calculate_density(block_x, block_y, block_z, &mut md, fl2, fl3);
        if density + e > 0.0 { return -1; }

        let fl4 = self.get_water_level_at(t);
        let f = Self::max_distance(o, q);
        if f > 0.0 {
            let g = d * f * self.calculate_density(block_x, block_y, block_z, &mut md, fl2, fl4);
            if density + g > 0.0 { return -1; }
        }
        let g2 = Self::max_distance(p, q);
        if g2 > 0.0 {
            let h = d * g2 * self.calculate_density(block_x, block_y, block_z, &mut md, fl3, fl4);
            if density + h > 0.0 { return -1; }
        }
        bs
    }

    pub fn estimate_surface_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        let bx = (block_x >> 2) << 2; let bz = (block_z >> 2) << 2;
        let ix = (bx >> 2) - self.cache_cx * 4 + CACHE_OFF_X;
        let iz = (bz >> 2) - self.cache_cz * 4 + CACHE_OFF_Z;
        let (in_c, ci) = (ix >= 0 && ix < CACHE_DIM && iz >= 0 && iz < CACHE_DIM, (ix * CACHE_DIM + iz) as usize);
        if in_c { let cached = self.surface_cache[ci]; if cached != i32::MIN { return cached; } }
        // b1-b：per-chunk 缓存 miss → 查跨 chunk L2（命中则回填本 chunk 缓存；blend 闸门）
        if let Some(l2) = &self.est_l2 {
            if !BLEND_ACTIVE {
                if let Ok(mut m) = l2.lock() {
                    if let Some(v) = m.get(bx, bz) {
                        if in_c { self.surface_cache[ci] = v; }
                        return v;
                    }
                }
            }
        }
        let mut val = i32::MAX;
        let mut l = self.min_y + self.height;
        let surf_watch = SURF_WATCH.load(std::sync::atomic::Ordering::Relaxed);
        if surf_watch { SURF_COUNT.with(|c| c.borrow_mut()[0] += 1); }
        while l >= self.min_y {
            if surf_watch { SURF_COUNT.with(|c| c.borrow_mut()[1] += 1); }
            if self.initial_density.sample(&NoisePos { x: bx, y: l, z: bz }) > 0.390625 { val = l; break; }
            l -= 8;
        }
        if in_c { self.surface_cache[ci] = val; }
        // b1-b：计算结果写回跨 chunk L2（精确值，重算同值 → 淘汰只影响命中率）
        if let Some(l2) = &self.est_l2 {
            if !BLEND_ACTIVE {
                if let Ok(mut m) = l2.lock() { m.put(bx, bz, val); }
            }
        }
        val
    }

    fn calculate_density(&self, block_x: i32, block_y: i32, block_z: i32, md: &mut MutableDouble, fl: FluidLevel, fl2: FluidLevel) -> f64 {
        let bs = fl.get_block_state(block_y);
        let bs2 = fl2.get_block_state(block_y);
        let lava_water = (bs == LAVA && bs2 == WATER) || (bs == WATER && bs2 == LAVA);
        if !lava_water {
            let j = (fl.y - fl2.y).abs();
            if j == 0 { return 0.0; }
            let d = 0.5 * (fl.y + fl2.y) as f64;
            let e = block_y as f64 + 0.5 - d;
            let f = j as f64 / 2.0;
            let o = f - e.abs();
            let qq = if e > 0.0 { let pp = 0.0 + o; if pp > 0.0 { pp / 1.5 } else { pp / 2.5 } }
                     else { let pp = 3.0 + o; if pp > 0.0 { pp / 3.0 } else { pp / 10.0 } };
            let rr = if !(qq < -2.0) && !(qq > 2.0) {
                if !md.has {
                    if BARRIER_WATCH.load(std::sync::atomic::Ordering::Relaxed) { BARRIER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                    let pos = NoisePos { x: block_x, y: block_y, z: block_z }; let tv = self.barrier.sample(&pos); md.v = tv; md.has = true; tv
                } else { md.v }
            } else { 0.0 };
            return 2.0 * (rr + qq);
        }
        2.0
    }

    fn get_water_level_at(&mut self, pos: i64) -> FluidLevel {
        if WL_WATCH.load(std::sync::atomic::Ordering::Relaxed) { WL_COUNT.with(|c| c.borrow_mut()[0] += 1); }
        let i = Self::unpack_x(pos); let j = Self::unpack_y(pos); let k = Self::unpack_z(pos);
        let bx = floor_div(i, 16); let by = floor_div(j, 12); let bz = floor_div(k, 16);
        let o = self.index(bx, by, bz);
        let fl = self.water_levels[o];
        if fl.y != i32::MAX { return fl; }
        if WL_WATCH.load(std::sync::atomic::Ordering::Relaxed) { WL_COUNT.with(|c| c.borrow_mut()[1] += 1); }
        let nf = self.get_fluid_level(i, j, k);
        self.water_levels[o] = nf;
        nf
    }

    fn get_fluid_level(&mut self, block_x: i32, block_y: i32, block_z: i32) -> FluidLevel {
        const OFFSETS: [[i32; 2]; 13] = [[0,0],[-2,-1],[-1,-1],[0,-1],[1,-1],[-3,0],[-2,0],[-1,0],[1,0],[-2,1],[-1,1],[0,1],[1,1]];
        let default_fl = FluidLevel::default_level(block_y);
        let mut i = i32::MAX;
        let j = block_y + 12;
        let k = block_y - 12;
        let mut bl = false;
        for off in &OFFSETS {
            let l = block_x + off[0] * 16;
            let mm = block_z + off[1] * 16;
            let n = self.estimate_surface_height(l, mm);
            let o = n + 8;
            let bl2 = off[0] == 0 && off[1] == 0;
            if bl2 && k > o { return default_fl; }
            let bl3 = j > o;
            if bl3 || bl2 {
                let fl2 = FluidLevel::default_level(o);
                if fl2.get_block_state(o) != AIR {
                    if bl2 { bl = true; }
                    if bl3 { return fl2; }
                }
            }
            i = i.min(n);
        }
        let p = self.get_fluid_block_y(block_x, block_y, block_z, &default_fl, i, bl);
        FluidLevel { y: p, block: self.get_fluid_block_state(block_x, block_y, block_z, &default_fl, p) }
    }

    fn get_fluid_block_y(&self, block_x: i32, block_y: i32, block_z: i32, default_fl: &FluidLevel, surface_height_estimate: i32, bl: bool) -> i32 {
        let pos = NoisePos { x: block_x, y: block_y, z: block_z };
        let (mut d, mut e): (f64, f64);
        if self.erosion.sample(&pos) < -0.225f32 as f64 && self.depth.sample(&pos) > 0.9f32 as f64 {
            d = -1.0; e = -1.0;
        } else {
            let ii = surface_height_estimate + 8 - block_y;
            let f = if bl { lerp_clamp2(ii as f64, 0.0, 64.0, 1.0, 0.0) } else { 0.0 };
            let g = clamp(self.fluid_floodedness.sample(&pos), -1.0, 1.0);
            let h = map2(f, 1.0, 0.0, -0.3, 0.8);
            let kk = map2(f, 1.0, 0.0, -0.8, 0.4);
            d = g - kk; e = g - h;
        }
        if e > 0.0 { default_fl.y }
        else if d > 0.0 { self.get_noise_based_fluid_level(block_x, block_y, block_z, surface_height_estimate) }
        else { -32512 }
    }

    fn get_noise_based_fluid_level(&self, block_x: i32, block_y: i32, block_z: i32, surface_height_estimate: i32) -> i32 {
        let k = floor_div(block_x, 16);
        let l = floor_div(block_y, 40);
        let m = floor_div(block_z, 16);
        let n = l * 40 + 20;
        let pos = NoisePos { x: k, y: l, z: m };
        let d = self.fluid_spread.sample(&pos) * 10.0;
        let p = round_down_to_multiple(d, 3);
        let q = n + p;
        surface_height_estimate.min(q)
    }

    fn get_fluid_block_state(&self, block_x: i32, block_y: i32, block_z: i32, default_fl: &FluidLevel, fluid_level: i32) -> i32 {
        let mut state = default_fl.block;
        if fluid_level <= -10 && fluid_level != -32512 && state != LAVA {
            let k = floor_div(block_x, 64);
            let l = floor_div(block_y, 40);
            let m = floor_div(block_z, 64);
            let pos = NoisePos { x: k, y: l, z: m };
            let d = self.fluid_type.sample(&pos);
            if d.abs() > 0.3 { state = LAVA; }
        }
        state
    }
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 { if v < lo { lo } else if v > hi { hi } else { v } }
fn lerp_clamp2(value: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    let t = ((value - a) / (b - a)).clamp(0.0, 1.0);
    c + t * (d - c)
}
fn map2(value: f64, fs: f64, fe: f64, ts: f64, te: f64) -> f64 { lerp_clamp2(value, fs, fe, ts, te) }
fn round_down_to_multiple(d: f64, mult: i32) -> i32 { (d / mult as f64).floor() as i32 * mult }
