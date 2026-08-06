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
    // 密度 dump 模式：-densityDump cx cz bx bz（下界；输出 y 0..128 每 4 的 finalDensity，格式同 Java DensityProbe）
    if (argc >= 4 && std::string(argv[3]) == "-densityDump") {
        int dcx = std::atoi(argv[4]), dcz = std::atoi(argv[5]), dbx = std::atoi(argv[6]), dbz = std::atoi(argv[7]);
        void* dh = wg_create(seed, argv[2], "nether.json", "biome_params_nether.json", 256);
        if (!dh) { std::fprintf(stderr, "wg_create(nether) failed\n"); return 1; }
        int wx = dcx * 16 + dbx, wz = dcz * 16 + dbz;
        for (int y = 0; y <= 128; y += 4) {
            std::printf("%d %.6f\n", y, wg_sample_density(dh, wx, y, wz));
        }
        wg_destroy(dh);
        return 0;
    }
    // 任意注册 density function dump：-namedDump <name> cx cz bx bz [dimension]
    if (argc >= 7 && std::string(argv[3]) == "-namedDump") {
        std::string nm = argv[4];
        int dcx = std::atoi(argv[5]), dcz = std::atoi(argv[6]), dbx = std::atoi(argv[7]), dbz = std::atoi(argv[8]);
        int ddim = 0;
        for (int a = 9; a + 1 < argc; a++)
            if (std::string(argv[a]) == "-dimension") { ddim = std::atoi(argv[a + 1]); break; }
        const char* sname = ddim == 1 ? "nether.json" : "overworld.json";
        const char* bparams = ddim == 1 ? "biome_params_nether.json" : "biome_params.json";
        int wh = ddim == 1 ? 256 : 0;
        void* dh = wg_create(seed, argv[2], sname, bparams, wh);
        if (!dh) { std::fprintf(stderr, "wg_create failed\n"); return 1; }
        int wx = dcx * 16 + dbx, wz = dcz * 16 + dbz;
        for (int y = -64; y <= 127; y += 4) {
            double v = wg_sample_named(dh, nm.c_str(), wx, y, wz);
            std::printf("%d %.17g\n", y, v);
        }
        wg_destroy(dh);
        return 0;
    }
    // base_3d_noise 分量 dump：-nbDump cx cz bx bz [dimension]（默认 1=nether；0=overworld）
    if (argc >= 4 && std::string(argv[3]) == "-nbDump") {
        int dcx = std::atoi(argv[4]), dcz = std::atoi(argv[5]), dbx = std::atoi(argv[6]), dbz = std::atoi(argv[7]);
        // 维度：argv[8] 直接是数字（旧用法），或后跟 "-dimension N"（新用法）
        int ddim = 1;
        if (argc >= 9) {
            if (std::isdigit((unsigned char)argv[8][0]) || argv[8][0] == '-') {
                long v = std::strtol(argv[8], nullptr, 10);
                if (std::strtol(argv[8], nullptr, 10) == 0 && (argv[8][0] != '0' || argv[8][1] != '\0')) {
                    // 非数字：找 "-dimension N"
                    for (int a = 8; a + 1 < argc; a++)
                        if (std::string(argv[a]) == "-dimension") { ddim = std::atoi(argv[a + 1]); break; }
                } else ddim = (int)v;
            }
        }
        const char* sname = ddim == 1 ? "nether.json" : "overworld.json";
        const char* bparams = ddim == 1 ? "biome_params_nether.json" : "biome_params.json";
        int wh = ddim == 1 ? 256 : 0;
        void* dh = wg_create(seed, argv[2], sname, bparams, wh);
        if (!dh) { std::fprintf(stderr, "wg_create failed\n"); return 1; }
        const char* dfName = ddim == 1 ? "minecraft:nether/base_3d_noise" : "minecraft:overworld/base_3d_noise";
        int wx = dcx * 16 + dbx, wz = dcz * 16 + dbz;
        for (int y = 0; y <= 128; y += 4) {
            double v = wg_sample_named(dh, dfName, wx, y, wz);
            std::printf("%d %.17g %a\n", y, v, v);
        }
        wg_destroy(dh);
        return 0;
    }
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
