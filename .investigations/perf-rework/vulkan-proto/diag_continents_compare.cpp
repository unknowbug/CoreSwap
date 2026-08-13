// diag_continents_compare.cpp —— DensityBuilder continents.sample vs 手动复刻
#include <cstdio>
#include <cstring>
#include <vector>
#include <fstream>
#include <sstream>
#include <cmath>
#include <map>
#include "json.h"
#include "density.h"
#include "density_builder.h"
#include "noise.h"
#include "xoroshiro.h"
#include "md5.h"

using namespace wg;
static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary); std::stringstream ss; ss << f.rdbuf(); return ss.str();
}
static std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
    std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> m;
    auto add = [&](const char* k, int32_t o, std::initializer_list<double> a) { m[std::string("minecraft:")+k] = DoublePerlinNoiseSampler::NoiseParameters{o, std::vector<double>(a)}; };
    add("continentalness", -9, {1.0,1.0,2.0,2.0,2.0,1.0,1.0,1.0,1.0});
    add("offset", -3, {1.0,1.0,1.0,0.0});
    add("erosion", -9, {1.0,1.0,0.0,1.0,1.0});
    add("ridge", -7, {1.0,2.0,1.0,0.0,0.0,0.0});
    return m;
}
int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    auto np = buildNoiseParams();
    DensityBuilder builder(worldSeed, np);
    std::string dfDir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/";
    builder.externalLoader = [&](const std::string& r, const std::string& n) -> DF {
        std::ifstream p(dfDir + n + ".json"); if (!p.good()) return nullptr;
        return builder.parseFile(r, readFile(dfDir + n + ".json"));
    };
    builder.registerFunction("minecraft:overworld/continents", std::make_shared<DensityBuilder::LazyRef>());
    auto cdf = builder.parseFile("minecraft:overworld/continents", readFile(dfDir + "continents.json"));
    builder.registerFunction("minecraft:overworld/continents", cdf);
    DF continents = builder.getRegistryEntry("minecraft:overworld/continents");

    // 手动复刻
    auto rd = builder.randomDeriverPublic();
    DoublePerlinNoiseSampler continentalness(rd.split("minecraft:continentalness"), np["minecraft:continentalness"]);
    DoublePerlinNoiseSampler offset(rd.split("minecraft:offset"), np["minecraft:offset"]);

    for (int x : {728, 720, 736}) {
        for (int z : {-428, -432, -420}) {
            int ax = (x >> 2) << 2, az = (z >> 2) << 2;
            double shiftX = offset.sample(ax*0.25, 0.0, az*0.25) * 4.0;
            double shiftZ = offset.sample(az*0.25, ax*0.25, 0.0) * 4.0;
            double manual = continentalness.sample(ax*0.25+shiftX, 0.0, az*0.25+shiftZ);
            NoisePos cp{ax, 0, az};
            double dfb = continents->sample(cp);
            printf("(%d,0,%d): DensityBuilder=%.9f manual=%.9f diff=%.3e\n", ax, az, dfb, manual, fabs(dfb-manual));
        }
    }
    return 0;
}
