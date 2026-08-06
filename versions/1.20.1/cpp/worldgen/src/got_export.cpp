// got_export：输出 C++ 生成的方块数组（小端 int32，便于 python 差异分析）
// 用法: got_export <seed> <worldgen dir> <out file>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

#include "worldgen_api.h"

int main(int argc, char** argv) {
    if (argc < 4) { std::fprintf(stderr, "usage: got_export <seed> <worldgen dir> <out> [originX originZ]\n"); return 1; }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    int ox = argc >= 5 ? std::atoi(argv[4]) : 3200;
    int oz = argc >= 6 ? std::atoi(argv[5]) : 3208;
    int dimension = argc >= 7 ? std::atoi(argv[6]) : 0;
    const char* settingsName = "overworld.json";
    const char* biomeParams = "biome_params.json";
    int worldHeight = 0;
    if (dimension == 1) { settingsName = "nether.json"; biomeParams = "biome_params_nether.json"; worldHeight = 256; }
    void* h = wg_create(seed, argv[2], settingsName, biomeParams, worldHeight);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }
    FILE* f = fopen(argv[3], "wb");
    if (!f) return 1;
    const int BPC = 16 * 16 * (dimension == 1 ? 256 : 384);
    int32_t hdr2[5] = {0x57474233, (int32_t)(seed & 0xFFFFFFFF), 4, ox, oz};
    std::fwrite(hdr2, 4, 5, f);
    int cxs[16], czs[16];
    std::vector<std::vector<int32_t>> outData(16, std::vector<int32_t>(BPC));
    std::vector<int32_t*> outs(16);
    for (int i = 0; i < 16; i++) { cxs[i] = ox / 16 + i % 4; czs[i] = oz / 16 + i / 4; outs[i] = outData[i].data(); }
    wg_fill_blocks_multi(h, cxs, czs, outs.data(), 16, 16);
    for (int i = 0; i < 16; i++) {
        int32_t pos[2] = {cxs[i], czs[i]};
        std::fwrite(pos, 4, 2, f);
        std::fwrite(outs[i], 4, BPC, f);
    }
    std::fclose(f);
    wg_destroy(h);
    std::printf("got exported\n");
    return 0;
}
