// ref_probe.cpp —— CPU 参照(DensityBuilder) 采样 final_density 及 sloped_cheese 等分量
// 用途：定位 D17 y>-64 语义差异——参照 sloped_cheese/range_choice 分支值 vs 模拟/GPU
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

static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(1); }
    return std::string((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
}
static std::vector<std::string> listJson(const std::string& dir) {
    std::vector<std::string> out;
    std::string cmd = "dir /b \"" + dir + "\\*.json\"";
    FILE* p = _popen(cmd.c_str(), "r");
    if (!p) return out;
    char buf[512];
    while (fgets(buf, sizeof(buf), p)) { std::string s(buf); while (!s.empty() && (s.back()=='\n'||s.back()=='\r')) s.pop_back(); if (!s.empty()) out.push_back(s); }
    _pclose(p);
    return out;
}
static DensityBuilder::NoiseParamsMap loadNoiseParams(const std::string& noiseDir) {
    DensityBuilder::NoiseParamsMap m;
    for (const auto& fn : listJson(noiseDir)) {
        std::string key = "minecraft:" + fn.substr(0, fn.size() - 5);
        JsonParser parser(readFile(noiseDir + "\\" + fn));
        JsonValue root = parser.parse();
        const JsonValue* amps = root.get("amplitudes");
        DoublePerlinNoiseSampler::NoiseParameters np;
        np.firstOctave = (int32_t)root.num("firstOctave", 0.0);
        if (amps && amps->isArray()) for (const auto& a : amps->arr) np.amplitudes.push_back(a.numVal);
        m[key] = np;
    }
    return m;
}

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    const std::string dfDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function";
    const std::string noiseDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise";
    const std::string settingsPath = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";

    auto noiseParams = loadNoiseParams(noiseDir);
    DensityBuilder builder(worldSeed, noiseParams);
    builder.externalLoader = [&](const std::string& ref, const std::string& name) -> DF {
        return builder.parseFile(ref, readFile(dfDir + "\\overworld\\" + name + ".json"));
    };
    // 注册分量（供 getRegistryEntry）
    std::vector<std::string> dfFiles = {
        "base_3d_noise", "continents", "depth", "erosion", "factor",
        "jaggedness", "offset", "ridges", "ridges_folded", "sloped_cheese",
        "caves/entrances", "caves/noodle", "caves/pillars",
        "caves/spaghetti_2d_thickness_modulator", "caves/spaghetti_2d",
        "caves/spaghetti_roughness_function",
    };
    for (const auto& f : dfFiles) builder.registerFunction("minecraft:overworld/" + f, std::make_shared<DensityBuilder::LazyRef>());
    for (const auto& f : dfFiles) {
        std::string path = dfDir + "\\overworld\\" + f + ".json";
        if (std::ifstream(path).good())
            builder.registerFunction("minecraft:overworld/" + f, builder.parseFile("minecraft:overworld/" + f, readFile(path)));
    }

    JsonParser sp(readFile(settingsPath));
    JsonValue settings = sp.parse();
    const JsonValue* nr = settings.get("noise_router");
    const JsonValue* fdv = nr ? nr->get("final_density") : nullptr;
    if (!fdv) { std::fprintf(stderr, "no final_density\n"); return 1; }
    DF fdDF = builder.buildNode(*fdv);
    DF fdArg1 = builder.buildNode(*fdv->get("argument1"));  // squeeze(0.64*interp(...))
    DF fdArg2 = builder.buildNode(*fdv->get("argument2"));  // range_choice(...)
    std::printf("final_density DF built\n");

    auto reg = [&](const char* key) -> DF { return builder.getRegistryEntry(std::string("minecraft:overworld/") + key); };
    DF sc  = reg("sloped_cheese");
    DF fac = reg("factor");
    DF dep = reg("depth");
    DF ent = reg("caves/entrances");
    DF lay = reg("base_3d_noise");
    DF s2d = reg("caves/spaghetti_2d");
    DF srg = reg("caves/spaghetti_roughness_function");
    DF pil = reg("caves/pillars");

    // base_3d_noise via InterpolatedNoiseDF
    XoroshiroRandom b3dRnd = builder.randomDeriverPublic().split("minecraft:terrain");
    InterpolatedNoiseDF b3d(b3dRnd, 0.25, 0.125, 80.0, 160.0, 8.0);

    NoisePos pos;
    std::printf("# x=0 z=0 column\n");
    for (int y = -64; y <= -40; y += 2) {
        pos.x = 0; pos.y = y; pos.z = 0;
        double fd = fdDF ? fdDF->sample(pos) : 0.0;
        double a1 = fdArg1 ? fdArg1->sample(pos) : 0.0;
        double a2 = fdArg2 ? fdArg2->sample(pos) : 0.0;
        double scv = sc ? sc->sample(pos) : 0.0;
        double fv = fac ? fac->sample(pos) : 0.0;
        double dv = dep ? dep->sample(pos) : 0.0;
        double ev = ent ? ent->sample(pos) : 0.0;
        double lv = lay ? lay->sample(pos) : 0.0;
        double s2 = s2d ? s2d->sample(pos) : 0.0;
        double sr = srg ? srg->sample(pos) : 0.0;
        double pv = pil ? pil->sample(pos) : 0.0;
        std::printf("y=%d fd=%.9f arg1=%.9f arg2=%.9f sloped=%.9f factor=%.9f depth=%.9f entrances=%.9f base3d=%.9f spag2d=%.9f spagrough=%.9f pillars=%.9f\n",
                    y, fd, a1, a2, scv, fv, dv, ev, lv, s2, sr, pv);
    }
    // 额外非零 x/z 点（ws split 跨角点验证）
    int extra[][3] = {{5, -52, 3}, {12, -58, 7}, {63, -50, 0}, {1, -55, 1}};
    std::printf("# extra points\n");
    for (auto& e : extra) {
        pos.x = e[0]; pos.y = e[1]; pos.z = e[2];
        std::printf("P %d %d %d fd=%.9f arg1=%.9f sloped=%.9f entrances=%.9f\n",
                    pos.x, pos.y, pos.z, fdDF ? fdDF->sample(pos) : 0.0,
                    fdArg1 ? fdArg1->sample(pos) : 0.0,
                    sc ? sc->sample(pos) : 0.0, ent ? ent->sample(pos) : 0.0);
    }
    // 全部 1024 点（与 e2e coords 一致：x=i%64, y=-64+(i/64%16), z=0）
    std::printf("# all1024\n");
    for (int i = 0; i < 1024; i++) {
        pos.x = i % 64; pos.y = -64 + (i / 64 % 16); pos.z = 0;
        std::printf("%d %.9f\n", i, fdDF ? fdDF->sample(pos) : 0.0);
    }
    return 0;
}
