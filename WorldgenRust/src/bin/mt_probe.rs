// mt_probe.rs — 多线程密度采样验证：共享 Arc<DensityFunction> 树，N 线程各 fill 不同 chunk，验证结果与单线程一致 + 测墙钟扩展。
// 目的：验证 arc + thread_local 缓存后，Rust 密度树跨线程共享采样正确（无 cache 争用），并发有扩展。
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn build_tree(seed: u64) -> Arc<DensityFunction> {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";
    let mut db = DensityBuilder::new(seed, -64, 384);
    db.load_noise_params_file(noise_params_path).unwrap();
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));
    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    Arc::new(db.build_node(fd).unwrap())
}

// fill 一个 chunk 的部分密度网格（16x16 列 × y∈[-64..320) step 8 = 2048 点），返回累加和（代表该 chunk 密度特征）
fn fill_chunk(tree: &DensityFunction, cx: i32, cz: i32) -> f64 {
    let mut sum = 0.0f64;
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16 + bx, cz*16 + bz);
        for y in (-64..320).step_by(8) {
            sum += tree.sample(&NoisePos { x, y, z });
        }
    }}
    sum
}

fn main() {
    let seed = 8576294172403134396u64;
    let tree = build_tree(seed);
    let chunks: Vec<(i32,i32)> = (0..8).map(|i| (40 + i, -30 + i)).collect();

    // 单线程参照（每 chunk 连续 fill，树预热，thread_local 缓存已建）
    let seq: Vec<f64> = chunks.iter().map(|&(c,d)| fill_chunk(&tree, c, d)).collect();

    println!("seq sum[0]={:.4} (reference)", seq[0]);
    for &t in &[1usize, 2, 4, 8] {
        let n = chunks.len();
        let t0 = Instant::now();
        // 每线程分一段 chunk，各持 tree.clone()（共享 Arc）
        let handles: Vec<_> = (0..t).map(|ti| {
            let tree = tree.clone();
            let chunks = chunks.clone();
            thread::spawn(move || {
                let mut local = Vec::new();
                let start = ti * n / t;
                let end = (ti + 1) * n / t;
                for i in start..end { let (c,d) = chunks[i]; local.push((i, fill_chunk(&tree, c, d))); }
                local
            })
        }).collect();
        let mut got = vec![0.0f64; n];
        for h in handles {
            for (i, v) in h.join().unwrap() { got[i] = v; }
        }
        let wall = t0.elapsed().as_secs_f64() * 1000.0;
        // 结果一致性
        let mut mism = 0u32;
        for i in 0..n { if (got[i] - seq[i]).abs() > 1e-9 { mism += 1; } }
        println!("T={} threads: wall={:.2}ms  mismatch={}/{}", t, wall, mism, n);
    }
}
