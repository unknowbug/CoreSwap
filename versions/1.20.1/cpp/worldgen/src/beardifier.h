// beardifier.h — StructureWeightSampler（Beardifier）结构密度修正
//
// Java 参考：net/minecraft/world/gen/StructureWeightSampler.java（1.20.1）
// 机制：ChunkNoiseSampler.getActualDensityFunction L469-470 将 DensityFunctionTypes.Beardifier.INSTANCE
//       替换为真实 beardifying（StructureWeightSampler）→ density 链 = add(finalDensity, Beardifier)
//       （CellCache 语义，见 worldgen_api.cpp L570 注释）
// 输入（pieces/junctions）由 Java 侧 vanilla 机制构造（createStructureWeightSampler）喂入，
// C++ 只移植纯算法（24^3 权重表 + sample 四分支 + fastInverseSqrt 位操作逐位对齐）。
#pragma once
#include <cstdint>
#include <cstring>
#include <vector>
#include <array>
#include <cmath>

namespace wg {

// ===== 结构地形适配枚举（StructureTerrainAdaptation，序数 = Java ordinal）=====
enum class TerrainAdaptation : int32_t {
    NONE = 0,
    BURY = 1,
    BEARD_THIN = 2,
    BEARD_BOX = 3,
};

// ===== Piece（StructureWeightSampler.Piece）=====
// box: BlockBox（minX/minY/minZ/maxX/maxY/maxZ，均为含边界）
struct BeardPiece {
    int32_t minX, minY, minZ, maxX, maxY, maxZ;
    TerrainAdaptation terrain;
    int32_t groundLevelDelta;
};

// ===== JigsawJunction（仅 sample 用到的三元组）=====
struct BeardJunction {
    int32_t sourceX;
    int32_t sourceGroundY;
    int32_t sourceZ;
};

// ===== MathHelper 逐位等价（仅本文件需要）=====
inline double beard_squaredMagnitude(double a, double b, double c) {
    return a * a + b * b + c * c;
}

inline double beard_magnitude(double a, double b, double c) {
    return std::sqrt(a * a + b * b + c * c);
}

// MathHelper.fastInverseSqrt（L517-523）：位操作近似 1/sqrt(x)，Newton 一步迭代
// 注意：Java long 有符号算术右移 >>，MSVC long=32 位 → 必须 int64_t/long long
inline double beard_fastInverseSqrt(double x) {
    double d = 0.5 * x;
    int64_t l;
    std::memcpy(&l, &x, 8);              // Double.doubleToRawLongBits
    l = 6910469410427058090LL - (l >> 1);
    std::memcpy(&x, &l, 8);              // Double.longBitsToDouble
    return x * (1.5 - d * x * x);
}

// MathHelper.lerp → clampedLerp → clampedMap 链（getLerpProgress=(value-start)/(end-start)）
inline double beard_clampedMap(double value, double oldStart, double oldEnd, double newStart, double newEnd) {
    double delta = (value - oldStart) / (oldEnd - oldStart);   // getLerpProgress
    // clampedLerp(newStart, newEnd, delta)
    if (delta < 0.0) return newStart;
    if (delta > 1.0) return newEnd;
    return newStart + delta * (newEnd - newStart);             // lerp
}

// ===== Beardifier（StructureWeightSampler）=====
class Beardifier {
public:
    static constexpr int INDEX_OFFSET = 12;
    static constexpr int EDGE_LENGTH = 24;

    std::vector<BeardPiece> pieces;
    std::vector<BeardJunction> junctions;

    // 24^3 权重表（Java static final float[13824]）
    // array[i*576 + j*24 + k] = (float)calculateStructureWeight(j-12, k-12, i-12)
    // sample 索引：STRUCTURE_WEIGHT_TABLE[k*576 + i*24 + j]（k=z+12, i=x+12, j=y+12）
    static const std::array<float, EDGE_LENGTH * EDGE_LENGTH * EDGE_LENGTH>& weightTable() {
        static const auto table = [] {
            std::array<float, EDGE_LENGTH * EDGE_LENGTH * EDGE_LENGTH> arr{};
            for (int i = 0; i < EDGE_LENGTH; i++) {
                for (int j = 0; j < EDGE_LENGTH; j++) {
                    for (int k = 0; k < EDGE_LENGTH; k++) {
                        arr[i * 24 * 24 + j * 24 + k] =
                            (float)calculateStructureWeight(j - 12, k - 12, i - 12);
                    }
                }
            }
            return arr;
        }();
        return table;
    }

    // calculateStructureWeight(x, y, z) = structureWeight(x, y+0.5, z)
    // structureWeight(x, y, z) = Math.pow(Math.E, -squaredMagnitude(x,y,z)/16.0)
    // ⚠️ Java 用 Math.pow(Math.E, ...)（fdlibm pow 通用路径）非 Math.exp——C++ 保持字面同语义 std::pow(M_E, ...)
    // @anchor.test("Beardifier 权重表生成对齐 Java StructureWeightSampler.calculateStructureWeight（pow 语义 + float 截断）", source="probe:block_probe!BEARD244#005")
    static double calculateStructureWeight(int x, int y, int z) {
        double d = beard_squaredMagnitude((double)x, (double)y + 0.5, (double)z);
        // Java Math.E = 2.718281828459045（double 位级）；用字面量避开 MSVC M_E 宏依赖
        return std::pow(2.718281828459045, -d / 16.0);
    }

    // getMagnitudeWeight(x, y, z) = clampedMap(magnitude(x, y/2.0, z), 0, 6, 1, 0)
    static double getMagnitudeWeight(int x, int y, int z) {
        double d = beard_magnitude((double)x, (double)y / 2.0, (double)z);
        return beard_clampedMap(d, 0.0, 6.0, 1.0, 0.0);
    }

    // getStructureWeight(x, y, z, yy)：表查找（越界 0）+ fastInverseSqrt 因子
    // @anchor.test("Beardifier sample 权重计算对齐 Java getStructureWeight（表索引 + fastInverseSqrt 因子）", source="probe:block_probe!BEARD244#005")
    static double getStructureWeight(int x, int y, int z, int yy) {
        int i = x + 12;
        int j = y + 12;
        int k = z + 12;
        if (i >= 0 && i < 24 && j >= 0 && j < 24 && k >= 0 && k < 24) {
            double d = (double)yy + 0.5;
            double e = beard_squaredMagnitude((double)x, d, (double)z);
            double f = -d * beard_fastInverseSqrt(e / 2.0) / 2.0;
            return f * weightTable()[k * 24 * 24 + i * 24 + j];
        }
        return 0.0;
    }

    // sample(pos)：pieces 累加 + junctions 累加（Java 用 iterator.back 重置 = 每次从头遍历）
    // @anchor.test("Beardifier sample 逐位对齐 Java StructureWeightSampler.sample（BEARD-244 8 点 y=55..62）", source="probe:block_probe!BEARD244#005")
    double sample(int32_t blockX, int32_t blockY, int32_t blockZ) const {
        double d = 0.0;
        for (const BeardPiece& piece : pieces) {
            int32_t l = piece.groundLevelDelta;
            int32_t m = std::max(0, std::max(piece.minX - blockX, blockX - piece.maxX));
            int32_t n = std::max(0, std::max(piece.minZ - blockZ, blockZ - piece.maxZ));
            int32_t o = piece.minY + l;
            int32_t p = blockY - o;
            int32_t q;
            switch (piece.terrain) {
                case TerrainAdaptation::NONE: q = 0; break;
                case TerrainAdaptation::BURY:
                case TerrainAdaptation::BEARD_THIN: q = p; break;
                case TerrainAdaptation::BEARD_BOX: q = std::max(0, std::max(o - blockY, blockY - piece.maxY)); break;
            }
            switch (piece.terrain) {
                case TerrainAdaptation::NONE: break;
                case TerrainAdaptation::BURY: d += getMagnitudeWeight(m, q, n); break;
                case TerrainAdaptation::BEARD_THIN:
                case TerrainAdaptation::BEARD_BOX: d += getStructureWeight(m, q, n, p) * 0.8; break;
            }
        }
        for (const BeardJunction& jj : junctions) {
            int32_t r = blockX - jj.sourceX;
            int32_t l = blockY - jj.sourceGroundY;
            int32_t m = blockZ - jj.sourceZ;
            d += getStructureWeight(r, l, m, l) * 0.4;
        }
        return d;
    }

    bool empty() const { return pieces.empty() && junctions.empty(); }
};

} // namespace wg
