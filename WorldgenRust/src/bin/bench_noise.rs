// bench_noise.rs — PerlinNoiseSampler.sample 吞吐基准（反映 sample_section 成本）。
// 作用：对比 编译开 avx（RUSTFLAGS=-C target-feature=+avx）vs 不开 的 noise 采样吞吐。
use std::time::Instant;
use WorldgenRust::xoroshiro::XoroshiroRandom;
use WorldgenRust::noise::PerlinNoiseSampler;

fn main() {
    let mut rand = XoroshiroRandom::new(42);
    let noise = PerlinNoiseSampler::new(&mut rand);
    // 采样大量点（模拟 noise 密集 corners 采样）
    let n = 2_000_000usize;
    // 预热
    for i in 0..20000 { let _ = noise.sample(i as f64*0.001, (i%100) as f64, (i%7) as f64); }
    let t0 = Instant::now();
    let mut acc = 0.0;
    for i in 0..n {
        acc += noise.sample(i as f64 * 0.001, (i % 100) as f64, (i % 7) as f64);
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("Perlin sample: {} 次 = {:.2}ms ({:.2}ns/次) sum={:.4}", n, dt*1e3, dt/n as f64*1e9, acc);
    println!("(对比：RUSTFLAGS='-C target-feature=+avx' cargo run --release --bin bench_noise 测 avx 后吞吐)");
}
