// conc_density_probe.cpp — 最小干净并发延迟探针（无 warmup、单轮、固定同批 chunk）
// 用法: conc_density_probe <seed> <worldgen dir> <threads>
// 生成 12 个固定 chunk（同批），用 wg_fill_blocks_multi 跑一次，WG_PHASETICK 抓每 chunk density 延迟。
// 核心：同一批 chunk 分别 T=1 / T=8，对比 single-chunk density 延迟（WG_PHASETICK 干净，无探针污染）。
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <vector>
#include <string>
#include "worldgen_api.h"

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    if (argc < 3) { std::fprintf(stderr, "usage: conc_density_probe <seed> <worldgen dir> [threads=1]\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];
    int threads = argc >= 4 ? std::atoi(argv[3]) : 1;

    void* h = wg_create(seed, wgDir.c_str(), "overworld.json", "biome_params.json", 0);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // 固定同批 12 chunks（起始 chunk 坐标，模拟连续区域）
    const int N = 12;
    int cxs[N], czs[N];
    std::vector<std::vector<int32_t>> bufs(N, std::vector<int32_t>(16*16*384, 0));
    std::vector<int32_t*> outs(N);
    for (int i = 0; i < N; i++) {
        cxs[i] = -6 + (i % 3);      // -6..-4
        czs[i] = -6 + (i / 3);      // -6..-4
        outs[i] = bufs[i].data();
    }
    std::fprintf(stderr, "[PROBE] seed=%lld chunks=%d threads=%d\n", (long long)seed, N, threads);
    std::fprintf(stderr, "[PROBE] WG_PHASETICK 需已设（env）；输出含 [PTICK] density 延迟\n");

    int r = wg_fill_blocks_multi(h, cxs, czs, outs.data(), N, threads);
    std::fprintf(stderr, "[PROBE] wg_fill_blocks_multi returned %d\n", r);

    wg_destroy(h);
    return r == N ? 0 : 1;
}
