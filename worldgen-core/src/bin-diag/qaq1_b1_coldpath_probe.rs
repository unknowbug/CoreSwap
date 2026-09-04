// qaq1_b1_coldpath_probe.rs — Q-AQ1 b1 v2 决定性探针（260903-10）
// 背景：GRID_ARG_SAMPLES 冷/暖均 0 → InterpolatedData 重建机制被反驳（见 qaq1-b1-candidate §5 修正）。
// 新归因假设 H*：冷态超额 = estimate_surface_height 全量扫描对 initial_density 的逐次采样成本，
// 其中 initial_density（add/mul/y_clamped_gradient + reference depth/factor）的重量叶为
//   depth → sloped_cheese → base_3d_noise = old_blended_noise（InterpolatedNoiseData，无任何缓存，
//   每次 sample = 8+16 octave sample_ys，density.rs L177-223）；
//   factor → flat_cache(cache_2d(spline(...)))（单 chunk 槽，build_grid 25 角点不计数）。
//   warm 态 surface_cache 命中 → est 不采样 → 0；cold 态每 est 列 34 次全价采样。
//   证据包 F5 的 0.089µs/sample 与本假设矛盾（若真，est 全链仅 0.65ms）——本探针直接重测。
//
// 分段：
//   A  init 树扫描式采样微测（固定列，y 自顶向下步 8，模拟 est 扫描）→ ns/sample
//   B  sloped_cheese 树单独同测（归因 old_blended_noise 占比）
//   B2 base_3d_noise(old_blended_noise) 单独同测（下界）
//   C  Fresh Aquifer diag_fluidlevel_cost pass1(cold) vs pass2(warm)（同 aq 复用）
//      + SURF 计数器（calls/iterations）→ 隐含 ns/iteration
// 编译运行（主会话）：
//   Set-Location E:\PYTHON\CoreSwap\WorldgenRust; rustc --edition 2021 -O `
//     --extern WorldgenRust=target/release/libWorldgenRust.rlib -L target/release/deps `
//     src/bin-diag/qaq1_b1_coldpath_probe.rs -o target/release/qaq1_b1_coldpath_probe.exe; `
//     target/release/qaq1_b1_coldpath_probe.exe
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::aquifer::{aquifer_surf_count_reset, aquifer_surf_watch, Aquifer};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::{parse, JsonValue};

fn bench_tree(df: &Arc<DensityFunction>, cx: i32, cz: i32, min_y: i32, height: i32, tag: &str) {
    // est 扫描形态：固定列，y 从顶步长 -8 直到底；换列模拟 13 offset 分布
    let cols: [(i32, i32); 8] = [(cx * 16 + 8, cz * 16 + 8), (cx * 16 - 48, cz * 16 + 8),
        (cx * 16 + 64, cz * 16 + 8), (cx * 16 + 8, cz * 16 - 16), (cx * 16 + 8, cz * 16 + 64),
        (cx * 16 - 48, cz * 16 - 16), (cx * 16 + 64, cz * 16 + 64), (cx * 16 + 24, cz * 16 + 40)];
    let rounds = 4usize;
    let t = Instant::now();
    let mut sink = 0.0f64;
    for _r in 0..rounds {
        for (bx, bz) in cols {
            let bxq = (bx >> 2) << 2; let bzq = (bz >> 2) << 2;
            let mut l = min_y + height;
            while l >= min_y {
                sink += df.sample(&NoisePos { x: bxq, y: l, z: bzq });
                l -= 8;
            }
        }
    }
    let el = t.elapsed().as_secs_f64();
    let per_round_cols = cols.len();
    let iters_per_col = (height / 8 + 1) as usize;
    let total = (rounds * per_round_cols * iters_per_col) as f64;
    println!("  [{}] total {:.2}ms | ns/sample = {:.0} | sink={:.3}",
        tag, el * 1e3 / rounds as f64, el * 1e9 / total, sink);
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = 8576294172403134396;
    let min_y: i32 = -64;
    let height: i32 = 384;
    let cx = 200; let cz = 200;

    let mut db = DensityBuilder::new(seed as u64, min_y, height);
    db.set_df_ns("overworld");
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
    let df_dir2 = df_dir.clone();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df_dir2, name)).unwrap()
    }));
    let settings = parse(&std::fs::read_to_string(
        format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let b = |db: &mut DensityBuilder, k: &str| -> Arc<DensityFunction> {
        let node = db.build_node(router.get(k).unwrap()).ok().unwrap_or_else(|| panic!("build_node failed for key={}", k));
        Arc::new(node)
    };
    let load = |db: &mut DensityBuilder, name: &str| -> Arc<DensityFunction> {
        let p = format!("{}/{}.json", df_dir, name);
        let v: JsonValue = parse(&std::fs::read_to_string(&p).unwrap()).unwrap();
        Arc::new(db.build_node(&v).ok().unwrap())
    };
    let init = b(&mut db, "initial_density_without_jaggedness");
    let depth = b(&mut db, "depth");
    let factor = load(&mut db, "factor"); // factor 不在 noise_router，是 density_function 独立文件
    let sloped = load(&mut db, "sloped_cheese");
    let base3d = load(&mut db, "base_3d_noise");
    let barrier = b(&mut db, "barrier");
    let flooded = b(&mut db, "fluid_level_floodedness");
    let spread = b(&mut db, "fluid_level_spread");
    let lava = b(&mut db, "lava");
    let erosion = b(&mut db, "erosion");
    let splitter = match db.random_deriver() {
        WorldgenRust::legacy_random::RsSplitter::Xoro(s) => s.clone(),
        _ => panic!("overworld 需 Xoroshiro splitter"),
    };

    println!("qaq1_b1_coldpath_probe seed={} chunk=({},{}):", seed, cx, cz);
    // 预热 TLS 槽结构（不预热任何 key 语义——只测稳态采样成本）
    let _ = init.sample(&NoisePos { x: cx * 16, y: 0, z: cz * 16 });

    bench_tree(&init, cx, cz, min_y, height, "A  init tree scan-sample");
    bench_tree(&depth, cx, cz, min_y, height, "B  depth tree");
    bench_tree(&factor, cx, cz, min_y, height, "B1 factor tree");
    bench_tree(&sloped, cx, cz, min_y, height, "B2 sloped_cheese tree");
    bench_tree(&base3d, cx, cz, min_y, height, "B3 base_3d_noise (old_blended)");

    // C: Fresh Aquifer get_fluid_level pass1(cold) vs pass2(warm)
    let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
        erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx * 16, cz * 16, min_y, height);
    let mut surf_it = 0usize;
    let mut t_cold = 0.0; let mut t_warm = 0.0;
    for pass in 0..2 {
        aquifer_surf_watch(true);
        let t = Instant::now();
        let _ = aq.diag_fluidlevel_cost(cx, cz, 1);
        let el = t.elapsed().as_secs_f64() * 1e3;
        let c = aquifer_surf_count_reset();
        if pass == 0 { t_cold = el; surf_it = c[1]; } else { t_warm = el; }
        println!("  [C pass{}] get_fluid_level 98304 pts: {:.2}ms | surf calls={} iters={} | ns/iter={:.0}",
            pass, el, c[0], c[1], if c[1] > 0 { el * 1e6 / c[1] as f64 } else { 0.0 });
    }
    println!("  cold-warm excess = {:.2}ms | est iterations/pass1 = {} (implied per-sample = {:.0}ns)",
        t_cold - t_warm, surf_it, if surf_it > 0 { t_cold * 1e6 / surf_it as f64 } else { 0.0 });
    println!("[判读] A 的 ns/sample ≫ 89ns（F5 口径）且 C cold−warm ≈ 20-30ms → H* 成立：");
    println!("       冷态超额 = est 全量扫描 × initial_density 全价采样（old_blended_noise 无缓存为主）；");
    println!("       F5 的 0.089µs/sample 为错误基线（作废）。");
}
