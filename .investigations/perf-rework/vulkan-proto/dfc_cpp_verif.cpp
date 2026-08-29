// dfc_cpp_verif.cpp — 验证 CpuBackend 新增的 C++ 采样函数（DFC 直排）与 dbg_full_sim.py 蓝本对齐
// 对拍 dbg_full_sim.py 的 4 个参照点（idx=0/128/640/896，坐标 (0,-64/62/54/50,0)）：
//   idx0=0.037482421875  idx128=0.036994792902  idx640=0.040212155346  idx896=0.049567354726
// 流程：CpuBackend::init(seed) + collectPerm(perm) + sample(x,y,z)，比对 eval_density 输出。
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cmath>
#include "E:/PYTHON/CoreSwap/.investigations/perf-rework/cpu_backend.h"

int main(int argc, char** argv) {
    setvbuf(stderr, nullptr, _IONBF, 0);
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    CpuBackend backend;
    backend.init((uint64_t)seed);
    backend.collectPerm(backend.perm);

    // 参照点：{(x,y,z, 期望 sim 值)}（取自 dbg_full_sim.py 输出）
    struct P { int x,y,z; double expect; };
    P pts[] = {
        {0,-64,0, 0.037482421875},
        {0,-62,0, 0.03699479290237214},
        {0,-54,0, 0.04021215534641957},
        {0,-50,0, 0.04956735472606925},
    };
    std::printf("[VERIF] seed=%lld  CpuBackend DFC C++ vs dbg_full_sim.py\n", (long long)seed);
    double maxdiff = 0.0;
    for (auto& p : pts) {
        float v = backend.sample(p.x, p.y, p.z);
        double d = std::fabs((double)v - p.expect);
        if (d > maxdiff) maxdiff = d;
        std::printf("  pos=(%d,%d,%d)  cpp=%.9f  sim(expect)=%.9f  diff=%.3e\n",
                    p.x, p.y, p.z, (double)v, p.expect, d);
    }
    std::printf("[VERIF] maxdiff=%.3e  %s\n", maxdiff, maxdiff < 1e-6 ? "PASS" : "FAIL(>1e-6)");
    return maxdiff < 1e-6 ? 0 : 1;
}
