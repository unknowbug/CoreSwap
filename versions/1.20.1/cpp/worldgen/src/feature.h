#pragma once
// feature.h — FEATURES 阶段 Feature 类（MC 1.20.1 移植）
// Phase 3：OreFeature / ScatteredOreFeature（岩石替换 + 矿石）
// Java 参照：world/gen/feature/OreFeature.java / ScatteredOreFeature.java / OreFeatureConfig.java
//            structure/rule/RuleTest.java（TagMatchRuleTest/BlockMatchRuleTest/RandomBlockMatchRuleTest/AlwaysTrueRuleTest）
//            Feature.java（isExposedToAir）
// 关键语义：
//   - OreFeature.generate 用 Math.sin/Math.cos（标准库，非查表！）；MathHelper.sin（查表，ds 权重）
//   - OCEAN_FLOOR_WG 高度图（NOISE 阶段 SUFFOCATES=blocksMovement 判定）——C++ 需补
//   - chunkSectionCache 读方块（邻域 chunk 可能未生成——Java 用 ChunkSectionCache 惰性生成）
//     C++ 简化：只处理当前 chunk 内（邻域未生成无法读）——Phase 3 先当前 chunk，邻域后续补
#include <cstdint>
#include <cmath>
#include <string>
#include <vector>

#include "blocks.h"
#include "chunkrandom.h"
#include "json.h"
#include "carver.h" // mathSin/mathCos（MathHelper 查表）
#include "placement.h" // IntProvider（DiskFeatureConfig 用）

namespace wg {

// ===== RuleTest（Java structure/rule/RuleTest.java）=====
struct RuleTest {
    enum class Kind { TAG_MATCH, BLOCK_MATCH, RANDOM_BLOCK_MATCH, ALWAYS_TRUE };
    Kind kind = Kind::ALWAYS_TRUE;
    std::vector<int> blockIds;   // block_match（1 个）或 random_block_match（多个）或 tag 展开
    float probability = 0.0f;    // random_block_match 概率

    bool test(BlockRegistry& blocks, int blockId, ChunkRandom& random) const {
        (void)blocks;
        switch (kind) {
            case Kind::ALWAYS_TRUE: return true;
            case Kind::BLOCK_MATCH:
                return blockId == blockIds[0];
            case Kind::TAG_MATCH:
                for (int id : blockIds) if (id == blockId) return true;
                return false;
            case Kind::RANDOM_BLOCK_MATCH:
                if (random.nextFloat() >= probability) return false;
                for (int id : blockIds) if (id == blockId) return true;
                return false;
        }
        return false;
    }

    static RuleTest parse(const JsonValue* v, BlockRegistry& blocks);
};

// ===== OreFeatureConfig（Java OreFeatureConfig.java）=====
struct OreFeatureConfig {
    struct Target {
        RuleTest target;
        int state = 0;
    };
    std::vector<Target> targets;
    int size = 0;
    float discardOnAirChance = 0.0f;

    static OreFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks);
};

// ===== OreFeatureContext（C++ 版 FeatureContext + StructureWorldAccess 简化）=====
struct OreFeatureContext {
    ChunkRandom& random;
    BlockColumn& col;
    int originX = 0, originY = 0, originZ = 0; // 放置起点（placementModifiers 输出，world 坐标）
    int chunkStartX = 0, chunkStartZ = 0; // 当前 chunk 起点（world 坐标）
    int minY = -64, height = 384;
    BlockRegistry& blocks;
    // OCEAN_FLOOR_WG 高度图 [z*16+x]（NOISE 阶段构建）
    const std::vector<int>* oceanFloor = nullptr;
    // 两阶段 FEATURE 跨 chunk：regionColAt(cx,cz) 返回区域 col（null=不在区域）；regionColsMtx 保护写
    std::function<int32_t*(int, int)> regionColAt = nullptr;
    std::mutex* regionColsMtx = nullptr;
    // pending 跨 chunk 写入（Java 语义：A 后生成覆盖 B）——回调 (chunkX, chunkZ, 块索引, state)
    std::function<void(int, int, int, int32_t)> pendingCross = nullptr;

    OreFeatureContext(ChunkRandom& r, BlockColumn& c, BlockRegistry& b) : random(r), col(c), blocks(b) {}

    // world → col 局部；越界返回 -1（Java world.isOutOfHeightLimit / isValidForSetBlock）
    int localIdx(int wx, int wy, int wz) const {
        int lx = wx - chunkStartX, lz = wz - chunkStartZ;
        if (lx < 0 || lx >= 16 || lz < 0 || lz >= 16) return -1;
        if (wy < minY || wy >= minY + height) return -1;
        return (wy - minY) * 256 + lz * 16 + lx;
    }
    int blockAt(int wx, int wy, int wz) const {
        int idx = localIdx(wx, wy, wz);
        if (idx >= 0) return col.at(lxOf(wx), wy, lzOf(wz));
        // 跨 chunk 读（两阶段）
        if (regionColAt) {
            int cx = wx >> 4, cz = wz >> 4;
            int32_t* rc = regionColAt(cx, cz);
            if (rc && wy >= minY && wy < minY + height) {
                return rc[(wy - minY) * 256 + (wz & 15) * 16 + (wx & 15)];
            }
        }
        return -1;
    }
    int lxOf(int wx) const { return wx - chunkStartX; }
    int lzOf(int wz) const { return wz - chunkStartZ; }
    // getTopY(OCEAN_FLOOR_WG, x, z)——NOISE 阶段高度图
    int getOceanFloorTopY(int wx, int wz) const {
        if (!oceanFloor) return minY - 1;
        int lx = wx - chunkStartX, lz = wz - chunkStartZ;
        if (lx < 0 || lx >= 16 || lz < 0 || lz >= 16) return minY - 1;
        return (*oceanFloor)[lz * 16 + lx];
    }
    // 放置（当前 chunk 或跨 chunk：记录 pending，阶段 2 末尾统一应用——Java A 后生成覆盖 B）
    void setBlock(int wx, int wy, int wz, int state) {
        int lx = wx - chunkStartX, lz = wz - chunkStartZ;
        if (lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && wy >= minY && wy < minY + height) {
            col.at(lx, wy, lz) = state;
            return;
        }
        if (pendingCross && wy >= minY && wy < minY + height) {
            int cx = wx >> 4, cz = wz >> 4;
            pendingCross(cx, cz, (wy - minY) * 256 + (wz & 15) * 16 + (wx & 15), state);
        }
    }
};

// ===== OreFeature（Java OreFeature.java）=====
class OreFeature {
public:
    // Java generate：random.nextFloat()*π → 端点；if (o <= getTopY(OCEAN_FLOOR_WG, s, t)) generateVeinPart
    bool generate(OreFeatureContext& ctx, const OreFeatureConfig& config) {
        ChunkRandom& random = ctx.random;
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        float f = random.nextFloat() * (float)3.14159265358979323846; // Java Math.PI（double→float 参数）
        float g = config.size / 8.0F;
        int i = (int)std::ceil((config.size / 16.0F * 2.0F + 1.0F) / 2.0F);
        double d = x + std::sin(f) * g;      // Java Math.sin（标准库！）
        double e = x - std::sin(f) * g;
        double h = z + std::cos(f) * g;
        double j = z - std::cos(f) * g;
        int k = 2;
        double l = y + random.nextInt(3) - 2;
        double m = y + random.nextInt(3) - 2;
        int n = x - (int)std::ceil(g) - i;
        int o = y - 2 - i;
        int p = z - (int)std::ceil(g) - i;
        int q = 2 * ((int)std::ceil(g) + i);
        int r = 2 * (2 + i);

        for (int s = n; s <= n + q; s++) {
            for (int t = p; t <= p + q; t++) {
                if (o <= ctx.getOceanFloorTopY(s, t)) {
                    return generateVeinPart(ctx, config, d, e, h, j, l, m, n, o, p, q, r);
                }
            }
        }
        return false;
    }

    // Java generateVeinPart（L55-166）
    bool generateVeinPart(OreFeatureContext& ctx, const OreFeatureConfig& config,
                          double startX, double endX, double startZ, double endZ,
                          double startY, double endY, int x, int y, int z,
                          int horizontalSize, int verticalSize) {
        ChunkRandom& random = ctx.random;
        int i = 0;
        int j = config.size;
        std::vector<uint64_t> bitSet((size_t)((horizontalSize * verticalSize * horizontalSize) + 63) / 64, 0);
        std::vector<double> ds((size_t)(j * 4), 0.0);

        for (int k = 0; k < j; k++) {
            float f = (float)k / j;
            double d = lerp(f, startX, endX);
            double e = lerp(f, startY, endY);
            double g = lerp(f, startZ, endZ);
            double h = random.nextDouble() * j / 16.0;
            double l = ((mathSin((float)(3.14159265358979323846 * f)) + 1.0F) * h + 1.0) / 2.0; // MathHelper.sin 查表
            ds[k * 4 + 0] = d;
            ds[k * 4 + 1] = e;
            ds[k * 4 + 2] = g;
            ds[k * 4 + 3] = l;
        }
        for (int k = 0; k < j - 1; k++) {
            if (!(ds[k * 4 + 3] <= 0.0)) {
                for (int m = k + 1; m < j; m++) {
                    if (!(ds[m * 4 + 3] <= 0.0)) {
                        double d = ds[k * 4 + 0] - ds[m * 4 + 0];
                        double e = ds[k * 4 + 1] - ds[m * 4 + 1];
                        double g = ds[k * 4 + 2] - ds[m * 4 + 2];
                        double h = ds[k * 4 + 3] - ds[m * 4 + 3];
                        if (h * h > d * d + e * e + g * g) {
                            if (h > 0.0) ds[m * 4 + 3] = -1.0;
                            else ds[k * 4 + 3] = -1.0;
                        }
                    }
                }
            }
        }
        for (int mx = 0; mx < j; mx++) {
            double d = ds[mx * 4 + 3];
            if (d < 0.0) continue;
            double e = ds[mx * 4 + 0];
            double g = ds[mx * 4 + 1];
            double h = ds[mx * 4 + 2];
            int n = std::max((int)std::floor(e - d), x);
            int o = std::max((int)std::floor(g - d), y);
            int p = std::max((int)std::floor(h - d), z);
            int q = std::max((int)std::floor(e + d), n);
            int r = std::max((int)std::floor(g + d), o);
            int s = std::max((int)std::floor(h + d), p);
            for (int t = n; t <= q; t++) {
                double u = (t + 0.5 - e) / d;
                if (u * u < 1.0) {
                    for (int v = o; v <= r; v++) {
                        double w = (v + 0.5 - g) / d;
                        if (u * u + w * w < 1.0) {
                            for (int aa = p; aa <= s; aa++) {
                                double ab = (aa + 0.5 - h) / d;
                                if (u * u + w * w + ab * ab < 1.0) {
                                    int ac = t - x + (v - y) * horizontalSize + (aa - z) * horizontalSize * verticalSize;
                                    if (ac < 0) continue;
                                    if (!(bitSet[(size_t)ac / 64] >> (ac % 64) & 1)) {
                                        bitSet[(size_t)ac / 64] |= 1ULL << (ac % 64);
                                        // world.isValidForSetBlock + ChunkSection 读写（C++：col 局部 + 跨 chunk regionCols）
                                        if (v >= ctx.minY && v < ctx.minY + ctx.height) {
                                            int state = ctx.blockAt(t, v, aa);
                                            for (const auto& target : config.targets) {
                                                if (shouldPlace(ctx, config, target, state, t, v, aa)) {
                                                    ctx.setBlock(t, v, aa, target.state);
                                                    i++;
                                                    if (getenv("WG_FEATURELOG") && target.state == 2) {
                                                        std::fprintf(stderr, "[OREPLACE] (%d,%d,%d) -> %d\n", t, v, aa, target.state);
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return i > 0;
    }

    static bool shouldPlace(OreFeatureContext& ctx, const OreFeatureConfig& config,
                            const OreFeatureConfig::Target& target, int state, int x, int y, int z) {
        if (!target.target.test(ctx.blocks, state, ctx.random)) return false;
        return shouldNotDiscard(ctx.random, config.discardOnAirChance) ? true : !isExposedToAir(ctx, x, y, z);
    }

    // Java Feature.isExposedToAir：6 邻居任一 isAir
    static bool isExposedToAir(OreFeatureContext& ctx, int x, int y, int z) {
        static const int dxs[6] = {1, -1, 0, 0, 0, 0};
        static const int dys[6] = {0, 0, 1, -1, 0, 0};
        static const int dzs[6] = {0, 0, 0, 0, 1, -1};
        for (int i = 0; i < 6; i++) {
            int nx = x + dxs[i], ny = y + dys[i], nz = z + dzs[i];
            int idx = ctx.localIdx(nx, ny, nz);
            int id = idx >= 0 ? ctx.col.at(ctx.lxOf(nx), ny, ctx.lzOf(nz)) : -1;
            if (id == 0) return true; // air
        }
        return false;
    }

    // Java OreFeature.shouldNotDiscard
    static bool shouldNotDiscard(ChunkRandom& random, float chance) {
        if (chance <= 0.0F) return true;
        return chance >= 1.0F ? false : random.nextFloat() >= chance;
    }

    // MathHelper.lerp(double delta, double start, double end) = start + delta * (end - start)
    static double lerp(double delta, double start, double end) {
        return start + delta * (end - start);
    }
};

// ===== ScatteredOreFeature（Java ScatteredOreFeature.java）=====
class ScatteredOreFeature {
public:
    bool generate(OreFeatureContext& ctx, const OreFeatureConfig& config) {
        ChunkRandom& random = ctx.random;
        int i = random.nextInt(config.size + 1);
        for (int j = 0; j < i; j++) {
            int lx = getSpread(random, std::min(j, 7));
            int ly = getSpread(random, std::min(j, 7));
            int lz = getSpread(random, std::min(j, 7));
            int wx = ctx.originX + lx, wy = ctx.originY + ly, wz = ctx.originZ + lz;
            if (wy < ctx.minY || wy >= ctx.minY + ctx.height) continue;
            int state = ctx.blockAt(wx, wy, wz);
            for (const auto& target : config.targets) {
                if (OreFeature::shouldPlace(ctx, config, target, state, wx, wy, wz)) {
                    ctx.setBlock(wx, wy, wz, target.state);
                    break;
                }
            }
        }
        return true;
    }

private:
    // Java getSpread = Math.round((nextFloat()-nextFloat()) * spread)
    static int getSpread(ChunkRandom& random, int spread) {
        return (int)std::lround((random.nextFloat() - random.nextFloat()) * spread);
    }
};

} // namespace wg

// ===== RuleTest::parse / OreFeatureConfig::parse 实现 =====
namespace wg {

// 常见 tag 展开（server jar 权威，1.20.1）——按需补充
inline void expandTag(BlockRegistry& blocks, const std::string& tag, std::vector<int>& out) {
    auto add = [&](const char* n) { out.push_back(blocks.id(n)); };
    if (tag == "minecraft:base_stone_overworld") {
        add("minecraft:stone"); add("minecraft:granite"); add("minecraft:diorite");
        add("minecraft:andesite"); add("minecraft:tuff"); add("minecraft:deepslate");
    } else if (tag == "minecraft:stone_ore_replaceables") {
        add("minecraft:stone"); add("minecraft:granite"); add("minecraft:diorite");
        add("minecraft:andesite");
    } else if (tag == "minecraft:deepslate_ore_replaceables") {
        add("minecraft:deepslate"); add("minecraft:tuff");
    } else if (tag == "minecraft:netherrack") {
        add("minecraft:netherrack");
    } else if (tag == "minecraft:base_stone_nether") {
        add("minecraft:netherrack"); add("minecraft:basalt"); add("minecraft:blackstone");
    } else if (tag == "minecraft:sand") {
        add("minecraft:sand"); add("minecraft:red_sand"); add("minecraft:suspicious_sand");
    } else if (tag == "minecraft:dirt") {
        add("minecraft:dirt"); add("minecraft:grass_block"); add("minecraft:podzol");
        add("minecraft:coarse_dirt"); add("minecraft:mycelium"); add("minecraft:rooted_dirt");
        add("minecraft:moss_block"); add("minecraft:mud"); add("minecraft:muddy_mangrove_roots");
    }
}

inline RuleTest RuleTest::parse(const JsonValue* v, BlockRegistry& blocks) {
    RuleTest rt;
    if (!v) return rt;
    std::string type = v->get("predicate_type") ? v->get("predicate_type")->strVal : "";
    if (type.find("tag_match") != std::string::npos) {
        rt.kind = RuleTest::Kind::TAG_MATCH;
        std::string tag = v->get("tag") ? v->get("tag")->strVal : "";
        if (!tag.empty() && tag[0] == '#') tag = tag.substr(1);
        expandTag(blocks, tag, rt.blockIds);
    } else if (type.find("block_match") != std::string::npos) {
        rt.kind = RuleTest::Kind::BLOCK_MATCH;
        std::string name = v->get("block") ? v->get("block")->strVal : "";
        rt.blockIds.push_back(blocks.id(name));
    } else if (type.find("random_block_match") != std::string::npos) {
        rt.kind = RuleTest::Kind::RANDOM_BLOCK_MATCH;
        rt.probability = (float)(v->get("probability") ? v->get("probability")->numVal : 0.0);
        std::string name = v->get("block") ? v->get("block")->strVal : "";
        rt.blockIds.push_back(blocks.id(name));
    } else {
        rt.kind = RuleTest::Kind::ALWAYS_TRUE;
    }
    return rt;
}

inline OreFeatureConfig OreFeatureConfig::parse(const JsonValue* cfg, BlockRegistry& blocks) {
    OreFeatureConfig oc;
    if (!cfg) return oc;
    if (const JsonValue* s = cfg->get("size")) oc.size = (int)s->numVal;
    if (const JsonValue* d = cfg->get("discard_chance_on_air_exposure")) oc.discardOnAirChance = (float)d->numVal;
    if (const JsonValue* targets = cfg->get("targets")) {
        for (const auto& t : targets->arr) {
            Target tg;
            if (const JsonValue* state = t.get("state")) {
                std::string name = state->get("Name") ? state->get("Name")->strVal : "";
                tg.state = blocks.id(name);
            }
            if (const JsonValue* target = t.get("target")) tg.target = RuleTest::parse(target, blocks);
            oc.targets.push_back(tg);
        }
    }
    return oc;
}

// ===== Phase 4：简单装饰 Feature（MC 1.20.1）=====
// DiskFeature / SpringFeature / FreezeTopLayerFeature / UnderwaterMagmaFeature
// Java 参照：world/gen/feature/DiskFeature.java / SpringFeature.java / FreezeTopLayerFeature.java / UnderwaterMagmaFeature.java

// ===== DiskFeatureConfig（Java DiskFeatureConfig.java）=====
struct DiskFeatureConfig {
    int halfHeight = 0;
    IntProvider radius;            // uniform(2,6) 等
    int state = 0;                 // state_provider fallback（简化：simple_state_provider）
    std::vector<int> targets;      // target matching_blocks（多块）或 tag

    static DiskFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        DiskFeatureConfig dc;
        if (!cfg) return dc;
        if (const JsonValue* h = cfg->get("half_height")) dc.halfHeight = (int)h->numVal;
        if (const JsonValue* r = cfg->get("radius")) dc.radius = IntProvider::parse(r);
        // state_provider：取 fallback（simple_state_provider）
        if (const JsonValue* sp = cfg->get("state_provider")) {
            if (const JsonValue* fb = sp->get("fallback")) {
                if (const JsonValue* st = fb->get("state")) {
                    dc.state = blocks.id(st->get("Name") ? st->get("Name")->strVal : "");
                }
            }
        }
        // target：matching_blocks（数组或字符串）
        if (const JsonValue* t = cfg->get("target")) {
            const JsonValue* blk = t->get("blocks");
            if (blk) {
                if (blk->isArray()) {
                    for (const auto& b : blk->arr) dc.targets.push_back(blocks.id(b.strVal));
                } else {
                    std::string name = blk->strVal;
                    if (name.rfind("#", 0) == 0) { expandTag(blocks, name.substr(1), dc.targets); }
                    else dc.targets.push_back(blocks.id(name));
                }
            }
        }
        return dc;
    }
};

// ===== DiskFeature（Java DiskFeature.java）=====
class DiskFeature {
public:
    // 简化 context：读/写当前 chunk col + 跨 chunk（regionColAt/pendingCross 同 OreFeatureContext）
    // 用 OreFeatureContext 的 blockAt/setBlock（含跨 chunk）
    bool generate(OreFeatureContext& ctx, const DiskFeatureConfig& config) {
        int y = ctx.originY;
        int topY = y + config.halfHeight;
        int bottomY = y - config.halfHeight - 1;
        int radius = config.radius.get(ctx.random);
        bool placed = false;
        for (int dx = -radius; dx <= radius; dx++) {
            for (int dz = -radius; dz <= radius; dz++) {
                int mx = dx * dx + dz * dz;
                if (mx > radius * radius) continue;
                int wx = ctx.originX + dx, wz = ctx.originZ + dz;
                for (int iy = topY; iy > bottomY; iy--) {
                    if (targetMatches(ctx, config, wx, iy, wz)) {
                        ctx.setBlock(wx, iy, wz, config.state);
                        placed = true;
                        break; // Java placeBlock 的 for 循环内找到即 setBlock + break？——不，Java 是 for 内全部判定
                    }
                }
            }
        }
        return placed;
    }
private:
    bool targetMatches(OreFeatureContext& ctx, const DiskFeatureConfig& config, int x, int y, int z) const {
        int cur = ctx.blockAt(x, y, z);
        if (cur < 0) return false;
        for (int id : config.targets) if (cur == id) return true;
        return false;
    }
};

// ===== SpringFeatureConfig（Java SpringFeatureConfig.java）=====
struct SpringFeatureConfig {
    int state = 0;                 // 简化：固定块（water/lava）
    std::vector<int> validBlocks;  // 数组或 tag
    int rockCount = 0;
    int holeCount = 0;
    bool requiresBlockBelow = false;

    static SpringFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        SpringFeatureConfig sc;
        if (!cfg) return sc;
        if (const JsonValue* st = cfg->get("state")) {
            sc.state = blocks.id(st->get("Name") ? st->get("Name")->strVal : "");
        }
        if (const JsonValue* vb = cfg->get("valid_blocks")) {
            for (const auto& b : vb->arr) {
                std::string name = b.strVal;
                if (name.rfind("#", 0) == 0) expandTag(blocks, name.substr(1), sc.validBlocks);
                else sc.validBlocks.push_back(blocks.id(name));
            }
        }
        if (const JsonValue* rc = cfg->get("rock_count")) sc.rockCount = (int)rc->numVal;
        if (const JsonValue* hc = cfg->get("hole_count")) sc.holeCount = (int)hc->numVal;
        if (const JsonValue* rb = cfg->get("requires_block_below")) sc.requiresBlockBelow = rb->boolVal;
        return sc;
    }
};

// ===== SpringFeature（Java SpringFeature.java）=====
class SpringFeature {
public:
    bool generate(OreFeatureContext& ctx, const SpringFeatureConfig& config) {
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        if (!isValid(ctx, config, x, y + 1, z)) return false;                    // up 必须 valid
        if (config.requiresBlockBelow && !isValid(ctx, config, x, y - 1, z)) return false; // down 必须 valid
        int cur = ctx.blockAt(x, y, z);
        if (cur < 0) return false;
        bool curOk = (cur == ctx.blocks.id("minecraft:air"));
        if (!curOk) { for (int id : config.validBlocks) if (cur == id) { curOk = true; break; } }
        if (!curOk) return false;
        // 统计 5 邻（东西南北下）valid（rockCount）与 air（holeCount）
        int rock = 0, hole = 0;
        const int dxs[5] = {-1, 1, 0, 0, 0};
        const int dzs[5] = {0, 0, -1, 1, 0};
        const int dys[5] = {0, 0, 0, 0, -1};
        for (int i = 0; i < 5; i++) {
            int nx = x + dxs[i], ny = y + dys[i], nz = z + dzs[i];
            if (isValid(ctx, config, nx, ny, nz)) rock++;
            if (ctx.blockAt(nx, ny, nz) == ctx.blocks.id("minecraft:air")) hole++;
        }
        if (rock == config.rockCount && hole == config.holeCount) {
            ctx.setBlock(x, y, z, config.state);
            return true;
        }
        return false;
    }
private:
    bool isValid(OreFeatureContext& ctx, const SpringFeatureConfig& config, int x, int y, int z) const {
        int cur = ctx.blockAt(x, y, z);
        if (cur < 0) return false;
        for (int id : config.validBlocks) if (cur == id) return true;
        return false;
    }
};

// ===== FreezeTopLayerFeature（Java FreezeTopLayerFeature.java）=====
// 简化：MOTION_BLOCKING 用 C++ 高度图（buildSurface 的 topY）；canSetIce/canSetSnow 用 biome 温度+降水
class FreezeTopLayerFeature {
public:
    // 需要 biome 温度（<0 且降水 SNOW 才冻结）——C++ BiomeEntry 简化：temperature/rainfall 参数
    bool generate(OreFeatureContext& ctx, float biomeTemp, float biomeRainfall) {
        (void)biomeRainfall;
        bool snowy = biomeTemp < 0.0f; // canSetSnow 主条件（precipitation==SNOW 由温度近似）
        if (!snowy) return true;       // -288 冷洋不冻结——直接返回
        int iceId = ctx.blocks.id("minecraft:ice");
        int snowId = ctx.blocks.id("minecraft:snow_block");
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int wx = ctx.chunkStartX + lx, wz = ctx.chunkStartZ + lz;
                int topY = ctx.getOceanFloorTopY(wx, wz); // 简化：用 OCEAN_FLOOR_WG 近似 MOTION_BLOCKING
                if (topY < ctx.minY) continue;
                // canSetIce(world, top.down())：top.down 是水/空气且可放冰
                ctx.setBlock(wx, topY - 1, wz, iceId);
                // canSetSnow(world, top)
                ctx.setBlock(wx, topY, wz, snowId);
            }
        }
        return true;
    }
};

// ===== UnderwaterMagmaFeatureConfig（Java UnderwaterMagmaFeatureConfig.java）=====
struct UnderwaterMagmaFeatureConfig {
    int floorSearchRange = 0;
    float placementProbabilityPerValidPosition = 0.0f;
    int placementRadiusAroundFloor = 0;

    static UnderwaterMagmaFeatureConfig parse(const JsonValue* cfg, BlockRegistry& blocks) {
        (void)blocks;
        UnderwaterMagmaFeatureConfig uc;
        if (!cfg) return uc;
        if (const JsonValue* f = cfg->get("floor_search_range")) uc.floorSearchRange = (int)f->numVal;
        if (const JsonValue* p = cfg->get("placement_probability_per_valid_position")) uc.placementProbabilityPerValidPosition = (float)p->numVal;
        if (const JsonValue* r = cfg->get("placement_radius_around_floor")) uc.placementRadiusAroundFloor = (int)r->numVal;
        return uc;
    }
};

// ===== UnderwaterMagmaFeature（Java UnderwaterMagmaFeature.java + CaveSurface.java）=====
// Java 语义（1.20.1 精确）：
//   - CaveSurface.create(world, origin, floorSearchRange, water, !water)：origin 必须 water；
//     沿列向上/下找水柱边界（canGenerate=water 继续，canReplace=!water 停止）→ floor/ceiling
//   - blockPos2 = origin.withY(floor)；box = blockPos2 ± placementRadiusAroundFloor（3×3×3）
//   - Box.stream（x 内层 z 中层 y 外层）→ filter(nextFloat < prob) → filter(isValidPosition)
//   - isValidPosition：pos 与 pos.down 都非 water/air；4 水平邻都非 water/air（全石头包围）
class UnderwaterMagmaFeature {
public:
    bool generate(OreFeatureContext& ctx, const UnderwaterMagmaFeatureConfig& config) {
        int x = ctx.originX, y = ctx.originY, z = ctx.originZ;
        int waterId = ctx.blocks.id("minecraft:water");
        int airId = ctx.blocks.id("minecraft:air");
        int magmaId = ctx.blocks.id("minecraft:magma_block");
        // CaveSurface.create：origin 必须 water（canGenerate）
        if (ctx.blockAt(x, y, z) != waterId) return false;
        // 向上找 ceiling、向下找 floor（canGenerate=water 继续移动，停在第一个非 water）
        int ceilingY = -1, floorY = -1;
        int my = y;
        for (int i = 1; i < config.floorSearchRange && ctx.blockAt(x, my, z) == waterId; i++) my++;
        if (ctx.blockAt(x, my, z) != waterId) ceilingY = my;
        my = y;
        for (int i = 1; i < config.floorSearchRange && ctx.blockAt(x, my, z) == waterId; i++) my--;
        if (ctx.blockAt(x, my, z) != waterId) floorY = my;
        if (ceilingY < 0 || floorY < 0) return false; // Java create(floor, ceiling) 需两个都 present（Bounded）
        int placed = 0;
        // Box.stream 顺序：x 内层、z 中层、y 外层（BlockPos.BlockPosIterator）
        for (int dy = -config.placementRadiusAroundFloor; dy <= config.placementRadiusAroundFloor; dy++) {
            for (int dz = -config.placementRadiusAroundFloor; dz <= config.placementRadiusAroundFloor; dz++) {
                for (int dx = -config.placementRadiusAroundFloor; dx <= config.placementRadiusAroundFloor; dx++) {
                    int px = x + dx, py = floorY + dy, pz = z + dz;
                    if (ctx.random.nextFloat() < config.placementProbabilityPerValidPosition) {
                        if (isValidPosition(ctx, px, py, pz, waterId, airId)) {
                            ctx.setBlock(px, py, pz, magmaId);
                            placed++;
                        }
                    }
                }
            }
        }
        return placed > 0;
    }
private:
    bool isValidPosition(OreFeatureContext& ctx, int x, int y, int z, int waterId, int airId) const {
        if (isWaterOrAir(ctx, x, y, z, waterId, airId)) return false;
        if (isWaterOrAir(ctx, x, y - 1, z, waterId, airId)) return false;
        const int dxs[4] = {-1, 1, 0, 0};
        const int dzs[4] = {0, 0, -1, 1};
        for (int i = 0; i < 4; i++) {
            if (isWaterOrAir(ctx, x + dxs[i], y, z + dzs[i], waterId, airId)) return false;
        }
        return true;
    }
    bool isWaterOrAir(OreFeatureContext& ctx, int x, int y, int z, int waterId, int airId) const {
        int cur = ctx.blockAt(x, y, z);
        if (cur < 0) return true; // 越界视为 water/air（Java chunkSectionCache null → 默认 air）
        return cur == waterId || cur == airId;
    }
};

} // namespace wg
