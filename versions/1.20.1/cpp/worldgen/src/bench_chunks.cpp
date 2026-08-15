// bench_chunks.cpp — C++ 世界生成吞吐基准（SURFACE 模式 = 实机 populateNoise 覆盖路径）
// 用法: bench_chunks <seed> <worldgen dir> [chunksPerSide=16] [reps=2] [originCX=0] [originCZ=0]
//   默认 16×16=256 chunks，2 轮取中位；origin 为区域中心 chunk 坐标
// 输出:
//   [A] 池并行批提交: wg_fill_blocks_multi(count=N, threads=T) 吞吐
//   [B] 模拟实机 JNI: T 个 worker 线程各自 wg_fill_blocks_multi(count=1)（M=1 非空即处理）
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <chrono>
#include <thread>
#include <vector>
#include <algorithm>
#include <atomic>

#include "worldgen_api.h"
#include "crash_handler.h"

static double nowMs() {
    using namespace std::chrono;
    return duration<double, std::milli>(steady_clock::now().time_since_epoch()).count();
}

// C2 频率归一化（2026-08-15）：rdtsc 标定实际运行频率（cycles/s）
// WG_CLOCKTRACE=1 时启用：每批开始/结束读 rdtsc + QPC，得实际 GHz，ms/chunk ÷ GHz 归一化
#ifdef _MSC_VER
#include <intrin.h>
static uint64_t rdtscNow() { return __rdtsc(); }
#else
static uint64_t rdtscNow() { uint32_t lo, hi; __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi)); return ((uint64_t)hi << 32) | lo; }
#endif

static double measureGhz() {
    // 忙等 ~10ms 标定 cycles/s（rdtsc 周期数 / QPC 墙钟）
    auto t0 = std::chrono::steady_clock::now();
    uint64_t r0 = rdtscNow();
    volatile uint64_t sink = 0;
    auto t1 = t0;
    while (std::chrono::duration<double, std::milli>(t1 - t0).count() < 10.0) {
        sink += rdtscNow() & 1;  // 消耗 rdtsc，防优化
        t1 = std::chrono::steady_clock::now();
    }
    uint64_t r1 = rdtscNow();
    double secs = std::chrono::duration<double>(t1 - t0).count();
    (void)sink;
    return (double)(r1 - r0) / secs / 1e9;
}

static double median(std::vector<double>& v) {
    std::sort(v.begin(), v.end());
    return v[v.size() / 2];
}

int main(int argc, char** argv) {
    wg::installCrashHandler();
    setvbuf(stderr, nullptr, _IONBF, 0);
    if (argc < 3) {
        std::fprintf(stderr, "usage: bench_chunks <seed> <worldgen dir> [chunksPerSide=16] [reps=2] [originCX=0] [originCZ=0]\n");
        return 1;
    }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];
    int side = argc >= 4 ? std::atoi(argv[3]) : 16;
    int reps = argc >= 5 ? std::atoi(argv[4]) : 2;
    int originCX = argc >= 6 ? std::atoi(argv[5]) : 0;
    int originCZ = argc >= 7 ? std::atoi(argv[6]) : 0;
    if (side <= 0 || reps <= 0) { std::fprintf(stderr, "bad args\n"); return 1; }
    int N = side * side;

    void* h = wg_create(seed, wgDir.c_str(), "overworld.json", "biome_params.json", 0);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // chunk 坐标：以 (originCX, originCZ) 为中心 side×side 方块
    std::vector<int> cxs(N), czs(N);
    int half = side / 2;
    for (int i = 0; i < N; i++) {
        cxs[i] = (i % side) - half + originCX;
        czs[i] = (i / side) - half + originCZ;
    }
    const int BUF = 16 * 16 * 384;
    std::vector<std::vector<int32_t>> bufs(N, std::vector<int32_t>(BUF, 0));
    std::vector<int32_t*> outs(N);
    for (int i = 0; i < N; i++) outs[i] = bufs[i].data();

    int maxThreads = (int)std::thread::hardware_concurrency();
    std::fprintf(stderr, "[BENCH] seed=%lld chunks=%d (%dx%d) reps=%d hw_threads=%d\n",
                 (long long)seed, N, side, side, reps, maxThreads);

    // ---- [A] 池并行批提交 ----
    std::vector<int> threadSet;
    // 第 7 参：单线程数（快速诊断）；否则默认 {1,8,12,22}
    if (argc >= 8) {
        threadSet.push_back(std::atoi(argv[7]));
    } else {
        for (int t : {1, 8, 12, 22}) if (t <= maxThreads) threadSet.push_back(t);
    }
    threadSet.push_back(0); // 自适应（physicalCoreCount）
    for (int T : threadSet) {
        // warmup
        wg_fill_blocks_multi(h, cxs.data(), czs.data(), outs.data(), N, T);
        std::vector<double> times;
        bool clockTrace = getenv("WG_CLOCKTRACE") != nullptr;
        double ghz[2] = {0, 0};
        for (int r = 0; r < reps; r++) {
            if (clockTrace) ghz[0] = measureGhz();
            double t0 = nowMs();
            wg_fill_blocks_multi(h, cxs.data(), czs.data(), outs.data(), N, T);
            double t1 = nowMs();
            if (clockTrace) ghz[1] = measureGhz();
            times.push_back(t1 - t0);
        }
        double med = median(times);
        double norm = med / N;
        if (clockTrace) {
            double avgGhz = (ghz[0] + ghz[1]) / 2.0;
            std::fprintf(stderr, "[A] threads=%3d  %6.1f ms/batch  %7.1f chunks/s  (%5.2f ms/chunk, GHz=%.2f/%.2f, norm=%.2f)\n",
                         T, med, N * 1000.0 / med, norm, ghz[0], ghz[1], avgGhz > 0 ? norm / avgGhz : 0.0);
        } else {
            std::fprintf(stderr, "[A] threads=%3d  %6.1f ms/batch  %7.1f chunks/s  (%5.2f ms/chunk)\n",
                         T, med, N * 1000.0 / med, norm);
        }
    }

    // ---- [B] 模拟实机 JNI: T worker 线程各自调 count=1（M=1 非空即处理）----
    std::fprintf(stderr, "\n[B] 模拟实机 JNI（T worker 各调 count=1）\n");
    for (int T : threadSet) {
        if (T == 0) T = maxThreads > 2 ? maxThreads - 2 : 1; // 实机默认 -2 客户端留核
        int Tq = T;
        // warmup：每 worker 各生成 N/T 个（至少 1 轮覆盖全量）
        {
            std::vector<std::thread> ts;
            std::atomic<int> next{0};
            for (int t = 0; t < Tq; t++) {
                ts.emplace_back([&] {
                    for (;;) {
                        int i = next.fetch_add(1);
                        if (i >= N) break;
                        int32_t* out = outs[i];
                        int cxx[1] = {cxs[i]}, czz[1] = {czs[i]};
                        int32_t* out1[1] = {out};
                        wg_fill_blocks_multi(h, cxx, czz, out1, 1, 1);
                    }
                });
            }
            for (auto& th : ts) th.join();
        }
        std::vector<double> times;
        for (int r = 0; r < reps; r++) {
            std::atomic<int> next{0};
            std::vector<std::thread> ts;
            double t0 = nowMs();
            for (int t = 0; t < Tq; t++) {
                ts.emplace_back([&] {
                    for (;;) {
                        int i = next.fetch_add(1);
                        if (i >= N) break;
                        int32_t* out = outs[i];
                        int cxx[1] = {cxs[i]}, czz[1] = {czs[i]};
                        int32_t* out1[1] = {out};
                        wg_fill_blocks_multi(h, cxx, czz, out1, 1, 1);
                    }
                });
            }
            for (auto& th : ts) th.join();
            double t1 = nowMs();
            times.push_back(t1 - t0);
        }
        double med = median(times);
        std::fprintf(stderr, "[B] workers=%3d  %6.1f ms/batch  %7.1f chunks/s  (%5.2f ms/chunk)\n",
                     Tq, med, N * 1000.0 / med, med / N);
    }

    wg_destroy(h);
    wg_profile_dump();  // WG_PROFILE=1 时打印阶段计数/耗时分布
    std::fprintf(stderr, "[BENCH] done\n");
    return 0;
}
