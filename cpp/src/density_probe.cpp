// density_probe：3c/3d 验证工具。
// 从 vanilla worldgen JSON 构建 finalDensity 树，对 density 文件采样点求值对比。
// 用法: density_probe <seed> <vanilla.density文件> <worldgen数据目录>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <chrono>
#include <string>
#include <vector>
#include <fstream>
#include <sstream>
#include <cmath>
#include "json.h"
#include "density.h"
#include "density_builder.h"

using namespace wg;

// 噪声参数表（BuiltinNoiseParameters 1.20.1）
static std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
    std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> m;
    auto add = [&](const char* key, int32_t oct, std::initializer_list<double> amps) {
        m[std::string("minecraft:") + key] = DoublePerlinNoiseSampler::NoiseParameters{oct, std::vector<double>(amps)};
    };
    add("temperature", -10, {1.5, 0.0, 1.0, 0.0, 0.0, 0.0});
    add("vegetation", -8, {1.0, 1.0, 0.0, 0.0, 0.0, 0.0});
    add("continentalness", -9, {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0});
    add("erosion", -9, {1.0, 1.0, 0.0, 1.0, 1.0});
    add("ridge", -7, {1.0, 2.0, 1.0, 0.0, 0.0, 0.0});
    add("offset", -3, {1.0, 1.0, 1.0, 0.0});
    add("aquifer_barrier", -3, {1.0});
    add("aquifer_fluid_level_floodedness", -7, {1.0});
    add("aquifer_lava", -1, {1.0});
    add("aquifer_fluid_level_spread", -5, {1.0});
    add("pillar", -7, {1.0, 1.0});
    add("pillar_rareness", -8, {1.0});
    add("pillar_thickness", -8, {1.0});
    add("spaghetti_2d", -7, {1.0});
    add("spaghetti_2d_elevation", -8, {1.0});
    add("spaghetti_2d_modulator", -11, {1.0});
    add("spaghetti_2d_thickness", -11, {1.0});
    add("spaghetti_3d_1", -7, {1.0});
    add("spaghetti_3d_2", -7, {1.0});
    add("spaghetti_3d_rarity", -11, {1.0});
    add("spaghetti_3d_thickness", -8, {1.0});
    add("spaghetti_roughness", -5, {1.0});
    add("spaghetti_roughness_modulator", -8, {1.0});
    add("cave_entrance", -7, {0.4, 0.5, 1.0});
    add("cave_layer", -8, {1.0});
    add("cave_cheese", -8, {0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0});
    add("ore_veininess", -8, {1.0});
    add("ore_vein_a", -7, {1.0});
    add("ore_vein_b", -7, {1.0});
    add("ore_gap", -5, {1.0});
    add("noodle", -8, {1.0});
    add("noodle_thickness", -8, {1.0});
    add("noodle_ridge_a", -7, {1.0});
    add("noodle_ridge_b", -7, {1.0});
    add("jagged", -16, {1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0});
    add("surface", -6, {1.0, 1.0, 1.0});
    add("surface_secondary", -6, {1.0, 1.0, 0.0, 1.0});
    add("clay_bands_offset", -8, {1.0});
    add("badlands_pillar", -2, {1.0, 1.0, 1.0, 1.0});
    add("badlands_pillar_roof", -8, {1.0});
    add("badlands_surface", -6, {1.0, 1.0, 1.0});
    add("iceberg_pillar", -6, {1.0, 1.0, 1.0, 1.0});
    add("iceberg_pillar_roof", -3, {1.0});
    add("iceberg_surface", -6, {1.0, 1.0, 1.0});
    add("surface_swamp", -2, {1.0});
    add("calcite", -9, {1.0, 1.0, 1.0, 1.0});
    add("gravel", -8, {1.0, 1.0, 1.0, 1.0});
    add("powder_snow", -6, {1.0, 1.0, 1.0, 1.0});
    add("packed_ice", -7, {1.0, 1.0, 1.0, 1.0});
    add("ice", -4, {1.0, 1.0, 1.0, 1.0});
    add("soul_sand_layer", -8, {1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334});
    add("gravel_layer", -8, {1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334});
    add("patch", -5, {1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334});
    add("netherrack", -3, {1.0, 0.0, 0.0, 0.35});
    add("nether_wart", -3, {1.0, 0.0, 0.0, 0.9});
    add("nether_state_selector", -4, {1.0});
    return m;
}

static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("cannot open " + path);
    std::stringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 4) {
        std::fprintf(stderr, "usage: density_probe <seed> <vanilla.density> <worldgen dir>\n");
        return 1;
    }
    uint64_t seed = std::strtoull(argv[1], nullptr, 10);
    std::string densityPath = argv[2];
    std::string wgDir = argv[3];

    auto noiseParams = buildNoiseParams();
    DensityBuilder builder(seed, noiseParams);

    // 惰性加载：引用 overworld/<name> 时从文件读取
    std::string dfDir = wgDir + "/data/minecraft/worldgen/density_function/overworld/";
    builder.externalLoader = [&](const std::string& fullRef, const std::string& name) -> DF {
        std::string path = dfDir + name + ".json";
        std::ifstream probe(path);
        if (!probe.good()) return nullptr;
        return builder.parseFile(fullRef, readFile(path));
    };

    // 1. 注册 overworld density functions（顶层文件）
    std::vector<std::string> dfFiles = {
        "base_3d_noise", "continents", "depth", "erosion", "factor",
        "jaggedness", "offset", "ridges", "ridges_folded", "sloped_cheese",
        "caves/entrances", "caves/noodle", "caves/pillars",
        "caves/spaghetti_2d_thickness_modulator", "caves/spaghetti_2d",
        "caves/spaghetti_roughness_function",
    };
    // 两阶段注册：先占位（循环引用保护），再填充
    for (const auto& f : dfFiles) {
        builder.registerFunction("minecraft:overworld/" + f, std::make_shared<DensityBuilder::LazyRef>());
    }
    for (const auto& f : dfFiles) {
        std::string path = dfDir + f + ".json";
        if (std::ifstream(path).good()) {
            auto df = builder.parseFile("minecraft:overworld/" + f, readFile(path));
            builder.registerFunction("minecraft:overworld/" + f, df);
            std::printf("registered overworld/%s\n", f.c_str());
        } else {
            std::printf("SKIP overworld/%s (no file)\n", f.c_str());
        }
    }

    // 2. 解析 overworld.json 的 final_density
    std::string settingsPath = wgDir + "/data/minecraft/worldgen/noise_settings/overworld.json";
    JsonParser sp(readFile(settingsPath));
    JsonValue settings = sp.parse();
    const JsonValue* router = settings.get("noise_router");
    const JsonValue* finalDensity = router->get("final_density");
    DF tree = builder.buildNode(*finalDensity);
    std::printf("final_density tree built\n");

    // 3. 读取 vanilla density 文件逐点对比（同时计时）
    auto t0 = std::chrono::steady_clock::now();
    FILE* f = fopen(densityPath.c_str(), "rb");
    if (!f) { std::perror("open density"); return 1; }
    uint32_t magic; uint64_t vseed; int32_t size, xzI, yI;
    std::fread(&magic, 4, 1, f);
    std::fread(&vseed, 8, 1, f);
    std::fread(&size, 4, 1, f);
    std::fread(&xzI, 4, 1, f);
    std::fread(&yI, 4, 1, f);
    // 字节序：Java DataOutputStream 大端 → 手动转换
    auto be32 = [](uint32_t v) { return ((v & 0xFF) << 24) | ((v & 0xFF00) << 8) | ((v >> 8) & 0xFF00) | ((v >> 24) & 0xFF); };
    auto be64 = [](uint64_t v) {
        return ((v & 0xFFULL) << 56) | ((v & 0xFF00ULL) << 40) | ((v & 0xFF0000ULL) << 24) | ((v & 0xFF000000ULL) << 8) |
               ((v >> 8) & 0xFF000000ULL) | ((v >> 24) & 0xFF0000ULL) | ((v >> 40) & 0xFF00ULL) | ((v >> 56) & 0xFFULL);
    };
    auto f64be = [&](double d) { uint64_t u; std::memcpy(&u, &d, 8); u = be64(u); std::memcpy(&d, &u, 8); return d; };
    auto i32be = [&](int32_t i) { uint32_t u; std::memcpy(&u, &i, 4); u = be32(u); std::memcpy(&i, &u, 4); return i; };
    // 头部转换
    magic = be32(magic);
    vseed = be64(vseed);
    size = i32be(size);
    xzI = i32be(xzI);
    yI = i32be(yI);
    std::printf("density file: magic=0x%08X seed=%lld size=%d xzI=%d yI=%d\n",
                magic, (long long)vseed, size, xzI, yI);

    int64_t match = 0, total = 0;
    double maxErr = 0, sumErr = 0;
    double worstVal[2] = {0, 0};
    std::map<int32_t, std::pair<int64_t, int64_t>> byY; // y -> (match, total)
    for (int c = 0; c < size * size; c++) {
        int32_t cx = i32be(0), cz = i32be(0);
        std::fread(&cx, 4, 1, f); std::fread(&cz, 4, 1, f);
        cx = i32be(cx); cz = i32be(cz);
        int32_t sx = i32be(0), sy = i32be(0), sz = i32be(0);
        std::fread(&sx, 4, 1, f); std::fread(&sy, 4, 1, f); std::fread(&sz, 4, 1, f);
        sx = i32be(sx); sy = i32be(sy); sz = i32be(sz);
        int32_t minY = i32be(0), height = i32be(0);
        std::fread(&minY, 4, 1, f); std::fread(&height, 4, 1, f);
        minY = i32be(minY); height = i32be(height);
        NoisePos pos;
        for (int y = 0; y < sy; y++) {
            for (int z = 0; z < sz; z++) {
                for (int x = 0; x < sx; x++) {
                    double v;
                    std::fread(&v, 8, 1, f);
                    v = f64be(v);
                    pos.x = cx * 16 + x * xzI;
                    pos.z = cz * 16 + z * xzI;
                    pos.y = minY + y * yI;
                    double got = tree->sample(pos);
                    double err = std::abs(got - v);
                    total++;
                    if (err < 1e-9) match++;
                    byY[pos.y].first += (err < 1e-9) ? 1 : 0;
                    byY[pos.y].second++;
                    if (err > maxErr) { maxErr = err; worstVal[0] = v; worstVal[1] = got; }
                    sumErr += err;
                }
            }
        }
    }
    std::fclose(f);
    auto t1 = std::chrono::steady_clock::now();
    double ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    std::printf("C++ density eval: %lld points in %.2f ms (%.2f ns/point, 16 chunks => %.2f ms/chunk)\n",
                total, ms, ms * 1e6 / total, ms / 16);
    std::printf("match=%lld/%lld (%.4f%%) maxErr=%.9g (vanilla=%.6f cpp=%.6f) avgErr=%.9g\n",
                match, total, 100.0 * match / total, maxErr, worstVal[0], worstVal[1], sumErr / total);
    std::printf("--- by Y layer ---\n");
    for (auto& [y, mt] : byY) {
        if (mt.second == 0) continue;
        std::printf("y=%4d match=%lld/%lld (%.1f%%)\n", y, mt.first, mt.second, 100.0 * mt.first / mt.second);
    }
    return 0;
}
