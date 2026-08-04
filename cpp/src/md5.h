#pragma once
#include <cstdint>
#include <string>
#include <array>
#include <vector>
#include <cstddef>

namespace wg {

// RFC 1321 MD5（Mojang RandomSeed.createXoroshiroSeed(String) 依赖）
std::array<uint8_t, 16> md5(const uint8_t* data, size_t len);
inline std::array<uint8_t, 16> md5(const std::string& s) { return md5(reinterpret_cast<const uint8_t*>(s.data()), s.size()); }

} // namespace wg
