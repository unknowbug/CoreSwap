// res12_probe.rs — 残 9 判别探针 A（scout §5.1，260904-12；克隆 res76_probe，overworld）
// 对 9 个 G2S 点 dump：pick_cell（hashed zoom）/ zoomed biome / cell-direct biome /
// pre / surface applied / final（全管线 post）。
// 判读（scout §5.1）：
//   applied=gravel & final=sand → H3/H4（feature 覆写）
//   applied=sand & zoom_biome∈{warm,lukewarm,deep_lukewarm} → Rust 诚实判 sand，
//     与 vanilla gravel 差 = biome 判定差 → H1（pick 差）/H2（同 cell 分类差），需 C++ 臂对比
// 用法（主会话执行）：rustc 单编（bin-diag 不入 cargo 默认构建）
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use WorldgenRust::biome::{biome_hash_seed, biome_pick_cell, BiomeClassifier};
use WorldgenRust::blocks::{BlockId, BlockRegistry};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::legacy_random::RsSplitter;
use WorldgenRust::noise::DoublePerlinNoiseSampler;
use WorldgenRust::surface_rules::{SurfaceBuilder, SurfaceContext, SurfaceRule, ENGINE_NOISE_KEYS};
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
const POINTS_PATH: &str = "E:/PYTHON/CoreSwap/.tmp/p2full/res12-points-clean.txt";
const SETTINGS: &str = "overworld.json";
const BIOME_PARAMS: &str = "biome_params.json";
const WORLD_HEIGHT: i32 = 384;

fn main() {
    unsafe {
        for k in ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES", "WG_TRANSPILER", "WG_DFC", "WG_GPU_CHANNELS", "WG_GPU_DENSITY"] {
            std::env::remove_var(k);
        }
    }

    let points_txt = match std::fs::read_to_string(POINTS_PATH) {
        Ok(t) => t,
        Err(e) => { eprintln!("[FAIL] cannot read {}: {}", POINTS_PATH, e); return; }
    };
    let mut points: Vec<(i32, i32, i32, String)> = Vec::new();
    for line in points_txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut it = line.split_whitespace();
        let (x, y, z) = match (it.next(), it.next(), it.next()) {
            (Some(a), Some(b), Some(c)) => match (a.parse(), b.parse(), c.parse()) {
                (Ok(x), Ok(y), Ok(z)) => (x, y, z),
                _ => continue,
            },
            _ => continue,
        };
        let label = it.next().unwrap_or("").to_string();
        points.push((x, y, z, label));
    }
    if points.is_empty() { eprintln!("[FAIL] no points parsed"); return; }

    let h = match WorldgenHandle::create(SEED, WG_DIR) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create failed"); return; }
    };

    let mut pre: HashMap<(i32, i32), (Vec<BlockId>, Vec<i32>)> = HashMap::new();
    let mut post: HashMap<(i32, i32), Vec<BlockId>> = HashMap::new();
    let mut need: HashSet<(i32, i32)> = HashSet::new();
    for (x, _y, z, _) in &points { need.insert((x >> 4, z >> 4)); }
    let mut need: Vec<(i32, i32)> = need.into_iter().collect();
    need.sort();
    for key in &need {
        pre.insert(*key, h.diag_pre_surface_column(key.0, key.1));
        post.insert(*key, h.fill_chunk_blocks(key.0, key.1));
        eprintln!("[RES12] chunk ({},{}) done", key.0, key.1);
    }

    let (min_y, noise_height, sea_level, legacy_random) = match read_dim_params() {
        Some(v) => v,
        None => { eprintln!("[FAIL] cannot read settings"); return; }
    };
    let mut db = DensityBuilder::new(SEED as u64, min_y, noise_height);
    db.set_df_ns("overworld");
    if legacy_random { db.set_legacy_random(); }
    let noise_params_path = format!("{}/../noise_params.json", WG_DIR);
    if db.load_noise_params_file(&noise_params_path).is_err() {
        eprintln!("[FAIL] cannot load noise_params.json"); return;
    }
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", WG_DIR);
    let df_dir2 = df_dir.clone();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = format!("{}/{}.json", df_dir2, name);
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("[LOADFAIL] {}: {}", p, e))
    }));
    let settings_txt = std::fs::read_to_string(settings_path()).expect("settings read");
    let settings = parse(&settings_txt).expect("settings parse");
    let router = settings.get("noise_router").expect("noise_router");
    let init = db.build_node(router.get("initial_density_without_jaggedness").expect("init df")).expect("build init");
    let tempf = db.build_node(router.get("temperature").expect("temperature")).expect("build temp");
    let humf = db.build_node(router.get("vegetation").expect("vegetation")).expect("build hum");
    let contf = db.build_node(router.get("continents").expect("continents")).expect("build cont");
    let erof = db.build_node(router.get("erosion").expect("erosion")).expect("build ero");
    let depthf = db.build_node(router.get("depth").expect("depth")).expect("build depth");
    let weirdf = db.build_node(router.get("ridges").expect("ridges")).expect("build weird");

    let biome_params_path = format!("{}/../{}", WG_DIR, BIOME_PARAMS);
    let bc = BiomeClassifier::load(&biome_params_path);

    let mut samplers: HashMap<String, Arc<DoublePerlinNoiseSampler>> = HashMap::new();
    for k in ["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset"] {
        samplers.insert(k.to_string(), db.get_noise_sampler(k));
    }
    for k in ENGINE_NOISE_KEYS.iter() {
        samplers.entry(k.to_string()).or_insert_with(|| db.get_noise_sampler(k));
    }
    let splitter: RsSplitter = db.random_deriver().clone();

    let blocks_path = format!("{}/../blocks.json", WG_DIR);
    let blocks_json = std::fs::read_to_string(&blocks_path).expect("blocks.json read");
    let blocks = BlockRegistry::load_from_json(&blocks_json).expect("blocks parse");
    let sb = SurfaceBuilder::new(&samplers, &splitter, sea_level, &blocks);
    let rule: SurfaceRule = sb.build_overworld_rule();

    let hseed = biome_hash_seed(SEED);
    println!("# seed={} hashSeed={} points={}", SEED, hseed, points.len());

    let top = min_y + WORLD_HEIGHT - 1;
    for (x, y, z, label) in &points {
        let (cx, cz) = (*x >> 4, *z >> 4);
        let (col, hmap) = match pre.get(&(cx, cz)) {
            Some(v) => v,
            None => { eprintln!("[FAIL] missing pre chunk"); continue; }
        };
        let pc = match post.get(&(cx, cz)) {
            Some(v) => v,
            None => { eprintln!("[FAIL] missing post chunk"); continue; }
        };
        let lx = (*x & 15) as usize;
        let lz = (*z & 15) as usize;
        let col_idx = lz * 16 + lx;
        let air_id = blocks.id("minecraft:air");
        let water_id = blocks.id("minecraft:water");
        let lava_id = blocks.id("minecraft:lava");
        let pre_at = |wy: i32| -> BlockId {
            if wy < min_y || wy > top { air_id } else { col[((wy - min_y) * 256) as usize + col_idx] }
        };
        let post_name = {
            let wy = *y;
            if wy < min_y || wy > top { "air".to_string() } else {
                blocks.name(pc[((wy - min_y) * 256) as usize + col_idx]).trim_start_matches("minecraft:").to_string()
            }
        };

        // zoomed pick（对齐 biome_at_surface：hashed seed + pick_cell + (px<<2) 对齐）
        let (px, py, pz) = biome_pick_cell(hseed, *x, *y, *z);
        let bp_zoom = NoisePos { x: px << 2, y: py << 2, z: pz << 2 };
        let biome_zoom = bc.biome_of(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &bp_zoom);
        let (dbg_name, dbg_dist, dbg6) = bc.biome_of_debug(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &bp_zoom);
        let dbg6_s = dbg6.iter().map(|v| format!("{v:.6}")).collect::<Vec<_>>().join(",");
        // cell-direct（旧口径对照）
        let bp_dir = NoisePos { x: (*x >> 2) << 2, y: (*y >> 2) << 2, z: (*z >> 2) << 2 };
        let biome_direct = bc.biome_of(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &bp_dir);

        let sd = samplers.get("minecraft:surface").expect("surface sampler");
        let d = sd.sample(*x as f64, 0.0, *z as f64);
        let extra = splitter.split_xyz(*x, 0, *z).next_double();
        let surface_depth = (d * 2.75 + 3.0 + extra * 0.25) as i32;

        let mut q = 0i32; let mut r = i32::MIN; let mut s = i32::MAX;
        let mut sda = 0i32; let mut sdb = 0i32;
        let mut wy = (hmap[col_idx] + 1).min(top);
        while wy >= min_y {
            let state = pre_at(wy);
            let is_air = state == air_id;
            let is_fluid = state == water_id || state == lava_id;
            if is_air { q = 0; r = i32::MIN; }
            else if is_fluid { if r == i32::MIN { r = wy + 1; } }
            else {
                if s >= wy {
                    s = i32::MAX;
                    let mut v = wy - 1;
                    while v >= min_y - 1 {
                        let st2 = pre_at(v);
                        if st2 != air_id && st2 != water_id && st2 != lava_id { v -= 1; continue; }
                        s = v + 1; break;
                    }
                }
                q += 1;
                let vx = wy - s + 1;
                if wy == *y { sda = q; sdb = vx; }
            }
            wy -= 1;
        }

        let est_at = |ex: i32, ez: i32| -> i32 {
            let mut est = i32::MAX;
            let mut ey = min_y + noise_height;
            while ey >= min_y {
                if init.sample(&NoisePos { x: ex, y: ey, z: ez }) > 0.390625 { est = ey; break; }
                ey -= 8;
            }
            est
        };
        let sh4 = [
            est_at(cx * 16, cz * 16), est_at(cx * 16 + 15, cz * 16),
            est_at(cx * 16, cz * 16 + 15), est_at(cx * 16 + 15, cz * 16 + 15),
        ];
        let init_at = |ix: i32, iy: i32, iz: i32| -> f64 { init.sample(&NoisePos { x: ix, y: iy, z: iz }) };
        let mut ctx = SurfaceContext::new(&samplers, &splitter, min_y, WORLD_HEIGHT);
        ctx.initial_density_at = Some(&init_at);
        ctx.surface_secondary_noise = samplers.get("minecraft:surface_secondary").map(|a| a.as_ref());
        ctx.surface_heights4 = Some(&sh4);
        ctx.surface_depth = surface_depth;
        ctx.init_vertical(sda, sdb, r, *x, *y, *z, &biome_zoom);
        let applied = rule.apply(&ctx);
        let applied_s = applied.map(|b| blocks.name(b).trim_start_matches("minecraft:")).unwrap_or("none");
        let pre_name = blocks.name(pre_at(*y)).trim_start_matches("minecraft:");

        println!("{},{},{},{},pre={},zoom_pick=({},{})>,zoom_biome={},direct_biome={},r={},sda={},sdb={},applied={},final={},dbg_best={},dbg_dist={:.9},dbg6=[{}]",
            x, y, z, label, pre_name, px, pz, biome_zoom, biome_direct, r, sda, sdb, applied_s, post_name, dbg_name, dbg_dist, dbg6_s);
    }
}

fn settings_path() -> String {
    format!("{}/data/minecraft/worldgen/noise_settings/{}", WG_DIR, SETTINGS)
}

fn read_dim_params() -> Option<(i32, i32, i32, bool)> {
    let txt = std::fs::read_to_string(settings_path()).ok()?;
    let settings = parse(&txt).ok()?;
    let mut min_y = -64;
    let mut noise_height = 384;
    if let Some(noise) = settings.get("noise") {
        if let Some(m) = noise.get("min_y") { min_y = m.as_f64().unwrap_or(-64.0) as i32; }
        if let Some(h) = noise.get("height") { noise_height = h.as_f64().unwrap_or(384.0) as i32; }
    }
    let sea_level = settings.get("sea_level").and_then(|s| s.as_f64()).unwrap_or(63.0) as i32;
    let legacy = settings.get("legacy_random_source").and_then(|l| l.as_bool()).unwrap_or(false);
    Some((min_y, noise_height, sea_level, legacy))
}
