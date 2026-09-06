// end_probe_diag.rs — D1/D2 对拍：Rust SimplexNoiseSampler + EndIslands vs Java EndProbe 真值
// 编译：rustc 单编（bin-diag 不参与默认构建），见 .tmp/end-takeover-260906/run-probe.ps1
use WorldgenRust::simplex_noise::{EndIslandsNoise, SimplexNoiseSampler};
use WorldgenRust::chunkrandom::CheckedRandom;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args[1].parse().unwrap();
    let mut rnd = CheckedRandom::new(seed);
    rnd.skip(17292);
    let sx = SimplexNoiseSampler::new(&mut rnd);
    println!("[EndProbeRust] seed={}", seed);
    println!("origin={:.17e} {:.17e} {:.17e}", sx.origin_x, sx.origin_y, sx.origin_z);
    // perm 校验和：与 Java ph = ph*31 + perm[i] 一致（i64 回绕）
    let mut ph: i64 = 0;
    for i in 0..256 { ph = ph.wrapping_mul(31).wrapping_add(sx.perm_at(i) as i64); }
    println!("permChecksum={} perm0={} perm1={} perm255={}", ph, sx.perm_at(0), sx.perm_at(1), sx.perm_at(255));
    let sxs = [0.5f64, -3.25, 17.75, -100.5, 4096.25];
    let sys = [0.5f64, 1.75, -9.5, 88.25, -4096.75];
    for i in 0..sxs.len() {
        println!("simplex({:.2},{:.2})={:.17e}", sxs[i], sys[i], sx.sample(sxs[i], sys[i]));
    }
    let e = EndIslandsNoise::new(seed);
    for a in (2..args.len()).step_by(2) {
        let bx: i32 = args[a].parse().unwrap();
        let bz: i32 = args[a + 1].parse().unwrap();
        let d = e.sample(bx, bz);
        let biome = if d > 0.25 { "end_highlands" } else if d >= -0.0625 { "end_midlands" } else if d < -0.21875 { "small_end_islands" } else { "end_barrens" };
        println!("erosion(x={},z={})={:.17e} biome={}", bx, bz, d, biome);
    }
}
