// ore_probe.cpp — OreVein 矿脉验证探针
// 对照 Java RouterProbe（无插值 veinToggle/veinRidged/veinGap）+ OreVeinSampler.apply 决策链
#include <cstdio>
#include <fstream>
#include <map>
#include <string>
#include <vector>
#include "blocks.h"
#include "density.h"
#include "density_builder.h"
#include "ore_vein.h"
#include "xoroshiro.h"

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
    add("jagged", -16, {1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0});
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
    FILE* f = std::fopen(path.c_str(), "rb");
    if (!f) return {};
    std::vector<char> buf;
    char tmp[4096];
    size_t n;
    while ((n = std::fread(tmp, 1, sizeof(tmp), f)) > 0) buf.insert(buf.end(), tmp, tmp + n);
    std::fclose(f);
    return std::string(buf.data(), buf.size());
}

int main(int argc, char** argv) {
    if (argc < 3) {
        std::fprintf(stderr, "usage: ore_probe <seed> <worldgenDir>\n");
        return 1;
    }
    long long seed = std::atoll(argv[1]);
    std::string wgDir = argv[2];

    auto noiseParams = buildNoiseParams();
    DensityBuilder builder((uint64_t)seed, noiseParams);
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

    std::string settingsPath = wgDir + "/data/minecraft/worldgen/noise_settings/overworld.json";
    JsonValue settings = JsonParser(readFile(settingsPath)).parse();
    const JsonValue* router = settings.get("noise_router");
    struct Comp { const char* name; const char* jsonKey; };
    std::vector<Comp> comps = {
        {"vein_toggle", "vein_toggle"}, {"vein_ridged", "vein_ridged"}, {"vein_gap", "vein_gap"},
    };
    std::map<std::string, DF> fns;
    for (auto& c : comps) {
        const JsonValue* v = router->get(c.jsonKey);
        if (!v) { std::printf("MISSING router component %s\n", c.jsonKey); continue; }
        fns[c.name] = builder.buildNode(*v);
    }

    BlockRegistry blocks;
    if (!blocks.loadFromJson(readFile(wgDir + "/../blocks.json"))) {
        std::fprintf(stderr, "cannot load blocks.json\n");
        return 1;
    }

    // OreVeinSampler（与 worldgen_api.cpp 相同的派生链）
    XoroshiroRandom oreRnd = builder.randomDeriverPublic().split("minecraft:ore");
    OreVeinSampler oreVein(fns["vein_toggle"], fns["vein_ridged"], fns["vein_gap"],
                           oreRnd.nextSplitter(), &blocks);

    // 无插值分量（与 Java RouterProbe 对照用）：buildNode 已含 interpolated 包装，
    // 需要"裸"版本 —— 直接构建 unwrapped 的 range_choice 版本：把 vein_toggle 的 argument 抽出
    const JsonValue* vtNode = router->get("vein_toggle");
    DF vtRaw = vtNode && vtNode->get("argument") ? builder.buildNode(*vtNode->get("argument")) : fns["vein_toggle"];
    const JsonValue* vrNode = router->get("vein_ridged");
    DF vrRaw = vrNode && vrNode->get("argument") ? builder.buildNode(*vrNode->get("argument")) : fns["vein_ridged"];
    const JsonValue* vgNode = router->get("vein_gap");
    DF vgRaw = vgNode && vgNode->get("argument") ? builder.buildNode(*vgNode->get("argument")) : fns["vein_gap"];

    // ore_veininess 噪声直接采样（scale 前/后）——对照 Java
    auto veininess = builder.getNoiseSampler("minecraft:ore_veininess");
    auto veinA = builder.getNoiseSampler("minecraft:ore_vein_a");
    auto veinB = builder.getNoiseSampler("minecraft:ore_vein_b");

    // 采样网格：chunk(200,200) 内 x=3200+z*4, z=3200+col*4, y 从 -64 到 100 每 4 块
    std::printf("#seed %lld\n", (long long)seed);
    NoisePos pos;
    for (int col = 0; col < 4; col++) {
        for (int row = 0; row < 4; row++) {
            pos.x = 200 * 16 + row * 4;
            pos.z = 200 * 16 + col * 4;
            for (int y = -64; y <= 100; y += 4) {
                pos.y = y;
                double vtRawV = vtRaw->sample(pos);
                double vrRawV = vrRaw->sample(pos);
                double vgRawV = vgRaw->sample(pos);
                double vtInterp = fns["vein_toggle"]->sample(pos);
                double vrInterp = fns["vein_ridged"]->sample(pos);
                double vgInterp = fns["vein_gap"]->sample(pos);
                int block = oreVein.apply(pos.x, pos.y, pos.z);
                double vn = veininess->sample(pos.x * 1.5, pos.y * 1.5, pos.z * 1.5);
                double vnA = veinA->sample(pos.x * 1.5, pos.y * 1.5, pos.z * 1.5);
                double vnB = veinB->sample(pos.x * 1.5, pos.y * 1.5, pos.z * 1.5);
                std::printf("P %d %d %d vt=%.6f vr=%.6f vg=%.6f vtI=%.6f vrI=%.6f vgI=%.6f vn=%.6f vnA=%.6f vnB=%.6f block=%d\n",
                            pos.x, pos.y, pos.z, vtRawV, vrRawV, vgRawV, vtInterp, vrInterp, vgInterp, vn, vnA, vnB, block);
            }
        }
    }
    return 0;
}
