// soul_tree_repro.rs — V4 矛盾裁决（bin-diag 隔离区）：
// 解析 nether surface_rule → 递归打印产物树（cond 参数全量）→ 用 V4 dump ctx
// （3260,1,3200: biome=soul_sand_valley, sda=22, sdb=2, surface_depth=3, y=1）走 apply。
// 只依赖 blocks.json（parse 侧），noise sampler 用空表（selector cond 会走 warn+0.0 回退，
// 与生产采样值不同 → apply 结论只看「是否进 soul 分支」，不依赖 selector 真值）。
//
// 用法：cargo run --release --bin soul_tree_repro

use std::collections::HashMap;
use std::sync::Arc;

use WorldgenRust::blocks::BlockRegistry;
use WorldgenRust::json::parse;
use WorldgenRust::legacy_random::RsSplitter;
use WorldgenRust::surface_rules::{SurfaceBuilder, SurfaceContext, SurfaceRule};

const WG_DIR: &str = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";

fn print_rule(r: &SurfaceRule, indent: usize) {
    let pad = "  ".repeat(indent);
    match r {
        SurfaceRule::Block(b) => println!("{}Block({})", pad, b),
        SurfaceRule::Seq(v) => {
            println!("{}Seq[{}]", pad, v.len());
            for c in v { print_rule(c, indent + 1); }
        }
        SurfaceRule::Cond { cond, rule } => {
            print_cond(cond, indent);
            print_rule(rule, indent + 1);
        }
        SurfaceRule::TerracottaBands => println!("{}TerracottaBands", pad),
    }
}

fn print_cond(c: &WorldgenRust::surface_rules::SurfaceCond, indent: usize) {
    let pad = "  ".repeat(indent);
    match c {
        WorldgenRust::surface_rules::SurfaceCond::Biome { biomes } =>
            println!("{}Cond Biome {:?}", pad, biomes),
        WorldgenRust::surface_rules::SurfaceCond::AboveY { anchor_y, mult, add_stone_depth } =>
            println!("{}Cond AboveY anchor={} mult={} asd={}", pad, anchor_y, mult, add_stone_depth),
        WorldgenRust::surface_rules::SurfaceCond::StoneDepth { offset, add_surface_depth, secondary_depth_range, ceiling } =>
            println!("{}Cond StoneDepth off={} asd={} sdr={} ceiling={}", pad, offset, add_surface_depth, secondary_depth_range, ceiling),
        WorldgenRust::surface_rules::SurfaceCond::NoiseThreshold { noise_key, min_th, max_th } =>
            println!("{}Cond NoiseThreshold key={} min={} max={}", pad, noise_key, min_th, max_th),
        WorldgenRust::surface_rules::SurfaceCond::VerticalGradient { name, true_y, false_y } =>
            println!("{}Cond VerticalGradient {} true_y={} false_y={}", pad, name, true_y, false_y),
        WorldgenRust::surface_rules::SurfaceCond::Not(inner) => {
            println!("{}Cond Not", pad);
            print_cond(inner, indent + 1);
        }
        WorldgenRust::surface_rules::SurfaceCond::Hole => println!("{}Cond Hole", pad),
        WorldgenRust::surface_rules::SurfaceCond::Water { offset, mult, add_stone_depth } =>
            println!("{}Cond Water off={} mult={} asd={}", pad, offset, mult, add_stone_depth),
        WorldgenRust::surface_rules::SurfaceCond::Temp => println!("{}Cond Temp", pad),
        WorldgenRust::surface_rules::SurfaceCond::Steep => println!("{}Cond Steep", pad),
        WorldgenRust::surface_rules::SurfaceCond::SurfaceCondC => println!("{}Cond SurfaceCondC", pad),
    }
}

fn main() {
    let settings_txt = std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/nether.json", WG_DIR)).expect("settings");
    let settings = parse(&settings_txt).expect("parse");
    let sr = settings.get("surface_rule").expect("surface_rule");

    // 顶层 sequence 长度直接从 JSON 数
    let n_json = sr.get("sequence").and_then(|s| s.as_array()).map(|a| a.len()).unwrap_or(0);
    println!("# JSON root sequence entries = {}", n_json);

    let blocks_path = format!("{}/../blocks.json", WG_DIR);
    let blocks_json = std::fs::read_to_string(&blocks_path).expect("blocks.json");
    let blocks = BlockRegistry::load_from_json(&blocks_json).expect("blocks");
    let samplers: HashMap<String, Arc<WorldgenRust::noise::DoublePerlinNoiseSampler>> = HashMap::new();
    let splitter = RsSplitter::Xoro(WorldgenRust::xoroshiro::XoroshiroRandom::new(1).next_splitter());
    let sb = SurfaceBuilder::new(&samplers, &splitter, 32, &blocks);

    let rule = match sb.parse_surface_rule(sr, 0, 128) {
        Some(r) => r,
        None => { println!("[FAIL] parse returned None"); return; }
    };
    println!("# ===== parsed tree =====");
    print_rule(&rule, 0);

    // apply @ 3260,1,3200 with V4 dump ctx（空 sampler 表：selector cond 走 warn→0.0）
    let mut ctx = SurfaceContext::new(&samplers, &splitter, 0, 256);
    ctx.initial_density_at = Some(&|_x, _y, _z| 0.0);
    ctx.surface_depth = 3;
    ctx.init_vertical(22, 2, 33, 3260, 1, 3200, "minecraft:soul_sand_valley");
    let applied = rule.apply(&ctx);
    println!("# ===== apply @3260,1,3200 (biome=soul_sand_valley sda=22 sdb=2 sd=3) => {:?} (soul_soil=258 soul_sand=257 netherrack=256)", applied);
}
