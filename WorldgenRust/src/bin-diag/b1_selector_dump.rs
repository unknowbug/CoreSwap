// b1_selector_dump.rs — B1 诊断（bin-diag 隔离区）：Java vs Rust 的 nether_state_selector /
// patch 噪声值逐列对拍。读 E:\PYTHON\CoreSwap\.tmp\b1-sel-points.txt（每行 `x z # y=NN:标签;...`），
// 对每列输出 Rust 侧 selector / patch 噪声采样值 + surface_depth，与 Java RouterProbe
// ===NOISEPT=== 输出（-Drouter.noisePoints 同文件）逐列对拍。
//
// 采样语义对齐 surface_rules.rs noise_threshold_sample L120-137：sample(x, 0.0, z)（每列恒定）。
// surface_depth 对齐 sample_run_depth：surface sampler sample(x,0,z)*2.75+3.0
//   + splitter.split_xyz(x,0,z).next_double()*0.25，`as i32` 截断。
//
// 用法（主会话执行；bin-diag 不参与默认构建）：
//   cargo build --release --bin b1_selector_dump
//   cargo run --release --bin b1_selector_dump > E:\PYTHON\CoreSwap\.tmp\b1-selector-rust.csv
//
// 输出（stdout，每列一行）：
//   x,z,selector=<.17e>,patch=<.17e>,surface_depth=<i32>,selector_bits=<u64 hex>
// 头部 #seed 注释行 + 两侧对拍以十进制 17 位有效数字为主（bits 仅备查；
// 注意 Rust {:.17e} 指数格式与 Java %.17g 不逐字符同形，对拍按数值解析，勿直接字符串 diff）。
//
// ⚠️ 未编译验证：本文件由 worker 产出（无 shell 沙箱），主会话负责 cargo 编译验证。
// 静态自检清单见文件末尾注释。

use std::collections::HashMap;
use std::sync::Arc;

use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use WorldgenRust::legacy_random::RsSplitter;
use WorldgenRust::noise::DoublePerlinNoiseSampler;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
const POINTS_PATH: &str = "E:/PYTHON/CoreSwap/.tmp/b1-sel-points.txt";
const SETTINGS: &str = "nether.json";
const BIOME_PARAMS: &str = "biome_params_nether.json";
const WORLD_HEIGHT: i32 = 256;
const SELECTOR_KEY: &str = "minecraft:nether_state_selector";
const PATCH_KEY: &str = "minecraft:patch";
const SURFACE_KEY: &str = "minecraft:surface";

fn main() {
    // 确定性：诊断 bin 不允许受 WG_SKIP_*/WG_TRANSPILER 开关影响（与 soul_selector_probe 同纪律）
    unsafe {
        for k in ["WG_SKIP_AQUIFER", "WG_SKIP_OREVEIN", "WG_SKIP_SURFACE", "WG_SKIP_CARVER", "WG_SKIP_FEATURES", "WG_TRANSPILER"] {
            std::env::remove_var(k);
        }
    }

    // 0. 读采样点（`x z # 注释`；忽略 # 注释行与行内 # 后内容）
    //    ⚠️ 文件首行含非 UTF-8 字节（read 实证 invalid UTF-8）——必须 lossy 解码，
    //    read_to_string 会整体报错 fail-fast。
    let raw = match std::fs::read(POINTS_PATH) {
        Ok(b) => b,
        Err(e) => { eprintln!("[FAIL] cannot read {}: {}", POINTS_PATH, e); return; }
    };
    let points_txt = String::from_utf8_lossy(&raw);
    let mut points: Vec<(i32, i32)> = Vec::new();
    for line in points_txt.lines() {
        let hash = line.find('#');
        let data = match hash { Some(i) => &line[..i], None => line };
        let mut it = data.split_whitespace();
        let (x, z) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => match (a.parse::<i32>(), b.parse::<i32>()) {
                (Ok(x), Ok(z)) => (x, z),
                _ => continue,
            },
            _ => continue,
        };
        points.push((x, z));
    }
    if points.is_empty() { eprintln!("[FAIL] no points parsed from {}", POINTS_PATH); return; }

    // 1. 生产句柄（镜像 create_for_dim 组装流程；seed 相同 → 同采样器）
    let (min_y, noise_height, _sea_level, legacy_random) = match read_dim_params() {
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

    // 2. samplers 预加载：base key（selector / patch / surface）+ settings.surface_rule 动态收集。
    //    patch 不在 nether.json surface_rule 的 noise_threshold 字段里（collect_noise_keys 收不到）
    //    → 显式插入。
    let mut samplers: HashMap<String, Arc<DoublePerlinNoiseSampler>> = HashMap::new();
    for k in [SELECTOR_KEY, PATCH_KEY, SURFACE_KEY] {
        samplers.insert(k.to_string(), db.get_noise_sampler(k));
    }
    let settings_txt = std::fs::read_to_string(settings_path()).expect("settings read");
    let settings = parse(&settings_txt).expect("settings parse");
    if let Some(sr) = settings.get("surface_rule") {
        let mut dyn_keys: Vec<String> = Vec::new();
        WorldgenRust::surface_rules::collect_noise_keys(sr, &mut dyn_keys);
        for k in &dyn_keys {
            samplers.entry(k.clone()).or_insert_with(|| db.get_noise_sampler(k));
        }
    }
    let splitter: RsSplitter = db.random_deriver().clone();

    let sel = samplers.get(SELECTOR_KEY).expect("selector sampler");
    let pat = samplers.get(PATCH_KEY).expect("patch sampler");
    let srf = samplers.get(SURFACE_KEY).expect("surface sampler");

    // decisive probe：独立复刻 Java modern 派生链，打印首 octave origins（stderr）
    // 链：LegacyRandom(seed).next_splitter() -> split_str(key) -> [Octave create modern]
    //   next_splitter() -> split_str("octave_-4") -> PerlinNoiseSampler；second 从推进后的 rand 再取。
    {
        let mut r0 = WorldgenRust::legacy_random::LegacyRandom::new(SEED);
        let sp1 = r0.next_splitter();
        eprintln!("[R-SEED] s1(deriver)={}", sp1.seed);
        let hkey = WorldgenRust::legacy_random::java_string_hash("minecraft:nether_state_selector");
        let rand_seed = (hkey as i64) ^ sp1.seed;
        eprintln!("[R-SEED] hashKey={} randSeed={}", hkey, rand_seed);
        let mut rand0 = WorldgenRust::legacy_random::LegacyRandom::new_seed(rand_seed);
        let sp2 = rand0.next_splitter();
        eprintln!("[R-SEED] s2(octSplitter)={}", sp2.seed);
        let h_oct = WorldgenRust::legacy_random::java_string_hash("octave_-4");
        eprintln!("[R-SEED] octSeed={}", (h_oct as i64) ^ sp2.seed);
        let mut rand = sp1.split_str("minecraft:nether_state_selector");
        for chain in 0..2 {
            let sp2 = rand.next_splitter();
            let mut rnd = sp2.split_str("octave_-4");
            let mut rnd = WorldgenRust::legacy_random::RsRandom::Legacy(rnd);
            let pn = WorldgenRust::noise::PerlinNoiseSampler::new(&mut rnd);
            let (ox, oy, oz) = pn.origin();
            eprintln!("[R-ORIGIN] chain{} origin=({:.17},{:.17},{:.17})", chain, ox, oy, oz);
        }
    }

    // 头部（# 注释行，不影响对拍解析）
    println!("# seed={} settings={} min_y={} noise_height={} world_height={} points={}",
        SEED, SETTINGS, min_y, noise_height, WORLD_HEIGHT, points.len());

    // 3. 逐列采样
    for (x, z) in &points {
        let selector = sel.sample(*x as f64, 0.0, *z as f64);
        let patch = pat.sample(*x as f64, 0.0, *z as f64);
        let d = srf.sample(*x as f64, 0.0, *z as f64);
        let extra = splitter.split_xyz(*x, 0, *z).next_double();
        let surface_depth = (d * 2.75 + 3.0 + extra * 0.25) as i32;
        println!("{},{},{:.17e},{:.17e},surface_depth={},selector_bits={:x}",
            x, z, selector, patch, surface_depth, selector.to_bits());
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
//    主会话负责编译（cargo build --release --bin b1_selector_dump）与运行。
//
// ① 类型宽度（显式标注）：
//    - SEED: i64（const 显式）；坐标 i32；selector/patch/surface 噪声值 f64。
//    - `SEED as u64` 显式转 DensityBuilder::new（对齐 worldgen_handle.rs create_for_dim 同步）。
//    - `(d * 2.75 + 3.0 + extra * 0.25) as i32`：向零截断（对齐 sample_run_depth `as i32`）。
//    - `{:x}` 对 f64 无实现——已用 selector.to_bits()（u64）十六进制替代，命名 selector_bits。
//    - Rust {:.17e} 输出形如 `1.23456789012345671e-1`（指数无 +/零填充），Java %.17g 形如
//      `0.123456789012345672` 或 `1.23...E-1`——对拍按浮点数值解析，禁止字符串直接 diff。
//
// ② panic 路径（unwrap/expect 使用点清单）：
//    - fs::read(POINTS_PATH)：Err 只 eprintln + return（不 panic；文件缺失属环境错误）。
//    - String::from_utf8_lossy：非法字节替换为 U+FFFD（b1-sel-points.txt 首行实证非纯 UTF-8，
//      read_to_string 会整体 Err——这是与 soul_selector_probe 的关键差异点，勿改回）。
//    - read_to_string(settings_path).expect / parse().expect：settings 缺失/损坏 fail-fast。
//    - samplers.get(...).expect：三个 base key 本文件显式插入，不可能缺。
//    - 外部 loader 内 panic![LOADFAIL]：镜像 create_for_dim 同语义。
//    - 无 unsafe 除 env::remove_var（edition 2024 要求 unsafe，与 soul_selector_probe L44 同模式）。
//    - 行解析失败（token 不足/parse 错）continue，不 panic、不计数。
//
// ③ 与 lib 对拍点（逐行核对签名）：
//    - WorldgenHandle::create_for_dim(seed:i64, worldgen_dir:&str, settings:&str, biome_params:&str,
//      height:i32)（worldgen_handle.rs L112）——本文件未直接调用（本探针不需要 chunk 填充/
//      biome 判定），但保留同参组装：DensityBuilder::new(SEED as u64, min_y, noise_height) +
//      set_df_ns + set_legacy_random（nether legacy_random_source=true）+ noise_params 文件 +
//      external loader，采样器派生与生产路径同源。
//    - get_noise_sampler(key) / random_deriver() / set_external_loader / load_noise_params_file /
//      set_df_ns / set_legacy_random：签名与 soul_selector_probe.rs L97-140 用法逐字一致（该文件
//      已在既有 lib 上验证过装配模式）。
//    - DoublePerlinNoiseSampler::sample(f64, f64, f64)：采样点 (x, 0.0, z)，对齐
//      surface_rules.rs noise_threshold_sample L132（sample(ctx.block_x as f64, 0.0, ctx.block_z
//      as f64)）。直采无 thread_local 列缓存，与缓存语义数值等价（缓存只省重复采样）。
//    - surface_depth ↔ sample_run_depth：surface.sample(x,0,z)*2.75+3.0
//      + splitter.split_xyz(x,0,z).next_double()*0.25，as i32 截断（与 soul_selector_probe
//      L188-191 逐字同式）。
//    - collect_noise_keys（surface_rules.rs L160）：只收 noise_threshold 的 noise 字段——
//      nether.json 实证不含 "minecraft:patch" 字符串（grep 三处 nether_state_selector，无 patch），
//      故 patch 必须显式插入 base key 表。
//    - Java 侧对照（RouterProbe NOISEPT 模式）：NoiseConfig.getOrCreateSampler 反射 + RegistryKey
//      of(RegistryKeys.NOISE_PARAMETERS, "minecraft:nether_state_selector"/"minecraft:patch")，
//      sample((double)x, 0.0, (double)z)。两侧 worldSeed 必须一致（8576294172403134396），
//      对比前先核对 Java 输出 `#seed` 行（seed 三查纪律）。
//
// ④ lib 改动声明：无（worldgen_handle.rs / surface_rules.rs / lib.rs 零改动；
//    bin-diag 目录不参与默认构建）。
//
// ⑤ 已知边界（@anchor.idk 精神——诚实声明）：
//    - surface_depth 的 extra 取 splitter.split_xyz(x, 0, z) 首 next_double()——与
//      sample_run_depth 实际消耗序列一致性未经运行时验证（soul_selector_probe 同构先例，
//      编译运行后如 surface_depth 与 Java 全列错位先查这里）。
//    - patch 采样仅取 (x,0,z) 单点；若 Java patch 实际语义含 y 缩放差异（噪声定义同源则无），
//      由 selector/patch 双值同列对拍自动暴露。
//    - 输出未含行内注释（y=NN:标签）——Java NOISEPT 行同样只输出 x z，两侧按 (x,z) 键 join。
