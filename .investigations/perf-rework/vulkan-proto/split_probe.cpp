// split_probe.cpp — 验证 CpuBackend.split(0,-64,0) 的 old_blended 区段（3456..）
#include <cstdio>
#include "cpu_backend.h"
int main() {
    CpuBackend backend;
    backend.init(8576294172403134396ULL);
    std::vector<float> out((size_t)backend.splitTotal, 0.0f);
    backend.split(0, -64, 0, out.data());
    std::printf("splitTotal=%d\n", backend.splitTotal);
    std::printf("out[3450..3475]:");
    for (int i = 3450; i < 3476; i++) std::printf(" %g", out[i]);
    std::printf("\n");
    // 也测角点 y=-56（interp 角点）
    backend.split(0, -56, 0, out.data());
    std::printf("y=-56 out[3456..3464]:");
    for (int i = 3456; i < 3465; i++) std::printf(" %g", out[i]);
    std::printf("\n");
    return 0;
}
