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
#include "blocks.h"
#include "biome.h"
#include "surface.h"
#include "aquifer.h"

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
    // router 分量（biome 采样 + aquifer）
    std::map<std::string, DF> router;
    // 噪声 sampler 表（surface rules + surface builder 用）
    std::map<std::string, DoublePerlinNoiseSampler> noiseSamplers;
    // 方块注册表
    BlockRegistry blocks;
    // biome 源
    BiomeSource biomeSource;
    // surface builder（全局状态）
    std::unique_ptr<SurfaceBuilder> surfaceBuilder;
    // 主世界规则树（构建一次缓存）
    RuleP overworldRule;
    // worldgen 数据目录（externalLoader 用，避免悬垂引用）
    std::string wgDir;
};

// 读取噪声参数到 sampler 表
static void fillNoiseSamplers(WorldgenHandle& h,
                              const std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters>& noiseParams,
                              const XoroshiroRandom::Splitter& splitter,
                              const std::vector<std::string>& keys) {
    for (const auto& k : keys) {
        auto it = noiseParams.find("minecraft:" + k);
        if (it == noiseParams.end()) continue;
        XoroshiroRandom rnd = splitter.split("minecraft:" + k);
        h.noiseSamplers["minecraft:" + k] = DoublePerlinNoiseSampler(rnd, it->second);
    }
}

} // namespace

extern "C" {

void* wg_create(int64_t seed, const char* worldgenDir) {
    if (!worldgenDir) return nullptr;
    try {
        auto h = std::make_unique<WorldgenHandle>();
        auto noiseParams = buildNoiseParams();
        h->builder = std::make_unique<DensityBuilder>((uint64_t)seed, noiseParams);
        h->wgDir = worldgenDir;

        std::string wgDir = worldgenDir;
        std::string dfDir = wgDir + "/data/minecraft/worldgen/density_function/overworld/";
        // 捕获 handle（长期存活）而非局部变量，避免悬垂引用
        h->builder->externalLoader = [hPtr = h.get()](const std::string& fullRef, const std::string& name) -> DF {
            std::string path = hPtr->wgDir + "/data/minecraft/worldgen/density_function/overworld/" + name + ".json";
            std::ifstream probe(path);
            if (!probe.good()) return nullptr;
            return hPtr->builder->parseFile(fullRef, readFile(path));
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

        // ---- 方块层装配 ----
        // router 分量
        struct Comp { const char* name; const char* jsonKey; };
        for (auto& c : std::vector<Comp>{
            {"barrier", "barrier"}, {"fluid_level_floodedness", "fluid_level_floodedness"},
            {"fluid_level_spread", "fluid_level_spread"}, {"lava", "lava"},
            {"temperature", "temperature"}, {"vegetation", "vegetation"},
            {"continents", "continents"}, {"erosion", "erosion"},
            {"depth", "depth"}, {"ridges", "ridges"},
            {"initial_density", "initial_density_without_jaggedness"},
        }) {
            const JsonValue* v = router->get(c.jsonKey);
            if (v) h->router[c.name] = h->builder->buildNode(*v);
        }

        // 方块注册表（blocks.json：vanilla raw id，Java 侧同表）
        std::string blocksPath = wgDir + "/../blocks.json";
        if (!h->blocks.loadFromJson(readFile(blocksPath))) {
            std::fprintf(stderr, "wg_create: cannot load %s\n", blocksPath.c_str());
            return nullptr;
        }

        // noise sampler 表（surface rules / surface builder）
        fillNoiseSamplers(*h, noiseParams, h->builder->randomDeriverPublic(), {
            "surface", "surface_secondary", "calcite", "gravel", "powder_snow",
            "packed_ice", "ice", "surface_swamp", "clay_bands_offset",
            "badlands_pillar", "badlands_pillar_roof", "badlands_surface",
            "iceberg_pillar", "iceberg_pillar_roof", "iceberg_surface",
        });

        // biome 源（biome_params.json：Java BiomeParamProbe 导出的 vanilla 参数表）
        std::string biomeParamsPath = wgDir + "/../biome_params.json";
        if (!h->biomeSource.loadFromJson(readFile(biomeParamsPath))) {
            std::fprintf(stderr, "wg_create: cannot load %s\n", biomeParamsPath.c_str());
            return nullptr;
        }

        // surface builder（主世界 seaLevel=63）
        std::string biomeDirForBuilder = wgDir + "/data/minecraft/worldgen/biome/";
        h->surfaceBuilder = std::make_unique<SurfaceBuilder>(
            &h->noiseSamplers, &h->builder->randomDeriverPublic(), 63, &h->blocks, biomeDirForBuilder);
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

// ---- 方块层：完整区块生成（density → aquifer → surface rules）----
// out: int32_t[16*16*384]（BlockId，vanilla raw id）
int wg_fill_blocks(void* handle, int chunkX, int chunkZ, int32_t* out) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !out) return 0;

    constexpr int XZ = XZ_INTERVAL, Y = Y_INTERVAL;
    constexpr int GX = 16 / XZ + 1, GY = HEIGHT / Y + 1, GZ = 16 / XZ + 1; // 5×49×5 halo 网格
    const int air = h->blocks.id("minecraft:air");
    const int stone = h->blocks.id("minecraft:stone");

    // 1. density 网格（含 halo 角点）
    std::vector<double> grid((size_t)GX * GY * GZ);
    NoisePos pos;
    for (int gy = 0; gy < GY; gy++)
        for (int gz = 0; gz < GZ; gz++)
            for (int gx = 0; gx < GX; gx++) {
                pos.x = chunkX * 16 + gx * XZ;
                pos.y = MIN_Y + gy * Y;
                pos.z = chunkZ * 16 + gz * XZ;
                grid[((size_t)gy * GZ + gz) * GX + gx] = h->finalDensity->sample(pos);
            }

    // 2. aquifer（per chunk）
    auto& R = h->router;
    for (const char* k : {"barrier", "fluid_level_floodedness", "fluid_level_spread",
                          "lava", "erosion", "depth", "initial_density",
                          "temperature", "vegetation", "continents", "ridges"}) {
        if (!R.count(k)) { std::fprintf(stderr, "wg_fill_blocks: missing router component %s\n", k); return 0; }
    }
    Aquifer aquifer(R["barrier"], R["fluid_level_floodedness"], R["fluid_level_spread"],
                    R["lava"], R["erosion"], R["depth"], R["initial_density"],
                    h->builder->randomDeriverPublic(), &h->blocks, chunkX * 16, chunkZ * 16, MIN_Y, HEIGHT);

    // 3. fillFromNoise：块级三线性插值 → aquifer → 方块 + heightmap
    BlockColumn col;
    std::vector<int> heightmap(256, MIN_Y - 1);
    for (int by = 0; by < HEIGHT; by++) {
        int wy = MIN_Y + by;
        int cgy = by / Y;
        double fy = (by % Y) / (double)Y;
        for (int bz = 0; bz < 16; bz++) {
            int cgz = bz / XZ;
            double fz = (bz % XZ) / (double)XZ;
            for (int bx = 0; bx < 16; bx++) {
                int cgx = bx / XZ;
                double fx = (bx % XZ) / (double)XZ;
                auto g = [&](int dx, int dy, int dz) {
                    return grid[((size_t)(cgy + dy) * GZ + (cgz + dz)) * GX + (cgx + dx)];
                };
                double d000 = g(0, 0, 0), d100 = g(1, 0, 0), d010 = g(0, 1, 0), d110 = g(1, 1, 0);
                double d001 = g(0, 0, 1), d101 = g(1, 0, 1), d011 = g(0, 1, 1), d111 = g(1, 1, 1);
                double d00 = d000 + (d100 - d000) * fx;
                double d10 = d010 + (d110 - d010) * fx;
                double d01 = d001 + (d101 - d001) * fx;
                double d11 = d011 + (d111 - d011) * fx;
                double d0 = d00 + (d10 - d00) * fy;
                double d1 = d01 + (d11 - d01) * fy;
                double density = d0 + (d1 - d0) * fz;
                int block = aquifer.apply(chunkX * 16 + bx, wy, chunkZ * 16 + bz, density);
                if (block < 0) block = stone;
                col.at(bx, wy, bz) = block;
                if (block != air && wy > heightmap[bz * 16 + bx]) heightmap[bz * 16 + bx] = wy;
            }
        }
    }

    // 4. buildSurface
    auto biomeAt = [&](int x, int y, int z) -> std::string {
        NoisePos p;
        p.x = x; p.y = y; p.z = z;
        float t = (float)R["temperature"]->sample(p);
        float hum = (float)R["vegetation"]->sample(p);
        float cont = (float)R["continents"]->sample(p);
        float ero = (float)R["erosion"]->sample(p);
        float dep = (float)R["depth"]->sample(p);
        float w = (float)R["ridges"]->sample(p);
        const std::string* id = h->biomeSource.find(t, hum, cont, ero, dep, w);
        return id ? *id : "minecraft:plains";
    };
    auto biomeTemp = [&](const std::string& id) -> double {
        return h->biomeSource.temperature(id);
    };
    std::vector<int> sh4(4);
    sh4[0] = aquifer.estimateSurfaceHeight(chunkX * 16, chunkZ * 16);
    sh4[1] = aquifer.estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16);
    sh4[2] = aquifer.estimateSurfaceHeight(chunkX * 16, chunkZ * 16 + 16);
    sh4[3] = aquifer.estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16 + 16);
    if (!h->overworldRule) h->overworldRule = h->surfaceBuilder->buildOverworldRule();
    h->surfaceBuilder->buildSurface(col, h->overworldRule, chunkX * 16, chunkZ * 16, heightmap, sh4, biomeAt, biomeTemp);

    // 5. 输出
    std::memcpy(out, col.data().data(), BLOCK_COUNT * sizeof(int32_t));
    return BLOCK_COUNT;
}

} // extern "C"
