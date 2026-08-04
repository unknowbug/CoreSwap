#pragma once
#include <cstdint>
#include <string>

namespace wg {

// RandomSeed.createXoroshiroSeed / mixStafford13 的 C++ 移植
inline uint64_t mixStafford13(uint64_t seed) {
    seed = (seed ^ (seed >> 30)) * 0xBF58476D1CE4E5B9ULL;
    seed = (seed ^ (seed >> 27)) * 0x94D049BB133111EBULL;
    return seed ^ (seed >> 31);
}

struct XoroshiroSeed {
    uint64_t seedLo, seedHi;
    XoroshiroSeed split(uint64_t lo, uint64_t hi) const {
        return {seedLo ^ lo, seedHi ^ hi};
    }
    XoroshiroSeed mix() const {
        return {mixStafford13(seedLo), mixStafford13(seedHi)};
    }
};

inline XoroshiroSeed createUnmixedXoroshiroSeed(uint64_t seed) {
    uint64_t lo = seed ^ 0x6A09E667F3BCC909ULL; // 7640891576956012809
    uint64_t hi = lo + 0x9E3779B97F4A7C15ULL;   // -7046029254386353131
    return {lo, hi};
}

inline XoroshiroSeed createXoroshiroSeed(uint64_t seed) {
    return createUnmixedXoroshiroSeed(seed).mix();
}

inline XoroshiroSeed createXoroshiroSeed(const std::string& seed);

// Xoroshiro128PlusPlus 核心
class Xoroshiro128PlusPlus {
public:
    uint64_t seedLo, seedHi;
    Xoroshiro128PlusPlus(uint64_t lo, uint64_t hi) : seedLo(lo), seedHi(hi) {
        if ((seedLo | seedHi) == 0) {
            seedLo = 0x9E3779B97F4A7C15ULL; // -7046029254386353131
            seedHi = 0x6A09E667F3BCC909ULL; // 7640891576956012809
        }
    }
    explicit Xoroshiro128PlusPlus(uint64_t seed) : Xoroshiro128PlusPlus(createXoroshiroSeed(seed).seedLo, createXoroshiroSeed(seed).seedHi) {}
    uint64_t next() {
        uint64_t lo = seedLo, hi = seedHi;
        uint64_t n = rotl(lo + hi, 17) + lo;
        hi ^= lo;
        seedLo = rotl(lo, 49) ^ hi ^ (hi << 21);
        seedHi = rotl(hi, 28);
        return n;
    }
private:
    static uint64_t rotl(uint64_t x, int c) { return (x << c) | (x >> (64 - c)); }
};

} // namespace wg
