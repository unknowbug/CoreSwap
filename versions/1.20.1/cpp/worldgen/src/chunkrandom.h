#pragma once
// chunkrandom.h — ChunkRandom + CheckedRandom（MC 1.20.1 移植，FEATURE/CARVER 种子派生）
// 语义来源：
//   - CheckedRandom.java（48 位 LCG，java.util.Random 算法）
//   - ChunkRandom.java（包装 baseRandom，next(bits) 按基类类型分发）
//   - BaseRandom.java（nextLong/nextInt(bound)/nextFloat/nextDouble 默认实现）
//   - Xoroshiro128PlusPlusRandom.java（FEATURES 用，C++ 已有 Xoroshiro128PlusPlus）
// 关键易错点（MC-239059）：BaseRandom.nextLong() = (long)next(32) << 32 + next(32)
//   —— i/j 都是 int 符号扩展后做有符号加法，j<0 时高 32 位被 0xFFFFFFFF 填充
//   （非无符号位拼接！）。setPopulationSeed 的 nextLong 走 ChunkRandom.next(bits)，
//   Xoroshiro 基类下 = 每次消费 1 轮 Xoroshiro 输出的高 32 位（共 4 轮）。
#include <cstdint>
#include <stdexcept>
#include "random.h"
#include "xoroshiro.h"

namespace wg {

// CheckedRandom（48 位 LCG）——CARVERS 阶段 ChunkRandom 的基类
class CheckedRandom {
public:
    static constexpr uint64_t MULTIPLIER = 25214903917ULL;
    static constexpr uint64_t INCREMENT = 11ULL;
    static constexpr uint64_t SEED_MASK = 281474976710655ULL; // (1<<48)-1

    uint64_t seed_ = 0;

    explicit CheckedRandom(int64_t seed) { setSeed(seed); }
    CheckedRandom() = default;

    void setSeed(int64_t seed) {
        seed_ = ((uint64_t)seed ^ MULTIPLIER) & SEED_MASK;
    }

    // Java next(int bits)：seed = seed*M + 11 & MASK；返回 (int)(seed >> 48-bits)
    int32_t next(int bits) {
        seed_ = (seed_ * MULTIPLIER + INCREMENT) & SEED_MASK;
        return (int32_t)(seed_ >> (48 - bits));
    }

    // BaseRandom.nextLong()：(long)next(32) << 32 + next(32)（有符号拼接，MC-239059）
    int64_t nextLong() {
        int32_t i = next(32);
        int32_t j = next(32);
        return ((int64_t)i << 32) + (int64_t)j;
    }

    // BaseRandom.nextInt(bound)（默认实现）：幂 2 用 next(31)，否则拒绝采样
    int32_t nextInt(int32_t bound) {
        if (bound <= 0) throw std::invalid_argument("Bound must be positive");
        if ((bound & (bound - 1)) == 0) {
            return (int32_t)(((int64_t)bound * next(31)) >> 31);
        }
        int32_t i, j;
        do {
            i = next(31);
            j = i % bound;
        } while ((int32_t)((uint32_t)i - (uint32_t)j + (uint32_t)(bound - 1)) < 0);
        return j;
    }

    float nextFloat() { return (float)next(24) * 5.9604645E-8F; }
};

// ChunkRandom：包装基类（CheckedRandom=LCG 或 Xoroshiro128PlusPlus）
// next(bits)：基类为 CheckedRandom → checkedRandom.next(bits)（LCG）
//             基类为 Xoroshiro → (int)(baseRandom.nextLong() >>> 64-bits)（高 bits 位）
// 其余 nextInt/nextLong/nextFloat/nextBoolean/nextDouble 走 BaseRandom 默认实现
//   （nextLong = (long)next(32) << 32 + next(32)，有符号拼接；nextInt(bound) 幂 2 分支
//    用 next(31)；非幂 2 用 do-while 拒绝采样 i % bound）
class ChunkRandom {
public:
    enum class BaseKind { CHECKED, XOROSHIRO };

    BaseKind kind_;
    CheckedRandom lcg_;      // kind=CHECKED 时使用
    Xoroshiro128PlusPlus xoro_; // kind=XOROSHIRO 时使用
    int sampleCount_ = 0;

    // 构造：Java 用 new ChunkRandom(new CheckedRandom(...)) / new ChunkRandom(new Xoroshiro128PlusPlusRandom(...))
    // 初始种子会被后续 setPopulationSeed/setCarverSeed 覆盖，只要求 setSeed 语义正确
    explicit ChunkRandom(BaseKind kind) : kind_(kind), xoro_(0) {}

    int getSampleCount() const { return sampleCount_; }

    void setSeed(int64_t seed) {
        if (kind_ == BaseKind::CHECKED) lcg_.setSeed(seed);
        else xoro_.seedLo = createXoroshiroSeed((uint64_t)seed).seedLo, xoro_.seedHi = createXoroshiroSeed((uint64_t)seed).seedHi;
    }

    // Java ChunkRandom.next(bits)
    int32_t next(int bits) {
        sampleCount_++;
        if (kind_ == BaseKind::CHECKED) return lcg_.next(bits);
        // (int)(baseRandom.nextLong() >>> 64 - bits)——Xoroshiro nextLong = 完整 64 位，取高 bits 位
        return (int32_t)(xoro_.next() >> (64 - bits));
    }

    // BaseRandom.nextLong()：(long)next(32) << 32 + next(32)
    int64_t nextLong() {
        int32_t i = next(32);
        int32_t j = next(32);
        return ((int64_t)i << 32) + (int64_t)j; // 有符号拼接（j 符号扩展相加，MC-239059）
    }

    int32_t nextInt() { return next(32); }

    // BaseRandom.nextInt(bound)：幂 2 用 (int)((long)bound * next(31) >> 31)，否则拒绝采样
    int32_t nextInt(int32_t bound) {
        if (bound <= 0) throw std::invalid_argument("Bound must be positive");
        if ((bound & (bound - 1)) == 0) {
            return (int32_t)(((int64_t)bound * next(31)) >> 31);
        }
        int32_t i, j;
        do {
            i = next(31);
            j = i % bound;
        } while ((int32_t)((uint32_t)i - (uint32_t)j + (uint32_t)(bound - 1)) < 0); // Java int 回绕（无符号模拟防 UB）
        return j;
    }

    bool nextBoolean() { return next(1) != 0; }

    // BaseRandom.nextFloat() = next(24) * 5.9604645E-8F（float 乘法）
    float nextFloat() { return (float)next(24) * 5.9604645E-8F; }

    // BaseRandom.nextDouble() = ((long)next(26) << 27 + next(27)) * 1.110223E-16F
    // Java 语义：long * float 是 float 乘法（精度截断），结果提升回 double——用 float 模拟
    double nextDouble() {
        int32_t i = next(26);
        int32_t j = next(27);
        int64_t l = ((int64_t)i << 27) + (int64_t)j;
        return (double)((float)l * 1.110223E-16F);
    }

    // ---- 种子派生（ChunkRandom.java）----

    // setPopulationSeed(worldSeed, blockX, blockZ)：FEATURES 阶段
    //   setSeed(worldSeed); l=nextLong()|1; m=nextLong()|1; n=blockX*l + blockZ*m ^ worldSeed; setSeed(n)
    int64_t setPopulationSeed(int64_t worldSeed, int32_t blockX, int32_t blockZ) {
        setSeed(worldSeed);
        int64_t l = nextLong() | 1LL;
        int64_t m = nextLong() | 1LL;
        // blockX * l：int * long → long 乘法（blockX 符号扩展）；(a + b) ^ worldSeed
        int64_t n = ((int64_t)blockX * l + (int64_t)blockZ * m) ^ worldSeed;
        setSeed(n);
        return n;
    }

    // setDecoratorSeed(populationSeed, index, step)：l = populationSeed + index + 10000*step
    void setDecoratorSeed(int64_t populationSeed, int32_t index, int32_t step) {
        int64_t l = populationSeed + index + 10000LL * step;
        setSeed(l);
    }

    // setCarverSeed(worldSeed, chunkX, chunkZ)：CARVERS 阶段
    //   setSeed(worldSeed); l=nextLong(); m=nextLong(); n=chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)
    void setCarverSeed(int64_t worldSeed, int32_t chunkX, int32_t chunkZ) {
        setSeed(worldSeed);
        int64_t l = nextLong();
        int64_t m = nextLong();
        int64_t n = ((int64_t)chunkX * l) ^ ((int64_t)chunkZ * m) ^ worldSeed;
        setSeed(n);
    }
};

} // namespace wg
