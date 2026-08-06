// worldgen_api.cpp — CoreSwap worldgen C API 实现
#include "worldgen_api.h"

#include <cstdio>
#include <cstring>
#include <cstdlib>
#include <fstream>
#include <memory>
#include <mutex>
#include <sstream>
#include <string>
#include <vector>
#include <map>
#include <thread>
#include <atomic>

#ifdef _WIN32
#include <windows.h>
// 阶段计时（WG_PROFILE=1 时启用；QPC 高精度）
static double nowMs() {
    LARGE_INTEGER f, c;
    QueryPerformanceCounter(&c);
    QueryPerformanceFrequency(&f);
    return 1000.0 * (double)c.QuadPart / (double)f.QuadPart;
}
#else
#include <chrono>
static double nowMs() {
    using namespace std::chrono;
    return duration<double, milli>(steady_clock::now().time_since_epoch()).count();
}
#endif

#include "json.h"
#include "density.h"
#include "density_builder.h"
#include "blocks.h"
#include "biome.h"
#include "surface.h"
#include "aquifer.h"
#include "ore_vein.h"

// ---- 剖析计数（WG_PROFILE=1 启用；变量为 inline 定义于 density.h）----
static void profileInit() { wg_profEnabled = getenv("WG_PROFILE") != nullptr; }
void wg_profile_dump() {
    if (!wg_profEnabled) return;
    std::fprintf(stderr,
                 "[PROF] base_3d_noise.sample=%lld  spline.sample=%lld  interpGrid.fill=%lld  aquiferDeep=%lld  biomeAt=%lld\n",
                 (long long)wg_profNoiseDF.load(), (long long)wg_profSpline.load(),
                 (long long)wg_profInterpGrid.load(), (long long)wg_profAquiferDeep.load(),
                 (long long)wg_profBiomeAt.load());
}

using namespace wg;

namespace {

// 噪声参数表（BuiltinNoiseParameters 1.20.1）——与 density_probe 一致
std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams(const std::string& wgDir) {
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
    // 数据驱动补充：noise_params.json（Java 导出的全量噪声参数；JSON 覆盖同 key，mod 维度噪声也由此进入）
    try {
        std::string p = wgDir + "/../noise_params.json";
        std::ifstream pf(p, std::ios::binary);
        if (pf.good()) {
            std::stringstream ss;
            ss << pf.rdbuf();
            JsonParser sp(ss.str());
            JsonValue root = sp.parse();
            for (auto& kv : root.obj) {
                const JsonValue* octV = kv.second.get("firstOctave");
                const JsonValue* ampsV = kv.second.get("amplitudes");
                if (!octV || !ampsV) continue;
                std::vector<double> amps;
                for (auto& v : ampsV->arr) amps.push_back(v.numVal);
                m[kv.first] = DoublePerlinNoiseSampler::NoiseParameters{(int)octV->numVal, amps};
            }
        }
    } catch (...) { /* noise_params.json 缺失/损坏不影响主世界硬编码表 */ }
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
    DimConfig dim;  // 维度配置（通用引擎：minY/worldHeight/noiseHeight/aquifer/biome 参数）
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

void* wg_create(int64_t seed, const char* worldgenDir, const char* settingsName, const char* biomeParamsFile, int worldHeight) {
    profileInit();
    if (!worldgenDir) return nullptr;
    try {
        auto h = std::make_unique<WorldgenHandle>();
        h->wgDir = worldgenDir;
        std::string wgDir = worldgenDir;

        // 纯数据驱动（通用引擎）：维度参数全部从 noise_settings/<settingsName>.json 读。
        // settingsName 决定 density namespace/目录（"overworld.json" -> overworld；mod 维度传自己的设置文件名）。
        // worldHeight 由 Java 侧传（维度定义里的世界高度；overworld 384 / nether 256 / mod 维度按定义）。
        std::string settingsFile = settingsName ? settingsName : "overworld.json";
        std::string dfNs = settingsFile.size() > 5 ? settingsFile.substr(0, settingsFile.size() - 5) : settingsFile;  // 去 ".json"
        std::string settingsPath = wgDir + "/data/minecraft/worldgen/noise_settings/" + settingsFile;
        JsonParser sp(readFile(settingsPath));
        JsonValue settings = sp.parse();
        const JsonValue* noise = settings.get("noise");
        if (noise) {
            const JsonValue* minY = noise->get("min_y");
            const JsonValue* hgt = noise->get("height");
            if (minY) h->dim.minY = (int)minY->numVal;
            if (hgt) h->dim.noiseHeight = (int)hgt->numVal;
        }
        h->dim.worldHeight = worldHeight > 0 ? worldHeight : h->dim.noiseHeight;  // 世界高度：Java 传（维度定义）；兜底 = 噪声高度
        const JsonValue* aq = settings.get("aquifers_enabled");
        h->dim.aquifersEnabled = aq ? aq->boolVal : true;
        if (biomeParamsFile) h->dim.biomeParamsFile = biomeParamsFile;

        auto noiseParams = buildNoiseParams(h->wgDir);
        h->builder = std::make_unique<DensityBuilder>((uint64_t)seed, noiseParams, h->dim.minY, h->dim.noiseHeight);
        std::string dfDir = wgDir + "/data/minecraft/worldgen/density_function/" + dfNs + "/";
        // 捕获 handle（长期存活）而非局部变量，避免悬垂引用
        h->builder->externalLoader = [hPtr = h.get(), dfNs](const std::string& fullRef, const std::string& name) -> DF {
            std::string path = hPtr->wgDir + "/data/minecraft/worldgen/density_function/" + dfNs + "/" + name + ".json";
            std::ifstream probe(path);
            if (!probe.good()) return nullptr;
            return hPtr->builder->parseFile(fullRef, readFile(path));
        };

        // 已知维度预注册（overworld 15 个官方密度文件；nether 只需 base_3d_noise）；mod 维度纯惰性（externalLoader 兜底读文件）
        std::vector<std::string> dfFiles;
        if (dfNs == "overworld") dfFiles = {
            "base_3d_noise", "continents", "depth", "erosion", "factor",
            "jaggedness", "offset", "ridges", "ridges_folded", "sloped_cheese",
            "caves/entrances", "caves/noodle", "caves/pillars",
            "caves/spaghetti_2d_thickness_modulator", "caves/spaghetti_2d",
            "caves/spaghetti_roughness_function",
        };
        else if (dfNs == "nether") dfFiles = {"base_3d_noise"};
        for (const auto& f : dfFiles) {
            h->builder->registerFunction("minecraft:" + dfNs + "/" + f, std::make_shared<DensityBuilder::LazyRef>());
        }
        for (const auto& f : dfFiles) {
            std::string path = dfDir + f + ".json";
            if (std::ifstream(path).good()) {
                auto df = h->builder->parseFile("minecraft:" + dfNs + "/" + f, readFile(path));
                h->builder->registerFunction("minecraft:" + dfNs + "/" + f, df);
            }
        }

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
            {"vein_toggle", "vein_toggle"}, {"vein_ridged", "vein_ridged"}, {"vein_gap", "vein_gap"},
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

        // biome 源（biome_params.json / biome_params_nether.json：Java BiomeParamProbe 导出的 vanilla 参数表）
        std::string biomeParamsPath = wgDir + "/../" + h->dim.biomeParamsFile;
        if (!h->biomeSource.loadFromJson(readFile(biomeParamsPath))) {
            std::fprintf(stderr, "wg_create: cannot load %s\n", biomeParamsPath.c_str());
            return nullptr;
        }
    
        // surface builder（主世界 seaLevel=63）
        std::string biomeDirForBuilder = wgDir + "/data/minecraft/worldgen/biome/";
        h->surfaceBuilder = std::make_unique<SurfaceBuilder>(
            &h->noiseSamplers, &h->builder->randomDeriverPublic(), 63, &h->blocks, biomeDirForBuilder);
        // 规则树预构建（多线程安全：fillOneChunk 只读 overworldRule）
        h->overworldRule = h->surfaceBuilder->buildOverworldRule();
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
// 单个 chunk 的生成逻辑（线程安全：InterpolatedDF 缓存 thread_local、
// SurfaceContext/aquifer/oreVein 均为 per-chunk 局部对象、split() 纯函数）
static int fillOneChunk(void* handle, int chunkX, int chunkZ, int32_t* out) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !out) return 0;

    constexpr int XZ = XZ_INTERVAL, Y = Y_INTERVAL;
    const int air = h->blocks.id("minecraft:air");
    const int stone = h->blocks.id("minecraft:stone");

    // 1. density：块级直接采样 finalDensity（InterpolatedDF 内部按 cell 网格插值，
    //    与 Java CellCache(add(DensityInterpolator(finalDensity), Beardifier)) 语义一致——
    //    只对 interpolated 节点插值，min/squeeze/mul 等非线性在插值后应用）
    NoisePos fpos;

    // 2. aquifer/oreVein（per chunk）——维度化：主世界有 aquifer+oreVein；下界 aquifersEnabled=false 且无 vein 组件（跳过）
    auto& R = h->router;
    const bool hasAquifer = h->dim.aquifersEnabled && R.count("fluid_level_floodedness") && R.count("vein_toggle");
    if (!hasAquifer) {
        // 下界：只校验存在的基础组件（barrier/continents/depth/erosion）
        for (const char* k : {"barrier", "continents", "depth", "erosion"}) {
            if (!R.count(k)) { std::fprintf(stderr, "wg_fill_blocks: missing router component %s\n", k); return 0; }
        }
    } else {
        for (const char* k : {"barrier", "fluid_level_floodedness", "fluid_level_spread",
                              "lava", "erosion", "depth", "initial_density",
                              "temperature", "vegetation", "continents", "ridges",
                              "vein_toggle", "vein_ridged", "vein_gap"}) {
            if (!R.count(k)) { std::fprintf(stderr, "wg_fill_blocks: missing router component %s\n", k); return 0; }
        }
    }
    std::unique_ptr<Aquifer> aquifer;
    std::unique_ptr<OreVeinSampler> oreVein;
    if (hasAquifer) {
        XoroshiroRandom aquiferRnd = h->builder->randomDeriverPublic().split("minecraft:aquifer");
        aquifer = std::make_unique<Aquifer>(R["barrier"], R["fluid_level_floodedness"], R["fluid_level_spread"],
                        R["lava"], R["erosion"], R["depth"], R["initial_density"],
                        aquiferRnd.nextSplitter(), &h->blocks, chunkX * 16, chunkZ * 16,
                        h->dim.minY, h->dim.worldHeight);
        // ore veins（NoiseConfig: split("ore").nextSplitter()）
        XoroshiroRandom oreRnd = h->builder->randomDeriverPublic().split("minecraft:ore");
        oreVein = std::make_unique<OreVeinSampler>(R["vein_toggle"], R["vein_ridged"], R["vein_gap"],
                                   oreRnd.nextSplitter(), &h->blocks);
    }

    // 3. fillFromNoise：块级三线性插值 → aquifer → 方块 + heightmap
    BlockColumn col(h->dim.minY, h->dim.worldHeight);
    std::vector<int> heightmap(256, h->dim.minY - 1);
    bool profiling = getenv("WG_PROFILE") != nullptr;
    double tA = 0, tB = 0, tC = 0, tD = 0, tE = 0;
    double t0 = profiling ? nowMs() : 0;
    std::vector<double> densityBuf((size_t)h->dim.worldHeight * 256);
    // 3a. density（独立循环，便于剖析与后续算法优化）
    for (int by = 0; by < h->dim.worldHeight; by++) {
        int wy = h->dim.minY + by;
        for (int bz = 0; bz < 16; bz++) {
            for (int bx = 0; bx < 16; bx++) {
                fpos.x = chunkX * 16 + bx;
                fpos.y = wy;
                fpos.z = chunkZ * 16 + bz;
                densityBuf[by * 256 + bz * 16 + bx] = h->finalDensity->sample(fpos);
            }
        }
    }
    if (profiling) tA = nowMs();
    // WG_SURFDUMP 诊断：dump 指定列的表面高度估计与 initialDensity/finalDensity 剖面
    if (getenv("WG_SURFDUMP")) {
        const char* sx = getenv("WG_SURFDUMP_X");
        const char* sz = getenv("WG_SURFDUMP_Z");
        if (sx && sz) {
            int bx = atoi(sx), bz = atoi(sz);
            if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
                NoisePos p;
                for (int y = -64; y <= 63; y += 4) {
                    p.x = bx; p.y = y; p.z = bz;
                    std::fprintf(stderr, "[SURF] (%d,%d,%d) initialDensity=%.6f finalDensity=%.6f\n", bx, y, bz,
                                 R["initial_density"]->sample(p), h->finalDensity->sample(p));
                }
                std::fprintf(stderr, "[SURF] estimateSurfaceHeight(%d,%d)=%d\n", bx, bz,
                             aquifer ? aquifer->estimateSurfaceHeight(bx, bz) : 0);
                // 分量 dump（y=31 深水处）
                p.y = 31;
                const char* comps[] = {"base_3d_noise", "factor", "depth", "jaggedness", "continents", "erosion"};
                for (const char* c : comps) {
                    DF df = h->builder->getFunction("minecraft:overworld/" + std::string(c));
                    if (df) std::fprintf(stderr, "[SURF] %s(y=31)=%.6f\n", c, df->sample(p));
                    else std::fprintf(stderr, "[SURF] %s(y=31)=<missing>\n", c);
                }
            }
        } else if (getenv("WG_SURFDUMP_SCAN")) {
            // 全列扫描 y=31：找出 finalDensity 偏正（>0.01）的列（vanilla 深水列应 ≤0）
            NoisePos p;
            p.y = 31;
            for (int bz = 0; bz < 16; bz++) {
                for (int bx = 0; bx < 16; bx++) {
                    p.x = chunkX * 16 + bx;
                    p.z = chunkZ * 16 + bz;
                    double fd = h->finalDensity->sample(p);
                    if (fd > 0.01) {
                        std::fprintf(stderr, "[SURF+] (%d,%d) fd=%.3f\n", p.x, p.z, fd);
                    }
                }
            }
        }
    }
    // 3b. aquifer + oreVein（ChainedBlockSource：aquifer null → oreVein；下界两者都 null）
    for (int by = 0; by < h->dim.worldHeight; by++) {
        int wy = h->dim.minY + by;
        for (int bz = 0; bz < 16; bz++) {
            for (int bx = 0; bx < 16; bx++) {
                int block = -1;
                if (aquifer) {
                    block = aquifer->apply(chunkX * 16 + bx, wy, chunkZ * 16 + bz, densityBuf[by * 256 + bz * 16 + bx]);
                    if (block < 0 && oreVein) block = oreVein->apply(chunkX * 16 + bx, wy, chunkZ * 16 + bz);
                }
                if (block < 0) block = stone;
                col.at(bx, wy, bz) = block;
                if (block != air && wy > heightmap[bz * 16 + bx]) heightmap[bz * 16 + bx] = wy;
            }
        }
    }
    if (profiling) tB = nowMs();

    // 4. buildSurface
    auto biomeAt = [&](int x, int y, int z) -> std::string {
        if (wg_profEnabled) wg_profBiomeAt.fetch_add(1, std::memory_order_relaxed);
        NoisePos p;
        // Java MultiNoiseBiomeSource.getBiome：sampler.sample(x >> 2, y >> 2, z >> 2)，
        // 内部 ×4 回 block → 采样位置 = floor(block/4)*4
        p.x = (x >> 2) << 2;
        p.y = (y >> 2) << 2;
        p.z = (z >> 2) << 2;
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
    sh4[0] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16, chunkZ * 16) : 0;
    sh4[1] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16) : 0;
    sh4[2] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16, chunkZ * 16 + 16) : 0;
    sh4[3] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16 + 16) : 0;
    h->surfaceBuilder->buildSurface(col, h->overworldRule, chunkX * 16, chunkZ * 16, heightmap, sh4, biomeAt, biomeTemp,
                                    h->dim.minY, h->dim.worldHeight);
    if (profiling) {
        double tEnd = nowMs();
        std::fprintf(stderr, "[PROF] chunk(%d,%d): density=%.2fms aquifer+oreVein=%.2fms sh4+surface=%.2fms total=%.2fms\n",
                     chunkX, chunkZ, tA - t0, tB - tA, tEnd - tB, tEnd - t0);
    }

    // 5. 输出
    std::memcpy(out, col.data().data(), BLOCK_COUNT * sizeof(int32_t));
    return BLOCK_COUNT;
}

// 单 chunk（串行兼容入口）
int wg_fill_blocks(void* handle, int chunkX, int chunkZ, int32_t* out) {
    return fillOneChunk(handle, chunkX, chunkZ, out);
}

// 多 chunk 并行：chunkXs/chunkZs/outs 为 count 个 chunk 的坐标与输出缓冲。
// 每个 chunk 独立生成（确定性随机派生 + thread_local 缓存），结果与串行逐位一致。
int wg_fill_blocks_multi(void* handle, const int* chunkXs, const int* chunkZs,
                         int32_t* const* outs, int count, int threads) {
    if (count <= 0) return 0;
    if (threads <= 0) {
        // 默认自适应：min(CPU 逻辑线程数, 任务数)；探测失败兜底 1，避免过度订阅
        threads = (int)std::thread::hardware_concurrency();
        if (threads <= 0) threads = 1;
    }
    if (threads > count) threads = count;
    std::vector<std::thread> pool;
    std::atomic<int> next{0};
    pool.reserve(threads);
    for (int t = 0; t < threads; t++) {
        pool.emplace_back([&]() {
            for (;;) {
                int i = next.fetch_add(1);
                if (i >= count) break;
                fillOneChunk(handle, chunkXs[i], chunkZs[i], outs[i]);
            }
        });
    }
    for (auto& th : pool) th.join();
    return count;
}

} // extern "C"





