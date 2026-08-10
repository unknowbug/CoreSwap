#pragma once
// carver.h — CARVERS 阶段（洞穴雕刻）MC 1.20.1 移植
// 对应 Java: carver/Carver.java / CaveCarver.java / RavineCarver.java / CarvingMask.java /
//            CarverContext.java / CarverConfig.java / CaveCarverConfig.java / RavineCarverConfig.java
// 语义要点（逐位对齐）：
//   - ChunkRandom（CheckedRandom LCG 基类）setCarverSeed(seed+l, cx, cz)；内部递归用 Random.create(seed)=Xoroshiro
//   - CarvingMask index = (x&15)|(z&15)<<4|(y-bottomY)<<8；get/set 按位
//   - carveRegion：g=(s+0.5-x)/width, h=(u+0.5-z)/width；y 从 o 递减到 m+1，w=(v-0.5-y)/height
//   - getState：y<=lavaLevel.getY(minY+8=-56) → lava；否则 aquifer.apply(pos, 0.0)
//   - replaceable tag：overworld_carver_replaceables（含 water！）
//   - applyMaterialRule（grass 被挖后 dirt 替换）：SurfaceContext.initVertical(1,1,fluidHeight) + rule.apply
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <string>
#include <vector>
#include <functional>
#include <set>

#include "blocks.h"
#include "chunkrandom.h"
#include "xoroshiro.h"
#include "aquifer.h"
#include "surface.h"

namespace wg {

// ===== MathHelper.sin/cos（MC 65536 项 SINE_TABLE 查表，非 std::sin！carve 漂移逐位对齐关键）=====
inline float mathSin(float value) {
    static const std::vector<float> table = [] {
        std::vector<float> t(65536);
        for (int i = 0; i < 65536; i++) t[i] = (float)std::sin(i * 3.14159265358979323846 * 2.0 / 65536.0);
        return t;
    }();
    return table[(int)(value * 10430.378F) & 65535];
}
inline float mathCos(float value) {
    static const std::vector<float> table = [] {
        std::vector<float> t(65536);
        for (int i = 0; i < 65536; i++) t[i] = (float)std::sin(i * 3.14159265358979323846 * 2.0 / 65536.0);
        return t;
    }();
    return table[(int)(value * 10430.378F + 16384.0F) & 65535];
}

// ===== CarvingMask（Java CarvingMask.java：BitSet 256*height）=====
class CarvingMask {
public:
    CarvingMask(int height, int bottomY) : bottomY(bottomY), bits((size_t)(256 * height + 63) / 64, 0) {}
    CarvingMask(const std::vector<uint64_t>& mask, int bottomY) : bottomY(bottomY), bits(mask) {}

    bool get(int offsetX, int y, int offsetZ) const {
        int idx = getIndex(offsetX, y, offsetZ);
        return (bits[(size_t)idx / 64] >> (idx % 64)) & 1;
    }
    void set(int offsetX, int y, int offsetZ) {
        int idx = getIndex(offsetX, y, offsetZ);
        bits[(size_t)idx / 64] |= 1ULL << (idx % 64);
    }
    int getBottomY() const { return bottomY; }

private:
    int bottomY;
    std::vector<uint64_t> bits;
    int getIndex(int offsetX, int y, int offsetZ) const {
        return (offsetX & 15) | ((offsetZ & 15) << 4) | ((y - bottomY) << 8);
    }
};

// ===== providers（YOffset / HeightProvider / FloatProvider 最小移植）=====
// YOffset：absolute / above_bottom / below_top（Java YOffset.java）
struct YOffset {
    enum class YOffsetKind { FIXED, ABOVE_BOTTOM, BELOW_TOP };
    YOffsetKind kind = YOffsetKind::FIXED;
    int value = 0;
    int getY(int minY, int height) const {
        switch (kind) {
            case YOffsetKind::FIXED: return value;
            case YOffsetKind::ABOVE_BOTTOM: return minY + value;
            case YOffsetKind::BELOW_TOP: return minY + height - 1 - value;
        }
        return value;
    }
    // 解析 {"absolute": n} / {"above_bottom": n} / {"below_top": n}
    static YOffset parse(const JsonValue* v) {
        YOffset y;
        if (!v) return y;
        if (const JsonValue* a = v->get("absolute")) { y.kind = YOffsetKind::FIXED; y.value = (int)a->numVal; }
        else if (const JsonValue* a = v->get("above_bottom")) { y.kind = YOffsetKind::ABOVE_BOTTOM; y.value = (int)a->numVal; }
        else if (const JsonValue* a = v->get("below_top")) { y.kind = YOffsetKind::BELOW_TOP; y.value = (int)a->numVal; }
        return y;
    }
};

// HeightProvider：uniform（min/max YOffset）——carver 用（Java UniformHeightProvider）
struct HeightProvider {
    YOffset minOffset, maxOffset;
    int get(ChunkRandom& r, int minY, int height) const {
        int i = minOffset.getY(minY, height);
        int j = maxOffset.getY(minY, height);
        if (i == j) return i;
        // MathHelper.nextBetween(random, i, j) = random.nextInt(j-i+1) + i（ChunkRandom CHECKED 基类）
        return r.nextInt(j - i + 1) + i;
    }
    static HeightProvider parse(const JsonValue* v) {
        HeightProvider hp;
        if (!v) return hp;
        if (v->isObject() && v->get("min_inclusive") && v->get("max_inclusive")) {
            hp.minOffset = YOffset::parse(v->get("min_inclusive"));
            hp.maxOffset = YOffset::parse(v->get("max_inclusive"));
        }
        return hp;
    }
};

// FloatProvider：uniform / trapezoid / constant（Java FloatProvider 体系）
struct FloatProvider {
    enum class Kind { UNIFORM, TRAPEZOID, CONSTANT };
    Kind kind = Kind::CONSTANT;
    float a = 0, b = 0, plateau = 0; // uniform: [a,b)；trapezoid: [a,b] plateau；constant: a
    float get(ChunkRandom& r) const {
        switch (kind) {
            case Kind::UNIFORM: return r.nextFloat() * (b - a) + a;   // nextFloat()*(max-min)+min
            case Kind::TRAPEZOID: {
                float f = b - a;
                float g = (f - plateau) / 2.0F;
                float h = f - g;
                return a + r.nextFloat() * h + r.nextFloat() * g;
            }
            case Kind::CONSTANT: return a;
        }
        return a;
    }
    // 重载：XoroshiroRandom（carveTunnels/carveRavine 内部 Random.create(seed) 用）
    float get(XoroshiroRandom& r) const {
        switch (kind) {
            case Kind::UNIFORM: return r.nextFloat() * (b - a) + a;
            case Kind::TRAPEZOID: {
                float f = b - a;
                float g = (f - plateau) / 2.0F;
                float h = f - g;
                return a + r.nextFloat() * h + r.nextFloat() * g;
            }
            case Kind::CONSTANT: return a;
        }
        return a;
    }
    // 重载：CheckedRandom（carveTunnels/carveRavine 内部 Random.create(seed) = CheckedRandom LCG！）
    float get(CheckedRandom& r) const {
        switch (kind) {
            case Kind::UNIFORM: return r.nextFloat() * (b - a) + a;
            case Kind::TRAPEZOID: {
                float f = b - a;
                float g = (f - plateau) / 2.0F;
                float h = f - g;
                return a + r.nextFloat() * h + r.nextFloat() * g;
            }
            case Kind::CONSTANT: return a;
        }
        return a;
    }
    // 解析：数值 = constant；{type:uniform,value:{min_inclusive,max_exclusive}} / {type:trapezoid,value:{min,max,plateau}}
    static FloatProvider parse(const JsonValue* v) {
        FloatProvider fp;
        if (!v) return fp;
        if (v->isNumber()) { fp.kind = Kind::CONSTANT; fp.a = (float)v->numVal; return fp; }
        if (!v->isObject()) return fp;
        std::string type = v->get("type") ? v->get("type")->strVal : "";
        const JsonValue* val = v->get("value");
        if (type.find("uniform") != std::string::npos && val) {
            fp.kind = Kind::UNIFORM;
            fp.a = (float)(val->get("min_inclusive") ? val->get("min_inclusive")->numVal : 0.0);
            fp.b = (float)(val->get("max_exclusive") ? val->get("max_exclusive")->numVal : 1.0);
        } else if (type.find("trapezoid") != std::string::npos && val) {
            fp.kind = Kind::TRAPEZOID;
            fp.a = (float)(val->get("min") ? val->get("min")->numVal : 0.0);
            fp.b = (float)(val->get("max") ? val->get("max")->numVal : 1.0);
            fp.plateau = (float)(val->get("plateau") ? val->get("plateau")->numVal : 0.0);
        }
        return fp;
    }
};

// ===== Config（CarverConfig / CaveCarverConfig / RavineCarverConfig）=====
struct CarverConfig {
    float probability = 0.0f;
    HeightProvider y;
    FloatProvider yScale;
    YOffset lavaLevel;                    // above_bottom(8) → minY+8 = -56
    std::vector<int> replaceableIds;      // #minecraft:overworld_carver_replaceables 展开（含 water！）
    bool hasDebug = false;

    static std::vector<int> buildOverworldReplaceable(BlockRegistry& blocks) {
        // server jar 权威 tag 展开（1.20.1）：
        // base_stone_overworld + dirt + sand + terracotta + iron_ores + copper_ores + 直接值
        static const char* names[] = {
            // base_stone_overworld
            "minecraft:stone","minecraft:granite","minecraft:diorite","minecraft:andesite",
            "minecraft:tuff","minecraft:deepslate",
            // dirt
            "minecraft:dirt","minecraft:grass_block","minecraft:podzol","minecraft:coarse_dirt",
            "minecraft:mycelium","minecraft:rooted_dirt","minecraft:moss_block","minecraft:mud",
            "minecraft:muddy_mangrove_roots",
            // sand
            "minecraft:sand","minecraft:red_sand","minecraft:suspicious_sand",
            // terracotta（17 色 + 无前缀）
            "minecraft:terracotta","minecraft:white_terracotta","minecraft:orange_terracotta",
            "minecraft:magenta_terracotta","minecraft:light_blue_terracotta","minecraft:yellow_terracotta",
            "minecraft:lime_terracotta","minecraft:pink_terracotta","minecraft:gray_terracotta",
            "minecraft:light_gray_terracotta","minecraft:cyan_terracotta","minecraft:purple_terracotta",
            "minecraft:blue_terracotta","minecraft:brown_terracotta","minecraft:green_terracotta",
            "minecraft:red_terracotta","minecraft:black_terracotta",
            // iron_ores / copper_ores
            "minecraft:iron_ore","minecraft:deepslate_iron_ore",
            "minecraft:copper_ore","minecraft:deepslate_copper_ore",
            // 直接值
            "minecraft:water","minecraft:gravel","minecraft:suspicious_gravel","minecraft:sandstone",
            "minecraft:red_sandstone","minecraft:calcite","minecraft:snow","minecraft:packed_ice",
            "minecraft:raw_iron_block","minecraft:raw_copper_block"
        };
        std::vector<int> ids;
        for (const char* n : names) ids.push_back(blocks.id(n));
        return ids;
    }

    void parseCommon(const JsonValue* cfg, BlockRegistry& blocks) {
        if (!cfg) return;
        if (const JsonValue* p = cfg->get("probability")) probability = (float)p->numVal;
        if (const JsonValue* yv = cfg->get("y")) y = HeightProvider::parse(yv);
        if (const JsonValue* ys = cfg->get("yScale")) yScale = FloatProvider::parse(ys);
        if (const JsonValue* ll = cfg->get("lava_level")) lavaLevel = YOffset::parse(ll);
        replaceableIds = buildOverworldReplaceable(blocks);
    }
};

struct CaveCarverConfig : CarverConfig {
    FloatProvider horizontalRadiusMultiplier;
    FloatProvider verticalRadiusMultiplier;
    FloatProvider floorLevel;   // validated [-1,1]
    void parse(const JsonValue* cfg, BlockRegistry& blocks) {
        parseCommon(cfg, blocks);
        if (const JsonValue* v = cfg->get("horizontal_radius_multiplier")) horizontalRadiusMultiplier = FloatProvider::parse(v);
        if (const JsonValue* v = cfg->get("vertical_radius_multiplier")) verticalRadiusMultiplier = FloatProvider::parse(v);
        if (const JsonValue* v = cfg->get("floor_level")) floorLevel = FloatProvider::parse(v);
    }
};

struct RavineCarverConfig : CarverConfig {
    FloatProvider verticalRotation;
    struct Shape {
        FloatProvider distanceFactor, thickness, horizontalRadiusFactor;
        int widthSmoothness = 0;
        float verticalRadiusDefaultFactor = 1.0f;
        float verticalRadiusCenterFactor = 0.0f;
    } shape;
    void parse(const JsonValue* cfg, BlockRegistry& blocks) {
        parseCommon(cfg, blocks);
        if (const JsonValue* v = cfg->get("vertical_rotation")) verticalRotation = FloatProvider::parse(v);
        if (const JsonValue* s = cfg->get("shape")) {
            if (const JsonValue* v = s->get("distance_factor")) shape.distanceFactor = FloatProvider::parse(v);
            if (const JsonValue* v = s->get("thickness")) shape.thickness = FloatProvider::parse(v);
            if (const JsonValue* v = s->get("horizontal_radius_factor")) shape.horizontalRadiusFactor = FloatProvider::parse(v);
            if (const JsonValue* v = s->get("width_smoothness")) shape.widthSmoothness = (int)v->numVal;
            if (const JsonValue* v = s->get("vertical_radius_default_factor")) shape.verticalRadiusDefaultFactor = (float)v->numVal;
            if (const JsonValue* v = s->get("vertical_radius_center_factor")) shape.verticalRadiusCenterFactor = (float)v->numVal;
        }
    }
};

// ===== CarverContext（Java CarverContext.java：HeightContext + applyMaterialRule）=====
// surface 单点 materialRule 通过回调注入（worldgen_api.cpp 提供实现，复用 SurfaceContext）
struct CarverContext {
    int minY = -64, height = 384;
    Aquifer* aquifer = nullptr;
    BlockRegistry* blocks = nullptr;
    // surface 单点应用：构造 SurfaceContext + initVertical(1,1,fluidHeight,x,y,z,biome) + overworldRule->apply
    // hasFluid ? j+1 : INT32_MIN；返回 block id 或 -1（Java applyMaterialRule 语义）
    std::function<int(int x, int y, int z, bool hasFluid)> applyMaterialRule;
};

// ===== Carver 基类（Java Carver.java）=====
class Carver {
public:
    virtual ~Carver() = default;
    virtual bool shouldCarve(ChunkRandom& random, float probability) const { return random.nextFloat() <= probability; }
    // carve：chunkX/chunkZ = 邻域 chunkPos2（洞穴中心 offset 随机）；target = 当前 chunk（写方块）
    // 调用前须设 targetChunkX/targetChunkZ（ConfiguredCarver::carve 负责）
    virtual bool carve(CarverContext& ctx, const CarverConfig& config, BlockColumn& col,
                       const std::function<std::string(int,int,int)>& biomeAt,
                       ChunkRandom& random, int chunkX, int chunkZ, CarvingMask& mask) = 0;
    int targetChunkX = 0, targetChunkZ = 0; // 当前 chunk（carveRegion 写方块目标 = chunk.getPos()）

    static int getBranchFactor() { return 4; }

    // ChunkSectionPos.getBlockCoord(getBranchFactor()*2-1) = (4*2-1)*16 = 112
    static int branchCoord() { return (getBranchFactor() * 2 - 1) * 16; }

protected:
    // Java Carver.carveRegion（x/y/z double 中心，width/height 半径，chunk 坐标由调用方传入）
    bool carveRegionImpl(CarverContext& ctx, const CarverConfig& config, BlockColumn& col,
                         const std::function<std::string(int,int,int)>& biomeAt,
                         Aquifer& aquifer, double x, double y, double z, double width, double height,
                         CarvingMask& mask,
                         const std::function<bool(double,double,double,int)>& skipPredicate,
                         int chunkX, int chunkZ) {
        double f = 16.0 + width * 2.0;
        double cx = chunkX * 16 + 8.0;   // chunkPos.getCenterX()
        double cz = chunkZ * 16 + 8.0;
        if (std::fabs(x - cx) > f || std::fabs(z - cz) > f) return false;
        // 洞穴中心 x/y/z 用邻域 chunk（chunkX/chunkZ 参数仅用于范围判断）；carveRegion 写方块用 targetChunkX/Z（当前）
        int i2 = targetChunkX * 16;      // 当前 chunk getStartX
        int j2 = targetChunkZ * 16;
        int k = std::max((int)std::floor(x - width) - i2 - 1, 0);
        int l = std::min((int)std::floor(x + width) - i2, 15);
        int m = std::max((int)std::floor(y - height) - 1, ctx.minY + 1);
        int n = 7;                       // hasBelowZeroRetrogen=false
        int o = std::min((int)std::floor(y + height) + 1, ctx.minY + ctx.height - 1 - n);
        int p = std::max((int)std::floor(z - width) - j2 - 1, 0);
        int q = std::min((int)std::floor(z + width) - j2, 15);
        bool bl = false;
        for (int r = k; r <= l; r++) {
            int s = targetChunkX * 16 + r; // 当前 chunk getOffsetX
            double g = (s + 0.5 - x) / width;
            for (int t = p; t <= q; t++) {
                int u = targetChunkZ * 16 + t; // 当前 chunk getOffsetZ
                double h = (u + 0.5 - z) / width;
                if (g * g + h * h >= 1.0) continue;
                bool replacedGrassy = false;
                for (int v = o; v > m; v--) {
                    double w = (v - 0.5 - y) / height;
                    if (skipPredicate && skipPredicate(g, w, h, v)) continue;
                    if (!mask.get(r, v, t)) {
                        mask.set(r, v, t);
                        if (getenv("WG_CARVE_TRACE")) {
                            std::fprintf(stderr, "[CARVE-T] center=(%.1f,%.1f,%.1f) w=%.1f h=%.1f block=(%d,%d,%d) r=%d t=%d v=%d\n",
                                         x, y, z, width, height, s, v, u, r, t, v);
                        }                        bl |= carveAtPoint(ctx, config, col, biomeAt, mask, r, v, t, s, u, aquifer, replacedGrassy);
                    }
                }
            }
        }
        return bl;
    }

    // Java Carver.carveAtPoint（mask 已标记；r/t 局部坐标 0-15，v 世界 y；s/u 世界 x/z）
    bool carveAtPoint(CarverContext& ctx, const CarverConfig& config, BlockColumn& col,
                      const std::function<std::string(int,int,int)>& biomeAt,
                      CarvingMask& mask, int r, int v, int t, int wx, int wz, Aquifer& aquifer,
                      bool& replacedGrassy) {
        int wy = v; // world y
        // Java chunk.setBlockState：isOutOfHeightLimit(pos) → 跳过（洞穴漂移可越界世界高度！）
        // C++ BlockColumn::at 无边界检查——必须手动挡（否则越界写堆破坏）
        if (wy < ctx.minY || wy >= ctx.minY + ctx.height) return false;
        int block = col.at(r, wy, t); // BlockColumn::at 局部 x/z（0-15）
        (void)mask;
        int grassId = ctx.blocks ? ctx.blocks->id("minecraft:grass_block") : -1;
        int myceliumId = ctx.blocks ? ctx.blocks->id("minecraft:mycelium") : -1;
        if (block == grassId || block == myceliumId) replacedGrassy = true;
        if (!canAlwaysCarveBlock(config, block)) return false;
        int state = getState(ctx, config, wx, wy, wz, aquifer); // aquifer 世界坐标
        if (state < 0) return false;
        if (getenv("WG_CARVE_TRACE") && wx == -285 && wz == -254 && wy >= -60 && wy <= -40) {
            std::fprintf(stderr, "[CARVE-T] (%d,%d,%d) block=%d -> state=%d\n", wx, wy, wz, block, state);
        }
        col.at(r, wy, t) = state;
        if (replacedGrassy) {
            int below = col.at(r, wy - 1, t);
            int dirtId = ctx.blocks ? ctx.blocks->id("minecraft:dirt") : -1;
            if (below == dirtId && ctx.applyMaterialRule) {
                bool hasFluid = isFluid(ctx, state);
                int ns = ctx.applyMaterialRule(wx, wy - 1, wz, hasFluid); // materialRule 世界坐标
                if (ns >= 0) col.at(r, wy - 1, t) = ns;
            }
        }
        return true;
    }

    static bool isFluid(CarverContext& ctx, int blockId) {
        // Java !blockState2.getFluidState().isEmpty()：air/water/lava 的 FluidState 判断
        // carver 的 getState 只返回 lava/water/air（aquifer.apply），water/lava 是流体
        int waterId = ctx.blocks ? ctx.blocks->id("minecraft:water") : -1;
        int lavaId = ctx.blocks ? ctx.blocks->id("minecraft:lava") : -1;
        return blockId == waterId || blockId == lavaId;
    }

    // Java Carver.getState：y <= lavaLevel → lava；否则 aquifer.apply(pos, 0.0)
    int getState(CarverContext& ctx, const CarverConfig& config, int x, int y, int z, Aquifer& aquifer) {
        int lavaId = ctx.blocks ? ctx.blocks->id("minecraft:lava") : -1;
        if (y <= config.lavaLevel.getY(ctx.minY, ctx.height)) return lavaId;
        return aquifer.apply(x, y, z, 0.0); // density=0.0（Java sampler.apply(pos, 0.0)）
    }

    // Java Carver.canAlwaysCarveBlock = state.isIn(config.replaceable)
    bool canAlwaysCarveBlock(const CarverConfig& config, int blockId) const {
        for (int id : config.replaceableIds) if (id == blockId) return true;
        return false;
    }

    // Java Carver.canCarveBranch
    static bool canCarveBranch(int chunkX, int chunkZ, double x, double z, int branchIndex, int branchCount, float baseWidth) {
        double d = chunkX * 16 + 8.0; // getCenterX
        double e = chunkZ * 16 + 8.0;
        double f = x - d;
        double g = z - e;
        double h = branchCount - branchIndex;
        double i = baseWidth + 2.0F + 16.0F;
        return f * f + g * g - h * h <= i * i;
    }
};

// ===== CaveCarver（Java CaveCarver.java）=====
class CaveCarver : public Carver {
public:
    int getMaxCaveCount() const { return 15; }

    float getTunnelSystemWidth(ChunkRandom& random) {
        float f = random.nextFloat() * 2.0F + random.nextFloat();
        if (random.nextInt(10) == 0) f *= random.nextFloat() * random.nextFloat() * 3.0F + 1.0F;
        return f;
    }

    double getTunnelSystemHeightWidthRatio() const { return 1.0; }

    bool carve(CarverContext& ctx, const CarverConfig& cfg, BlockColumn& col,
               const std::function<std::string(int,int,int)>& biomeAt,
               ChunkRandom& random, int chunkX, int chunkZ, CarvingMask& mask) override {
        auto& config = static_cast<const CaveCarverConfig&>(cfg);
        int i = branchCoord(); // 112
        int j = random.nextInt(random.nextInt(random.nextInt(getMaxCaveCount()) + 1) + 1);
        if (getenv("WG_CARVE_TRACE")) {
            std::fprintf(stderr, "[CARVE-T] CAVE entry chunk=(%d,%d) caveCount=%d\n", chunkX, chunkZ, j);
        }
        for (int k = 0; k < j; k++) {
            double d = chunkX * 16 + random.nextInt(16);         // getOffsetX(nextInt(16))
            double e = (double)config.y.get(random, ctx.minY, ctx.height);
            double f = chunkZ * 16 + random.nextInt(16);         // getOffsetZ(nextInt(16))
            double g = config.horizontalRadiusMultiplier.get(random);
            double h = config.verticalRadiusMultiplier.get(random);
            double l = config.floorLevel.get(random);
            if (getenv("WG_CARVE_TRACE")) {
                std::fprintf(stderr, "[CARVE-T]   cave#%d start=(%.1f,%.1f,%.1f) hrm=%.3f vrm=%.3f floor=%.3f\n",
                             k, d, e, f, g, h, l);
            }
            auto skipPredicate = [&](double rx, double ry, double rz, int y) -> bool {
                (void)y;
                // isPositionExcluded：scaledRelativeY <= floorLevel || x²+y²+z² >= 1
                return ry <= l || rx * rx + ry * ry + rz * rz >= 1.0;
            };
            int m = 1;
            if (random.nextInt(4) == 0) {
                double n = config.yScale.get(random);
                float o = 1.0F + random.nextFloat() * 6.0F;
                carveCave(ctx, config, col, biomeAt, d, e, f, o, n, mask, skipPredicate);
                m += random.nextInt(4);
            }
            for (int p = 0; p < m; p++) {
                float q = random.nextFloat() * (float)(3.14159265358979323846 * 2);
                float o = (random.nextFloat() - 0.5F) / 4.0F;
                float r = getTunnelSystemWidth(random);
                int s = i - random.nextInt(i / 4);
                carveTunnels(ctx, config, col, biomeAt, random.nextLong(), d, e, f, g, h, r, q, o, 0, s,
                             getTunnelSystemHeightWidthRatio(), mask, skipPredicate);
            }
        }
        return true;
    }

    void carveCave(CarverContext& ctx, const CaveCarverConfig& config, BlockColumn& col,
                   const std::function<std::string(int,int,int)>& biomeAt,
                   double d, double e, double f, float g, double h,
                   CarvingMask& mask,
                   const std::function<bool(double,double,double,int)>& skipPredicate) {
        double i = 1.5 + mathSin((float)(3.14159265358979323846 / 2)) * g;
        double j = i * h;
        // Java carveCave → carveRegion 内部 chunk.getPos() = 当前 chunk（写方块目标）——用 targetChunkX/Z
        carveRegionImpl(ctx, config, col, biomeAt, *ctx.aquifer, d + 1.0, e, f, i, j, mask, skipPredicate, targetChunkX, targetChunkZ);
    }

    void carveTunnels(CarverContext& ctx, const CaveCarverConfig& config, BlockColumn& col,
                      const std::function<std::string(int,int,int)>& biomeAt,
                      long long seed, double x, double y, double z,
                      double horizontalScale, double verticalScale, float width, float yaw, float pitch,
                      int branchStartIndex, int branchCount, double yawPitchRatio,
                      CarvingMask& mask,
                      const std::function<bool(double,double,double,int)>& skipPredicate) {
        // Java CaveCarver.carveTunnels L145: Random.create(seed) = CheckedRandom（48 位 LCG，非 Xoroshiro！）
        // 2026-08-10 根因：C++ 曾误用 XoroshiroRandom → 漂移序列全错 → 挖洞位置不重合
        CheckedRandom random((int64_t)seed);
        int i = random.nextInt(branchCount / 2) + branchCount / 4;
        bool bl = random.nextInt(6) == 0;
        float f = 0.0F, g = 0.0F;
        for (int j = branchStartIndex; j < branchCount; j++) {
            // Java: MathHelper.sin((float)Math.PI * j / branchCount)——(float)Math.PI=3.1415927F 全程 float！
            // C++ 旧: double π 全程 double → sin 表索引差 → 111 步漂移累积数格（挖洞位置偏移根因）
            double d = 1.5 + mathSin(3.1415927F * j / branchCount) * width;
            double e = d * yawPitchRatio;
            float h = mathCos(pitch);
            x += mathCos(yaw) * h;
            y += mathSin(pitch);
            z += mathSin(yaw) * h;
            pitch *= bl ? 0.92F : 0.7F;
            pitch += g * 0.1F;
            yaw += f * 0.1F;
            g *= 0.9F;
            f *= 0.75F;
            g += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 2.0F;
            f += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 4.0F;
            if (j == i && width > 1.0F) {
                carveTunnels(ctx, config, col, biomeAt, random.nextLong(), x, y, z, horizontalScale, verticalScale,
                             random.nextFloat() * 0.5F + 0.5F, yaw - (float)(3.14159265358979323846 / 2), pitch / 3.0F,
                             j, branchCount, 1.0, mask, skipPredicate);
                carveTunnels(ctx, config, col, biomeAt, random.nextLong(), x, y, z, horizontalScale, verticalScale,
                             random.nextFloat() * 0.5F + 0.5F, yaw + (float)(3.14159265358979323846 / 2), pitch / 3.0F,
                             j, branchCount, 1.0, mask, skipPredicate);
                return;
            }
            if (random.nextInt(4) != 0) {
                if (!canCarveBranch(targetChunkX, targetChunkZ, x, z, j, branchCount, width)) return;
                carveRegionImpl(ctx, config, col, biomeAt, *ctx.aquifer, x, y, z, d * horizontalScale, e * verticalScale,
                                mask, skipPredicate, targetChunkX, targetChunkZ);
            }
        }
    }
};

// ===== RavineCarver（Java RavineCarver.java）=====
class RavineCarver : public Carver {
public:
    bool carve(CarverContext& ctx, const CarverConfig& cfg, BlockColumn& col,
               const std::function<std::string(int,int,int)>& biomeAt,
               ChunkRandom& random, int chunkX, int chunkZ, CarvingMask& mask) override {
        auto& config = static_cast<const RavineCarverConfig&>(cfg);
        int i = branchCoord(); // 112
        double d = chunkX * 16 + random.nextInt(16);
        int j = config.y.get(random, ctx.minY, ctx.height);
        double e = chunkZ * 16 + random.nextInt(16);
        float f = random.nextFloat() * (float)(3.14159265358979323846 * 2);
        float g = config.verticalRotation.get(random);
        double h = config.yScale.get(random);
        float k = config.shape.thickness.get(random);
        int l = (int)(i * config.shape.distanceFactor.get(random));
        carveRavine(ctx, config, col, biomeAt, random.nextLong(), d, j, e, k, f, g, 0, l, h, mask);
        return true;
    }

    void carveRavine(CarverContext& ctx, const RavineCarverConfig& config, BlockColumn& col,
                     const std::function<std::string(int,int,int)>& biomeAt,
                     long long seed, double x, double y, double z, float width, float yaw, float pitch,
                     int branchStartIndex, int branchCount, double yawPitchRatio,
                     CarvingMask& mask) {
        // Java RavineCarver.carveRavine L65: Random.create(seed) = CheckedRandom（48 位 LCG）
        CheckedRandom random((int64_t)seed);
        std::vector<float> fs = createHorizontalStretchFactors(ctx, config, random);
        float f = 0.0F, g = 0.0F;
        for (int i = branchStartIndex; i < branchCount; i++) {
            double d = 1.5 + mathSin(i * (float)3.14159265358979323846 / branchCount) * width;
            double e = d * yawPitchRatio;
            d *= config.shape.horizontalRadiusFactor.get(random);
            e = getVerticalScale(config, random, e, branchCount, i);
            float h = mathCos(pitch);
            float j = mathSin(pitch);
            x += mathCos(yaw) * h;
            y += j;
            z += mathSin(yaw) * h;
            pitch *= 0.7F;
            pitch += g * 0.05F;
            yaw += f * 0.05F;
            g *= 0.8F;
            f *= 0.5F;
            g += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 2.0F;
            f += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 4.0F;
            if (random.nextInt(4) != 0) {
                if (!canCarveBranch(targetChunkX, targetChunkZ, x, z, i, branchCount, width)) return;
                auto skip = [&](double rx, double ry, double rz, int yv) -> bool {
                    return isPositionExcluded(ctx, fs, rx, ry, rz, yv);
                };
                carveRegionImpl(ctx, config, col, biomeAt, *ctx.aquifer, x, y, z, d, e, mask, skip, targetChunkX, targetChunkZ);
            }
        }
    }

    std::vector<float> createHorizontalStretchFactors(CarverContext& ctx, const RavineCarverConfig& config, CheckedRandom& random) {
        int i = ctx.height;
        std::vector<float> fs((size_t)i, 1.0F);
        float f = 1.0F;
        for (int j = 0; j < i; j++) {
            if (j == 0 || random.nextInt(config.shape.widthSmoothness) == 0) {
                f = 1.0F + random.nextFloat() * random.nextFloat();
            }
            fs[j] = f * f; // Java RavineCarver.java L122: fs[j] = f * f（缺平方曾导致 ravine 挖更宽）
        }
        return fs;
    }

    // Java getVerticalScale（RavineCarver.java L128-132）：
    //   f = 1.0 - abs(0.5 - branchIndex/branchCount) * 2.0；g = default + center * f
    double getVerticalScale(const RavineCarverConfig& config, CheckedRandom& random, double pitch,
                            int branchCount, int i) {
        float f = 1.0F - std::fabs(0.5F - (float)i / branchCount) * 2.0F;
        float g = config.shape.verticalRadiusDefaultFactor + config.shape.verticalRadiusCenterFactor * f;
        // MathHelper.nextBetween(random, 0.75F, 1.0F) = nextFloat()*0.25+0.75
        float nb = random.nextFloat() * 0.25F + 0.75F;
        return g * pitch * nb;
    }

    // Java isPositionExcluded（RavineCarver）：(rx²+rz²)*fs[y-1] + ry²/6 >= 1
    bool isPositionExcluded(CarverContext& ctx, const std::vector<float>& fs,
                            double scaledRelativeX, double scaledRelativeY, double scaledRelativeZ, int y) {
        int i = y - ctx.minY;
        if (i - 1 < 0 || i - 1 >= (int)fs.size()) return true;
        return (scaledRelativeX * scaledRelativeX + scaledRelativeZ * scaledRelativeZ) * fs[i - 1]
               + scaledRelativeY * scaledRelativeY / 6.0 >= 1.0;
    }
};

// ===== ConfiguredCarver 包装（type 分派）=====
// carverStep.air 的 configured_carver JSON：type = minecraft:cave / cave_extra_underground / canyon
struct ConfiguredCarver {
    enum class Type { CAVE, RAVINE };
    Type type = Type::CAVE;
    CaveCarverConfig caveCfg;
    RavineCarverConfig ravineCfg;
    float probability = 0.0f;

    bool shouldCarve(ChunkRandom& random) const { return random.nextFloat() <= probability; }

    bool carve(CarverContext& ctx, BlockColumn& col,
               const std::function<std::string(int,int,int)>& biomeAt,
               ChunkRandom& random, int chunkX, int chunkZ, int targetX, int targetZ, CarvingMask& mask) const {
        if (type == Type::CAVE) {
            CaveCarver c;
            c.targetChunkX = targetX; c.targetChunkZ = targetZ;
            return c.carve(ctx, caveCfg, col, biomeAt, random, chunkX, chunkZ, mask);
        } else {
            RavineCarver c;
            c.targetChunkX = targetX; c.targetChunkZ = targetZ;
            return c.carve(ctx, ravineCfg, col, biomeAt, random, chunkX, chunkZ, mask);
        }
    }

    // 解析 configured_carver JSON（type + config）
    static ConfiguredCarver parse(const JsonValue& root, BlockRegistry& blocks) {
        ConfiguredCarver cc;
        std::string typeName = root.get("type") ? root.get("type")->strVal : "";
        const JsonValue* cfg = root.get("config");
        if (typeName.find("canyon") != std::string::npos) {
            cc.type = Type::RAVINE;
            cc.ravineCfg.parse(cfg, blocks);
            cc.probability = cc.ravineCfg.probability;
        } else {
            cc.type = Type::CAVE;
            cc.caveCfg.parse(cfg, blocks);
            cc.probability = cc.caveCfg.probability;
        }
        return cc;
    }
};

} // namespace wg
