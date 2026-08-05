// got_export：输出 C++ 生成的方块数组（小端 int32，便于 python 差异分析）
// 用法: got_export <seed> <worldgen dir> <out file>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

#include "worldgen_api.h"

int main(int argc, char** argv) {
    if (argc < 4) { std::fprintf(stderr, "usage: got_export <seed> <worldgen dir> <out>\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    void* h = wg_create(seed, argv[2]);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }
    FILE* f = fopen(argv[3], "wb");
    if (!f) return 1;
    const int BPC = 16 * 16 * 384;
    int32_t hdr2[5] = {0x57474233, (int32_t)(seed & 0xFFFFFFFF), 4, 3200, 3208};
    std::fwrite(hdr2, 4, 5, f);
    std::vector<int32_t> got(BPC);
    for (int cz = 0; cz < 4; cz++) {
        for (int cx = 0; cx < 4; cx++) {
            int32_t pos[2] = {3200 / 16 + cx, 3208 / 16 + cz};
            std::fwrite(pos, 4, 2, f);
            wg_fill_blocks(h, pos[0], pos[1], got.data());
            std::fwrite(got.data(), 4, BPC, f);
        }
    }
    std::fclose(f);
    wg_destroy(h);
    std::printf("got exported\n");
    return 0;
}
