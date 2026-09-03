// qaq1_grid_thrash_probe.rs — Q-AQ1 一锤定音：冷态 classify 循环中 Interpolated 单槽 key 抖动的
// build_grid 全量重建次数（GRID_ARG_SAMPLES 增量）vs 暖态对照（260903-10）。
// 预期（b1' 假设）：冷态每 chunk 增量 = 数十次 × 1225 角点级 arg 采样；暖态 ≈ 0。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::density::{GRID_ARG_SAMPLES, DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::terrain::{AquiferSource, ChunkDensitySampler, DensityMacroSampler, VanillaAquifer};

struct StubBiome;
impl WorldgenRust::terrain::BiomeSource for StubBiome {
    fn biome(&self, _pos: &NoisePos) -> String { String::new() }
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = 8576294172403134396;
    let min_y: i32 = -64; let height: i32 = 384;
    let mut db = DensityBuilder::new(seed as u64, min_y, height);
    db.set_df_ns("overworld");
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
    let df_dir2 = df_dir.clone();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df_dir2, name)).unwrap()
    }));
    let settings = parse(&std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let b = |db: &mut DensityBuilder, k: &str| -> Arc<DensityFunction> {
        Arc::new(db.build_node(router.get(k).unwrap()).ok().unwrap())
    };
    let barrier = b(&mut db, "barrier"); let flooded = b(&mut db, "fluid_level_floodedness");
    let spread = b(&mut db, "fluid_level_spread"); let lava = b(&mut db, "lava");
    let erosion = b(&mut db, "erosion"); let depth = b(&mut db, "depth");
    let init = b(&mut db, "initial_density_without_jaggedness");
    let tree = b(&mut db, "final_density");
    let splitter = match db.random_deriver() {
        WorldgenRust::legacy_random::RsSplitter::Xoro(s) => s.clone(),
        _ => panic!("overworld 需 Xoroshiro splitter"),
    };
    let macro_sampler = DensityMacroSampler::new(&tree, min_y, height);
    let biome = StubBiome;

    let run_fill = |slices: &[f64], aqua: &mut VanillaAquifer, cx: i32, cz: i32| -> usize {
        let mut applied = 0usize;
        for lz in 0..16i32 { for lx in 0..16i32 {
            let x = cx*16+lx; let z = cz*16+lz;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                let d = macro_sampler.sample_interp(slices, &NoisePos { x, y, z });
                let kind = aqua.classify(x, y, z, d);
                if d <= 0.0 { applied += 1; }
                std::hint::black_box(kind);
            }
        }}
        applied
    };

    println!("=== qaq1_grid_thrash_probe (260903-10) seed={} ===", seed);
    // 冷态：每 chunk 新建 Aquifer（生产语义），量 GRID_ARG_SAMPLES 增量 + 时间
    let n = 8;
    let mut cold_grid_rebuilds = 0u64; let mut cold_args = 0u64;
    let mut t_cold = 0.0f64;
    for i in 0..n {
        let cx = 200 + i; let cz = 200;
        let slices = macro_sampler.build_slices_for(cx, cz);
        let aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
            erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height);
        let mut va = VanillaAquifer { aq, enabled: true, skip_aquifer: false, sea_level: 63 };
        let g0 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
        let t = Instant::now();
        let _ = run_fill(&slices, &mut va, cx, cz);
        t_cold += t.elapsed().as_secs_f64() * 1e3;
        let g1 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
        cold_args += (g1 - g0) as u64; cold_grid_rebuilds += 1; // 每次翻 key 恰好一次 build_grid（增量=角点数，见下）
    }
    // 暖态：同一 chunk 同一 Aquifer 第二遍
    let cx = 200; let cz = 200;
    let slices = macro_sampler.build_slices_for(cx, cz);
    let aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
        erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height);
    let mut va = VanillaAquifer { aq, enabled: true, skip_aquifer: false, sea_level: 63 };
    let _ = run_fill(&slices, &mut va, cx, cz); // 灌缓存
    let g0 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);
    let t = Instant::now();
    let _ = run_fill(&slices, &mut va, cx, cz);
    let t_warm = t.elapsed().as_secs_f64() * 1e3;
    let g1 = GRID_ARG_SAMPLES.load(std::sync::atomic::Ordering::Relaxed);

    println!("[cold] {} chunks: total={:.2}ms per_chunk={:.2}ms  GRID_ARG_SAMPLES 增量={} (per chunk {:.0} 角点采样)",
        n, t_cold, t_cold / n as f64, cold_args, cold_args as f64 / n as f64);
    println!("[warm] per_chunk={:.2}ms  GRID_ARG_SAMPLES 增量={}", t_warm, g1 - g0);
    println!("[判读] 冷态增量 ≫ 暖态 → Interpolated 单槽抖动成立；增量双态 ≈ 0 → 抖动机制不成立（260903-10 judge R4：本行须与实测对读，勿预写结论）");
}
