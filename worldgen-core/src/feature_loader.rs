// feature_loader.rs — FEATURES 阶段数据加载 + 调度（MC 1.20.1）
// 对应 C++: versions/1.20.1/cpp/worldgen/src/feature_loader.h
// Java 参照：world/gen/feature/util/PlacedFeatureIndexer.java + ChunkGenerator.generateFeatures
// 调度：set 3×3 biome → intSet 全局索引排序 → setDecoratorSeed(l,p,k) → PlacedFeature.generate
// 简化（Phase 3）：set = 当前 chunk biome；structure 部分跳过

use std::collections::HashMap;

use crate::blocks::BlockRegistry;
use crate::feature::{DiskFeatureConfig, OreFeatureConfig, SpringFeatureConfig, UnderwaterMagmaFeatureConfig};
use crate::json::JsonValue;
use crate::placement::{FeaturePlacementContext, PlacementModifier, PlacedFeature};

// ===== ConfiguredFeature 解析（type 分发）=====
// 260905-05：解除 2026-08-10 拍板注释（原 L50）——supersedes：G2 归因（g2-convergence-260905-03）
// 判定树/植被残差为「feature 类型未实现」而非「调度错位」+ 本课题（b2-tree-plan-260905-05 / feature parity Phase 4）。
#[derive(Clone)]
pub struct ConfiguredFeature {
    pub id: String,
    pub type_name: String,
    pub ore_config: OreFeatureConfig,
    pub disk_config: DiskFeatureConfig,
    pub spring_config: SpringFeatureConfig,
    pub magma_config: UnderwaterMagmaFeatureConfig,
    pub freeze_top: bool,
    // —— tree/植被载荷（260905-05 新增；恰好一个 Some，其余 None）——
    pub tree_config: Option<crate::tree::TreeFeatureConfig>,          // minecraft:tree
    pub selector_config: Option<crate::tree::RandomSelectorConfig>,   // minecraft:random_selector
    pub patch_config: Option<crate::tree::RandomPatchConfig>,         // minecraft:random_patch / flower
    pub simple_block_config: Option<crate::tree::SimpleBlockConfig>,  // minecraft:simple_block
}

impl ConfiguredFeature {
    pub fn parse(id: &str, root: &JsonValue, blocks: &BlockRegistry) -> ConfiguredFeature {
        let type_name = root.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let cfg = root.get("config");
        let mut cf = ConfiguredFeature {
            id: id.to_string(),
            type_name: type_name.clone(),
            ore_config: OreFeatureConfig::parse(None, blocks),
            disk_config: DiskFeatureConfig::parse(None, blocks),
            spring_config: SpringFeatureConfig::parse(None, blocks),
            magma_config: UnderwaterMagmaFeatureConfig::parse(None, blocks),
            freeze_top: false,
            tree_config: None,
            selector_config: None,
            patch_config: None,
            simple_block_config: None,
        };
        if type_name.contains("ore") {
            cf.ore_config = OreFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("disk") {
            cf.disk_config = DiskFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("spring") {
            cf.spring_config = SpringFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("underwater_magma") {
            cf.magma_config = UnderwaterMagmaFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("freeze_top_layer") {
            cf.freeze_top = true;
        } else if type_name == "minecraft:tree" {
            // 精确匹配（不用 contains：防 azalea_tree 等误伤；azalea_tree 在数据集 type 也是
            // minecraft:tree，走同一分支由 placer/size 解析告警兜底）
            cf.tree_config = crate::tree::TreeFeatureConfig::parse(cfg, blocks);
            if cf.tree_config.is_none() {
                eprintln!("[feature-loader] tree config parse failed: {id}");
            }
        } else if type_name == "minecraft:random_selector" {
            cf.selector_config = crate::tree::RandomSelectorConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:random_patch" || type_name == "minecraft:flower" {
            cf.patch_config = crate::tree::RandomPatchConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:simple_block" {
            cf.simple_block_config = crate::tree::SimpleBlockConfig::parse(cfg, blocks);
        } else {
            // 未知 configured type 显式告警（消除静默丢弃，b2 S2）——不 panic（S4 全量加载门）
            eprintln!("[feature-loader] unknown configured feature type: {type_name} ({id})");
        }
        cf
    }
}

// ===== PlacedFeatureIndexer（Java PlacedFeatureIndexer.java）=====
// Java 关键语义（generateFeatures L373-412 实测确认）：
//   - featureIndex = 遍历 biomes 首次出现递增编号（Object2IntMap.computeIfAbsent）
//   - IndexedFeatures.features[step] = 拓扑排序后按 step 过滤的列表（vanilla 无 cycle → featureIndex 升序）
//   - indexMapping = Util.lastIndexGetter = feature 在 features[step] 中的 lastIndex（map.put 覆盖）
//   - p = setDecoratorSeed(l, p, k) 的 p = indexMapping(feature) —— 不是 featureIndex！
//   - structure 的 setDecoratorSeed(l, m, k) 独立重置，不影响 feature 随机序列（Rust 可跳过 structure）
pub struct PlacedFeatureIndexer {
    // featureId → featureIndex（首现递增）
    pub index: HashMap<String, i32>,
    // [step] = features 列表（featureIndex 升序，Java 拓扑排序后无 cycle 结果）
    pub step_features: Vec<Vec<String>>,
    // [step][featureId] = lastIndex（Java Util.lastIndexGetter）
    pub last_index_map: Vec<HashMap<String, i32>>,
    // [featureIndex] = featureId
    pub all_features: Vec<String>,
}

impl PlacedFeatureIndexer {
    pub fn new() -> Self {
        PlacedFeatureIndexer { index: HashMap::new(), step_features: Vec::new(), last_index_map: Vec::new(), all_features: Vec::new() }
    }

    // biomes: 每个 biome 的 features 列表（features[step][]）
    pub fn build(&mut self, biomes_features: &[Vec<Vec<String>>]) {
        let mut next = 0;
        let mut max_step = 0;
        // 1. featureIndex（首现递增）——遍历顺序 = biomes 列表
        for e in biomes_features {
            max_step = max_step.max(e.len());
            for step in 0..e.len() {
                for fid in &e[step] {
                    if !self.index.contains_key(fid) {
                        self.index.insert(fid.clone(), next);
                        next += 1;
                    }
                }
            }
        }
        self.all_features = vec![String::new(); next as usize];
        for (fid, gidx) in &self.index {
            self.all_features[*gidx as usize] = fid.clone();
        }
        // 2. stepFeatures：按 featureIndex 升序分组到 step
        let mut all: Vec<(i32, i32, String)> = Vec::new();
        for e in biomes_features {
            for step in 0..e.len() {
                for fid in &e[step] {
                    if let Some(&gi) = self.index.get(fid) {
                        all.push((step as i32, gi, fid.clone()));
                    }
                }
            }
        }
        all.sort();
        self.step_features = vec![Vec::new(); max_step];
        for (st, _gi, fid) in all {
            let st = st as usize;
            if self.step_features[st].is_empty() || self.step_features[st].last() != Some(&fid) {
                self.step_features[st].push(fid);
            }
        }
        // 3. lastIndexMap（Java lastIndexGetter：map.put 覆盖 → 最后出现索引）
        self.last_index_map = vec![HashMap::new(); max_step];
        for st in 0..self.step_features.len() {
            for (i2, fid) in self.step_features[st].iter().enumerate() {
                self.last_index_map[st].insert(fid.clone(), i2 as i32);
            }
        }
    }

    // 某 biome 的 step k features → indexMapping 值集合（Java intSet），排序后返回
    pub fn int_set_for(&self, entry_features: &[Vec<String>], step: i32) -> Vec<i32> {
        let mut s: Vec<i32> = Vec::new();
        if step >= 0 && (step as usize) < entry_features.len() && (step as usize) < self.last_index_map.len() {
            for fid in &entry_features[step as usize] {
                if let Some(&li) = self.last_index_map[step as usize].get(fid) {
                    if !s.contains(&li) { s.push(li); }
                }
            }
        }
        s.sort();
        s
    }
}

// 懒加载 placed_feature / configured_feature 的缓存
pub struct FeatureCache {
    pub placed: HashMap<String, PlacedFeature>,
    pub configured: HashMap<String, ConfiguredFeature>,
}

impl FeatureCache {
    pub fn new() -> Self {
        FeatureCache { placed: HashMap::new(), configured: HashMap::new() }
    }

    // 加载 placed_feature JSON（懒加载）
    pub fn get_placed(&mut self, wg_dir: &str, id: &str, blocks: &BlockRegistry) -> Option<&PlacedFeature> {
        if self.placed.contains_key(id) { return self.placed.get(id); }
        let name = if let Some(s) = id.strip_prefix("minecraft:") { s } else { id };
        let path = format!("{}/data/minecraft/worldgen/placed_feature/{}.json", wg_dir, name);
        let txt = std::fs::read_to_string(&path).ok()?;
        let root = crate::json::parse(&txt).ok()?;
        let mut pf = PlacedFeature {
            id: id.to_string(),
            modifiers: Vec::new(),
            configured_feature: root.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
            step: 0,
            global_index: -1,
        };
        if let Some(mods) = root.get("placement") {
            if let Some(arr) = mods.as_array() {
                for m in arr {
                    if let Some(pm) = PlacementModifier::parse(m, blocks) {
                        pf.modifiers.push(pm);
                    }
                }
            }
        }
        self.placed.insert(id.to_string(), pf);
        self.placed.get(id)
    }

    // 加载 configured_feature JSON（懒加载）
    pub fn get_configured(&mut self, wg_dir: &str, id: &str, blocks: &BlockRegistry) -> Option<&ConfiguredFeature> {
        if self.configured.contains_key(id) { return self.configured.get(id); }
        let name = if let Some(s) = id.strip_prefix("minecraft:") { s } else { id };
        let path = format!("{}/data/minecraft/worldgen/configured_feature/{}.json", wg_dir, name);
        let txt = std::fs::read_to_string(&path).ok()?;
        let root = crate::json::parse(&txt).ok()?;
        let cf = ConfiguredFeature::parse(id, &root, blocks);
        self.configured.insert(id.to_string(), cf);
        self.configured.get(id)
    }

    // 预加载所有 placed_feature + 其引用的 configured_feature（创建时调用，之后只读无锁）。
    // placed_ids: 需要预加载的 placed_feature 全集（来自 all_feature_ids）。
    pub fn preload_all(&mut self, wg_dir: &str, placed_ids: &[String], blocks: &BlockRegistry) {
        for id in placed_ids {
            if self.placed.contains_key(id) { continue; }
            let name = if let Some(s) = id.strip_prefix("minecraft:") { s } else { id };
            let path = format!("{}/data/minecraft/worldgen/placed_feature/{}.json", wg_dir, name);
            let Ok(txt) = std::fs::read_to_string(&path) else { continue };
            let Ok(root) = crate::json::parse(&txt) else { continue };
            let mut pf = PlacedFeature {
                id: id.to_string(),
                modifiers: Vec::new(),
                configured_feature: root.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                step: 0,
                global_index: -1,
            };
            if let Some(mods) = root.get("placement") {
                if let Some(arr) = mods.as_array() {
                    for m in arr {
                        if let Some(pm) = PlacementModifier::parse(m, blocks) {
                            pf.modifiers.push(pm);
                        }
                    }
                }
            }
            // 预加载引用的 configured_feature
            let cname = if let Some(s) = pf.configured_feature.strip_prefix("minecraft:") { s } else { &pf.configured_feature };
            let cpath = format!("{}/data/minecraft/worldgen/configured_feature/{}.json", wg_dir, cname);
            if let Ok(ctxt) = std::fs::read_to_string(&cpath) {
                if let Ok(croot) = crate::json::parse(&ctxt) {
                    let cf = ConfiguredFeature::parse(&pf.configured_feature, &croot, blocks);
                    self.configured.insert(pf.configured_feature.clone(), cf);
                }
            }
            // —— 260905-05 增补：selector/patch 内嵌 placed 的递归预加载——
            // placed JSON 的 feature 字段可能是对象（内嵌 placed）而非 id 字符串；此时 configured_feature
            // 为空串，内嵌体已在 PlacementModifier::parse 阶段随 PlacedFeature::parse_inline 解析，
            // 其引用的 configured id 需在此登记到 self.configured（防运行时 cache miss）。
            if let Some(emb) = root.get("feature").filter(|f| f.as_object().is_some()) {
                let cid = emb.get("feature").and_then(|f| f.as_str()).unwrap_or("");
                if !cid.is_empty() && !self.configured.contains_key(cid) {
                    let cname = cid.strip_prefix("minecraft:").unwrap_or(cid);
                    let cpath = format!("{}/data/minecraft/worldgen/configured_feature/{}.json", wg_dir, cname);
                    if let Ok(ctxt2) = std::fs::read_to_string(&cpath) {
                        if let Ok(croot2) = crate::json::parse(&ctxt2) {
                            let cf = ConfiguredFeature::parse(cid, &croot2, blocks);
                            self.configured.insert(cid.to_string(), cf);
                        }
                    }
                }
            }
            self.placed.insert(id.to_string(), pf);
        }
    }
}

// 生成分发（ConfiguredFeature.generate → Feature.generate）
// 返回是否放置了方块。
// ⚠️ 签名变更（260905-05）：增 cache 参数——selector/patch 的内嵌 placed feature 走
// PlacedFeature::generate（同一 RNG 流 DFS 语义）。F-1 教训：加参后所有调用点必须同批改
// （本仓库唯一调用点 = worldgen_handle.rs L914-915 闭包，见 §2.4）。
pub fn generate_configured(
    cf: &ConfiguredFeature,
    ctx: &FeaturePlacementContext,
    octx: &mut crate::feature::OreFeatureContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    biome_temp: f32, biome_rainfall: f32,
    cache: &FeatureCache,
) -> bool {
    octx.origin_x = x; octx.origin_y = y; octx.origin_z = z;
    if cf.type_name.contains("ore") {
        let is_scattered = cf.type_name.contains("scattered_ore");
        if is_scattered {
            crate::feature::ScatteredOreFeature.generate(octx, &cf.ore_config, random)
        } else {
            crate::feature::OreFeature.generate(octx, &cf.ore_config, random)
        }
    } else if cf.type_name.contains("disk") {
        crate::feature::DiskFeature.generate(octx, &cf.disk_config, random)
    } else if cf.type_name.contains("spring") {
        crate::feature::SpringFeature.generate(octx, &cf.spring_config, random)
    } else if cf.type_name.contains("freeze_top_layer") {
        crate::feature::FreezeTopLayerFeature.generate(octx, biome_temp, biome_rainfall, random)
    } else if cf.type_name.contains("underwater_magma") {
        crate::feature::UnderwaterMagmaFeature.generate(octx, &cf.magma_config, random)
    } else if cf.type_name == "minecraft:tree" {
        match &cf.tree_config {
            Some(tc) => crate::tree::TreeFeatureConfig::generate(tc, octx, random, x, y, z),
            None => { eprintln!("[feature-loader] tree without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:random_selector" {
        match &cf.selector_config {
            Some(sc) => {
                // ⚠️ 占位公式（idk-7 未裁决）：「逐项 nextFloat() < chance 即选即返，否则 default」
                // 形态来自 JSON 结构推断——S6 动工前必须以 RandomSelectorFeature.java 核对，
                // 核对前本分支结果只可用于 smoke 不可用于对拍。
                for (chance, pf) in &sc.features {
                    if random.next_float() < *chance {
                        return pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                            generate_nested(c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                        });
                    }
                }
                match &sc.default_feature {
                    Some(pf) => pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                        generate_nested(c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                    }),
                    None => false,
                }
            }
            None => false,
        }
    } else if cf.type_name == "minecraft:random_patch" || cf.type_name == "minecraft:flower" {
        match &cf.patch_config {
            Some(pc) => {
                // ⚠️ 占位公式（idk-7 未裁决）：tries 循环 + nextInt(2*xz+1)-xz 偏移形态为推断。
                // 且嵌套 placed 完整 placement 链（含 in_square/count）的 RNG 流连续性未核对。
                let tries = pc.tries;
                for _ in 0..tries {
                    let sx = pc.xz_spread.get(random);
                    let sy = pc.y_spread.get(random);
                    let sz = pc.xz_spread.get(random);
                    let _ = (sx, sy, sz); // 偏移合成方式待 RandomPatchFeature.java 裁决后落地
                    // S7 落地点：pf.generate(ctx, random, x+?, y+?, z+?, ...)
                }
                false
            }
            None => false,
        }
    } else if cf.type_name == "minecraft:simple_block" {
        match &cf.simple_block_config {
            Some(sc) => {
                let state = sc.to_place.get(random);
                if crate::tree::can_replace_pub(octx, x, y, z) { // 公开包装 tree::can_replace
                    octx.set_block(x, y, z, state);
                    true
                } else { false }
            }
            None => false,
        }
    } else {
        false
    }
}

// 嵌套 placed → configured 的二次分发（selector/patch 内层用；保持同一 random 流）
// configured 引用经 cache 只读查找；未知 id 显式告警。
fn generate_nested(
    ctx: &FeaturePlacementContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    octx: &mut crate::feature::OreFeatureContext,
    cache: &FeatureCache,
    biome_temp: f32, biome_rainfall: f32,
) -> bool {
    // pf.generate 闭包只给坐标，configured id 由 parse_inline 时记入 PlacedFeature.configured_feature
    // 实现注记：PlacedFeature 需增 pub embedded_configured: Option<ConfiguredFeature>（内嵌对象时）
    // 或 configured_feature: String（id 引用时经 cache.get_configured 查）。
    // 本函数体在接线时二选一实现；此处仅声明签名与告警路径：
    eprintln!("[feature-loader] generate_nested hit (S6/S7 接线点)");
    let _ = (ctx, random, x, y, z, octx, cache, biome_temp, biome_rainfall);
    false
}
