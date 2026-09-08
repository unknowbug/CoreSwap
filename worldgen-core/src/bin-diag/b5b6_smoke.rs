// b5b6_smoke.rs — 260908-15 B5/B6 冒烟诊断（bin-diag 隔离区，不参与默认构建，rustc 单编）
// 用途：① parse 冒烟——1.21.6 数据下 5 个 fallen_*.json + oak_leaf_litter.json 走 ConfiguredFeature::parse，
//      断言 fallen_config Some / decorator 非 Unsupported；② 固定 seed generate 冒烟——fallen_oak_tree
//      走 generate_configured fallen_tree 分发，打印放置计数 / 首 log 坐标 / §4 对照表各消费点期望次数。
// ⚠️ 替代口径声明：ChunkRandom 是具体枚举（Checked/Xoroshiro）非 trait，无法透明包装计数 →
//    RNG 消费按交付文档 §4 对照表静态推导 + 放置结果（log 数/坐标）佐证；精确逐笔计数需 wrapper 改造（未做）。
// 编译（主会话执行，PowerShell，仓库根 E:\PYTHON\CoreSwap）：
//   $rlib = Get-ChildItem target\release\deps\libWorldgenRust*.rlib | Sort-Object LastWriteTime | Select-Object -Last 1
//   rustc --edition 2021 -O --extern WorldgenRust=$($rlib.FullName) -L target\release\deps `
//     worldgen-core\src\bin-diag\b5b6_smoke.rs -o target\release\b5b6_smoke.exe
//   target\release\b5b6_smoke.exe
// 输出全 ASCII（[OK]/[FAIL]）。
use WorldgenRust::blocks::{BlockColumn, BlockRegistry};
use WorldgenRust::chunkrandom::ChunkRandom;
use WorldgenRust::feature::OreFeatureContext;
use WorldgenRust::feature_loader::{generate_configured, ConfiguredFeature, FeatureCache};
use WorldgenRust::placement::FeaturePlacementContext;
use WorldgenRust::tree::TreeDecorator;

const DATA_ROOT: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.21.6\\data";
const FALLEN_CFS: [&str; 5] = [
    "fallen_oak_tree", "fallen_birch_tree", "fallen_spruce_tree", "fallen_jungle_tree", "fallen_super_birch_tree",
];
const LITTER_CF: &str = "oak_leaf_litter";

fn deco_name(d: &TreeDecorator) -> (&'static str, String) {
    match d {
        TreeDecorator::Cocoa { probability } => ("Cocoa", format!("prob={probability}")),
        TreeDecorator::TrunkVine => ("TrunkVine", String::new()),
        TreeDecorator::LeaveVine { probability } => ("LeaveVine", format!("prob={probability}")),
        TreeDecorator::Beehive { probability } => ("Beehive", format!("prob={probability}")),
        TreeDecorator::PlaceOnGround { tries, radius, height, .. } => {
            ("PlaceOnGround", format!("tries={tries} radius={radius} height={height}"))
        }
        TreeDecorator::AttachedToLogs { probability, directions, .. } => {
            ("AttachedToLogs", format!("prob={probability} dirs={}", directions.len()))
        }
        TreeDecorator::Unsupported { type_name } => ("Unsupported", type_name.clone()),
    }
}

fn load_cf(id: &str, blocks: &BlockRegistry) -> Option<ConfiguredFeature> {
    let path = format!("{DATA_ROOT}\\worldgen\\data\\minecraft\\worldgen\\configured_feature\\{id}.json");
    let txt = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let root = WorldgenRust::json::parse(&txt).unwrap_or_else(|e| panic!("parse {id}: {e}"));
    Some(ConfiguredFeature::parse(&format!("minecraft:{id}"), &root, blocks))
}

fn main() {
    println!("== b5b6_smoke 260908-15 ==");
    let blocks_txt = std::fs::read_to_string(format!("{DATA_ROOT}\\blocks.json")).expect("blocks.json");
    let blocks = BlockRegistry::load_from_json(&blocks_txt).expect("BlockRegistry");
    println!("[OK] blocks.json loaded");

    // ===== Part 1: parse 冒烟 =====
    let mut fails = 0usize;
    for id in FALLEN_CFS {
        let Some(cf) = load_cf(id, &blocks) else {
            println!("[FAIL] {id}: parse returned None"); fails += 1; continue;
        };
        let Some(fc) = &cf.fallen_config else {
            println!("[FAIL] {id}: fallen_config is None"); fails += 1; continue;
        };
        println!("[OK] {id}: fallen_config Some");
        for (tag, list) in [("stump", &fc.stump_decorators), ("log", &fc.log_decorators)] {
            if list.is_empty() { println!("     {id} {tag}_decorators: (empty)"); continue; }
            for d in list {
                let (name, info) = deco_name(d);
                if name == "Unsupported" {
                    println!("[FAIL] {id} {tag}_decorator Unsupported: {info}"); fails += 1;
                } else {
                    println!("[OK] {id} {tag}_decorator: {name} {info}");
                }
            }
        }
    }
    // leaf_litter 族抽 1：place_on_ground parse 非 Unsupported
    match load_cf(LITTER_CF, &blocks) {
        Some(cf) => match &cf.tree_config {
            Some(tc) if !tc.decorators.is_empty() => {
                for (i, d) in tc.decorators.iter().enumerate() {
                    let (name, info) = deco_name(d);
                    if name == "Unsupported" {
                        println!("[FAIL] {LITTER_CF} decorator[{i}] Unsupported: {info}"); fails += 1;
                    } else {
                        println!("[OK] {LITTER_CF} decorator[{i}]: {name} {info}");
                    }
                }
            }
            Some(_) => { println!("[FAIL] {LITTER_CF}: zero decorators parsed"); fails += 1; }
            None => { println!("[FAIL] {LITTER_CF}: tree_config None"); fails += 1; }
        },
        None => { println!("[FAIL] {LITTER_CF}: parse returned None"); fails += 1; }
    }

    // ===== Part 2: 固定 seed generate 冒烟（fallen_oak_tree，经 generate_configured fallen_tree 分发）=====
    const SEED: i64 = 26090815;
    const ROUNDS: i32 = 8;
    const GROUND_Y: i32 = -61;
    let cf = load_cf("fallen_oak_tree", &blocks).expect("fallen_oak_tree");
    let oak_log = blocks.id("minecraft:oak_log");
    let cache = FeatureCache::new();
    println!("-- generate smoke seed={SEED} rounds={ROUNDS} ground_y={GROUND_Y} --");
    for round in 0..ROUNDS {
        let mut col = BlockColumn::new(-64, 384);
        // ⚠️ 修复（260908-15 panic 复盘）：BlockColumn.at/at_mut 是 chunk 局部坐标直索引
        //    （无守卫，blocks.rs:169-176）——此前用世界 -16..16 直填 → 局部 -16 → usize 回绕 panic。
        //    改 chunk_start=-8：世界 -8..8 ↔ 局部 0..16，4 向 fallen（N/W 向为负世界坐标）全落列内；
        //    引擎侧 block_at/set_block 经 OreFeatureContext::local_idx 守卫（越界 -1 = Java 越界放弃）。
        const HALF: i32 = 8; // 世界列半径；局部 = world + HALF
        // 平地：y<=GROUND_Y 实心（dirt 表层 + stone 基岩层），fallen 可找地面/悬空预检通过
        let dirt = blocks.id("minecraft:dirt");
        let stone = blocks.id("minecraft:stone");
        for x in -HALF..HALF { for z in -HALF..HALF {
            *col.at_mut(x + HALF, GROUND_Y, z + HALF) = dirt;
            *col.at_mut(x + HALF, GROUND_Y - 1, z + HALF) = stone;
            *col.at_mut(x + HALF, GROUND_Y - 2, z + HALF) = stone;
        }}
        let mut ctx = OreFeatureContext {
            col: &mut col,
            origin_x: 0, origin_y: GROUND_Y + 1, origin_z: 0,
            chunk_start_x: -HALF, chunk_start_z: -HALF,
            min_y: -64, height: 384,
            blocks: &blocks,
            ocean_floor: None, world_surface: None,
            region_col_at: None, pending_cross: None, block_at_ext: None,
        };
        let pctx = FeaturePlacementContext {
            biome_at: None, ocean_floor: None, world_surface: None,
            min_y: -64, height: 384, pos_to_biome: None,
            chunk_start_x: 0, chunk_start_z: 0,
            block_at: None, biome_allows: None, feature_id: None,
        };
        let mut random = ChunkRandom::checked();
        random.set_seed(SEED + round as i64);
        let ok = generate_configured(&cf, &pctx, &mut ctx, &mut random, 0, GROUND_Y + 1, 0, 0.0, 0.0, &cache);
        // 统计放置的 log（排除 origin 桩位本身；读走 ctx.block_at 世界坐标，local_idx 守卫越界）
        let mut logs: Vec<(i32, i32, i32)> = Vec::new();
        for x in -HALF..HALF { for z in -HALF..HALF { for y in -64..64 {
            if (x, y, z) != (0, GROUND_Y + 1, 0) && ctx.block_at(x, y, z) == oak_log {
                logs.push((x, y, z));
            }
        }}}
        logs.sort();
        let first = logs.first().map(|p| format!("({},{},{})", p.0, p.1, p.2)).unwrap_or_else(|| "(none)".into());
        let n = logs.len() as i32;
        // §4 对照表消费期望（fallen_oak：stump_decorators=[trunk_vine]，log_decorators=[attached_to_logs]，
        // trunk_provider=simple（放置 0 消费）；attached 的 weighted provider 仅成功路径消费，按 0..n 区间）
        let shuffle = if n > 0 { n - 1 } else { 0 }; // Σ_{j=2..n} 每档 1 次 nextInt(j)
        let per_log = 2; // nextInt(directions.len()) + nextFloat()（无条件，即使必败）
        println!(
            "[R{round}] placed_ok={ok} logs={n} first_log={first} | RNG expect: \
             p1_stump_provider=0 p2_stump_deco_trunkvine=4 p3_dir_nextInt4=1 p4_loglen_uniform=1 \
             p5_stub_nextInt2=1 p6_groundcheck=0 p7_log_provider=0 p8_shuffle={shuffle} \
             p9_dir_int10={per_log}x{n} p10_nextfloat={per_log}x{n} p11_provider_hit=0..{n}"
        );
        if !ok { println!("[FAIL] round {round}: generate_configured returned false"); fails += 1; }
        if n == 0 { println!("[WARN] round {round}: 0 logs placed (length 抽取可能为 0，或地面判定拒绝)"); }
    }

    if fails == 0 { println!("== SMOKE ALL [OK] =="); } else { println!("== SMOKE FAILURES: {fails} =="); }
}
