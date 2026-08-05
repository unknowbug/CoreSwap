#include "worldgen.h"

#include <cstdint>

namespace wg {

std::int64_t probe(std::int64_t seed, std::int32_t x, std::int32_t z) {
    // 确定性散列，仅用于验证通路（Phase 3 会换成真正的噪声核心）
    std::uint64_t h = static_cast<std::uint64_t>(seed) ^ 0x9E3779B97F4A7C15ULL;
    h ^= static_cast<std::uint64_t>(static_cast<std::int64_t>(x) * 374761393);
    h ^= static_cast<std::uint64_t>(static_cast<std::int64_t>(z) * 668265263);
    h = (h ^ (h >> 30)) * 0xBF58476D1CE4E5B9ULL;
    h = (h ^ (h >> 27)) * 0x94D049BB133111EBULL;
    return static_cast<std::int64_t>(h ^ (h >> 31));
}

} // namespace wg
