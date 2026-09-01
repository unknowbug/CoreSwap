// jni_bridge.rs — Rust JNI 桥（对齐 C++ jni_bridge.cpp，供 Java wg.CppWorldgen 调用）。
// 用 jni crate 实现 Java_wg_CppWorldgen_* 6 个 native 方法，内部调用 Rust 的 wg_* C ABI（api.rs）。
// jni 0.22 惯例：native 方法第一参数 = EnvUnowned（FFI safe），用 with_env() 升级到 Env 访问完整 API。
// 与 C++ jni_bridge.cpp 语义逐一对齐（含 fillBlocks 本地 buffer 再拷回的安全模式）。
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use jni::errors::Error;
use jni::objects::{JClass, JIntArray, JObjectArray, JString};
use jni::sys::{jint, jlong};
use jni::{Env, EnvUnowned};

use crate::api::{
    wg_clear_beardifier, wg_create, wg_density_points_per_chunk, wg_density_xz_interval,
    wg_density_y_interval, wg_destroy, wg_fill_blocks_multi, wg_fill_density, wg_get_flags,
    wg_height, wg_min_y, wg_set_beardifier, wg_set_flags,
};

const BLOCK_COUNT: usize = 16 * 16 * 384;

// 创建 worldgen 句柄（seed + worldgen JSON 数据目录）。返回 handle（jlong）。
// 对齐 C++ Java_wg_CppWorldgen_init：wg_create 5 参，settings_name/biome_params_file 传 null（overworld 默认），world_height=384。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_init<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, seed: jlong, worldgen_dir: JString,
) -> jlong {
    unowned_env
        .with_env(|env| -> Result<jlong, Error> {
            let dir = env.get_string(&worldgen_dir)?.to_string();
            let c_dir = std::ffi::CString::new(dir).map_err(|_| Error::JavaException)?;
            let h = wg_create(seed, c_dir.as_ptr(), ptr::null(), ptr::null(), 384);
            Ok(h as jlong)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 多世界：按维度创建句柄（seed + 目录 + settings 名 + biome 参数文件 + 世界高度）。
// Java 侧：initDim(seed, dir, "nether.json", "biome_params_nether.json", 256)。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_initDim<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, seed: jlong, worldgen_dir: JString,
    settings_name: JString, biome_params_file: JString, world_height: jint,
) -> jlong {
    unowned_env
        .with_env(|env| -> Result<jlong, Error> {
            let dir = env.get_string(&worldgen_dir)?.to_string();
            let settings = env.get_string(&settings_name)?.to_string();
            let biome_params = env.get_string(&biome_params_file)?.to_string();
            let c_dir = std::ffi::CString::new(dir).map_err(|_| Error::JavaException)?;
            let c_settings = std::ffi::CString::new(settings).map_err(|_| Error::JavaException)?;
            let c_biome = std::ffi::CString::new(biome_params).map_err(|_| Error::JavaException)?;
            let h = wg_create(seed, c_dir.as_ptr(), c_settings.as_ptr(), c_biome.as_ptr(), world_height);
            Ok(h as jlong)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 释放句柄
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_destroy<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
) {
    let _ = unowned_env.with_env(|_env: &mut Env| -> Result<(), Error> {
        if handle != 0 {
            wg_destroy(handle as *mut c_void);
        }
        Ok(())
    });
}

// 句柄级阶段开关（双跑修复 2026-09-08）：Java CppBridge 在 init/initNether 后设 flag 关 Rust carver/features。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_setFlags<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong, mask: jint,
) {
    let _ = unowned_env.with_env(|_env: &mut Env| -> Result<(), Error> {
        if handle != 0 {
            wg_set_flags(handle as *mut c_void, mask as c_int);
        }
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_getFlags<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
) -> jint {
    unowned_env
        .with_env(|_env: &mut Env| -> Result<jint, Error> {
            if handle == 0 { return Ok(0); }
            Ok(wg_get_flags(handle as *mut c_void))
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 密度场批量求值（size×size chunks）：out = double[]，大小 = size*size*pointsPerChunk。返回 points。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_fillDensity<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
    min_chunk_x: jint, min_chunk_z: jint, size: jint, out: jni::objects::JDoubleArray,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint, Error> {
            if handle == 0 { return Ok(0); }
            let len = env.get_array_length(&out)? as usize;
            let needed = (size as usize) * (size as usize) * (wg_density_points_per_chunk(handle as *mut c_void) as usize);
            if len < needed { return Ok(0); }
            let mut buf = vec![0.0f64; needed];
            env.get_double_array_region(&out, 0, &mut buf)?;
            let points = wg_fill_density(handle as *mut c_void, min_chunk_x, min_chunk_z, size, buf.as_mut_ptr());
            env.set_double_array_region(&out, 0, &buf)?;
            Ok(points)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 密度网格参数 {xzInterval, yInterval, minY, height}。返回 4。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_densityParams<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong, out4: JIntArray,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint, Error> {
            let h = handle as *mut c_void;
            let vals = [
                wg_density_xz_interval(h),
                wg_density_y_interval(h),
                wg_min_y(h),
                wg_height(h),
            ];
            env.set_int_array_region(&out4, 0, &vals)?;
            Ok(4)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 完整区块生成（方块层）：chunkXs/chunkZs = count 个 chunk 坐标；outs[i] = int[16*16*384]（vanilla raw block id）。
// threads <= 0 自适应。返回 count。
// 安全模式（对齐 C++）：本地 buffer 写，主线程再拷回 Java 数组（避免跨线程写 Java 数组 pin 问题）。
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_fillBlocks<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
    chunk_xs: JIntArray, chunk_zs: JIntArray, outs: JObjectArray<JIntArray>, threads: jint,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint, Error> {
            if handle == 0 { return Ok(0); }
            let count = env.get_array_length(&chunk_xs)? as usize;
            if count == 0 { return Ok(0); }
            // 防御校验（对齐 C++ jni_bridge.cpp L84）：chunkZs/outs 长度必须与 chunkXs 一致
            if env.get_array_length(&chunk_zs)? as usize != count
                || env.get_array_length(&outs)? as usize != count
            {
                return Ok(0);
            }
            let mut cxs = vec![0i32; count];
            let mut czs = vec![0i32; count];
            env.get_int_array_region(&chunk_xs, 0, &mut cxs)?;
            env.get_int_array_region(&chunk_zs, 0, &mut czs)?;

            // 本地 buffer：按 Java 侧 out 数组的实际长度分配（overworld 98304 / nether 65536）。
            // ⚠️ 历史根因（M13 后续）：硬编码 BLOCK_COUNT(384 高=98304)——nether 时 dll 写前 65536 正确，
            // 但拷回 set_int_array_region 把 98304 长数据拷进 65536 的 Java 数组 → 越界被 jni crate 拒绝、
            // 错误被 let _ 吞掉 → Java 数组保持全 0 → 实机下界「一片虚空」（overworld 长度恰好相等从未暴露）。
            let out0 = env.get_object_array_element(&outs, 0)?;
            let out_len = env.get_array_length(&out0)? as usize;
            let mut local: Vec<Vec<i32>> = (0..count).map(|_| vec![0i32; out_len]).collect();
            let mut bufs: Vec<*mut c_int> = local.iter_mut().map(|v| v.as_mut_ptr()).collect();

            let r = wg_fill_blocks_multi(
                handle as *mut c_void,
                cxs.as_ptr(),
                czs.as_ptr(),
                bufs.as_ptr(),
                count as c_int,
                threads,
            );

            // 主线程拷回 Java 数组（长度匹配；错误不吞——对齐「不吞异常」铁律）
            let r = r as usize;
            for i in 0..r.min(count) {
                if let Ok(arr) = env.get_object_array_element(&outs, i) {
                    env.set_int_array_region(&arr, 0, &local[i])?;
                }
            }
            Ok(r as jint)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

// 设置指定 chunk 的 Beardifier（StructureWeightSampler）输入。
// pieces 每 8 int：{minX,minY,minZ,maxX,maxY,maxZ,terrain(0-3),groundLevelDelta}；junctions 每 3 int：{sourceX,sourceGroundY,sourceZ}
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_setBeardifier<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
    chunk_x: jint, chunk_z: jint,
    pieces: JIntArray, piece_count: jint,
    junctions: JIntArray, junction_count: jint,
) {
    let _ = unowned_env.with_env(|env| -> Result<(), Error> {
        if handle == 0 { return Ok(()); }
        let mut p: Vec<i32> = Vec::new();
        if piece_count > 0 {
            let n = (piece_count as usize) * 8;
            p.resize(n, 0);
            let _ = env.get_int_array_region(&pieces, 0, &mut p);
        }
        let mut j: Vec<i32> = Vec::new();
        if junction_count > 0 {
            let n = (junction_count as usize) * 3;
            j.resize(n, 0);
            let _ = env.get_int_array_region(&junctions, 0, &mut j);
        }
        wg_set_beardifier(
            handle as *mut c_void,
            chunk_x,
            chunk_z,
            if p.is_empty() { ptr::null() } else { p.as_ptr() },
            piece_count,
            if j.is_empty() { ptr::null() } else { j.as_ptr() },
            junction_count,
        );
        Ok(())
    });
}

// 清空全部 chunk 的 Beardifier 输入（Java 侧可能未声明，保留对齐 C++ 能力）
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_clearBeardifier<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
) {
    let _ = unowned_env.with_env(|_env: &mut Env| -> Result<(), Error> {
        if handle != 0 {
            wg_clear_beardifier(handle as *mut c_void);
        }
        Ok(())
    });
}
