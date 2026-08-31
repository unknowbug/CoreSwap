// api.rs — Rust worldgen C ABI 导出（对齐 C++ worldgen_api.h）
// 用 #[unsafe(no_mangle)] extern "C" 导出 wg_* 函数，供 JNI 层 / 任何 C 桥接调用。
// 核心：wg_create（构建 WorldgenHandle）→ wg_fill_blocks_multi（块级管线）→ wg_destroy。

use std::os::raw::{c_char, c_int, c_void};

use crate::worldgen_handle::WorldgenHandle;

const XZ_INTERVAL: i32 = 4;
const Y_INTERVAL: i32 = 8;
const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;
const POINTS_PER_CHUNK: i32 = (16 / XZ_INTERVAL) * (HEIGHT / Y_INTERVAL) * (16 / XZ_INTERVAL);

pub fn density_xz_interval() -> i32 { XZ_INTERVAL }
pub fn density_y_interval() -> i32 { Y_INTERVAL }
pub fn density_height() -> i32 { HEIGHT }
pub fn density_min_y() -> i32 { MIN_Y }

// 自适应线程数：实测确定（bench_threads，144 chunks）——
//   物理核-2（本机 12-2=10）最优 > 全物理核(12) > 逻辑核(22/24, SMT 超线程不利)
// 物理核 ≈ available_parallelism()(逻辑核) / 2（SMT=2 假设，本机 24/2=12）
// 留 2 核给 OS/其他任务（避免调度中断/缓存/内存带宽饱和）
fn adaptive_threads(param_threads: i32, count: usize) -> usize {
    // 显式 env 覆盖（CORESWAP_THREADS，对齐 C++）
    if let Ok(env) = std::env::var("CORESWAP_THREADS") {
        if let Ok(t) = env.parse::<i32>() { if t > 0 { return (t as usize).min(count).max(1); } }
    }
    let threads = if param_threads > 0 {
        param_threads as usize // 显式指定
    } else {
        // 自适应：物理核-2
        let logical = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let physical = (logical / 2).max(1); // SMT=2 假设 -> 物理核（本机 24/2=12）
        let t = physical.saturating_sub(2).max(1); // 留 2 核
        t
    };
    threads.min(count).max(1) // clamp 到任务数
}

// 输出指针 Send 包装：闭包调用 write() 方法（而非访问 .0 字段）写自己线程的 out，
// 避免编译器穿透字段访问导致裸指针跨线程 Send 报错（错误台账 M2）。
// 每个 chunk 一个 out 指针，各线程对不同 chunk 写（索引区分），无数据竞争。
struct SendOut(*mut c_int);
unsafe impl Send for SendOut {}
unsafe impl Sync for SendOut {}
impl SendOut {
    #[inline]
    fn write(&self, data: &[c_int]) {
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), self.0, data.len()); }
    }
}

// 创建 worldgen 句柄：一次 seed 初始化（构建全部 noise samplers + density 树 + biome + surface）。
// worldgenDir: vanilla worldgen JSON 数据目录（含 data/minecraft/worldgen/...）
// 失败返回 NULL
#[unsafe(no_mangle)]
pub extern "C" fn wg_create(seed: i64, worldgen_dir: *const c_char,
                            settings_name: *const c_char,
                            biome_params_file: *const c_char,
                            world_height: c_int) -> *mut c_void {
    if worldgen_dir.is_null() { return std::ptr::null_mut(); }
    let dir = unsafe { std::ffi::CStr::from_ptr(worldgen_dir) }.to_string_lossy().into_owned();
    // 维度参数（multi-world）：nil → overworld 默认；否则用 create_for_dim 加载任意维度
    let sn = if settings_name.is_null() { "overworld.json" } else {
        unsafe { std::ffi::CStr::from_ptr(settings_name) }.to_string_lossy().into_owned().leak()
    };
    let bp = if biome_params_file.is_null() { "biome_params.json" } else {
        unsafe { std::ffi::CStr::from_ptr(biome_params_file) }.to_string_lossy().into_owned().leak()
    };
    match WorldgenHandle::create_for_dim(seed, &dir, sn, bp, world_height) {
        Some(h) => Box::into_raw(Box::new(h)) as *mut c_void,
        None => std::ptr::null_mut(),
    }
}

// 释放句柄
#[unsafe(no_mangle)]
pub extern "C" fn wg_destroy(handle: *mut c_void) {
    if handle.is_null() { return; }
    unsafe { drop(Box::from_raw(handle as *mut WorldgenHandle)); }
}

// 完整区块生成（方块层）：count 个 chunk，outs[i] = int[16*16*384]（vanilla raw block id）
// threads: 并行线程数；0 或负 = 自适应（物理核-2，实测最优）。返回 count。
// 自适应多线程：按核数分块（threads 个 scoped 线程交错处理 count chunks），线程数受控，
// 不是「每 chunk 一个线程」。确定性由「每 chunk 算固定 chunk + 写固定 out」保证。
#[unsafe(no_mangle)]
pub extern "C" fn wg_fill_blocks_multi(handle: *mut c_void,
                                        chunk_xs: *const c_int, chunk_zs: *const c_int,
                                        outs: *const *mut c_int, count: c_int, threads: c_int) -> c_int {
    if handle.is_null() || count <= 0 { return 0; }
    let h = unsafe { &*(handle as *const WorldgenHandle) };
    let count = count as usize;
    let cxs = unsafe { std::slice::from_raw_parts(chunk_xs, count) };
    let czs = unsafe { std::slice::from_raw_parts(chunk_zs, count) };
    let out_ptrs = unsafe { std::slice::from_raw_parts(outs, count) };
    let nthreads = adaptive_threads(threads, count);

    let h_arc = std::sync::Arc::new(h);
    // 把 out 指针包成 SendOut（每个 chunk 一个），Arc 共享，闭包按索引写各自的 out（Write 方法，非 .0 字段）
    let outs_arc = std::sync::Arc::new((0..count).map(|i| SendOut(out_ptrs[i])).collect::<Vec<_>>());
    // 分块：nthreads 个 scoped 线程，每个交错处理 {t, t+n, t+2n, ...} chunks
    std::thread::scope(|s| {
        for t in 0..nthreads {
            let h = h_arc.clone();
            let cxs = cxs;
            let czs = czs;
            let outs = outs_arc.clone();
            s.spawn(move || {
                let mut i = t;
                let dbg = std::env::var("WG_DEBUG").is_ok();
                while i < count {
                    let blocks = h.fill_chunk_blocks(cxs[i], czs[i]);
                    if dbg {
                        let nz = blocks.iter().filter(|&&v| v != 0).count();
                        // 采样 y=32/64/96 三个点的 density 组成参考（首列 x=0,z=0）
                        eprintln!("[WG-DBG] fill chunk({},{}) nonzero={}/{}", cxs[i], czs[i], nz, blocks.len());
                    }
                    // ⚠️ 按 handle 实际高度填充（nether 256×256=65536），切片长度不能假设 overworld 384
                    // （M13 后续：下界实机崩溃 = &blocks[..98304] 越界 65536 长的调用方 buffer → abort）
                    outs[i].write(&blocks);
                    i += nthreads;
                }
            });
        }
    });
    count as c_int
}

// 设置指定 chunk 的 Beardifier（StructureWeightSampler）输入。
// pieces 每 8 int：{minX,minY,minZ,maxX,maxY,maxZ,terrain(0-3),groundLevelDelta}
// junctions 每 3 int：{sourceX,sourceGroundY,sourceZ}
#[unsafe(no_mangle)]
pub extern "C" fn wg_set_beardifier(handle: *mut c_void, chunk_x: c_int, chunk_z: c_int,
                                    pieces: *const c_int, piece_count: c_int,
                                    junctions: *const c_int, junction_count: c_int) {
    if handle.is_null() { return; }
    let h = unsafe { &*(handle as *const WorldgenHandle) };
    let pieces_slice = if pieces.is_null() || piece_count <= 0 { &[] } else {
        unsafe { std::slice::from_raw_parts(pieces, (piece_count * 8) as usize) }
    };
    let junctions_slice = if junctions.is_null() || junction_count <= 0 { &[] } else {
        unsafe { std::slice::from_raw_parts(junctions, (junction_count * 3) as usize) }
    };
    h.set_beardifier(chunk_x, chunk_z, pieces_slice, junctions_slice);
}

// 清空全部 chunk 的 Beardifier 输入
#[unsafe(no_mangle)]
pub extern "C" fn wg_clear_beardifier(handle: *mut c_void) {
    if handle.is_null() { return; }
    let h = unsafe { &*(handle as *const WorldgenHandle) };
    h.clear_beardifier();
}

// 密度网格参数
#[unsafe(no_mangle)]
pub extern "C" fn wg_density_xz_interval(_h: *mut c_void) -> c_int { XZ_INTERVAL }
#[unsafe(no_mangle)]
pub extern "C" fn wg_density_y_interval(_h: *mut c_void) -> c_int { Y_INTERVAL }
#[unsafe(no_mangle)]
pub extern "C" fn wg_min_y(_h: *mut c_void) -> c_int { MIN_Y }
#[unsafe(no_mangle)]
pub extern "C" fn wg_height(h: *mut c_void) -> c_int {
    if h.is_null() { return HEIGHT; }
    unsafe { (*(h as *const WorldgenHandle)).height }
}
#[unsafe(no_mangle)]
pub extern "C" fn wg_density_points_per_chunk(_h: *mut c_void) -> c_int { POINTS_PER_CHUNK }

// 密度场批量求值（fillDensity 用）：size×size chunks 的 finalDensity 网格采样
#[unsafe(no_mangle)]
pub extern "C" fn wg_fill_density(handle: *mut c_void, min_chunk_x: c_int, min_chunk_z: c_int,
                                  size: c_int, out: *mut f64) -> c_int {
    if handle.is_null() || out.is_null() || size <= 0 { return 0; }
    let h = unsafe { &*(handle as *const WorldgenHandle) };
    let points = h.fill_density(min_chunk_x, min_chunk_z, size);
    let n = points.len();
    let dst = unsafe { std::slice::from_raw_parts_mut(out, n) };
    dst.copy_from_slice(&points);
    POINTS_PER_CHUNK
}


