// biome.h — MultiNoiseBiomeSource 复刻：六维噪声参数 → 最近 biome
// 查找等价 vanilla MultiNoiseUtil.getSquaredDistance 遍历全部 biome（SearchTree 仅是加速，结果相同）
#pragma once
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <algorithm>

#include "json.h"

namespace wg {

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

    // ParameterRange.getDistance(noise)：区间外距离
    static long rangeDistance(long min, long max, long noise) {
        long l = noise - max;
        long m = min - noise;
        return l > 0 ? l : (m > 0 ? m : 0);
    }

    long getSquaredDistance(long t, long h, long c, long e, long d, long w) const {
        long dt = rangeDistance(tempMin, tempMax, t);
        long dh = rangeDistance(humMin, humMax, h);
        long dc = rangeDistance(contMin, contMax, c);
        long de = rangeDistance(eroMin, eroMax, e);
        long dd = rangeDistance(depthMin, depthMax, d);
        long dw = rangeDistance(weirdMin, weirdMax, w);
        return dt * dt + dh * dh + dc * dc + de * de + dd * dd + dw * dw + offset * offset;
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

    // 六维噪声值 → 最近 biome id（等价 vanilla getValueSimple）
    const std::string* find(float temp, float hum, float cont, float ero, float depth, float weird) const {
        long t = noiseToLong(temp), h = noiseToLong(hum), c = noiseToLong(cont);
        long e = noiseToLong(ero), d = noiseToLong(depth), w = noiseToLong(weird);
        const std::string* best = nullptr;
        long bestDist = -1;
        for (const auto& entry : entries) {
            long dist = entry.cube.getSquaredDistance(t, h, c, e, d, w);
            if (bestDist < 0 || dist < bestDist) {
                bestDist = dist;
                best = &entry.id;
            }
        }
        return best;
    }

    size_t size() const { return entries.size(); }

private:
    std::vector<BiomeEntry> entries;

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
