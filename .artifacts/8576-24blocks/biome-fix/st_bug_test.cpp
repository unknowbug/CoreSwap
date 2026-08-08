// st_bug_test.cpp — SearchTree 独立复现（崩溃 0xC0000005 read 0x0，mov rdx,[rdx]）
// 用与 biome_params.json 相似规模/分布的合成 entries（1500+，大树路径）触发构建+查询。
#include "searchtree.h"
#include <cstdio>
#include <cstdint>
#include <memory>
#include <random>
#include <string>
#include <vector>

using namespace wg;

int main() {
    std::vector<SearchTree<std::string>::Entry> es;
    std::mt19937_64 rng(8576);
    auto rnd = [&](long lo, long hi) { return lo + (long)(rng() % (uint64_t)(hi - lo + 1)); };

    // 13 个 weirdness 区间 × 组合（≈ vanilla overworld 参数规模）
    long wspan = 700;
    for (int wb = 0; wb < 13; wb++) {
        long wmin = -10000 + wb * wspan;
        long wmax = wmin + wspan;
        long temps[] = {-10000, -4500, -1500, 2000, 5500};
        long hums[]  = {-10000, -3500, -1000, 1000, 3000};
        long conts[] = {-12000, -10500, -4550, -1900, -1100, 300, 10000};
        long eros[]  = {-10000, -7799, -3750, -2225, 500, 4500, 5500, 10000};
        long depths[] = {0, 10000};
        for (long t : temps)
        for (long h : hums)
        for (long c : conts)
        for (long e : eros)
        for (long d : depths) {
            SearchTree<std::string>::Entry entry;
            entry.parameters[0] = STRange{t, t + rnd(0, 3000)};
            entry.parameters[1] = STRange{h, h + rnd(0, 3000)};
            entry.parameters[2] = STRange{c, c + rnd(0, 3000)};
            entry.parameters[3] = STRange{e, e + rnd(0, 3000)};
            entry.parameters[4] = STRange{d, d};
            entry.parameters[5] = STRange{wmin, wmax};
            entry.parameters[6] = STRange{0, 0};
            entry.value = "minecraft:biome_" + std::to_string(es.size());
            es.push_back(std::move(entry));
        }
    }
    std::fprintf(stderr, "entries=%zu\n", es.size());

    auto tree = std::make_unique<SearchTree<std::string>>(std::move(es));
    std::fprintf(stderr, "built ok\n");

    // 大量查询（含判定点量级），触发 getResultingNode 递归
    for (int i = 0; i < 200000; i++) {
        long point[SearchTree<std::string>::DIM] = {
            rnd(-10000, 10000), rnd(-10000, 10000), rnd(-10000, 10000),
            rnd(-10000, 10000), rnd(-10000, 10000), rnd(-10000, 10000), 0L};
        const std::string* id = tree->get(point);
        if (!id) { std::fprintf(stderr, "NULL RESULT at i=%d\n", i); return 1; }
    }
    std::fprintf(stderr, "queries ok\n");
    return 0;
}
