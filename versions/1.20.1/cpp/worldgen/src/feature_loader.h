#pragma once
// feature_loader.h — FEATURES 阶段数据加载 + 调度（MC 1.20.1）
// Java 参照：world/gen/feature/util/PlacedFeatureIndexer.java + ChunkGenerator.generateFeatures
// 调度：set 3×3 biome → intSet 全局索引排序 → setDecoratorSeed(l,p,k) → PlacedFeature.generate
// 简化（Phase 3）：set = 当前 chunk biome；structure 部分跳过（-288 深海无结构影响）
#include <algorithm>
#include <map>
#include <set>
#include <string>
#include <vector>

#include "json.h"
#include "biome.h"
#include "placement.h"
#include "feature.h"

namespace wg {

// ===== ConfiguredFeature 解析（type 分发）=====
struct ConfiguredFeature {
    std::string id;                 // "minecraft:ore_granite_upper"
    std::string type;               // "minecraft:ore" / "minecraft:scattered_ore" / "minecraft:disk" / ...
    OreFeatureConfig oreConfig;     // ore / scattered_ore 用
    DiskFeatureConfig diskConfig;   // disk 用
    SpringFeatureConfig springConfig; // spring_feature 用
    UnderwaterMagmaFeatureConfig magmaConfig; // underwater_magma 用
    RandomPatchFeatureConfig randomPatchConfig; // flower / random_patch 用
    SimpleBlockFeatureConfig simpleConfig; // simple_block 用
    TreeFeatureConfig treeConfig; // tree 用
    RandomSelectorFeatureConfig randomSelectorConfig; // random_selector 用（trees_*）
    bool freezeTop = false;         // freeze_top_layer 用

    // 生成（Java ConfiguredFeature.generate → Feature.generate）
    bool generate(FeaturePlacementContext& ctx, OreFeatureContext& octx, const OreFeatureConfig& cfg,
                  int type, int x, int y, int z) const {
        (void)ctx;
        octx.originX = x; octx.originY = y; octx.originZ = z;
        if (type == 0) { // ore
            OreFeature f;
            return f.generate(octx, cfg);
        } else { // scattered_ore
            ScatteredOreFeature f;
            return f.generate(octx, cfg);
        }
    }
    // Phase 4：非 ore 分发（disk/spring/freeze_top_layer/underwater_magma）
    bool generateOther(FeaturePlacementContext& ctx, OreFeatureContext& octx,
                       int x, int y, int z, float biomeTemp, float biomeRainfall) const {
        (void)ctx;
        octx.originX = x; octx.originY = y; octx.originZ = z;
        if (type.find("disk") != std::string::npos) {
            DiskFeature f;
            return f.generate(octx, diskConfig);
        }
        if (type.find("spring") != std::string::npos) {
            SpringFeature f;
            return f.generate(octx, springConfig);
        }
        if (type.find("freeze_top_layer") != std::string::npos) {
            FreezeTopLayerFeature f;
            return f.generate(octx, biomeTemp, biomeRainfall);
        }
        if (type.find("underwater_magma") != std::string::npos) {
            UnderwaterMagmaFeature f;
            bool r = f.generate(octx, magmaConfig);
            if (getenv("WG_FEATURELOG") && r) {
                std::fprintf(stderr, "[MAGMA] fid=%s origin=(%d,%d,%d) placed\n", id.c_str(), octx.originX, octx.originY, octx.originZ);
            }
            return r;
        }
        // 生态装饰（flower/random_patch/tree）——2026-08-10 用户拍板范围外：实机 Mod 装饰主要挂 FEATURES 阶段，
        // C++ 全接管会丢 Mod 花/草/树；且 vanilla 装饰 JSON 版本差异大（1.20→1.21 大量变动）。block_probe 自证不依赖此。
        // 实现代码留档（RandomPatchFeature/SimpleBlockFeature/TreeFeature 在 feature.h），此处不接入。
        if (type.find("flower") != std::string::npos || type.find("random_patch") != std::string::npos) {
            return false;
        }
        if (type.find("simple_block") != std::string::npos) {
            return false;
        }
        if (type.find("tree") != std::string::npos) {
            return false;
        }
        return false;
    }

    static ConfiguredFeature parse(const std::string& id, const JsonValue& root, BlockRegistry& blocks) {
        ConfiguredFeature cf;
        cf.id = id;
        cf.type = root.get("type") ? root.get("type")->strVal : "";
        const JsonValue* cfg = root.get("config");
        if (cf.type.find("ore") != std::string::npos) {
            cf.oreConfig = OreFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("disk") != std::string::npos) {
            cf.diskConfig = DiskFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("spring") != std::string::npos) {
            cf.springConfig = SpringFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("underwater_magma") != std::string::npos) {
            cf.magmaConfig = UnderwaterMagmaFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("flower") != std::string::npos || cf.type.find("random_patch") != std::string::npos) {
            cf.randomPatchConfig = RandomPatchFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("simple_block") != std::string::npos) {
            cf.simpleConfig = SimpleBlockFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("tree") != std::string::npos) {
            cf.treeConfig = TreeFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("random_selector") != std::string::npos) {
            cf.randomSelectorConfig = RandomSelectorFeatureConfig::parse(cfg, blocks);
        } else if (cf.type.find("freeze_top_layer") != std::string::npos) {
            cf.freezeTop = true;
        }
        return cf;
    }
};

// ===== PlacedFeatureIndexer（Java PlacedFeatureIndexer.java）=====
// Java 关键语义（generateFeatures L373-412 实测确认）：
//   - featureIndex = 遍历 biomes 首次出现递增编号（Object2IntMap.computeIfAbsent）
//   - IndexedFeatures.features[step] = 拓扑排序后按 step 过滤的列表（vanilla 无 cycle → featureIndex 升序）
//   - indexMapping = Util.lastIndexGetter = feature 在 features[step] 中的 lastIndex（map.put 覆盖）
//   - p = setDecoratorSeed(l, p, k) 的 p = indexMapping(feature) —— 不是 featureIndex！
//   - structure 的 setDecoratorSeed(l, m, k) 独立重置，不影响 feature 随机序列（C++ 可跳过 structure）
class PlacedFeatureIndexer {
public:
    // featureId → featureIndex（首现递增）
    std::map<std::string, int> index;
    // [step] = features 列表（featureIndex 升序，Java 拓扑排序后无 cycle 结果）
    std::vector<std::vector<std::string>> stepFeatures;
    // [step][featureId] = lastIndex（Java Util.lastIndexGetter）
    std::vector<std::map<std::string, int>> lastIndexMap;

    void build(BiomeSource& biomes) {
        int next = 0;
        int maxStep = 0;
        // 1. featureIndex（首现递增）——遍历顺序 = biomes 列表 = getBiomes() = 参数表文件序去重（ImmutableSet 保序）
        for (size_t i = 0; i < biomes.size(); i++) {
            const BiomeEntry* e = &biomes.allEntries()[i];
            maxStep = (int)std::max(maxStep, (int)e->features.size());
            for (size_t step = 0; step < e->features.size(); step++) {
                for (const auto& fid : e->features[step]) {
                    if (index.find(fid) == index.end()) {
                        index[fid] = next++;
                    }
                }
            }
        }
        allFeatures.resize((size_t)next, "");
        for (auto& [fid, gidx] : index) {
            allFeatures[(size_t)gidx] = fid;
        }
        // 2. stepFeatures：按 featureIndex 升序分组到 step（拓扑排序稳定结果，vanilla 无 cycle）
        stepFeatures.assign((size_t)maxStep, {});
        for (auto& [fid, gidx] : index) {
            for (size_t step = 0; step < (size_t)maxStep; step++) {
                // 该 feature 出现在哪些 step？从 biomes 查
            }
        }
        // 更简单：重新遍历 biomes 收集 (step, featureIndex, featureId) 排序
        std::vector<std::tuple<int, int, std::string>> all;
        for (size_t i = 0; i < biomes.size(); i++) {
            const BiomeEntry* e = &biomes.allEntries()[i];
            for (size_t step = 0; step < e->features.size(); step++) {
                for (const auto& fid : e->features[step]) {
                    auto it = index.find(fid);
                    if (it != index.end()) all.push_back({(int)step, it->second, fid});
                }
            }
        }
        std::sort(all.begin(), all.end());
        for (auto& [st, gi, fid] : all) {
            if (stepFeatures[(size_t)st].empty() || stepFeatures[(size_t)st].back() != fid) {
                stepFeatures[(size_t)st].push_back(fid);
            }
        }
        // 3. lastIndexMap（Java lastIndexGetter：map.put 覆盖 → 最后出现索引）
        lastIndexMap.assign((size_t)maxStep, {});
        for (size_t st = 0; st < stepFeatures.size(); st++) {
            for (size_t i2 = 0; i2 < stepFeatures[st].size(); i2++) {
                lastIndexMap[st][stepFeatures[st][i2]] = (int)i2;
            }
        }
    }

    // 某 biome 的 step k features → indexMapping 值集合（Java intSet），排序后返回
    std::vector<int> intSetFor(const BiomeEntry* entry, int step) const {
        std::set<int> s;
        if (entry && step < (int)entry->features.size() && step < (int)lastIndexMap.size()) {
            for (const auto& fid : entry->features[step]) {
                auto it = lastIndexMap[(size_t)step].find(fid);
                if (it != lastIndexMap[(size_t)step].end()) s.insert(it->second);
            }
        }
        return std::vector<int>(s.begin(), s.end()); // 已排序（Java Arrays.sort）
    }

    std::vector<std::string> allFeatures; // [featureIndex] = featureId

private:
};

} // namespace wg
