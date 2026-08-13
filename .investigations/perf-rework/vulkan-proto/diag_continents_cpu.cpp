// diag_continents_cpu.cpp —— 纯 CPU 诊断：noise.h 采样 vs 手动复刻 shader 逻辑
#include <cstdio>
#include <cstring>
#include <vector>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "md5.h"

static double maintainPrecision(double v) { return v - (long)(v / 3.3554432E7 + 0.5) * 3.3554432E7; }

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    wg::XoroshiroRandom base(worldSeed);
    auto randomDeriver = base.nextSplitter();
    wg::DoublePerlinNoiseSampler continentalness(randomDeriver.split("minecraft:continentalness"),
        wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1.0,1.0,2.0,2.0,2.0,1.0,1.0,1.0,1.0}});

    // 单坐标
    double x = 30000000.0 * 0.25;
    double y = 64.0 * 0.0;
    double z = 30000001.0 * 0.25;
    double ref = continentalness.sample(x, y, z);
    printf("noise.h sample: %.9f\n", ref);

    // 手动复刻 shader 逻辑（double 拆分 + float 采样）
    const int n = 9;
    const double lacunarity = 512.0;
    const double persistence = 0.50097847358121328;
    const double amplitude = 1.5;
    const double amps[9] = {1,1,2,2,2,1,1,1,1};
    double d = 0.0, e = lacunarity, f = persistence;
    for (int i = 0; i < n; i++) {
        const wg::PerlinNoiseSampler* pn = continentalness.firstSampler.octaveSamplers[i].get();
        double cx = maintainPrecision(x * e);
        double cy = maintainPrecision(y * e);
        double cz = maintainPrecision(z * e);
        double dx = cx + pn->originX, dy = cy + pn->originY, dz = cz + pn->originZ;
        int ix = (int)std::floor(dx), iy = (int)std::floor(dy), iz = (int)std::floor(dz);
        // float 采样
        float gx = (float)(dx - ix), gy = (float)(dy - iy), gz = (float)(dz - iz);
        // 手动 grad + fade + lerp（float）
        static const int GRAD[16][3] = {{1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},{1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},{0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},{1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1}};
        auto map = [&](int v){ return pn->permutation[v & 0xFF]; };
        auto grad = [&](int h, float x2, float y2, float z2){ return (float)GRAD[h&15][0]*x2 + (float)GRAD[h&15][1]*y2 + (float)GRAD[h&15][2]*z2; };
        auto fade = [](float v){ return v*v*v*(v*(v*6-15)+10); };
        auto lerp = [](float dd, float s, float ee){ return s + dd*(ee-s); };
        int m0=map(ix), m1=map(ix+1), m2=map(m0+iy), m3=map(m0+iy+1), m4=map(m1+iy), m5=map(m1+iy+1);
        float g0=grad(map(m2+iz),gx,gy,gz), g1=grad(map(m4+iz),gx-1,gy,gz);
        float g2=grad(map(m3+iz),gx,gy-1,gz), g3=grad(map(m5+iz),gx-1,gy-1,gz);
        float g4=grad(map(m2+iz+1),gx,gy,gz-1), g5=grad(map(m4+iz+1),gx-1,gy,gz-1);
        float g6=grad(map(m3+iz+1),gx,gy-1,gz-1), g7=grad(map(m5+iz+1),gx-1,gy-1,gz-1);
        float r=fade(gx), s=fade(gy), t=fade(gz);
        float x0=lerp(r,g0,g1), x1=lerp(r,g2,g3), x2=lerp(r,g4,g5), x3=lerp(r,g6,g7);
        float y0=lerp(s,x0,x1), y1=lerp(s,x2,x3);
        float ns = lerp(t,y0,y1);
        d += amps[i] * ns * f;
        e *= 2.0; f /= 2.0;
    }
    // second sampler
    double d2 = 0.0; e = lacunarity; f = persistence;
    double x2s = x * 1.0181268882175227, y2s = y * 1.0181268882175227, z2s = z * 1.0181268882175227;
    for (int i = 0; i < n; i++) {
        const wg::PerlinNoiseSampler* pn = continentalness.secondSampler.octaveSamplers[i].get();
        double cx = maintainPrecision(x2s * e), cy = maintainPrecision(y2s * e), cz = maintainPrecision(z2s * e);
        double dx = cx + pn->originX, dy = cy + pn->originY, dz = cz + pn->originZ;
        int ix = (int)std::floor(dx), iy = (int)std::floor(dy), iz = (int)std::floor(dz);
        float gx = (float)(dx-ix), gy = (float)(dy-iy), gz = (float)(dz-iz);
        static const int GRAD[16][3] = {{1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},{1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},{0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},{1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1}};
        auto map = [&](int v){ return pn->permutation[v & 0xFF]; };
        auto grad = [&](int h, float xa, float ya, float za){ return (float)GRAD[h&15][0]*xa + (float)GRAD[h&15][1]*ya + (float)GRAD[h&15][2]*za; };
        auto fade = [](float v){ return v*v*v*(v*(v*6-15)+10); };
        auto lerp = [](float dd, float s, float ee){ return s + dd*(ee-s); };
        int m0=map(ix), m1=map(ix+1), m2=map(m0+iy), m3=map(m0+iy+1), m4=map(m1+iy), m5=map(m1+iy+1);
        float g0=grad(map(m2+iz),gx,gy,gz), g1=grad(map(m4+iz),gx-1,gy,gz);
        float g2=grad(map(m3+iz),gx,gy-1,gz), g3=grad(map(m5+iz),gx-1,gy-1,gz);
        float g4=grad(map(m2+iz+1),gx,gy,gz-1), g5=grad(map(m4+iz+1),gx-1,gy,gz-1);
        float g6=grad(map(m3+iz+1),gx,gy-1,gz-1), g7=grad(map(m5+iz+1),gx-1,gy-1,gz-1);
        float r=fade(gx), s=fade(gy), t=fade(gz);
        float x0=lerp(r,g0,g1), x1=lerp(r,g2,g3), x2=lerp(r,g4,g5), x3=lerp(r,g6,g7);
        float y0=lerp(s,x0,x1), y1=lerp(s,x2,x3);
        float ns = lerp(t,y0,y1);
        d2 += amps[i] * ns * f;
        e *= 2.0; f /= 2.0;
    }
    double manual = (d + d2) * amplitude;
    printf("手动复刻(float): %.9f  差 %.3e\n", manual, std::fabs(manual - ref));
    return 0;
}
