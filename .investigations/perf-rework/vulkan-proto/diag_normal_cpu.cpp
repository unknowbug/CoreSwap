// diag_normal_cpu.cpp —— 对比 noise.h DoublePerlinNoiseSampler.sample vs 手动复刻 GPU 采样（double 拆分 + float）
#include <cstdio>
#include <cstring>
#include <vector>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "md5.h"

static double maintainPrecision(double v) { return v - (long)(v / 3.3554432E7 + 0.5) * 3.3554432E7; }
static const int GRAD[16][3] = {{1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},{1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},{0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},{1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1}};

// 手动复刻 GPU 采样：double 拆分 + float，用 perm/origin（garbage 时 perm=0/origin=0）
static float pn_sample_f32(const wg::PerlinNoiseSampler* pn, int ix, int iy, int iz, float gx, float gy, float gz) {
    auto map = [&](int v) { return pn ? (int)pn->permutation[v & 0xFF] : 0; };
    auto grad = [&](int h, float x, float y, float z) { return (float)GRAD[h&15][0]*x + (float)GRAD[h&15][1]*y + (float)GRAD[h&15][2]*z; };
    auto fade = [](float v) { return v*v*v*(v*(v*6-15)+10); };
    auto lerp = [](float d, float s, float e) { return s + d*(e-s); };
    int m0=map(ix), m1=map(ix+1), m2=map(m0+iy), m3=map(m0+iy+1), m4=map(m1+iy), m5=map(m1+iy+1);
    float d0=grad(map(m2+iz),gx,gy,gz), d1=grad(map(m4+iz),gx-1,gy,gz);
    float d2=grad(map(m3+iz),gx,gy-1,gz), d3=grad(map(m5+iz),gx-1,gy-1,gz);
    float d4=grad(map(m2+iz+1),gx,gy,gz-1), d5=grad(map(m4+iz+1),gx-1,gy,gz-1);
    float d6=grad(map(m3+iz+1),gx,gy-1,gz-1), d7=grad(map(m5+iz+1),gx-1,gy-1,gz-1);
    float r=fade(gx), s=fade(gy), t=fade(gz);
    float x0=lerp(r,d0,d1), x1=lerp(r,d2,d3), x2=lerp(r,d4,d5), x3=lerp(r,d6,d7);
    float y0=lerp(s,x0,x1), y1=lerp(s,x2,x3);
    return lerp(t,y0,y1);
}

static double gpu_normal(const wg::DoublePerlinNoiseSampler& dn, double dx, double dy, double dz) {
    int n = (int)dn.firstSampler.octaveSamplers.size();
    double lacunarity = std::pow(2.0, dn.firstSampler.firstOctave);
    double persistence = std::pow(2.0, n-1) / (std::pow(2.0, n) - 1.0);
    int jj=INT_MAX, kk=INT_MIN;
    for (int l=0; l<n; l++) if (dn.firstSampler.amplitudes[l]!=0.0) { jj=std::min(jj,l); kk=std::max(kk,l); }
    double createAmp = 0.1 * (1.0 + 1.0/(kk-jj+1));
    double amplitude = 0.16666666666666666 / createAmp;
    double d = 0.0, e = lacunarity, f = persistence;
    for (int i = 0; i < n; i++) {
        const wg::PerlinNoiseSampler* pn = dn.firstSampler.octaveSamplers[i].get();
        double cx = maintainPrecision(dx*e), cy = maintainPrecision(dy*e), cz = maintainPrecision(dz*e);
        double ox = pn ? pn->originX : 0.0, oy = pn ? pn->originY : 0.0, oz = pn ? pn->originZ : 0.0;
        int ix = (int)std::floor(cx+ox), iy = (int)std::floor(cy+oy), iz = (int)std::floor(cz+oz);
        float gx = (float)(cx+ox-ix), gy = (float)(cy+oy-iy), gz = (float)(cz+oz-iz);
        float ns = pn_sample_f32(pn, ix, iy, iz, gx, gy, gz);
        d += dn.firstSampler.amplitudes[i] * (double)ns * f;
        e *= 2.0; f /= 2.0;
    }
    double d2 = 0.0; e = lacunarity; f = persistence;
    for (int i = 0; i < n; i++) {
        const wg::PerlinNoiseSampler* pn = dn.secondSampler.octaveSamplers[i].get();
        double cx = maintainPrecision(dx*1.0181268882175227*e), cy = maintainPrecision(dy*1.0181268882175227*e), cz = maintainPrecision(dz*1.0181268882175227*e);
        double ox = pn ? pn->originX : 0.0, oy = pn ? pn->originY : 0.0, oz = pn ? pn->originZ : 0.0;
        int ix = (int)std::floor(cx+ox), iy = (int)std::floor(cy+oy), iz = (int)std::floor(cz+oz);
        float gx = (float)(cx+ox-ix), gy = (float)(cy+oy-iy), gz = (float)(cz+oz-iz);
        float ns = pn_sample_f32(pn, ix, iy, iz, gx, gy, gz);
        d2 += dn.secondSampler.amplitudes[i] * (double)ns * f;
        e *= 2.0; f /= 2.0;
    }
    return (d + d2) * amplitude;
}

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    wg::XoroshiroRandom base(worldSeed);
    auto rd = base.nextSplitter();
    wg::DoublePerlinNoiseSampler erosion(rd.split("minecraft:erosion"), wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1.0,1.0,0.0,1.0,1.0}});
    wg::DoublePerlinNoiseSampler offset(rd.split("minecraft:offset"), wg::DoublePerlinNoiseSampler::NoiseParameters{-3, {1.0,1.0,1.0,0.0}});

    // offset.sample(182, 0, -107)
    double ref_offset = offset.sample(182.0, 0.0, -107.0);
    double gpu_offset = gpu_normal(offset, 182.0, 0.0, -107.0);
    printf("offset(182,0,-107): noise.h=%.9f gpu复刻=%.9f diff=%.3e\n", ref_offset, gpu_offset, fabs(ref_offset-gpu_offset));

    // erosion.sample(180.5362, 0, -105.0575)
    double ref_erosion = erosion.sample(180.536177101, 0.0, -105.057457121);
    double gpu_erosion = gpu_normal(erosion, 180.536177101, 0.0, -105.057457121);
    printf("erosion(180.54,0,-105.06): noise.h=%.9f gpu复刻=%.9f diff=%.3e\n", ref_erosion, gpu_erosion, fabs(ref_erosion-gpu_erosion));
    return 0;
}
