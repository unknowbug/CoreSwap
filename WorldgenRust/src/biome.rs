// biome.rs — 宏观 biome 分类（MultiNoise 最近邻，对齐 vanilla MultiNoiseBiomeSource）。
// 之前是盒包含近似（丢失 biome 多样性，4096x4096 全 plains）；改为 vanilla 语义：
// 每个 biome 是 6 维超立方体（参数范围）+ offset，匹配 = 6 维平方距离 + offset² 最近邻。
// 参考：MultiNoiseUtil.java NoiseHypercube.getSquaredDistance + ParameterRange.getDistance。
// 性能：线性扫描 7593 行慢（54us/pt）；按 depth 分桶减半（30us/pt）；SearchTree（KD-tree，vanilla 方案）O(log n)。
use crate::density::{DensityFunction, NoisePos};
use crate::json::{JsonValue, parse as json_parse};
use std::sync::Arc;
use std::fs;

const NPARAMS: usize = 6;
const NDIMS: usize = 7; // 6 参数 + offset

// 每个 biome 超立方体：6 参数范围 [min,max] + offset
#[derive(Clone)]
struct BiomeEntry {
    biome: String,
    ranges: Vec<[f64; 2]>,  // temperature, humidity, continentalness, erosion, depth, weirdness
    offset: f64,
}

// ===== SearchTree（KD-tree，对齐 vanilla MultiNoiseUtil.SearchTree）=====
// 7 维（6 参数 + offset）。TreeBranchNode 存 enclosing 范围，getResultingNode 用子树距离剪枝。
enum SearchTreeNode {
    Branch { params: Vec<[f64; 2]>, sub: Vec<SearchTreeNode> },
    Leaf { params: Vec<[f64; 2]>, value: String },
}

impl SearchTreeNode {
    // 子树 enclosing 距离（点不在子树任何维度范围内则累加，否则 0）
    fn get_squared_distance(&self, vals: &[f64; 7]) -> f64 {
        let mut dist = 0.0;
        match self {
            SearchTreeNode::Branch { params, .. } | SearchTreeNode::Leaf { params, .. } => {
                for i in 0..NDIMS {
                    let r = params[i];
                    if r[0].is_nan() { continue; }
                    let l = vals[i] - r[1];
                    let m = r[0] - vals[i];
                    let dd = if l > 0.0 { l } else { m.max(0.0) };
                    dist += dd * dd;
                }
            }
        }
        dist
    }

    // 递归搜索最近叶子（剪枝：子树距离 >= best 则跳过）
    fn get_resulting_node(&self, vals: &[f64; 7], best_dist: &mut f64, best: &mut String) {
        match self {
            SearchTreeNode::Leaf { params, value } => {
                let mut dist = 0.0;
                for i in 0..NDIMS {
                    let r = params[i];
                    if r[0].is_nan() { continue; }
                    let l = vals[i] - r[1];
                    let m = r[0] - vals[i];
                    let dd = if l > 0.0 { l } else { m.max(0.0) };
                    dist += dd * dd;
                }
                if dist < *best_dist {
                    *best_dist = dist;
                    *best = value.clone();
                }
            }
            SearchTreeNode::Branch { sub, .. } => {
                for child in sub {
                    let d = child.get_squared_distance(vals);
                    if d < *best_dist {
                        child.get_resulting_node(vals, best_dist, best);
                    }
                }
            }
        }
    }
}

// 构建 SearchTree（对齐 vanilla createNode：size<=6 排序，否则按维度分桶）
fn build_search_tree(entries: &[BiomeEntry]) -> SearchTreeNode {
    if entries.is_empty() {
        return SearchTreeNode::Leaf { params: vec![[f64::NAN; 2]; NDIMS], value: "minecraft:plains".to_string() };
    }
    if entries.len() == 1 {
        let e = &entries[0];
        let mut params = e.ranges.clone();
        params.push([e.offset, e.offset]);
        return SearchTreeNode::Leaf { params, value: e.biome.clone() };
    }
    if entries.len() <= 6 {
        // 排序（按各维中点绝对值之和）
        let mut sorted: Vec<&BiomeEntry> = entries.iter().collect();
        sorted.sort_by_key(|e| {
            let mut sum = 0.0f64;
            for i in 0..NPARAMS {
                let r = e.ranges[i];
                if !r[0].is_nan() { sum += (r[0] + r[1]).abs() / 2.0; }
            }
            (sum * 10000.0) as i64
        });
        let sub: Vec<SearchTreeNode> = sorted.iter().map(|e| {
            let mut params = e.ranges.clone();
            params.push([e.offset, e.offset]);
            SearchTreeNode::Leaf { params, value: e.biome.clone() }
        }).collect();
        let params = enclosing(&sub);
        return SearchTreeNode::Branch { params, sub };
    }
    // 分桶：选使 range 长度和最小的维度，按该维中点分桶
    let mut best_dim = 0;
    let mut best_cost = f64::INFINITY;
    for dim in 0..NDIMS {
        let mut sorted: Vec<&BiomeEntry> = entries.iter().collect();
        sorted.sort_by(|a, b| {
            let am = if a.ranges.get(dim).map(|r| r[0].is_nan()).unwrap_or(true) { 0.0 } else { (a.ranges[dim][0] + a.ranges[dim][1]) / 2.0 };
            let bm = if b.ranges.get(dim).map(|r| r[0].is_nan()).unwrap_or(true) { 0.0 } else { (b.ranges[dim][0] + b.ranges[dim][1]) / 2.0 };
            am.partial_cmp(&bm).unwrap()
        });
        // 分桶（每桶 ~sqrt(n)）
        let bucket = (entries.len() as f64).sqrt().ceil() as usize;
        let mut cost = 0.0;
        for chunk in sorted.chunks(bucket) {
            let mut params = vec![[f64::NAN; 2]; NDIMS];
            for e in chunk {
                for i in 0..NPARAMS {
                    let r = e.ranges[i];
                    if r[0].is_nan() { continue; }
                    if params[i][0].is_nan() || r[0] < params[i][0] { params[i][0] = r[0]; }
                    if params[i][1].is_nan() || r[1] > params[i][1] { params[i][1] = r[1]; }
                }
                let o = e.offset;
                if params[6][0].is_nan() || o < params[6][0] { params[6][0] = o; }
                if params[6][1].is_nan() || o > params[6][1] { params[6][1] = o; }
            }
            for i in 0..NDIMS {
                if !params[i][0].is_nan() { cost += (params[i][1] - params[i][0]).abs(); }
            }
        }
        if cost < best_cost { best_cost = cost; best_dim = dim; }
    }
    // 按 best_dim 分桶
    let mut sorted: Vec<&BiomeEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        let am = if a.ranges.get(best_dim).map(|r| r[0].is_nan()).unwrap_or(true) { 0.0 } else { (a.ranges[best_dim][0] + a.ranges[best_dim][1]) / 2.0 };
        let bm = if b.ranges.get(best_dim).map(|r| r[0].is_nan()).unwrap_or(true) { 0.0 } else { (b.ranges[best_dim][0] + b.ranges[best_dim][1]) / 2.0 };
        am.partial_cmp(&bm).unwrap()
    });
    let bucket = (entries.len() as f64).sqrt().ceil() as usize;
    let sub: Vec<SearchTreeNode> = sorted.chunks(bucket).map(|chunk| {
        let chunk_owned: Vec<BiomeEntry> = chunk.iter().map(|e| (*e).clone()).collect();
        build_search_tree(&chunk_owned)
    }).collect();
    let params = enclosing(&sub);
    SearchTreeNode::Branch { params, sub }
}

// 计算子树 enclosing 范围（7 维）
fn enclosing(sub: &[SearchTreeNode]) -> Vec<[f64; 2]> {
    let mut params = vec![[f64::NAN; 2]; NDIMS];
    for node in sub {
        let node_params = match node {
            SearchTreeNode::Branch { params, .. } | SearchTreeNode::Leaf { params, .. } => params,
        };
        for i in 0..NDIMS {
            let r = node_params[i];
            if r[0].is_nan() { continue; }
            if params[i][0].is_nan() || r[0] < params[i][0] { params[i][0] = r[0]; }
            if params[i][1].is_nan() || r[1] > params[i][1] { params[i][1] = r[1]; }
        }
    }
    params
}

pub struct BiomeClassifier {
    // 按 depth 分桶（depth 0/1 各 ~3800 行），每桶一个 SearchTree
    tree_depth0: SearchTreeNode,
    tree_depth1: SearchTreeNode,
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
        let tree_depth0 = build_search_tree(&rows_depth0);
        let tree_depth1 = build_search_tree(&rows_depth1);
        BiomeClassifier { tree_depth0, tree_depth1 }
    }

    // 对齐 vanilla ParameterRange.getDistance：点不在 [min,max] 内则到最近边界距离，否则 0
    #[inline]
    fn range_distance(noise: f64, range: &[f64; 2]) -> f64 {
        let l = noise - range[1];
        let m = range[0] - noise;
        if l > 0.0 { l } else { m.max(0.0) }
    }

    // 对齐 vanilla NoiseHypercube.getSquaredDistance：6 维平方距离 + offset²，找最近邻
    // 优化：按 depth 分桶 + SearchTree（KD-tree 剪枝）
    pub fn biome_of(&self, tempf: &DensityFunction, humf: &DensityFunction, contf: &DensityFunction,
        erof: &DensityFunction, depthf: &DensityFunction, weirdf: &DensityFunction, pos: &NoisePos) -> String {
        let t = tempf.sample(pos); let h = humf.sample(pos); let c = contf.sample(pos);
        let e = erof.sample(pos); let d = depthf.sample(pos); let w = weirdf.sample(pos);
        let vals = [t, h, c, e, d, w, 0.0]; // 7 维（offset 采样点=0）
        // 选 depth 桶
        let tree = if d < 0.5 { &self.tree_depth0 } else { &self.tree_depth1 };
        let mut best_dist = f64::INFINITY;
        let mut best = "minecraft:plains".to_string();
        tree.get_resulting_node(&vals, &mut best_dist, &mut best);
        best
    }
}
