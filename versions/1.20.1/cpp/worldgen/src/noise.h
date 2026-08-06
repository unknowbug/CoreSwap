#pragma once
#include <cstdint>
#include <cmath>
#include <vector>
#include <memory>
#include "xoroshiro.h"

namespace wg {

// MathHelper.floor / lfloor / lerp / perlinFade
inline int32_t floorD(double v) { int32_t i = (int32_t)v; return v < i ? i - 1 : i; }
inline int64_t lfloor(double v) { int64_t l = (int64_t)v; return v < l ? l - 1 : l; }
inline double lerp(double d, double s, double e) { return s + d * (e - s); }
inline double perlinFade(double v) { return v * v * v * (v * (v * 6.0 - 15.0) + 10.0); }

// SimplexNoiseSampler.GRADIENTS（MC 1.20.1 的 16 个梯度）
static constexpr int32_t GRADIENTS[16][3] = {
    {1, 1, 0}, {-1, 1, 0}, {1, -1, 0}, {-1, -1, 0},
    {1, 0, 1}, {-1, 0, 1}, {1, 0, -1}, {-1, 0, -1},
    {0, 1, 1}, {0, -1, 1}, {0, 1, -1}, {0, -1, -1},
    {1, 1, 0}, {0, -1, 1}, {-1, 1, 0}, {0, -1, -1},
};

inline double dot3(const int32_t* g, double x, double y, double z) {
    return g[0] * x + g[1] * y + g[2] * z;
}

// PerlinNoiseSampler（= ImprovedNoise，MC 1.20.1）
class PerlinNoiseSampler {
public:
    double originX, originY, originZ;
    std::vector<uint8_t> permutation; // 256

    explicit PerlinNoiseSampler(XoroshiroRandom& random) {
        originX = random.nextDouble() * 256.0;
        originY = random.nextDouble() * 256.0;
        originZ = random.nextDouble() * 256.0;
        permutation.resize(256);
        for (int i = 0; i < 256; i++) permutation[i] = (uint8_t)i;
        for (int i = 0; i < 256; i++) {
            int j = random.nextInt(256 - i);
            uint8_t b = permutation[i];
            permutation[i] = permutation[i + j];
            permutation[i + j] = b;
        }
    }

    int32_t map(int32_t input) const { return permutation[input & 0xFF] & 0xFF; }

    double sample(double x, double y, double z) const {
        return sample(x, y, z, 0.0, 0.0);
    }

    double sample(double x, double y, double z, double yScale, double yMax) const {
        double d = x + originX;
        double e = y + originY;
        double f = z + originZ;
        int32_t i = floorD(d);
        int32_t j = floorD(e);
        int32_t k = floorD(f);
        double g = d - i;
        double h = e - j;
        double l = f - k;
        double n;
        if (yScale != 0.0) {
            double m = (yMax >= 0.0 && yMax < h) ? yMax : h;
            n = floorD(m / yScale + 1.0E-7F) * yScale;
        } else {
            n = 0.0;
        }
        return sampleSection(i, j, k, g, h - n, l, h);
    }

    static double grad(int32_t hash, double x, double y, double z) {
        return dot3(GRADIENTS[hash & 15], x, y, z);
    }

private:
    double sampleSection(int32_t sx, int32_t sy, int32_t sz,
                         double lx, double ly, double lz, double fadeY) const {
        int32_t i = map(sx);
        int32_t j = map(sx + 1);
        int32_t k = map(i + sy);
        int32_t l = map(i + sy + 1);
        int32_t m = map(j + sy);
        int32_t n = map(j + sy + 1);
        double d = grad(map(k + sz), lx, ly, lz);
        double e = grad(map(m + sz), lx - 1.0, ly, lz);
        double f = grad(map(l + sz), lx, ly - 1.0, lz);
        double g = grad(map(n + sz), lx - 1.0, ly - 1.0, lz);
        double h = grad(map(k + sz + 1), lx, ly, lz - 1.0);
        double o = grad(map(m + sz + 1), lx - 1.0, ly, lz - 1.0);
        double p = grad(map(l + sz + 1), lx, ly - 1.0, lz - 1.0);
        double q = grad(map(n + sz + 1), lx - 1.0, ly - 1.0, lz - 1.0);
        double r = perlinFade(lx);
        double s = perlinFade(fadeY);
        double t = perlinFade(lz);
        // lerp3
        double v000 = d, v100 = e, v010 = f, v110 = g, v001 = h, v101 = o, v011 = p, v111 = q;
        double x0 = lerp(r, v000, v100);
        double x1 = lerp(r, v010, v110);
        double x2 = lerp(r, v001, v101);
        double x3 = lerp(r, v011, v111);
        double y0 = lerp(s, x0, x1);
        double y1 = lerp(s, x2, x3);
        return lerp(t, y0, y1);
    }
};

// OctavePerlinNoiseSampler（MC 1.20.1 modern 版）
class OctavePerlinNoiseSampler {
public:
    std::vector<std::unique_ptr<PerlinNoiseSampler>> octaveSamplers;
    int32_t firstOctave;
    std::vector<double> amplitudes;
    double persistence;
    double lacunarity;
    double maxValue = 0;

    static double maintainPrecision(double v) {
        // Java 1.20.1 OctavePerlinNoiseSampler.maintainPrecision: (long)(v / 3.3554432E7) 向零截断
        // （曾写成 lfloor(v/3.35e7+0.5) 四舍五入——主世界折叠值小数<0.5 未暴露；下界 e*2^12≈2.5e7 小数 0.75 触发分叉）
        long l = (long)(v / 3.3554432E7);
        return v - (double)l * 3.3554432E7;
    }

    // 默认构造（std::map operator[] 用；调用前必须赋值）
    OctavePerlinNoiseSampler() : firstOctave(0) {}

    OctavePerlinNoiseSampler(XoroshiroRandom& random, int32_t firstOctave, const std::vector<double>& amplitudes)
        : firstOctave(firstOctave), amplitudes(amplitudes) {
        size_t i = amplitudes.size();
        int32_t j = -firstOctave;
        octaveSamplers.resize(i);
        auto splitter = random.nextSplitter();
        for (size_t k = 0; k < i; k++) {
            if (amplitudes[k] != 0.0) {
                int32_t l = firstOctave + (int32_t)k;
                XoroshiroRandom rnd = splitter.split("octave_" + std::to_string(l));
                octaveSamplers[k] = std::make_unique<PerlinNoiseSampler>(rnd);
            }
        }
        lacunarity = std::pow(2.0, -j);
        persistence = std::pow(2.0, (double)i - 1.0) / (std::pow(2.0, (double)i) - 1.0);
        maxValue = getTotalAmplitude(2.0);
    }

    // legacy 构造（createLegacy：直接消费 random，非 splitter 派生；用于 old_blended_noise）
    OctavePerlinNoiseSampler(XoroshiroRandom& random, bool legacy, int32_t firstOctave_, const std::vector<double>& amplitudes_)
        : firstOctave(firstOctave_), amplitudes(amplitudes_) {
        (void)legacy;
        size_t i = amplitudes.size();
        int32_t j = -firstOctave;
        octaveSamplers.resize(i);
        // Java: 无论如何都 new PerlinNoiseSampler(random)（消费 random），可能丢弃
        auto firstPN = std::make_unique<PerlinNoiseSampler>(random);
        if (j >= 0 && j < (int32_t)i) {
            double d = amplitudes[j];
            if (d != 0.0) {
                octaveSamplers[j] = std::move(firstPN);
            }
        }
        for (int32_t kx = j - 1; kx >= 0; kx--) {
            if (kx < (int32_t)i) {
                double e = amplitudes[kx];
                if (e != 0.0) {
                    octaveSamplers[kx] = std::make_unique<PerlinNoiseSampler>(random);
                } else {
                    random.skip(262);
                }
            } else {
                random.skip(262);
            }
        }
        lacunarity = std::pow(2.0, -j);
        persistence = std::pow(2.0, (double)i - 1.0) / (std::pow(2.0, (double)i) - 1.0);
        maxValue = getTotalAmplitude(2.0);
    }

    // 便捷：IntStream.rangeClosed(a, b) 全 1 振幅
    static std::vector<double> rangeClosedAmplitudes(int32_t from, int32_t to) {
        int32_t n = to - from + 1;
        return std::vector<double>(n, 1.0);
    }

    const PerlinNoiseSampler* getOctave(int32_t octave) const {
        int32_t idx = (int32_t)octaveSamplers.size() - 1 - octave;
        return (idx >= 0 && idx < (int32_t)octaveSamplers.size()) ? octaveSamplers[idx].get() : nullptr;
    }

    double method_40556(double d) const {
        return getTotalAmplitude(d + 2.0);
    }

    double getMaxValue() const { return maxValue; }

private:
    double getTotalAmplitude(double scale) const {
        double d = 0.0;
        double e = persistence;
        for (size_t i = 0; i < octaveSamplers.size(); i++) {
            if (octaveSamplers[i]) d += amplitudes[i] * scale * e;
            e /= 2.0;
        }
        return d;
    }

public:

    double sample(double x, double y, double z) const {
        double d = 0.0;
        double e = lacunarity;
        double f = persistence;
        for (size_t i = 0; i < octaveSamplers.size(); i++) {
            const auto& pn = octaveSamplers[i];
            if (pn) {
                double g = pn->sample(maintainPrecision(x * e), maintainPrecision(y * e), maintainPrecision(z * e));
                d += amplitudes[i] * g * f;
            }
            e *= 2.0;
            f /= 2.0;
        }
        return d;
    }
};

// DoublePerlinNoiseSampler（= NormalNoise，MC 1.20.1）
class DoublePerlinNoiseSampler {
public:
    static constexpr double DOMAIN_SCALE = 1.0181268882175227;

    struct NoiseParameters {
        int32_t firstOctave;
        std::vector<double> amplitudes;
    };

    double amplitude;
    OctavePerlinNoiseSampler firstSampler;
    OctavePerlinNoiseSampler secondSampler;
    double maxValue;

    // 默认构造（std::map operator[] 用；调用前必须赋值）
    DoublePerlinNoiseSampler() : amplitude(1.0), maxValue(1.0) {}

    DoublePerlinNoiseSampler(XoroshiroRandom& random, const NoiseParameters& params)
        : firstSampler(random, params.firstOctave, params.amplitudes),
          secondSampler(random, params.firstOctave, params.amplitudes) {
        int32_t j = INT32_MAX, k = INT32_MIN;
        for (size_t l = 0; l < params.amplitudes.size(); l++) {
            if (params.amplitudes[l] != 0.0) {
                j = std::min(j, (int32_t)l);
                k = std::max(k, (int32_t)l);
            }
        }
        amplitude = 0.16666666666666666 / createAmplitude(k - j);
        maxValue = (firstSampler.getMaxValue() + secondSampler.getMaxValue()) * amplitude;
    }

    double getMaxValue() const { return maxValue; }

    static double createAmplitude(int32_t octaves) {
        return 0.1 * (1.0 + 1.0 / (octaves + 1));
    }

    double sample(double x, double y, double z) const {
        double d = x * DOMAIN_SCALE;
        double e = y * DOMAIN_SCALE;
        double f = z * DOMAIN_SCALE;
        return (firstSampler.sample(x, y, z) + secondSampler.sample(d, e, f)) * amplitude;
    }
};

} // namespace wg
