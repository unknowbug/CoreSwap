// end_abi_diag.rs — 模拟服务端调用链：wg_create → wg_set_flags(3) → wg_fill_blocks_multi
use WorldgenRust::api::{wg_create, wg_set_flags, wg_fill_blocks_multi, wg_destroy};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args[1].parse().unwrap();
    let dir = std::ffi::CString::new(args[2].clone()).unwrap();
    let settings = std::ffi::CString::new("end.json").unwrap();
    let biome = std::ffi::CString::new("").unwrap();
    let h = unsafe { wg_create(seed, dir.as_ptr(), settings.as_ptr(), biome.as_ptr(), 128) };
    println!("[handle] {}", h as usize);
    if h.is_null() { return; }
    unsafe { wg_set_flags(h, 3) };
    let cx = [0i32]; let cz = [0i32];
    let mut buf = vec![0i32; 16 * 16 * 128];
    let bufs = [buf.as_mut_ptr()];
    let r = unsafe { wg_fill_blocks_multi(h, cx.as_ptr(), cz.as_ptr(), bufs.as_ptr(), 1, -1) };
    let nz = buf.iter().filter(|&&v| v != 0).count();
    println!("[fill] r={} nonzero={}/{}", r, nz, buf.len());
    unsafe { wg_destroy(h) };
}
