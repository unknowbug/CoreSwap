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
    idx: usize, // 参数表行序（平局裁决序，260904-13）
}

// 260904-13 残 9 H2 根因修复：Java MultiNoiseUtil 在 1e-4 定点 long 域运算（toLong(v)= (long)(v*10000F)，
// 输入先过 float）。Rust 此前用 f64 全精度距离 → 定点域精确平局（如残 9 簇 deep_ocean/deep_lukewarm
// 平局 5041）在 f64 下变成 1e-9 假严格差 → 分类翻转（gravel→sand 残差）。
// 修复：输入与参数双向量化 i64 + i64 距离；平局按参数表序（Java/C++ 树序第一个的等价近似，残 9 一致）。
#[inline]
fn noise_to_long(v: f64) -> i64 {
    ((v as f32) * 10000.0f32) as i64 // C++ biome.h L155 同构：(long)(v * 10000.0F)
}

#[inline]
fn long_range_distance(noise: i64, range: &[i64; 2]) -> i64 {
    let l = noise - range[1];
    let m = range[0] - noise;
    if l > 0 { l } else { m.max(0) }
}

// ===== SearchTree（KD-tree，对齐 vanilla MultiNoiseUtil.SearchTree）=====
// 7 维（6 参数 + offset）。TreeBranchNode 存 enclosing 范围，getResultingNode 用子树距离剪枝。
#[derive(Clone)]
enum SearchTreeNode {
    Branch { params: Vec<[f64; 2]>, lparams: Vec<[i64; 2]>, sub: Vec<SearchTreeNode> },
    Leaf { params: Vec<[f64; 2]>, lparams: Vec<[i64; 2]>, value: String, idx: usize },
}

fn lenclosing(sub: &[SearchTreeNode]) -> Vec<[i64; 2]> {
    let mut params = vec![[i64::MAX, i64::MIN]; NDIMS]; // 空范围哨兵
    for node in sub {
        let lp = match node {
            SearchTreeNode::Branch { lparams, .. } | SearchTreeNode::Leaf { lparams, .. } => lparams,
        };
        for i in 0..NDIMS {
            if lp[i][0] > lp[i][1] { continue; } // 空范围哨兵
            if params[i][0] > params[i][1] {
                params[i][0] = lp[i][0];
                params[i][1] = lp[i][1];
            } else {
                if lp[i][0] < params[i][0] { params[i][0] = lp[i][0]; }
                if lp[i][1] > params[i][1] { params[i][1] = lp[i][1]; }
            }
        }
    }
    params
}

fn entry_lparams(e: &BiomeEntry) -> Vec<[i64; 2]> {
    let mut lp: Vec<[i64; 2]> = e.ranges.iter()
        .map(|r| if r[0].is_nan() { [i64::MAX, i64::MIN] } else { [noise_to_long(r[0]), noise_to_long(r[1])] })
        .collect();
    let o = noise_to_long(e.offset);
    lp.push([o, o]);
    lp
}

impl SearchTreeNode {
    // 子树 enclosing 距离（i64 定点域；点不在子树任何维度范围内则累加，否则 0）
    fn get_squared_distance(&self, vals: &[i64; 7]) -> i64 {
        let mut dist: i64 = 0;
        match self {
            SearchTreeNode::Branch { lparams, .. } | SearchTreeNode::Leaf { lparams, .. } => {
                for i in 0..NDIMS {
                    let r = &lparams[i];
                    if r[0] > r[1] { continue; } // 空范围哨兵（NAN 等价）
                    let dd = long_range_distance(vals[i], r);
                    dist += dd * dd;
                }
            }
        }
        dist
    }

    // 递归搜索最近叶子（i64 距离；平局按参数表序 idx 取先——对齐 Java/C++ 树序第一个语义）
    // @anchor.idk("平局 idx 序裁决是 Java 树遍历序的近似：跨 seed 出现跨树分支精确平局且参数表低 idx 行在遍历序后位时，可能与 vanilla 翻转（judge 260904-13 b3）；若未来出现成对互换型 biome 残差，第一嫌疑此处")
    fn get_resulting_node(&self, vals: &[i64; 7], best_dist: &mut i64, best: &mut (String, usize)) {
        match self {
            SearchTreeNode::Leaf { lparams, value, idx, .. } => {
                let mut dist: i64 = 0;
                for i in 0..NDIMS {
                    let r = &lparams[i];
                    if r[0] > r[1] { continue; }
                    let dd = long_range_distance(vals[i], r);
                    dist += dd * dd;
                }
                if dist < *best_dist || (dist == *best_dist && *idx < best.1) {
                    *best_dist = dist;
                    best.0 = value.clone();
                    best.1 = *idx;
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
        return SearchTreeNode::Leaf { params: vec![[f64::NAN; 2]; NDIMS], lparams: vec![[i64::MAX, i64::MIN]; NDIMS], value: "minecraft:plains".to_string(), idx: usize::MAX };
    }
    if entries.len() == 1 {
        let e = &entries[0];
        let mut params = e.ranges.clone();
        params.push([e.offset, e.offset]);
        let mut lparams = entry_lparams(e);
        let idx = e.idx;
        return SearchTreeNode::Leaf { params, lparams, value: e.biome.clone(), idx };
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
            let lparams = entry_lparams(e);
            let idx = e.idx;
            SearchTreeNode::Leaf { params, lparams, value: e.biome.clone(), idx }
        }).collect();
        let params = enclosing(&sub);
        let lparams = lenclosing(&sub);
        return SearchTreeNode::Branch { params, lparams, sub };
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
    let lparams = lenclosing(&sub);
    SearchTreeNode::Branch { params, lparams, sub }
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

#[derive(Clone)]
pub struct BiomeClassifier {
    // 单个 SearchTree（KD-tree）。judge review-002：depth 分桶有正确性 bug
    // （dripstone/lush_caves depth=[0.2,0.9] 中点 0.55 进 depth1，但采样 d<0.5 走 depth0 被排除），
    // 放弃分桶仅用 SearchTree（本身已 10×）。
    tree: SearchTreeNode,
    // biome id → carvers.air 列表（从 biome/*.json 加载，CARVERS 阶段用）
    carvers: std::collections::BTreeMap<String, Vec<String>>,
    // biome id → features 列表（features[step][]，从 biome/*.json 加载，FEATURES 阶段用）
    // BTreeMap（原 HashMap）：all_features_lists 迭代序每进程随机 → PlacedFeatureIndexer 编号随机 →
    // nether features 放置运行间不确定（2796 块差，2026-08-30）——按键序确定。
    features: std::collections::BTreeMap<String, Vec<Vec<String>>>,
}

fn read_box(v: &JsonValue) -> [f64; 2] {
    if let Some(a) = v.as_array() {
        let mut r = [f64::NAN; 2];
        for (i, x) in a.iter().enumerate() { if i < 2 { if let Some(n) = x.as_f64() { r[i] = n; } } }
        r
    } else { [f64::NAN; 2] }
}

// ===== BiomeAccess.getBiome 8 邻域选点（对齐 C++ biome.h biomePickCell）=====
// BiomeAccess.hashSeed：Hashing.sha256().hashLong(seed).asLong()（C++ biome.h biomeHashSeed 同构移植）
// Guava putLong 写 little-endian 8 字节；asLong 取摘要前 8 字节 little-endian。
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
    let mut h: [u32; 8] = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
    let len = data.len();
    let total = ((len + 8) / 64 + 1) * 64;
    let mut msg = vec![0u8; total];
    msg[..len].copy_from_slice(data);
    msg[len] = 0x80;
    let bit_len = (len as u64) * 8;
    for i in 0..8 { msg[total - 1 - i] = (bit_len >> (8 * i)) as u8; }
    for off in (0..total).step_by(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = ((msg[off + i*4] as u32) << 24) | ((msg[off + i*4+1] as u32) << 16)
                 | ((msg[off + i*4+2] as u32) << 8) | (msg[off + i*4+3] as u32);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b); h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f); h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i*4] = (h[i] >> 24) as u8; out[i*4+1] = (h[i] >> 16) as u8;
        out[i*4+2] = (h[i] >> 8) as u8; out[i*4+3] = h[i] as u8;
    }
    out
}

/// Java BiomeAccess.hashSeed（ChunkRegion L102：new BiomeAccess(this, BiomeAccess.hashSeed(seed))）
pub fn biome_hash_seed(seed: i64) -> i64 {
    let le: [u8; 8] = seed.to_le_bytes();
    let dg = sha256(&le);
    i64::from_le_bytes([dg[0], dg[1], dg[2], dg[3], dg[4], dg[5], dg[6], dg[7]])
}


// SeedMixer.mixSeed（1.20.1 无符号回绕语义）
fn mix_seed(seed: i64, salt: i64) -> i64 {
    let s = seed as u64;
    let v = s.wrapping_mul(s.wrapping_mul(6364136223846793005u64).wrapping_add(1442695040888963407u64));
    (v.wrapping_add(salt as u64)) as i64
}

// method_38108：floorMod(l >> 24, 1024) / 1024.0，然后 (d - 0.5) * 0.9
fn biome_jitter(l: i64) -> f64 {
    let shifted = l >> 24;
    let mut fm = shifted % 1024;
    if fm < 0 { fm += 1024; }
    let d = fm as f64 / 1024.0;
    (d - 0.5) * 0.9
}

// method_38106(seed, q, r, s, d, e, f)：8 邻域候选点到 block 的哈希扰动距离
fn biome_cell_distance(seed: i64, q: i32, r: i32, s: i32, d: f64, e: f64, f: f64) -> f64 {
    let mut m = mix_seed(seed, q as i64);
    m = mix_seed(m, r as i64);
    m = mix_seed(m, s as i64);
    m = mix_seed(m, q as i64);
    m = mix_seed(m, r as i64);
    m = mix_seed(m, s as i64);
    let g = biome_jitter(m);
    m = mix_seed(m, seed);
    let h = biome_jitter(m);
    m = mix_seed(m, seed);
    let n = biome_jitter(m);
    (f + n) * (f + n) + (e + h) * (e + h) + (d + g) * (d + g)
}

// BiomeAccess.getBiome(BlockPos) 的选点：block 坐标 → 选中的 biome 坐标 (px, py, pz)
// 等价 Java：i=x-2, j=y-2, k=z-2; l=i>>2...；8 邻域取最近扰动点
pub fn biome_pick_cell(access_seed: i64, block_x: i32, block_y: i32, block_z: i32) -> (i32, i32, i32) {
    let i = block_x - 2;
    let j = block_y - 2;
    let k = block_z - 2;
    let l = i >> 2;
    let m = j >> 2;
    let n = k >> 2;
    let d = (i & 3) as f64 / 4.0;
    let e = (j & 3) as f64 / 4.0;
    let f = (k & 3) as f64 / 4.0;
    let mut o = 0;
    let mut best = 1e300;
    for p in 0..8 {
        let bl = (p & 4) == 0;
        let bl2 = (p & 2) == 0;
        let bl3 = (p & 1) == 0;
        let q = if bl { l } else { l + 1 };
        let r = if bl2 { m } else { m + 1 };
        let s = if bl3 { n } else { n + 1 };
        let h = if bl { d } else { d - 1.0 };
        let t = if bl2 { e } else { e - 1.0 };
        let u = if bl3 { f } else { f - 1.0 };
        let v = biome_cell_distance(access_seed, q, r, s, h, t, u);
        if best > v { o = p; best = v; }
    }
    let px = if (o & 4) == 0 { l } else { l + 1 };
    let py = if (o & 2) == 0 { m } else { m + 1 };
    let pz = if (o & 1) == 0 { n } else { n + 1 };
    (px, py, pz)
}

impl BiomeClassifier {
    pub fn load(path: &str) -> Self {
        let txt = fs::read_to_string(path).expect("biome_params.json");
        let arr = json_parse(&txt).expect("parse biome_params").as_array().cloned().unwrap_or_default();
        let mut rows = Vec::new();
        for e in arr {
            let biome = e.get("biome").and_then(|b| b.as_str()).unwrap_or("").to_string();
            let params = e.get("parameters");
            let mut ranges = Vec::with_capacity(6);
            for key in ["temperature", "humidity", "continentalness", "erosion", "depth", "weirdness"] {
                ranges.push(params.map(|p| read_box(&p.get(key).unwrap_or(&JsonValue::Null))).unwrap_or([f64::NAN; 2]));
            }
            let offset = params.and_then(|p| p.get("offset")).and_then(|o| o.as_f64()).unwrap_or(0.0);
            rows.push(BiomeEntry { biome, ranges, offset, idx: rows.len() });
        }
        let tree = build_search_tree(&rows);
        BiomeClassifier { tree, carvers: std::collections::BTreeMap::new(), features: std::collections::BTreeMap::new() }
    }

    // 从 biome/*.json 加载 carvers.air（CARVERS 阶段用）。biome id "minecraft:plains" → plains.json。
    // 缺失/解析失败跳过（记 stderr）。返回加载的 biome 数（唯一 biome id）。
    pub fn load_carvers(&mut self, biome_dir: &str) -> usize {
        let mut count = 0;
        // 收集所有 biome id（从 SearchTree 叶子遍历，去重）
        let mut ids = Vec::new();
        self.collect_biome_ids(&self.tree, &mut ids);
        ids.sort();
        ids.dedup();
        for id in ids {
            let name = if let Some(stripped) = id.strip_prefix("minecraft:") { stripped } else { &id };
            let path = format!("{}/{}.json", biome_dir, name);
            let txt = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => { eprintln!("biome: no settings file {} (skip)", path); continue; }
            };
            let root = match json_parse(&txt) { Ok(r) => r, Err(_) => { eprintln!("biome: parse {} failed (skip)", path); continue; } };
            let mut carvers = Vec::new();
            if let Some(carvers_node) = root.get("carvers") {
                if let Some(air) = carvers_node.get("air") {
                    if let Some(arr) = air.as_array() {
                        for c in arr { if let Some(s) = c.as_str() { carvers.push(s.to_string()); } }
                    }
                }
            }
            if !carvers.is_empty() {
                self.carvers.insert(id.clone(), carvers);
                count += 1;
            }
        }
        count
    }

    fn collect_biome_ids(&self, node: &SearchTreeNode, out: &mut Vec<String>) {
        match node {
            SearchTreeNode::Leaf { value, .. } => out.push(value.clone()),
            SearchTreeNode::Branch { sub, .. } => { for c in sub { self.collect_biome_ids(c, out); } }
        }
    }

    // CARVERS 阶段：取 biome 的 carvers.air 列表（无则空）
    pub fn carvers_for(&self, biome: &str) -> &[String] {
        self.carvers.get(biome).map(|v| v.as_slice()).unwrap_or(&[])
    }

    // SteelMC uniform_carver_biome 优化：若所有已加载 biome 的 carvers 列表统一，返回该列表，
    // 则 apply_carvers 可跳过 289 次 biome 采样（overworld 统一为 [canyon,cave,cave_extra]）。
    // 不统一（下界/末地混合）→ None，回退 vanilla per-source 查找。
    pub fn uniform_carver_list(&self) -> Option<Vec<String>> {
        let mut uniform: Option<Vec<String>> = None;
        for v in self.carvers.values() {
            match &uniform {
                None => uniform = Some(v.clone()),
                Some(u) => {
                    if u != v { return None; }
                }
            }
        }
        uniform
    }

    // 从 biome/*.json 加载 features（FEATURES 阶段用）。features[step][] 分层列表。
    // 缺失/解析失败跳过。返回加载的 biome 数（唯一 biome id）。
    pub fn load_features(&mut self, biome_dir: &str) -> usize {
        let mut count = 0;
        let mut ids = Vec::new();
        self.collect_biome_ids(&self.tree, &mut ids);
        ids.sort();
        ids.dedup();
        for id in ids {
            let name = if let Some(stripped) = id.strip_prefix("minecraft:") { stripped } else { &id };
            let path = format!("{}/{}.json", biome_dir, name);
            let txt = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => { continue; }
            };
            let root = match json_parse(&txt) { Ok(r) => r, Err(_) => { continue; } };
            let mut features: Vec<Vec<String>> = Vec::new();
            if let Some(features_node) = root.get("features") {
                if let Some(arr) = features_node.as_array() {
                    for step in arr {
                        let mut step_list = Vec::new();
                        if let Some(step_arr) = step.as_array() {
                            for f in step_arr { if let Some(s) = f.as_str() { step_list.push(s.to_string()); } }
                        }
                        features.push(step_list);
                    }
                }
            }
            if !features.is_empty() {
                self.features.insert(id.clone(), features);
                count += 1;
            }
        }
        count
    }

    // FEATURES 阶段：取 biome 的 features 列表（features[step][]，无则空）
    pub fn features_for(&self, biome: &str) -> &[Vec<String>] {
        self.features.get(biome).map(|v| v.as_slice()).unwrap_or(&[])
    }

    // 返回所有 biome 的 features 列表（PlacedFeatureIndexer 构建用）
    pub fn all_features_lists(&self) -> Vec<Vec<Vec<String>>> {
        self.features.values().cloned().collect()
    }

    // 返回所有 unique carver id（预加载 carver 用）
    pub fn all_carver_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for v in self.carvers.values() {
            for c in v {
                if !ids.contains(c) { ids.push(c.clone()); }
            }
        }
        ids
    }

    // 返回所有 unique placed_feature id（预加载 feature 用）
    pub fn all_feature_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for v in self.features.values() {
            for step in v {
                for f in step {
                    if !ids.contains(f) { ids.push(f.clone()); }
                }
            }
        }
        ids
    }

    // 对齐 vanilla ParameterRange.getDistance：点不在 [min,max] 内则到最近边界距离，否则 0
    #[inline]
    fn range_distance(noise: f64, range: &[f64; 2]) -> f64 {
        let l = noise - range[1];
        let m = range[0] - noise;
        if l > 0.0 { l } else { m.max(0.0) }
    }

    // 对齐 vanilla MultiNoiseUtil：1e-4 定点 long 域最近邻（残 9 H2 修复，260904-13）
    pub fn biome_of(&self, tempf: &DensityFunction, humf: &DensityFunction, contf: &DensityFunction,
        erof: &DensityFunction, depthf: &DensityFunction, weirdf: &DensityFunction, pos: &NoisePos) -> String {
        let vals = [
            noise_to_long(tempf.sample(pos)), noise_to_long(humf.sample(pos)),
            noise_to_long(contf.sample(pos)), noise_to_long(erof.sample(pos)),
            noise_to_long(depthf.sample(pos)), noise_to_long(weirdf.sample(pos)),
            0i64, // offset 采样点=0
        ];
        let mut best_dist = i64::MAX;
        let mut best = ("minecraft:plains".to_string(), usize::MAX);
        self.tree.get_resulting_node(&vals, &mut best_dist, &mut best);
        best.0
    }

    /// 诊断：同 biome_of，但额外返回判定距离（定点域 raw i64）与 6 维采样值（biome 分类精度排查用，非热路径）
    pub fn biome_of_debug(&self, tempf: &DensityFunction, humf: &DensityFunction, contf: &DensityFunction,
        erof: &DensityFunction, depthf: &DensityFunction, weirdf: &DensityFunction, pos: &NoisePos)
        -> (String, f64, [f64; 6]) {
        let t = tempf.sample(pos); let h = humf.sample(pos); let c = contf.sample(pos);
        let e = erof.sample(pos); let d = depthf.sample(pos); let w = weirdf.sample(pos);
        let vals = [noise_to_long(t), noise_to_long(h), noise_to_long(c), noise_to_long(e), noise_to_long(d), noise_to_long(w), 0i64];
        let mut best_dist = i64::MAX;
        let mut best = ("minecraft:plains".to_string(), usize::MAX);
        self.tree.get_resulting_node(&vals, &mut best_dist, &mut best);
        (best.0, best_dist as f64, [t, h, c, e, d, w])
    }
}

