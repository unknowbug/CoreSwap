// diag_factor_cpu.cpp —— 纯 CPU 诊断：DensityBuilder 构建 factor + 采样
#include <cstdio>
#include <cstring>
#include <vector>
#include <fstream>
#include <sstream>
#include <cmath>
#include <map>
#include <chrono>
#include "json.h"
#include "density.h"
#include "density_builder.h"

using namespace wg;
static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("cannot open " + path);
    std::stringstream ss; ss << f.rdbuf(); return ss.str();
}
static std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
    std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> m;
    auto add = [&](const char* key, int32_t oct, std::initializer_list<double> amps) {
        m[std::string("minecraft:") + key] = DoublePerlinNoiseSampler::NoiseParameters{oct, std::vector<double>(amps)};
    };
    add("continentalness", -9, {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0});
    add("offset", -3, {1.0, 1.0, 1.0, 0.0});
    add("erosion", -9, {1.0, 1.0, 0.0, 1.0, 1.0});
    add("ridge", -7, {1.0, 2.0, 1.0, 0.0, 0.0, 0.0});
    return m;
}
int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    auto noiseParams = buildNoiseParams();
    DensityBuilder builder(worldSeed, noiseParams);
    std::string dfDir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/";
    builder.externalLoader = [&](const std::string& fullRef, const std::string& name) -> DF {
        std::string path = dfDir + name + ".json";
        std::ifstream probe(path);
        if (!probe.good()) return nullptr;
        return builder.parseFile(fullRef, readFile(path));
    };
    for (const char* f : {"continents", "erosion", "ridges", "ridges_folded", "factor"}) {
        builder.registerFunction(std::string("minecraft:overworld/") + f, std::make_shared<DensityBuilder::LazyRef>());
    }
    for (const char* f : {"continents", "erosion", "ridges", "ridges_folded", "factor"}) {
        auto df = builder.parseFile(std::string("minecraft:overworld/") + f, readFile(dfDir + f + ".json"));
        builder.registerFunction(std::string("minecraft:overworld/") + f, df);
    }
    DF factor = builder.getRegistryEntry("minecraft:overworld/factor");
    std::printf("factor DF built\n");
    auto t0 = std::chrono::steady_clock::now();
    for (int i = 0; i < 64; i++) {
        NoisePos pos{728 + (i % 8), -8 + (i / 8), -428};
        double v = factor->sample(pos);
        if (i < 8) std::printf("  sample(%d,%d,%d) = %.9f\n", pos.x, pos.y, pos.z, v);
    }
    auto t1 = std::chrono::steady_clock::now();
    std::printf("64 samples in %lld ms\n", (long long)std::chrono::duration_cast<std::chrono::milliseconds>(t1 - t0).count());
    return 0;
}
