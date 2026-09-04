// noise_check.rs — 验证 md5 + DoublePerlinNoiseSampler 初步输出
// 后续：对照 Java/C++ 参照逐位（本项目 ref 数据）。
use WorldgenRust::noise::{DoublePerlinNoiseSampler, NoiseParameters};
use WorldgenRust::xoroshiro::XoroshiroRandom;
use WorldgenRust::md5::md5_lo_hi;

fn main() {
    let (lo, hi) = md5_lo_hi("octave_0");
    println!("md5(octave_0) lo=0x{:016x} hi=0x{:016x}", lo, hi);

    // continentalness: first_octave=-9, amplitudes（Java worldgen 参数）
    let params = NoiseParameters {
        first_octave: -9,
        amplitudes: vec![1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
    };
    let mut random = XoroshiroRandom::new(0);
    let sampler = DoublePerlinNoiseSampler::new(&mut random, &params);
    for (x, y, z) in [(0.0, 0.0, 0.0), (1.5, -2.0, 3.25), (100.0, 50.0, -40.0)] {
        println!("noise({},{},{}) = {:.12}", x, y, z, sampler.sample(x, y, z));
    }
}
