// dump_splits.cpp —— 用 CpuBackend 生成 splitCoord/perm/coords dump（免 GPU/免 shader 编译，供 Python 模拟）
#include <cstdio>
#include <cstdint>
#include <vector>
#include <fstream>
#include "cpu_backend.h"

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    CpuBackend backend;
    backend.init(worldSeed);
    std::fprintf(stderr, "[step] init done, splitTotal=%d permSize=%d\n", backend.splitTotal, backend.permSize);

    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = 0 + (i % 64);
        coords[3*i+1] = -64 + (i / 64 % 16);
        coords[3*i+2] = 0 + (i / 1024);
    }
    std::vector<float> splitCoord((size_t)backend.splitTotal * N);
    for (uint32_t s = 0; s < N; s++) {
        backend.split(coords[3*s+0], coords[3*s+1], coords[3*s+2], splitCoord.data() + s * backend.splitTotal);
    }
    std::vector<uint32_t> perm;
    backend.collectPerm(perm);
    { std::ofstream f("split_dump.bin", std::ios::binary); f.write((const char*)splitCoord.data(), splitCoord.size()*4); }
    { std::ofstream f("perm_dump.bin", std::ios::binary); f.write((const char*)perm.data(), perm.size()*4); }
    { std::ofstream f("coords_dump.txt"); for (uint32_t i = 0; i < N; i++) f << coords[3*i+0] << " " << coords[3*i+1] << " " << coords[3*i+2] << "\n"; }
    std::fprintf(stderr, "[step] dumped splitCoord(%zu) perm(%zu) coords(%u)\n", splitCoord.size(), perm.size(), N);
    return 0;
}
