// op_probe.cpp —— FP32 算子库最小验证：单算子（continentalness normal_noise）
// 流程：CpuBackend.split 全量 → 提取目标实例行（12 floats/点）→ GPU op_noise.comp → 读回
// 对比：GPU 输出 vs CpuBackend 直接采样（CPU double 参照）
// 验证：① 带宽账（每点 12 floats vs 8672）② 精度（GPU FP32 vs CPU double）③ 吞吐
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <string>
#include <chrono>
#include "vulkan_runtime.h"
#include "cpu_backend.h"

// 目标实例：continentalness@c0（normal，n=9 octave）
// 从生成器 manifest 取：n=9, octBase, splitBase, persistence, amplitude, amps
// 简化：用 gen_noise_manifest 输出的参数（手动填，验证用）
const int OP_N = 9;
const int OP_ROW = OP_N * 12;  // 单实例 split 行宽

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    const int N = (argc > 1) ? std::atoi(argv[1]) : 100000;  // 默认 10 万点

    // 1. CpuBackend（种子级数据：perm + split）
    CpuBackend backend;
    backend.init(8576294172403134396ULL);
    std::vector<uint32_t> perm;
    backend.collectPerm(perm);
    std::fprintf(stderr, "[op] CpuBackend ready, splitTotal=%d perm=%zu\n", backend.splitTotal, perm.size());

    // 2. 生成 N 个坐标（大坐标域，验证跨域）并全量 split
    std::vector<int32_t> coords((size_t)N * 3);
    for (int i = 0; i < N; i++) {
        coords[(size_t)i*3+0] = 720 + (i % 1000);        // x 大坐标
        coords[(size_t)i*3+1] = -64 + (i % 384);         // y 全高度
        coords[(size_t)i*3+2] = -432 + (i / 1000 % 1000); // z 大坐标
    }
    std::vector<float> fullSplit((size_t)N * backend.splitTotal);
    auto t0 = std::chrono::steady_clock::now();
    for (int i = 0; i < N; i++)
        backend.split(coords[(size_t)i*3+0], coords[(size_t)i*3+1], coords[(size_t)i*3+2],
                      fullSplit.data() + (size_t)i * backend.splitTotal);
    auto t1 = std::chrono::steady_clock::now();
    double splitMs = std::chrono::duration<double, std::milli>(t1 - t0).count();
    std::fprintf(stderr, "[op] split %d 点全量: %.1f ms (%.2f MB)\n", N, splitMs,
                 (double)N * backend.splitTotal * 4 / 1e6);

    // 3. 提取目标实例行（splitBase 从生成器布局取；用 manifest 参数——先取实例 0 的 splitBase）
    //    注意：实例 0 = continentalness@c0，splitBase 需从生成器读。先 dump 前 20 个 splitBase 找规律。
    //    这里简化：用 dbg 工具确认 splitBase，然后硬编码。先打印诊断。
    std::fprintf(stderr, "[op] 提示：实例 splitBase 布局见 gen_noise_manifest；此处用实例 0 行（假设 splitBase=0）\n");
    const int opSplitBase = 0;  // 实例 0 的 splitBase（continentalness@c0）
    std::vector<float> opSplit((size_t)N * OP_ROW);
    for (int i = 0; i < N; i++)
        std::memcpy(opSplit.data() + (size_t)i * OP_ROW,
                    fullSplit.data() + (size_t)i * backend.splitTotal + opSplitBase,
                    OP_ROW * sizeof(float));
    double splitCompact = (double)N * OP_ROW * 4 / 1e6;
    std::fprintf(stderr, "[op] 紧凑 split: %d 点 × %d floats = %.1f MB（vs 全量 %.0f MB，带宽降 %.0fx）\n",
                 N, OP_ROW, splitCompact, (double)N * backend.splitTotal * 4 / 1e6,
                 (double)backend.splitTotal / OP_ROW);

    // 4. CPU 参照：与 GPU 用「同一 split 行」做 double 精度计算（隔离算法正确性）——
    // GPU(FP32) vs CPU(double) 同输入应 1e-7 级差；若差大 = GPU 算法/FP32 问题。
    // 实现：C++ 内联 double 版 pn_sample3（读 perm + split 行）
    auto gradDotD = [](int hash, double x, double y, double z) -> double {
        int h = hash & 15;
        double u = (h < 8) ? x : y;
        double v = (h < 4) ? y : ((h == 12 || h == 14) ? x : z);
        return (((h & 1) == 0) ? u : -u) + (((h & 2) == 0) ? v : -v);
    };
    auto mapPermD = [&](int octBase, int v) -> int {
        return (int)perm[octBase * 256 + (uint32_t)(v & 255)];
    };
    auto pnSampleD = [&](int octBase, int sx, int sy, int sz, double lx, double ly, double lz) -> double {
        int i = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
        int k = mapPermD(octBase, i + sy); int l = mapPermD(octBase, i + sy + 1);
        int m = mapPermD(octBase, j + sy); int nn = mapPermD(octBase, j + sy + 1);
        double d = gradDotD(mapPermD(octBase, k + sz), lx, ly, lz);
        double e = gradDotD(mapPermD(octBase, m + sz), lx - 1.0, ly, lz);
        double f = gradDotD(mapPermD(octBase, l + sz), lx, ly - 1.0, lz);
        double g = gradDotD(mapPermD(octBase, nn + sz), lx - 1.0, ly - 1.0, lz);
        double h = gradDotD(mapPermD(octBase, k + sz + 1), lx, ly, lz - 1.0);
        double o = gradDotD(mapPermD(octBase, m + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
        double p = gradDotD(mapPermD(octBase, l + sz + 1), lx, ly - 1.0, lz - 1.0);
        double q = gradDotD(mapPermD(octBase, nn + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
        double u = lx * lx * lx * (lx * (lx * 6.0 - 15.0) + 10.0);
        double v = ly * ly * ly * (ly * (ly * 6.0 - 15.0) + 10.0);
        double w = lz * lz * lz * (lz * (lz * 6.0 - 15.0) + 10.0);
        double x1 = d + u * (e - d); double y1 = f + u * (g - f);
        double z1 = h + u * (o - h); double z2 = p + u * (q - p);
        double y2 = z1 + v * (z2 - z1); double x2 = x1 + v * (y1 - x1);
        return x2 + w * (y2 - x2);
    };
    std::vector<double> cpuRef(N);
    auto t2 = std::chrono::steady_clock::now();
    const double OP_AMPS[9] = {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0};
    for (int i = 0; i < N; i++) {
        const float* row = opSplit.data() + (size_t)i * OP_ROW;
        double d = 0.0, f = 0.5009784735812133;
        for (int j = 0; j < OP_N; j++) {
            int b = j * 6;
            int ix = (int)row[b+0], iy = (int)row[b+1], iz = (int)row[b+2];
            double gx = row[b+3], gy = row[b+4], gz = row[b+5];
            double ns = pnSampleD(0 + j, ix, iy, iz, gx, gy, gz);
            d += OP_AMPS[j] * ns * f;
            f /= 2.0;
        }
        double d2 = 0.0; f = 0.5009784735812133;
        for (int j = 0; j < OP_N; j++) {
            int b = 6 * OP_N + j * 6;
            int ix = (int)row[b+0], iy = (int)row[b+1], iz = (int)row[b+2];
            double gx = row[b+3], gy = row[b+4], gz = row[b+5];
            double ns = pnSampleD(OP_N + j, ix, iy, iz, gx, gy, gz);
            d2 += OP_AMPS[j] * ns * f;
            f /= 2.0;
        }
        cpuRef[i] = (d + d2) * 1.5;
    }
    auto t3 = std::chrono::steady_clock::now();
    std::fprintf(stderr, "[op] CPU 参照（double 版 op_normal_noise，同 split 行）%d 点: %.1f ms\n", N,
                 std::chrono::duration<double, std::milli>(t3 - t2).count());

    // 5. GPU：op_noise.comp
    VkRuntime rt;
    rt.init();
    // pipeline 从 spv 创建（先 glslc 编译 op_noise.comp）
    rt.createPipeline("op_noise.spv");
    VkRuntime::Buffer coordBuf = rt.createBuffer((VkDeviceSize)N * 3 * sizeof(int32_t));
    VkRuntime::Buffer splitBuf = rt.createBuffer((VkDeviceSize)N * OP_ROW * sizeof(float));
    VkRuntime::Buffer permBuf = rt.createBuffer((VkDeviceSize)perm.size() * sizeof(uint32_t));
    VkRuntime::Buffer outBuf = rt.createBuffer((VkDeviceSize)N * sizeof(float));
    rt.upload(coordBuf, coords.data(), (VkDeviceSize)N * 3 * sizeof(int32_t));
    rt.upload(splitBuf, opSplit.data(), (VkDeviceSize)N * OP_ROW * sizeof(float));
    rt.upload(permBuf, perm.data(), (VkDeviceSize)perm.size() * sizeof(uint32_t));
    VkRuntime::Buffer bufs[4] = {coordBuf, splitBuf, permBuf, outBuf};
    int wb[4] = {0, 1, 2, 3};
    VkDeviceSize sizes[4] = {
        (VkDeviceSize)N * 3 * sizeof(int32_t), (VkDeviceSize)N * OP_ROW * sizeof(float),
        (VkDeviceSize)perm.size() * sizeof(uint32_t), (VkDeviceSize)N * sizeof(float)};
    VkDescriptorSet ds = rt.makeDescriptorSet<4>(bufs, wb, sizes, 4);
    auto t4 = std::chrono::steady_clock::now();
    rt.dispatch(ds, (uint32_t)N);
    std::vector<float> gpuOut(N);
    rt.readback(outBuf, gpuOut.data(), (VkDeviceSize)N * sizeof(float));
    auto t5 = std::chrono::steady_clock::now();
    std::fprintf(stderr, "[op] GPU dispatch+readback %d 点: %.1f ms\n", N,
                 std::chrono::duration<double, std::milli>(t5 - t4).count());

    // 6. 对比：GPU FP32 vs CPU double
    double maxDiff = 0; int maxIdx = -1; double sumDiff = 0;
    for (int i = 0; i < N; i++) {
        double d = std::fabs((double)gpuOut[i] - cpuRef[i]);
        sumDiff += d;
        if (d > maxDiff) { maxDiff = d; maxIdx = i; }
    }
    std::fprintf(stderr, "[op] GPU vs CPU: N=%d maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
    if (maxIdx >= 0)
        std::fprintf(stderr, "  worst @ (%d,%d,%d) gpu=%.6f cpu=%.6f\n",
                     coords[(size_t)maxIdx*3+0], coords[(size_t)maxIdx*3+1], coords[(size_t)maxIdx*3+2],
                     gpuOut[maxIdx], cpuRef[maxIdx]);
    // 带宽账
    std::fprintf(stderr, "[op] 带宽: 全量 %.0f MB vs 紧凑 %.1f MB (%.0fx)\n",
                 (double)N * backend.splitTotal * 4 / 1e6, splitCompact, (double)backend.splitTotal / OP_ROW);
    // 吞吐（GPU 算子）
    std::fprintf(stderr, "[op] 吞吐: GPU %.0f 点/s vs CPU %.0f 点/s\n",
                 N / (std::chrono::duration<double>(t5 - t4).count()),
                 N / (std::chrono::duration<double>(t3 - t2).count()));

    VkDevice dev = rt.device();
    VkRuntime::destroyBuffer(dev, coordBuf); VkRuntime::destroyBuffer(dev, splitBuf);
    VkRuntime::destroyBuffer(dev, permBuf); VkRuntime::destroyBuffer(dev, outBuf);
    return 0;
}
