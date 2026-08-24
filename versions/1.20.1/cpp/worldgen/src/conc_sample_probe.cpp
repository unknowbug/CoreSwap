// conc_sample_probe.cpp — 判定并发延迟根因：共享数据读争用
// 用 wg_sample_density（单点，跳过 cell 缓存）测 T 线程并发采样同一棵 finalDensity 树的延迟扩展。
// 对照：wg_sample_noise 纯噪声采样（无 spline 表）；wg_sample_spline 直接采样单个 SplineDF（绕过 wrapper 链）。
// 若 density（含 spline 树表读）随 T 线性减慢 → 共享树/表读争用；若纯噪声正常 → 定位到 spline 表。
// 对比 density vs spline（同 seed/dir/threads）并发放大 → 隔离 ② wrapper 链（min/interpolated/blend/mul）贡献。
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <string>
#include <thread>
#include <vector>
#include <atomic>
#include <chrono>
#include "worldgen_api.h"

static double nowMs() {
    using namespace std::chrono;
    return duration<double, std::milli>(steady_clock::now().time_since_epoch()).count();
}

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    // usage: conc_sample_probe <seed> <worldgen dir> <threads> <mode=density|noise|spline|interp> [N=20000] [which]
    if (argc < 3) { std::fprintf(stderr, "usage: conc_sample_probe <seed> <worldgen dir> <threads> <mode=|density|noise|spline|interp> [N] [which]\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];
    int threads = std::atoi(argv[3]);
    std::string mode = argc >= 5 ? argv[4] : "density";
    int N = argc >= 6 ? std::atoi(argv[5]) : 20000;
    int which = argc >= 7 ? std::atoi(argv[6]) : 0;

    void* h = wg_create(seed, wgDir.c_str(), "overworld.json", "biome_params.json", 0);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // 诊断：spline 模式打印可用 SplineDF 清单（便于选定 which）
    if (mode == "spline") {
        int sc = wg_spline_count(h);
        std::fprintf(stderr, "[SPLINES] count=%d\n", sc);
        for (int i = 0; i < sc; i++) {
            int nd = wg_spline_nodes(h, i);
            std::fprintf(stderr, "[SPLINES] [%d] nodes=%d %s\n", i, nd,
                         i == which ? "  <-- sampling this" : "");
        }
        std::fprintf(stderr, "[SPLINES] sampling which=%d\n", which);
    }

    std::fprintf(stderr, "[PROBE] mode=%s threads=%d N_total=%d/each\n", mode.c_str(), threads, N);
    auto start = nowMs();
    std::vector<std::thread> ts;
    std::vector<double> sums(threads, 0.0);
    for (int t = 0; t < threads; t++) {
        ts.emplace_back([&, t] {
            double s = 0;
            for (int i = 0; i < N; i++) {
                int x, z, y;
                // 统一「固定同 chunk」访问模式（density/spline/noise 都同 chunk，模拟 production fillOneChunkCore
                // 的 grid 命中）：spline/whole-tree 的 locFn（FlatCacheDF/FinalLocFn）thread_local grid key 只依赖
                // chunkX/chunkZ，y 变化不触发重建。scattered 坐标（跨 128 chunk）会让 InterpolatedDF/FinalLocFn
                // 每换 chunk 重建 grid → per-sample 失真（0.44ms 级，非生产路径）。
                // 固定同 chunk 后 grid 命中 → per-sample 接近生产单点；据此可靠测各入口自身并发放大。
                x = 3200 + (i % 16);
                z = 3224 + ((i / 16) % 16);
                y = -64 + (i * 7) % 384;
                if (mode == "noise") s += wg_sample_noise(h, "minecraft:continentalness", (double)x, 0.0, (double)z);
                else if (mode == "spline") s += wg_sample_spline(h, which, x, y, z);
                else if (mode == "interp") s += wg_sample_interp(h, x, y, z);  // M3：interp#1 grid 命中（预建后，排除长链/怪物树）
                else s += wg_sample_density(h, x, y, z);
            }
            sums[t] = s;
        });
    }
    for (auto& th : ts) th.join();
    double end = nowMs();
    double total = 0; for (double v : sums) total += v;
    std::fprintf(stderr, "[PROBE] mode=%s T=%d  wall=%.1fms  per-sample=%.1fns  acc=%.2f\n",
                 mode.c_str(), threads, end - start, 1000000.0*(end-start)/((double)threads*N), total);

    wg_destroy(h);
    return 0;
}
