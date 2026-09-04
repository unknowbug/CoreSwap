// eroded_diag.rs — eroded_badlands 距离诊断：对采样点输出到 eroded_badlands 超立方体的
// 最小距离 + 六维各自贡献，判断"种子没有"（某维噪声从不进入所需区间）vs"分类器偏"。
// 评估结论：eroded 需 高温(≥0.55)×干燥(≤-0.1)×内陆(cont≥0.03 多数)×低侵蚀(ero≤-0.375 多数)×weirdness≥-0.05。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let seed: i64 = -2032795982907864146;
    let mut db = DensityBuilder::new(seed as u64, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let t: Arc<DensityFunction> = Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap());
    let h: Arc<DensityFunction> = Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap());
    let c: Arc<DensityFunction> = Arc::new(db.build_node(router.get("continents").unwrap()).unwrap());
    let e: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let d: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let w: Arc<DensityFunction> = Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap());
    let _bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");

    // eroded_badlands 的关键约束（从 biome_params.json 评估摘要）：
    // temp >= 0.55, hum <= -0.1, cont >= 0.03, ero <= -0.375, weirdness >= -0.05
    // 直接统计采样点的六维原始值分布 + 满足各条件的比例
    let n = 6;
    let names = ["temperature", "humidity", "continentalness", "erosion", "depth", "weirdness"];
    let mut mins = [f64::INFINITY; 6];
    let mut maxs = [f64::NEG_INFINITY; 6];
    let mut cond_hit = [0u64; 5]; // temp/hum/cont/ero/weird 单条件命中
    let mut all5 = 0u64;
    let mut total: u64 = 0;
    let mut best_pt: Option<(i32, i32, [f64; 6])> = None;
    let mut best_miss = f64::INFINITY; // 距 5 条件联合命中的最小缺口

    for cz in (-1024..1024).step_by(16) {
        for cx in (-1024..1024).step_by(16) {
            let x = cx * 16; let z = cz * 16;
            let bp = NoisePos { x: x >> 2 << 2, y: 0, z: z >> 2 << 2 };
            let vals = [t.sample(&bp), h.sample(&bp), c.sample(&bp), e.sample(&bp), d.sample(&bp), w.sample(&bp)];
            for i in 0..n {
                mins[i] = mins[i].min(vals[i]);
                maxs[i] = maxs[i].max(vals[i]);
            }
            // 条件缺口（<0 = 满足）
            let gaps = [
                0.55 - vals[0],   // temp>=0.55
                vals[1] - (-0.1), // hum<=-0.1
                0.03 - vals[2],   // cont>=0.03
                vals[3] - (-0.375), // ero<=-0.375
                -0.05 - vals[4 + 1], // weirdness>=-0.05（vals[5]）
            ];
            for k in 0..5 { if gaps[k] < 0.0 { cond_hit[k] += 1; } }
            let max_gap = gaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            if max_gap < 0.0 { all5 += 1; }
            if max_gap < best_miss {
                best_miss = max_gap;
                best_pt = Some((cx, cz, vals));
            }
            total += 1;
        }
    }
    println!("eroded_diag ±1024 chunk ({} pts):", total);
    println!("  六维范围:");
    for i in 0..n {
        println!("    {:<16} [{:.4}, {:.4}]", names[i], mins[i], maxs[i]);
    }
    println!("  单条件命中率:");
    let cond_names = ["temp>=0.55", "hum<=-0.1", "cont>=0.03", "ero<=-0.375", "weird>=-0.05"];
    for k in 0..5 {
        println!("    {:<14} {}/{} ({:.2}%)", cond_names[k], cond_hit[k], total, 100.0 * cond_hit[k] as f64 / total as f64);
    }
    println!("  5 条件联合命中: {} ({:.4}%)", all5, 100.0 * all5 as f64 / total as f64);
    if let Some((cx, cz, vals)) = best_pt {
        println!("  最接近点 chunk({},{}) max_gap={:.4}", cx, cz, best_miss);
        println!("    t={:.3} h={:.3} c={:.3} e={:.3} d={:.3} w={:.3}", vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]);
    }
    if all5 == 0 {
        println!("  => 5 条件从未联合命中：该种子 ±1024 内无 eroded_badlands 参数区（记录待验证，转 C）");
    } else {
        println!("  => 有联合命中点！badlands pillar 可在该区域验证");
    }
}
