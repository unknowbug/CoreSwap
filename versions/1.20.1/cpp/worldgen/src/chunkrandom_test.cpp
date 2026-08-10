// chunkrandom_test.cpp — ChunkRandom/CheckedRandom 移植对拍（参照 = ChunkRandomProbe Java 真实输出）
#include <cstdio>
#include <cstring>
#include <cstdint>
#include "chunkrandom.h"

using namespace wg;

static uint32_t f2u(float f) { uint32_t u; std::memcpy(&u, &f, 4); return u; }

static int failures = 0;
#define CHECK(name, cond) do { if (cond) std::printf("[OK] %s\n", name); else { std::printf("[FAIL] %s\n", name); failures++; } } while (0)
#define CHECK_EQ(name, got, want) do { auto g_ = (got); if (g_ == (want)) std::printf("[OK] %s = %lld\n", name, (long long)g_); else { std::printf("[FAIL] %s = %lld (want %lld)\n", name, (long long)g_, (long long)(want)); failures++; } } while (0)

int main() {
    const int64_t worldSeed = 8576294172403134396LL;

    // === CheckedRandom (LCG 48-bit) ===
    CheckedRandom cr(worldSeed);
    CHECK_EQ("cr.next(32)#1", cr.next(32), -1045225129);
    CHECK_EQ("cr.next(32)#2", cr.next(32), 1084206043);
    // 同一对象连续调用（对齐 Java probe 序列：next(32)x2 → nextLong x2 → nextInt(10) x2）
    CHECK_EQ("cr.nextLong#1", cr.nextLong(), -3933948616470016951LL);
    CHECK_EQ("cr.nextLong#2", cr.nextLong(), -518819946905544879LL);
    CHECK_EQ("cr.nextInt(10)#1", cr.nextInt(10), 4);
    CHECK_EQ("cr.nextInt(10)#2", cr.nextInt(10), 6);

    // === Xoroshiro128PlusPlusRandom（独立验证基类）===
    {
        XoroshiroRandom xr(worldSeed);
        CHECK_EQ("xr.nextLong#1", (int64_t)xr.next(), -6173829750206801647LL);
        CHECK_EQ("xr.nextLong#2", (int64_t)xr.next(), -2257156978324007003LL);
        CHECK_EQ("xr.nextLong#3", (int64_t)xr.next(), 215407908469699695LL);
        CHECK_EQ("xr.nextLong#4", (int64_t)xr.next(), 1678382591273684319LL);
        CHECK_EQ("xr.nextInt(10)", (int64_t)xr.nextInt(10), 5);
        CHECK_EQ("xr.nextFloat bits", (int64_t)f2u(xr.nextFloat()), (int64_t)f2u(0.5679502f));
    }

    // === ChunkRandom(Xoroshiro base) setPopulationSeed/setDecoratorSeed ===
    {
        ChunkRandom crx(ChunkRandom::BaseKind::XOROSHIRO);
        long long pop = crx.setPopulationSeed(worldSeed, 720 * 16, -432 * 16);
        CHECK_EQ("populationSeed", pop, -3665859634238804548LL);
        CHECK_EQ("afterPop.nextLong#1", crx.nextLong(), -7508349385403582096LL);
        CHECK_EQ("afterPop.nextLong#2", crx.nextLong(), -5481884486643468655LL);
        CHECK_EQ("afterPop.nextInt(256)", crx.nextInt(256), 7);
        CHECK_EQ("afterPop.nextFloat bits", (int64_t)f2u(crx.nextFloat()), (int64_t)f2u(0.49389488f));

        for (int step = 0; step < 2; step++) {
            for (int index = 0; index < 3; index++) {
                ChunkRandom c2(ChunkRandom::BaseKind::XOROSHIRO);
                c2.setPopulationSeed(worldSeed, 720 * 16, -432 * 16);
                c2.setDecoratorSeed(pop, index, step);
                long long nl = c2.nextLong();
                int ni = c2.nextInt(64);
                float nf = c2.nextFloat();
                std::printf("[deco(step=%d,index=%d)] nextLong=%lld nextInt(64)=%d nextFloat=0x%08X\n", step, index, nl, ni, f2u(nf));
            }
        }
    }

    // === ChunkRandom(CheckedRandom base) setCarverSeed ===
    {
        ChunkRandom crc(ChunkRandom::BaseKind::CHECKED);
        crc.setCarverSeed(worldSeed, -18, -15);
        CHECK_EQ("carver nextFloat bits", (int64_t)f2u(crc.nextFloat()), (int64_t)f2u(0.5614767f));
        CHECK_EQ("carver nextInt(16)#1", crc.nextInt(16), 12);
        CHECK_EQ("carver nextInt(16)#2", crc.nextInt(16), 11);
        CHECK_EQ("carver nextInt(16)#3", crc.nextInt(16), 2);

        ChunkRandom crc2(ChunkRandom::BaseKind::CHECKED);
        crc2.setCarverSeed(worldSeed, -18, -15);
        CHECK_EQ("carver2 nextFloat bits", (int64_t)f2u(crc2.nextFloat()), (int64_t)f2u(0.5614767f));
        CHECK_EQ("carver2 nextLong", crc2.nextLong(), -3711936206981428316LL);
    }

    std::printf("=== %s (failures=%d) ===\n", failures == 0 ? "ALL PASS" : "FAILED", failures);
    return failures == 0 ? 0 : 1;
}
