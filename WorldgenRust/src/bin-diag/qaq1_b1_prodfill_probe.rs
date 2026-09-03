// qaq1_b1_prodfill_probe.rs — Q-AQ1 b1 候选决定性探针（260903-10）
// 目的：复刻生产 fill_chunk 循环（terrain.rs L265-287 逐行镜像），在同一进程内分段计时：
//   T0 宏观网格构建（DensityMacroSampler::build_slices_for）
//   T1 插值循环（无 aquifer：sample_interp only）
//   T2 插值 + classify（skip_aquifer=true，d<=0 早退 Air —— 对齐生产 no-aquifer 配置）
//   T3 插值 + classify（真实 aquifer，每 chunk 新建 Aquifer —— 对齐生产 fill_chunk_blocks L446 冷缓存语义）
//   T4 同 T3 但 Aquifer 跨 chunk 复用（暖缓存对照 —— 隔离「每 chunk 新建」结构性成本）
//   T5 d<=0 点直调 aq.apply（隔离 classify 包装层）
// 判读：T3-T2 ≈ 生产 aquifer 段 35.07ms → 差异在 fill 循环内部（b1 支持）；
//       T3-T2 ≪ 35ms → 35ms 不在 fill 循环内（carver apply / 计数盲区 → b2/b3）。
// 注意：T0-T2 每轮用同一 slices（生产 fill_chunk_blocks 也是每 chunk build 一次 slices）。
// 编译（主会话执行，bin-diag 不参与 cargo 默认构建）：
//   rustc --edition 2021 -O --extern WorldgenRust=target/release/libWorldgenRust.rlib -L target/release/deps
//     src/bin-diag/qaq1_b1_prodfill_probe.rs -o target/release/qaq1_b1_prodfill_probe.exe
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::aquifer::aquifer_bp_count_reset;
use WorldgenRust::aquifer::{aquifer_bp_watch, aquifer_wl_count_reset, aquifer_wl_watch, Aquifer};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::terrain::{
    AquiferSource, BiomeSource, BlockKind, ChunkDensitySampler, DensityMacroSampler, VanillaAquifer,
};

// BiomeSource stub（fill_chunk 里 biome 采样在列循环末尾，本探针聚焦 aquifer 段，stub 掉）
struct StubBiome;
impl BiomeSource for StubBiome {
    fn biome(&self, _pos: &NoisePos) -> String { String::new() }
}

const CX: i32 = 200;
const CZ: i32 = 200;
const NCHUNK: i32 = 8; // 8 chunk 线性带（对齐 F2 两批线性口径）
const ROUNDS: usize = 3;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = 8576294172403134396;
    let min_y: i32 = -64;
    let height: i32 = 384;

    // —— 生产同源构建（worldgen_handle.rs create L180-246 镜像，非 overworld 分支跳过）——
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
        Arc::new(db.build_node(router.get(k).unwrap()).ok().unwrap())
    };
    let barrier = b(&mut db, "barrier");
    let flooded = b(&mut db, "fluid_level_floodedness");
    let spread = b(&mut db, "fluid_level_spread");
    let lava = b(&mut db, "lava");
    let erosion = b(&mut db, "erosion");
    let depth = b(&mut db, "depth");
    let init = b(&mut db, "initial_density_without_jaggedness");
    let tree = b(&mut db, "final_density");
    let splitter = match db.random_deriver() {
        WorldgenRust::legacy_random::RsSplitter::Xoro(s) => s.clone(),
        _ => panic!("overworld 需 Xoroshiro splitter"),
    };

    // 生产宏观采样器（fill_chunk_blocks 用 DensityMacroSampler，worldgen_handle L199）
    let macro_sampler = DensityMacroSampler::new(&tree, min_y, height);
    let biome = StubBiome;
    let sea_level = 63i32;

    // 计数器开启（对照 F2 生产计数）
    aquifer_bp_watch(true);
    aquifer_wl_watch(true);
    let mut bp_total = [0usize; 2];
    let mut wl_total = [0usize; 2];

    // 预热一次（编译器/缓存稳定），不计
    {
        let slices = macro_sampler.build_slices_for(CX, CZ);
        let mut va = make_va(&barrier, &flooded, &spread, &lava, &erosion, &depth, &init, &splitter, CX, CZ, min_y, height, sea_level, true);
        let _ = run_fill(&macro_sampler, &slices, &mut va, &biome, CX, CZ, min_y, height, 0);
    }

    // T0 宏观网格构建（每 chunk 一次，生产 fill_chunk → sample_chunk → build_slices）
    let t0 = Instant::now();
    for r in 0..ROUNDS {
        for i in 0..NCHUNK {
            let _ = macro_sampler.build_slices_for(CX + i, CZ + ((r % 2) as i32));
        }
    }
    let t_grid = t0.elapsed().as_secs_f64() / (ROUNDS as f64) * 1e3 / NCHUNK as f64;

    // T1 插值循环（无 aquifer）
    let mut t_interp_only = 0.0f64;
    // T2 插值 + skip classify（生产 no-aquifer 配置语义）
    let mut t_skip = 0.0f64;
    // T3 插值 + 真实 classify（每 chunk 新建 Aquifer = 生产冷缓存）
    let mut t_prod = 0.0f64;
    // T4 同 T3 但 Aquifer 复用（暖缓存对照）
    let mut t_warm = 0.0f64;
    // T5 d<=0 直调 aq.apply（隔离 classify 包装；每 chunk 新建 Aquifer）
    let mut t_apply_direct = 0.0f64;
    let mut applied_prod = 0usize;
    let mut dle0_count = 0usize;

    for r in 0..ROUNDS {
        let cz_off = (r % 2) as i32; // 两批 chunk 行，防同 chunk 重复
        // slices 每批构建一次（每 chunk 各自 build，这里逐 chunk）
        for i in 0..NCHUNK {
            let cx = CX + i; let cz = CZ + cz_off;
            let slices = macro_sampler.build_slices_for(cx, cz);

            // T1
            let t = Instant::now();
            for lz in 0..16i32 { for lx in 0..16i32 {
                let x = cx*16+lx; let z = cz*16+lz;
                for ly in (0..height).rev() {
                    let y = min_y + ly;
                    let d = macro_sampler.sample_interp(&slices, &NoisePos { x, y, z });
                    std::hint::black_box(d);
                }
            }}
            t_interp_only += t.elapsed().as_secs_f64() * 1e3;

            // T2（skip_aquifer）
            let mut va = VanillaAquifer { aq: make_aq(&barrier, &flooded, &spread, &lava, &erosion, &depth, &init, &splitter, cx, cz, min_y, height), enabled: true, skip_aquifer: true, sea_level };
            let t = Instant::now();
            run_fill(&macro_sampler, &slices, &mut va, &biome, cx, cz, min_y, height, 0);
            t_skip += t.elapsed().as_secs_f64() * 1e3;

            // T3（真实 classify，新建 Aquifer）+ 计数采集（只在 r==0 中段 chunk 统计一次口径）
            let mut va = VanillaAquifer { aq: make_aq(&barrier, &flooded, &spread, &lava, &erosion, &depth, &init, &splitter, cx, cz, min_y, height), enabled: true, skip_aquifer: false, sea_level };
            aquifer_bp_count_reset(); aquifer_wl_count_reset();
            let t = Instant::now();
            let applied = run_fill(&macro_sampler, &slices, &mut va, &biome, cx, cz, min_y, height, 0);
            t_prod += t.elapsed().as_secs_f64() * 1e3;
            if r == 0 {
                applied_prod += applied;
                let bp = aquifer_bp_count_reset(); let wl = aquifer_wl_count_reset();
                bp_total[0] += bp[0]; bp_total[1] += bp[1];
                wl_total[0] += wl[0]; wl_total[1] += wl[1];
            }

            // T4（暖缓存：复用同一 Aquifer —— 第二遍 fill 同 chunk）
            let mut va2 = VanillaAquifer { aq: make_aq(&barrier, &flooded, &spread, &lava, &erosion, &depth, &init, &splitter, cx, cz, min_y, height), enabled: true, skip_aquifer: false, sea_level };
            let _ = run_fill(&macro_sampler, &slices, &mut va2, &biome, cx, cz, min_y, height, 0); // 灌缓存
            let t = Instant::now();
            let _ = run_fill(&macro_sampler, &slices, &mut va2, &biome, cx, cz, min_y, height, 0);
            t_warm += t.elapsed().as_secs_f64() * 1e3;

            // T5（d<=0 直调 apply，新建 Aquifer）
            let mut aq = make_aq(&barrier, &flooded, &spread, &lava, &erosion, &depth, &init, &splitter, cx, cz, min_y, height);
            let t = Instant::now();
            let mut n = 0usize;
            for lz in 0..16i32 { for lx in 0..16i32 {
                let x = cx*16+lx; let z = cz*16+lz;
                for ly in (0..height).rev() {
                    let y = min_y + ly;
                    let d = macro_sampler.sample_interp(&slices, &NoisePos { x, y, z });
                    if d <= 0.0 { let _ = aq.apply(x, y, z, d); n += 1; }
                }
            }}
            t_apply_direct += t.elapsed().as_secs_f64() * 1e3;
            if r == 0 && i == 0 { dle0_count = n; }
        }
    }
    let div = ROUNDS as f64 * NCHUNK as f64;
    t_interp_only /= div; t_skip /= div; t_prod /= div; t_warm /= div; t_apply_direct /= div;

    println!("qaq1_b1_prodfill_probe seed={} region=({},{})+{}chunks rounds={} (per chunk, ms):", seed, CX, CZ, NCHUNK, ROUNDS);
    println!("  T0  macro grid build            : {:8.2}", t_grid);
    println!("  T1  interp loop (no aquifer)    : {:8.2}", t_interp_only);
    println!("  T2  interp + classify(skip)     : {:8.2}   (skip-纯classify开销 {:.2})", t_skip, t_skip - t_interp_only);
    println!("  T3  interp + classify(real,cold): {:8.2}   (aquifer 段={:.2}; 每apply={:.0}ns @applied≈{}/chunk)",
        t_prod, t_prod - t_skip, (t_prod - t_skip) * 1e6 / applied_prod.max(1) as f64, applied_prod / NCHUNK as usize);
    println!("  T4  interp + classify(warm)     : {:8.2}   (每chunk新建成本={:.2})", t_warm, t_prod - t_warm);
    println!("  T5  d<=0 direct aq.apply(cold)  : {:8.2}   (含采样; 首chunk d<=0点={})", t_apply_direct, dle0_count);
    println!("  counters (r0, per chunk): bp calls={} miss={} | wl calls={} miss={}",
        bp_total[0] / NCHUNK as usize, bp_total[1] / NCHUNK as usize,
        wl_total[0] / NCHUNK as usize, wl_total[1] / NCHUNK as usize);
    println!("[判读] T3-T2 ≈ 35ms → b1 成立（fill 循环内部复现）；T3-T2 ≪ 35ms → 差异在 fill 循环外（b2/b3）");
}

fn make_aq(
    barrier: &Arc<DensityFunction>, flooded: &Arc<DensityFunction>, spread: &Arc<DensityFunction>,
    lava: &Arc<DensityFunction>, erosion: &Arc<DensityFunction>, depth: &Arc<DensityFunction>,
    init: &Arc<DensityFunction>, splitter: &WorldgenRust::xoroshiro::XoroshiroSplitter,
    cx: i32, cz: i32, min_y: i32, height: i32,
) -> Aquifer {
    Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(),
        erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx * 16, cz * 16, min_y, height)
}

fn make_va(
    barrier: &Arc<DensityFunction>, flooded: &Arc<DensityFunction>, spread: &Arc<DensityFunction>,
    lava: &Arc<DensityFunction>, erosion: &Arc<DensityFunction>, depth: &Arc<DensityFunction>,
    init: &Arc<DensityFunction>, splitter: &WorldgenRust::xoroshiro::XoroshiroSplitter,
    cx: i32, cz: i32, min_y: i32, height: i32, sea_level: i32, skip: bool,
) -> VanillaAquifer {
    VanillaAquifer { aq: make_aq(barrier, flooded, spread, lava, erosion, depth, init, splitter, cx, cz, min_y, height), enabled: true, skip_aquifer: skip, sea_level }
}

// terrain.rs fill_chunk L265-287 逐行镜像（biome stub、无 beard/gpu 分支），返回 d<=0 classify 次数
fn run_fill(
    macro_sampler: &DensityMacroSampler, slices: &[f64], aqua: &mut VanillaAquifer,
    _biome: &StubBiome, cx: i32, cz: i32, min_y: i32, height: i32, _noise_height: i32,
) -> usize {
    let mut applied = 0usize;
    let mut top_all = i32::MIN;
    for lz in 0..16i32 {
        for lx in 0..16i32 {
            let x = cx*16+lx; let z = cz*16+lz;
            let mut top = i32::MIN;
            for ly in (0..height).rev() {
                let y = min_y + ly;
                let d = macro_sampler.sample_interp(slices, &NoisePos { x, y, z });
                // classify（VanillaAquifer::AquiferSource，生产 terrain.rs L277 同路径）
                let kind = aqua.classify(x, y, z, d);
                if !aqua.skip_aquifer && d <= 0.0 { applied += 1; }
                std::hint::black_box(kind);
                if top == i32::MIN && d > 0.0 { top = y; }
            }
            top_all = top_all.max(top);
            std::hint::black_box(top);
        }
    }
    std::hint::black_box(top_all);
    applied
}
