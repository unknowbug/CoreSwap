// mlp_probe.cpp — 软流（K 路交错）vs 顺序，单线程吞吐验证（MLP 假说）
// 目的：模拟 production 的「访存依赖链」（每级读内存 -> 算 -> 下一级，L 层），
//       对比【顺序】每点算完一个链 vs【软流】K 个点的链交错（K 点独立 load 交叠）。
//       若软流显著快 -> 「打断依赖链提升 MLP」成立（值得做 production 完整 MLP）。
// 用法：mlp_probe <N> <mode=1顺序 2软流4 3软流8> <cacheUnroll>
#include <cstdio>
#include <cstdlib>
#include <vector>
#include <chrono>
#include <cstdint>

static const int L = 15;   // 依赖链层数（贴近 production top 链 ~15-20）
static volatile int g_sink;
static double wall() { using namespace std::chrono; return duration<double, std::milli>(steady_clock::now().time_since_epoch()).count(); }

int main(int argc, char** argv) {
    int N = argc >= 2 ? atoi(argv[1]) : 200000;
    int mode = argc >= 3 ? atoi(argv[2]) : 2;   // 1=seq; 2=soft4; 3=soft8
    const size_t SZ = 32u * 1024 * 1024;         // 32MB 数组（超 L3 16.5MB，模拟 production 真实 DRAM 访存 miss）
    std::vector<double> a(SZ); for (size_t i = 0; i < SZ; i++) a[i] = i * 0.001;
    std::vector<double> b(SZ); for (size_t i = 0; i < SZ; i++) b[i] = (i * 3) % 103 * 0.01;
    // 伪随机索引（cache miss，贴近生产访存延迟放大）
    std::vector<uint32_t> idx(SZ); for (size_t i = 0; i < SZ; i++) idx[i] = (uint32_t)((i * 2654435761u) % SZ);
    double t0 = wall();
    double acc = 0;
    if (mode == 1) {
        for (int i = 0; i < N; i++) {
            size_t base = (size_t)i;
            double d = a[idx[base % SZ]];
            for (int l = 0; l < L; l++) { double x = b[idx[(base + l * 17) % SZ]]; d = d + x * 0.5 - 1.1; d = d * 0.999 + 0.001; }
            acc += d;
        }
    } else {
        int K = (mode == 2) ? 4 : (mode == 3 ? 8 : 16);
        double d[16]; size_t base[16];   // K max=16（不用 VLA，MSVC 不支持）
        for (int i = 0; i < N; i += K) {
            for (int k = 0; k < K; k++) { base[k] = (size_t)(i + k); d[k] = a[idx[base[k] % SZ]]; }
            // 软流：K 点的同一层交错，使 K 个独立 load 在飞行
            for (int l = 0; l < L; l++) {
                for (int k = 0; k < K; k++) {
                    double x = b[idx[(base[k] + l * 17) % SZ]];
                    d[k] = d[k] + x * 0.5 - 1.1;
                    d[k] = d[k] * 0.999 + 0.001;
                }
            }
            for (int k = 0; k < K; k++) acc += d[k];
        }
    }
    double t1 = wall();
    if (acc == -1.0) g_sink = (int)acc;   // 防优化
    double per = 1e6 * (t1 - t0) / (double)N;
    std::fprintf(stderr, "[MLP] mode=%s N=%d L=%d K=%d wall=%.1fms per-point=%.2fus\n",
                 mode == 1 ? "seq" : (mode == 2 ? "soft4" : (mode == 3 ? "soft8" : "soft16")), N, L, (mode == 1 ? 1 : (mode == 2 ? 4 : (mode == 3 ? 8 : 16))), t1 - t0, per);
    return 0;
}
