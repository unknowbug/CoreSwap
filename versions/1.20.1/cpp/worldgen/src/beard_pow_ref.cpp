// beard_pow_ref.cpp — 生成 Beardifier 权重表参照（std::pow），供 Rust powf 逐位对拍。
// 复刻 C++ beardifier.h calculateStructureWeight：pow(2.718281828459045, -d/16.0)，float 截断。
#include <cstdio>
#include <cmath>

int main() {
    const int pts[][3] = {
        {0,0,0}, {1,0,0}, {0,1,0}, {3,2,1}, {5,5,5}, {11,11,11}
    };
    printf("C++ std::pow reference (calculateStructureWeight, f32):\n");
    for (auto& p : pts) {
        int x = p[0], y = p[1], z = p[2];
        double dy = (double)y + 0.5;
        double d = (double)(x*x) + dy*dy + (double)(z*z);
        double v = std::pow(2.718281828459045, -d / 16.0);
        float f = (float)v;
        printf("  (%d,%d,%d) d=%.4f weight=%.9f (f32)\n", x, y, z, d, f);
    }
    return 0;
}
