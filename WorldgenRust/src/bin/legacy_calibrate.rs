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

    // CAL-TRACE: Perlin 构造逐调用（origin 3x nextDouble + permutation 256x nextIntBound(256-i)）
    let mut rt = RsRandom::Legacy(LegacyRandom::new(0));
    print!("[CAL-TRACE-D] ");
    for _ in 0..3 { print!("{:.12} ", rt.next_double()); }
    println!();
    print!("[CAL-TRACE-I] ");
    for i in 0..256 { print!("{},", rt.next_int_bound(256 - i)); }
    println!();
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

    // ===== S3b: DoublePerlin createLegacy(CheckedRandom(2), (-7,[1,1])) 采样（vegetation 位对拍）=====
    println!("=== S3b: DoublePerlin legacy seed=2 sample ===");
    let mut rs2 = LegacyRandom::new(2);
    let mut rnds2 = RsRandom::Legacy(rs2);
    let dp2 = DoublePerlinNoiseSampler::new_legacy(&mut rnds2, -7, &[1.0, 1.0]);
    let ptsb: [(i32, i32, i32); 10] = [
        (5, 1, 0), (12, 1, 0), (10, 1, 1), (14, 1, 2), (7, 1, 3),
        (11, 1, 3), (5, 1, 4), (8, 1, 4), (2, 1, 5), (6, 1, 5),
    ];
    for (x, y, z) in ptsb {
        let v = dp2.sample(x as f64 * 0.25, 0.0, z as f64 * 0.25);
        println!("({},{},{}) -> {:.6}", x, y, z, v);
    }
    // ===== S6: split 链对拍（worldSeed = -2032795982907864146）=====
    println!("=== S6: split chain (LocalRandom(ws).nextSplitter().split_str) ===");
    let ws: i64 = -2032795982907864146;
    let mut lr = LegacyRandom::new(ws);
    let sp = lr.next_splitter();
    println!("[SPLIT] worldSeed={} splitterSeed={}", ws, sp.seed);
    let mut rt = sp.split_str("minecraft:temperature");
    let vals: Vec<i32> = (0..4).map(|_| rt.next(32)).collect();
    println!("[SPLIT] temp stream: {:?}", vals);
    // ===== S7: temperature DoublePerlin modern Octave 对拍 =====
    // Java: OctavePerlinNoiseSampler.create(splitter.split("minecraft:temperature"), -10, [1.5,0,1,0,0,0])
    println!("=== S7: temperature modern Octave origins ===");
    let ws7: i64 = -2032795982907864146;
    let mut lr7 = LegacyRandom::new(ws7);
    let mut rr7 = RsRandom::Legacy(lr7);
    let sp7 = rr7.next_splitter();
    let mut r7 = sp7.split_str("minecraft:temperature");
    let amp_t7 = vec![1.5, 0.0, 1.0, 0.0, 0.0, 0.0];
    let oct7 = OctavePerlinNoiseSampler::new(&mut r7, -10, &amp_t7);
    for octave in 0..6 {
        match oct7.get_octave(octave) {
            Some(p) => { let o = p.origin(); println!("tempoct oct{}: origin=({:.6},{:.6},{:.6})", octave, o.0, o.1, o.2); }
            None => println!("tempoct oct{}: <null>", octave),
        }
    }
    let v = oct7.sample(3.0, 0.0, 0.0);
    println!("tempoct sample(3,0,0) = {:.6}", v);
    // ===== S8: try-seed sweep——对照 Java router 实测 firstSampler origins [21.877383, 47.402641] =====
    println!("=== S8: try-seed sweep (createLegacy(seed, -7, [1,1])) ===");
    let ws8: i64 = -2032795982907864146;
    let candidates: Vec<(&str, i64)> = vec![
        ("0", 0),
        ("worldSeed", ws8),
        ("worldSeed*2", ws8.wrapping_mul(2)),
        ("worldSeed+worldSeed", ws8.wrapping_add(ws8)),
    ];
    for (label, s) in candidates {
        let mut rr = RsRandom::Legacy(LegacyRandom::new(s));
        let oct = OctavePerlinNoiseSampler::new_legacy(&mut rr, -7, &[1.0, 1.0]);
        let o0 = oct.get_octave(1).map(|p| p.origin());  // samplers[0]（数组序）
        let o1 = oct.get_octave(0).map(|p| p.origin());  // samplers[1]
        let s0 = match o0 { Some(o) => format!("({:.6},{:.6},{:.6})", o.0, o.1, o.2), None => "<null>".into() };
        let s1 = match o1 { Some(o) => format!("({:.6},{:.6},{:.6})", o.0, o.1, o.2), None => "<null>".into() };
        println!("seed={} ({}): samplers[0]={} samplers[1]={}", s, label, s0, s1);
    }
    // ===== S9: vegetation try-seed sweep——对照 Java 实测 [22.432566, 96.308409] / [31.985435, 112.350786] =====
    println!("=== S9: vegetation try-seed sweep ===");
    let ws9: i64 = -2032795982907864146;
    let cand9: Vec<(&str, i64)> = vec![
        ("2", 2),
        ("1", 1),
        ("worldSeed", ws9),
        ("worldSeed+1", ws9.wrapping_add(1)),
        ("worldSeed+2", ws9.wrapping_add(2)),
        ("worldSeed*2+2", ws9.wrapping_mul(2).wrapping_add(2)),
    ];
    for (label, s) in cand9 {
        let mut rr = RsRandom::Legacy(LegacyRandom::new(s));
        let oct = OctavePerlinNoiseSampler::new_legacy(&mut rr, -7, &[1.0, 1.0]);
        let o0 = oct.get_octave(1).map(|p| p.origin());
        let o1 = oct.get_octave(0).map(|p| p.origin());
        let s0 = match o0 { Some(o) => format!("({:.6},{:.6},{:.6})", o.0, o.1, o.2), None => "<null>".into() };
        let s1 = match o1 { Some(o) => format!("({:.6},{:.6},{:.6})", o.0, o.1, o.2), None => "<null>".into() };
        let mut rt9 = oct.sample(3.0, 0.0, 0.0);
        println!("seed={} ({}): [0]={} [1]={} sample(3,0,0)={:.6}", s, label, s0, s1, rt9);
        let _ = &mut rt9;
    }
    // ===== S4: blended（old_blended_noise）legacy 构造逐 octave 对拍 =====
    // Java: lower/upper = createLegacy(-15, [1 x16]), interp = createLegacy(-7, [1 x8])，同一 CheckedRandom(0) 连续消耗
    println!("=== S4: blended Octave origins (CheckedRandom(0) 连续消耗) ===");
    let mut r7 = LegacyRandom::new(0);
    let mut rr7 = RsRandom::Legacy(r7);
    let amp_l: Vec<f64> = vec![1.0; 16];
    let amp_i: Vec<f64> = vec![1.0; 8];
    let lower = OctavePerlinNoiseSampler::new_legacy(&mut rr7, -15, &amp_l);
    let upper = OctavePerlinNoiseSampler::new_legacy(&mut rr7, -15, &amp_l);
    let interp = OctavePerlinNoiseSampler::new_legacy(&mut rr7, -7, &amp_i);
    for octave in 0..16 {
        match lower.get_octave(octave) {
            Some(p) => { let o = p.origin(); println!("lower oct{}: origin=({:.6},{:.6},{:.6})", octave, o.0, o.1, o.2); }
            None => println!("lower oct{}: <null>", octave),
        }
    }
    for octave in 0..2 {
        match interp.get_octave(octave) {
            Some(p) => { let o = p.origin(); println!("interp oct{}: origin=({:.6},{:.6},{:.6})", octave, o.0, o.1, o.2); }
            None => println!("interp oct{}: <null>", octave),
        }
    }
    // blended 采样：InterpolatedNoiseData（xz_scale 0.25 y_scale 0.375 xz_factor 80 y_factor 60 smear 8）
    let bn = WorldgenRust::density::InterpolatedNoiseData::new(lower, upper, interp, 0.25, 0.375, 80.0, 60.0, 8.0);
    println!("=== S5: blended sample @ mismatch 列 ===");
    let pts2: [(i32, i32); 6] = [(5, 0), (12, 0), (10, 1), (14, 2), (7, 3), (2, 5)];
    // 高 y 对拍（y=52：密度差最大的层）
    for (x, z) in pts2 {
        for yy in [1i32, 32, 52] {
            let pos = WorldgenRust::density::NoisePos { x, y: yy, z };
            println!("(x={},y={},z={}) blended = {:.6}", x, yy, z, bn.sample(&pos));
        }
    }
    let _ = upper;
}













