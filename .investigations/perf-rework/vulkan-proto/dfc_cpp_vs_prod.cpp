// dfc_cpp_vs_prod.cpp — Phase 3: DFC C++ (CpuBackend::sample) vs production (finalDensity->sample)
// 借助 worldgen_api: wg_create(seed) + wg_sample_density(production) ；DFC 用 CpuBackend::sample。
// 遍历同区域坐标网格，逐点比对，输出 maxdiff。
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cmath>
#include <vector>
#include "worldgen_api.h"
#include "E:/PYTHON/CoreSwap/.investigations/perf-rework/cpu_backend.h"

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    if (argc < 2) { std::fprintf(stderr, "usage: dfc_cpp_vs_prod <seed> [worldgen dir]\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    const char* wgDir = argc >= 3 ? argv[2] : "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen";

    // production 路径：wg_create + wg_sample_density（完整 finalDensity，含 InterpolatedDF）
    void* h = wg_create(seed, wgDir, "overworld.json", "biome_params.json", 0);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // DFC 路径：CpuBackend::init + collectPerm + sample
    CpuBackend backend;
    backend.init((uint64_t)seed);
    backend.collectPerm(backend.perm);

    std::fprintf(stderr, "[PHASE3] seed=%lld  DFC C++ vs production finalDensity\n", (long long)seed);
    double maxdiff = 0.0; int maxn = 0;
    long long n = 0, cnt = 0;
    // 单 chunk 域（x,z∈[0,16)，y 覆盖多 cell）——chunk 内 production grid 缓存生效，采样高效
    for (int y = -64; y < -16; y += 4) {
        for (int z = 0; z < 16; z += 2) {
            for (int x = 0; x < 16; x += 2) {
                double prod = wg_sample_density(h, x, y, z);
                float dfc = backend.sample(x, y, z);
                double d = std::fabs(prod - (double)dfc);
                if (d > maxdiff) { maxdiff = d; maxn = (int)n; }
                n++;
                if (d > 1e-5) cnt++;
            }
        }
    }
    std::fprintf(stderr, "[PHASE3] n=%lld  >1e-5:%lld  maxdiff=%.6e  @n=%d  %s\n",
                 n, cnt, maxdiff, maxn, maxdiff < 1e-4 ? "PASS(<1e-4)" : "FAIL(>1e-4)");
    wg_destroy(h);
    return maxdiff < 1e-4 ? 0 : 1;
}
