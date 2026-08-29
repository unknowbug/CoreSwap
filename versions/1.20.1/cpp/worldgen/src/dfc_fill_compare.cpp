// dfc_fill_compare.cpp — 验证 fillOneChunkCore 接入的 WG_DFC_CPU=1 == production
// WG_DFC_CPU=1 时 density 阶段用 dfcBackend->sample，默认用 finalDensity->sample。
// 对比：关(WG_DFC_CPU 不设) vs 开(WG_DFC_CPU=1)，同 seed 同 chunk，整 chunk 输出逐块对比。
// 用法：dfc_fill_compare <seed> <worldgen dir>
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cmath>
#include <vector>
#include <algorithm>
#include "worldgen_api.h"

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    if (argc < 3) { std::fprintf(stderr, "usage: dfc_fill_compare <seed> <worldgen dir>\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    const char* wgDir = argv[2];

    const int N = 2;   // 2 个 chunk（小规模，DFC 每点慢）
    const int BPC = 16*16*384;
    int cxs[N], czs[N];
    for (int i = 0; i < N; i++) { cxs[i] = i*4; czs[i] = i*3; }

    // 关（production）
    _putenv_s("WG_DFC_CPU", "");   // 置空 = 不启用
    void* h1 = wg_create(seed, wgDir, "overworld.json", "biome_params.json", 0);
    std::vector<std::vector<int32_t>> out1(N, std::vector<int32_t>(BPC,0));
    std::vector<int32_t*> o1(N);
    for (int i=0;i<N;i++) o1[i]=out1[i].data();
    wg_fill_blocks_multi(h1, cxs, czs, o1.data(), N, 1);
    wg_destroy(h1);

    // 开（DFC）
    _putenv_s("WG_DFC_CPU", "1");
    void* h2 = wg_create(seed, wgDir, "overworld.json", "biome_params.json", 0);
    std::vector<std::vector<int32_t>> out2(N, std::vector<int32_t>(BPC,0));
    std::vector<int32_t*> o2(N);
    for (int i=0;i<N;i++) o2[i]=out2[i].data();
    wg_fill_blocks_multi(h2, cxs, czs, o2.data(), N, 1);
    wg_destroy(h2);

    // 对比（整 chunk blocks；块 id 应逐块一致，DFC 对齐 9.57e-07 → 块 id 相同）
    long long ndiff = 0; int maxdiff = 0;
    for (int i=0;i<N;i++) for (int j=0;j<BPC;j++) {
        if (out1[i][j] != out2[i][j]) { ndiff++; if (std::abs(out1[i][j]-out2[i][j])>maxdiff) maxdiff=std::abs(out1[i][j]-out2[i][j]); }
    }
    std::fprintf(stderr, "[FILL-CMP] N=%d BPC=%d  diff_blocks=%lld / %d  max_blockdiff=%d  %s\n",
                 N, BPC, ndiff, N*BPC, maxdiff, ndiff==0 ? "PASS(逐块一致)" : "DIFF(见 blockdiff)");
    return ndiff==0 ? 0 : 1;
}
