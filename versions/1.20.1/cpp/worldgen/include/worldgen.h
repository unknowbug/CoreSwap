#pragma once
#include <cstdint>

namespace wg {

// JNI hello-world 用最小接口。
// 返回基于 seed/坐标的确定性 64 位值，仅验证 JNI 数据通路。
std::int64_t probe(std::int64_t seed, std::int32_t x, std::int32_t z);

} // namespace wg
