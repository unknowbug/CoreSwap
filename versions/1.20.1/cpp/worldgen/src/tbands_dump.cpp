// tbands_dump.cpp — 一次性：复刻 Java SurfaceBuilder.createTerracottaBands，导出 192 带（block raw id）
// 对比 RouterProbe 的 TBANDS 输出。随机链：splitter.split("minecraft:clay_bands")（与 Java randomDeriver.split(clay_bands) 一致）
#include <cstdio>
#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "blocks.h"

using namespace wg;

int main(int argc, char** argv) {
    uint64_t seed = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 8576294172403134396ULL;
    XoroshiroRandom base(seed);
    auto splitter = base.nextSplitter();
    XoroshiroRandom bandRandom = splitter.split("minecraft:clay_bands");

    BlockRegistry blocks;
    {
        std::string p = argc > 2 ? argv[2] : "E:/PYTHON/MC/data/worldgen/../blocks.json";
        // worldgen_api.cpp 用 wgDir + "/../blocks.json"，wgDir=E:/PYTHON/MC/data/worldgen → E:/PYTHON/MC/data/blocks.json
        FILE* f = fopen(p.c_str(), "rb");
        if (!f) { std::fprintf(stderr, "cannot open %s\n", p.c_str()); return 2; }
        std::string txt;
        char buf[65536];
        size_t n;
        while ((n = fread(buf, 1, sizeof(buf), f)) > 0) txt.append(buf, n);
        fclose(f);
        if (!blocks.loadFromJson(txt)) { std::fprintf(stderr, "loadFromJson failed\n"); return 3; }
    }

    int terracotta = blocks.id("minecraft:terracotta");
    int orange = blocks.id("minecraft:orange_terracotta");
    int yellow = blocks.id("minecraft:yellow_terracotta");
    int brown = blocks.id("minecraft:brown_terracotta");
    int red = blocks.id("minecraft:red_terracotta");
    int white = blocks.id("minecraft:white_terracotta");
    int lightGray = blocks.id("minecraft:light_gray_terracotta");

    std::vector<int> bands(192, terracotta);
    // Java: for (int i = 0; i < 192; i++) { i += nextInt(5)+1; if (i < 192) bands[i]=ORANGE; }
    for (int i = 0; i < 192; i++) {
        i += bandRandom.nextInt(5) + 1;
        if (i < 192) bands[i] = orange;
    }
    // addTerracottaBands(r, bands, 1, YELLOW)
    {
        int i = bandRandom.nextBetween(6, 15);
        for (int j = 0; j < i; j++) {
            int k = 1 + bandRandom.nextInt(3);
            int l = bandRandom.nextInt(192);
            for (int m = 0; l + m < 192 && m < k; m++) bands[l + m] = yellow;
        }
    }
    // addTerracottaBands(r, bands, 2, BROWN)
    {
        int i = bandRandom.nextBetween(6, 15);
        for (int j = 0; j < i; j++) {
            int k = 2 + bandRandom.nextInt(3);
            int l = bandRandom.nextInt(192);
            for (int m = 0; l + m < 192 && m < k; m++) bands[l + m] = brown;
        }
    }
    // addTerracottaBands(r, bands, 1, RED)
    {
        int i = bandRandom.nextBetween(6, 15);
        for (int j = 0; j < i; j++) {
            int k = 1 + bandRandom.nextInt(3);
            int l = bandRandom.nextInt(192);
            for (int m = 0; l + m < 192 && m < k; m++) bands[l + m] = red;
        }
    }
    // Java: int ix = nextBetween(9,15); int j=0; for (k=0; j<ix && k<192; k+=nextInt(16)+4) {...}
    {
        int ix = bandRandom.nextBetween(9, 15);
        int j = 0;
        for (int k = 0; j < ix && k < 192; k += bandRandom.nextInt(16) + 4) {
            bands[k] = white;
            if (k - 1 > 0 && bandRandom.nextBoolean()) bands[k - 1] = lightGray;
            if (k + 1 < 192 && bandRandom.nextBoolean()) bands[k + 1] = lightGray;
            j++;
        }
    }
    printf("TBANDS");
    for (int b : bands) printf(" %d", b);
    printf("\n");
    return 0;
}
