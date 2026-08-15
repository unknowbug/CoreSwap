// dfc_final_backend_e2e.cpp —— final_density 完整树：DensityBuilder(CPU 参照) + CpuBackend + GPU
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <string>
#include <cmath>
#include <chrono>
#include "vulkan_runtime.h"
#include "cpu_backend.h"
#include "density_builder.h"

#define CHECK_VK(fn) do { VkResult _r = (fn); if (_r != VK_SUCCESS) { \
    std::fprintf(stderr, "VK error %d at %s:%d (%s)\n", _r, __FILE__, __LINE__, #fn); \
    std::exit(1); } } while (0)

static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(1); }
    return std::string((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
}
static std::vector<uint32_t> loadSpv(const char* path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path); std::exit(1); }
    std::streamsize n = f.tellg(); f.seekg(0);
    std::vector<uint32_t> code((size_t)n / 4); f.read((char*)code.data(), n);
    return code;
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

    // CPU 参照：从 noise_router.final_density 构建 DF
    auto noiseParams = loadNoiseParams(noiseDir);
    wg::DensityBuilder builder(worldSeed, noiseParams);
    builder.externalLoader = [&](const std::string& ref, const std::string& name) -> wg::DF {
        return builder.parseFile(ref, readFile(dfDir + "\\overworld\\" + name + ".json"));
    };
    wg::JsonParser sp(readFile(settingsPath));
    wg::JsonValue settings = sp.parse();
    const wg::JsonValue* nr = settings.get("noise_router");
    const wg::JsonValue* fdv = nr ? nr->get("final_density") : nullptr;
    if (!fdv) { std::fprintf(stderr, "no final_density in noise_router\n"); return 1; }
    wg::DF fdDF = builder.buildNode(*fdv);
    std::printf("final_density DF built\n");

    // GPU 侧：CpuBackend
    CpuBackend backend;
    backend.init(worldSeed);
    std::fprintf(stderr, "[step] CpuBackend.init done, splitTotal=%d\n", backend.splitTotal);

    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    // P2-1：z 采样覆盖（judge 项——原坐标 z=i/1024 恒 0，只覆盖 z=0 单列，spline 4 种 coordType
    // （ridges_folded/ridges/erosion/continents）触发未证实）。WG_E2E_Z=1 时 z 覆盖多平面：
    //   i=0..255   → z=-2，i=256..511 → z=0，i=512..767 → z=2，i=768..1023 → z=4
    //   每 256 组内 x=i%64（4 轮）× y=-64+(i/64%16)（4 层）
    const bool zCover = std::getenv("WG_E2E_Z") != nullptr;
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = 0 + (i % 64);
        coords[3*i+1] = -64 + (i / 64 % 16);
        coords[3*i+2] = zCover ? (int32_t)((i / 256) * 2 - 2) : 0 + (i / 1024);
    }
    std::fprintf(stderr, "[step] coords zCover=%d\n", (int)zCover);
    std::vector<float> splitCoord((size_t)backend.splitTotal * N);
    for (uint32_t s = 0; s < N; s++) {
        backend.split(coords[3*s+0], coords[3*s+1], coords[3*s+2], splitCoord.data() + s * backend.splitTotal);
    }
    std::vector<uint32_t> perm;
    backend.collectPerm(perm);
    // ---- dump splitCoord + perm + coords（供 Python CPU 模拟对比）----
    { std::ofstream f("split_dump.bin", std::ios::binary); f.write((const char*)splitCoord.data(), splitCoord.size()*4); }
    { std::ofstream f("perm_dump.bin", std::ios::binary); f.write((const char*)perm.data(), perm.size()*4); }
    { std::ofstream f("coords_dump.txt"); for (uint32_t i = 0; i < N; i++) f << coords[3*i+0] << " " << coords[3*i+1] << " " << coords[3*i+2] << "\n"; }
    std::fprintf(stderr, "[step] dumped splitCoord(%zu) perm(%zu) coords(%u)\n", splitCoord.size(), perm.size(), N);
    std::fprintf(stderr, "[step] split + collectPerm done, N=%u permSize=%d\n", N, backend.permSize);

    // ---- Vulkan（I1：复用 VkRuntime 组件，语义与内联版逐位一致）----
    VkRuntime rt;
    rt.init();
    auto tp0 = std::chrono::steady_clock::now();
    rt.createPipeline("final_density.spv");
    auto tp1 = std::chrono::steady_clock::now();
    std::printf("[dbg] pipeline created in %.1fs\n", std::chrono::duration<double>(tp1 - tp0).count());

    VkDeviceSize coordSize = coords.size() * sizeof(int32_t);
    VkDeviceSize permSize = perm.size() * sizeof(uint32_t);
    VkDeviceSize splitSize = splitCoord.size() * sizeof(float);
    VkDeviceSize outSize = N * sizeof(float);
    const uint32_t PER_SAMPLE = (uint32_t)backend.perSample;   // D19: 从生成器取（曾硬编码 320 → ws 后 352 越界 → 尾部输出 0）
    VkDeviceSize valSize = (VkDeviceSize)N * PER_SAMPLE * sizeof(float);
    VkRuntime::Buffer coordBuf = rt.createBuffer(coordSize), permBuf = rt.createBuffer(permSize);
    VkRuntime::Buffer splitBuf = rt.createBuffer(splitSize), outBuf = rt.createBuffer(outSize), valBuf = rt.createBuffer(valSize);
    // A1b/A2：spline SSBO（生成器导出数据，binding 6-11）
    VkDeviceSize npSize = backend.splineNodePack.size() * sizeof(int32_t);
    VkDeviceSize locSize = backend.splineLocs.size() * sizeof(float);
    VkDeviceSize derSize = backend.splineDers.size() * sizeof(float);
    VkDeviceSize vfSize  = backend.splineValF.size() * sizeof(float);
    VkDeviceSize vkSize  = backend.splineValKind.size() * sizeof(int32_t);
    VkDeviceSize vnSize  = backend.splineValNode.size() * sizeof(int32_t);
    VkRuntime::Buffer npBuf = rt.createBuffer(npSize), locBuf = rt.createBuffer(locSize), derBuf = rt.createBuffer(derSize);
    VkRuntime::Buffer vfBuf = rt.createBuffer(vfSize), vkBuf = rt.createBuffer(vkSize), vnBuf = rt.createBuffer(vnSize);
    rt.upload(npBuf, backend.splineNodePack.data(), npSize); rt.upload(locBuf, backend.splineLocs.data(), locSize);
    rt.upload(derBuf, backend.splineDers.data(), derSize); rt.upload(vfBuf, backend.splineValF.data(), vfSize);
    rt.upload(vkBuf, backend.splineValKind.data(), vkSize); rt.upload(vnBuf, backend.splineValNode.data(), vnSize);
    rt.upload(coordBuf, coords.data(), coordSize); rt.upload(permBuf, perm.data(), permSize); rt.upload(splitBuf, splitCoord.data(), splitSize);

    // P2-2：binding 号从生成器取（D19 补全）——wb = {0,1,3,4,5} + splineBindBase..+5
    int wb[11] = {0, 1, 3, 4, 5,
                  backend.splineBindBase + 0, backend.splineBindBase + 1, backend.splineBindBase + 2,
                  backend.splineBindBase + 3, backend.splineBindBase + 4, backend.splineBindBase + 5};
    VkRuntime::Buffer bufs[11] = {coordBuf, permBuf, outBuf, splitBuf, valBuf, npBuf, locBuf, derBuf, vfBuf, vkBuf, vnBuf};
    VkDeviceSize sizes[11] = {coordSize, permSize, outSize, splitSize, valSize, npSize, locSize, derSize, vfSize, vkSize, vnSize};
    VkDescriptorSet ds = rt.makeDescriptorSet<11>(bufs, wb, sizes, 11);
    rt.dispatch(ds, N);

    { std::vector<float> out(N); rt.readback(outBuf, out.data(), outSize);
        double maxDiff = 0.0, sumDiff = 0.0;
        { std::ofstream f("out_dump.txt"); for (uint32_t i = 0; i < N; i++) f << out[i] << "\n"; }
        struct DiffRec { uint32_t i; float gpu; double ref; double diff; };
        std::vector<DiffRec> top;
        for (uint32_t i = 0; i < N; i++) {
            wg::NoisePos pos{ coords[3*i+0], coords[3*i+1], coords[3*i+2] };
            double ref = fdDF->sample(pos);
            double diff = std::fabs((double)out[i] - ref);
            if (i < 16 || i % 128 == 0) std::printf("[DBG] i=%u pos=(%d,%d,%d) gpu=%.9f cpu=%.9f diff=%.3e\n", i, pos.x, pos.y, pos.z, out[i], ref, diff);
            if (diff > maxDiff) maxDiff = diff;
            sumDiff += diff;
            top.push_back({i, out[i], ref, diff});
        }
        std::sort(top.begin(), top.end(), [](const DiffRec& a, const DiffRec& b) { return a.diff > b.diff; });
        for (int k = 0; k < 12 && k < (int)top.size(); k++) {
            wg::NoisePos pos{ coords[3*top[k].i+0], coords[3*top[k].i+1], coords[3*top[k].i+2] };
            std::printf("[TOP%02d] i=%u pos=(%d,%d,%d) gpu=%.9f cpu=%.9f diff=%.3e\n", k, top[k].i, pos.x, pos.y, pos.z, top[k].gpu, top[k].ref, top[k].diff);
        }
        std::printf("[result] N=%u, final_density 完整树: GPU float vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
    }

    std::printf("[done]\n");
    VkRuntime::destroyBuffer(rt.device(), coordBuf); VkRuntime::destroyBuffer(rt.device(), permBuf);
    VkRuntime::destroyBuffer(rt.device(), splitBuf); VkRuntime::destroyBuffer(rt.device(), outBuf); VkRuntime::destroyBuffer(rt.device(), valBuf);
    VkRuntime::destroyBuffer(rt.device(), npBuf); VkRuntime::destroyBuffer(rt.device(), locBuf);
    VkRuntime::destroyBuffer(rt.device(), derBuf); VkRuntime::destroyBuffer(rt.device(), vfBuf);
    VkRuntime::destroyBuffer(rt.device(), vkBuf); VkRuntime::destroyBuffer(rt.device(), vnBuf);
    return 0;
}
