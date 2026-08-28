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

// 创建 worldgen 句柄：一次 seed 初始化（构建全部 noise samplers + density 树 + biome + surface）。
// worldgenDir: vanilla worldgen JSON 数据目录（含 data/minecraft/worldgen/...）
// 失败返回 NULL
#[unsafe(no_mangle)]
pub extern "C" fn wg_create(seed: i64, worldgen_dir: *const c_char,
                            _settings_name: *const c_char,
                            _biome_params_file: *const c_char,
                            _world_height: c_int) -> *mut c_void {
    if worldgen_dir.is_null() { return std::ptr::null_mut(); }
    let dir = unsafe { std::ffi::CStr::from_ptr(worldgen_dir) }.to_string_lossy().into_owned();
    match WorldgenHandle::create(seed, &dir) {
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
// threads: 并行线程数；0 或负 = 自适应。返回 count。
// 注：当前串行生成（Rust thread_local 缓存 + 只读树并发安全，后续可加线程池）。
#[unsafe(no_mangle)]
pub extern "C" fn wg_fill_blocks_multi(handle: *mut c_void,
                                        chunk_xs: *const c_int, chunk_zs: *const c_int,
                                        outs: *const *mut c_int, count: c_int, _threads: c_int) -> c_int {
    if handle.is_null() || count <= 0 { return 0; }
    let h = unsafe { &*(handle as *const WorldgenHandle) };
    let count = count as usize;
    let cxs = unsafe { std::slice::from_raw_parts(chunk_xs, count) };
    let czs = unsafe { std::slice::from_raw_parts(chunk_zs, count) };
    let out_ptrs = unsafe { std::slice::from_raw_parts(outs, count) };
    let block_count = (16 * 16 * HEIGHT) as usize;

    // 串行生成（先保证正确性，多线程后续加）
    for i in 0..count {
        let blocks = h.fill_chunk_blocks(cxs[i], czs[i]);
        let dst = unsafe { std::slice::from_raw_parts_mut(out_ptrs[i], block_count) };
        dst.copy_from_slice(&blocks[..block_count]);
    }
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
pub extern "C" fn wg_height(_h: *mut c_void) -> c_int { HEIGHT }
#[unsafe(no_mangle)]
pub extern "C" fn wg_density_points_per_chunk(_h: *mut c_void) -> c_int { POINTS_PER_CHUNK }

// 密度场批量求值（fillDensity 用；Rust 侧暂未实现完整 density 网格，返回 0）
#[unsafe(no_mangle)]
pub extern "C" fn wg_fill_density(_h: *mut c_void, _min_chunk_x: c_int, _min_chunk_z: c_int,
                                  _size: c_int, _out: *mut f64) -> c_int { 0 }
