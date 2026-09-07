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
    // 260905-08 重写：Java PlacedFeatureIndexer.collectIndexedFeatures 精确移植 ——
    // p 不是首现序，而是「biome 内相邻 feature 建边 → (step, featureIndex) 比较器 DFS 拓扑
    // → 后序反转 → 按 step 分组」，p = step 内末位索引（lastIndexGetter）。E-C2 对拍实锤
    // 首现序与 Java p 不一致（amethyst_geode java p=2 vs rust p=0 等）。
    pub fn build(&mut self, biomes_features: &[Vec<Vec<String>>]) {
        let mut next = 0;
        // 1. featureIndex（Java Object2IntMap.computeIfAbsent：首现递增，仅作比较器 tie-breaker）
        for e in biomes_features {
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
        // 2. 邻接：每个 biome 内 list（step 升序、step 内按 JSON 顺序）相邻节点建边
        type Node = (i32, i32); // (step, featureIndex)
        let mut adj: std::collections::BTreeMap<Node, std::collections::BTreeSet<Node>> = std::collections::BTreeMap::new();
        for e in biomes_features {
            let mut list: Vec<Node> = Vec::new();
            for (j, step_feats) in e.iter().enumerate() {
                for fid in step_feats {
                    if let Some(&gi) = self.index.get(fid) {
                        list.push((j as i32, gi));
                    }
                }
            }
            for w in list.windows(2) {
                adj.entry(w[0]).or_default().insert(w[1]);
            }
        }
        // 3. DFS（Java TopologicalSorts.sort：后序收集 + reverse；邻居按 (step,fidx) 升序——TreeSet 序）
        fn dfs(
            adj: &std::collections::BTreeMap<Node, std::collections::BTreeSet<Node>>,
            visited: &mut std::collections::HashSet<Node>,
            visiting: &mut std::collections::HashSet<Node>,
            out: &mut Vec<Node>,
            now: Node,
        ) -> bool {
            if visited.contains(&now) { return false; }
            if visiting.contains(&now) { return true; }
            visiting.insert(now);
            if let Some(succ) = adj.get(&now) {
                for &nxt in succ {
                    if dfs(adj, visited, visiting, out, nxt) { return true; }
                }
            }
            visiting.remove(&now);
            visited.insert(now);
            out.push(now);
            false
        }
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        let mut order: Vec<Node> = Vec::new();
        for &key in adj.keys() {
            if !visited.contains(&key) {
                dfs(&adj, &mut visited, &mut visiting, &mut order, key);
            }
        }
        order.reverse();
        // 4. 按 step 分组（Java builder：list.stream().filter(step == jx)）
        let max_step = biomes_features.iter().map(|e| e.len()).max().unwrap_or(0);
        self.step_features = vec![Vec::new(); max_step];
        for (st, fidx) in &order {
            let fid = self.all_features[*fidx as usize].clone();
            self.step_features[*st as usize].push(fid);
        }
        // 5. lastIndexMap（Java lastIndexGetter：map.put 覆盖 → 最后出现索引）
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

// 260907-05（A 组缺口 3）：feature id 自带命名空间（"minecraft:foo" / "modid:bar"）→
// 数据路径按 id 命名空间解析 data/<ns>/worldgen/<kind>/<short>.json（无冒号默认 minecraft）。
fn data_path(wg_dir: &str, kind: &str, id: &str) -> String {
    let (ns, short) = match id.split_once(':') { Some((n, s)) => (n, s), None => ("minecraft", id) };
    format!("{}/data/{}/worldgen/{}/{}.json", wg_dir, ns, kind, short)
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
        let path = data_path(wg_dir, "placed_feature", id);
        let txt = std::fs::read_to_string(&path).ok()?;
        let root = crate::json::parse(&txt).ok()?;
        let mut pf = PlacedFeature {
            id: id.to_string(),
            modifiers: Vec::new(),
            configured_feature: root.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
            step: 0,
            global_index: -1,
            inline_configured: None,
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
        let path = data_path(wg_dir, "configured_feature", id);
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
            let path = data_path(wg_dir, "placed_feature", id);
            let Ok(txt) = std::fs::read_to_string(&path) else { continue };
            let Ok(root) = crate::json::parse(&txt) else { continue };
            let mut pf = PlacedFeature {
                id: id.to_string(),
                modifiers: Vec::new(),
                configured_feature: root.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                step: 0,
                global_index: -1,
                inline_configured: None,
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
            let cpath = data_path(wg_dir, "configured_feature", &pf.configured_feature);
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
                    let cpath = data_path(wg_dir, "configured_feature", cid);
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
        // —— 260905-06 idk-7 递归补载：selector 的内层是 placed feature id（如 jungle_bush），
        // 不在任何 biome features 列表 → 主循环不加载 → 运行时 cache.placed miss。
        // 遍历已加载 configured 的 selector，递归补载内层 placed（及其 configured），到不动点。
        let mut queue: Vec<String> = Vec::new();
        for cf in self.configured.values() {
            if let Some(sc) = &cf.selector_config {
                for (_, pf) in &sc.features { queue.push(pf.configured_feature.clone()); }
                if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
            }
        }
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        while let Some(placed_id) = queue.pop() {
            if !seen.insert(placed_id.clone()) { continue; }
            if self.placed.contains_key(&placed_id) { continue; }
            let path = data_path(wg_dir, "placed_feature", &placed_id);
            let Ok(txt) = std::fs::read_to_string(&path) else { continue };
            let Ok(root) = crate::json::parse(&txt) else { continue };
            let mut pf = PlacedFeature {
                id: placed_id.clone(),
                modifiers: Vec::new(),
                configured_feature: root.get("feature").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                step: 0,
                global_index: -1,
                inline_configured: None,
            };
            if let Some(mods) = root.get("placement").and_then(|p| p.as_array()) {
                for m in mods {
                    if let Some(pm) = PlacementModifier::parse(m, blocks) { pf.modifiers.push(pm); }
                }
            }
            // 内层 placed 引用的 configured 也补载
            let cf_key = pf.configured_feature.clone();
            if !cf_key.is_empty() && !self.configured.contains_key(&cf_key) {
                let cpath = data_path(wg_dir, "configured_feature", &cf_key);
                if let Ok(ctxt) = std::fs::read_to_string(&cpath) {
                    if let Ok(croot) = crate::json::parse(&ctxt) {
                        let cf = ConfiguredFeature::parse(&cf_key, &croot, blocks);
                        // 嵌套 selector（selector 内层 placed → configured 又是 selector）继续入队
                        if let Some(sc) = &cf.selector_config {
                            for (_, ipf) in &sc.features { queue.push(ipf.configured_feature.clone()); }
                            if let Some(d) = &sc.default_feature { queue.push(d.configured_feature.clone()); }
                        }
                        self.configured.insert(cf_key, cf);
                    }
                }
            }
            self.placed.insert(placed_id, pf);
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
                // 260905-06 idk-7 取证落地（RandomSelectorFeature.java L22-28，mojmap 一手源）：
                // 逐项 nextFloat()<chance 即选即返（后续项不消费 RNG）；全落空走 default（不抽选择 RNG）。
                for (chance, pf) in &sc.features {
                    if random.next_float() < *chance {
                        return pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                            generate_nested(&pf.configured_feature, c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                        });
                    }
                }
                match &sc.default_feature {
                    Some(pf) => pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
                        generate_nested(&pf.configured_feature, c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                    }),
                    None => false,
                }
            }
            None => false,
        }
    } else if cf.type_name == "minecraft:random_patch" || cf.type_name == "minecraft:flower" {
        match &cf.patch_config {
            Some(pc) => {
                // 260905-06 idk-7 取证落地（RandomPatchFeature.java L15-33，yarn 一手源）：
                // tries 循环（feature JSON 字段，非 placement Count）；每 try 恒 6 次 nextInt：
                // nextInt(j)-nextInt(j) 差分（j=xz_spread+1）x→y→z（k=y_spread+1），值域 [-spread,+spread] 三角分布。
                // 内层 = PlacedFeature.generateUnregistered：链执行但 biome modifier 禁用（placedFeature=empty 会 throw；
                // Rust biome filter 为透传实现，无 throw 面——1.20.1 数据内嵌 placed 只用 block_predicate_filter，无实际差异）。
                let j = pc.xz_spread + 1;
                let k = pc.y_spread + 1;
                let mut placed_any = false;
                for _ in 0..pc.tries {
                    let sx = random.next_int_bound(j) - random.next_int_bound(j);
                    let sy = random.next_int_bound(k) - random.next_int_bound(k);
                    let sz = random.next_int_bound(j) - random.next_int_bound(j);
                    if pc.feature.generate(ctx, random, x + sx, y + sy, z + sz, |c2, r2, gx, gy, gz| {
                        // 260905-12：内联 configured 对象（Holder.direct）直发，无 id 不查 cache
                        if let Some(icf) = &pc.feature.inline_configured {
                            return generate_configured(icf, c2, octx, r2, gx, gy, gz, biome_temp, biome_rainfall, cache);
                        }
                        generate_nested(&pc.feature.configured_feature, c2, r2, gx, gy, gz, octx, cache, biome_temp, biome_rainfall)
                    }) {
                        placed_any = true;
                    }
                }
                placed_any // Java: return i > 0
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
// 260905-06 接线：configured 引用经 cache 查找后回 generate_configured（递归支持嵌套 selector/patch）。
// 嵌套分发（selector/patch 内层用；保持同一 random 流）。
// 260905-06 idk-7 语义修正：selector 的 features[].feature / default 是 **placed feature id**
// （Java WeightedPlacedFeature/PlacedFeature Holder——内层有自己的 placement 链要消费同一 RNG 流）；
// patch 内嵌对象（parse_inline 展开过 modifiers）的 configured_feature 字段则是 configured id。
// 统一策略：先查 placed（有 placement 链则完整走链），miss 再查 configured 直发。
fn generate_nested(
    nested_id: &str,
    ctx: &FeaturePlacementContext,
    random: &mut crate::chunkrandom::ChunkRandom,
    x: i32, y: i32, z: i32,
    octx: &mut crate::feature::OreFeatureContext,
    cache: &FeatureCache,
    biome_temp: f32, biome_rainfall: f32,
) -> bool {
    if let Some(pf) = cache.placed.get(nested_id) {
        // 260905-12：placed 内嵌 configured 为内联对象时 configured_feature 为空串，直发实体。
        // ⚠️ 死防御分支（judge WARN 260905-12）：当前 1.20.1 数据 placed JSON 的内联对象 feature
        // 为 0 个，本分支不可达；且此处绕过 pf.generate placement 链（Java 语义应先链后 configured）。
        // 残差专项（FEA-11）开工前如需启用：改镜像 random_patch 分支，走 pf.generate 完整链。
        if let Some(icf) = &pf.inline_configured {
            return generate_configured(icf, ctx, octx, random, x, y, z, biome_temp, biome_rainfall, cache);
        }
        let cf_id = pf.configured_feature.clone();
        return pf.generate(ctx, random, x, y, z, |c2, r2, gx, gy, gz| {
            match cache.configured.get(&cf_id) {
                Some(cf) => generate_configured(cf, c2, octx, r2, gx, gy, gz, biome_temp, biome_rainfall, cache),
                None => {
                    eprintln!("[feature-loader] generate_nested: unknown configured id: {cf_id} (via placed {nested_id})");
                    false
                }
            }
        });
    }
    if let Some(cf) = cache.configured.get(nested_id) {
        return generate_configured(cf, ctx, octx, random, x, y, z, biome_temp, biome_rainfall, cache);
    }
    eprintln!("[feature-loader] generate_nested: unknown id (placed+configured miss): {nested_id}");
    false
}
