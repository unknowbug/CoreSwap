// block_probe：方块层验证工具。
// 用 C++ 生成 4×4 chunk 区块（density → aquifer → surface），与 vanilla .blocks 参照对比。
// 用法: block_probe <seed> <worldgen dir> <vanilla.blocks 文件>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <chrono>
#include <map>
#include <string>
#include <thread>
#include <vector>

#include "worldgen_api.h"

// -compXY 用（确保 C 符号链接）
extern "C" double wg_router_sample(void* handle, const char* name, int x, int y, int z);

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
    try {
    if (argc < 4) {
        std::fprintf(stderr, "usage: block_probe <seed> <worldgen dir> <vanilla.blocks> [-threads N] [-dimension N]\n"
                             "  -threads N: 并行线程数；省略时按 min(CPU 线程数, chunk 数) 自适应\n"
                             "  -dimension N: 维度（0=overworld 默认，1=nether）\n");
        return 1;
    }
    int64_t seed = (int64_t)std::strtoull(argv[1], nullptr, 10);
    std::string wgDir = argv[2];
    std::string blocksPath = argv[3];
    int threadsArg = 0;
    int dimension = 0;
    bool listMismatch = false;
    for (int a = 4; a < argc; a++) {
        if (std::string(argv[a]) == "-threads" && a + 1 < argc) threadsArg = std::atoi(argv[a + 1]);
        else if (std::string(argv[a]) == "-dimension" && a + 1 < argc) dimension = std::atoi(argv[a + 1]);
        else if (std::string(argv[a]) == "-mismatch") listMismatch = true;
    }
    setvbuf(stdout, nullptr, _IONBF, 0); // 无缓冲，崩溃时保留输出定位

    const char* settingsName = "overworld.json";
    const char* biomeParams = "biome_params.json";
    int worldHeight = 0;
    if (dimension == 1) { settingsName = "nether.json"; biomeParams = "biome_params_nether.json"; worldHeight = 256; }
    void* h = wg_create(seed, wgDir.c_str(), settingsName, biomeParams, worldHeight);
    if (!h) { std::fprintf(stderr, "wg_create failed\n"); return 1; }

    // -blockDump X Y Z：生成指定 chunk 并输出指定 block 的方块
    for (int a = 4; a < argc; a++) {
        if (std::string(argv[a]) == "-blockDump" && a + 3 < argc) {
            int bx = std::atoi(argv[a + 1]), by = std::atoi(argv[a + 2]), bz = std::atoi(argv[a + 3]);
            int cxs[1] = {bx >> 4}, czs[1] = {bz >> 4};
            std::vector<int32_t> buf(16 * 16 * 384, 0);
            int32_t* outs[1] = {buf.data()};
            wg_fill_blocks_multi(h, cxs, czs, outs, 1, 1);
            int lx = bx & 15, lz = bz & 15;
            int idx = ((by - (-64)) * 16 + lz) * 16 + lx;
            std::fprintf(stderr, "[BLOCK] (%d,%d,%d) id=%d\n", bx, by, bz, idx >= 0 && idx < (int)buf.size() ? buf[idx] : -1);
            wg_destroy(h);
            return 0;
        }
    }

    // -biomeDump X Y Z：采样 biome 判定
    for (int a = 4; a < argc; a++) {
        if (std::string(argv[a]) == "-biomeDump" && a + 3 < argc) {
            int bx = std::atoi(argv[a + 1]), by = std::atoi(argv[a + 2]), bz = std::atoi(argv[a + 3]);
            char buf[64] = "";
            wg_sample_biome(h, bx, by, bz, buf, sizeof(buf));
            std::fprintf(stderr, "[BIOME] (%d,%d,%d) = %s\n", bx, by, bz, buf);
            wg_destroy(h);
            return 0;
        }
    }

    // -compXY X Z [BX BZ]：生成指定坐标的 router 分量（temperature/continents 等），不依赖参照
    for (int a = 4; a < argc; a++) {
        if (std::string(argv[a]) == "-compXY" && a + 2 < argc) {
            int cx = std::atoi(argv[a + 1]), cz = std::atoi(argv[a + 2]);
            int bx = a + 3 < argc ? std::atoi(argv[a + 3]) : 0;
            int bz = a + 4 < argc ? std::atoi(argv[a + 4]) : 0;
            std::fprintf(stderr, "[COMPXY] chunk(%d,%d) router 分量 @(%d,0,%d)……\n", cx, cz, cx * 16 + bx, cz * 16 + bz);
            const char* comps[] = {"temperature", "vegetation", "continents", "erosion", "depth", "factor", "ridges", "final_density", "initial_density", "sloped_cheese"};
            for (const char* c : comps) {
                std::fprintf(stderr, "[COMPXY] %s %d %.9f\n", c, cx * 16 + bx, wg_router_sample(h, c, cx * 16 + bx, 0, cz * 16 + bz));
            }
            wg_destroy(h);
            return 0;
        }
    }

    // -concTest N：模拟 MC 多 Worker 并发 fillBlocks（验证 CoreSwapPool 并发 run 修复）
    for (int a = 4; a < argc; a++) {
        if (std::string(argv[a]) == "-concTest" && a + 1 < argc) {
            int conc = std::atoi(argv[a + 1]);
            std::fprintf(stderr, "[CONC] 并发批次测试：%d 线程 × 20 批（每批 4 chunk）……\n", conc);
            std::vector<std::thread> ts;
            std::atomic<int> ok{0};
            for (int t = 0; t < conc; t++) {
                ts.emplace_back([&, t] {
                    for (int b = 0; b < 20; b++) {
                        int cxs[4], czs[4];
                        int32_t* outs4[4];
                        std::vector<int32_t> bufs[4];
                        for (int i = 0; i < 4; i++) {
                            cxs[i] = 3200 + (t * 4 + i) * 16 + b * 4;
                            czs[i] = 3208 + b * 16 - (t % 3) * 16;
                            bufs[i].assign(16 * 16 * 384, 0);
                            outs4[i] = bufs[i].data();
                        }
                        int r = wg_fill_blocks_multi(h, cxs, czs, outs4, 4, 0);
                        if (r != 4) { std::fprintf(stderr, "[CONC] 线程 %d 批 %d 返回 %d（异常！）\n", t, b, r); return; }
                    }
                    ok.fetch_add(1);
                });
            }
            for (auto& th : ts) th.join();
            std::fprintf(stderr, "[CONC] 完成：%d/%d 线程全部批次成功（无崩溃/异常）\n", ok.load(), conc);
            wg_destroy(h);
            return ok.load() == conc ? 0 : 1;
        }
    }

    // 读 vanilla 参照（大端）
    FILE* f = fopen(blocksPath.c_str(), "rb");
    if (!f) { std::fprintf(stderr, "cannot open %s\n", blocksPath.c_str()); return 1; }
    int32_t magic, vseedHi = 0, vseedLo = 0;
    int32_t size, originX, originZ, minY, height;
    uint8_t buf[256];  // biome 段字符串 blen < 128，必须 ≥ 128（曾用 8 导致栈越界写）
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

    const int BPC = 16 * 16 * height;   // 按参照头高度（overworld 384 / nether 256）
    int64_t total = 0, match = 0, matchNonAir = 0, totalNonAir = 0;
    std::vector<int> chunkX, chunkZ;
    std::vector<std::vector<int32_t>> vanillaAll, gotAll;
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
        // 跳过 biome 段（BlockProbe 新版格式：256 项 UTF 字符串）
        for (int i = 0; i < 256; i++) {
            if (std::fread(buf, 1, 2, f) != 2) break;
            int blen = ((int)buf[0] << 8) | buf[1];
            if (blen > 0 && blen < 128) std::fread(buf, 1, blen, f);
        }
        vanillaAll.push_back(std::move(vanilla));
        gotAll.emplace_back(BPC);
    }
    std::fclose(f);

    // 并行生成全部 chunk（线程数：-threads N 或 0=自适应 min(CPU, chunk 数)；结果与串行逐位一致）
    std::vector<int32_t*> outs;
    for (auto& g : gotAll) outs.push_back(g.data());
    auto tp0 = std::chrono::steady_clock::now();
    wg_fill_blocks_multi(h, chunkX.data(), chunkZ.data(), outs.data(), (int)gotAll.size(), threadsArg);
    auto tp1 = std::chrono::steady_clock::now();
    double wall = std::chrono::duration<double, std::milli>(tp1 - tp0).count();
    int hwc = (int)std::thread::hardware_concurrency();
    int used = threadsArg > 0 ? threadsArg : std::min(hwc > 0 ? hwc : 1, (int)gotAll.size());
    if (used > (int)gotAll.size()) used = (int)gotAll.size(); // 库内同样 clamp：线程数不超过任务数
    std::printf("parallel: %d chunks, %d threads (arg=%d hw=%d), wall=%.1fms (%.2fms/chunk)\n",
                (int)gotAll.size(), used, threadsArg, hwc, wall, wall / gotAll.size());

    for (int c = 0; c < (int)gotAll.size(); c++) {
        const auto& vanilla = vanillaAll[c];
        const auto& got = gotAll[c];
        int64_t cm = 0, cna = 0, tna = 0;
        for (int i = 0; i < BPC; i++) {
            total++;
            bool airV = (vanilla[i] == 0);
            if (!airV) { totalNonAir++; tna++; }
            if (vanilla[i] == got[i]) {
                match++; cm++;
                if (!airV) { matchNonAir++; cna++; }
            } else if (listMismatch) {
                int lx = i % 16, ly = i / 256, lz = (i / 16) % 16;
                if (chunkX[c] == 45 && chunkZ[c] == -27 && lx == 13 && lz == 5 && (ly == 159 || ly == 95 || ly == 128)) {
                    std::fprintf(stderr, "[DBG] chunk(45,-27) local(13,%d,5) i=%d got=%d vanilla=%d\n", ly, i, got[i], vanilla[i]);
                }
                char biomeBuf[64] = "";
                wg_sample_biome(h, chunkX[c] * 16 + lx, minY + ly, chunkZ[c] * 16 + lz, biomeBuf, sizeof(biomeBuf));
                std::printf("MISMATCH chunk(%d,%d) pos(%d,%d,%d) got=%d vanilla=%d biome=%s\n",
                            chunkX[c], chunkZ[c], chunkX[c] * 16 + lx, minY + ly, chunkZ[c] * 16 + lz,
                            got[i], vanilla[i], biomeBuf);
            }
        }
        std::printf("chunk (%d,%d): match=%lld/%d (%.2f%%) nonAir=%lld/%lld (%.2f%%)\n",
                    chunkX[c], chunkZ[c], cm, BPC, 100.0 * cm / BPC, cna, tna,
                    tna ? 100.0 * cna / tna : 100.0);
    }
    std::printf("TOTAL: match=%lld/%lld (%.4f%%) nonAir match=%lld/%lld (%.4f%%)\n",
                match, total, 100.0 * match / total, matchNonAir, totalNonAir,
                totalNonAir ? 100.0 * matchNonAir / totalNonAir : 0);
    wg_profile_dump();
    wg_destroy(h);
    return 0;
    } catch (const std::exception& e) {
        std::fprintf(stderr, "[EXC] %s\n", e.what());
        return 2;
    } catch (...) {
        std::fprintf(stderr, "[EXC] unknown\n");
        return 2;
    }
}
