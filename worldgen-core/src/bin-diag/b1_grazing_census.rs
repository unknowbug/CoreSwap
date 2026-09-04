// b1_grazing_census.rs — B1 封闭验证：全区 4×4 chunk × y0..127 精确 d 普查，统计零面擦边集合与 13 差异格的重合度（bin-diag）
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = std::env::var("WG_SEED").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(8576294172403134396);
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] create_for_dim"); return; }
    };
    let diffs: [(i32, i32, i32, char); 13] = [
        (51200, 75, 51339, 'V'), (51202, 33, 51336, 'V'), (51204, 109, 51361, 'R'),
        (51213, 97, 51381, 'R'), (51221, 84, 51337, 'R'), (51221, 55, 51339, 'R'),
        (51222, 96, 51354, 'R'), (51222, 72, 51365, 'V'), (51227, 73, 51338, 'R'),
        (51229, 43, 51334, 'R'), (51231, 48, 51329, 'R'), (51240, 73, 51348, 'R'),
        (51256, 51, 51364, 'R'),
    ];
    let diffset: std::collections::HashSet<(i32,i32,i32)> = diffs.iter().map(|&(x,y,z,_)| (x,y,z)).collect();
    let x0 = 51200i64; let z0 = 51328i64; // chunk 3200..3203, 3208..3211
    println!("[sanity] seed={} census 64x64 blocks x y0..127 = 524288 pts", seed);
    let mut hist = [0usize; 6]; // <-6, -3, -1.5, 0, +1.5, +3 (log10 exponents buckets for |d|)
    let mut grazing = Vec::new();
    let mut total = 0usize;
    for dx in 0..64i64 {
        for dz in 0..64i64 {
            let x = (x0 + dx) as i32; let z = (z0 + dz) as i32;
            for y in 0..128i32 {
                let d = h.sample_density_exact(x, y, z);
                total += 1;
                let a = d.abs();
                let b = if a == 0.0 { 5 } else if a < 1e-6 { 4 } else if a < 1e-5 { 3 } else if a < 1e-4 { 2 } else if a < 1e-3 { 1 } else { 0 };
                hist[b] += 1;
                if a < 1e-5 { grazing.push((x, y, z, d)); }
            }
        }
        if dx % 16 == 0 { eprintln!("progress dx={}", dx); }
    }
    println!("[hist] |d| buckets: <1e-3={} <1e-4={} <1e-5={} <1e-6={} ==0={} total={}", hist[0], hist[1], hist[2], hist[3], hist[4], total);
    println!("[grazing] |d|<1e-5 count={}", grazing.len());
    let gset: std::collections::HashSet<(i32,i32,i32)> = grazing.iter().map(|&(x,y,z,_)| (x,y,z)).collect();
    let inter = gset.intersection(&diffset).count();
    println!("[overlap] grazing&diff={} diff={} grazing_only={} diff_not_grazing={}", inter, diffset.len(), gset.len()-inter, diffset.len()-inter);
    for &(x,y,z,d) in grazing.iter().filter(|g| !diffset.contains(&(g.0,g.1,g.2))).take(20) {
        println!("  grazing-only {},{},{},{:.6e}", x, y, z, d);
    }
    for &(x,y,z,_) in diffs.iter().filter(|c| !gset.contains(&(c.0,c.1,c.2))) {
        println!("  diff-not-grazing {},{},{}", x, y, z);
    }
}
