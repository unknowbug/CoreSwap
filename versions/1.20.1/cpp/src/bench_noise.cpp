#include <cstdio>
#include <chrono>
#include "noise.h"
using namespace wg;
int main() {
    XoroshiroRandom base(12345);
    auto rd = base.nextSplitter();
    DoublePerlinNoiseSampler::NoiseParameters p{-9, {1.0,1.0,2.0,2.0,2.0,1.0,1.0,1.0,1.0}};
    auto r = rd.split("minecraft:continentalness");
    DoublePerlinNoiseSampler n(r, p);
    // ??
    volatile double acc = 0;
    for (int i = 0; i < 100000; i++) acc += n.sample(i*0.5, 100.0, i*0.25);
    auto t0 = std::chrono::steady_clock::now();
    const int N = 20000000;
    for (int i = 0; i < N; i++) acc += n.sample(i*0.5, 100.0, i*0.25);
    auto t1 = std::chrono::steady_clock::now();
    double ms = std::chrono::duration<double, std::milli>(t1-t0).count();
    printf("C++ DoublePerlin sample: %d calls in %.1f ms = %.2f ns/call (acc=%.3f)\n", N, ms, ms*1e6/N, acc);
    return 0;
}
