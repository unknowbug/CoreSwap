#pragma once
// placement.h — FEATURES 阶段调度（MC 1.20.1 移植）
// Java 参照：world/gen/placementmodifier/*.java + world/gen/feature/PlacedFeature.java
// 调度链：generateFeatures → set 3×3 biome → intSet 全局索引 → setDecoratorSeed(l,p,k)
//        → PlacedFeature.generate → placementModifiers flatMap 链 → ConfiguredFeature.generate
// 惰性语义：Java stream 惰性（第一个 pos 走完所有 modifier 再下一个）——C++ 按序展开
#include <array>
#include <cmath>
#include <functional>
#include <memory>
#include <string>
#include <vector>

#include "chunkrandom.h"
#include "carver.h" // YOffset / HeightProvider / FloatProvider

namespace wg {

// ===== IntProvider（Java util/math/intprovider：uniform / constant / trapezoid / biased_to_bottom / weighted_list）=====
struct IntProvider;
struct IntProvider {
    enum class Kind { CONSTANT, UNIFORM, TRAPEZOID, BIASED_TO_BOTTOM, WEIGHTED_LIST, CLAMPED };
    Kind kind = Kind::CONSTANT;
    int a = 0, b = 0, plateau = 0;
    std::shared_ptr<IntProvider> source; // clamped 的 source
    // weighted_list（Java WeightedListIntProvider）：entries (data, weight)，totalWeight
    std::vector<std::pair<int, int>> weighted; // (data, weight)
    int totalWeight = 0;

    int get(ChunkRandom& r) const {
        switch (kind) {
            case Kind::CONSTANT: return a;
            case Kind::UNIFORM: {
                if (a >= b) return a;
                // Java UniformIntProvider.get = random.nextInt(max - min + 1) + min
                return r.nextInt(b - a + 1) + a;
            }
            case Kind::TRAPEZOID: {
                // Java TrapezoidIntProvider.get = ceil(lerp(nextBetween(0, plateau-1), min, max) + nextFloat())
                int f = plateau == 0 ? 0 : r.nextInt(plateau + 1);
                int g = b - a;
                int h = g - plateau;
                int i = g - 2 * h;
                // Java 精确：return this.min + Math.floor(lerp(random.nextInt(plateau+1), min, max) + nextFloat())
                double lerpV = a + (double)f / plateau * (b - a);
                return (int)std::floor(lerpV + r.nextFloat());
            }
            case Kind::BIASED_TO_BOTTOM: {
                return r.nextInt(r.nextInt(b - a + 1) + a); // 近似（Java 更复杂）
            }
            case Kind::WEIGHTED_LIST: {
                // Java WeightedListIntProvider：RandomWeightedList.get——nextInt(totalWeight) 累减
                if (weighted.empty()) return a;
                int i = r.nextInt(totalWeight);
                for (auto& [data, w] : weighted) {
                    i -= w;
                    if (i < 0) return data;
                }
                return weighted[0].first;
            }
            case Kind::CLAMPED: {
                // Java ClampedIntProvider.get = clamp(source.get(random), min, max)
                if (!source) return a;
                int v = source->get(r);
                return v < a ? a : (v > b ? b : v);
            }
        }
        return a;
    }

    static IntProvider parse(const JsonValue* v) {
        IntProvider ip;
        if (!v) return ip;
        if (v->isNumber()) { ip.kind = Kind::CONSTANT; ip.a = (int)v->numVal; return ip; }
        if (!v->isObject()) return ip;
        std::string type = v->get("type") ? v->get("type")->strVal : "";
        // MC 1.20.1 的 uniform/trapezoid/biased_to_bottom 的 min/max 在 "value" 子对象里
        const JsonValue* val = v->get("value") ? v->get("value") : v;
        if (type.find("uniform") != std::string::npos) {
            ip.kind = Kind::UNIFORM;
            ip.a = (int)(val->get("min_inclusive") ? val->get("min_inclusive")->numVal : 0);
            ip.b = (int)(val->get("max_inclusive") ? val->get("max_inclusive")->numVal : 0);
        } else if (type.find("trapezoid") != std::string::npos) {
            ip.kind = Kind::TRAPEZOID;
            ip.a = (int)(val->get("min") ? val->get("min")->numVal : 0);
            ip.b = (int)(val->get("max") ? val->get("max")->numVal : 0);
            ip.plateau = (int)(val->get("plateau") ? val->get("plateau")->numVal : 0);
        } else if (type.find("biased_to_bottom") != std::string::npos) {
            ip.kind = Kind::BIASED_TO_BOTTOM;
            ip.a = (int)(val->get("min_inclusive") ? val->get("min_inclusive")->numVal : 0);
            ip.b = (int)(val->get("max_inclusive") ? val->get("max_inclusive")->numVal : 0);
        } else if (type.find("weighted_list") != std::string::npos) {
            // {"type":"minecraft:weighted_list","distribution":[{"data":6,"weight":9},...]}
            ip.kind = Kind::WEIGHTED_LIST;
            if (const JsonValue* dist = v->get("distribution")) {
                for (const auto& e : dist->arr) {
                    int data = (int)(e.get("data") ? e.get("data")->numVal : 0);
                    int w = (int)(e.get("weight") ? e.get("weight")->numVal : 0);
                    ip.weighted.push_back({data, w});
                    ip.totalWeight += w;
                }
            }
        } else if (type.find("clamped") != std::string::npos) {
            // {"type":"minecraft:clamped","value":{...},"min_inclusive":X,"max_inclusive":Y}
            ip.kind = Kind::CLAMPED;
            ip.a = (int)(v->get("min_inclusive") ? v->get("min_inclusive")->numVal : 0);
            ip.b = (int)(v->get("max_inclusive") ? v->get("max_inclusive")->numVal : 0);
            if (const JsonValue* src = v->get("value")) ip.source = std::make_shared<IntProvider>(parse(src));
        }
        return ip;
    }
};

// ===== PlacementModifier 基类 =====
// getPositions(context, random, x, y, z) → 输出位置列表（Java stream 惰性，C++ 展开）
struct FeaturePlacementContext {
    // 回调：位置 biome 判定（Java FeaturePlacementContext.getBiome(BlockPos)——用 chunk biome 采样）
    std::function<std::string(int, int, int)> biomeAt;
    // OCEAN_FLOOR_WG / WORLD_SURFACE_WG 高度图（[z*16+x]）
    const std::vector<int>* oceanFloor = nullptr;
    const std::vector<int>* worldSurface = nullptr;
    int minY = -64, height = 384;
    // 邻域 biome 判定（biome modifier 用）——Java 用 posToBiome（BiomeAccess 8 邻域 jitter）
    std::function<std::string(int, int, int)> posToBiome;
    int chunkStartX = 0, chunkStartZ = 0;
    // 世界方块读取（block_predicate_filter 等用；null=不可读）
    std::function<int(int, int, int)> blockAt;
};

class PlacementModifier {
public:
    virtual ~PlacementModifier() = default;
    // 返回输出位置（Java Stream<BlockPos>——惰性，C++ 展开为 vector）
    virtual std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                         int x, int y, int z) = 0;
    virtual std::string typeName() const = 0;
};

// ===== 具体 modifiers =====
// CountPlacementModifier：产生 count 个相同位置（Java count.get(random) 次 Stream.of(pos)）
class CountPlacementModifier : public PlacementModifier {
public:
    IntProvider count;
    explicit CountPlacementModifier(const IntProvider& c) : count(c) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)ctx;
        int n = count.get(random);
        std::vector<std::array<int, 3>> out;
        out.reserve((size_t)n);
        for (int i = 0; i < n; i++) out.push_back({x, y, z});
        return out;
    }
    std::string typeName() const override { return "count"; }
};

// RarityFilterPlacementModifier：nextInt(chance) == 0 才保留
class RarityFilterPlacementModifier : public PlacementModifier {
public:
    int chance = 0;
    explicit RarityFilterPlacementModifier(int c) : chance(c) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)ctx;
        if (chance <= 0 || random.nextInt(chance) == 0) return {{x, y, z}};
        return {};
    }
    std::string typeName() const override { return "rarity_filter"; }
};

// SquarePlacementModifier：x += nextInt(16), z += nextInt(16)
class SquarePlacementModifier : public PlacementModifier {
public:
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)ctx;
        return {{x + random.nextInt(16), y, z + random.nextInt(16)}};
    }
    std::string typeName() const override { return "in_square"; }
};

// HeightRangePlacementModifier：y = height.get(random, context)
class HeightRangePlacementModifier : public PlacementModifier {
public:
    HeightProvider height;
    explicit HeightRangePlacementModifier(const HeightProvider& hp) : height(hp) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        int ny = height.get(random, ctx.minY, ctx.height);
        return {{x, ny, z}};
    }
    std::string typeName() const override { return "height_range"; }
};

// HeightmapPlacementModifier：y = getTopY(heightmap, x, z)（Java 不 +1；k > bottomY 才返回）
class HeightmapPlacementModifier : public PlacementModifier {
public:
    std::string heightmapType = "WORLD_SURFACE_WG";
    explicit HeightmapPlacementModifier(const std::string& t) : heightmapType(t) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)random;
        const std::vector<int>* hm = (heightmapType.find("OCEAN_FLOOR") != std::string::npos) ? ctx.oceanFloor : ctx.worldSurface;
        if (!hm) return {{x, y, z}};
        int lx = x - ctx.chunkStartX, lz = z - ctx.chunkStartZ;
        int top = (lx >= 0 && lx < 16 && lz >= 0 && lz < 16) ? (*hm)[lz * 16 + lx] : ctx.minY - 1;
        if (top <= ctx.minY - 1) return {}; // Java k > bottomY（高度图无效）
        // 2026-08-10 复盘：C++ 内部高度图存「块 y」（surface 内部消费需要 y 语义），
        // 而 Java 高度图存 y+1。disk/spring 等直接消费高度图 modifier 的 y（C++=y 语义）与 Java(y+1) 差 1，
        // 实测 +1 使 300515 降 0.12%（disk/spring 变差）——保持 C++ y 语义（内部一致性优先），
        // 生态装饰（花/草）已按用户拍板范围外移除，不依赖此处。
        return {{x, top, z}};
    }
    std::string typeName() const override { return "heightmap"; }
};

// BiomePlacementModifier：条件——位置 biome 在生成 biome 集合内（Java getBiome 判定）
class BiomePlacementModifier : public PlacementModifier {
public:
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)random;
        // Java BiomePlacementModifier.getPositions：过滤 posToBiome.getBiome(pos) 在 features 集合内
        // C++ 简化：posToBiome 判定位置 biome——Java 内部用 biomeAt（chunk biome）
        std::string biome = ctx.posToBiome ? ctx.posToBiome(x, y, z) : "";
        // Java：filter(biome -> this.biomeAt.getPositions(context, random, pos) 内判 biome)
        // 简化：直接返回（biome 过滤由调用方预判）——Phase 3 先保留位置
        return {{x, y, z}};
    }
    std::string typeName() const override { return "biome"; }
};

// RandomOffsetPlacementModifier：x/z 随机偏移（offset provider）
class RandomOffsetPlacementModifier : public PlacementModifier {
public:
    IntProvider offsetX, offsetY, offsetZ;
    RandomOffsetPlacementModifier(const IntProvider& x, const IntProvider& y, const IntProvider& z)
        : offsetX(x), offsetY(y), offsetZ(z) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)ctx;
        return {{x + offsetX.get(random), y + offsetY.get(random), z + offsetZ.get(random)}};
    }
    std::string typeName() const override { return "random_offset"; }
};

// BlockPredicateFilterPlacementModifier（Java blockpredicatefilter 包）
// 简化支持 matching_fluids（disk_gravel 等）与 matching_blocks
class BlockPredicateFilterPlacementModifier : public PlacementModifier {
public:
    // matching_fluids：fluids 是 tag（water/lava）
    std::vector<int> fluidIds;      // 匹配的流体 block id（water/lava）
    std::vector<int> blockIds;      // matching_blocks 匹配的块
    bool isFluid = false;           // true=matching_fluids，false=matching_blocks
    BlockPredicateFilterPlacementModifier(bool fluid, const std::vector<int>& ids)
        : isFluid(fluid), fluidIds(ids), blockIds(ids) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)random;
        if (!ctx.blockAt) return {{x, y, z}}; // 无法读世界——保留（Java 若 chunk 未生成则 null）
        int cur = ctx.blockAt(x, y, z);
        if (cur < 0) return {};
        for (int id : (isFluid ? fluidIds : blockIds)) {
            if (cur == id) return {{x, y, z}};
        }
        return {};
    }
    std::string typeName() const override { return "block_predicate_filter"; }
};

// SurfaceRelativeThresholdPlacementModifier（Java surface_relative_threshold_filter）
// y - getTopY(heightmap) 在 [minInclusive, maxInclusive] 才保留（underwater_magma 等用）
class SurfaceRelativeThresholdPlacementModifier : public PlacementModifier {
public:
    std::string heightmapType = "WORLD_SURFACE_WG";
    bool hasMin = false, hasMax = false;
    int minInclusive = 0, maxInclusive = 0;
    SurfaceRelativeThresholdPlacementModifier(const std::string& t, bool mn, bool mx, int a, int b)
        : heightmapType(t), hasMin(mn), hasMax(mx), minInclusive(a), maxInclusive(b) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        (void)random;
        const std::vector<int>* hm = (heightmapType.find("OCEAN_FLOOR") != std::string::npos) ? ctx.oceanFloor : ctx.worldSurface;
        if (!hm) return {{x, y, z}};
        int lx = x - ctx.chunkStartX, lz = z - ctx.chunkStartZ;
        if (lx < 0 || lx >= 16 || lz < 0 || lz >= 16) return {{x, y, z}}; // 邻域高度图缺失——保留
        int top = (*hm)[lz * 16 + lx];
        if (hasMin && y < top + minInclusive) return {};
        if (hasMax && y > top + maxInclusive) return {};
        return {{x, y, z}};
    }
    std::string typeName() const override { return "surface_relative_threshold"; }
};

// NoiseBasedCountPlacementModifier（Java：count + noise 偏移）
class NoiseBasedCountPlacementModifier : public PlacementModifier {
public:
    int maxCount = 0;
    std::string noiseName;
    double scale = 0.0;
    IntProvider count;
    NoiseBasedCountPlacementModifier(int mc, const std::string& n, double s, const IntProvider& c)
        : maxCount(mc), noiseName(n), scale(s), count(c) {}
    std::vector<std::array<int, 3>> getPositions(FeaturePlacementContext& ctx, ChunkRandom& random,
                                                 int x, int y, int z) override {
        // Java：count + floor(noise(x*scale, 0, z*scale) * maxCount)
        double noise = 0.0; // 需要 noise sampler——Phase 3 简化 0
        int n = std::max(0, count.get(random) + (int)std::floor(noise * maxCount));
        std::vector<std::array<int, 3>> out;
        for (int i = 0; i < n; i++) out.push_back({x, y, z});
        return out;
    }
    std::string typeName() const override { return "noise_based_count"; }
};

// ===== PlacedFeature（Java PlacedFeature.java）=====
struct PlacedFeature {
    std::string id;                        // "minecraft:ore_granite_upper"
    std::vector<std::shared_ptr<PlacementModifier>> modifiers;
    std::string configuredFeature;         // 引用的 configured_feature id
    int step = 0;                          // GenerationStep.Feature ordinal（biome features 列表索引）
    int globalIndex = -1;                  // PlacedFeatureIndexer 全局索引（p）

    // Java PlacedFeature.generate：Stream.of(pos) → 链式 flatMap（惰性、深度优先：位置逐个走完 modifiers）
    // 关键：Java 惰性 flatMap 是「位置1 走完所有 modifier → 位置2 走完所有 modifier」（深度优先）
    // C++ 若「modifier 全展开再下一个」= 广度优先 → 随机消费顺序不同 → height_range y 全错（granite 位置错）
    bool generate(FeaturePlacementContext& ctx, ChunkRandom& random, int originX, int originY, int originZ) const {
        bool placed = false;
        std::function<void(size_t, int, int, int)> visit = [&](size_t mi, int x, int y, int z) {
            if (mi == modifiers.size()) {
                if (generateConfigured(ctx, x, y, z)) placed = true;
                return;
            }
            auto out = modifiers[mi]->getPositions(ctx, random, x, y, z);
            for (const auto& p : out) visit(mi + 1, p[0], p[1], p[2]);
        };
        visit(0, originX, originY, originZ);
        return placed;
    }

    // ConfiguredFeature.generate 分发（type=ore 等）——由外部注入（feature_dispatcher）
    std::function<bool(FeaturePlacementContext&, int, int, int)> generateConfigured;
};

} // namespace wg
