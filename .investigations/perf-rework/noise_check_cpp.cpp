// noise_check_cpp.cpp — C++ noise 参照（vs Rust noise_check，同 seed 0 + continentalness -9 + 同坐标）
#include <cstdio>
#include <vector>
#include "noise.h"
int main() {
    wg::XoroshiroRandom random(0);
    wg::DoublePerlinNoiseSampler::NoiseParameters params;
    params.firstOctave = -9;
    params.amplitudes = std::vector<double>{1, 1, 2, 2, 2, 1, 1, 1, 1};
    wg::DoublePerlinNoiseSampler sampler(random, params);
    std::printf("noise(0,0,0) = %.12f\n", sampler.sample(0.0, 0.0, 0.0));
    std::printf("noise(1.5,-2,3.25) = %.12f\n", sampler.sample(1.5, -2.0, 3.25));
    std::printf("noise(100,50,-40) = %.12f\n", sampler.sample(100.0, 50.0, -40.0));
    return 0;
}
