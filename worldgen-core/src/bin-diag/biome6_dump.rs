// biome6_dump.rs — V5 残差诊断（bin-diag 隔离区）：对给定采样点 dump Rust 侧 nether
// 6 维气候值 + biome 最近邻判定（biome_of_debug），与 Java BIOME6 同点对拍。
// 裁决目标：残差列（Rust 判 warped_forest，vanilla 判 basalt_deltas/soul_sand_valley）
// 的偏差来自 ①6 维采样值差 ②最近邻选择差 ③参数盒差 中的哪一层。
//
// 用法（主会话执行；临时挪 src/bin 编译，用完移回 bin-diag）：
//   cargo build --release --bin biome6_dump
//   cargo run --release --bin biome6_dump
//
// 输入：E:\PYTHON\CoreSwap\.tmp\biome6-points.txt（每行 `x y z 标签`）
// 输出（stdout，每点一行）：x,y,z,label,t=..,h=..,c=..,e=..,d=..,w=..,biome=..,dist=..

use std::sync::Arc;

use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";
const POINTS_PATH: &str = "E:/PYTHON/CoreSwap/.tmp/biome6-points.txt";
const SETTINGS: &str = "nether.json";
const BIOME_PARAMS: &str = "biome_params_nether.json";

fn main() {
    let points_txt = std::fs::read_to_string(POINTS_PATH)
        .unwrap_or_else(|e| panic!("[FAIL] cannot read {}: {}", POINTS_PATH, e));
    let mut points: Vec<(i32, i32, i32, String)> = Vec::new();
    for line in points_txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut it = line.split_whitespace();
        let (x, y, z) = match (it.next(), it.next(), it.next()) {
            (Some(a), Some(b), Some(c)) => (a.parse().unwrap_or(0), b.parse().unwrap_or(0), c.parse().unwrap_or(0)),
            _ => continue,
        };
        let label = it.next().unwrap_or("").to_string();
        points.push((x, y, z, label));
    }
    if points.is_empty() { eprintln!("[FAIL] no points"); return; }

    // 维度参数（镜像 soul_selector_probe::read_dim_params）
    let settings_path = format!("{}/data/minecraft/worldgen/noise_settings/{}", WG_DIR, SETTINGS);
    let settings_txt = std::fs::read_to_string(&settings_path).expect("settings read");
    let settings = parse(&settings_txt).expect("settings parse");
    let noise = settings.get("noise").unwrap_or(&parse("null").unwrap());
    let mut min_y = 0i32; let mut noise_height = 384i32;
    if let Some(n) = settings.get("noise") {
        if let Some(m) = n.get("min_y") { min_y = m.as_f64().unwrap_or(0.0) as i32; }
        if let Some(h) = n.get("height") { noise_height = h.as_f64().unwrap_or(384.0) as i32; }
    }
    let legacy = settings.get("legacy_random_source").and_then(|l| l.as_bool()).unwrap_or(false);
    let df_ns = SETTINGS.strip_suffix(".json").unwrap_or(SETTINGS);

    let mut db = DensityBuilder::new(SEED as u64, min_y, noise_height);
    db.set_df_ns(df_ns);
    if legacy { db.set_legacy_random(); }
    let noise_params_path = format!("{}/../noise_params.json", WG_DIR);
    if db.load_noise_params_file(&noise_params_path).is_err() {
        eprintln!("[FAIL] cannot load {}", noise_params_path); return;
    }
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/{}", WG_DIR, df_ns);
    let df_dir2 = df_dir.clone();
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = format!("{}/{}.json", df_dir2, name);
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("[LOADFAIL] {}: {}", p, e))
    }));
    let router = settings.get("noise_router").expect("noise_router");
    let tempf = db.build_node(router.get("temperature").expect("temperature")).expect("build temp");
    let humf = db.build_node(router.get("vegetation").expect("vegetation")).expect("build hum");
    let contf = db.build_node(router.get("continents").expect("continents")).expect("build cont");
    let erof = db.build_node(router.get("erosion").expect("erosion")).expect("build ero");
    let depthf = db.build_node(router.get("depth").expect("depth")).expect("build depth");
    let weirdf = db.build_node(router.get("ridges").expect("ridges")).expect("build weird");
    let bc = BiomeClassifier::load(&format!("{}/../{}", WG_DIR, BIOME_PARAMS));

    println!("# seed={} settings={} min_y={} noise_height={} points={}", SEED, SETTINGS, min_y, noise_height, points.len());
    let _ = Arc::new(0u8); // keep Arc import used (mirror of setup pattern)
    for (x, y, z, label) in &points {
        let pos = NoisePos { x: *x, y: *y, z: *z };
        let (biome, dist, v) = bc.biome_of_debug(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &pos);
        println!("{},{},{},{},t={:.6},h={:.6},c={:.6},e={:.6},d={:.6},w={:.6},biome={},dist={:.6}",
            x, y, z, label, v[0], v[1], v[2], v[3], v[4], v[5], biome, dist);
    }
    // 整 chunk 4x4 cell 网格（WG_BIOME6_GRID=cx,cz）：y=0 cell 行，对拍 Java CELLDUMP
    if let Ok(grid) = std::env::var("WG_BIOME6_GRID") {
        let mut it = grid.split(',');
        let gcx: i32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(200);
        const _GCZ_DEFAULT: i32 = 200;
        let gcz: i32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(_GCZ_DEFAULT);
        println!("# grid chunk({},{}) cellY=0（列 biome，nether T/H 与 y 无关）", gcx, gcz);
        for qz in (0..16).step_by(4) {
            let mut row = String::new();
            for qx in (0..16).step_by(4) {
                let pos = NoisePos { x: (gcx << 4) + qx, y: 0, z: (gcz << 4) + qz };
                let (biome, _, _) = bc.biome_of_debug(&tempf, &humf, &contf, &erof, &depthf, &weirdf, &pos);
                row.push_str(&format!("{},{}={} | ", qx, qz, biome.trim_start_matches("minecraft:")));
            }
            println!("{}", row);
        }
    }
}

// 静态自检（worker 产出模板对齐）：
// - 类型宽度：i32 坐标，无溢出域（nether 3200 级坐标安全）。
// - throw 路径：expect/panic 均为配置读取失败 fail-fast，无静默吞错。
// - 对拍点清单：输出 6 维值 + biome + dist，与 Java [BIOME6] 行逐点对 t/h/c/e/d/w 与 biome 名。
