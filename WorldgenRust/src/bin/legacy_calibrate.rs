// legacy_calibrate.rs — Legacy-Perlin 对拍校准探针（语义专项）。
// 目标：定位 Rust LegacyRandom(0)+new_legacy(-7,[1,1]) 与 Java CheckedRandom(0)+createLegacy(-7,[1,1])
// 的数值差异段。分三段输出：
//   S1 LCG 裸输出：setSeed(0) 后前 8 次 next(32)（对照 Java java.util.Random/CheckedRandom 同序列）
//   S2 Perlin 构造产物：origin xyz + permutation 前 16（对照 Java PerlinNoiseSampler 构造）
//   S3 采样值：DoublePerlin legacy 在 mismatch 坐标 (x*0.25, 0, z*0.25) 的值（对照 Java 同构造 sample）
// Java 侧参照：Biome6Probe 扩展（-Dbiome6cal=true）输出同格式。
use WorldgenRust::legacy_random::{LegacyRandom, RsRandom};
use WorldgenRust::noise::{DoublePerlinNoiseSampler, OctavePerlinNoiseSampler};

fn main() {
    let seed: i64 = -8248318472910187742; // 参照 seed（climate 特例固定种子 0 与 seed 无关，仅打印）

    // ===== S1: LCG 裸输出（CheckedRandom(0) 语义：setSeed(0)→(0^25214903917)&mask）=====
    println!("=== S1: LegacyRandom::new(0) LCG next(32) x8 ===");
    let mut r = LegacyRandom::new(0);
    let seq: Vec<i32> = (0..8).map(|_| r.next(32)).collect();
    println!("{:?}", seq);
    // 该序列应与 Java CheckedRandom(0).next(32) x8 一致（LCG 25214903917/11/2^48）

    // next_long / next_double / next_float（Perlin 构造消费的接口）
    let mut r2 = LegacyRandom::new(0);
    println!("next_long x4: {:?}", (0..4).map(|_| r2.next_long()).collect::<Vec<i64>>());
    let mut r3 = LegacyRandom::new(0);
    println!("next_double x3: {:?}", (0..3).map(|_| {
        let i = r3.next(26); let j = r3.next(27);
        let l = ((i as i64) << 27) + j as i64;
        (l as f32 * 1.110223E-16f32) as f64
    }).collect::<Vec<f64>>());

    // ===== S2: Octave createLegacy(-7, [1,1]) 构造产物（blended/temperature/vegetation 用）=====
    println!("=== S2: OctavePerlinNoiseSampler::new_legacy(0, -7, [1,1]) ===");
    let mut r4 = LegacyRandom::new(0);
    let mut rr4 = RsRandom::Legacy(r4);
    let oct = OctavePerlinNoiseSampler::new_legacy(&mut rr4, -7, &[1.0, 1.0]);
    for octave in 0..2 {
        if let Some(p) = oct.get_octave(octave) {
            let o = p.origin(); println!("octave{}: origin=({:.6},{:.6},{:.6})", octave, o.0, o.1, o.2);
        } else {
            println!("octave{}: <null>", octave);
        }
    }
    // DoublePerlin createLegacy：first/second 两个 Octave 连续消耗同一 random
    let mut r5 = LegacyRandom::new(0);
    let mut rnd = RsRandom::Legacy(r5);
    let dp = DoublePerlinNoiseSampler::new_legacy(&mut rnd, -7, &[1.0, 1.0]);
    let _ = dp; // 构造成功即可（内部随机消耗序与 Java createLegacy 对拍）

    // ===== S3: 采样值 @ mismatch 坐标 (x*0.25, 0, z*0.25)*1 =====
    println!("=== S3: DoublePerlin legacy sample ===");
    let pts: [(i32, i32, i32); 10] = [
        (5, 1, 0), (12, 1, 0), (10, 1, 1), (14, 1, 2), (7, 1, 3),
        (11, 1, 3), (5, 1, 4), (8, 1, 4), (2, 1, 5), (6, 1, 5),
    ];
    let mut r6 = LegacyRandom::new(0);
    let mut rnd6 = RsRandom::Legacy(r6);
    let dp2 = DoublePerlinNoiseSampler::new_legacy(&mut rnd6, -7, &[1.0, 1.0]);
    for (x, y, z) in pts {
        let v = dp2.sample(x as f64 * 0.25, 0.0, z as f64 * 0.25);
        println!("({},{},{}) -> {:.6}", x, y, z, v);
    }
    println!("seed_note: climate 固定种子与 worldSeed 无关（seed 参数仅打印用）");
    let _ = seed;
}

