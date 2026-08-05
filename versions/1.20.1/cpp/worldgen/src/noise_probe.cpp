// noise_probe：3a 验证工具。
// 复刻 NoiseConfig 的 randomDeriver 派生链，对指定 noise key 采样输出。
// 用法: noise_probe <seed> <count>   (输出到 stdout: key x y z value)
#include <cstdio>
#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <cmath>
#include "noise.h"

using namespace wg;

struct NoiseDef {
    const char* key;         // registry key 全名（含命名空间）
    int32_t firstOctave;
    std::vector<double> amps;
};

static const std::vector<NoiseDef> NOISES = {
    {"minecraft:temperature", -10, {1.5, 0.0, 1.0, 0.0, 0.0, 0.0}},
    {"minecraft:vegetation", -8, {1.0, 1.0, 0.0, 0.0, 0.0, 0.0}},
    {"minecraft:continentalness", -9, {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0}},
    {"minecraft:erosion", -9, {1.0, 1.0, 0.0, 1.0, 1.0}},
    {"minecraft:ridge", -7, {1.0, 2.0, 1.0, 0.0, 0.0, 0.0}},
    {"minecraft:offset", -3, {1.0, 1.0, 1.0, 0.0}},
    {"minecraft:aquifer_barrier", -3, {1.0}},
    {"minecraft:aquifer_fluid_level_floodedness", -7, {1.0}},
    {"minecraft:aquifer_lava", -1, {1.0}},
    {"minecraft:aquifer_fluid_level_spread", -5, {1.0}},
    {"minecraft:pillar", -7, {1.0, 1.0}},
    {"minecraft:spaghetti_2d", -7, {1.0}},
    {"minecraft:spaghetti_2d_elevation", -8, {1.0}},
    {"minecraft:spaghetti_2d_modulator", -11, {1.0}},
    {"minecraft:spaghetti_2d_thickness", -11, {1.0}},
    {"minecraft:spaghetti_3d_1", -7, {1.0}},
    {"minecraft:spaghetti_3d_2", -7, {1.0}},
    {"minecraft:spaghetti_3d_rarity", -11, {1.0}},
    {"minecraft:spaghetti_3d_thickness", -8, {1.0}},
    {"minecraft:spaghetti_roughness", -5, {1.0}},
    {"minecraft:spaghetti_roughness_modulator", -8, {1.0}},
    {"minecraft:cave_entrance", -7, {0.4, 0.5, 1.0}},
    {"minecraft:cave_layer", -8, {1.0}},
    {"minecraft:cave_cheese", -8, {0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0}},
    {"minecraft:ore_veininess", -8, {1.0}},
    {"minecraft:ore_vein_a", -7, {1.0}},
    {"minecraft:ore_vein_b", -7, {1.0}},
    {"minecraft:ore_gap", -5, {1.0}},
    {"minecraft:noodle", -8, {1.0}},
    {"minecraft:noodle_thickness", -8, {1.0}},
    {"minecraft:noodle_ridge_a", -7, {1.0}},
    {"minecraft:noodle_ridge_b", -7, {1.0}},
    {"minecraft:jagged", -16, {1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0}},
    {"minecraft:surface", -6, {1.0, 1.0, 1.0}},
    {"minecraft:surface_secondary", -6, {1.0, 1.0, 0.0, 1.0}},
    {"minecraft:clay_bands_offset", -8, {1.0}},
    {"minecraft:badlands_pillar", -2, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:badlands_pillar_roof", -8, {1.0}},
    {"minecraft:badlands_surface", -6, {1.0, 1.0, 1.0}},
    {"minecraft:iceberg_pillar", -6, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:iceberg_pillar_roof", -3, {1.0}},
    {"minecraft:iceberg_surface", -6, {1.0, 1.0, 1.0}},
    {"minecraft:surface_swamp", -2, {1.0}},
    {"minecraft:calcite", -9, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:gravel", -8, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:powder_snow", -6, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:packed_ice", -7, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:ice", -4, {1.0, 1.0, 1.0, 1.0}},
    {"minecraft:soul_sand_layer", -8, {1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334}},
    {"minecraft:gravel_layer", -8, {1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334}},
    {"minecraft:patch", -5, {1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334}},
    {"minecraft:netherrack", -3, {1.0, 0.0, 0.0, 0.35}},
    {"minecraft:nether_wart", -3, {1.0, 0.0, 0.0, 0.9}},
    {"minecraft:nether_state_selector", -4, {1.0}},
};

int main(int argc, char** argv) {
    if (argc < 3) {
        std::fprintf(stderr, "usage: noise_probe <seed> <count>\n");
        return 1;
    }
    uint64_t seed = std::strtoull(argv[1], nullptr, 10);
    int count = std::atoi(argv[2]);

    // NoiseConfig: randomDeriver = ChunkRandom.RandomProvider.XOROSHIRO.create(seed).nextSplitter()
    XoroshiroRandom base(seed);
    auto randomDeriver = base.nextSplitter();
    std::printf("===DEBUG_SPLITTER seedLo: %lld, seedHi: %lld===\n",
                (long long)randomDeriver.seedLo, (long long)randomDeriver.seedHi);

    // 生成采样点网格（与 Java 探针相同：x,z in [0,64) step 4, y in [-64,320) step 8 → 固定 100 点）
    struct Pt { double x, y, z; };
    std::vector<Pt> pts;
    for (int i = 0; i < count; i++) {
        double x = (i * 37) % 128;
        double z = (i * 73) % 128;
        double y = -64 + (i * 29) % 384;
        pts.push_back({x, y, z});
    }

    for (const auto& nd : NOISES) {
        // createNoiseSampler: DoublePerlinNoiseSampler.create(randomDeriver.split(key), params)
        DoublePerlinNoiseSampler::NoiseParameters params{nd.firstOctave, nd.amps};
        auto sampler = randomDeriver.split(nd.key);
        DoublePerlinNoiseSampler noise(sampler, params);
        if (std::string(nd.key) == "minecraft:temperature") {
            const auto& p0 = noise.firstSampler.octaveSamplers[0];
            const auto& p1 = noise.secondSampler.octaveSamplers[0];
            std::printf("===DEBUG_TEMP xo=%.3f, yo=%.3f, zo=%.3f, p0=%d, p255=%d / xo=%.3f, yo=%.3f, zo=%.3f, p0=%d, p255=%d===\n",
                        p0->originX, p0->originY, p0->originZ, (int)p0->permutation[0], (int)p0->permutation[255],
                        p1->originX, p1->originY, p1->originZ, (int)p1->permutation[0], (int)p1->permutation[255]);
        }
        for (const auto& p : pts) {
            double v = noise.sample(p.x, p.y, p.z);
            std::printf("%s %d %d %d %.17g\n", nd.key, (int)p.x, (int)p.y, (int)p.z, v);
        }
    }
    return 0;
}
