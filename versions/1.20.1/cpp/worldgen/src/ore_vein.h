// ore_vein.h — OreVeinSampler 复刻（1.20.1，88 行 Java 翻译）
// 矿脉：veinToggle > 0 → 铜矿脉(y 0..50)；≤ 0 → 铁矿脉(y -60..-8)
#pragma once
#include <cmath>
#include <string>

#include "blocks.h"
#include "density.h"
#include "xoroshiro.h"

namespace wg {

class OreVeinSampler {
public:
    struct VeinType {
        std::string ore, rawOreBlock, stone;
        int minY, maxY;
    };

    OreVeinSampler(DF veinToggle_, DF veinRidged_, DF veinGap_,
                   const XoroshiroRandom::Splitter& splitter_, const BlockRegistry* blocks_)
        : veinToggle(std::move(veinToggle_)), veinRidged(std::move(veinRidged_)),
          veinGap(std::move(veinGap_)), splitter(splitter_), blocks(blocks_) {
        copper = {"minecraft:copper_ore", "minecraft:raw_copper_block", "minecraft:granite", 0, 50};
        iron = {"minecraft:deepslate_iron_ore", "minecraft:raw_iron_block", "minecraft:tuff", -60, -8};
    }

    // 返回矿脉方块 BlockId；不适用返回 -1
    int apply(int blockX, int blockY, int blockZ) {
        NoisePos pos;
        pos.x = blockX; pos.y = blockY; pos.z = blockZ;
        double d = veinToggle->sample(pos);
        static int vc = 0;
        if (vc < 5 && blockY >= -60 && blockY <= 51) { std::fprintf(stderr, "[ov] bX=%d bY=%d bZ=%d toggle=%.4f\n", blockX, blockY, blockZ, d); vc++; }
        const VeinType& t = (d > 0.0) ? copper : iron;
        double e = std::abs(d);
        int j = t.maxY - blockY;
        int k = blockY - t.minY;
        if (k >= 0 && j >= 0) {
            int l = std::min(j, k);
            double f = lerpClamp((double)l, 0.0, 20.0, -0.2, 0.0);
            if (e + f < 0.4) return -1;
            XoroshiroRandom random = splitter.split(blockX, blockY, blockZ);
            if (random.nextFloat() > 0.7F) return -1;
            if (veinRidged->sample(pos) >= 0.0) return -1;
            double g = lerpClamp(e, 0.4, 0.6, 0.1, 0.3);
            if (random.nextFloat() < g && veinGap->sample(pos) > -0.3) {
                return random.nextFloat() < 0.02F ? blocks->id(t.rawOreBlock) : blocks->id(t.ore);
            }
            return blocks->id(t.stone);
        }
        return -1;
    }

private:
    DF veinToggle, veinRidged, veinGap;
    XoroshiroRandom::Splitter splitter;
    const BlockRegistry* blocks;
    VeinType copper, iron;

    static double lerpClamp(double value, double fromStart, double fromEnd, double toStart, double toEnd) {
        double t = (value - fromStart) / (fromEnd - fromStart);
        t = t < 0 ? 0 : (t > 1 ? 1 : t);
        return toStart + t * (toEnd - toStart);
    }
};

} // namespace wg

