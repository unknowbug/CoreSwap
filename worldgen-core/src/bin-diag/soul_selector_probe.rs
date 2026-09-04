// soul_selector_probe.rs — B1 诊断（bin-diag 隔离区）：对给定采样点 dump nether surface rule
// soul 分支（soul_sand_valley biome + stone_depth ceiling/floor 条件 + nether_state_selector
// noise_threshold min=0.0 → soul_sand / 否则 soul_soil）的内部判定输入。
//
// 用途：裁决 .b1a（前置条件差：biome/stone_depth 不一致 → 分支没进）vs .b1b（阈值判定差：
// 进了分支但 selector 噪声/阈值边界偏）。
//
// 用法（主会话执行；bin-diag 不参与默认构建）：
//   cargo build --release --bin soul_selector_probe
//   cargo run --release --bin soul_selector_probe > E:\PYTHON\CoreSwap\.tmp\soul-selector-probe.csv
//
// 输入：E:\PYTHON\CoreSwap\.tmp\soul-mismatch-points.txt（每行 `x y z 标签`）
// 输出（stdout，每点一行）：
//   x,y,z,label,biome=<id>,biome_temp=<f64>,stone_depth_above=<i32>,stone_depth_below=<i32>,
//   surface_depth=<i32>,selector=<f64>,soul_branch_entered=<bool>,selector_pass=<bool>
//
// ⚠️ 未编译验证：本文件由 worker 产出（无 shell 沙箱），主会话负责 cargo 编译验证。
// 静态自检清单见文件末尾注释。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::blocks::{BlockId, BlockRegistry};
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::legacy_random::RsSplitter;
use WorldgenRust::noise::DoublePerlinNoiseSampler;
use WorldgenRust::surface_rules::{SurfaceBuilder, SurfaceContext, SurfaceRule, biome_temperature};
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
const POINTS_PATH: &str = "E:/PYTHON/CoreSwap/.tmp/soul-mismatch-points.txt";
const SETTINGS: &str = "nether.json";
const BIOME_PARAMS: &str = "biome_params_nether.json";
const WORLD_HEIGHT: i32 = 256;
const SOUL_BIOME: &str = "minecraft:soul_sand_valley";
const SELECTOR_KEY: &str = "minecraft:nether_state_selector";

fn main() {
    // 确定性：诊断 bin 不允许受 WG_SKIP_*/WG_TRANSPILER 开关影响（与 b1_blackstone_source 同纪律）
    unsafe {
        for k in ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES", "WG_TRANSPILER"] {
            std::env::remove_var(k);
        }
    }

    // 0. 读采样点（x y z 标签）
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
    if points.is_empty() { eprintln!("[FAIL] no points parsed from {}", POINTS_PATH); return; }

    // 1. 生产句柄：与 .tmp/b1_ctypes_wg.py 的 ctypes wg_create 调用同参
    //    （wg_create(seed=8576294172403134396, dir=..., settings="nether.json",
    //                biomeParams="biome_params_nether.json", height=256)）
    let h = match WorldgenHandle::create_for_dim(SEED, WG_DIR, SETTINGS, BIOME_PARAMS, WORLD_HEIGHT) {
        Some(h) => h,
        None => { eprintln!("[FAIL] create_for_dim failed"); return; }
    };

    // 2. surface 前列状态（diag 增量 API：fill_chunk 宏观+aquifer+ore_vein，surface/carver 之前）
    //    stone_depth_above/below 只能由 surface 前列扫描得出（fill_chunk_blocks 是 surface 后结果）
    let mut cols: HashMap<(i32, i32), (Vec<BlockId>, Vec<i32>)> = HashMap::new();
    let mut need: HashSet<(i32, i32)> = HashSet::new();
    for (x, _y, z, _) in &points { need.insert((x >> 4, z >> 4)); }
    let mut need: Vec<(i32, i32)> = need.into_iter().collect();
    need.sort();
    for key in &need {
        cols.insert(*key, h.diag_pre_surface_column(key.0, key.1));
    }

    // 3. 用 lib pub API 重组组件（镜像 worldgen_handle::create_for_dim 的组装流程；seed 相同 → 同采样器）
    let (min_y, noise_height, sea_level, legacy_random) = match read_dim_params() {
        Some(v) => v,
        None => { eprintln!("[FAIL] cannot read {}", settings_path()); return; }
    };
    let df_ns = SETTINGS.strip_suffix(".json").unwrap_or(SETTINGS);
    let mut db = DensityBuilder::new(SEED as u64, min_y, noise_height);
    db.set_df_ns(df_ns);
    if legacy_random { db.set_legacy_random(); }
    let noise_params_path = format!("{}/../noise_params.json", WG_DIR);
    if db.load_noise_params_file(&noise_params_path).is_err() {
        eprintln!("[FAIL] cannot load {}", noise_params_path);
        return;
    }
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/{}", WG_DIR, df_ns);
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

    // biome classifier（与 create_for_dim 同参数文件）
    let biome_params_path = format!("{}/../{}", WG_DIR, BIOME_PARAMS);
    let bc = BiomeClassifier::load(&biome_params_path);

    // noise samplers：基础 3 key + settings.surface_rule 动态收集（对齐 create_for_dim 步骤 4）
    let mut samplers: HashMap<String, Arc<DoublePerlinNoiseSampler>> = HashMap::new();
    for k in ["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset", SELECTOR_KEY] {
        samplers.insert(k.to_string(), db.get_noise_sampler(k));
    }
    if let Some(sr) = settings.get("surface_rule") {
        let mut dyn_keys: Vec<String> = Vec::new();
        WorldgenRust::surface_rules::collect_noise_keys(sr, &mut dyn_keys);
        for k in &dyn_keys {
            samplers.entry(k.clone()).or_insert_with(|| db.get_noise_sampler(k));
        }
    }
    let splitter: RsSplitter = db.random_deriver().clone();

    // blocks + SurfaceBuilder + nether surface rule（数据驱动解析，与 create_for_dim 同路径）
    let blocks_path = format!("{}/../blocks.json", WG_DIR);
    let blocks_json = std::fs::read_to_string(&blocks_path).expect("blocks.json read");
    let blocks = BlockRegistry::load_from_json(&blocks_json).expect("blocks parse");
    let sb = SurfaceBuilder::new(&samplers, &splitter, sea_level, &blocks);
    let rule: SurfaceRule = match settings.get("surface_rule") {
        Some(sr) => sb.parse_surface_rule(sr, min_y, noise_height).expect("nether surface_rule parse"),
        None => { eprintln!("[FAIL] settings has no surface_rule"); return; }
    };

    let soul_sand_id = blocks.id("minecraft:soul_sand");
    let soul_soil_id = blocks.id("minecraft:soul_soil");
    let netherrack_id = blocks.id("minecraft:netherrack");
    let air_id = blocks.id("minecraft:air");
    let stone_id = blocks.id("minecraft:stone");
    let water_id = blocks.id("minecraft:water");
    let lava_id = blocks.id("minecraft:lava");

    // 头部（# 注释行，不影响 CSV 解析）
    println!("# seed={} settings={} min_y={} noise_height={} world_height={} points={}",
        SEED, SETTINGS, min_y, noise_height, WORLD_HEIGHT, points.len());
    println!("# block ids: soul_sand={} soul_soil={} netherrack={} (stderr 附带整规则 apply 结果)",
        soul_sand_id, soul_soil_id, netherrack_id);

    // 4. 逐点判定
    for (x, y, z, label) in &points {
        let (cx, cz) = (*x >> 4, *z >> 4);
        let (col, hmap) = match cols.get(&(cx, cz)) {
            Some(v) => v,
            None => { eprintln!("[FAIL] missing chunk ({},{})", cx, cz); continue; }
        };
        let lx = x & 15;
        let lz = z & 15;
        let col_idx = (lz * 16 + lx) as usize;
        let top = min_y + WORLD_HEIGHT - 1;
        let at = |wy: i32| -> BlockId {
            if wy < min_y || wy > top { air_id } else {
                col[((wy - min_y) * 256) as usize + col_idx]
            }
        };
        // biome：对齐 build_surface 的 biome_at —— NoisePos 按 4 的倍数对齐（(c>>2)<<2）
        let bp = NoisePos { x: (*x >> 2) << 2, y: (*y >> 2) << 2, z: (*z >> 2) << 2 };
        let biome_id = bc.biome_of(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &bp);
        let biome_temp = biome_temperature(&biome_id);

        // surface_depth（runDepth）：逐行对拍 SurfaceBuilder::sample_run_depth（surface_rules.rs L480-495）
        let sd = samplers.get("minecraft:surface").expect("surface sampler");
        let d = sd.sample(*x as f64, 0.0, *z as f64);
        let extra = splitter.split_xyz(*x, 0, *z).next_double();
        let surface_depth = (d * 2.75 + 3.0 + extra * 0.25) as i32;

        // 列扫描（逐行对拍 SurfaceBuilder::build_surface L1180-1230 的 q/r/s 循环；
        // 起点 = heightmap+1（WORLD_SURFACE_WG），nether 无 eroded_badlands pillar，跳过 pillar 段）
        let mut q = 0i32;
        let mut r = i32::MIN;
        let mut s = i32::MAX;
        let mut sda = 0i32;
        let mut sdb = 0i32;
        let mut is_default = false;
        let mut wy = (hmap[col_idx] + 1).min(top);
        while wy >= min_y {
            let state = at(wy);
            let is_air = state == air_id;
            let is_fluid = state == water_id || state == lava_id;
            if is_air {
                q = 0;
                r = i32::MIN;
            } else if is_fluid {
                if r == i32::MIN { r = wy + 1; }
            } else {
                if s >= wy {
                    s = i32::MAX;
                    let mut v = wy - 1;
                    while v >= min_y - 1 {
                        let st2 = at(v);
                        if st2 != air_id && st2 != water_id && st2 != lava_id { v -= 1; continue; }
                        s = v + 1;
                        break;
                    }
                }
                q += 1;
                let vx = wy - s + 1;
                if wy == *y {
                    sda = q;
                    sdb = vx;
                    is_default = state == stone_id;
                }
            }
            wy -= 1;
        }

        // soul 分支入口判定（逐行对拍 SurfaceCond::StoneDepth，surface_rules.rs L84-93：
        //   ceiling: i=stone_depth_below, j=surface_depth(add_surface_depth=true), k=0(secondary_depth_range=0)
        //            → i <= 1 + offset(0) + j + k；floor 同构用 stone_depth_above）
        let ceiling_ok = sdb <= 1 + 0 + surface_depth + 0;
        let floor_ok = sda <= 1 + 0 + surface_depth + 0;
        let soul_branch_entered = biome_id == SOUL_BIOME && (ceiling_ok || floor_ok);

        // selector：nether_state_selector 在 (x,0,z) 采样（对齐 noise_threshold_sample 的
        // sample(x, 0.0, z)，surface_rules.rs L132）；阈值 min=0.0（nether.json L306）
        let selector = samplers.get(SELECTOR_KEY)
            .map(|n| n.sample(*x as f64, 0.0, *z as f64))
            .unwrap_or(f64::NAN);
        let selector_pass = selector >= 0.0;

        // stderr 交叉验证：完整 SurfaceContext + 整条 nether rule apply 的结果块
        // （仅诊断输出，不进 stdout CSV；诊断一次性构造，非热路径）
        {
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
            ctx.init_vertical(sda, sdb, r, *x, *y, *z, &biome_id);
            ctx.biome_temp = biome_temp;
            let applied = rule.apply(&ctx);
            let applied_s = match applied {
                Some(b) if b == soul_sand_id => "soul_sand".to_string(),
                Some(b) if b == soul_soil_id => "soul_soil".to_string(),
                Some(b) if b == netherrack_id => "netherrack".to_string(),
                Some(b) => format!("id={}", b),
                None => "none".to_string(),
            };
            eprintln!("# point {},{},{} rule_applied={} (is_default_stone={}) ceiling_ok={} floor_ok={}",
                x, y, z, applied_s, is_default, ceiling_ok, floor_ok);
        }

        // stdout CSV 行（规定格式）
        println!("{},{},{},{},biome={},biome_temp={},stone_depth_above={},stone_depth_below={},surface_depth={},selector={},soul_branch_entered={},selector_pass={}",
            x, y, z, label, biome_id, biome_temp, sda, sdb, surface_depth, selector, soul_branch_entered, selector_pass);
    }
}

fn settings_path() -> String {
    format!("{}/data/minecraft/worldgen/noise_settings/{}", WG_DIR, SETTINGS)
}

/// 读维度参数（镜像 create_for_dim：noise.min_y / noise.height / sea_level / legacy_random_source）
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

// ========== 静态自检清单（未编译验证声明）==========
//
// ⚠️ 未编译验证：本文件由 worker 在无 shell 沙箱下产出，未运行 cargo check/build，
//    主会话负责编译（cargo build --release --bin soul_selector_probe）与运行。
//
// ① 类型宽度（显式标注）：
//    - SEED: i64（const 显式）；坐标/深度/高度全部 i32；selector/biome_temp/surface_depth 噪声值 f64。
//    - `SEED as u64` 显式转 DensityBuilder::new（对齐 worldgen_handle.rs L133）。
//    - `(d * 2.75 + 3.0 + extra * 0.25) as i32`：向零截断（对齐 sample_run_depth L491 的 `as i32`）。
//    - col_idx/hmap 索引经 `as usize`；(wy - min_y) * 256 在 nether 域（y∈[0,256)）无 i32 溢出。
//    - i32::MIN/MAX 哨兵（r/s）与 build_surface 同值。
//
// ② panic 路径（unwrap/expect 使用点清单）：
//    - read_to_string(POINTS_PATH/settings/blocks.json)：expect —— 输入文件缺失即 fail-fast（主会话环境保证存在）。
//    - parse(settings_txt).expect：JSON 损坏即 fail-fast。
//    - router.get(...).expect + build_node(...).expect：nether.json router 缺 key 即 fail-fast。
//    - samplers.get(...).expect（"minecraft:surface"）：base key 由本文件显式插入，不可能缺。
//    - cols.get(&chunk) 缺失：continue 并 eprintln（不 panic，单点失败不阻断整批）。
//    - 外部 loader 内 panic![LOADFAIL]：镜像 create_for_dim L145 同语义。
//    - 无 unwrap_or 死循环路径；无 unsafe 除 env::remove_var（edition 2024 要求 unsafe，与
//      b1_blackstone_source.rs L14 同模式）。
//
// ③ 与 surface_rules.rs / worldgen_handle.rs 判定逻辑逐行对拍点：
//    - 组件组装 ↔ create_for_dim L103-247：DensityBuilder(seed,min_y,noise_height)/set_df_ns/
//      set_legacy_random(nether legacy_random_source=true)/noise_params 路径/external loader/
//      router 七 DF + 6 biome DF/预加载 key（base 3 + collect_noise_keys 动态）/BiomeClassifier::load/
//      SurfaceBuilder::new(samplers,splitter,sea_level,blocks)/parse_surface_rule(sr,min_y,noise_height)。
//      注意：noise_height（128）而非 world_height（256）传给 parse_surface_rule，与 L244 一致。
//    - biome_at ↔ fill_chunk_blocks L426-429：NoisePos { (x>>2)<<2, (y>>2)<<2, (z>>2)<<2 }。
//    - surface_depth ↔ sample_run_depth L489-491：surface 噪声 sample(x,0,z)*2.75+3.0
//      + splitter.split_xyz(x,0,z).next_double()*0.25，`as i32` 截断。
//    - 列扫描 q/r/s ↔ build_surface L1180-1230：air→q=0,r=MIN；fluid→r=wy+1(首次)；
//      solid→ s>=wy 时内层向下扫到首个非 default 定 s，q+=1，vx=wy-s+1；
//      起点 = heightmap[idx]+1（L1165，WORLD_SURFACE_WG+1）；nether 无 eroded_badlands → 跳过 pillar。
//    - soul 分支入口 ↔ SurfaceCond::StoneDepth L84-93：i = ceiling? below : above；
//      j = add_surface_depth(true)? surface_depth : 0；k = 0（secondary_depth_range=0）；
//      命中 = i <= 1 + offset(0) + j + k。ceiling 分支（nether.json L290-297）与
//      floor 分支（L326-333）均 add_surface_depth=true。
//    - selector ↔ NoiseThresholdCond L95-98 + noise_threshold_sample L132：sample(x, 0.0, z)；
//      pass = d >= min_th(0.0)（max_th=f64::MAX 恒真，nether.json L305-307）。
//      注：探针直采 sampler（无 thread_local 缓存），与 noise_threshold_sample 纯函数同值（缓存语义等价）。
//    - diag_pre_surface_column（worldgen_handle.rs 增量 fn）↔ fill_chunk_blocks L369-411 步骤 1-2 逐行同源：
//      Aquifer::new 参数/VanillaAquifer/beardifier 读取/fill_chunk 调用/BlockKind 映射/ore_vein 替换全一致；
//      仅截断在 surface/carver/features 之前，另返回 cd.surface_height。
//    - biome_temperature：nether biome 不在温度表 → 默认 0.5（表内默认分支，对齐 C++ biomeTemp 用法）。
//
// ④ lib 改动声明（最小增量）：
//    - worldgen_handle.rs 新增 pub fn diag_pre_surface_column（增量 API，不改任何现有行为/函数体）；
//    - lib.rs / surface_rules.rs 无改动。
//
// ⑤ 已知边界（@anchor.idk 精神——诚实声明）：
//    - 列扫描起点取 heightmap+1：若 macro 层在 WORLD_SURFACE_WG 之上有非空块（理论不该有），
//      stone_depth_above 会与 build_surface 一致地忽略之（对齐原实现，非偏差）。
//    - selector 直采不带 thread_local 缓存；多线程缓存竞态在生产路径存在但单线程诊断无影响。
//    - rule_applied 交叉验证输出经完整 ctx（含 sh4/init/secondary），但 ctx.column_heightmap=None
//      （nether 规则无 steep/surface cond，不触达该字段——已核对 nether.json 无 steep/"minecraft:surface" 条件）。
