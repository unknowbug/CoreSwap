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
#[derive(Clone)]
pub struct ConfiguredFeature {
    pub id: String,                 // "minecraft:ore_granite_upper"
    pub type_name: String,          // "minecraft:ore" / "minecraft:scattered_ore" / "minecraft:disk" / ...
    pub ore_config: OreFeatureConfig,     // ore / scattered_ore 用
    pub disk_config: DiskFeatureConfig,   // disk 用
    pub spring_config: SpringFeatureConfig, // spring_feature 用
    pub magma_config: UnderwaterMagmaFeatureConfig, // underwater_magma 用
    pub freeze_top: bool,           // freeze_top_layer 用
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
        }
        // 树花植被（flower/random_patch/simple_block/tree/random_selector）不解析——2026-08-10 用户拍板范围外
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
}

// 生成分发（ConfiguredFeature.generate → Feature.generate）
// 返回是否放置了方块
pub fn generate_configured(
    cf: &ConfiguredFeature,
    ctx: &FeaturePlacementContext,
    octx: &mut crate::feature::OreFeatureContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    biome_temp: f32, biome_rainfall: f32,
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
    } else {
        // 生态装饰（flower/random_patch/simple_block/tree/random_selector）——2026-08-10 用户拍板范围外
        false
    }
}
