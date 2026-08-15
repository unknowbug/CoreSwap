// b3d_probe.cpp — 用参照实现（density.h InterpolatedNoiseDF）采样 base_3d_noise，WG_B3DDUMP 打印
#include <cstdio>
#include <cstdlib>
#include "xoroshiro.h"
#include "noise.h"
#include "density.h"
int main() {
    _putenv_s("WG_B3DDUMP", "1");
    wg::XoroshiroRandom rng(8576294172403134396ULL);
    wg::XoroshiroRandom r = rng.nextSplitter().split(std::string("minecraft:terrain"));
    wg::InterpolatedNoiseDF ob(r, 0.25, 0.125, 80.0, 160.0, 8.0);
    wg::NoisePos p;
    for (int y = -64; y <= -48; y += 8) {
        p.x = 0; p.y = y; p.z = 0;
        double v = ob.sample(p);
        std::printf("[b3d] y=%d value=%.17g\n", y, v);
    }
    return 0;
}
