// aquifer.h — AquiferSampler.Impl 复刻（1.20.1 版，412 行 Java 翻译）
// 决定密度 < 0 区域的流体/空洞方块
#pragma once
#include <cmath>
#include <cstdint>
#include <memory>
#include <vector>

#include "blocks.h"
#include "density.h"
#include "xoroshiro.h"

namespace wg {

// FluidLevel = (y, blockId)
struct FluidLevel {
    int y = INT32_MAX;
    int block = BlockRegistry::AIR;
    FluidLevel() = default;
    FluidLevel(int y_, int b_) : y(y_), block(b_) {}
};

class Aquifer {
public:
    // 构造参数（对应 Impl 构造）
    Aquifer(DF barrierNoise, DF fluidFloodedness, DF fluidSpread, DF fluidType,
            DF erosionDF, DF depthDF, DF initialDensityDF,
            const XoroshiroRandom::Splitter& splitter,
            const BlockRegistry* blocks,
            int chunkStartX, int chunkStartZ, int minY, int height)
        : barrierNoise(std::move(barrierNoise)), fluidFloodedness(std::move(fluidFloodedness)),
          fluidSpread(std::move(fluidSpread)), fluidType(std::move(fluidType)),
          erosionDF(std::move(erosionDF)), depthDF(std::move(depthDF)),
          initialDensity(std::move(initialDensityDF)), splitter(splitter), blocks(blocks),
          minY(minY), height(height) {
        // 网格范围：chunkPos ±1（floorDiv）
        int startXLocal = floorDiv(chunkStartX, 16) - 1;
        int endXLocal = floorDiv(chunkStartX + 15, 16) + 1;
        int startYLocal = floorDiv(minY, 12) - 1;
        int endYLocal = floorDiv(minY + height, 12) + 1;
        int startZLocal = floorDiv(chunkStartZ, 16) - 1;
        int endZLocal = floorDiv(chunkStartZ + 15, 16) + 1;
        startX = startXLocal;
        startY = startYLocal;
        startZ = startZLocal;
        sizeX = endXLocal - startXLocal + 1;
        sizeY = endYLocal - startYLocal + 1;
        sizeZ = endZLocal - startZLocal + 1;
        int m = sizeX * sizeY * sizeZ;
        blockPositions.assign(m, INT64_MAX);
        waterLevels.assign(m, FluidLevel());
    }

    // apply：density > 0 → -1（石头/null）；否则流体决策
    // 返回 BlockId；-1 表示 null（保持默认方块）
    int apply(int blockX, int blockY, int blockZ, double density) {
        if (density > 0.0) return -1;
        // fluidLevelSampler 默认（主世界）：y < -54 lava；y < 63 water；否则 air
        int fluidBlock = -1;
        int fluidY = -1;
        if (blockY < -54) { fluidBlock = blocks->id("minecraft:lava"); fluidY = -54; }
        else if (blockY < 63) { fluidBlock = blocks->id("minecraft:water"); fluidY = 63; }
        else { fluidBlock = blocks->id("minecraft:air"); fluidY = 63; }
        if (fluidBlock == blocks->id("minecraft:lava")) return fluidBlock;

        int l = floorDiv(blockX - 5, 16);
        int m = floorDiv(blockY + 1, 12);
        int n = floorDiv(blockZ - 5, 16);
        int o = INT32_MAX, p = INT32_MAX, q = INT32_MAX;
        int64_t r = 0, s = 0, t = 0;

        for (int u = 0; u <= 1; u++) {
            for (int v = -1; v <= 1; v++) {
                for (int w = 0; w <= 1; w++) {
                    int x = l + u, y = m + v, z = n + w;
                    int64_t ab = getBlockPos(x, y, z);
                    int ad = unpackX(ab) - blockX;
                    int ae = unpackY(ab) - blockY;
                    int af = unpackZ(ab) - blockZ;
                    int ag = ad * ad + ae * ae + af * af;
                    if (o >= ag) { t = s; s = r; r = ab; q = p; p = o; o = ag; }
                    else if (p >= ag) { t = s; s = ab; q = p; p = ag; }
                    else if (q >= ag) { t = ab; q = ag; }
                }
            }
        }

        FluidLevel fl2 = getWaterLevelAt(r);
        double d = maxDistance(o, p);
        int bs = fl2.block;
        if (d <= 0.0) return bs;
        if (bs == blocks->id("minecraft:water") && getFluidLevel(blockX, blockY - 1, blockZ).block == blocks->id("minecraft:lava")) {
            return bs;
        }

        FluidLevel fl3 = getWaterLevelAt(s);
        MutableDouble md;
        double e = d * calculateDensity(blockX, blockY, blockZ, md, fl2, fl3);
        if (density + e > 0.0) return -1;

        FluidLevel fl4 = getWaterLevelAt(t);
        double f = maxDistance(o, q);
        if (f > 0.0) {
            double g = d * f * calculateDensity(blockX, blockY, blockZ, md, fl2, fl4);
            if (density + g > 0.0) return -1;
        }
        double g2 = maxDistance(p, q);
        if (g2 > 0.0) {
            double h = d * g2 * calculateDensity(blockX, blockY, blockZ, md, fl3, fl4);
            if (density + h > 0.0) return -1;
        }
        return bs;
    }

    // estimateSurfaceHeight（ChunkNoiseSampler）：initialDensityWithoutJaggedness > 0.390625
    int estimateSurfaceHeight(int blockX, int blockZ) {
        NoisePos pos;
        for (int l = minY + height; l >= minY; l -= 8) { // verticalCellBlockCount=8
            pos.x = blockX; pos.y = l; pos.z = blockZ;
            if (initialDensity->sample(pos) > 0.390625) return l;
        }
        return INT32_MAX;
    }

private:
    DF barrierNoise, fluidFloodedness, fluidSpread, fluidType;
    DF erosionDF, depthDF;
    DF initialDensity;
    XoroshiroRandom::Splitter splitter;
    const BlockRegistry* blocks;
    int minY, height;
    int startX, startY, startZ, sizeX, sizeY, sizeZ;
    std::vector<int64_t> blockPositions;
    std::vector<FluidLevel> waterLevels;

    // MutableDouble（barrier 噪声缓存 per apply 调用）
    struct MutableDouble {
        double v = NAN;
        bool has = false;
        double get() { return has ? v : NAN; }
        void set(double d) { v = d; has = true; }
    };

    static int floorDiv(int a, int b) { int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }

    int index(int x, int y, int z) {
        int i = x - startX, j = y - startY, k = z - startZ;
        return (j * sizeZ + k) * sizeX + i;
    }

    static int64_t pack(int x, int y, int z) {
        return ((int64_t)(uint32_t)x << 38) | ((int64_t)(uint32_t)(y & 0xFFF) << 26) | ((int64_t)(uint32_t)z & 0x3FFFFFF);
    }
    static int unpackX(int64_t l) { return (int)(l >> 38); }
    static int unpackY(int64_t l) {
        // 12 位符号扩展（对应 Java BlockPos.unpackLongY：l << 16 >> 52）
        int64_t y12 = (l >> 26) & 0xFFF;
        if (y12 & 0x800) y12 |= ~0xFFFLL;
        return (int)y12;
    }
    static int unpackZ(int64_t l) { return (int)(l & 0x3FFFFFF); }

    int64_t getBlockPos(int x, int y, int z) {
        int aa = index(x, y, z);
        int64_t ab = blockPositions[aa];
        if (ab != INT64_MAX) return ab;
        // Random.random.nextInt(10/9/10)
        XoroshiroRandom random = splitter.split(x, y, z);
        int rx = random.nextInt(10), ry = random.nextInt(9), rz = random.nextInt(10);
        ab = pack(x * 16 + rx, y * 12 + ry, z * 16 + rz);
        blockPositions[aa] = ab;
        return ab;
    }

    static double maxDistance(int i, int a) {
        double d = 25.0;
        return 1.0 - std::abs(a - i) / 25.0;
    }

    double calculateDensity(int blockX, int blockY, int blockZ, MutableDouble& md,
                            const FluidLevel& fl, const FluidLevel& fl2) {
        int bs = fl.block;
        int bs2 = fl2.block;
        bool lavaWater = (bs == blocks->id("minecraft:lava") && bs2 == blocks->id("minecraft:water"))
                      || (bs == blocks->id("minecraft:water") && bs2 == blocks->id("minecraft:lava"));
        if (!lavaWater) {
            int j = std::abs(fl.y - fl2.y);
            if (j == 0) return 0.0;
            double d = 0.5 * (fl.y + fl2.y);
            double e = blockY + 0.5 - d;
            double f = j / 2.0;
            double o = f - std::abs(e);
            double q;
            if (e > 0.0) {
                double p = 0.0 + o;
                q = p > 0.0 ? p / 1.5 : p / 2.5;
            } else {
                double p = 3.0 + o;
                q = p > 0.0 ? p / 3.0 : p / 10.0;
            }
            double r;
            if (!(q < -2.0) && !(q > 2.0)) {
                if (!md.has) {
                    NoisePos pos;
                    pos.x = blockX; pos.y = blockY; pos.z = blockZ;
                    double t = barrierNoise->sample(pos);
                    md.set(t);
                    r = t;
                } else {
                    r = md.v;
                }
            } else {
                r = 0.0;
            }
            return 2.0 * (r + q);
        }
        return 2.0;
    }

    // getWaterLevel(long pos)：缓存网格 fluid level
    FluidLevel getWaterLevelAt(int64_t pos) {
        int i = unpackX(pos), j = unpackY(pos), k = unpackZ(pos);
        int bx = floorDiv(i, 16), by = floorDiv(j, 12), bz = floorDiv(k, 16);
        int o = index(bx, by, bz);
        FluidLevel& fl = waterLevels[o];
        if (fl.y != INT32_MAX) return fl;
        fl = getFluidLevel(i, j, k);
        return fl;
    }

    // getFluidLevel(blockX, blockY, blockZ)：13 邻居 surface 扫描
    FluidLevel getFluidLevel(int blockX, int blockY, int blockZ) {
        static const int OFFSETS[13][2] = {
            {0, 0}, {-2, -1}, {-1, -1}, {0, -1}, {1, -1}, {-3, 0}, {-2, 0},
            {-1, 0}, {1, 0}, {-2, 1}, {-1, 1}, {0, 1}, {1, 1}
        };
        FluidLevel defaultFL = defaultFluidLevel(blockY);
        int i = INT32_MAX;
        int j = blockY + 12;
        int k = blockY - 12;
        bool bl = false;
        for (auto& off : OFFSETS) {
            int l = blockX + off[0] * 16;
            int m = blockZ + off[1] * 16;
            int n = estimateSurfaceHeight(l, m);
            int o = n + 8;
            bool bl2 = (off[0] == 0 && off[1] == 0);
            if (bl2 && k > o) return defaultFL;
            bool bl3 = j > o;
            if (bl3 || bl2) {
                FluidLevel fl2 = defaultFluidLevel(o);
                if (fl2.block != blocks->id("minecraft:air")) {
                    if (bl2) bl = true;
                    if (bl3) return fl2;
                }
            }
            i = std::min(i, n);
        }
        int p = getFluidBlockY(blockX, blockY, blockZ, defaultFL, i, bl);
        return FluidLevel(p, getFluidBlockState(blockX, blockY, blockZ, defaultFL, p));
    }

    // 默认 fluidLevelSampler：y<-54 lava，y<63 water，否则 air
    FluidLevel defaultFluidLevel(int blockY) {
        if (blockY < -54) return FluidLevel(-54, blocks->id("minecraft:lava"));
        if (blockY < 63) return FluidLevel(63, blocks->id("minecraft:water"));
        return FluidLevel(63, blocks->id("minecraft:air"));
    }

    int getFluidBlockY(int blockX, int blockY, int blockZ, const FluidLevel& defaultFL,
                       int surfaceHeightEstimate, bool bl) {
        NoisePos pos;
        pos.x = blockX; pos.y = blockY; pos.z = blockZ;
        double d, e;
        // VanillaBiomeParameters.method_43718：erosion < -0.225 && depth > 0.9
        if (erosionDF->sample(pos) < -0.225 && depthDF->sample(pos) > 0.9) {
            d = -1.0;
            e = -1.0;
        } else {
            int i = surfaceHeightEstimate + 8 - blockY;
            double f = bl ? lerpClamp2((double)i, 0.0, 64.0, 1.0, 0.0) : 0.0;
            double g = clamp(fluidFloodedness->sample(pos), -1.0, 1.0);
            double h = map2(f, 1.0, 0.0, -0.3, 0.8);
            double k = map2(f, 1.0, 0.0, -0.8, 0.4);
            d = g - k;
            e = g - h;
        }
        if (e > 0.0) {
            return defaultFL.y;
        } else if (d > 0.0) {
            return getNoiseBasedFluidLevel(blockX, blockY, blockZ, surfaceHeightEstimate);
        }
        return INT32_MAX; // DimensionType.field_35479（无效层）
    }

    int getNoiseBasedFluidLevel(int blockX, int blockY, int blockZ, int surfaceHeightEstimate) {
        int k = floorDiv(blockX, 16);
        int l = floorDiv(blockY, 40);
        int m = floorDiv(blockZ, 16);
        int n = l * 40 + 20;
        NoisePos pos;
        pos.x = k; pos.y = l; pos.z = m;
        double d = fluidSpread->sample(pos) * 10.0;
        int p = roundDownToMultiple(d, 3);
        int q = n + p;
        return std::min(surfaceHeightEstimate, q);
    }

    int getFluidBlockState(int blockX, int blockY, int blockZ, const FluidLevel& defaultFL, int fluidLevel) {
        int state = defaultFL.block;
        if (fluidLevel <= -10 && fluidLevel != INT32_MAX && state != blocks->id("minecraft:lava")) {
            int k = floorDiv(blockX, 64);
            int l = floorDiv(blockY, 40);
            int m = floorDiv(blockZ, 64);
            NoisePos pos;
            pos.x = k; pos.y = l; pos.z = m;
            double d = fluidType->sample(pos);
            if (std::abs(d) > 0.3) state = blocks->id("minecraft:lava");
        }
        return state;
    }

    static double clamp(double v, double lo, double hi) { return v < lo ? lo : (v > hi ? hi : v); }
    static double lerpClamp2(double value, double a, double b, double c, double d) {
        double t = (value - a) / (b - a);
        t = t < 0 ? 0 : (t > 1 ? 1 : t);
        return c + t * (d - c);
    }
    static double map2(double value, double fromStart, double fromEnd, double toStart, double toEnd) {
        return lerpClamp2(value, fromStart, fromEnd, toStart, toEnd);
    }
    static int roundDownToMultiple(double d, int mult) {
        return (int)std::floor(d / mult) * mult;
    }
};

} // namespace wg
