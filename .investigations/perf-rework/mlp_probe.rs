// mlp_probe.rs — Rust 软流（K 路交错）vs 顺序（read_volatile 防 LLVM 优化访存）
// 同 C++ mlp_probe.cpp 基准；用 read_volatile 强制真实访存（防 LLVM 重排/合并/消除），
// 否则 -O 会优化掉依赖链的访存延迟（初版 0.06us/点 = 优化产物，不可比）。
// 用法: mlp_probe <N> <mode=1 seq 2 soft4 3 soft8>
use std::ptr;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(400_000);
    let mode: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2);
    const L: usize = 15;
    let sz: usize = 32 * 1024 * 1024;   // 32MB（超 L3 16.5MB，模拟 production 真实 DRAM 访存 miss）

    let a: Vec<f64> = (0..sz).map(|i| i as f64 * 0.001).collect();
    let b: Vec<f64> = (0..sz).map(|i| ((i * 3) % 103) as f64 * 0.01).collect();
    let idx: Vec<u32> = (0..sz).map(|i| ((i as u64 * 2654435761u64) % (sz as u64)) as u32).collect();

    let t = Instant::now();
    let mut acc = 0.0f64;
    if mode == 1 {
        for i in 0..n {
            let mut d = unsafe { ptr::read_volatile(&a[idx[i % sz] as usize]) };
            for l in 0..L {
                let x = unsafe { ptr::read_volatile(&b[idx[(i + l * 17) % sz] as usize]) };
                d = d + x * 0.5 - 1.1;
                d = d * 0.999 + 0.001;
            }
            acc += d;
        }
    } else {
        let k = if mode == 2 { 4 } else { 8 };
        let mut d = [0.0f64; 8];
        let mut base = [0usize; 8];
        let mut i = 0;
        while i < n {
            for kk in 0..k { base[kk] = i + kk; d[kk] = unsafe { ptr::read_volatile(&a[idx[base[kk] % sz] as usize]) }; }
            for l in 0..L {
                for kk in 0..k {
                    let x = unsafe { ptr::read_volatile(&b[idx[(base[kk] + l * 17) % sz] as usize]) };
                    d[kk] = d[kk] + x * 0.5 - 1.1;
                    d[kk] = d[kk] * 0.999 + 0.001;
                }
            }
            for kk in 0..k { acc += d[kk]; }
            i += k;
        }
    }
    let per = t.elapsed().as_secs_f64() * 1e6 / n as f64;
    let kn = if mode == 1 { 1 } else if mode == 2 { 4 } else { 8 };
    eprintln!("[RUST-MLP] mode={} N={} L={} K={} wall={:.1}ms per={:.2}us acc={:.2}",
             mode, n, L, kn, t.elapsed().as_secs_f64() * 1e3, per, acc);
}
