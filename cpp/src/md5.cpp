#include "md5.h"
#include <cstring>

namespace wg {

namespace {
constexpr uint32_t S[64] = {
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21};
constexpr uint32_t K[64] = {
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391};
constexpr uint32_t rotl(uint32_t x, int c) { return (x << c) | (x >> (32 - c)); }
} // namespace

std::array<uint8_t, 16> md5(const uint8_t* data, size_t len) {
    uint32_t a0 = 0x67452301, b0 = 0xefcdab89, c0 = 0x98badcfe, d0 = 0x10325476;
    size_t new_len = ((len + 8) / 64 + 1) * 64;
    std::vector<uint8_t> buf(new_len, 0);
    std::memcpy(buf.data(), data, len);
    buf[len] = 0x80;
    uint64_t bit_len = static_cast<uint64_t>(len) * 8;
    for (int i = 0; i < 8; i++) buf[new_len - 8 + i] = static_cast<uint8_t>(bit_len >> (8 * i));

    for (size_t off = 0; off < new_len; off += 64) {
        uint32_t M[16];
        for (int i = 0; i < 16; i++) {
            M[i] = static_cast<uint32_t>(buf[off + 4 * i]) |
                   (static_cast<uint32_t>(buf[off + 4 * i + 1]) << 8) |
                   (static_cast<uint32_t>(buf[off + 4 * i + 2]) << 16) |
                   (static_cast<uint32_t>(buf[off + 4 * i + 3]) << 24);
        }
        uint32_t A = a0, B = b0, C = c0, D = d0;
        for (int i = 0; i < 64; i++) {
            uint32_t F;
            int g;
            if (i < 16) { F = (B & C) | (~B & D); g = i; }
            else if (i < 32) { F = (D & B) | (~D & C); g = (5 * i + 1) % 16; }
            else if (i < 48) { F = B ^ C ^ D; g = (3 * i + 5) % 16; }
            else { F = C ^ (B | ~D); g = (7 * i) % 16; }
            uint32_t tmp = D;
            D = C;
            C = B;
            B = B + rotl(A + F + K[i] + M[g], S[i]);
            A = tmp;
        }
        a0 += A; b0 += B; c0 += C; d0 += D;
    }
    std::array<uint8_t, 16> out;
    auto put = [&](size_t idx, uint32_t v) {
        out[idx] = static_cast<uint8_t>(v);
        out[idx + 1] = static_cast<uint8_t>(v >> 8);
        out[idx + 2] = static_cast<uint8_t>(v >> 16);
        out[idx + 3] = static_cast<uint8_t>(v >> 24);
    };
    put(0, a0); put(4, b0); put(8, c0); put(12, d0);
    return out;
}

} // namespace wg
