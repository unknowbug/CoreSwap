#pragma once
#include <cstdint>
#include <string>
#include <array>
#include <stdexcept>
#include "md5.h"
#include "random.h"

namespace wg {

// MathHelper.hashCode(x, y, z)——3 参数 LCG 混合（1.20.1）
// 用于 RandomSplitter.split(x, y, z) 派生（aquifer/oreVein/surface/verticalGradient）
inline int64_t hashXYZ(int32_t x, int32_t y, int32_t z) {
    int32_t i = 1664525 * x + 1013904223;
    int32_t j = 1664525 * (y ^ -559038737) + 1013904223;
    int32_t k = 1664525 * (z ^ 984120213) + 1013904223;
    return (int64_t)(int32_t)(i ^ j ^ k); // int 溢出 → 符号扩展为 long
}

// Xoroshiro128PlusPlusRandom + Splitter（MC 1.20.1 移植）
class XoroshiroRandom {
public:
    Xoroshiro128PlusPlus impl;

    explicit XoroshiroRandom(uint64_t seed) : impl(seed) {}
    XoroshiroRandom(uint64_t lo, uint64_t hi) : impl(lo, hi) {}
    XoroshiroRandom(XoroshiroSeed s) : impl(s.seedLo, s.seedHi) {}

    uint64_t next() { return impl.next(); }
    int32_t nextInt() { return (int32_t)impl.next(); }

    int32_t nextInt(int32_t bound) {
        if (bound <= 0) throw std::invalid_argument("Bound must be positive");
        uint64_t l = (uint32_t)nextInt();
        uint64_t m = l * (uint64_t)bound;
        uint64_t n = m & 0xFFFFFFFFULL;
        if (n < (uint64_t)bound) {
            // Integer.remainderUnsigned(~bound + 1, bound)
            uint32_t rem = (uint32_t)((~(uint32_t)bound + 1u) % (uint32_t)bound);
            while (n < rem) {
                l = (uint32_t)nextInt();
                m = l * (uint64_t)bound;
                n = m & 0xFFFFFFFFULL;
            }
        }
        return (int32_t)(m >> 32);
    }

    double nextDouble() { return (double)(impl.next() >> 11) * 1.1102230246251565E-16; }
    float nextFloat() { return (float)(impl.next() >> 40) * 5.9604645E-8F; }
    bool nextBoolean() { return (impl.next() & 1ULL) != 0; }
    int32_t nextBetween(int32_t min, int32_t max) { return nextInt(max - min + 1) + min; }

    void skip(int64_t count) {
        for (int64_t i = 0; i < count; i++) impl.next();
    }

    XoroshiroRandom split() {
        uint64_t a = impl.next();
        uint64_t b = impl.next();
        return XoroshiroRandom(a, b);
    }

    class Splitter {
    public:
        uint64_t seedLo, seedHi;
        Splitter() : seedLo(0), seedHi(0) {}
        Splitter(uint64_t lo, uint64_t hi) : seedLo(lo), seedHi(hi) {}

        XoroshiroRandom split(int32_t x, int32_t y, int32_t z) const {
            int64_t l = hashXYZ(x, y, z);
            uint64_t m = (uint64_t)l ^ seedLo;
            return XoroshiroRandom(m, seedHi);
        }

        XoroshiroRandom split(const std::string& seed) const {
            XoroshiroSeed s = createXoroshiroSeed(seed);
            return XoroshiroRandom(s.split(seedLo, seedHi));
        }
    };

    Splitter nextSplitter() {
        uint64_t a = impl.next();
        uint64_t b = impl.next();
        return Splitter(a, b);
    }
};

inline XoroshiroSeed createXoroshiroSeed(const std::string& seed) {
    auto h = md5(seed);
    // Longs.fromBytes = big-endian
    uint64_t lo = 0, hi = 0;
    for (int i = 0; i < 8; i++) lo |= (uint64_t)h[i] << (8 * (7 - i));
    for (int i = 0; i < 8; i++) hi |= (uint64_t)h[8 + i] << (8 * (7 - i));
    return {lo, hi};
}

} // namespace wg
