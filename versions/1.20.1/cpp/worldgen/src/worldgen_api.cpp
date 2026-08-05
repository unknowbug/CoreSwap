// worldgen_api.cpp — CoreSwap worldgen C API 实现
#include "worldgen_api.h"

#include <cstdio>
#include <cstring>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>
#include <map>

#include "json.h"
#include "density.h"
#include "density_builder.h"

using namespace wg;

namespace {

// 噪声参数表（BuiltinNoiseParameters 1.20.1）——与 density_probe 一致
std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
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

std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("cannot open " + path);
    std::stringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

constexpr int XZ_INTERVAL = 4;   // density 采样间隔（16 块 / 4 = 4 点）
constexpr int Y_INTERVAL = 8;    // 384 高度 / 8 = 48 点
constexpr int MIN_Y = -64;
constexpr int HEIGHT = 384;
constexpr int SX = 16 / XZ_INTERVAL;   // 4
constexpr int SY = HEIGHT / Y_INTERVAL; // 48
constexpr int SZ = 16 / XZ_INTERVAL;   // 4
constexpr int POINTS_PER_CHUNK = SX * SY * SZ; // 768

struct WorldgenHandle {
    std::unique_ptr<DensityBuilder> builder;
    DF finalDensity;
};

} // namespace

extern "C" {

void* wg_create(int64_t seed, const char* worldgenDir) {
    if (!worldgenDir) return nullptr;
    try {
        auto h = std::make_unique<WorldgenHandle>();
        auto noiseParams = buildNoiseParams();
        h->builder = std::make_unique<DensityBuilder>((uint64_t)seed, noiseParams);

        std::string wgDir = worldgenDir;
        std::string dfDir = wgDir + "/data/minecraft/worldgen/density_function/overworld/";
        h->builder->externalLoader = [&](const std::string& fullRef, const std::string& name) -> DF {
            std::string path = dfDir + name + ".json";
            std::ifstream probe(path);
            if (!probe.good()) return nullptr;
            return h->builder->parseFile(fullRef, readFile(path));
        };

        std::vector<std::string> dfFiles = {
            "base_3d_noise", "continents", "depth", "erosion", "factor",
            "jaggedness", "offset", "ridges", "ridges_folded", "sloped_cheese",
            "caves/entrances", "caves/noodle", "caves/pillars",
            "caves/spaghetti_2d_thickness_modulator", "caves/spaghetti_2d",
            "caves/spaghetti_roughness_function",
        };
        for (const auto& f : dfFiles) {
            h->builder->registerFunction("minecraft:overworld/" + f, std::make_shared<DensityBuilder::LazyRef>());
        }
        for (const auto& f : dfFiles) {
            std::string path = dfDir + f + ".json";
            if (std::ifstream(path).good()) {
                auto df = h->builder->parseFile("minecraft:overworld/" + f, readFile(path));
                h->builder->registerFunction("minecraft:overworld/" + f, df);
            }
        }

        std::string settingsPath = wgDir + "/data/minecraft/worldgen/noise_settings/overworld.json";
        JsonParser sp(readFile(settingsPath));
        JsonValue settings = sp.parse();
        const JsonValue* router = settings.get("noise_router");
        const JsonValue* finalDensity = router->get("final_density");
        h->finalDensity = h->builder->buildNode(*finalDensity);
        return h.release();
    } catch (const std::exception& e) {
        std::fprintf(stderr, "wg_create: %s\n", e.what());
        return nullptr;
    }
}

void wg_destroy(void* handle) {
    delete static_cast<WorldgenHandle*>(handle);
}

int wg_fill_density(void* handle, int minChunkX, int minChunkZ, int size, double* out) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !out || size <= 0) return 0;
    NoisePos pos;
    double* p = out;
    for (int cz = 0; cz < size; cz++) {
        for (int cx = 0; cx < size; cx++) {
            int chunkX = minChunkX + cx;
            int chunkZ = minChunkZ + cz;
            for (int y = 0; y < SY; y++) {
                for (int z = 0; z < SZ; z++) {
                    for (int x = 0; x < SX; x++) {
                        pos.x = chunkX * 16 + x * XZ_INTERVAL;
                        pos.z = chunkZ * 16 + z * XZ_INTERVAL;
                        pos.y = MIN_Y + y * Y_INTERVAL;
                        *p++ = h->finalDensity->sample(pos);
                    }
                }
            }
        }
    }
    return POINTS_PER_CHUNK;
}

int wg_density_xz_interval(void*) { return XZ_INTERVAL; }
int wg_density_y_interval(void*) { return Y_INTERVAL; }
int wg_min_y(void*) { return MIN_Y; }
int wg_height(void*) { return HEIGHT; }
int wg_density_points_per_chunk(void*) { return POINTS_PER_CHUNK; }

} // extern "C"
