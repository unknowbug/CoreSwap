// gpu_mt_wall_retest.rs — P-C2：0.61× 双线程异常无探针复测（260903-08）
// 判据（judge 两步走）：整批 wall + 调用计数，无 per-call 计时/无原子热路径（计数仅每次 fill 一次）。
// 口径可比性（§9.7）：同 seed/spv/dll/坐标集/n=8（原探针实参口径）+ 增设 n=6144 全批对照。
// 附带：P0「fill 全同步串行」数据层验证——fill 返回后输出立即可用（返回即含 readback ⇒ 同步）。
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const SEED: i64 = -8248318472910187742; // 与 260903-04 原探针同 seed（可比性）
const SPV: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\worldgen\\gpu-assets\\final_density.spv";
const DLL: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\build-msvc\\bin\\gpu_ffi.dll";
const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;

static FILL_CALLS: AtomicUsize = AtomicUsize::new(0);

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

#[derive(Clone, Copy)] struct H(*mut c_void);
unsafe impl Send for H {}
unsafe impl Sync for H {}

fn corner_coords(chunks: &[(i32, i32)]) -> Vec<i32> {
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

fn median(v: &mut [f64]) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }

#[inline] fn do_fill(fill: FillFn, hp: H, c: &[i32], n: i32, o: &[f32]) {
    unsafe { fill(hp.0, c.as_ptr(), n, o.as_ptr() as *mut f32); }
}

fn main() {
    println!("=== gpu_mt_wall_retest (260903-08, P-C2) ===");
    let name = wide(DLL);
    let m = unsafe { LoadLibraryW(name.as_ptr()) };
    assert!(!m.is_null(), "LoadLibraryW failed");
    let create: CreateFn = unsafe { get_sym(m, "gpu_ffi_create") };
    let fill: FillFn = unsafe { get_sym(m, "gpu_ffi_fill") };
    let destroy: DestroyFn = unsafe { get_sym(m, "gpu_ffi_destroy") };
    let spv = format!("{}\0", SPV);
    let h = unsafe { create(SEED as u64, spv.as_ptr()) };
    assert!(!h.is_null(), "create failed");
    let handle: Arc<Mutex<H>> = Arc::new(Mutex::new(H(h)));

    let chunks = vec![(-288, -256), (-287, -256), (-286, -255), (-288, -255), (0, 0), (1, 0), (0, 1), (-3, 2)];
    let coords = corner_coords(&chunks);
    let total_pts = coords.len() / 3;
    let out_ref = vec![0f32; total_pts];

    for &n_per_fill in &[8i32, total_pts as i32] {
        let rounds = 5usize; // S/P 交替轮，取中位数
        let calls_per_round = 20usize;
        let mut ser_ms = Vec::new();
        let mut par_ms = Vec::new();
        // 预热（每口径 6 次）
        for _ in 0..6 {
            let _g = handle.lock().unwrap();
            unsafe { fill(h, coords.as_ptr(), n_per_fill, out_ref.as_ptr() as *mut f32); }
            FILL_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        for r in 0..rounds {
            // 串行：1 线程 × 20 次调用
            let t0 = std::time::Instant::now();
            for _ in 0..calls_per_round {
                let _g = handle.lock().unwrap();
                unsafe { fill(h, coords.as_ptr(), n_per_fill, out_ref.as_ptr() as *mut f32); }
                FILL_CALLS.fetch_add(1, Ordering::Relaxed);
            }
            let s = t0.elapsed().as_secs_f64() * 1e3;
            ser_ms.push(s);
            // 并行：2 线程 × 10 次调用（总调用数相同 = 计数核对 20）
            let before = FILL_CALLS.load(Ordering::Relaxed);
            let t0 = std::time::Instant::now();
            let hp = H(h);
            std::thread::scope(|s| {
                let hA = &handle; let c = &coords; let o = &out_ref;
                s.spawn(move || for _ in 0..calls_per_round/2 { let _g = hA.lock().unwrap(); do_fill(fill, hp, c, n_per_fill, o); FILL_CALLS.fetch_add(1, Ordering::Relaxed); });
                s.spawn(move || for _ in 0..calls_per_round/2 { let _g = hA.lock().unwrap(); do_fill(fill, hp, c, n_per_fill, o); FILL_CALLS.fetch_add(1, Ordering::Relaxed); });
            });
            let p = t0.elapsed().as_secs_f64() * 1e3;
            assert_eq!(FILL_CALLS.load(Ordering::Relaxed) - before, calls_per_round, "call count mismatch");
            par_ms.push(p);
            println!("[n={}] round {}: serial={:.2}ms parallel={:.2}ms ratio(par/ser)={:.3}×", n_per_fill, r + 1, s, p, p / s);
        }
        let (sm, pm) = (median(&mut ser_ms), median(&mut par_ms));
        println!("[n={}] MEDIAN serial={:.2}ms parallel={:.2}ms ratio={:.3}× (mutex 真串行化预期 ≈1.0×)", n_per_fill, sm, pm, pm / sm);
    }

    // P0 数据层验证：fill 返回后输出立即可用（正确性对照 ⇒ readback 在返回前完成 ⇒ 同步）
    {
        let _g = handle.lock().unwrap();
        unsafe { fill(h, coords.as_ptr(), total_pts as i32, out_ref.as_ptr() as *mut f32); }
    }
    let mut out2 = vec![f32::NAN; total_pts]; // 先污染
    {
        let _g = handle.lock().unwrap();
        unsafe { fill(h, coords.as_ptr(), total_pts as i32, out2.as_mut_ptr()); }
    }
    let mism = out_ref.iter().zip(&out2).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    println!("[sync-check] fill-return-immediate-valid: mismatch={}/{} → {}", mism, total_pts,
        if mism == 0 { "SYNCHRONOUS（返回即可用）" } else { "NOT-READY-AT-RETURN（异步残留）" });
    println!("[count] total fill calls = {}", FILL_CALLS.load(Ordering::Relaxed));
    unsafe { destroy(h) };
    println!("=== done ===");
}
