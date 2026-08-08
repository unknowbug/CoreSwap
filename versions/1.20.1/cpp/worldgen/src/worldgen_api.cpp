// worldgen_api.cpp — CoreSwap worldgen C API 实现
#include "worldgen_api.h"
#include "crash_handler.h"

// 崩溃上下文（thread_local）：当前 JNI 入口名，崩溃 handler 打印
namespace wg { thread_local const char* g_crashContext = nullptr; }

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
static void profileInit() { wg_profEnabled = getenv("WG_PROFILE") != nullptr; wg_splineDebug = getenv("WG_SPLINEDEBUG") != nullptr; wg_surfaceTrace = getenv("WG_SURFTRACE") != nullptr; wg_aqfDump = getenv("WG_AQFDUMP") != nullptr; if (getenv("WG_SURFTRACE_X")) wg_surfaceTraceX = atoi(getenv("WG_SURFTRACE_X")); if (getenv("WG_SURFTRACE_Z")) wg_surfaceTraceZ = atoi(getenv("WG_SURFTRACE_Z")); if (getenv("WG_AQF_YMIN")) wg_aqfYMin = atoi(getenv("WG_AQF_YMIN")); if (getenv("WG_AQF_YMAX")) wg_aqfYMax = atoi(getenv("WG_AQF_YMAX")); }
void wg_profile_dump() {
    if (!wg_profEnabled) return;
    std::fprintf(stderr,
                 "[PROF] base_3d_noise.sample=%lld  spline.sample=%lld  interpGrid.fill=%lld  aquiferDeep=%lld  biomeAt=%lld\n",
                 (long long)wg_profNoiseDF.load(), (long long)wg_profSpline.load(),
                 (long long)wg_profInterpGrid.load(), (long long)wg_profAquiferDeep.load(),
                 (long long)wg_profBiomeAt.load());
    std::fprintf(stderr,
                 "[PROF] noise=%.1fms(%lld次)  spline=%.1fms(%lld次)  单次: noise=%.0fns spline=%.0fns\n",
                 wg_profNoiseNs.load() / 1e6, (long long)wg_profNoiseDF.load(),
                 wg_profSplineNs.load() / 1e6, (long long)wg_profSpline.load(),
                 wg_profNoiseDF.load() ? (double)wg_profNoiseNs.load() / wg_profNoiseDF.load() : 0.0,
                 wg_profSpline.load() ? (double)wg_profSplineNs.load() / wg_profSpline.load() : 0.0);
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
    // 世界种子 + BiomeAccess access seed（BiomeAccess.hashSeed(seed)，8 邻域选点用）
    int64_t seed = 0;
    int64_t biomeAccessSeed = 0;
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

// ===== surface_rule JSON 解析（通用引擎：任意维度 surface_rule 数据驱动）=====
static CondP parseSurfaceCond(const JsonValue& j, int minY, int worldHeight, const BlockRegistry* blocks, bool& ok);
static int anchorAbsY(const JsonValue& a, int minY, int worldHeight) {
    if (const JsonValue* v = a.get("absolute")) return (int)v->numVal;
    if (const JsonValue* v = a.get("above_bottom")) return minY + (int)v->numVal;
    if (const JsonValue* v = a.get("below_top")) return minY + worldHeight - (int)v->numVal;
    return 0;
}
static RuleP parseSurfaceRule(const JsonValue& j, int minY, int worldHeight, const BlockRegistry* blocks, bool& ok) {
    std::string type = j.isString() ? j.strVal : (j.get("type") ? j.get("type")->strVal : "");
    if (type == "minecraft:sequence") {
        std::vector<RuleP> rules;
        if (const JsonValue* seq = j.get("sequence"))
            for (const auto& r : seq->arr) rules.push_back(parseSurfaceRule(r, minY, worldHeight, blocks, ok));
        return sequence(std::move(rules));
    }
    if (type == "minecraft:condition") {
        if (const JsonValue* c = j.get("if_true")) {
            CondP cond = parseSurfaceCond(*c, minY, worldHeight, blocks, ok);
            RuleP then = j.get("then_run") ? parseSurfaceRule(*j.get("then_run"), minY, worldHeight, blocks, ok) : nullptr;
            return condition(cond, then);
        }
    }
    if (type == "minecraft:block") {
        if (const JsonValue* rs = j.get("result_state")) {
            if (const JsonValue* n = rs->get("Name")) return blockRule(blocks->id(n->strVal));
        }
    }
    ok = false;  // 未支持节点
    return nullptr;
}
static CondP parseSurfaceCond(const JsonValue& j, int minY, int worldHeight, const BlockRegistry* blocks, bool& ok) {
    std::string type = j.isString() ? j.strVal : (j.get("type") ? j.get("type")->strVal : "");
    if (type == "minecraft:not") {
        if (const JsonValue* inv = j.get("invert")) return notCond(parseSurfaceCond(*inv, minY, worldHeight, blocks, ok));
    }
    if (type == "minecraft:biome") {
        std::set<std::string> s;
        if (const JsonValue* b = j.get("biome_is")) for (const auto& x : b->arr) s.insert(x.strVal);
        return biomeCond(std::move(s));
    }
    if (type == "minecraft:y_above") {
        const JsonValue* a = j.get("anchor");
        if (a) {
            int anchor = anchorAbsY(*a, minY, worldHeight);
            bool addStoneDepth = j.get("add_stone_depth") ? j.get("add_stone_depth")->boolVal : false;
            return aboveY(anchor, 0, addStoneDepth);  // surface_depth_multiplier=0 → mult=0
        }
    }
    if (type == "minecraft:stone_depth") {
        int offset = j.get("offset") ? (int)j.get("offset")->numVal : 0;
        bool addSurface = j.get("add_surface_depth") ? j.get("add_surface_depth")->boolVal : false;
        int range = j.get("secondary_depth_range") ? (int)j.get("secondary_depth_range")->numVal : 0;
        bool ceiling = j.get("surface_type") && j.get("surface_type")->strVal == "ceiling";
        return stoneDepth(offset, addSurface, range, ceiling);
    }
    if (type == "minecraft:noise_threshold") {
        double min = j.get("min_threshold") ? j.get("min_threshold")->numVal : -1.7e308;
        double max = j.get("max_threshold") ? j.get("max_threshold")->numVal : 1.7e308;
        if (const JsonValue* n = j.get("noise")) return noiseThreshold(n->strVal, min, max);
    }
    if (type == "minecraft:vertical_gradient") {
        std::string name = j.get("random_name") ? j.get("random_name")->strVal : "";
        int trueY = j.get("true_at_and_below") ? anchorAbsY(*j.get("true_at_and_below"), minY, worldHeight) : 0;
        int falseY = j.get("false_at_and_above") ? anchorAbsY(*j.get("false_at_and_above"), minY, worldHeight) : 0;
        return verticalGradient(name, trueY, falseY);
    }
    if (type == "minecraft:hole") return std::make_shared<HoleCond>();
    if (type == "minecraft:steep") return std::make_shared<SteepCond>();
    if (type == "minecraft:water") {
        auto c = std::make_shared<WaterCond>();
        c->offset = j.get("offset") ? (int)j.get("offset")->numVal : 0;
        c->mult = 0;
        c->addStoneDepth = j.get("add_stone_depth") ? j.get("add_stone_depth")->boolVal : false;
        return c;
    }
    if (type == "minecraft:temperature") return std::make_shared<TempCond>();
    if (type == "minecraft:surface") return std::make_shared<SurfaceCondC>();
    ok = false;
    return nullptr;
}

void* wg_create(int64_t seed, const char* worldgenDir, const char* settingsName, const char* biomeParamsFile, int worldHeight) {
    // 崩溃日志：独立进程（block_probe/got_export）装 VEH——JVM 进程（jvm.dll 已加载）不装！
    // 实测（2026-08-08）：AddVectoredExceptionHandler 干扰 JVM 硬件异常处理（JIT null-check / GC guard page 均为
    // SEH 异常，VEH 先执行 StackWalk64/打印重活 → Server thread 堆损坏崩溃，用户 0x34001 同根因）。
    // JVM 侧崩溃由 JVM 自带 hs_err（含 native 栈 dll 偏移）兜底。
    if (!GetModuleHandleA("jvm.dll")) wg::installCrashHandler();
    profileInit();
    if (!worldgenDir) return nullptr;
    try {
        auto h = std::make_unique<WorldgenHandle>();
        h->wgDir = worldgenDir;
        h->seed = seed;
        h->biomeAccessSeed = wg::biomeHashSeed(seed);
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
            std::ifstream ifs(path);
            if (getenv("WG_PRE")) std::fprintf(stderr, "[PRE] %s good=%d\n", path.c_str(), (int)ifs.good());
            if (ifs.good()) {
                auto df = h->builder->parseFile("minecraft:" + dfNs + "/" + f, readFile(path));
                h->builder->registerFunction("minecraft:" + dfNs + "/" + f, df);
            }
        }

        const JsonValue* router = settings.get("noise_router");
        const JsonValue* finalDensity = router->get("final_density");
        h->finalDensity = h->builder->buildNode(*finalDensity);
        if (getenv("WG_PROFILE"))
            std::fprintf(stderr, "[BUILD] InterpolatedDF instances=%d (Java cns has 8)\n",
                         wg::InterpolatedDF::getInstanceCount());
    
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
    
        // surface builder（seaLevel 从 settings 读；主世界 63 / 下界 32）
        int seaLevel = 63;
        if (const JsonValue* sl = settings.get("sea_level")) seaLevel = (int)sl->numVal;
        std::string biomeDirForBuilder = wgDir + "/data/minecraft/worldgen/biome/";
        h->surfaceBuilder = std::make_unique<SurfaceBuilder>(
            &h->noiseSamplers, &h->builder->randomDeriverPublic(), seaLevel, &h->blocks, biomeDirForBuilder);
        // 规则树：主世界用代码规则（已逐位验证）；其他维度（下界/mod）用 surface_rule JSON 数据驱动
        const JsonValue* sr = settings.get("surface_rule");
        if (dfNs == "overworld" || !sr) {
            h->overworldRule = h->surfaceBuilder->buildOverworldRule();
        } else {
            bool ok = true;
            h->overworldRule = parseSurfaceRule(*sr, h->dim.minY, h->dim.worldHeight, &h->blocks, ok);
            if (!ok || !h->overworldRule) {
                std::fprintf(stderr, "wg_create: surface_rule JSON 解析失败（未支持节点），回退主世界代码规则\n");
                h->overworldRule = h->surfaceBuilder->buildOverworldRule();
            }
        }
        return h.release();
    } catch (const std::exception& e) {
        std::fprintf(stderr, "wg_create: %s\n", e.what());
        return nullptr;
    }
}

// 线程池停止（前向声明，定义在 CoreSwapPool 之后）
void shutdownCoreSwapPool();

void wg_destroy(void* handle) {
    // 停止线程池（等待所有 worker 完成，避免 use-after-free：JVM shutdown 时 destroy 后无 worker 再用 handle）
    shutdownCoreSwapPool();
    delete static_cast<WorldgenHandle*>(handle);
}

// 直接采样 finalDensity（密度级对比/诊断用；维度由 handle 决定）
double wg_sample_density(void* handle, int x, int y, int z) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !h->finalDensity) return 0.0;
    NoisePos pos;
    pos.x = x; pos.y = y; pos.z = z;
    return h->finalDensity->sample(pos);
}

// 采样注册的 density function（分量对比：如 "minecraft:nether/base_3d_noise"）
double wg_sample_named(void* handle, const char* name, int x, int y, int z) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !name) return 0.0;
    DF df = h->builder->getFunction(name);
    if (!df) return 0.0;
    NoisePos pos;
    pos.x = x; pos.y = y; pos.z = z;
    return df->sample(pos);
}

// 采样噪声（如 "minecraft:jagged"）：直接 sample 底层 DoublePerlinNoiseSampler
double wg_sample_noise(void* handle, const char* name, double x, double y, double z) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !name) return 0.0;
    auto ns = h->builder->getNoiseSampler(name);
    if (!ns) return 0.0;
    return ns->sample(x, y, z);
}

// 采样 router 分量（temperature/continents 等 @block 坐标）
double wg_router_sample(void* handle, const char* name, int x, int y, int z) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !name) return 0.0;
    auto it = h->router.find(name);
    if (it == h->router.end()) return 0.0;
    NoisePos p; p.x = x; p.y = y; p.z = z;
    return it->second->sample(p);
}

// 采样 biome（复刻 fillOneChunk 的 biomeAt）：返回 biome id 字符串（写入 out）
void wg_sample_biome(void* handle, int x, int y, int z, char* out, int outLen) {
    auto* h = static_cast<WorldgenHandle*>(handle);
    if (!h || !out || outLen <= 0) return;
    std::string id = "minecraft:plains";
    {
        NoisePos p;
        // Java BiomeAccess.getBiome(BlockPos)：8 邻域 seed 哈希选点
        int px, py, pz;
        wg::biomePickCell(h->biomeAccessSeed, x, y, z, px, py, pz);
        p.x = px << 2; p.y = py << 2; p.z = pz << 2;
        auto samp = [&](const char* k, const NoisePos& q) -> float {
            auto it = h->router.find(k);
            return it != h->router.end() ? (float)it->second->sample(q) : 0.0f;
        };
        float t = samp("temperature", p), hum = samp("vegetation", p);
        float cont = samp("continents", p), ero = samp("erosion", p);
        float dep = samp("depth", p), w = samp("ridges", p);
        // WG_BIOMEDUMP 诊断：打印选点坐标 + 判定输入 6 维 + find 结果（对比 Java BiomeAccess/MultiNoiseSampler）
        if (getenv("WG_BIOMEDUMP")) {
            std::fprintf(stderr, "[BIOMEIN] (%d,%d,%d) pick=(%d,%d,%d) sample=(%d,%d,%d) "
                        "t=%.9f hum=%.9f cont=%.9f ero=%.9f dep=%.9f w=%.9f\n",
                        x, y, z, px, py, pz, p.x, p.y, p.z, t, hum, cont, ero, dep, w);
        }
        const std::string* bid = h->biomeSource.find(t, hum, cont, ero, dep, w);
        if (bid) id = *bid;
    }
    strncpy(out, id.c_str(), (size_t)outLen - 1);
    out[outLen - 1] = '\0';
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

    // 排查用户 1.0.16 崩溃：memset 函数指针存储位 0x34001 被堆覆盖（call 目标=堆地址）。
    // 每 chunk 校验其值 vs 基线（首个 chunk 记录）——被写坏立即打印（定位写坏时机/线程）。
    {
        static HMODULE selfM = GetModuleHandleA("worldgen.dll");
        uintptr_t baseM = (uintptr_t)selfM;
        if (baseM) {
            static uint64_t baseline = 0;
            static bool haveBase = false;
            uint64_t v0 = 0;
            void* p0 = (void*)(baseM + 0x34001);
            if (IsBadReadPtr(p0, 8) == FALSE) memcpy(&v0, p0, 8);
            if (!haveBase) { baseline = v0; haveBase = true; }
            if (v0 != baseline) {
                std::fprintf(stderr, "[MEM-CHK] chunk(%d,%d) 0x34001=0x%llX（基线 0x%llX——被写坏！）\n", chunkX, chunkZ, v0, baseline);
            }
        }
    }

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
    // 3a. density（独立循环，便于剖析与后续算法优化）；y 上限 = noiseHeight（下界 128，上方留 air）
    for (int by = 0; by < h->dim.noiseHeight; by++) {
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
    // WG_DBDEBUG 诊断：dump 指定列（世界坐标 WG_DBDEBUG_X/Z）的 densityBuf 原始密度（不经 aquifer/surface），
    // 格式对齐 cns 反射 dump（vanilla_density_*_cns.txt：y 递减，%.6f），便于直接 diff 区分 density 错 vs aquifer/surface 错
    if (getenv("WG_DBDEBUG")) {
        const char* sx = getenv("WG_DBDEBUG_X");
        const char* sz = getenv("WG_DBDEBUG_Z");
        if (sx && sz) {
            int bx = atoi(sx), bz = atoi(sz);
            if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
                int lx = bx - chunkX * 16;  // chunk 内局部坐标 0..15（densityBuf 按局部索引）
                int lz = bz - chunkZ * 16;
                for (int by = h->dim.noiseHeight - 1; by >= 0; by--) {
                    int wy = h->dim.minY + by;
                    std::fprintf(stderr, "%d %.6f\n", wy, densityBuf[by * 256 + lz * 16 + lx]);
                }
            }
        }
    }
    // WG_COMPDUMP 诊断：dump 指定列全部 router 组件（barrier/fluid/vein 等纯噪声无插值，与 vanilla router.*().sample() 可直接对比）
    if (getenv("WG_COMPDUMP")) {
        const char* sx = getenv("WG_COMPDUMP_X");
        const char* sz = getenv("WG_COMPDUMP_Z");
        if (sx && sz) {
            int bx = atoi(sx), bz = atoi(sz);
            if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
                const char* comps[] = {"depth", "continents", "erosion", "barrier", "fluid_level_floodedness",
                                       "fluid_level_spread", "lava", "vein_toggle", "vein_ridged", "vein_gap",
                                       "temperature", "vegetation"};
                NoisePos p;
                for (const char* c : comps) {
                    auto it = R.find(c);
                    if (it == R.end()) continue;
                    for (int y = h->dim.minY; y <= h->dim.minY + h->dim.noiseHeight - 1; y += 4) {
                        p.x = bx; p.y = y; p.z = bz;
                        std::fprintf(stderr, "[COMP] %s %d %.6f\n", c, y, it->second->sample(p));
                    }
                }
            }
        }
    }
    // WG_SURFDUMP 诊断：dump 指定列的表面高度估计与 initialDensity/finalDensity 剖面
    if (getenv("WG_SURFDUMP")) {
        const char* sx = getenv("WG_SURFDUMP_X");
        const char* sz = getenv("WG_SURFDUMP_Z");
        if (sx && sz) {
            int bx = atoi(sx), bz = atoi(sz);
            if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
                NoisePos p;
                for (int y = -64; y <= 127; y += 4) {
                    p.x = bx; p.y = y; p.z = bz;
                    std::fprintf(stderr, "[SURF] (%d,%d,%d) initialDensity=%.6f finalDensity=%.6f\n", bx, y, bz,
                                 R["initial_density"]->sample(p), h->finalDensity->sample(p));
                }
                std::fprintf(stderr, "[SURF] estimateSurfaceHeight(%d,%d)=%d\n", bx, bz,
                             aquifer ? aquifer->estimateSurfaceHeight(bx, bz) : 0);
                // 分量 dump（y 可配 WG_SURFDUMP_Y，默认 31）
                p.y = getenv("WG_SURFDUMP_Y") ? atoi(getenv("WG_SURFDUMP_Y")) : 31;
                const char* comps[] = {"base_3d_noise", "factor", "depth", "jaggedness", "continents", "erosion"};
                for (const char* c : comps) {
                    DF df = h->builder->getFunction("minecraft:overworld/" + std::string(c));
                    if (df) std::fprintf(stderr, "[SURF] %s(y=%d)=%.6f\n", c, p.y, df->sample(p));
                    else std::fprintf(stderr, "[SURF] %s(y=%d)=<missing>\n", c, p.y);
                }
                if (R.count("barrier"))
                    std::fprintf(stderr, "[SURF] barrier(y=%d)=%.6f\n", p.y, R["barrier"]->sample(p));
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
    // WG_NOODLEDUMP 诊断：dump noodle 树 raw 噪声采样 + noodle 树值（高频丢失排查，-288 含水层课题）
    if (getenv("WG_NOODLEDUMP")) {
        const char* sx = getenv("WG_NOODLEDUMP_X");
        const char* sz = getenv("WG_NOODLEDUMP_Z");
        if (sx && sz) {
            int bx = atoi(sx), bz = atoi(sz);
            if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
                NoisePos p;
                auto ns = h->builder->getNoiseSampler("minecraft:noodle");
                auto ts = h->builder->getNoiseSampler("minecraft:noodle_thickness");
                auto ra = h->builder->getNoiseSampler("minecraft:noodle_ridge_a");
                auto rb = h->builder->getNoiseSampler("minecraft:noodle_ridge_b");
                auto cc = h->builder->getNoiseSampler("minecraft:cave_cheese");
                auto cl = h->builder->getNoiseSampler("minecraft:cave_layer");
                DF noodle = h->builder->getFunction("minecraft:overworld/caves/noodle");
                for (int y = 0; y <= 30; y++) {
                    p.x = bx; p.y = y; p.z = bz;
                    double nv = ns ? ns->sample((double)bx, (double)y, (double)bz) : -999.0;
                    double tv = ts ? ts->sample((double)bx, (double)y, (double)bz) : -999.0;
                    double av = ra ? ra->sample((double)bx * 2.6666666666666665, (double)y * 2.6666666666666665, (double)bz * 2.6666666666666665) : -999.0;
                    double bv = rb ? rb->sample((double)bx * 2.6666666666666665, (double)y * 2.6666666666666665, (double)bz * 2.6666666666666665) : -999.0;
                    double ccv = cc ? cc->sample((double)bx, (double)y * 0.6666666666666666, (double)bz) : -999.0;
                    double clv = cl ? cl->sample((double)bx, (double)y * 8.0, (double)bz) : -999.0;
                    double nl = noodle ? noodle->sample(p) : -999.0;
                    std::fprintf(stderr, "[NOODLE] (%d,%d,%d) raw_n=%.6f raw_t=%.6f raw_a=%.6f raw_b=%.6f raw_cheese=%.6f raw_layer=%.6f tree=%.6f\n",
                                 bx, y, bz, nv, tv, av, bv, ccv, clv, nl);
                }
            }
        }
    }
    // 3b. aquifer + oreVein（ChainedBlockSource：aquifer null → oreVein；下界两者都 null）
    for (int by = 0; by < h->dim.noiseHeight; by++) {
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
        // Java BiomeAccess.getBiome(BlockPos)：8 邻域 seed 哈希选点（method_38106 距离）
        // → storage.getBiomeForNoiseGen(px, py, pz) → MultiNoiseBiomeSource.getBiome
        //   → sampler.sample(px,py,pz) 内部 ×4 回 block 采样
        int px, py, pz;
        wg::biomePickCell(h->biomeAccessSeed, x, y, z, px, py, pz);
        p.x = px << 2;
        p.y = py << 2;
        p.z = pz << 2;
        auto samp = [&](const char* k, const NoisePos& q) -> float {
            auto it = R.find(k);
            return it != R.end() ? (float)it->second->sample(q) : 0.0f;  // 维度缺组件（mod/简化维度）→ 0
        };
        float t = samp("temperature", p);
        float hum = samp("vegetation", p);
        float cont = samp("continents", p);
        float ero = samp("erosion", p);
        float dep = samp("depth", p);
        float w = samp("ridges", p);
        const std::string* id = h->biomeSource.find(t, hum, cont, ero, dep, w);
        return id ? *id : "minecraft:plains";
    };
    auto biomeTemp = [&](const std::string& id) -> double {
        return h->biomeSource.temperature(id);
    };
    auto biomeCellKey = [&](int x, int y, int z) -> int64_t {
        int px, py, pz;
        wg::biomePickCell(h->biomeAccessSeed, x, y, z, px, py, pz);
        return ((int64_t)((uint64_t)(uint32_t)px << 40)) | ((int64_t)((uint64_t)(uint32_t)py << 20)) | (uint32_t)pz;
    };
    std::vector<int> sh4(4);
    sh4[0] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16, chunkZ * 16) : 0;
    sh4[1] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16) : 0;
    sh4[2] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16, chunkZ * 16 + 16) : 0;
    sh4[3] = aquifer ? aquifer->estimateSurfaceHeight(chunkX * 16 + 16, chunkZ * 16 + 16) : 0;
    if (getenv("WG_ESTDUMP")) {
        std::fprintf(stderr, "[ESTDUMP] chunk(%d,%d) sh4=%d %d %d %d\n", chunkX, chunkZ, sh4[0], sh4[1], sh4[2], sh4[3]);
        int bx = getenv("WG_ESTDUMP_X") ? atoi(getenv("WG_ESTDUMP_X")) : -244;
        int bz = getenv("WG_ESTDUMP_Z") ? atoi(getenv("WG_ESTDUMP_Z")) : -256;
        if (chunkX * 16 <= bx && bx < chunkX * 16 + 16 && chunkZ * 16 <= bz && bz < chunkZ * 16 + 16) {
            std::fprintf(stderr, "[ESTDUMP] (%d,%d) singleEst=%d\n", bx, bz,
                         aquifer ? aquifer->estimateSurfaceHeight(bx, bz) : 0);
        }
    }
    h->surfaceBuilder->buildSurface(col, h->overworldRule, chunkX * 16, chunkZ * 16, heightmap, sh4, biomeAt, biomeCellKey, biomeTemp,
                                    h->dim.minY, h->dim.worldHeight,
                                    [&R](int x, int y, int z) -> double {
                                        auto it = R.find("initial_density");
                                        if (it == R.end()) return 0.0;
                                        NoisePos q; q.x = x; q.y = y; q.z = z;
                                        return it->second->sample(q);
                                    });
    if (profiling) {
        double tEnd = nowMs();
        std::fprintf(stderr, "[PROF] chunk(%d,%d): density=%.2fms aquifer+oreVein=%.2fms sh4+surface=%.2fms total=%.2fms\n",
                     chunkX, chunkZ, tA - t0, tB - tA, tEnd - tB, tEnd - t0);
    }

    // 5. 输出（维度化：worldHeight 决定 out 大小；overworld 98304 / nether 65536）
    const size_t outCount = (size_t)h->dim.worldHeight * 256;
    std::memcpy(out, col.data().data(), outCount * sizeof(int32_t));
    return (int)outCount;
}

// 单 chunk（串行兼容入口）
int wg_fill_blocks(void* handle, int chunkX, int chunkZ, int32_t* out) {
    return fillOneChunk(handle, chunkX, chunkZ, out);
}

// 多 chunk 并行：chunkXs/chunkZs/outs 为 count 个 chunk 的坐标与输出缓冲。
// 每个 chunk 独立生成（确定性随机派生 + thread_local 缓存），结果与串行逐位一致。
// Windows：GetLogicalProcessorInformationEx 数物理核（SMT 不重复计；未来可区分 P/E 核）
#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#endif
static int physicalCoreCount() {
#ifdef _WIN32
    DWORD len = 0;
    GetLogicalProcessorInformationEx(RelationProcessorCore, nullptr, &len);
    if (len > 0) {
        std::vector<char> buf(len);
        if (GetLogicalProcessorInformationEx(RelationProcessorCore,
                reinterpret_cast<PSYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(buf.data()), &len)) {
            int cores = 0;
            size_t off = 0;
            while (off < len) {
                auto* p = reinterpret_cast<PSYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(buf.data() + off);
                if (p->Relationship == RelationProcessorCore) cores++;
                off += p->Size;
            }
            if (cores > 0) return cores;
        }
    }
#endif
    int hc = (int)std::thread::hardware_concurrency();
    return hc > 1 ? hc / 2 : 1;  // 非 Windows 兜底：假设 SMT 减半
}

// ---- 持久线程池（复用，避免每次 fillBlocks 创建/销毁线程——用户指出）----
// 首次调用时按物理核数创建；后续 fillBlocks 只分发任务，线程常驻。
class CoreSwapPool {
public:
    static CoreSwapPool& instance() { static CoreSwapPool p; return p; }
    void shutdownNow() { shutdown(); }

    void ensure(int n) {
        std::lock_guard<std::mutex> l(mtx);
        if (n <= (int)workers.size()) return;
        int add = n - (int)workers.size();
        for (int i = 0; i < add; i++) {
            workers.emplace_back([this] {
                for (;;) {
                    int taskId;
                    {
                        std::unique_lock<std::mutex> l(mtx);
                        cvTask.wait(l, [this] { return stop || taskQueue > 0; });
                        if (stop && taskQueue == 0) return;
                        taskId = nextTask++;
                        taskQueue--;
                    }
                    if (fn) fn(taskId);
                    {
                        std::lock_guard<std::mutex> l(mtx);
                        doneCount++;
                        if (doneCount == totalTasks) cvDone.notify_one();
                    }
                }
            });
        }
    }

    // 执行 count 个任务（并行），主线程阻塞直到全部完成。
    void run(int count, const std::function<void(int)>& f) {
        if (count <= 0) return;
        // ⚠️ 并发 run 保护：MC 的 worldgen 线程池（多个 Worker）会并发调 fillBlocks → wg_fill_blocks_multi → run。
        // fn/totalTasks/doneCount/nextTask/taskQueue 是共享成员——并发 run 会互相覆盖（A 的 run 尾 fn=nullptr
        // 被 B 的 workers 读空 → 调用空 std::function → 读地址 0 崩溃（用户 32 视距崩溃的根因）。
        static std::mutex runMtx;
        std::lock_guard<std::mutex> lr(runMtx);
        if (workers.empty()) ensure(count);
        {
            std::lock_guard<std::mutex> l(mtx);
            fn = f;
            totalTasks = count;
            doneCount = 0;
            nextTask = 0;
            taskQueue = count;
        }
        cvTask.notify_all();
        {
            std::unique_lock<std::mutex> l(mtx);
            cvDone.wait(l, [this] { return doneCount >= totalTasks; });
            fn = nullptr;
        }
    }

    // 停止并回收所有 worker（进程退出 / wg_destroy 时调用，避免 terminate/use-after-free）
    void shutdown() {
        {
            std::lock_guard<std::mutex> l(mtx);
            stop = true;
        }
        cvTask.notify_all();
        for (auto& w : workers)
            if (w.joinable()) w.join();
        workers.clear();
    }

private:
    CoreSwapPool() = default;
    ~CoreSwapPool() { shutdown(); }
    CoreSwapPool(const CoreSwapPool&) = delete;
    CoreSwapPool& operator=(const CoreSwapPool&) = delete;
    std::vector<std::thread> workers;
    std::function<void(int)> fn;
    std::mutex mtx;
    std::condition_variable cvTask, cvDone;
    bool stop = false;
    int taskQueue = 0, nextTask = 0, doneCount = 0, totalTasks = 0;
};

// 线程池停止（定义在 CoreSwapPool 之后；wg_destroy 前向调用）
void shutdownCoreSwapPool() { CoreSwapPool::instance().shutdownNow(); }

int wg_fill_blocks_multi(void* handle, const int* chunkXs, const int* chunkZs,
                         int32_t* const* outs, int count, int threads) {
    if (count <= 0) return 0;
    // 崩溃定位日志（每次调用打批次信息；正常运行时开 WG_FBLOG=1 才打印，防刷屏）
    if (getenv("WG_FBLOG")) {
        std::fprintf(stderr, "[FBLOCK] count=%d first=(%d,%d) last=(%d,%d) threads=%d\n",
                     count, chunkXs[0], chunkZs[0], chunkXs[count - 1], chunkZs[count - 1], threads);
    }
    if (threads <= 0) {
        // 模式自适应：-1=服务端全核、-2=客户端留 2 核（渲染/主线程）、0=默认（同 -1）
        // Issue #7：4C8T 上逻辑线程(8)过分配；CORESWAP_THREADS 显式覆盖优先
        const char* envT = getenv("CORESWAP_THREADS");
        if (envT && *envT) threads = std::atoi(envT);
        else {
            int pc = physicalCoreCount();
            threads = (threads == -2) ? (pc > 2 ? pc - 2 : 1) : pc;
        }
        if (threads <= 0) threads = 1;
    }
    if (threads > count) threads = count;
    // 线程复用：持久线程池（首次按模式线程数创建，后续复用——不每次创建/销毁 std::thread）
    int poolThreads = threads;  // 已含 CORESWAP_THREADS 处理（threads<=0 分支），不二次读 env
    if (poolThreads <= 0) poolThreads = 1;
    CoreSwapPool::instance().ensure(poolThreads);
    CoreSwapPool::instance().run(count, [&](int i) {
        wg::g_crashContext = "wg_fill_blocks_multi/fillOneChunk";
        try {
            fillOneChunk(handle, chunkXs[i], chunkZs[i], outs[i]);
        } catch (const std::exception& e) {
            std::fprintf(stderr, "[CORESWAP-EXC] chunk(%d,%d) C++ exception: %s\n", chunkXs[i], chunkZs[i], e.what());
        } catch (...) {
            std::fprintf(stderr, "[CORESWAP-EXC] chunk(%d,%d) unknown C++ exception\n", chunkXs[i], chunkZs[i]);
        }
        wg::g_crashContext = nullptr;
    });
    return count;
}

} // extern "C"













