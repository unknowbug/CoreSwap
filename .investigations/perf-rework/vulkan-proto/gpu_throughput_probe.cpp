// gpu_throughput_probe.cpp —— I5：wg_fill_density 语义的 GPU vs CPU 吞吐对比
// 同批量坐标（N chunks × 768 点），CPU 路径（finalDensity->sample 循环）vs GPU 路径（GpuDensityEngine.fill）
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <string>
#include <cmath>
#include <chrono>
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
    // chunk 批量档位（默认 1/4/16/64）
    std::vector<int> batchChunks = {1, 4, 16, 64};
    if (argc > 1) { batchChunks.clear(); for (int a = 1; a < argc; a++) batchChunks.push_back(std::atoi(argv[a])); }

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
    wg::DF fdDF = builder.buildNode(*fdv);

    // GPU 引擎
    std::fprintf(stderr, "[probe] constructing GpuDensityEngine (pipeline one-time)...\n");
    GpuDensityEngine engine(worldSeed, "final_density.spv");

    constexpr int SX = 4, SY = 48, SZ = 4;         // 与 wg_fill_density 相同网格
    constexpr int XZ = 4, Y = 8, MIN_Y = -64;      // 与 wg_fill_density 相同间隔
    constexpr int PPC = SX * SY * SZ;              // 768/chunk

    std::printf("# throughput GPU vs CPU (final_density, seed 8576294172403134396)\n");
    std::printf("# chunks | points | CPU(ms) | GPU(ms) | CPU pts/s | GPU pts/s | speedup\n");
    for (int nChunks : batchChunks) {
        const int n = nChunks * PPC;
        std::vector<int32_t> coords(3 * n);
        for (int c = 0; c < nChunks; c++) {
            int chunkX = 720 / 16 + (c % 8);   // 8576 区域附近
            int chunkZ = -432 / 16 + (c / 8);
            int idx = c * PPC;
            for (int y = 0; y < SY; y++)
                for (int z = 0; z < SZ; z++)
                    for (int x = 0; x < SX; x++) {
                        coords[3*(idx)+0] = chunkX * 16 + x * XZ;
                        coords[3*(idx)+1] = MIN_Y + y * Y;
                        coords[3*(idx)+2] = chunkZ * 16 + z * XZ;
                        idx++;
                    }
        }
        // CPU：finalDensity->sample 循环（wg_fill_density 语义）
        std::vector<double> cpuOut(n);
        auto t0 = std::chrono::steady_clock::now();
        for (int i = 0; i < n; i++) {
            wg::NoisePos pos{ coords[3*i+0], coords[3*i+1], coords[3*i+2] };
            cpuOut[i] = fdDF->sample(pos);
        }
        auto t1 = std::chrono::steady_clock::now();
        // GPU：fill
        std::vector<float> gpuOut(n);
        auto t2 = std::chrono::steady_clock::now();
        engine.fill(coords.data(), n, gpuOut.data());
        auto t3 = std::chrono::steady_clock::now();
        double cpuMs = std::chrono::duration<double, std::milli>(t1 - t0).count();
        double gpuMs = std::chrono::duration<double, std::milli>(t3 - t2).count();
        // 正确性抽查（同点 diff）
        double maxDiff = 0;
        int maxDiffIdx = -1;
        for (int i = 0; i < n; i++) {
            double d = std::fabs((double)gpuOut[i] - cpuOut[i]);
            if (d > maxDiff) { maxDiff = d; maxDiffIdx = i; }
        }
        if (maxDiffIdx >= 0)
            std::printf("  top diff @ (%d,%d,%d) gpu=%.9f cpu=%.9f\n",
                        coords[3*maxDiffIdx+0], coords[3*maxDiffIdx+1], coords[3*maxDiffIdx+2],
                        gpuOut[maxDiffIdx], cpuOut[maxDiffIdx]);
        std::printf("%6d | %7d | %7.2f | %7.2f | %.0f | %.0f | %.2fx (maxDiff=%.2e)\n",
                    nChunks, n, cpuMs, gpuMs,
                    n / (cpuMs / 1000.0), n / (gpuMs / 1000.0),
                    cpuMs / gpuMs, maxDiff);
    }
    std::printf("[done]\n");
    return 0;
}
