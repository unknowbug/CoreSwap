// block_probe：方块层验证工具。
// 用 C++ 生成 4×4 chunk 区块（density → aquifer → surface），与 vanilla .blocks 参照对比。
// 用法: block_probe <seed> <worldgen dir> <vanilla.blocks 文件>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <chrono>
#include <string>
#include <vector>

#include "worldgen_api.h"

static int32_t be32(int32_t i) {
    uint32_t u = (uint32_t)i;
    return (int32_t)(((u & 0xFF) << 24) | ((u & 0xFF00) << 8) | ((u >> 8) & 0xFF00) | ((u >> 24) & 0xFF));
}
static int64_t be64(int64_t v) {
    uint64_t u = (uint64_t)v;
    return (int64_t)(((u & 0xFFULL) << 56) | ((u & 0xFF00ULL) << 40) | ((u & 0xFF0000ULL) << 24) | ((u & 0xFF000000ULL) << 8) |
                     ((u >> 8) & 0xFF000000ULL) | ((u >> 24) & 0xFF0000ULL) | ((u >> 40) & 0xFF00ULL) | ((u >> 56) & 0xFFULL));
}

int main(int argc, char** argv) {
    if (argc < 4) {
        std::fprintf(stderr, "usage: block_probe <seed> <worldgen dir> <vanilla.blocks>\n");
        return 1;
    }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];
    std::string blocksPath = argv[3];
    setvbuf(stdout, nullptr, _IONBF, 0); // 无缓冲，崩溃时保留输出定位

    void* h = wg_create(seed, wgDir.c_str());
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // 读 vanilla 参照（大端）
    FILE* f = fopen(blocksPath.c_str(), "rb");
    if (!f) { std::fprintf(stderr, "cannot open %s\n", blocksPath.c_str()); return 1; }
    int32_t magic, vseedHi = 0, vseedLo = 0;
    int32_t size, originX, originZ, minY, height;
    uint8_t buf[8];
    auto rd32 = [&]() { std::fread(buf, 1, 4, f); int32_t v; std::memcpy(&v, buf, 4); return be32(v); };
    auto rd64 = [&]() { std::fread(buf, 1, 8, f); int64_t v; std::memcpy(&v, buf, 8); return be64(v); };
    magic = rd32();
    int64_t vseed = rd64();
    size = rd32();
    originX = rd32();
    originZ = rd32();
    minY = rd32();
    height = rd32();
    std::printf("blocks file: magic=0x%08X seed=%lld size=%d origin=(%d,%d) minY=%d height=%d\n",
                magic, (long long)vseed, size, originX, originZ, minY, height);

    const int BPC = 16 * 16 * 384;
    int64_t total = 0, match = 0, matchNonAir = 0, totalNonAir = 0;
    std::vector<int> chunkX, chunkZ;
    for (int c = 0; c < size * size; c++) {
        int cx = rd32(), cz = rd32();
        chunkX.push_back(cx);
        chunkZ.push_back(cz);
        std::vector<int32_t> vanilla(BPC);
        for (int i = 0; i < BPC; i++) {
            std::fread(buf, 1, 2, f);
            uint16_t v = (uint16_t)((buf[0] << 8) | buf[1]);
            vanilla[i] = (int32_t)v;
        }
        std::vector<int32_t> got(BPC);
        auto t0 = std::chrono::steady_clock::now();
        wg_fill_blocks(h, cx, cz, got.data());
        auto t1 = std::chrono::steady_clock::now();
        double ms = std::chrono::duration<double, std::milli>(t1 - t0).count();

        int64_t cm = 0, cna = 0, tna = 0;
        for (int i = 0; i < BPC; i++) {
            total++;
            bool airV = (vanilla[i] == 0);
            if (!airV) { totalNonAir++; tna++; }
            if (vanilla[i] == got[i]) {
                match++; cm++;
                if (!airV) { matchNonAir++; cna++; }
            }
        }
        std::printf("chunk (%d,%d): match=%lld/%d (%.2f%%) nonAir=%lld/%lld (%.2f%%) %.2f ms\n",
                    cx, cz, cm, BPC, 100.0 * cm / BPC, cna, tna,
                    tna ? 100.0 * cna / tna : 100.0, ms);
    }
    std::fclose(f);
    std::printf("TOTAL: match=%lld/%lld (%.4f%%) nonAir match=%lld/%lld (%.4f%%)\n",
                match, total, 100.0 * match / total, matchNonAir, totalNonAir,
                totalNonAir ? 100.0 * matchNonAir / totalNonAir : 0);
    wg_destroy(h);
    return 0;
}
