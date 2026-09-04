// gpu_corner_probe.rs — lossless-accel 路线② W3/W4：Rust FFI → C++ GpuDensityEngine 三事实实测 + 三路对拍（260903-04）
// 架构：架构计划-260903-04-route2-ffi.md
// 实测项：① create 耗时 + handle 级缓存；② fillMtx 跨 FFI 并发行为；③ float32 输出口径
// 对拍：GPU 角点 vs DFC-CPU oracle（{:.6} 舍入 + f32 精确位）——覆盖 algorithm-fingerprints #13 域：
//   多 chunk（含远端 + 原点附近）/ 全 48 y 层（含常数分支层）/ 全 cell（corner 网格天然覆盖 cy≥1、cz≥2）
// 运行态：bin-diag 隔离区（cargo 不自动编译），编译命令见同目录 README 或构建输出。
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use WorldgenRust::density::NoisePos;
use WorldgenRust::terrain::{DensitySource, DfcDensity};

const SEED: i64 = -8248318472910187742;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const SPV: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\worldgen\\gpu-assets\\final_density.spv";
const DLL: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\build-msvc\\bin\\gpu_ffi.dll";
const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;

// ---- Win32 动态加载（零新依赖） ----
type HMODULE = *mut c_void;
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> HMODULE;
    fn GetProcAddress(h: HMODULE, name: *const u8) -> *mut c_void;
}
fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }
unsafe fn get_sym<T>(h: HMODULE, name: &str) -> T {
    let p = GetProcAddress(h, format!("{}\0", name).as_ptr());
    if p.is_null() { panic!("symbol not found: {}", name); }
    std::mem::transmute_copy::<*mut c_void, T>(&p)
}

type CreateFn = unsafe extern "system" fn(u64, *const u8) -> *mut c_void;
type FillFn = unsafe extern "system" fn(*mut c_void, *const i32, i32, *mut f32);
type DestroyFn = unsafe extern "system" fn(*mut c_void);
type ErrFn = unsafe extern "system" fn() -> *const u8;
type IntFn = unsafe extern "system" fn(*mut c_void) -> i32;

struct Ffi {
    create: CreateFn, fill: FillFn, destroy: DestroyFn, last_err: ErrFn,
    split_total: IntFn, per_sample: IntFn,
    _mod: HMODULE,
}
unsafe impl Send for Ffi {}
unsafe impl Sync for Ffi {}
impl Ffi {
    unsafe fn load() -> Ffi {
        let name = wide(DLL);
        let m = LoadLibraryW(name.as_ptr());
        if m.is_null() { panic!("LoadLibraryW failed: {}", DLL); }
        Ffi {
            create: get_sym(m, "gpu_ffi_create"),
            fill: get_sym(m, "gpu_ffi_fill"),
            destroy: get_sym(m, "gpu_ffi_destroy"),
            last_err: get_sym(m, "gpu_ffi_last_error"),
            split_total: get_sym(m, "gpu_ffi_split_total"),
            per_sample: get_sym(m, "gpu_ffi_per_sample"),
            _mod: m,
        }
    }
    fn err(&self) -> String {
        unsafe {
            let p = (self.last_err)();
            if p.is_null() { String::new() } else {
                std::ffi::CStr::from_ptr(p as *const i8).to_string_lossy().into_owned()
            }
        }
    }
}

fn corner_coords(chunks: &[(i32, i32)]) -> Vec<i32> {
    // 与 C++ wg_fill_density GPU 路径同口径：SX=SZ=4 间隔 4，SY=48 间隔 8
    let mut v = Vec::new();
    for &(cx, cz) in chunks {
        for y in 0..HEIGHT / 8 {
            for z in 0..4i32 {
                for x in 0..4i32 {
                    v.push(cx * 16 + x * 4);
                    v.push(MIN_Y + y * 8);
                    v.push(cz * 16 + z * 4);
                }
            }
        }
    }
    v
}

#[derive(Clone, Copy)] struct H(*mut c_void);
unsafe impl Send for H {}
unsafe impl Sync for H {}

fn main() {
    println!("=== gpu_corner_probe (260903-04) ===");
    println!("seed={} spv={}", SEED, SPV);
    let ffi = unsafe { Ffi::load() };
    let seed_u = SEED as u64;
    let spv = format!("{}\0", SPV);

    // ---- 事实①：create 耗时 + handle 复用 ----
    let t0 = std::time::Instant::now();
    let h = unsafe { (ffi.create)(seed_u, spv.as_ptr()) };
    let create_ms = t0.elapsed().as_secs_f64() * 1e3;
    if h.is_null() { panic!("gpu_ffi_create failed: {}", ffi.err()); }
    println!("[fact1] create #1 = {:.1} ms  (splitTotal={} perSample={})",
        create_ms, unsafe { (ffi.split_total)(h) }, unsafe { (ffi.per_sample)(h) });
    let t0 = std::time::Instant::now();
    let h2 = unsafe { (ffi.create)(seed_u, spv.as_ptr()) };
    println!("[fact1] create #2 = {:.1} ms（同 seed 第二实例，判断缓存/重复成本）", t0.elapsed().as_secs_f64() * 1e3);

    let handle: Arc<Mutex<H>> = Arc::new(Mutex::new(H(h))); // 引擎非线程安全：Rust 侧串行化（与 C++ 生产 h->gpu 同语义）

    // ---- 域覆盖（#13 清单）：4 远端 chunk + 4 原点附近 chunk ----
    let chunks = vec![(-288, -256), (-287, -256), (-286, -255), (-288, -255), (0, 0), (1, 0), (0, 1), (-3, 2)];
    let coords = corner_coords(&chunks);
    let n = (coords.len() / 3) as i32;
    let mut out = vec![0f32; coords.len() / 3];

    // ---- 对拍：GPU 角点 vs DFC-CPU oracle ----
    let dfc = DfcDensity::new(seed_u);
    {
        let _g = handle.lock().unwrap();
        unsafe { (ffi.fill)(h, coords.as_ptr(), n, out.as_mut_ptr()); }
        let e = ffi.err();
        if !e.is_empty() { panic!("fill failed: {}", e); }
    }
    let mut exact = 0usize; let mut r6 = 0usize; let mut max_diff = 0.0f64;
    let mut n_pts = 0usize;
    for (i, &ci) in chunks.iter().enumerate() {
        let base = i * (HEIGHT / 8) as usize * 16;
        for ly in 0..(HEIGHT / 8) as usize {
            for lz in 0..4usize { for lx in 0..4usize {
                let idx = base + ly * 16 + lz * 4 + lx;
                let pos = NoisePos { x: ci.0 * 16 + lx as i32 * 4, y: MIN_Y + ly as i32 * 8, z: ci.1 * 16 + lz as i32 * 4 };
                let a = dfc.sample(&pos) as f32;
                let b = out[idx];
                let d = (a - b).abs() as f64;
                if d > max_diff { max_diff = d; }
                if a.to_bits() == b.to_bits() { exact += 1; }
                if format!("{:.6}", a) == format!("{:.6}", b) { r6 += 1; }
                n_pts += 1;
    } } } }
    println!("[compare] gpu_vs_dfc_oracle: n={} f32_exact={} ({:.4}%) rounded6={} ({:.4}%) max_diff={:.3e}",
        n_pts, exact, exact as f64 / n_pts as f64 * 100.0, r6, r6 as f64 / n_pts as f64 * 100.0, max_diff);

    // ---- 事实②：fill 吞吐（单线程）+ 双线程 Mutex 争用 ----
    let fill_n = 8i32; // 8 次批量（8 chunk×768=6144 点/次）
    let warm = coords.clone(); let mut warm_out = out.clone();
    for _ in 0..2 { let _g = handle.lock().unwrap(); unsafe { (ffi.fill)(h, warm.as_ptr(), fill_n, warm_out.as_mut_ptr()); } }
    let reps = 10;
    let t0 = std::time::Instant::now();
    for _ in 0..reps { let _g = handle.lock().unwrap(); unsafe { (ffi.fill)(h, coords.as_ptr(), fill_n, warm_out.as_mut_ptr()); } }
    let serial_ms = t0.elapsed().as_secs_f64() * 1e3 / reps as f64;
    let pts_per_chunk = 768.0;
    let serial_per_batch = serial_ms; // fill_n=8 → 8 chunk 的批量
    println!("[fact2] serial: {:.2} ms / batch(8 chunks) = {:.3} ms/chunk-corner = {:.1} us/pt",
        serial_ms, serial_ms / 8.0, serial_ms * 1000.0 / (fill_n as f64 * pts_per_chunk));
    // 双线程：同 handle Mutex 串行化（预期 ≈ 串行 ×2，量化调度/锁开销）
    let h_ = Arc::clone(&handle);
    let hh = H(h);
    let coords2 = coords.clone(); let mut out2 = out.clone();
    let t0 = std::time::Instant::now();
    std::thread::scope(|s| {
        let hA = &handle; let cA = &coords; let mut oA = warm_out.clone();
        s.spawn(move || { let hp = hh; for _ in 0..reps/2 { let _g = hA.lock().unwrap(); unsafe { (ffi.fill)(hp.0, cA.as_ptr(), fill_n, oA.as_mut_ptr()); } } });
        s.spawn(move || { let hp = hh; for _ in 0..reps/2 { let _g = h_.lock().unwrap(); unsafe { (ffi.fill)(hp.0, coords2.as_ptr(), fill_n, out2.as_mut_ptr()); } } });
    });
    let par_ms = t0.elapsed().as_secs_f64() * 1e3 / reps as f64;
    println!("[fact2] 2-thread (mutex, same handle): {:.2} ms / batch-equivalent → 衰减系数 {:.2}×",
        par_ms, par_ms / serial_per_batch);

    unsafe { (ffi.destroy)(h2); (ffi.destroy)(h); }
    println!("=== done ===");
}


