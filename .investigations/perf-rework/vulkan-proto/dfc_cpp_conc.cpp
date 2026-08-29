// dfc_cpp_conc.cpp — 并发探针：DFC C++ (CpuBackend::sample, thread_local grid) 多线程采样，测并发放大
// 对标 conc_density_probe（production 的 density 11×）。DFC 是否能显著降低并发放大？
// 用法：dfc_cpp_conc <seed> <worldgen dir> <threads> <Npoints/thread>
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cmath>
#include <thread>
#include <vector>
#include <chrono>
#include "E:/PYTHON/CoreSwap/.investigations/perf-rework/cpu_backend.h"

static double nowMs() {
    using namespace std::chrono;
    return duration<double, std::milli>(steady_clock::now().time_since_epoch()).count();
}

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    if (argc < 4) { std::fprintf(stderr, "usage: dfc_cpp_conc <seed> <worldgen dir> <threads> [N=20000]\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    int threads = std::atoi(argv[3]);
    int N = argc >= 5 ? std::atoi(argv[4]) : 20000;

    // 每个线程独立 CpuBackend（init + collectPerm）——thread_local 内部缓存已独立
    // 但 CpuBackend init 是重操作；线程共享同一 backend（init 一次），thread_local splitCoord/gridSlots 已 isolate。
    // 注意：CpuBackend 的 normals 等是共享只读（init 后只读），splitCoord/gridSlots 是 thread_local → 并发采样安全。
    CpuBackend shared;
    shared.init((uint64_t)seed);
    shared.collectPerm(shared.perm);

    std::fprintf(stderr, "[DFC-CONC] seed=%lld threads=%d N=%d/thread\n", (long long)seed, threads, N);
    auto t0 = nowMs();
    std::vector<std::thread> ts;
    std::vector<double> sums(threads, 0.0);
    for (int t = 0; t < threads; t++) {
        ts.emplace_back([&, t] {
            double s = 0;
            // 每线程在一个固定 chunk 区域内采样（grid 缓存命中，模拟 production 的 density 阶段在 chunk 内）
            // 每线程用不同 chunk（t 偏移），thread_local grid 缓存独立
            int chunkX = t * 2, chunkZ = t * 3;
            for (int i = 0; i < N; i++) {
                int x = chunkX * 16 + (i * 3) % 16;   // 一个 chunk 内 x
                int y = -64 + (i * 7) % 384;
                int z = chunkZ * 16 + (i * 5) % 16;   // 一个 chunk 内 z
                s += shared.sample(x, y, z);
            }
            sums[t] = s;
        });
    }
    for (auto& th : ts) th.join();
    double t1 = nowMs();
    double total = 0; for (double v : sums) total += v;
    double ms = t1 - t0;
    std::fprintf(stderr, "[DFC-CONC] T=%d wall=%.1fms  per-sample=%.1fns  acc=%.2f  (usec/sample = %.2f)\n",
                 threads, ms, 1e6 * ms / ((double)threads * N), total, ms * 1000.0 / ((double)threads * N));
    return 0;
}
