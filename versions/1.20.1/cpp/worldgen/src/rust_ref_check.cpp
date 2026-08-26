// rust_ref_check.cpp — C++ 参照：用 density_builder.h 构建同一批 overworld density functions 并在同点采样。
// 用于与 WorldgenRust 的 overworld_probe.rs 输出逐位对比（校验 Rust buildNode 对齐）。
// 编译：cl /EHsc /utf-8 /std:c++17 /DNOMINMAX /MD /O2 /I worldgen\src rust_ref_check.cpp /Fe:rust_ref_check.exe
#include <cstdio>
#include <cstdint>
#include <map>
#include <string>
#include <vector>
#include <fstream>
#include <sstream>
#include "json.h"
#include "density.h"
#include "density_builder.h"

using namespace wg;

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
    std::stringstream ss; ss << f.rdbuf(); return ss.str();
}

int main() {
    uint64_t seed = 8576294172403134396ULL;
    auto np = buildNoiseParams();
    DensityBuilder builder(seed, np);
    std::string dfDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld\\";
    builder.externalLoader = [&](const std::string& fullRef, const std::string& name) -> DF {
        std::string path = dfDir + name + ".json";
        std::ifstream probe(path);
        if (!probe.good()) return nullptr;
        return builder.parseFile(fullRef, readFile(path));
    };
    const char* names[] = {"base_3d_noise","continents","erosion","ridges","ridges_folded","factor","offset","jaggedness","depth","sloped_cheese",
                           "caves/entrances","caves/noodle","caves/pillars","caves/spaghetti_2d","caves/spaghetti_2d_thickness_modulator","caves/spaghetti_roughness_function"};
    int pts[10][3] = {{0,0,0},{4,64,4},{8,128,8},{40,192,40},{100,-64,-40},{-64,64,-64},{128,288,128},{200,0,200},{16,-112,16},{72,320,72}};
    for (const char* name : names) {
        std::string full = std::string("minecraft:overworld/") + name;
        std::string path = dfDir + std::string(name) + ".json";
        DF df = builder.parseFile(full, readFile(path));
        builder.registerFunction(full, df);
        std::string line = std::string(name);
        for (auto& p : pts) {
            NoisePos pos{p[0], p[1], p[2]};
            char buf[128]; std::snprintf(buf, sizeof buf, " (%d,%d,%d)=%.8f", p[0], p[1], p[2], df->sample(pos));
            line += buf;
        }
        std::printf("%s  min=%.4f max=%.4f\n", line.c_str(), df->minValue(), df->maxValue());
    }
    // 构建完整 overworld finalDensity（noise_router.final_density）
    {
        std::string settingsPath = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
        JsonParser sp(readFile(settingsPath));
        JsonValue settings = sp.parse();
        const JsonValue* router = settings.get("noise_router");
        const JsonValue* fd = router->get("final_density");
        DF tree = builder.buildNode(*fd);
        int fpts[10][3] = {{0,0,0},{8,64,8},{100,-64,-40},{4,120,4},{-64,320,-64},{200,40,200},{16,-112,16},{72,240,72},{-200,96,96},{0,200,-16}};
        std::string line = "final_density";
        for (auto& p : fpts) {
            NoisePos pos{p[0], p[1], p[2]};
            char buf[160]; std::snprintf(buf, sizeof buf, " (%d,%d,%d)=%.8f", p[0], p[1], p[2], tree->sample(pos));
            line += buf;
        }
        std::printf("%s  min=%.8f max=%.8f\n", line.c_str(), tree->minValue(), tree->maxValue());
        // 抽样 (728,-408) 列（cpp_density_8576_45_-26_b8_8 参照；确认当前 C++ 与该参照是否一致）
        int cpts[6][3] = {{728,-64,-408},{728,-40,-408},{728,-8,-408},{728,0,-408},{728,120,-408},{728,319,-408}};
        std::string cline = "col728";
        for (auto& p : cpts) { NoisePos pos{p[0],p[1],p[2]}; char b[160]; std::snprintf(b,sizeof b," (%d,%d,%d)=%.8f",p[0],p[1],p[2],tree->sample(pos)); cline += b; }
        std::printf("%s\n", cline.c_str());
        // 整列 dump (728,-408) 到 stdout（供 chunkfill_probe 对比当前 C++）
        for (int y=-64; y<=319; y++) { NoisePos pos{728,y,-408}; std::printf("COL %d %.10f\n", y, tree->sample(pos)); }
        // 整块网格 dump：chunk(45,-26) 全部 16x16 列 × 10 个代表 y（覆盖 interpolated cell 网格 + 关键层）
        int ys[10] = {-64,-32,0,32,63,96,128,200,256,319};
        for (int bx=0; bx<16; bx++) for (int bz=0; bz<16; bz++) {
            int gx = 45*16+bx, gz = -26*16+bz;
            for (int y : ys) { NoisePos pos{gx,y,gz}; std::printf("GRID %d %d %d %.10f\n", gx, gz, y, tree->sample(pos)); }
        }
    }
    return 0;
}
