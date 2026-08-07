// tbands_test.cpp — 一次性验证：C++ clay_bands_offset 采样 + getTerracottaBlock vs Java
#include <cstdio>
#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "blocks.h"
#include "density.h"
#include "surface.h"

using namespace wg;

// 复刻 worldgen_api.cpp buildNoiseParams 的派生链（仅需 clay_bands_offset + surface）
static void fillNoiseSamplers(std::map<std::string, DoublePerlinNoiseSampler>& samplers,
                              const XoroshiroRandom::Splitter& splitter) {
    struct ND { const char* key; int32_t fo; std::vector<double> amps; };
    static const ND defs[] = {
        {"minecraft:clay_bands_offset", -8, {1.0}},
        {"minecraft:surface", -6, {1.0, 1.0, 1.0}},
        {"minecraft:surface_secondary", -6, {1.0, 1.0, 0.0, 1.0}},
    };
    for (const auto& nd : defs) {
        DoublePerlinNoiseSampler::NoiseParameters params{nd.fo, nd.amps};
        XoroshiroRandom rnd = splitter.split(nd.key);
        samplers[nd.key] = DoublePerlinNoiseSampler(rnd, params);
    }
}

int main(int argc, char** argv) {
    uint64_t seed = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 8576294172403134396ULL;
    XoroshiroRandom base(seed);
    auto splitter = base.nextSplitter();
    std::map<std::string, DoublePerlinNoiseSampler> samplers;
    fillNoiseSamplers(samplers, splitter);

    // clay_bands_offset 采样对比（Java 用 sample(x, 0.0, z)）
    for (int x = 804; x <= 814; x += 2) {
        double v = samplers["minecraft:clay_bands_offset"].sample(x, 0.0, -368);
        double v4 = v * 4.0;
        int iJava = (int)std::floor(v4 + 0.5);   // Java Math.round
        int iCpp = (int)std::lround(v4);          // C++ 现状
        std::printf("x=%d v=%.17g v4=%.6f JavaRound=%d CppLround=%d diff=%d\n",
                    x, v, v4, iJava, iCpp, iJava - iCpp);
    }
    // sampleRunDepth 对比（Java 实测 (804,-368)=4）：surface 噪声 + splitter extra
    {
        double d = samplers["minecraft:surface"].sample(804, 0.0, -368);
        double extra = splitter.split(804, 0, -368).nextDouble();
        int rd = (int)(d * 2.75 + 3.0 + extra * 0.25);
        std::printf("sampleRunDepth(804,-368): d=%.17g extra=%.17g rd=%d (Java rd=4)\n", d, extra, rd);
        for (int x : {804, 805, 806, 808, 810, 812, 814}) {
            double d2 = samplers["minecraft:surface"].sample(x, 0.0, -368);
            double e2 = splitter.split(x, 0, -368).nextDouble();
            std::printf("  rd(%d,-368): d=%.6f extra=%.6f rd=%d\n", x, d2, e2, (int)(d2 * 2.75 + 3.0 + e2 * 0.25));
        }
    }
    // base_3d_noise 正/负坐标对比（负坐标 bug 定位）
    {
        std::map<std::string, DoublePerlinNoiseSampler> all;
        struct ND { const char* key; int32_t fo; std::vector<double> amps; };
        static const ND defs[] = {
            {"minecraft:temperature", -10, {1.5, 0.0, 1.0, 0.0, 0.0, 0.0}},
            {"minecraft:vegetation", -8, {1.0, 1.0, 0.0, 0.0, 0.0, 0.0}},
            {"minecraft:continentalness", -9, {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0}},
            {"minecraft:erosion", -9, {1.0, 1.0, 0.0, 1.0, 1.0}},
            {"minecraft:offset", -3, {1.0, 1.0, 1.0, 0.0}},
            {"minecraft:surface", -6, {1.0, 1.0, 1.0}},
        };
        for (const auto& nd : defs) {
            DoublePerlinNoiseSampler::NoiseParameters params{nd.fo, nd.amps};
            XoroshiroRandom rnd = splitter.split(nd.key);
            all[nd.key] = DoublePerlinNoiseSampler(rnd, params);
        }
        // base_3d_noise（InterpolatedNoiseSampler）：randomDeriver.split("minecraft:terrain")
        XoroshiroRandom terrRnd = splitter.split("minecraft:terrain");
        InterpolatedNoiseDF b3d(terrRnd, 0.25, 0.125, 80.0, 160.0, 8.0);
        for (const auto& pt : std::vector<std::tuple<int,int,int>>{{-244,58,-256},{244,58,256},{-244,-8,-256},{244,-8,256},{-244,58,0},{244,58,0},{-244,0,0},{244,0,0}}) {
            auto [x,y,z] = pt;
            NoisePos np; np.x = x; np.y = y; np.z = z;
            std::printf("PT %d %d %d\n", x, y, z);
            std::printf("  base_3d_noise = %.6f\n", b3d.sample(np));
            for (const auto& nd : defs) {
                std::printf("  %s = %.6f\n", nd.key + 10, all[nd.key].sample(x, y, z));
            }
        }
    }
    return 0;
}
