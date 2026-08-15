// gpu_fill_probe.cpp —— I2 引擎验证：GpuDensityEngine.fill 批量 vs DensityBuilder 参照逐点对比
// 复用 e2e 的参照构建逻辑（worldgen data + seed），验证 GPU 引擎接入正确性（先于 worldgen 集成）。
#include <cstdio>
#include <cstdlib>
#include <cstring>
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

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    const uint64_t worldSeed = 8576294172403134396ULL;
    const std::string dfDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function";
    const std::string noiseDir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise";
    const std::string settingsPath = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";

    // CPU 参照（DensityBuilder）
    auto noiseParams = loadNoiseParams(noiseDir);
    wg::DensityBuilder builder(worldSeed, noiseParams);
    builder.externalLoader = [&](const std::string& ref, const std::string& name) -> wg::DF {
        return builder.parseFile(ref, readFile(dfDir + "\\overworld\\" + name + ".json"));
    };
    wg::JsonParser sp(readFile(settingsPath));
    wg::JsonValue settings = sp.parse();
    const wg::JsonValue* nr = settings.get("noise_router");
    const wg::JsonValue* fdv = nr ? nr->get("final_density") : nullptr;
    if (!fdv) { std::fprintf(stderr, "no final_density\n"); return 1; }
    wg::DF fdDF = builder.buildNode(*fdv);

    // GPU 引擎（I2）
    std::fprintf(stderr, "[probe] constructing GpuDensityEngine (pipeline compile one-time)...\n");
    GpuDensityEngine engine(worldSeed, "final_density.spv");

    // 采样：N=1024（x=i%64, y=-64+i/64%16, z=0）+ z 覆盖（WG_E2E_Z 同语义，多平面）
    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    const bool zCover = std::getenv("WG_E2E_Z") != nullptr;
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = 0 + (i % 64);
        coords[3*i+1] = -64 + (i / 64 % 16);
        coords[3*i+2] = zCover ? (int32_t)((i / 256) * 2 - 2) : 0 + (i / 1024);
    }
    std::vector<float> out(N);
    engine.fill(coords.data(), (int)N, out.data());

    double maxDiff = 0.0, sumDiff = 0.0;
    struct DiffRec { uint32_t i; float gpu; double ref; double diff; };
    std::vector<DiffRec> top;
    for (uint32_t i = 0; i < N; i++) {
        wg::NoisePos pos{ coords[3*i+0], coords[3*i+1], coords[3*i+2] };
        double ref = fdDF->sample(pos);
        double diff = std::fabs((double)out[i] - ref);
        if (i < 8 || i % 256 == 0) std::printf("[DBG] i=%u pos=(%d,%d,%d) gpu=%.9f cpu=%.9f diff=%.3e\n", i, pos.x, pos.y, pos.z, out[i], ref, diff);
        if (diff > maxDiff) maxDiff = diff;
        sumDiff += diff;
        top.push_back({i, out[i], ref, diff});
    }
    std::sort(top.begin(), top.end(), [](const DiffRec& a, const DiffRec& b) { return a.diff > b.diff; });
    for (int k = 0; k < 8 && k < (int)top.size(); k++) {
        wg::NoisePos pos{ coords[3*top[k].i+0], coords[3*top[k].i+1], coords[3*top[k].i+2] };
        std::printf("[TOP%02d] i=%u pos=(%d,%d,%d) gpu=%.9f cpu=%.9f diff=%.3e\n", k, top[k].i, pos.x, pos.y, pos.z, top[k].gpu, top[k].ref, top[k].diff);
    }
    std::printf("[result] gpu_fill_probe N=%u: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
    std::printf("[done]\n");
    return 0;
}
