// surface.h — MaterialRules 引擎复刻 + VanillaSurfaceRules 翻译 + SurfaceBuilder
// 对应 Java: MaterialRules.java / VanillaSurfaceRules.java / SurfaceBuilder.java
#pragma once
#include <cmath>
#include <cstdint>
#include <functional>
#include <map>
#include <memory>
#include <set>
#include <string>
#include <vector>

#include "blocks.h"
#include "noise.h"
#include "xoroshiro.h"

namespace wg {

inline double lerpClamp(double value, double fromStart, double fromEnd, double toStart, double toEnd) {
    double t = (value - fromStart) / (fromEnd - fromStart);
    t = t < 0 ? 0 : (t > 1 ? 1 : t);
    return toStart + t * (toEnd - toStart);
}

class SurfaceContext;
struct SurfaceCond;
struct SurfaceRule;

using CondP = std::shared_ptr<SurfaceCond>;
using RuleP = std::shared_ptr<SurfaceRule>;

// ========== 条件 / 规则接口 ==========
struct SurfaceCond {
    virtual ~SurfaceCond() = default;
    virtual bool test(const SurfaceContext& ctx) const = 0;
};
struct SurfaceRule {
    virtual ~SurfaceRule() = default;
    virtual int apply(const SurfaceContext& ctx) const = 0; // BlockId，不适用返回 -1
};

// ---- 规则实现 ----
struct BlockRule : SurfaceRule {
    int block;
    explicit BlockRule(int b) : block(b) {}
    int apply(const SurfaceContext&) const override { return block; }
};
struct CondRule : SurfaceRule {
    CondP cond;
    RuleP rule;
    CondRule(CondP c, RuleP r) : cond(std::move(c)), rule(std::move(r)) {}
    int apply(const SurfaceContext& ctx) const override {
        return cond->test(ctx) ? rule->apply(ctx) : -1;
    }
};
struct SeqRule : SurfaceRule {
    std::vector<RuleP> rules;
    explicit SeqRule(std::vector<RuleP> rs) : rules(std::move(rs)) {}
    int apply(const SurfaceContext& ctx) const override {
        for (auto& r : rules) {
            int b = r->apply(ctx);
            if (b >= 0) return b;
        }
        return -1;
    }
};
struct TerracottaBandsRule : SurfaceRule {
    int apply(const SurfaceContext& ctx) const override;
};

// ---- 条件实现 ----
struct BiomeCond : SurfaceCond {
    std::set<std::string> biomes;
    bool test(const SurfaceContext& ctx) const override;
};
struct AboveYCond : SurfaceCond {
    int anchorY, mult;
    bool addStoneDepth;
    bool test(const SurfaceContext& ctx) const override;
};
struct WaterCond : SurfaceCond {
    int offset, mult;
    bool addStoneDepth;
    bool test(const SurfaceContext& ctx) const override;
};
struct StoneDepthCond : SurfaceCond {
    int offset;
    bool addSurfaceDepth;
    int secondaryDepthRange;
    bool ceiling;
    bool test(const SurfaceContext& ctx) const override;
};
struct NoiseThresholdCond : SurfaceCond {
    std::string noiseKey;
    double minTh, maxTh;
    bool test(const SurfaceContext& ctx) const override;
};
struct HoleCond : SurfaceCond {
    bool test(const SurfaceContext& ctx) const override;
};
struct SteepCond : SurfaceCond {
    bool test(const SurfaceContext& ctx) const override;
};
struct SurfaceCondC : SurfaceCond { // above_preliminary_surface
    bool test(const SurfaceContext& ctx) const override;
};
struct TempCond : SurfaceCond { // temperature()
    bool test(const SurfaceContext& ctx) const override;
};
struct VerticalGradientCond : SurfaceCond {
    std::string name;
    int trueY, falseY;
    bool test(const SurfaceContext& ctx) const override;
};
struct NotCond : SurfaceCond {
    CondP inner;
    explicit NotCond(CondP c) : inner(std::move(c)) {}
    bool test(const SurfaceContext& ctx) const override { return !inner->test(ctx); }
};

// ========== SurfaceContext（MaterialRuleContext）==========
class SurfaceContext {
public:
    int blockX = 0, blockY = 0, blockZ = 0;
    int runDepth = 0;
    int stoneDepthAbove = 0, stoneDepthBelow = 0;
    int fluidHeight = INT32_MIN;
    std::string biomeId;
    double biomeTemp = 0.5;      // biome 温度（用于 temperature()）
    int terracottaBlock = 0;     // 由 SurfaceBuilder.getTerracottaBlock 预计算

    std::map<std::string, DoublePerlinNoiseSampler>* noiseSamplers = nullptr;
    const XoroshiroRandom::Splitter* splitter = nullptr;
    std::function<int(int, int, int)> terracottaBandsGetter; // (x,y,z) → 红陶带方块
    const std::vector<int>* columnHeightmap = nullptr; // [256] WORLD_SURFACE_WG
    const std::vector<int>* surfaceHeights4 = nullptr; // chunk 4 角 estimateSurfaceHeight
    const DoublePerlinNoiseSampler* surfaceSecondaryNoise = nullptr;
    mutable int64_t secondaryCacheKey = INT64_MIN;
    mutable double secondaryCache = 0;

    double getSecondaryDepth() const {
        int64_t key = ((int64_t)(uint32_t)blockX << 32) ^ (uint32_t)blockZ;
        if (key != secondaryCacheKey) {
            secondaryCacheKey = key;
            secondaryCache = surfaceSecondaryNoise->sample(blockX, 0.0, blockZ);
        }
        return secondaryCache;
    }

    int estimateSurfaceHeight() const {
        double fx = (blockX & 15) / 16.0;
        double fz = (blockZ & 15) / 16.0;
        double v = lerp2(fx, fz, (*surfaceHeights4)[0], (*surfaceHeights4)[1],
                         (*surfaceHeights4)[2], (*surfaceHeights4)[3]);
        return (int)std::floor(v) + runDepth - 8;
    }

    void initVertical(int stoneDepthAbove, int stoneDepthBelow, int fluidHeight,
                      int x, int y, int z, const std::string& biome) {
        this->stoneDepthAbove = stoneDepthAbove;
        this->stoneDepthBelow = stoneDepthBelow;
        this->fluidHeight = fluidHeight;
        this->blockX = x; this->blockY = y; this->blockZ = z;
        this->biomeId = biome;
    }

    static double lerp2(double fx, double fz, double a, double b, double c, double d) {
        return a + (b - a) * fx + (c - a) * fz + (a - b - c + d) * fx * fz;
    }
};

// ========== 条件实现 ==========
inline bool BiomeCond::test(const SurfaceContext& ctx) const { return biomes.count(ctx.biomeId) > 0; }
inline bool AboveYCond::test(const SurfaceContext& ctx) const {
    int y = ctx.blockY + (addStoneDepth ? ctx.stoneDepthAbove : 0);
    return y >= anchorY + ctx.runDepth * mult;
}
inline bool WaterCond::test(const SurfaceContext& ctx) const {
    if (ctx.fluidHeight == INT32_MIN) return true;
    int y = ctx.blockY + (addStoneDepth ? ctx.stoneDepthAbove : 0);
    return y >= ctx.fluidHeight + offset + ctx.runDepth * mult;
}
inline bool StoneDepthCond::test(const SurfaceContext& ctx) const {
    int i = ceiling ? ctx.stoneDepthBelow : ctx.stoneDepthAbove;
    int j = addSurfaceDepth ? ctx.runDepth : 0;
    int k = secondaryDepthRange == 0 ? 0
        : (int)std::floor(lerpClamp(ctx.getSecondaryDepth(), -1.0, 1.0, 0.0, (double)secondaryDepthRange));
    return i <= 1 + offset + j + k;
}
inline bool NoiseThresholdCond::test(const SurfaceContext& ctx) const {
    auto it = ctx.noiseSamplers->find(noiseKey);
    if (it == ctx.noiseSamplers->end()) return false;
    double d = it->second.sample(ctx.blockX, 0.0, ctx.blockZ);
    return d >= minTh && d <= maxTh;
}
inline bool HoleCond::test(const SurfaceContext& ctx) const { return ctx.runDepth <= 0; }
inline bool SteepCond::test(const SurfaceContext& ctx) const {
    int i = ctx.blockX & 15, j = ctx.blockZ & 15;
    int m = (*ctx.columnHeightmap)[i * 16 + std::max(j - 1, 0)];
    int n = (*ctx.columnHeightmap)[i * 16 + std::min(j + 1, 15)];
    if (n >= m + 4) return true;
    int o = std::max(i - 1, 0), p = std::min(i + 1, 15);
    int q = (*ctx.columnHeightmap)[o * 16 + j];
    int r = (*ctx.columnHeightmap)[p * 16 + j];
    return q >= r + 4;
}
inline bool SurfaceCondC::test(const SurfaceContext& ctx) const {
    return ctx.blockY >= ctx.estimateSurfaceHeight();
}
inline bool TempCond::test(const SurfaceContext& ctx) const { return ctx.biomeTemp < 0.15; }
inline bool VerticalGradientCond::test(const SurfaceContext& ctx) const {
    int y = ctx.blockY;
    if (y <= trueY) return true;
    if (y >= falseY) return false;
    double d = lerpClamp((double)y, (double)trueY, (double)falseY, 1.0, 0.0);
    XoroshiroRandom r = ctx.splitter->split(ctx.blockX, y, ctx.blockZ);
    return r.nextFloat() < d;
}
inline int TerracottaBandsRule::apply(const SurfaceContext& ctx) const {
    return ctx.terracottaBandsGetter ? ctx.terracottaBandsGetter(ctx.blockX, ctx.blockY, ctx.blockZ) : 0;
}

// ========== 便捷构造（对应 MaterialRules 静态方法）==========
inline RuleP blockRule(int b) { return std::make_shared<BlockRule>(b); }
inline RuleP condition(CondP c, RuleP r) { return std::make_shared<CondRule>(std::move(c), std::move(r)); }
inline RuleP sequence(std::vector<RuleP> rs) { return std::make_shared<SeqRule>(std::move(rs)); }
inline CondP biomeCond(std::set<std::string> keys) {
    auto c = std::make_shared<BiomeCond>();
    c->biomes = std::move(keys);
    return c;
}
inline CondP aboveY(int anchor, int mult, bool addStoneDepth) {
    auto c = std::make_shared<AboveYCond>();
    c->anchorY = anchor; c->mult = mult; c->addStoneDepth = addStoneDepth;
    return c;
}
inline CondP waterCond(int offset, int mult, bool addStoneDepth) {
    auto c = std::make_shared<WaterCond>();
    c->offset = offset; c->mult = mult; c->addStoneDepth = addStoneDepth;
    return c;
}
inline CondP stoneDepth(int offset, bool addSurfaceDepth, int secondaryRange, bool ceiling) {
    auto c = std::make_shared<StoneDepthCond>();
    c->offset = offset; c->addSurfaceDepth = addSurfaceDepth;
    c->secondaryDepthRange = secondaryRange; c->ceiling = ceiling;
    return c;
}
inline CondP noiseThreshold(const std::string& key, double min, double max) {
    auto c = std::make_shared<NoiseThresholdCond>();
    c->noiseKey = key; c->minTh = min; c->maxTh = max;
    return c;
}
inline CondP noiseThresholdNoMax(const std::string& key, double min) {
    auto c = std::make_shared<NoiseThresholdCond>();
    c->noiseKey = key; c->minTh = min; c->maxTh = 1e300;
    return c;
}
inline CondP holeCond() { return std::make_shared<HoleCond>(); }
inline CondP steepCond() { return std::make_shared<SteepCond>(); }
inline CondP surfaceCondC() { return std::make_shared<SurfaceCondC>(); }
inline CondP tempCond() { return std::make_shared<TempCond>(); }
inline CondP verticalGradient(const std::string& name, int trueY, int falseY) {
    auto c = std::make_shared<VerticalGradientCond>();
    c->name = name; c->trueY = trueY; c->falseY = falseY;
    return c;
}
inline CondP notCond(CondP c) { return std::make_shared<NotCond>(std::move(c)); }

// ========== SurfaceBuilder ==========
class SurfaceBuilder {
public:
    SurfaceBuilder(std::map<std::string, DoublePerlinNoiseSampler>* samplers,
                   const XoroshiroRandom::Splitter* splitter,
                   int seaLevel, const BlockRegistry* blocks,
                   const std::string& biomeDir)
        : samplers(samplers), splitter(splitter), seaLevel(seaLevel), blocks(blocks) {
        // clay_bands random：预生成 192 长度红陶带数组
        XoroshiroRandom bandRandom = splitter->split("minecraft:clay_bands");
        terracottaBands.assign(192, blocks->id("minecraft:terracotta"));
        int orange = blocks->id("minecraft:orange_terracotta");
        int yellow = blocks->id("minecraft:yellow_terracotta");
        int brown = blocks->id("minecraft:brown_terracotta");
        int red = blocks->id("minecraft:red_terracotta");
        int white = blocks->id("minecraft:white_terracotta");
        int lightGray = blocks->id("minecraft:light_gray_terracotta");
        for (int i = 0; i < 192; i++) {
            i += bandRandom.nextInt(5) + 1;
            if (i < 192) terracottaBands[i] = orange;
        }
        addTerracottaBand(bandRandom, terracottaBands, 1, yellow);
        addTerracottaBand(bandRandom, terracottaBands, 2, brown);
        addTerracottaBand(bandRandom, terracottaBands, 1, red);
        int ix = bandRandom.nextBetween(9, 15);
        int j = 0;
        for (int k = 0; j < ix && k < 192; k += bandRandom.nextInt(16) + 4) {
            terracottaBands[k] = white;
            if (k - 1 > 0 && bandRandom.nextBoolean()) terracottaBands[k - 1] = lightGray;
            if (k + 1 < 192 && bandRandom.nextBoolean()) terracottaBands[k + 1] = lightGray;
            j++;
        }
    }

    int sampleRunDepth(int blockX, int blockZ) {
        double d = getNoise("minecraft:surface").sample(blockX, 0.0, blockZ);
        double extra = splitter->split(blockX, 0, blockZ).nextDouble();
        return (int)(d * 2.75 + 3.0 + extra * 0.25);
    }
    double sampleSecondaryDepth(int blockX, int blockZ) {
        return getNoise("minecraft:surface_secondary").sample(blockX, 0.0, blockZ);
    }
    // getTerracottaBlock(x, y, z)：按 y 索引红陶带
    int getTerracottaBlock(int x, int y, int z) {
        double d = getNoise("minecraft:clay_bands_offset").sample(x, 0.0, z) * 4.0;
        int i = (int)std::lround(d);
        int n = (int)terracottaBands.size();
        return terracottaBands[((y + i) % n + n) % n];
    }

    // 主世界规则树：VanillaSurfaceRules.createDefaultRule(true, false, true)
    RuleP buildOverworldRule();

    // buildSurface 引擎：对已生成的 BlockColumn 应用规则
    void buildSurface(BlockColumn& col, const RuleP& rule,
                      int chunkStartX, int chunkStartZ,
                      const std::vector<int>& heightmap,
                      const std::vector<int>& surfaceHeights4,
                      const std::function<std::string(int, int, int)>& biomeAt,
                      const std::function<double(const std::string&)>& biomeTemp);

    static void addTerracottaBand(XoroshiroRandom& r, std::vector<int>& bands, int minBandSize, int state) {
        int i = r.nextBetween(6, 15);
        for (int j = 0; j < i; j++) {
            int k = minBandSize + r.nextInt(3);
            int l = r.nextInt((int)bands.size());
            for (int m = 0; l + m < (int)bands.size() && m < k; m++) bands[l + m] = state;
        }
    }

private:
    std::map<std::string, DoublePerlinNoiseSampler>* samplers;
    const XoroshiroRandom::Splitter* splitter;
    int seaLevel;
    const BlockRegistry* blocks;
    std::vector<int> terracottaBands;

    DoublePerlinNoiseSampler& getNoise(const std::string& key) { return (*samplers)[key]; }

    friend class TerracottaBandsRule;
};

// ========== 主世界规则树翻译（VanillaSurfaceRules.createDefaultRule(true,false,true)）==========
inline RuleP SurfaceBuilder::buildOverworldRule() {
    auto B = [this](const char* name) { return blockRule(blocks->id(std::string("minecraft:") + name)); };
    const int AIR = blocks->id("minecraft:air");

    // materialCondition 1..13
    CondP mc1 = aboveY(97, 2, false);
    CondP mc2 = aboveY(256, 0, false);
    CondP mc3 = aboveY(63, -1, true);   // aboveYWithStoneDepth(fixed(63), -1)
    CondP mc4 = aboveY(74, 1, true);
    CondP mc5 = aboveY(60, 0, false);
    CondP mc6 = aboveY(62, 0, false);
    CondP mc7 = aboveY(63, 0, false);
    CondP mc8 = waterCond(-1, 0, false);
    CondP mc9 = waterCond(0, 0, false);
    CondP mc10 = waterCond(-6, -1, true); // waterWithStoneDepth(-6, -1)
    CondP mc11 = holeCond();
    CondP mc12 = biomeCond({"minecraft:frozen_ocean", "minecraft:deep_frozen_ocean"});
    CondP mc13 = steepCond();

    // materialRule
    RuleP mr = sequence({condition(mc9, B("grass_block")), B("dirt")});
    RuleP mr2 = sequence({condition(stoneDepth(0, false, 0, false), B("sandstone")), B("sand")}); // STONE_DEPTH_CEILING
    RuleP mr3 = sequence({condition(stoneDepth(0, false, 0, false), B("stone")), B("gravel")});  // STONE_DEPTH_CEILING

    CondP mc14 = biomeCond({"minecraft:warm_ocean", "minecraft:beach", "minecraft:snowy_beach"});
    CondP mc15 = biomeCond({"minecraft:desert"});

    // materialRule4
    RuleP mr4 = sequence({
        condition(biomeCond({"minecraft:stony_peaks"}),
            sequence({condition(noiseThreshold("minecraft:calcite", -0.0125, 0.0125), B("calcite")), B("stone")})),
        condition(biomeCond({"minecraft:stony_shore"}),
            sequence({condition(noiseThreshold("minecraft:gravel", -0.05, 0.05), mr3), B("stone")})),
        condition(biomeCond({"minecraft:windswept_hills"}), condition(noiseThresholdNoMax("minecraft:surface", 1.0), B("stone"))),
        condition(mc14, mr2),
        condition(mc15, mr2),
        condition(biomeCond({"minecraft:dripstone_caves"}), B("stone")),
    });

    RuleP mr5 = condition(noiseThreshold("minecraft:powder_snow", 0.45, 0.58), condition(mc9, B("powder_snow")));
    RuleP mr6 = condition(noiseThreshold("minecraft:powder_snow", 0.35, 0.6), condition(mc9, B("powder_snow")));

    RuleP mr7 = sequence({
        condition(biomeCond({"minecraft:frozen_peaks"}),
            sequence({
                condition(mc13, B("packed_ice")),
                condition(noiseThreshold("minecraft:packed_ice", -0.5, 0.2), B("packed_ice")),
                condition(noiseThreshold("minecraft:ice", -0.0625, 0.025), B("ice")),
                condition(mc9, B("snow_block")),
            })),
        condition(biomeCond({"minecraft:snowy_slopes"}),
            sequence({condition(mc13, B("stone")), mr5, condition(mc9, B("snow_block"))})),
        condition(biomeCond({"minecraft:jagged_peaks"}), B("stone")),
        condition(biomeCond({"minecraft:grove"}), sequence({mr5, B("dirt")})),
        mr4,
        condition(biomeCond({"minecraft:windswept_savanna"}), condition(noiseThresholdNoMax("minecraft:surface", 1.75), B("stone"))),
        condition(biomeCond({"minecraft:windswept_gravelly_hills"}),
            sequence({
                condition(noiseThresholdNoMax("minecraft:surface", 2.0), mr3),
                condition(noiseThresholdNoMax("minecraft:surface", 1.0), B("stone")),
                condition(noiseThresholdNoMax("minecraft:surface", -1.0), B("dirt")),
                mr3,
            })),
        condition(biomeCond({"minecraft:old_growth_pine_taiga", "minecraft:old_growth_spruce_taiga"}),
            sequence({
                condition(noiseThresholdNoMax("minecraft:surface", 1.75), B("coarse_dirt")),
                condition(noiseThresholdNoMax("minecraft:surface", -0.95), B("podzol")),
            })),
        condition(biomeCond({"minecraft:ice_spikes"}), condition(mc9, B("snow_block"))),
        condition(biomeCond({"minecraft:mangrove_swamp"}), B("mud")),
        condition(biomeCond({"minecraft:mushroom_fields"}), B("mycelium")),
        mr,
    });

    CondP mc16 = noiseThreshold("minecraft:surface", -0.909, -0.5454);
    CondP mc17 = noiseThreshold("minecraft:surface", -0.1818, 0.1818);
    CondP mc18 = noiseThreshold("minecraft:surface", 0.5454, 0.909);

    // materialRule8（海洋段使用，阈值与 mr7 不同）
    RuleP mr8 = sequence({
        condition(biomeCond({"minecraft:frozen_peaks"}),
            sequence({
                condition(mc13, B("packed_ice")),
                condition(noiseThreshold("minecraft:packed_ice", 0.0, 0.2), B("packed_ice")),
                condition(noiseThreshold("minecraft:ice", 0.0, 0.025), B("ice")),
                condition(mc9, B("snow_block")),
            })),
        condition(biomeCond({"minecraft:snowy_slopes"}),
            sequence({condition(mc13, B("stone")), mr6, condition(mc9, B("snow_block"))})),
        condition(biomeCond({"minecraft:jagged_peaks"}),
            sequence({condition(mc13, B("stone")), condition(mc9, B("snow_block"))})),
        condition(biomeCond({"minecraft:grove"}), sequence({mr6, condition(mc9, B("snow_block"))})),
        mr4,
        condition(biomeCond({"minecraft:windswept_savanna"}),
            sequence({
                condition(noiseThresholdNoMax("minecraft:surface", 1.75), B("stone")),
                condition(noiseThresholdNoMax("minecraft:surface", -0.5), B("coarse_dirt")),
            })),
        condition(biomeCond({"minecraft:windswept_gravelly_hills"}),
            sequence({
                condition(noiseThresholdNoMax("minecraft:surface", 2.0), mr3),
                condition(noiseThresholdNoMax("minecraft:surface", 1.0), B("stone")),
                condition(noiseThresholdNoMax("minecraft:surface", -1.0), mr),
                mr3,
            })),
        condition(biomeCond({"minecraft:old_growth_pine_taiga", "minecraft:old_growth_spruce_taiga"}),
            sequence({
                condition(noiseThresholdNoMax("minecraft:surface", 1.75), B("coarse_dirt")),
                condition(noiseThresholdNoMax("minecraft:surface", -0.95), B("podzol")),
            })),
        condition(biomeCond({"minecraft:ice_spikes"}), condition(mc9, B("snow_block"))),
        condition(biomeCond({"minecraft:mangrove_swamp"}), B("mud")),
        condition(biomeCond({"minecraft:mushroom_fields"}), B("mycelium")),
        mr,
    });

    // 红陶带规则（terracottaBands 需 (x,y,z)）
    RuleP bandsRule = std::make_shared<TerracottaBandsRule>();

    RuleP mr9 = sequence({
        // STONE_DEPTH_FLOOR 段
        condition(stoneDepth(0, false, 0, false),
            sequence({
                condition(biomeCond({"minecraft:wooded_badlands"}),
                    condition(mc1,
                        sequence({
                            condition(mc16, B("coarse_dirt")),
                            condition(mc17, B("coarse_dirt")),
                            condition(mc18, B("coarse_dirt")),
                            mr,
                        }))),
                condition(biomeCond({"minecraft:swamp"}),
                    condition(mc6,
                        condition(notCond(mc7),
                            condition(noiseThresholdNoMax("minecraft:surface_swamp", 0.0), B("water"))))),
                condition(biomeCond({"minecraft:mangrove_swamp"}),
                    condition(mc5,
                        condition(notCond(mc7),
                            condition(noiseThresholdNoMax("minecraft:surface_swamp", 0.0), B("water"))))),
            })),
        // badlands 段
        condition(biomeCond({"minecraft:badlands", "minecraft:eroded_badlands", "minecraft:wooded_badlands"}),
            sequence({
                condition(stoneDepth(0, false, 0, false),
                    sequence({
                        condition(mc2, B("orange_terracotta")),
                        condition(mc4,
                            sequence({
                                condition(mc16, B("terracotta")),
                                condition(mc17, B("terracotta")),
                                condition(mc18, B("terracotta")),
                                bandsRule,
                            })),
                        condition(mc8, sequence({condition(stoneDepth(0, false, 0, false), B("red_sandstone")), B("red_sand")})),
                        condition(notCond(mc11), B("orange_terracotta")),
                        condition(mc10, B("white_terracotta")),
                        mr3,
                    })),
                condition(mc3,
                    sequence({
                        condition(mc7, condition(notCond(mc4), B("orange_terracotta"))),
                        bandsRule,
                    })),
                condition(stoneDepth(0, true, 0, false), condition(mc10, B("white_terracotta"))), // STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH
            })),
        // 海洋段
        condition(stoneDepth(0, false, 0, false),
            condition(mc8,
                sequence({
                    condition(mc12,
                        condition(mc11,
                            sequence({
                                condition(mc9, blockRule(AIR)),
                                condition(tempCond(), B("ice")),
                                B("water"),
                            }))),
                    mr8, // 见下
                }))),
        condition(mc10,
            sequence({
                condition(stoneDepth(0, false, 0, false), condition(mc12, condition(mc11, B("water")))),
                condition(stoneDepth(0, true, 0, false), mr7), // STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH
                condition(mc14, condition(stoneDepth(0, true, 6, false), B("sandstone"))), // RANGE_6
                condition(mc15, condition(stoneDepth(0, true, 30, false), B("sandstone"))), // RANGE_30
            })),
        condition(stoneDepth(0, false, 0, false),
            sequence({
                condition(biomeCond({"minecraft:frozen_peaks", "minecraft:jagged_peaks"}), B("stone")),
                condition(biomeCond({"minecraft:warm_ocean", "minecraft:lukewarm_ocean", "minecraft:deep_lukewarm_ocean"}), mr2),
                mr3,
            })),
    });

    // 最终序列：bedrock_roof(false) + bedrock_floor(true) + surface(materialRule9) + deepslate
    std::vector<RuleP> finalRules;
    // bedrockFloor
    finalRules.push_back(condition(verticalGradient("minecraft:bedrock_floor", -64, -59), B("bedrock")));
    // surface → materialRule9（surface=true）
    finalRules.push_back(condition(surfaceCondC(), mr9));
    // deepslate：verticalGradient("deepslate", fixed(0), fixed(8))
    finalRules.push_back(condition(verticalGradient("minecraft:deepslate", 0, 8), B("deepslate")));
    return sequence(std::move(finalRules));
}

// ========== buildSurface 引擎（对应 SurfaceBuilder.buildSurface）==========
inline void SurfaceBuilder::buildSurface(BlockColumn& col,
                                         const RuleP& rule,
                                         int chunkStartX, int chunkStartZ,
                                         const std::vector<int>& heightmap,
                                         const std::vector<int>& surfaceHeights4,
                                         const std::function<std::string(int, int, int)>& biomeAt,
                                         const std::function<double(const std::string&)>& biomeTemp) {
    SurfaceContext ctx;
    ctx.noiseSamplers = samplers;
    ctx.splitter = splitter;
    ctx.columnHeightmap = &heightmap;
    ctx.surfaceHeights4 = &surfaceHeights4;
    ctx.surfaceSecondaryNoise = &getNoise("minecraft:surface_secondary");
    ctx.terracottaBandsGetter = [this](int x, int y, int z) { return getTerracottaBlock(x, y, z); };

    const int defaultBlock = blocks->id("minecraft:stone");
    const int airBlock = blocks->id("minecraft:air");
    const int waterBlock = blocks->id("minecraft:water");
    const int lavaBlock = blocks->id("minecraft:lava");
    const int minY = BLOCK_MIN_Y;

    // biome 缓存：按 4×4×4 块粒度（biome coords），packed key
    std::map<int64_t, std::pair<std::string, double>> biomeCache;
    auto biomeAtCached = [&](int bx, int by, int bz) -> std::pair<std::string, double> {
        int64_t key = ((int64_t)(uint32_t)(bx >> 2) << 40) | ((int64_t)(uint32_t)(by >> 2) << 20) | (uint32_t)(bz >> 2);
        auto it = biomeCache.find(key);
        if (it != biomeCache.end()) return it->second;
        std::string id = biomeAt(bx, by, bz);
        double t = biomeTemp(id);
        auto r = std::make_pair(id, t);
        biomeCache[key] = r;
        return r;
    };

    for (int k = 0; k < 16; k++) {
        for (int l = 0; l < 16; l++) {
            int m = chunkStartX + k, n = chunkStartZ + l; // 世界坐标
            int p = heightmap[k * 16 + l] + 1; // WORLD_SURFACE_WG + 1（chunk 内 y）
            ctx.blockX = m;
            ctx.blockZ = n;
            ctx.runDepth = sampleRunDepth(m, n);

            int q = 0;
            int r = INT32_MIN;   // 最高流体 y + 1
            int s = INT32_MAX;   // 第一个非 default 块位置
            int u = p;

            for (int wy = p; wy >= minY; wy--) {
                int state;
                if (wy >= BLOCK_MIN_Y + BLOCK_HEIGHT) {
                    state = airBlock; // 世界高度以上视为空气（vanilla HeightLimitView 越界返回 AIR）
                } else {
                    state = col.at(k, wy, l);
                }
                bool isAir = (state == airBlock);
                bool isFluid = (state == waterBlock || state == lavaBlock);
                if (isAir) {
                    q = 0;
                    r = INT32_MIN;
                } else if (isFluid) {
                    if (r == INT32_MIN) r = wy + 1;
                } else {
                    if (s >= wy) {
                        s = INT32_MAX;
                        for (int v = wy - 1; v >= minY - 1; v--) {
                            int st2;
                            if (v < BLOCK_MIN_Y) {
                                st2 = airBlock; // 世界底以下视为空气
                            } else {
                                st2 = col.at(k, v, l);
                            }
                            if (st2 != airBlock && st2 != waterBlock && st2 != lavaBlock) {
                                // 找到 default 块 → 继续向上找非 default
                                continue;
                            }
                            s = v + 1;
                            break;
                        }
                    }
                    q++;
                    int vx = wy - s + 1;
                    auto b = biomeAtCached(m, wy, n);
                    ctx.initVertical(q, vx, r, m, wy, n, b.first);
                    ctx.biomeTemp = b.second;
                    if (state == defaultBlock) {
                        int newState = rule->apply(ctx);
                        if (newState >= 0) col.at(k, wy, l) = newState;
                    }
                }
            }
        }
    }
}

} // namespace wg
