// gpu_domain_probe.cpp —— I5 盲区定位：GPU 引擎在已知正确域 vs 新 chunk 域的行为
// 对比 engine.sample vs DensityBuilder 参照，跨多个 chunk 坐标域
#include <cstdio>
#include <cstdlib>
#include <vector>
#include <fstream>
#include <string>
#include <cmath>
#include "gpu_density_engine.h"
#include "density_builder.h"

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
static wg::DensityBuilder::NoiseParamsMap loadNoiseParams(const std::string& noiseDir) {
    wg::DensityBuilder::NoiseParamsMap m;
    for (const auto& fn : listJson(noiseDir)) {
        std::string key = "minecraft:" + fn.substr(0, fn.size() - 5);
        wg::JsonParser parser(readFile(noiseDir + "\\" + fn));
        wg::JsonValue root = parser.parse();
        const wg::JsonValue* amps = root.get("amplitudes");
        wg::DoublePerlinNoiseSampler::NoiseParameters np;
        np.firstOctave = (int32_t)root.num("firstOctave", 0.0);
        if (amps && amps->isArray()) for (const auto& a : amps->arr) np.amplitudes.push_back(a.numVal);
        m[key] = np;
    }
    return m;
}

int main() {
    setvbuf(stderr, nullptr, _IONBF, 0);
    const uint64_t worldSeed = 8576294172403134396ULL;
    const std::string dfDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function";
    const std::string noiseDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise";
    const std::string settingsPath = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    auto noiseParams = loadNoiseParams(noiseDir);
    wg::DensityBuilder builder(worldSeed, noiseParams);
    builder.externalLoader = [&](const std::string& ref, const std::string& name) -> wg::DF {
        return builder.parseFile(ref, readFile(dfDir + "\\overworld\\" + name + ".json"));
    };
    wg::JsonParser sp(readFile(settingsPath));
    wg::JsonValue settings = sp.parse();
    const wg::JsonValue* nr = settings.get("noise_router");
    wg::DF fdDF = builder.buildNode(*nr->get("final_density"));

    std::fprintf(stderr, "[probe] constructing engine...\n");
    GpuDensityEngine engine(worldSeed, "final_density.spv");

    // 测试点：已知正确域 + 各 chunk 域
    int pts[][3] = {
        {0, -64, 0}, {44, -49, 4}, {63, -49, 2},           // e2e 域（已验证正确）
        {784, 160, -408}, {784, -64, -408}, {784, 160, -416}, // 新 chunk(49,-26) 域
        {720, 160, -432}, {816, 160, -336},                  // 8576 区域
        {45*16, 160, -27*16}, {52*16, 160, -26*16},          // 边界
    };
    std::printf("# domain probe (seed 8576294172403134396)\n");
    for (auto& p : pts) {
        float g = engine.sample(p[0], p[1], p[2]);
        wg::NoisePos pos{ p[0], p[1], p[2] };
        double c = fdDF->sample(pos);
        std::printf("(%6d,%5d,%6d) gpu=%.9f cpu=%.9f diff=%.3e %s\n",
                    p[0], p[1], p[2], g, c, std::fabs((double)g - c),
                    std::fabs((double)g - c) > 1e-4 ? " <== DIFF" : "");
    }
    // z 网格扫描（y=160, x=784, z=-432..-404 每 4 一格——定位 z 网格 6 特殊性）
    std::printf("# z-scan y=160 x=784\n");
    for (int z = -432; z <= -404; z += 4) {
        float g = engine.sample(784, 160, z);
        wg::NoisePos pos{ 784, 160, z };
        double c = fdDF->sample(pos);
        std::printf("  z=%d gpu=%.9f cpu=%.9f diff=%.3e %s\n", z, g, c, std::fabs((double)g - c),
                    std::fabs((double)g - c) > 1e-4 ? " <== DIFF" : "");
    }
    // y 扫描（x=784, z=-408, y=-64..312 每 8 一格）
    std::printf("# y-scan x=784 z=-408\n");
    for (int y = -64; y <= 312; y += 8) {
        float g = engine.sample(784, y, -408);
        wg::NoisePos pos{ 784, y, -408 };
        double c = fdDF->sample(pos);
        std::printf("  y=%d gpu=%.9f cpu=%.9f diff=%.3e %s\n", y, g, c, std::fabs((double)g - c),
                    std::fabs((double)g - c) > 1e-4 ? " <== DIFF" : "");
    }
    std::printf("[done]\n");
    return 0;
}
