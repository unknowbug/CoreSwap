// named_ref_probe.cpp —— D23 参照对比：wg_sample_named 采样分量 vs sim 噪声
// 对 (784,160,-408)（错点）与 (784,160,-416)（对点），采样 final_density + 各分量。
#include <cstdio>
#include <cstdlib>
#include <string>
#include "worldgen_api.h"

int main() {
    setvbuf(stderr, nullptr, _IONBF, 0);
    const int64_t seed = 8576294172403134396LL;
    void* h = wg_create(seed, "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen");
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }
    const char* comps[] = {"final_density", "continentalness", "erosion", "ridges", "sloped_cheese", "factor", "depth"};
    int pts[][3] = {{784,160,-408}, {784,160,-416}, {784,-64,-408}, {0,-64,0}};
    for (auto& p : pts) {
        std::printf("# point (%d,%d,%d)\n", p[0], p[1], p[2]);
        for (const char* c : comps) {
            double v = wg_sample_named(h, c, p[0], p[1], p[2]);
            std::printf("  %-16s = %.9f\n", c, v);
        }
    }
    wg_destroy(h);
    std::printf("[done]\n");
    return 0;
}
