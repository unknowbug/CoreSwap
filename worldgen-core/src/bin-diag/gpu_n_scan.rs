// gpu_n_scan.rs — GPU (b) 全树单 dispatch 重估：n 扫描（260903-17）
// 判据（NEXT_SESSION 260903-16 开工点）：n=768/6144/49152/196608 四档，
//   验证 49152+ 点边际成本是否显著低于 0.233ms/点（260903-16 旧读数，本轮重采复现核对结构一致性）；
//   折算 per-chunk（768 点）是否可达 <4.5ms（对标 8 线程 density 段，Amdahl 立项判据）。
// 纪律：#28 严格串行（本探针只单线程 + 运行期间无任何并发 bench）；#29 时间戳头注（下方 BUILD-TIME）。
// 口径（§9.7）：载体=gpu_ffi_fill 同步 fill（含 readback）；seed=-8248318472910187742（同 260903-04/08 可比）；
//   坐标=corner 采样（每 chunk 768 点，同 corner_coords 口径）；无并行段（纯串行中位数）。
// BUILD-TIME: (编译时填充占位，见输出首行打印的编译环境说明——实际以 exe LastWriteTime 为准，用前核 #16)
use std::ffi::c_void;

const SEED: i64 = -8248318472910187742;
const SPV: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\worldgen\\gpu-assets\\final_density.spv";
const DLL: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\build-msvc\\bin\\gpu_ffi.dll";
const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;

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

fn main() {
    println!("=== gpu_n_scan (260903-17, GPU-(b) re-eval) ===");
    println!("[meta] strict-serial single-thread; seed={}; per-chunk pts=768 (corner 口径同 260903-04/08)", SEED);
    println!("[meta] build-time: see exe LastWriteTime (核 #16: bin-diag 用前必须核产物时间戳)");
    let name = wide(DLL);
    let m = unsafe { LoadLibraryW(name.as_ptr()) };
    assert!(!m.is_null(), "LoadLibraryW failed");
    let create: CreateFn = unsafe { get_sym(m, "gpu_ffi_create") };
    let fill: FillFn = unsafe { get_sym(m, "gpu_ffi_fill") };
    let destroy: DestroyFn = unsafe { get_sym(m, "gpu_ffi_destroy") };
    let spv = format!("{}\0", SPV);
    let h = unsafe { create(SEED as u64, spv.as_ptr()) };
    assert!(!h.is_null(), "create failed");

    // 256 chunks 网格（16x16，含负坐标），取前 K 个 chunk 凑各档 n
    let mut chunk_list = Vec::new();
    for gx in 0..16i32 {
        for gz in 0..16i32 {
            chunk_list.push((gx - 8, gz - 8));
        }
    }
    let coords_full = corner_coords(&chunk_list); // 256*768 = 196608 点
    assert_eq!(coords_full.len() / 3, 196608);

    let ns: Vec<(i32, usize)> = vec![
        (768, 1), (6144, 8), (49152, 64), (196608, 256),
    ];
    let rounds = 7usize;
    let mut med_by_n: Vec<(i32, f64)> = Vec::new();

    for &(n_pts, n_chunks) in &ns {
        let coords = &coords_full[..n_chunks * 768 * 3];
        let mut out = vec![0f32; n_pts as usize];
        // 预热 3 次（本口径）
        for _ in 0..3 {
            unsafe { fill(h, coords.as_ptr(), n_pts, out.as_mut_ptr()); }
        }
        let mut ms_v = Vec::new();
        for r in 0..rounds {
            let t0 = std::time::Instant::now();
            unsafe { fill(h, coords.as_ptr(), n_pts, out.as_mut_ptr()); }
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            ms_v.push(ms);
            println!("[n={}] round {}/{}: {:.2}ms", n_pts, r + 1, rounds, ms);
        }
        let med = median(&mut ms_v);
        println!("[n={}] chunks={} MEDIAN={:.2}ms  per-pt={:.4}ms  per-chunk(768pt)={:.2}ms",
            n_pts, n_chunks, med, med / n_pts as f64, med / (n_pts as f64 / 768.0));
        med_by_n.push((n_pts, med));
    }

    // 边际成本曲线（相邻档差分）
    println!("--- marginal cost (相邻档差分) ---");
    for w in med_by_n.windows(2) {
        let (n0, t0) = w[0]; let (n1, t1) = w[1];
        println!("[{}→{}] marginal={:.4}ms/pt  delta_t={:.2}ms", n0, n1, (t1 - t0) / ((n1 - n0) as f64), t1 - t0);
    }
    // sanity：末档输出非全零（防假跑）
    let out = vec![0f32; 768];
    unsafe { fill(h, coords_full.as_ptr(), 768, out.as_ptr() as *mut f32); }
    let nz = out.iter().filter(|v| **v != 0.0).count();
    println!("[sanity] n=768 output nonzero: {}/768 (非零面需 >0)", nz);

    unsafe { destroy(h) };
    println!("=== done (strict serial, no concurrent bench during run) ===");
}
