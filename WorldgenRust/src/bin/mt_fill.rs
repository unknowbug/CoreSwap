// mt_fill.rs — 多线程 fill_chunk 完整管线（含 biome SearchTree）并发扩展 + 冷缓存争用。
// judge 建议：Rust 多线程/冷缓存并发争用测（C++ 11x 课题核心）。
// 测：N 线程各 fill 不同 chunk（完整管线），验证结果一致 + 墙钟扩展 + 冷缓存（每线程独立 thread_local）。
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::terrain::{fill_chunk, VanillaDensity, VanillaAquifer, BiomeSource};
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

struct MacroBiome { bc: BiomeClassifier, tempf: Arc<DensityFunction>, humf: Arc<DensityFunction>, contf: Arc<DensityFunction>, erof: Arc<DensityFunction>, depthf: Arc<DensityFunction>, weirdf: Arc<DensityFunction> }
impl BiomeSource for MacroBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}

struct Ctx {
    tree: Arc<DensityFunction>,
    barrier: Arc<DensityFunction>, flooded: Arc<DensityFunction>, spread: Arc<DensityFunction>,
    lava: Arc<DensityFunction>, erosion: Arc<DensityFunction>, depth: Arc<DensityFunction>, init: Arc<DensityFunction>,
    splitter: WorldgenRust::xoroshiro::XoroshiroSplitter,
    biomesrc: MacroBiome,
}

fn build_ctx(seed: u64) -> Ctx {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let mut db = DensityBuilder::new(seed, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).unwrap());
    let barrier: Arc<DensityFunction> = Arc::new(db.build_node(router.get("barrier").unwrap()).unwrap());
    let flooded: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_floodedness").unwrap()).unwrap());
    let spread: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_spread").unwrap()).unwrap());
    let lava: Arc<DensityFunction> = Arc::new(db.build_node(router.get("lava").unwrap()).unwrap());
    let erosion: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let depth: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let init: Arc<DensityFunction> = Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).unwrap());
    let t_temp = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let t_hum = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let t_cont = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let t_ero = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let t_dep = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let t_wei = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let biomesrc = MacroBiome { bc, tempf: t_temp, humf: t_hum, contf: t_cont, erof: t_ero, depthf: t_dep, weirdf: t_wei };
    Ctx { tree, barrier, flooded, spread, lava, erosion, depth, init, splitter, biomesrc }
}

fn fill_one(ctx: &Ctx, cx: i32, cz: i32) -> i64 {
    let dense = VanillaDensity { df: &ctx.tree };
    let mut aq = Aquifer::new(ctx.barrier.clone(), ctx.flooded.clone(), ctx.spread.clone(), ctx.lava.clone(), ctx.erosion.clone(), ctx.depth.clone(), ctx.init.clone(), ctx.splitter.clone(), cx*16, cz*16, -64, 384i32);
    let mut va = VanillaAquifer { aq };
    let cd = fill_chunk(&dense, &mut va, &ctx.biomesrc, cx, cz, -64, 384i32, None);
    // 返回块分类计数（代表 chunk 特征）
    let mut rock = 0i64;
    for b in &cd.blocks { if *b == WorldgenRust::terrain::BlockKind::Rock { rock += 1; } }
    rock
}

fn main() {
    let seed = 8576294172403134396u64;
    let ctx = Arc::new(build_ctx(seed));
    let chunks: Vec<(i32,i32)> = (0..16).map(|i| (40 + i, -30 + i)).collect();

    // 单线程参照（预热）
    let seq: Vec<i64> = chunks.iter().map(|&(c,d)| fill_one(&ctx, c, d)).collect();
    println!("seq rock[0]={} (reference)", seq[0]);

    for &t in &[1usize, 2, 4, 8] {
        let n = chunks.len();
        let t0 = Instant::now();
        let handles: Vec<_> = (0..t).map(|ti| {
            let ctx = ctx.clone();
            let chunks = chunks.clone();
            thread::spawn(move || {
                let mut local = Vec::new();
                let start = ti * n / t;
                let end = (ti + 1) * n / t;
                for i in start..end { let (c,d) = chunks[i]; local.push((i, fill_one(&ctx, c, d))); }
                local
            })
        }).collect();
        let mut got = vec![0i64; n];
        for h in handles {
            for (i, v) in h.join().unwrap() { got[i] = v; }
        }
        let wall = t0.elapsed().as_secs_f64() * 1000.0;
        let mut mism = 0u32;
        for i in 0..n { if got[i] != seq[i] { mism += 1; } }
        println!("T={} threads: wall={:.2}ms  mismatch={}/{}", t, wall, mism, n);
    }
    println!("mt_fill done (full pipeline incl biome SearchTree; check scaling + consistency)");
}
