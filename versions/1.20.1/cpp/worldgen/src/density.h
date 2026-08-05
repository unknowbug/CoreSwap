#pragma once
#include <cstdint>
#include <cmath>
#include <vector>
#include <memory>
#include <functional>
#include <algorithm>
#include "noise.h"

namespace wg {

// ===== NoisePos =====
struct NoisePos {
    int32_t x, y, z;
};

// ===== DensityFunction 抽象 =====
class DensityFunction {
public:
    virtual ~DensityFunction() = default;
    virtual double sample(const NoisePos& pos) const = 0;
    virtual double minValue() const = 0;
    virtual double maxValue() const = 0;
};
using DF = std::shared_ptr<DensityFunction>;

// ===== BinaryOperation 类型 =====
enum class BinOp { ADD, MUL, MIN, MAX };

// ===== LinearOperation：add/mul 折叠后（input * c 或 input + c）=====
class LinearOperation : public DensityFunction {
public:
    BinOp op;           // ADD 或 MUL
    DF input;
    double mn, mx, c;
    LinearOperation(BinOp op_, DF input_, double mn_, double mx_, double c_)
        : op(op_), input(std::move(input_)), mn(mn_), mx(mx_), c(c_) {}
    double sample(const NoisePos& pos) const override {
        double x = input->sample(pos);
        return op == BinOp::MUL ? x * c : x + c;
    }
    double minValue() const override { return mn; }
    double maxValue() const override { return mx; }
};

inline double clampD(double v, double lo, double hi) { return v < lo ? lo : (v > hi ? hi : v); }

// ===== Constant =====
class Constant : public DensityFunction {
public:
    double value;
    explicit Constant(double v) : value(v) {}
    double sample(const NoisePos&) const override { return value; }
    double minValue() const override { return value; }
    double maxValue() const override { return value; }
};

// ===== BinaryOperation (add/mul/min/max) =====
class BinaryOperation : public DensityFunction {
public:
    BinOp op;
    DF a, b;
    double mn, mx;
    BinaryOperation(BinOp op_, DF a_, DF b_, double mn_, double mx_)
        : op(op_), a(std::move(a_)), b(std::move(b_)), mn(mn_), mx(mx_) {}

    static DF create(BinOp op, DF a, DF b) {
        double d = a->minValue(), e = b->minValue();
        double f = a->maxValue(), g = b->maxValue();
        double h, i;
        switch (op) {
            case BinOp::ADD: h = d + e; i = f + g; break;
            case BinOp::MAX: h = std::max(d, e); i = std::max(f, g); break;
            case BinOp::MIN: h = std::min(d, e); i = std::min(f, g); break;
            case BinOp::MUL:
                h = d > 0.0 && e > 0.0 ? d * e : (f < 0.0 && g < 0.0 ? f * g : std::min(d * g, f * e));
                i = d > 0.0 && e > 0.0 ? f * g : (f < 0.0 && g < 0.0 ? d * e : std::max(d * e, f * g));
                break;
        }
        // 常量折叠：add/mul 带 Constant → LinearOperation（sample 等价：x*const 或 x+const）
        if (op == BinOp::ADD || op == BinOp::MUL) {
            double cval;
            DF input;
            if (auto c = std::dynamic_pointer_cast<Constant>(a)) { cval = c->value; input = b; }
            else if (auto c = std::dynamic_pointer_cast<Constant>(b)) { cval = c->value; input = a; }
            else { return std::make_shared<BinaryOperation>(op, a, b, h, i); }
            return std::make_shared<LinearOperation>(op, input, h, i, cval);
        }
        return std::make_shared<BinaryOperation>(op, a, b, h, i);
    }

    double sample(const NoisePos& pos) const override {
        double da = a->sample(pos);
        switch (op) {
            case BinOp::ADD: return da + b->sample(pos);
            case BinOp::MUL: return da == 0.0 ? 0.0 : da * b->sample(pos);
            case BinOp::MIN: return da < b->minValue() ? da : std::min(da, b->sample(pos));
            case BinOp::MAX: return da > b->maxValue() ? da : std::max(da, b->sample(pos));
        }
        return 0;
    }
    double minValue() const override { return mn; }
    double maxValue() const override { return mx; }
};

// ===== UnaryOperation =====
enum class UnaryOp { ABS, SQUARE, CUBE, HALF_NEGATIVE, QUARTER_NEGATIVE, SQUEEZE };

inline double applyUnary(UnaryOp op, double x) {
    switch (op) {
        case UnaryOp::ABS: return std::abs(x);
        case UnaryOp::SQUARE: return x * x;
        case UnaryOp::CUBE: return x * x * x;
        case UnaryOp::HALF_NEGATIVE: return x > 0.0 ? x : 0.5 * x;
        case UnaryOp::QUARTER_NEGATIVE: return x > 0.0 ? x : 0.25 * x;
        case UnaryOp::SQUEEZE: {
            double d = clampD(x, -1.0, 1.0);
            return d / 2.0 - d * d * d / 24.0;
        }
    }
    return 0;
}

class UnaryOperation : public DensityFunction {
public:
    UnaryOp op;
    DF input;
    double mn, mx;
    UnaryOperation(UnaryOp op_, DF input_, double mn_, double mx_)
        : op(op_), input(std::move(input_)), mn(mn_), mx(mx_) {}

    static DF create(UnaryOp op, DF input) {
        double imin = input->minValue(), imax = input->maxValue();
        double mn = applyUnary(op, imin), mx = applyUnary(op, imax);
        if (op == UnaryOp::ABS || op == UnaryOp::SQUARE) {
            mn = std::max(0.0, imin);
            mx = std::max(applyUnary(op, imin), applyUnary(op, imax));
        }
        if (mn > mx) std::swap(mn, mx);
        return std::make_shared<UnaryOperation>(op, input, mn, mx);
    }
    double sample(const NoisePos& pos) const override { return applyUnary(op, input->sample(pos)); }
    double minValue() const override { return mn; }
    double maxValue() const override { return mx; }
};

// ===== Clamp =====
class Clamp : public DensityFunction {
public:
    DF input;
    double mn, mx;
    Clamp(DF input_, double mn_, double mx_) : input(std::move(input_)), mn(mn_), mx(mx_) {}
    double sample(const NoisePos& pos) const override { return clampD(input->sample(pos), mn, mx); }
    double minValue() const override { return mn; }
    double maxValue() const override { return mx; }
};

// ===== Noise（带 sampler 注入）=====
class NoiseDF : public DensityFunction {
public:
    std::shared_ptr<DoublePerlinNoiseSampler> noise;
    double xzScale, yScale;
    NoiseDF(std::shared_ptr<DoublePerlinNoiseSampler> n, double xz, double y)
        : noise(std::move(n)), xzScale(xz), yScale(y) {}
    double sample(const NoisePos& pos) const override {
        if (!noise) return 0.0;
        return noise->sample(pos.x * xzScale, pos.y * yScale, pos.z * xzScale);
    }
    double minValue() const override { return -maxValue(); }
    double maxValue() const override { return noise ? noise->getMaxValue() : 2.0; }
};

// ===== Shift / ShiftA / ShiftB =====
class ShiftDF : public DensityFunction {
public:
    enum class Mode { SHIFT, SHIFT_A, SHIFT_B };
    std::shared_ptr<DoublePerlinNoiseSampler> noise;
    Mode mode;
    ShiftDF(std::shared_ptr<DoublePerlinNoiseSampler> n, Mode m) : noise(std::move(n)), mode(m) {}
    double sample(const NoisePos& pos) const override {
        if (!noise) return 0.0;
        double x, y, z;
        switch (mode) {
            case Mode::SHIFT: x = pos.x; y = pos.y; z = pos.z; break;
            case Mode::SHIFT_A: x = pos.x; y = 0.0; z = pos.z; break;
            case Mode::SHIFT_B: x = pos.z; y = pos.x; z = 0.0; break;
        }
        return noise->sample(x * 0.25, y * 0.25, z * 0.25) * 4.0;
    }
    double minValue() const override { return -maxValue(); }
    double maxValue() const override { return noise ? noise->getMaxValue() * 4.0 : 2.0; }
};

// ===== ShiftedNoise =====
class ShiftedNoiseDF : public DensityFunction {
public:
    DF shiftX, shiftY, shiftZ;
    double xzScale, yScale;
    std::shared_ptr<DoublePerlinNoiseSampler> noise;
    ShiftedNoiseDF(DF sx, DF sy, DF sz, double xz, double y, std::shared_ptr<DoublePerlinNoiseSampler> n)
        : shiftX(std::move(sx)), shiftY(std::move(sy)), shiftZ(std::move(sz)), xzScale(xz), yScale(y), noise(std::move(n)) {}
    double sample(const NoisePos& pos) const override {
        double d = pos.x * xzScale + shiftX->sample(pos);
        double e = pos.y * yScale + shiftY->sample(pos);
        double f = pos.z * xzScale + shiftZ->sample(pos);
        return noise ? noise->sample(d, e, f) : 0.0;
    }
    double minValue() const override { return -maxValue(); }
    double maxValue() const override { return noise ? noise->getMaxValue() : 2.0; }
};

// ===== RangeChoice =====
class RangeChoice : public DensityFunction {
public:
    DF input, inRange, outOfRange;
    double minInclusive, maxExclusive;
    RangeChoice(DF input_, double minIn_, double maxEx_, DF in_, DF out_)
        : input(std::move(input_)), inRange(std::move(in_)), outOfRange(std::move(out_)),
          minInclusive(minIn_), maxExclusive(maxEx_) {}
    double sample(const NoisePos& pos) const override {
        double d = input->sample(pos);
        return (minInclusive <= d && d < maxExclusive) ? inRange->sample(pos) : outOfRange->sample(pos);
    }
    double minValue() const override {
        return std::min(inRange->minValue(), outOfRange->minValue());
    }
    double maxValue() const override {
        return std::max(inRange->maxValue(), outOfRange->maxValue());
    }
};

// ===== YClampedGradient =====
class YClampedGradient : public DensityFunction {
public:
    int32_t fromY, toY;
    double fromValue, toValue;
    YClampedGradient(int32_t fy, int32_t ty, double fv, double tv)
        : fromY(fy), toY(ty), fromValue(fv), toValue(tv) {}

    static double clampedMap(double v, int32_t a, int32_t b, double c, double d) {
        if (a == b) return (c + d) / 2.0;
        if (v < a) return c;
        if (v > b) return d;
        return c + (v - a) / (double)(b - a) * (d - c);
    }
    double sample(const NoisePos& pos) const override {
        return clampedMap(pos.y, fromY, toY, fromValue, toValue);
    }
    double minValue() const override { return std::min(fromValue, toValue); }
    double maxValue() const override { return std::max(fromValue, toValue); }
};

// ===== WeirdScaledSampler =====
class WeirdScaledSampler : public DensityFunction {
public:
    enum class Rarity { TUNNELS, CAVES };
    DF input;
    std::shared_ptr<DoublePerlinNoiseSampler> noise;
    Rarity rarity;
    WeirdScaledSampler(DF input_, std::shared_ptr<DoublePerlinNoiseSampler> n, Rarity r)
        : input(std::move(input_)), noise(std::move(n)), rarity(r) {}

    static double scaleValue(Rarity r, double v) {
        if (r == Rarity::CAVES) {
            if (v < -0.75) return 0.5;
            if (v < -0.5) return 0.75;
            if (v < 0.5) return 1.0;
            return v < 0.75 ? 2.0 : 3.0;
        } else {
            if (v < -0.5) return 0.75;
            if (v < 0.0) return 1.0;
            return v < 0.5 ? 1.5 : 2.0;
        }
    }
    double sample(const NoisePos& pos) const override {
        double d = scaleValue(rarity, input->sample(pos));
        if (!noise) return 0.0;
        return d * std::abs(noise->sample(pos.x / d, pos.y / d, pos.z / d));
    }
    double minValue() const override { return 0.0; }
    double maxValue() const override {
        double mult = rarity == Rarity::CAVES ? 3.0 : 2.0;
        return noise ? mult * noise->getMaxValue() : 2.0;
    }
};

// ===== InterpolatedNoiseSampler（old_blended_noise）=====
class InterpolatedNoiseDF : public DensityFunction {
public:
    OctavePerlinNoiseSampler lower, upper, interpolation;
    double xzScale, yScale, xzFactor, yFactor, smearScaleMultiplier;
    double scaledXzScale, scaledYScale, maxVal;

    InterpolatedNoiseDF(XoroshiroRandom& random, double xzS, double yS, double xzF, double yF, double smear)
        : lower(random, true, -15, OctavePerlinNoiseSampler::rangeClosedAmplitudes(-15, 0)),
          upper(random, true, -15, OctavePerlinNoiseSampler::rangeClosedAmplitudes(-15, 0)),
          interpolation(random, true, -7, OctavePerlinNoiseSampler::rangeClosedAmplitudes(-7, 0)),
          xzScale(xzS), yScale(yS), xzFactor(xzF), yFactor(yF), smearScaleMultiplier(smear) {
        scaledXzScale = 684.412 * xzScale;
        scaledYScale = 684.412 * yScale;
        maxVal = lower.method_40556(scaledYScale);
    }

    double sample(const NoisePos& pos) const override {
        double d = pos.x * scaledXzScale;
        double e = pos.y * scaledYScale;
        double f = pos.z * scaledXzScale;
        double g = d / xzFactor;
        double h = e / yFactor;
        double i = f / xzFactor;
        double j = scaledYScale * smearScaleMultiplier;
        double k = j / yFactor;
        double l = 0.0, m = 0.0, n = 0.0;
        double o = 1.0;
        for (int p = 0; p < 8; p++) {
            const PerlinNoiseSampler* pn = interpolation.getOctave(p);
            if (pn) {
                n += pn->sample(
                         OctavePerlinNoiseSampler::maintainPrecision(g * o),
                         OctavePerlinNoiseSampler::maintainPrecision(h * o),
                         OctavePerlinNoiseSampler::maintainPrecision(i * o),
                         k * o, h * o) / o;
            }
            o /= 2.0;
        }
        double q = (n / 10.0 + 1.0) / 2.0;
        bool bl2 = q >= 1.0;
        bool bl3 = q <= 0.0;
        o = 1.0;
        for (int r = 0; r < 16; r++) {
            double s = OctavePerlinNoiseSampler::maintainPrecision(d * o);
            double t = OctavePerlinNoiseSampler::maintainPrecision(e * o);
            double u = OctavePerlinNoiseSampler::maintainPrecision(f * o);
            double v = j * o;
            if (!bl2) {
                const PerlinNoiseSampler* pn = lower.getOctave(r);
                if (pn) l += pn->sample(s, t, u, v, e * o) / o;
            }
            if (!bl3) {
                const PerlinNoiseSampler* pn = upper.getOctave(r);
                if (pn) m += pn->sample(s, t, u, v, e * o) / o;
            }
            o /= 2.0;
        }
        // clampedLerp(l/512, m/512, q) / 128
        double qq = clampD(q, 0.0, 1.0);
        return (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0;
    }
    double minValue() const override { return -maxValue(); }
    double maxValue() const override { return maxVal; }
};

// ===== InterpolatedDF（minecraft:interpolated）：NoiseChunk cell 插值（4×4×8）=====
// vanilla 语义：fillFromNoise 对该函数做 cell 角点采样 + 三线性插值（高频噪声防 alias）
// 实现：lazy 按 chunk 缓存网格（单线程 POC；构建成本 5×49×5 点采样）
class InterpolatedDF : public DensityFunction {
public:
    DF arg;
    static constexpr int CELL_X = 4, CELL_Y = 8, CELL_Z = 4;
    static constexpr int MIN_Y = -64, HEIGHT = 384;

    explicit InterpolatedDF(DF a) : arg(std::move(a)) {}

    double sample(const NoisePos& pos) const override {
        int chunkX = floorDivP(pos.x, 16);
        int chunkZ = floorDivP(pos.z, 16);
        int64_t key = ((int64_t)(uint32_t)chunkX << 32) ^ (uint32_t)chunkZ;
        if (key != cachedKey) {
            buildGrid(chunkX, chunkZ);
            cachedKey = key;
        }
        constexpr int GX = 16 / CELL_X + 1, GY = HEIGHT / CELL_Y + 1, GZ = 16 / CELL_Z + 1;
        int gx = pos.x - chunkX * 16;         // 0..15
        int gy = pos.y - MIN_Y;               // 0..383
        int gz = pos.z - chunkZ * 16;
        int cx = gx / CELL_X, cy = gy / CELL_Y, cz = gz / CELL_Z;
        // 越界保护：chunk 外坐标（如 aquifer 边界扫描 / 世界顶 y=320）→ clamp
        // 注意需保证 cx+dx ≤ GX-1（三线性插值访问 +1）
        if (cx < 0 || cy < 0 || cz < 0 || cx >= GX || cy >= GY || cz >= GZ ||
            cx >= GX - 1 || cy >= GY - 1 || cz >= GZ - 1) {
            static bool warned = false;
            if (!warned) {
                std::fprintf(stderr, "[InterpolatedDF] OOB pos=(%d,%d,%d) chunk=(%d,%d) g=(%d,%d,%d) cell=(%d,%d,%d)\n",
                             pos.x, pos.y, pos.z, chunkX, chunkZ, gx, gy, gz, cx, cy, cz);
                warned = true;
            }
            cx = cx < 0 ? 0 : (cx > GX - 2 ? GX - 2 : cx);
            cy = cy < 0 ? 0 : (cy > GY - 2 ? GY - 2 : cy);
            cz = cz < 0 ? 0 : (cz > GZ - 2 ? GZ - 2 : cz);
        }
        double fx = (gx % CELL_X) / (double)CELL_X;
        double fy = (gy % CELL_Y) / (double)CELL_Y;
        double fz = (gz % CELL_Z) / (double)CELL_Z;
        auto g = [&](int dx, int dy, int dz) {
            return grid[((size_t)(cy + dy) * GZ + (cz + dz)) * GX + (cx + dx)];
        };
        double d000 = g(0, 0, 0), d100 = g(1, 0, 0), d010 = g(0, 1, 0), d110 = g(1, 1, 0);
        double d001 = g(0, 0, 1), d101 = g(1, 0, 1), d011 = g(0, 1, 1), d111 = g(1, 1, 1);
        double d00 = d000 + (d100 - d000) * fx;
        double d10 = d010 + (d110 - d010) * fx;
        double d01 = d001 + (d101 - d001) * fx;
        double d11 = d011 + (d111 - d011) * fx;
        double d0 = d00 + (d10 - d00) * fy;
        double d1 = d01 + (d11 - d01) * fy;
        return d0 + (d1 - d0) * fz;
    }

    double minValue() const override { return arg->minValue(); }
    double maxValue() const override { return arg->maxValue(); }

private:
    mutable int64_t cachedKey = INT64_MIN;
    mutable std::vector<double> grid;

    static int floorDivP(int a, int b) { int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }

    void buildGrid(int chunkX, int chunkZ) const {
        constexpr int GX = 16 / CELL_X + 1, GY = HEIGHT / CELL_Y + 1, GZ = 16 / CELL_Z + 1;
        grid.assign((size_t)GX * GY * GZ, 0.0);
        NoisePos p;
        for (int gy = 0; gy < GY; gy++)
            for (int gz = 0; gz < GZ; gz++)
                for (int gx = 0; gx < GX; gx++) {
                    p.x = chunkX * 16 + gx * CELL_X;
                    p.y = MIN_Y + gy * CELL_Y;
                    p.z = chunkZ * 16 + gz * CELL_Z;
                    grid[((size_t)gy * GZ + gz) * GX + gx] = arg->sample(p);
                }
    }
};

// ===== Blend（新世界 NoBlending）=====
class BlendAlpha : public DensityFunction {
public:
    double sample(const NoisePos&) const override { return 1.0; }
    double minValue() const override { return 1.0; }
    double maxValue() const override { return 1.0; }
};
class BlendOffset : public DensityFunction {
public:
    double sample(const NoisePos&) const override { return 0.0; }
    double minValue() const override { return 0.0; }
    double maxValue() const override { return 0.0; }
};
class BlendDensityDF : public DensityFunction {
public:
    DF input;
    explicit BlendDensityDF(DF i) : input(std::move(i)) {}
    double sample(const NoisePos& pos) const override { return input->sample(pos); } // NoBlending 恒等
    double minValue() const override { return input->minValue(); }
    double maxValue() const override { return input->maxValue(); }
};

// ===== 包装（interpolated/cache 等，语义上委托）=====
class WrappingDF : public DensityFunction {
public:
    DF wrapped;
    explicit WrappingDF(DF w) : wrapped(std::move(w)) {}
    double sample(const NoisePos& pos) const override { return wrapped->sample(pos); }
    double minValue() const override { return wrapped->minValue(); }
    double maxValue() const override { return wrapped->maxValue(); }
};

// ===== Spline（1.20.1 Hermite 插值）=====
class SplineDF : public DensityFunction {
public:
    // 位置函数 + 位置点 + 子样条 + 导数
    std::shared_ptr<DensityFunction> locationFunction;
    std::vector<float> locations;
    std::vector<std::shared_ptr<SplineDF>> subSplines; // 可能为叶子（固定值）或嵌套
    std::vector<float> derivatives;
    // 叶子：values 是固定值
    bool isLeaf;
    float fixedValue;

    SplineDF() : isLeaf(true), fixedValue(0) {}

    double sample(const NoisePos& pos) const override {
        if (isLeaf) return fixedValue;
        double f = locationFunction->sample(pos);
        return apply(f, pos);
    }

    double apply(double f, const NoisePos& pos) const {
        size_t n = locations.size();
        if (n == 1) return sampleOutsideRange(f, pos, 0);
        // Java: i = binarySearch(0, n, f < locations[i]) - 1
        size_t lo = 0, hi = n;
        while (lo < hi) {
            size_t mid = (lo + hi) / 2;
            if (f < locations[mid]) hi = mid; else lo = mid + 1;
        }
        int64_t i = (int64_t)lo - 1;
        if (i < 0) return sampleOutsideRange(f, pos, 0);                 // f < locations[0]
        if (i == (int64_t)n - 1) return sampleOutsideRange(f, pos, (size_t)i); // f >= locations[n-1]
        size_t k = (size_t)i;
        float g = locations[k], h = locations[k + 1];
        double kd = (f - g) / (double)(h - g);
        double nv = subSplines[k] ? subSplines[k]->sample(pos) : 0.0;
        double ov = subSplines[k + 1] ? subSplines[k + 1]->sample(pos) : 0.0;
        float l = derivatives[k], m = derivatives[k + 1];
        double p = l * (h - g) - (ov - nv);
        double q = -m * (h - g) + (ov - nv);
        return lerp(kd, nv, ov) + kd * (1.0 - kd) * lerp(kd, p, q);
    }

    double sampleOutsideRange(double f, const NoisePos& pos, size_t i) const {
        // Java: index==0 → subSplines.get(0)；否则 → subSplines.get(size-1)（i=n-1 时越界，需换算）
        size_t idx = (i == 0) ? 0 : (subSplines.size() - 1);
        float d = derivatives[idx];
        double base = subSplines[idx] ? subSplines[idx]->sample(pos) : (double)fixedValue;
        return base + d * (f - locations[idx]);
    }

    double minValue() const override { return isLeaf ? fixedValue : computeMin(); }
    double maxValue() const override { return isLeaf ? fixedValue : computeMax(); }

private:
    double computeMin() const {
        double mn = std::numeric_limits<double>::infinity();
        for (auto& s : subSplines) mn = std::min(mn, s ? s->minValue() : fixedValue);
        return mn;
    }
    double computeMax() const {
        double mx = -std::numeric_limits<double>::infinity();
        for (auto& s : subSplines) mx = std::max(mx, s ? s->maxValue() : fixedValue);
        return mx;
    }
};

} // namespace wg
