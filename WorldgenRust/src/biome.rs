// biome.rs — 宏观 biome 分类（MultiNoise 最近邻，对齐 vanilla MultiNoiseBiomeSource）。
// 之前是盒包含近似（丢失 biome 多样性，4096x4096 全 plains）；改为 vanilla 语义：
// 每个 biome 是 6 维超立方体（参数范围）+ offset，匹配 = 6 维平方距离 + offset² 最近邻。
// 参考：MultiNoiseUtil.java NoiseHypercube.getSquaredDistance + ParameterRange.getDistance。
// 不移植 SearchTree（分层优化）——biome 数 ~64，线性扫描足够（宽松判据）。
use crate::density::{DensityFunction, NoisePos};
use crate::json::{JsonValue, parse as json_parse};
use std::sync::Arc;
use std::fs;

const NPARAMS: usize = 6;

// 每个 biome 超立方体：6 参数范围 [min,max] + offset
struct BiomeEntry {
    biome: String,
    ranges: Vec<[f64; 2]>,  // temperature, humidity, continentalness, erosion, depth, weirdness
    offset: f64,
}

pub struct BiomeClassifier {
    // 按 depth 分桶（depth 0/1 各 ~3800 行），biome_of 采样时选对应桶（减半扫描）
    rows_depth0: Vec<BiomeEntry>,
    rows_depth1: Vec<BiomeEntry>,
}

fn read_box(v: &JsonValue) -> [f64; 2] {
    if let Some(a) = v.as_array() {
        let mut r = [f64::NAN; 2];
        for (i, x) in a.iter().enumerate() { if i < 2 { if let Some(n) = x.as_f64() { r[i] = n; } } }
        r
    } else { [f64::NAN; 2] }
}

impl BiomeClassifier {
    pub fn load(path: &str) -> Self {
        let txt = fs::read_to_string(path).expect("biome_params.json");
        let arr = json_parse(&txt).expect("parse biome_params").as_array().cloned().unwrap_or_default();
        let mut rows_depth0 = Vec::new();
        let mut rows_depth1 = Vec::new();
        for e in arr {
            let biome = e.get("biome").and_then(|b| b.as_str()).unwrap_or("").to_string();
            let params = e.get("parameters");
            let mut ranges = Vec::with_capacity(6);
            for key in ["temperature", "humidity", "continentalness", "erosion", "depth", "weirdness"] {
                ranges.push(params.map(|p| read_box(&p.get(key).unwrap_or(&JsonValue::Null))).unwrap_or([f64::NAN; 2]));
            }
            let offset = params.and_then(|p| p.get("offset")).and_then(|o| o.as_f64()).unwrap_or(0.0);
            // depth 分桶：depth range 中点 < 0.5 → depth0，否则 depth1
            let depth_mid = (ranges[4][0] + ranges[4][1]) / 2.0;
            let entry = BiomeEntry { biome, ranges, offset };
            if depth_mid < 0.5 { rows_depth0.push(entry); } else { rows_depth1.push(entry); }
        }
        BiomeClassifier { rows_depth0, rows_depth1 }
    }

    // 对齐 vanilla ParameterRange.getDistance：点不在 [min,max] 内则到最近边界距离，否则 0
    #[inline]
    fn range_distance(noise: f64, range: &[f64; 2]) -> f64 {
        let l = noise - range[1];
        let m = range[0] - noise;
        if l > 0.0 { l } else { m.max(0.0) }
    }

    // 对齐 vanilla NoiseHypercube.getSquaredDistance：6 维平方距离 + offset²，找最近邻
    // 优化：按 depth 分桶（depth 0/1 各 ~3800 行），采样后选对应桶（减半扫描）
    pub fn biome_of(&self, tempf: &DensityFunction, humf: &DensityFunction, contf: &DensityFunction,
        erof: &DensityFunction, depthf: &DensityFunction, weirdf: &DensityFunction, pos: &NoisePos) -> String {
        let t = tempf.sample(pos); let h = humf.sample(pos); let c = contf.sample(pos);
        let e = erof.sample(pos); let d = depthf.sample(pos); let w = weirdf.sample(pos);
        let vals = [t, h, c, e, d, w];
        // 选 depth 桶
        let rows = if d < 0.5 { &self.rows_depth0 } else { &self.rows_depth1 };
        let mut best = "minecraft:plains".to_string();
        let mut best_dist = f64::INFINITY;
        for entry in rows {
            let mut dist = 0.0;
            for i in 0..6 {
                let r = entry.ranges[i];
                if r[0].is_nan() { continue; }
                let dd = Self::range_distance(vals[i], &r);
                dist += dd * dd;
            }
            dist += entry.offset * entry.offset;
            if dist < best_dist {
                best_dist = dist;
                best = entry.biome.clone();
            }
        }
        best
    }
}
