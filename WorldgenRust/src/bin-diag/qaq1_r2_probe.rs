// qaq1_r2_probe.rs — Q-AQ1 judge R2 调和探针：est 单价双口径（3557 vs 1646 ns/iter）区分实验（260903-10）。
// 假设：coldpath 探针 C 段的 1646ns/iter 是同进程 A/B 段预热 thread_local 缓存的假冷；
//       独立进程冷态应 ≈3µs+/iter（与 A 段扫描口径一致）。
// 用法（两次独立进程运行，互不预热）：
//   qaq1_r2_probe.exe fill   → 冷/暖 fill 循环（无 bp/wl 计数器，counter-free 差分）
//   qaq1_r2_probe.exe diag   → 新鲜 Aquifer diag_fluidlevel_cost 单轮（SURF 计数 → ns/iter）
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::aquifer::{self, Aquifer};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::terrain::{AquiferSource, ChunkDensitySampler, DensityMacroSampler, VanillaAquifer};

struct StubBiome;
impl WorldgenRust::terrain::BiomeSource for StubBiome {
    fn biome(&self, _pos: &NoisePos) -> String { String::new() }
}

fn build(wg_dir: &str, seed: i64, min_y: i32, height: i32) -> (
    Arc<DensityFunction>, Arc<DensityFunction>, Arc<DensityFunction>, Arc<DensityFunction>,
    Arc<DensityFunction>, Arc<DensityFunction>, Arc<DensityFunction>, Arc<DensityFunction>,
    WorldgenRust::xoroshiro::XoroshiroSplitter, DensityMacroSampler,
) {
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
        _ => panic!("need xoro"),
    };
    let macro_sampler = DensityMacroSampler::new(&tree, min_y, height);
    (barrier, flooded, spread, lava, erosion, depth, init, tree, splitter, macro_sampler)
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = 8576294172403134396;
    let min_y = -64i32; let height = 384i32;
    let mode = std::env::args().nth(1).unwrap_or_default();
    let (barrier, flooded, spread, lava, erosion, depth, init, _tree, splitter, macro_sampler) = build(wg_dir, seed, min_y, height);

    if mode == "diag" {
        // 新鲜进程 + 新鲜 Aquifer：diag_fluidlevel_cost 单轮（est 冷扫描形态），SURF 计数开
        aquifer::aquifer_surf_watch(true);
        let cx = 200; let cz = 200;
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
            erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height);
        let t = Instant::now();
        let el = aq.diag_fluidlevel_cost(cx, cz, 1);
        let [sc, si] = aquifer::aquifer_surf_count_reset();
        println!("[diag-cold-fresh] get_fluid_level 98304 pts: {:.2}ms | surf calls={} iters={} | ns/iter={:.0}",
            el*1e3, sc, si, if si > 0 { el*1e9/si as f64 } else { 0.0 });
        // 第二轮（暖）对照
        let el2 = aq.diag_fluidlevel_cost(cx, cz, 1);
        println!("[diag-warm] {:.2}ms", el2*1e3);
        return;
    }

    // mode == fill：冷/暖 fill 循环，counter-free
    for i in 0..4 { let _ = 0; } // noop
    let n = 8;
    let mut t_cold = 0.0f64; let mut t_warm = 0.0f64;
    for i in 0..n {
        let cx = 200 + i; let cz = 200;
        let slices = macro_sampler.build_slices_for(cx, cz);
        let mut va = VanillaAquifer { aq: Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
            erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height),
            enabled: true, skip_aquifer: false, sea_level: 63 };
        let t = Instant::now();
        for lz in 0..16i32 { for lx in 0..16i32 {
            let x = cx*16+lx; let z = cz*16+lz;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                let d = macro_sampler.sample_interp(&slices, &NoisePos { x, y, z });
                let kind = va.classify(x, y, z, d);
                std::hint::black_box(kind);
            }
        }}
        t_cold += t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        for lz in 0..16i32 { for lx in 0..16i32 {
            let x = cx*16+lx; let z = cz*16+lz;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                let d = macro_sampler.sample_interp(&slices, &NoisePos { x, y, z });
                let kind = va.classify(x, y, z, d);
                std::hint::black_box(kind);
            }
        }}
        t_warm += t.elapsed().as_secs_f64() * 1e3;
    }
    println!("[fill] per chunk: cold={:.2}ms warm={:.2}ms excess={:.2}ms (counter-free, 8 chunks)",
        t_cold / n as f64, t_warm / n as f64, (t_cold - t_warm) / n as f64);
}
