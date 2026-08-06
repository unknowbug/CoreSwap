// blocks.h — 方块 ID 表 + 区块方块存储（16×16×384）
#pragma once
#include <cstdint>
#include <map>
#include <string>
#include <vector>

#include "json.h"

namespace wg {

constexpr int BLOCK_MIN_Y = -64;   // 默认主世界（向后兼容；实际维度参数走 DimConfig）
constexpr int BLOCK_HEIGHT = 384;
constexpr int BLOCK_16 = 16;
constexpr int BLOCK_COUNT = BLOCK_16 * BLOCK_16 * BLOCK_HEIGHT; // 98304（主世界）

// 维度配置（通用引擎：每个维度一套参数，wg_create 按维度初始化）
struct DimConfig {
    int minY = -64;           // 世界 minY（overworld -64 / nether 0）
    int worldHeight = 384;    // 世界高度（buffer/写入范围；overworld 384 / nether 256）
    int noiseHeight = 384;    // 噪声高度（density 采样/插值范围；overworld 384 / nether 128）
    bool aquifersEnabled = true;
    std::string noiseSettingsFile = "overworld.json";
    std::string biomeParamsFile = "biome_params.json";
};

// 方块 ID：使用 vanilla block 注册表 raw id（从 blocks.json 加载，Java 侧同表）
using BlockId = int32_t;

class BlockRegistry {
public:
    // 从 blocks.json（{ "minecraft:stone": 1, ... }）加载 id→name 双向表
    bool loadFromJson(const std::string& jsonText) {
        JsonParser parser(jsonText);
        JsonValue root = parser.parse();
        if (!root.isObject()) return false;
        for (auto& [k, v] : root.obj) {
            if (v.isNumber()) nameToId[k] = (int)v.numVal;
        }
        idToName.assign(16384, "");
        for (auto& [k, id] : nameToId) {
            if (id >= 0 && id < (int)idToName.size()) idToName[id] = k;
        }
        return !nameToId.empty();
    }
    BlockId id(const std::string& name) const {
        auto it = nameToId.find(name);
        return it == nameToId.end() ? AIR : it->second;
    }
    const std::string& name(BlockId id) const {
        static const std::string unknown = "?";
        if (id >= 0 && id < (int)idToName.size() && !idToName[id].empty()) return idToName[id];
        return unknown;
    }
    bool contains(const std::string& name) const { return nameToId.count(name) > 0; }

    static constexpr BlockId AIR = 0;

private:
    std::map<std::string, BlockId> nameToId;
    std::vector<std::string> idToName;
};

// 区块方块列：16×16×height，index = (y - minY) * 256 + z * 16 + x（维度参数化）
class BlockColumn {
public:
    explicit BlockColumn(int minY_ = BLOCK_MIN_Y, int height_ = BLOCK_HEIGHT)
        : minY(minY_), height(height_), blocks((size_t)BLOCK_16 * BLOCK_16 * height_, BlockRegistry::AIR) {}
    BlockId& at(int x, int y, int z) { return blocks[((size_t)(y - minY) * 256) + z * 16 + x]; }
    BlockId at(int x, int y, int z) const { return blocks[((size_t)(y - minY) * 256) + z * 16 + x]; }
    const std::vector<BlockId>& data() const { return blocks; }

private:
    int minY, height;
    std::vector<BlockId> blocks;
};

} // namespace wg
