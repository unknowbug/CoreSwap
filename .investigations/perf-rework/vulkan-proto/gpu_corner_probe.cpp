// gpu_corner_probe.cpp —— D23 H1 裁决：GPU valBuf 8 角点值 vs CPU 参照角点值
// 对错点 (784,160,-408)（cy=28, cz=2）与对点 (784,160,-416)（cy=28, cz=1），
// 读 GPU valBuf 的 8 角点区段（D15：每采样点 9 区段 = 8 角点 + 1 顶层），与 CPU 参照 finalDensity
// 的 cell 8 角点值对比——若 GPU 角点值错 → H1（角点读取/映射）；若角点对但插值错 → 插值层。
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

static int floorDivI(int a, int b) { int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }

static void checkPoint(GpuDensityEngine& engine, wg::DF& fdDF, int px, int py, int pz, const char* tag) {
    constexpr int MIN_Y = -64;
    constexpr int INTERP4_BASE = 304;   // val 布局（dump_val_layout.py）：interp_4 角点区段基址
    constexpr int INTERP4_PEAK = 6;     // 每角点槽数
    constexpr int INTERP4_ROOT_SLOT = 3; // SLOT_OF_4[17]=3（角点最终值槽）
    int chunkX = floorDivI(px, 16), chunkZ = floorDivI(pz, 16);
    int gx = px - chunkX * 16, gy = py - MIN_Y, gz = pz - chunkZ * 16;
    int cx = gx / 4, cy = gy / 8, cz = gz / 4;
    std::printf("[%s] (%d,%d,%d) cell cx=%d cy=%d cz=%d\n", tag, px, py, pz, cx, cy, cz);
    // GPU valBuf（N=1）
    int32_t c[3] = {px, py, pz};
    std::vector<float> vb((size_t)engine.perSample());
    int ps = engine.dumpValBuf(c, 1, vb.data());
    // GPU 8 角点值（valBuf[INTERP4_BASE + corner*6 + 3]）vs CPU 参照 cell 角点值
    std::printf("  corner | GPU角点值 | CPU ref @角点 | diff\n");
    for (int cc = 0; cc < 8; cc++) {
        int dx = cc & 1, dy = (cc >> 1) & 1, dz = (cc >> 2) & 1;
        int ax = chunkX * 16 + (cx + dx) * 4, ay = MIN_Y + (cy + dy) * 8, az = chunkZ * 16 + (cz + dz) * 4;
        wg::NoisePos pos{ ax, ay, az };
        double ref = fdDF->sample(pos);
        float gv = vb[(size_t)INTERP4_BASE + cc * INTERP4_PEAK + INTERP4_ROOT_SLOT];
        std::printf("  c%d (%d,%d,%d): gpu=%.6f cpu=%.6f diff=%.3e %s\n",
                    cc, ax, ay, az, gv, ref, std::fabs((double)gv - ref),
                    std::fabs((double)gv - ref) > 1e-4 ? "<== DIFF" : "");
    }
    // GPU 最终输出 vs CPU 参照采样点
    float gpuVal = vb[(size_t)ps - 1];   // 顶层区段尾 = 最终值（近似）
    wg::NoisePos p2{ px, py, pz };
    double cpuVal = fdDF->sample(p2);
    std::printf("  [final] gpu(tail)=%.6f cpu=%.6f\n", gpuVal, cpuVal);
    std::printf("  [tail] vb[%d..%d]=", ps - 6, ps - 1);
    for (int k = ps - 6; k < ps; k++) std::printf(" %.4f", vb[(size_t)k]);
    std::printf("\n");
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
    checkPoint(engine, fdDF, 784, 160, -408, "BAD cy=28 cz=2");
    checkPoint(engine, fdDF, 784, 160, -416, "OK  cy=28 cz=1");
    checkPoint(engine, fdDF, 784, -64, -408, "OK  cy=0  cz=2");
    std::printf("[done]\n");
    return 0;
}
