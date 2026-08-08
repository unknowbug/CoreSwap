// biome.h — MultiNoiseBiomeSource 复刻：六维噪声参数 → 最近 biome
// 查找 = vanilla MultiNoiseUtil.SearchTree（平局 tie-break 对齐 Java 树序遍历；非平局 = 唯一最近邻）
// @anchor.test("biomeJitter 扰动对齐 Java（8 邻域 seed 哈希选点），surface rule 逐块 biome 判定", source="probe:block_probe!SURFBIOME#002")
// + BiomeAccess.getBiome(BlockPos) 的 8 邻域 seed 哈希选点（surface rule 逐块 biome 判定的真实路径）
#pragma once
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <algorithm>

#include "json.h"
#include "searchtree.h"   // MultiNoiseUtil.SearchTree 移植（平局 tie-break 对齐）
#include <memory>         // std::unique_ptr
#include <mutex>          // std::once_flag / std::call_once（searchTree 懒构建线程安全）

namespace wg {

// ===== SHA-256（仅用于 BiomeAccess.hashSeed = Hashing.sha256().hashLong(seed).asLong()）=====
namespace sha256detail {
inline uint32_t rotr(uint32_t x, int n) { return (x >> n) | (x << (32 - n)); }
inline uint32_t shr(uint32_t x, int n) { return x >> n; }
} // namespace sha256detail

// 标准 SHA-256（单块/多块通用，输出 32 字节大端）
inline void sha256(const uint8_t* data, size_t len, uint8_t out[32]) {
    static const uint32_t K[64] = {
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};
    uint32_t h[8] = {0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
    // padding：先复制数据（len + 1 补 0x80 + 至 56 mod 64 + 8 字节长度）
    size_t total = ((len + 8) / 64 + 1) * 64;
    std::vector<uint8_t> msg(total, 0);
    std::memcpy(msg.data(), data, len);
    msg[len] = 0x80;
    uint64_t bitLen = (uint64_t)len * 8;
    for (int i = 0; i < 8; i++) msg[total - 1 - i] = (uint8_t)(bitLen >> (8 * i));
    for (size_t off = 0; off < total; off += 64) {
        uint32_t w[64];
        for (int i = 0; i < 16; i++)
            w[i] = ((uint32_t)msg[off + i*4] << 24) | ((uint32_t)msg[off + i*4+1] << 16)
                 | ((uint32_t)msg[off + i*4+2] << 8) | (uint32_t)msg[off + i*4+3];
        for (int i = 16; i < 64; i++) {
            uint32_t s0 = sha256detail::rotr(w[i-15], 7) ^ sha256detail::rotr(w[i-15], 18) ^ sha256detail::shr(w[i-15], 3);
            uint32_t s1 = sha256detail::rotr(w[i-2], 17) ^ sha256detail::rotr(w[i-2], 19) ^ sha256detail::shr(w[i-2], 10);
            w[i] = w[i-16] + s0 + w[i-7] + s1;
        }
        uint32_t a=h[0],b=h[1],c=h[2],d=h[3],e=h[4],f=h[5],g=h[6],hh=h[7];
        for (int i = 0; i < 64; i++) {
            uint32_t S1 = sha256detail::rotr(e,6) ^ sha256detail::rotr(e,11) ^ sha256detail::rotr(e,25);
            uint32_t ch = (e & f) ^ (~e & g);
            uint32_t t1 = hh + S1 + ch + K[i] + w[i];
            uint32_t S0 = sha256detail::rotr(a,2) ^ sha256detail::rotr(a,13) ^ sha256detail::rotr(a,22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t t2 = S0 + maj;
            hh=g; g=f; f=e; e=d+t1; d=c; c=b; b=a; a=t1+t2;
        }
        h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d; h[4]+=e; h[5]+=f; h[6]+=g; h[7]+=hh;
    }
    for (int i = 0; i < 8; i++) {
        out[i*4] = (uint8_t)(h[i] >> 24); out[i*4+1] = (uint8_t)(h[i] >> 16);
        out[i*4+2] = (uint8_t)(h[i] >> 8); out[i*4+3] = (uint8_t)h[i];
    }
}

// BiomeAccess.hashSeed：Hashing.sha256().hashLong(seed).asLong()
// Guava putLong 写 little-endian 8 字节；asLong 取结果前 8 字节 little-endian
inline int64_t biomeHashSeed(int64_t seed) {
    uint8_t le[8];
    for (int i = 0; i < 8; i++) le[i] = (uint8_t)((uint64_t)seed >> (8 * i));
    uint8_t dg[32];
    sha256(le, 8, dg);
    int64_t out = 0;
    for (int i = 0; i < 8; i++) out |= (int64_t)dg[i] << (8 * i);
    return out;
}

// SeedMixer.mixSeed（1.20.1 无符号回绕语义）
inline int64_t mixSeed(int64_t seed, int64_t salt) {
    uint64_t s = (uint64_t)seed;
    uint64_t v = s * (s * 6364136223846793005ULL + 1442695040888963407ULL);
    return (int64_t)(v + (uint64_t)salt);
}

// method_38108：floorMod(l >> 24, 1024) / 1024.0，然后 (d - 0.5) * 0.9
inline double biomeJitter(int64_t l) {
    int64_t shifted = l >> 24;
    int64_t fm = shifted % 1024;
    if (fm < 0) fm += 1024;
    double d = (double)fm / 1024.0;
    return (d - 0.5) * 0.9;
}

// method_38106(seed, q, r, s, d, e, f)：8 邻域候选点到 block 的哈希扰动距离
inline double biomeCellDistance(int64_t seed, int q, int r, int s, double d, double e, double f) {
    int64_t m = mixSeed(seed, q);
    m = mixSeed(m, r);
    m = mixSeed(m, s);
    m = mixSeed(m, q);
    m = mixSeed(m, r);
    m = mixSeed(m, s);
    double g = biomeJitter(m);
    m = mixSeed(m, seed);
    double h = biomeJitter(m);
    m = mixSeed(m, seed);
    double n = biomeJitter(m);
    return (f + n) * (f + n) + (e + h) * (e + h) + (d + g) * (d + g);
}

// @anchor.test("biomePickCell 8 邻域选点对齐 Java BiomeAccess.getBiome（负坐标 >>2 算术右移 + seed 哈希）", source="probe:block_probe!SURFBIOME#001")
// BiomeAccess.getBiome(BlockPos) 的选点：block 坐标 → 选中的 biome 坐标 (px, py, pz)
// 等价 Java：i=x-2, j=y-2, k=z-2; l=i>>2...；8 邻域取最近扰动点
inline void biomePickCell(int64_t accessSeed, int blockX, int blockY, int blockZ, int& px, int& py, int& pz) {
    int i = blockX - 2;
    int j = blockY - 2;
    int k = blockZ - 2;
    int l = i >> 2;
    int m = j >> 2;
    int n = k >> 2;
    double d = (i & 3) / 4.0;
    double e = (j & 3) / 4.0;
    double f = (k & 3) / 4.0;
    int o = 0;
    double best = 1e300;
    for (int p = 0; p < 8; p++) {
        bool bl = (p & 4) == 0;
        bool bl2 = (p & 2) == 0;
        bool bl3 = (p & 1) == 0;
        int q = bl ? l : l + 1;
        int r = bl2 ? m : m + 1;
        int s = bl3 ? n : n + 1;
        double h = bl ? d : d - 1.0;
        double t = bl2 ? e : e - 1.0;
        double u = bl3 ? f : f - 1.0;
        double v = biomeCellDistance(accessSeed, q, r, s, h, t, u);
        if (best > v) { o = p; best = v; }
    }
    px = (o & 4) == 0 ? l : l + 1;
    py = (o & 2) == 0 ? m : m + 1;
    pz = (o & 1) == 0 ? n : n + 1;
}

// toLong/toFloat：与 MultiNoiseUtil 一致（float 精度！）
inline long noiseToLong(float v) { return (long)(v * 10000.0F); }
inline float noiseToFloat(long v) { return (float)v / 10000.0F; }

// 六维超立方体（对应 NoiseHypercube）
struct NoiseHypercube {
    long tempMin, tempMax;
    long humMin, humMax;
    long contMin, contMax;
    long eroMin, eroMax;
    long depthMin, depthMax;
    long weirdMin, weirdMax;
    long offset;

    // 6 维距离平方和（MSVC 铁律：long=32 位，距离平方和可能超 2^31 → long long）
    long long getSquaredDistance(long long t, long long h, long long c, long long e, long long d, long long w) const {
        long long dt = rangeDistance(tempMin, tempMax, t);
        long long dh = rangeDistance(humMin, humMax, h);
        long long dc = rangeDistance(contMin, contMax, c);
        long long de = rangeDistance(eroMin, eroMax, e);
        long long dd = rangeDistance(depthMin, depthMax, d);
        long long dw = rangeDistance(weirdMin, weirdMax, w);
        return dt * dt + dh * dh + dc * dc + de * de + dd * dd + dw * dw + (long long)offset * offset;
    }

    // 区间外距离（64 位）
    static long long rangeDistance(long long min, long long max, long long noise) {
        long long l = noise - max;
        long long m = min - noise;
        return l > 0 ? l : (m > 0 ? m : 0);
    }
};

struct BiomeEntry {
    NoiseHypercube cube;
    std::string id; // "minecraft:plains"
    double temperature = 0.5;
};

class BiomeSource {
public:
    // 从 biome_params.json 加载（Java BiomeParamProbe 导出的 vanilla 运行时参数表）
    bool loadFromJson(const std::string& jsonText) {
        JsonParser parser(jsonText);
        JsonValue root = parser.parse();
        if (!root.isArray()) return false;
        for (auto& e : root.arr) {
            if (!e.isObject()) continue;
            const JsonValue* params = e.get("parameters");
            if (!params) continue;
            NoiseHypercube cube;
            parseRange(params->get("temperature"), cube.tempMin, cube.tempMax);
            parseRange(params->get("humidity"), cube.humMin, cube.humMax);
            parseRange(params->get("continentalness"), cube.contMin, cube.contMax);
            parseRange(params->get("erosion"), cube.eroMin, cube.eroMax);
            parseRange(params->get("depth"), cube.depthMin, cube.depthMax);
            parseRange(params->get("weirdness"), cube.weirdMin, cube.weirdMax);
            cube.offset = noiseToLong((float)params->num("offset", 0.0));
            const JsonValue* b = e.get("biome");
            entries.push_back({cube, b ? b->str() : "?", e.num("temperature", 0.5)});
        }
        return !entries.empty();
    }

    // biome 温度（用于 temperature() 条件：isCold = temp < 0.15）
    double temperature(const std::string& id) const {
        for (auto& e : entries) if (e.id == id) return e.temperature;
        return 0.5;
    }

    // 六维噪声值 → 最近 biome id（等价 vanilla MultiNoiseUtil.SearchTree.getValue，L146-152）
    // 非平局 = getValueSimple（唯一最近邻，与旧线性 find 结果一致）；平局 = 树序遍历第一个最小距离 leaf（对齐 vanilla）
    const std::string* find(float temp, float hum, float cont, float ero, float depth, float weird) const {
        long t = noiseToLong(temp), h = noiseToLong(hum), c = noiseToLong(cont);
        long e = noiseToLong(ero), d = noiseToLong(depth), w = noiseToLong(weird);
        debugFindTop(t, h, c, e, d, w);   // 方案 C 诊断（env 开关，不改结果）
        long point[SearchTree<std::string>::DIM] = {t, h, c, e, d, w, 0L};
        return searchTree().get(point);
    }

    size_t size() const { return entries.size(); }

private:
    std::vector<BiomeEntry> entries;
    mutable std::unique_ptr<SearchTree<std::string>> tree_;   // 首次 find 懒构建（SearchTree 平局语义）
    mutable std::once_flag treeOnce_;                          // 线程安全：spawn 预生成多线程并发首调 find → data race（2026-08-08 服务器崩溃根因）

    // 懒构建 SearchTree（树内容只依赖 entries，构建一次后复用；call_once 保证并发安全）
    const SearchTree<std::string>& searchTree() const {
        std::call_once(treeOnce_, [this] {
            std::vector<SearchTree<std::string>::Entry> es;
            es.reserve(entries.size());
            for (const auto& e : entries) {
                SearchTree<std::string>::Entry entry;
                entry.parameters[0] = STRange{e.cube.tempMin,    e.cube.tempMax};
                entry.parameters[1] = STRange{e.cube.humMin,     e.cube.humMax};
                entry.parameters[2] = STRange{e.cube.contMin,    e.cube.contMax};
                entry.parameters[3] = STRange{e.cube.eroMin,     e.cube.eroMax};
                entry.parameters[4] = STRange{e.cube.depthMin,   e.cube.depthMax};
                entry.parameters[5] = STRange{e.cube.weirdMin,   e.cube.weirdMax};
                entry.parameters[6] = STRange{e.cube.offset,     e.cube.offset};   // 第 7 维 [offset,offset]，点第 7 维恒 0
                entry.value = e.id;
                es.push_back(std::move(entry));
            }
            tree_ = std::make_unique<SearchTree<std::string>>(std::move(es));
            // 默认关闭 previousResult 缓存（确定性，平局=树序遍历第一个）；WG_SEARCHTREE_CACHE=1 复刻 Java 缓存语义（A/B 对照用）
            tree_->setUsePrevious(getenv("WG_SEARCHTREE_CACHE") != nullptr);
        });
        return *tree_;
    }

    // 方案 C 诊断：验证平局。WG_FINDTOP=任意值 → 打印 Top3 距离+id（含平局标记）；WG_FINDDUMP=任意值 → 打印全量距离。
    // 不改变 find 结果；仅当 env 存在时走线性遍历开销（诊断用）。
    void debugFindTop(long t, long h, long c, long e, long d, long w) const {
        static const bool top  = getenv("WG_FINDTOP")  != nullptr;
        static const bool dump = getenv("WG_FINDDUMP") != nullptr;
        if (!top && !dump) return;
        struct Hit { long long dist; const std::string* id; };
        Hit top3[3] = {{INT64_MAX, nullptr}, {INT64_MAX, nullptr}, {INT64_MAX, nullptr}};
        std::vector<Hit> all;
        if (dump) all.reserve(entries.size());
        for (const auto& entry : entries) {
            long long dist = entry.cube.getSquaredDistance(t, h, c, e, d, w);
            if (dump) all.push_back({dist, &entry.id});
            for (int i = 0; i < 3; i++) {   // 稳定 Top3（相等保留先出现）
                if (!top3[i].id || dist < top3[i].dist) {
                    for (int j = 2; j > i; j--) top3[j] = top3[j - 1];
                    top3[i] = {dist, &entry.id};
                    break;
                }
            }
        }
        std::fprintf(stderr, "[FIND] point t=%ld h=%ld c=%ld e=%ld d=%ld w=%ld\n", t, h, c, e, d, w);
        if (top) {
            for (int i = 0; i < 3 && top3[i].id; i++) {
                const char* tie = (i > 0 && top3[i].dist == top3[0].dist) ? "  <== TIE with #1" : "";
                std::fprintf(stderr, "  #%d %-36s dist=%lld%s\n", i + 1, top3[i].id->c_str(), (long long)top3[i].dist, tie);
            }
        }
        if (dump) {
            std::sort(all.begin(), all.end(), [](const Hit& a, const Hit& b) { return a.dist < b.dist; });
            for (const auto& hit : all)
                std::fprintf(stderr, "  %-36s dist=%lld\n", hit.id->c_str(), (long long)hit.dist);
        }
    }

    // 解析单个参数值（number、{min,max} 或 [min,max]）
    static void parseRange(const JsonValue* v, long& min, long& max) {
        if (!v) { min = max = 0; return; }
        if (v->isNumber()) {
            min = max = noiseToLong((float)v->num());
        } else if (v->isObject()) {
            min = noiseToLong((float)v->num("min", 0.0));
            max = noiseToLong((float)v->num("max", 0.0));
        } else if (v->isArray() && v->arr.size() >= 2) {
            min = noiseToLong((float)v->arr[0].num());
            max = noiseToLong((float)v->arr[1].num());
        } else {
            min = max = 0;
        }
    }
};

} // namespace wg
