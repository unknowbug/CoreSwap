// router_probe：输出 C++ 构建的 noise router 全分量采样，与 Java RouterProbe 对比。
// 用法: router_probe <seed> <worldgen dir>
#include <cstdio>
#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <fstream>
#include <sstream>
#include <cmath>
#include "json.h"
#include "density.h"
#include "density_builder.h"

using namespace wg;

static std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
    std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> m;
    auto add = [&](const char* key, int32_t oct, std::initializer_list<double> amps) {
        m[std::string("minecraft:") + key] = {oct, std::vector<double>(amps)};
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
    add("pillar", -7, {1.0, 1.0});
    add("pillar_rareness", -8, {1.0});
    add("pillar_thickness", -8, {1.0});
    add("jagged", -16, {1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1});
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
    if (argc < 3) { std::fprintf(stderr, "usage: router_probe <seed> <worldgen dir>\n"); return 1; }
    uint64_t seed = std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];

    auto noiseParams = buildNoiseParams();
    DensityBuilder builder(seed, noiseParams);
    std::string dfDir = wgDir + "/data/minecraft/worldgen/density_function/overworld/";
    builder.externalLoader = [&](const std::string& fullRef, const std::string& name) -> DF {
        std::string path = dfDir + name + ".json";
        std::ifstream probe(path);
        if (!probe.good()) return nullptr;
        return builder.parseFile(fullRef, readFile(path));
    };
    std::vector<std::string> dfFiles = {
        "base_3d_noise", "continents", "depth", "erosion", "factor",
        "jaggedness", "offset", "ridges", "ridges_folded", "sloped_cheese",
        "caves/entrances", "caves/noodle", "caves/pillars",
        "caves/spaghetti_2d_thickness_modulator", "caves/spaghetti_2d",
        "caves/spaghetti_roughness_function",
    };
    for (const auto& f : dfFiles) builder.registerFunction("minecraft:overworld/" + f, std::make_shared<DensityBuilder::LazyRef>());
    for (const auto& f : dfFiles) {
        std::string path = dfDir + f + ".json";
        if (std::ifstream(path).good()) {
            builder.registerFunction("minecraft:overworld/" + f, builder.parseFile("minecraft:overworld/" + f, readFile(path)));
        }
    }

    // noise_router 分量
    std::string settingsPath = wgDir + "/data/minecraft/worldgen/noise_settings/overworld.json";
    JsonValue settings = JsonParser(readFile(settingsPath)).parse();
    const JsonValue* router = settings.get("noise_router");
    struct Comp { const char* name; const char* jsonKey; };
    std::vector<Comp> comps = {
        {"barrier", "barrier"},
        {"temperature", "temperature"},
        {"vegetation", "vegetation"},
        {"continents", "continents"},
        {"erosion", "erosion"},
        {"depth", "depth"},
        {"ridges", "ridges"},
        {"initial_density", "initial_density_without_jaggedness"},
        {"final_density", "final_density"},
        {"vein_toggle", "vein_toggle"},
        {"vein_ridged", "vein_ridged"},
        {"vein_gap", "vein_gap"},
    };
    std::map<std::string, DF> fns;
    for (auto& c : comps) {
        const JsonValue* v = router->get(c.jsonKey);
        if (!v) { std::printf("MISSING router component %s\n", c.jsonKey); continue; }
        fns[c.name] = builder.buildNode(*v);
    }
    // final_density 拆解：min(arg1, arg2)
    const JsonValue* fdNode = router->get("final_density");
    DF fdArg1 = fdNode ? builder.buildNode(*fdNode->get("argument1")) : nullptr;
    DF fdArg2 = fdNode ? builder.buildNode(*fdNode->get("argument2")) : nullptr;

    // 采样点与 Java RouterProbe 一致
    const int count = 16;
    NoisePos pos;
    std::printf("#seed %lld\n", (long long)seed);
    for (int i = 0; i < count; i++) {
        pos.x = 200 * 16 + (i % 4) * 4;
        pos.z = 200 * 16 + (i / 4) * 4;
        pos.y = -64 + (i * 13) % 384;
        std::printf("P %d %d %d\n", pos.x, pos.y, pos.z);
        for (auto& c : comps) {
            auto it = fns.find(c.name);
            if (it == fns.end()) continue;
            double v = it->second->sample(pos);
            std::printf("%s %.17g\n", c.name, v);
        }
        if (fdArg1 && fdArg2) {
            std::printf("fd_arg1 %.17g\n", fdArg1->sample(pos));
            std::printf("fd_arg2 %.17g\n", fdArg2->sample(pos));
        }
        // sloped_cheese（registry 已构建）
        DF sc = builder.getRegistryEntry("minecraft:overworld/sloped_cheese");
        if (sc) {
            std::printf("sloped_cheese %.17g\n", sc->sample(pos));
        }
        // base_3d_noise（InterpolatedNoiseSampler）；random = split("minecraft:terrain")
        XoroshiroRandom b3dRnd = builder.randomDeriverPublic().split("minecraft:terrain");
        InterpolatedNoiseDF b3d(b3dRnd, 0.25, 0.125, 80.0, 160.0, 8.0);
        std::printf("base_3d_noise %.17g\n", b3d.sample(pos));
    }
    return 0;
}
