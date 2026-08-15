// gpu_split_probe.cpp —— D23 判别（纯 CPU，免 GPU）：CpuBackend.split 对错点 vs 对点 的拆分对比
// 目的：判别 H3（split 数据本身错）vs H1/H2（GPU kernel 读取/推导错）。
// 方法：对错点 (784,160,-408) 与对点 (784,160,-416)（同 cell cy=28，cz 2 vs 1），
//       dump CpuBackend.split 输出，检查拆分坐标是否合理（无 NaN/越界/模式异常）。
// 若 split 输出正常 → 问题在 GPU kernel 侧（H1/H2）；若 split 异常 → H3。
#include <cstdio>
#include <cstdint>
#include <vector>
#include <fstream>
#include <cmath>
#include "cpu_backend.h"

static void dumpPoint(CpuBackend& backend, int x, int y, int z, const char* tag) {
    std::vector<float> out((size_t)backend.splitTotal);
    backend.split(x, y, z, out.data());
    // 统计：NaN 数、全 0 区段、min/max
    int nanCnt = 0, zeroCnt = 0;
    float mn = 1e30f, mx = -1e30f;
    for (size_t i = 0; i < out.size(); i++) {
        if (std::isnan(out[i])) nanCnt++;
        if (out[i] == 0.0f) zeroCnt++;
        if (out[i] < mn) mn = out[i];
        if (out[i] > mx) mx = out[i];
    }
    std::printf("[%s] (%d,%d,%d) splitTotal=%d nan=%d zero=%d min=%.6f max=%.6f\n",
                tag, x, y, z, backend.splitTotal, nanCnt, zeroCnt, mn, mx);
    // 打印前 24 个拆分值（normals[0] 的 9 octave × 6？实际是 splitDouble 输出 [ix,iy,iz,gx,gy,gz]×n）
    std::printf("  head:");
    for (int i = 0; i < 24 && i < backend.splitTotal; i++) std::printf(" %.4f", out[i]);
    std::printf("\n");
    // D23：打印 3D y 敏感噪声区段（normals[40] base=5984、normals[24] base=1920 是 *0 y 的 2D；
    // 用 base 5984 的 normals[40]（*1 *1 *1 3D）对比 cy 影响；base 1920 normals[24]（*1500 *0 *1500 2D））
    for (int base : {1920, 5984, 6656, 7520, 8192, 8288, 8384, 8480, 8576}) {
        std::printf("  base=%d:", base);
        for (int i = 0; i < 12; i++) std::printf(" %.4f", out[base + i]);
        std::printf("\n");
    }
    // 打印每区段首值（定位 8 角点 × 实例的区段边界）
    std::printf("  seg-starts:");
    for (int seg = 0; seg * 54 < backend.splitTotal && seg < 12; seg++) {
        std::printf(" [%d]=%.4f", seg * 54, out[seg * 54]);
    }
    std::printf("\n");
}

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    CpuBackend backend;
    backend.init(worldSeed);
    std::fprintf(stderr, "[probe] init done, splitTotal=%d\n", backend.splitTotal);
    // D23：dump 指定点 splitCoord（供 Python sim 单点判别）
    int sx = 784, sy = 160, sz = -408;
    std::vector<float> sc((size_t)backend.splitTotal);
    backend.split(sx, sy, sz, sc.data());
    { std::ofstream f("split_single.bin", std::ios::binary); f.write((const char*)sc.data(), sc.size()*4); }
    { std::ofstream f("coords_single.txt"); f << sx << " " << sy << " " << sz << "\n"; }
    std::fprintf(stderr, "[probe] dumped split_single.bin for (%d,%d,%d)\n", sx, sy, sz);
    dumpPoint(backend, 784, 160, -408, "BAD (cz=2)");
    dumpPoint(backend, 784, 160, -416, "OK  (cz=1)");
    dumpPoint(backend, 784, 160, -432, "OK  (cz=0)");
    dumpPoint(backend, 784, -64, -408, "OK  (cy=0)");
    dumpPoint(backend, 0, -64, 0,     "OK  (e2e)");
    dumpPoint(backend, 784, 256, -408, "OK  (y=256)");
    return 0;
}
