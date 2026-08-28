// biome_rows.rs — 分析 biome_params.json 的 rows 数量 + 约束维度分布（设计预筛）。
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;

fn main() {
    let txt = fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json").unwrap();
    let arr = parse(&txt).unwrap().as_array().cloned().unwrap_or_default();
    println!("biome_params.json rows: {}", arr.len());
    // 统计每个维度的约束（range 非 [-1,1] 全开）
    let keys = ["temperature", "humidity", "continentalness", "erosion", "depth", "weirdness"];
    let mut constrained = [0usize; 6];
    let mut full_open = 0usize;
    let mut biome_count = std::collections::HashSet::new();
    for e in &arr {
        let biome = e.get("biome").and_then(|b| b.as_str()).unwrap_or("").to_string();
        biome_count.insert(biome);
        let params = e.get("parameters");
        for (i, key) in keys.iter().enumerate() {
            let r = params.and_then(|p| p.get(*key)).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let min = r.get(0).and_then(|x| x.as_f64()).unwrap_or(-1.0);
            let max = r.get(1).and_then(|x| x.as_f64()).unwrap_or(1.0);
            if min > -1.0 || max < 1.0 { constrained[i] += 1; }
        }
        // 全开（6 维都 [-1,1]）
        let mut all_open = true;
        for key in &keys {
            let r = params.and_then(|p| p.get(*key)).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let min = r.get(0).and_then(|x| x.as_f64()).unwrap_or(-1.0);
            let max = r.get(1).and_then(|x| x.as_f64()).unwrap_or(1.0);
            if min > -1.0 || max < 1.0 { all_open = false; break; }
        }
        if all_open { full_open += 1; }
    }
    println!("unique biomes: {}", biome_count.len());
    println!("每维度约束行数:");
    for (i, k) in keys.iter().enumerate() {
        println!("  {:<16} {}/{} constrained", k, constrained[i], arr.len());
    }
    println!("全开行（6维都[-1,1]）: {}", full_open);
    // depth 分布
    let mut depth_vals = std::collections::HashMap::new();
    for e in &arr {
        let params = e.get("parameters");
        let d = params.and_then(|p| p.get("depth")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let key = format!("{:?}", d.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect::<Vec<_>>());
        *depth_vals.entry(key).or_insert(0) += 1;
    }
    println!("depth 分布:");
    for (k, v) in &depth_vals { println!("  depth={} : {} rows", k, v); }
}
