#pragma once
#include <cstdint>
#include <cmath>
#include <vector>
#include <memory>
#include <functional>
#include <algorithm>
#include <atomic>
#include <cstdlib>
#include <mutex>
#include <chrono>
#include "noise.h"

// 剖析计数（WG_PROFILE=1 启用；C++17 inline 变量：多 TU 单一实体）
inline bool wg_profEnabled = false;
inline bool wg_splineDebug = false;
inline bool wg_surfaceTrace = false;
inline bool wg_aqfDump = false;
inline int wg_surfaceTraceX = 804;
inline int wg_surfaceTraceZ = -368;
inline int wg_aqfYMin = 55;
inline int wg_aqfYMax = 62;
inline std::atomic<int64_t> wg_profNoiseDF{0};
inline std::atomic<int64_t> wg_profSpline{0};
inline std::atomic<int64_t> wg_profAquiferDeep{0};
inline std::atomic<int64_t> wg_profBiomeAt{0};
inline std::atomic<int64_t> wg_profInterpGrid{0};
// 耗时（ns，WG_PROFILE 下累计；决定“noise 是否热点”的基石证据）
inline std::atomic<int64_t> wg_profNoiseNs{0};
inline std::atomic<int64_t> wg_profSplineNs{0};

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
        double r;
        switch (op) {
            case BinOp::ADD: r = da + b->sample(pos); break;
            case BinOp::MUL: r = da == 0.0 ? 0.0 : da * b->sample(pos); break;
            case BinOp::MIN: r = da < b->minValue() ? da : std::min(da, b->sample(pos)); break;
            case BinOp::MAX: {
                double bmax = b->maxValue();
                double bv = b->sample(pos);
                r = da > bmax ? da : std::max(da, bv);
                if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
                    std::fprintf(stderr, "[MAXDBG] pos=(%d,%d,%d) da=%.6f bmax=%.6f bv=%.6f -> %.6f\n", pos.x, pos.y, pos.z, da, bmax, bv, r);
                }
                break;
            }
            default: r = 0;
        }
        if (wg_splineDebug && (r < -900000.0 || r > 900000.0)) {
            std::fprintf(stderr, "[BINOP] pos=(%d,%d,%d) op=%d a=%.6f b=%.6f -> %.6f\n",
                         pos.x, pos.y, pos.z, (int)op, da, (op == BinOp::MIN && da < b->minValue()) ? b->minValue() : b->sample(pos), r);
        }
        if (wg_splineDebug && op == BinOp::MIN && (pos.y == -8 || pos.y == 0)) {
            std::fprintf(stderr, "[MIN] pos=(%d,%d,%d) a=%.6f bmin=%.6f -> %.6f\n",
                         pos.x, pos.y, pos.z, da, b->minValue(), r);
        }
        return r;
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
    double sample(const NoisePos& pos) const override {
        double r = applyUnary(op, input->sample(pos));
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408 && (op == UnaryOp::CUBE || op == UnaryOp::ABS)) {
            std::fprintf(stderr, "[UNARY] pos=(%d,%d,%d) op=%d in=%.6f out=%.6f\n", pos.x, pos.y, pos.z, (int)op, input->sample(pos), r);
        }
        return r;
    }
    double minValue() const override { return mn; }
    double maxValue() const override { return mx; }
};

// ===== Clamp =====
class Clamp : public DensityFunction {
public:
    DF input;
    double mn, mx;
    Clamp(DF input_, double mn_, double mx_) : input(std::move(input_)), mn(mn_), mx(mx_) {}
    double sample(const NoisePos& pos) const override {
        double r = clampD(input->sample(pos), mn, mx);
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
            std::fprintf(stderr, "[CLAMP] pos=(%d,%d,%d) mn=%.1f mx=%.1f out=%.6f\n", pos.x, pos.y, pos.z, mn, mx, r);
        }
        return r;
    }
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
        double r = noise->sample(pos.x * xzScale, pos.y * yScale, pos.z * xzScale);
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
            std::fprintf(stderr, "[NOISE] pos=(%d,%d,%d) scale=(%g,%g) in=(%.1f,%.1f,%.1f) value=%.6f\n",
                         pos.x, pos.y, pos.z, xzScale, yScale,
                         pos.x * xzScale, pos.y * yScale, pos.z * xzScale, r);
        }
        return r;
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
        double x = pos.x, y = pos.y, z = pos.z;
        switch (mode) {
            case Mode::SHIFT: break;
            case Mode::SHIFT_A: y = 0.0; break;
            case Mode::SHIFT_B: x = pos.z; y = pos.x; z = 0.0; break;
            default: break;  // 防御：非法 mode 按 SHIFT 处理（避免未初始化读取）
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
        double r = noise ? noise->sample(d, e, f) : 0.0;
        if (wg_splineDebug && pos.x == 800 && pos.y == 0 && pos.z == -428) {
            std::fprintf(stderr, "[SHIFT] pos=(%d,%d,%d) sx=%.6f sy=%.6f sz=%.6f in=(%.6f,%.6f,%.6f) out=%.9f\n",
                         pos.x, pos.y, pos.z, shiftX->sample(pos), shiftY->sample(pos), shiftZ->sample(pos), d, e, f, r);
        }
        return r;
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
        double r = (minInclusive <= d && d < maxExclusive) ? inRange->sample(pos) : outOfRange->sample(pos);
        if (wg_splineDebug && minInclusive < -1000.0) {  // final_density 的 range_choice（min=-1e6）
            const auto* ic = dynamic_cast<const Constant*>(inRange.get());
            const auto* ib = dynamic_cast<const BinaryOperation*>(inRange.get());
            const auto* iu = dynamic_cast<const UnaryOperation*>(inRange.get());
            std::fprintf(stderr, "[RANGECHOICE] pos=(%d,%d,%d) input=%.6f -> %s (%.6f) inRange=%s%s%s\n",
                         pos.x, pos.y, pos.z, d, (minInclusive <= d && d < maxExclusive) ? "in" : "out", r,
                         ic ? "Constant" : (ib ? "BinOp" : (iu ? "Unary" : "other")),
                         ic ? (", val=" + std::to_string(ic->value)).c_str() : "",
                         ib ? (", op=" + std::to_string((int)ib->op)).c_str() : "");
        }
        return r;
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

    // @anchor.test("clampedMap 插值映射对齐 Java DensityFunctionTypes.map2 语义", source="probe:block_probe!densityBuf#001")
    static double clampedMap(double v, int32_t a, int32_t b, double c, double d) {
        if (a == b) return (c + d) / 2.0;
        if (v < a) return c;
        if (v > b) return d;
        return c + (v - a) / (double)(b - a) * (d - c);
    }
    double sample(const NoisePos& pos) const override {
        double r = clampedMap(pos.y, fromY, toY, fromValue, toValue);
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
            std::fprintf(stderr, "[YCG] pos=(%d,%d,%d) from=(%d,%d) val=(%g,%g) out=%.6f\n", pos.x, pos.y, pos.z, fromY, toY, fromValue, toValue, r);
        }
        return r;
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
        double r = d * std::abs(noise->sample(pos.x / d, pos.y / d, pos.z / d));
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
            std::fprintf(stderr, "[WEIRD] pos=(%d,%d,%d) rarity=%d input=%.6f scale=%.6f noiseIn=(%.1f,%.1f,%.1f) out=%.6f\n",
                         pos.x, pos.y, pos.z, (int)rarity, input->sample(pos), d,
                         pos.x / d, pos.y / d, pos.z / d, r);
        }
        return r;
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
        scaledXzScale = (double)(float)684.412 * xzScale;  // Java: 684.412F
        scaledYScale = (double)(float)684.412 * yScale;  // Java: 684.412F
        maxVal = lower.method_40556(scaledYScale);
    }

    double sample(const NoisePos& pos) const override {
        if (wg_profEnabled) {
            wg_profNoiseDF.fetch_add(1, std::memory_order_relaxed);
            auto t0 = std::chrono::steady_clock::now();
            double r = sampleImpl(pos);
            wg_profNoiseNs.fetch_add((int64_t)std::chrono::duration_cast<std::chrono::nanoseconds>(
                std::chrono::steady_clock::now() - t0).count(), std::memory_order_relaxed);
            return r;
        }
        return sampleImpl(pos);
    }
    // @anchor.test("InterpolatedDF 4x4x8 cell 插值逐位对齐 Java DensityInterpolator", source="probe:block_probe!densityBuf#002")
    double sampleImpl(const NoisePos& pos) const {
        double d = pos.x * scaledXzScale;
        double e = pos.y * scaledYScale;
        double f = pos.z * scaledXzScale;
        double g = d / xzFactor;
        double h = e / yFactor;
        double i = f / xzFactor;
        double j = scaledYScale * smearScaleMultiplier;
        double k = j / yFactor;
        bool b3dDump = []() { static const bool v = getenv("WG_B3DDUMP") != nullptr; return v; }();
        if (b3dDump) std::fprintf(stderr, "[B3D] pos=(%d,%d,%d) d=%.17g e=%.17g f=%.17g g=%.17g h=%.17g i=%.17g j=%.17g k=%.17g\n",
            pos.x, pos.y, pos.z, d, e, f, g, h, i, j, k);
        double l = 0.0, m = 0.0, n = 0.0;
        double o = 1.0;
        for (int p = 0; p < 8; p++) {
            const PerlinNoiseSampler* pn = interpolation.getOctave(p);
            if (pn) {
                double go = OctavePerlinNoiseSampler::maintainPrecision(g * o);
                double ho = OctavePerlinNoiseSampler::maintainPrecision(h * o);
                double io = OctavePerlinNoiseSampler::maintainPrecision(i * o);
                double r0 = pn->sample(go, ho, io, k * o, h * o);
                if (b3dDump) std::fprintf(stderr, "[B3D] interp oct=%d s=%.17g t=%.17g u=%.17g res=%.17g contrib=%.17g\n",
                    p, go, ho, io, r0, r0 / o);
                n += r0 / o;
            }
            o /= 2.0;
        }
        double q = (n / 10.0 + 1.0) / 2.0;
        if (b3dDump) std::fprintf(stderr, "[B3D] n=%.17g q=%.17g\n", n, q);
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
                if (pn) {
                    double r0 = pn->sample(s, t, u, v, e * o);
                    if (b3dDump) std::fprintf(stderr, "[B3D] lower oct=%d s=%.17g t=%.17g u=%.17g v=%.17g w=%.17g res=%.17g contrib=%.17g\n",
                        r, s, t, u, v, e * o, r0, r0 / o);
                    l += r0 / o;
                }
            }
            if (!bl3) {
                const PerlinNoiseSampler* pn = upper.getOctave(r);
                if (pn) {
                    double r0 = pn->sample(s, t, u, v, e * o);
                    if (b3dDump) std::fprintf(stderr, "[B3D] upper oct=%d s=%.17g t=%.17g u=%.17g v=%.17g w=%.17g res=%.17g contrib=%.17g\n",
                        r, s, t, u, v, e * o, r0, r0 / o);
                    m += r0 / o;
                }
            }
            o /= 2.0;
        }
        // clampedLerp(l/512, m/512, q) / 128
        double qq = clampD(q, 0.0, 1.0);
        double rr = (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0;
        if (b3dDump) std::fprintf(stderr, "[B3D] l=%.17g m=%.17g result=%.17g\n", l, m, rr);
        return rr;
    }
    double minValue() const override { return -maxValue(); }
    double maxValue() const override { return maxVal; }
};

// ===== InterpolatedDF（minecraft:interpolated）：NoiseChunk cell 插值（4×4×8）=====
// @anchor.idk("结构 Beardifier 密度修正未实现：结构附近 density 差 ~0.12 可翻转 aquifer 判定（-288 岛缺失根因，2026-08-08 确认）")
// vanilla 语义：fillFromNoise 对该函数做 cell 角点采样 + 三线性插值（高频噪声防 alias）
// 实现：lazy 按 chunk 缓存网格（单线程 POC；构建成本 5×49×5 点采样）
class InterpolatedDF : public DensityFunction {
public:
    DF arg;
    static constexpr int CELL_X = 4, CELL_Y = 8, CELL_Z = 4;
    const int minY;     // 噪声 minY（overworld -64 / nether 0）
    const int height;   // 噪声高度（overworld 384 / nether 128）

    explicit InterpolatedDF(DF a, int minY_ = -64, int height_ = 384)
        : arg(std::move(a)), minY(minY_), height(height_), cacheId(nextId.fetch_add(1)) {
        updateInstanceCount();
    }

    int getCacheId() const { return cacheId; }
    static int getInstanceCount() { return instanceCount.load(); }

    double sample(const NoisePos& pos) const override {
        int chunkX = floorDivP(pos.x, 16);
        int chunkZ = floorDivP(pos.z, 16);
        int64_t key = ((int64_t)((uint64_t)(uint32_t)chunkX << 32)) ^ (uint32_t)chunkZ;
        // 多线程：per-instance thread_local 缓存（每线程独立 vector，按实例 id 索引，O(1)）
        // 一次性扩到实例总数（构造后固定）：递归 buildGrid 中不会 resize，外层 slot 引用不悬垂
        auto& slots = tlSlots();
        if (slots.size() < (size_t)instanceCount.load()) slots.resize(instanceCount.load());
        Slot& slot = slots[cacheId];
        if (slot.key != key) {
            slot.key = key;
            buildGrid(chunkX, chunkZ, slot.grid);
            if (wg_profEnabled) wg_profInterpGrid.fetch_add(1, std::memory_order_relaxed);
        }
        const int GX = 16 / CELL_X + 1, GY = height / CELL_Y + 1, GZ = 16 / CELL_Z + 1;
        int gx = pos.x - chunkX * 16;         // 0..15
        int gy = pos.y - minY;               // 0..height-1
        int gz = pos.z - chunkZ * 16;
        int cx = gx / CELL_X, cy = gy / CELL_Y, cz = gz / CELL_Z;
        // 越界保护：clamp 到网格边界内（POC：与 Java DensityInterpolator 直接采样略有差异，但稳定性优先）
        if (cx < 0 || cy < 0 || cz < 0 || cx >= GX || cy >= GY || cz >= GZ ||
            cx >= GX - 1 || cy >= GY - 1 || cz >= GZ - 1) {
            cx = cx < 0 ? 0 : (cx > GX - 2 ? GX - 2 : cx);
            cy = cy < 0 ? 0 : (cy > GY - 2 ? GY - 2 : cy);
            cz = cz < 0 ? 0 : (cz > GZ - 2 ? GZ - 2 : cz);
        }
        if (wg_splineDebug && pos.z == -256 && (pos.y == 58 || pos.y == 52 || pos.y == 60) && (pos.x == -244 || pos.x == -260)) {
            // dump 该 cell 8 角点值 + 插值输入（对比 chunk(-16,-16) vs chunk(-17,-16)）
            auto at = [&](int ix, int iy, int iz) {
                return slot.grid[((size_t)(cy + iy) * GZ + (cz + iz)) * GX + (cx + ix)];
            };
            std::fprintf(stderr, "[GRID] interp@(%d,58,-256) cacheId=%d chunkX=%d key=%lld gx=%d cx=%d cy=%d cz=%d "
                        "c000=%.6f c100=%.6f c010=%.6f c110=%.6f c001=%.6f c101=%.6f c011=%.6f c111=%.6f\n",
                        pos.x, cacheId, chunkX, (long long)key, gx, cx, cy, cz,
                        at(0,0,0), at(1,0,0), at(0,1,0), at(1,1,0),
                        at(0,0,1), at(1,0,1), at(0,1,1), at(1,1,1));
        }
        double fx = (gx % CELL_X) / (double)CELL_X;
        double fy = (gy % CELL_Y) / (double)CELL_Y;
        double fz = (gz % CELL_Z) / (double)CELL_Z;
        auto g = [&](int dx, int dy, int dz) {
            return slot.grid[((size_t)(cy + dy) * GZ + (cz + dz)) * GX + (cx + dx)];
        };
        double d000 = g(0, 0, 0), d100 = g(1, 0, 0), d010 = g(0, 1, 0), d110 = g(1, 1, 0);
        double d001 = g(0, 0, 1), d101 = g(1, 0, 1), d011 = g(0, 1, 1), d111 = g(1, 1, 1);
        double d00 = d000 + (d100 - d000) * fx;
        double d10 = d010 + (d110 - d010) * fx;
        double d01 = d001 + (d101 - d001) * fx;
        double d11 = d011 + (d111 - d011) * fx;
        double d0 = d00 + (d10 - d00) * fy;
        double d1 = d01 + (d11 - d01) * fy;
        double rr = d0 + (d1 - d0) * fz;
        if (wg_splineDebug && pos.y == -8 && pos.x == 728 && pos.z == -408) {
            std::fprintf(stderr, "[INTERP] pos=(%d,%d,%d) cx=%d cy=%d cz=%d result=%.6f\n",
                         pos.x, pos.y, pos.z, cx, cy, cz, rr);
        }
        if (wg_splineDebug && pos.x == -244 && pos.z == -256 && pos.y == 58) {
            std::fprintf(stderr, "[INTERP] pos=(-244,58,-256) fx=%.3f fy=%.3f fz=%.3f result=%.6f\n",
                         fx, fy, fz, rr);
        }
        return rr;
    }

    double minValue() const override { return arg->minValue(); }
    double maxValue() const override { return arg->maxValue(); }

private:
    // 多线程：per-instance thread_local 网格缓存（每线程 vector，按 cacheId 索引）
    struct Slot {
        int64_t key = INT64_MIN;
        std::vector<double> grid;
    };
    int cacheId;
    static std::atomic<int> nextId;
    static std::atomic<int> instanceCount;  // 构造后固定（wg_create 单线程构建）
    static std::vector<Slot>& tlSlots() {
        static thread_local std::vector<Slot> slots;
        return slots;
    }
    static void updateInstanceCount() {
        int n = nextId.load(std::memory_order_relaxed) + 1;
        int cur = instanceCount.load(std::memory_order_relaxed);
        while (n > cur && !instanceCount.compare_exchange_weak(cur, n)) {}
    }

    static int floorDivP(int a, int b) { int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }

    // @anchor.test("InterpolatedDF grid 角点对齐 Java 无插值 finalDensity（y=8 倍数验证点）", source="probe:block_probe!GRID#003")
    void buildGrid(int chunkX, int chunkZ, std::vector<double>& grid) const {
        const int GX = 16 / CELL_X + 1, GY = height / CELL_Y + 1, GZ = 16 / CELL_Z + 1;
        grid.assign((size_t)GX * GY * GZ, 0.0);
        NoisePos p;
        for (int gy = 0; gy < GY; gy++)
            for (int gz = 0; gz < GZ; gz++)
                for (int gx = 0; gx < GX; gx++) {
                    p.x = chunkX * 16 + gx * CELL_X;
                    p.y = minY + gy * CELL_Y;
                    p.z = chunkZ * 16 + gz * CELL_Z;
                    grid[((size_t)gy * GZ + gz) * GX + gx] = arg->sample(p);
                    if (wg_splineDebug && p.x == 728 && p.y == -8 && p.z == -408) {
                        std::fprintf(stderr, "[GRID] interp@(728,-8,-408) gx=%d gy=%d gz=%d value=%.6f\n",
                                     gx, gy, gz, grid[((size_t)gy * GZ + gz) * GX + gx]);
                    }
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

// ===== Cache2DDF（minecraft:cache_2d）：列缓存 =====
// Java ChunkNoiseSampler.Cache2D：同一 (x>>4, z>>4) 列复用首次采样值（列首 pos）。
// cache_2d 仅用于 2D 函数（不依赖 y）→ 列内任意 pos 采样值相同 → 完全无损。
// 块循环 y→z→x 顺序下同列连续 384 次采样，命中率 100%（spline 每列只算一次）。
class Cache2DDF : public DensityFunction {
public:
    DF arg;
    explicit Cache2DDF(DF a) : arg(std::move(a)), cacheId(nextId.fetch_add(1)) {
        updateInstanceCount();
    }

    double sample(const NoisePos& pos) const override {
        auto& slots = tlSlots();
        if (slots.size() < (size_t)instanceCount.load()) slots.resize(instanceCount.load());
        Slot& slot = slots[cacheId];
        // Java ChunkNoiseSampler.Cache2D：key = ChunkPos.toLong(blockX, blockZ)（block 级，同 x,z 列复用）
        // 注意：不是 chunk 级——FlatCache 5×5 角点（不同 x,z）必须各自采样，chunk 级缓存会错误共享
        int64_t key = ((int64_t)((uint64_t)(uint32_t)pos.x << 32)) ^ (uint32_t)pos.z;
        if (slot.key != key) {
            if (wg_splineDebug) std::fprintf(stderr, "[CACHE2D] cacheId=%d miss pos=(%d,%d,%d)\n", cacheId, pos.x, pos.y, pos.z);
            slot.key = key;
            slot.value = arg->sample(pos);
        }
        return slot.value;
    }
    double minValue() const override { return arg->minValue(); }
    double maxValue() const override { return arg->maxValue(); }

private:
    struct Slot { int64_t key = INT64_MIN; double value = 0.0; };
    int cacheId;
    static std::atomic<int> nextId;
    static std::atomic<int> instanceCount;  // 构造后固定（wg_create 单线程构建）
    static std::vector<Slot>& tlSlots() {
        static thread_local std::vector<Slot> slots;
        return slots;
    }
    static void updateInstanceCount() {
        int n = nextId.load(std::memory_order_relaxed) + 1;
        int cur = instanceCount.load(std::memory_order_relaxed);
        while (n > cur && !instanceCount.compare_exchange_weak(cur, n)) {}
    }
};

// ===== FlatCacheDF（minecraft:flat_cache）：chunk 级 5×5 网格缓存 =====
// Java ChunkNoiseSampler.FlatCache：按 biome 坐标网格（horizontalBiomeEnd+1 = 5，间距 4 块）
// 预计算 delegate.sample(blockX, 0, blockZ)（y=0），块级查表；网格外回退直接采样。
// 用于 2D 大陆样条（continents/erosion/ridges/factor/jaggedness/offset）→ 完全无损。
class FlatCacheDF : public DensityFunction {
public:
    DF arg;
    explicit FlatCacheDF(DF a) : arg(std::move(a)), cacheId(nextId.fetch_add(1)) {
        updateInstanceCount();
    }
    int getCacheId() const { return cacheId; }

    double sample(const NoisePos& pos) const override {
        auto& slots = tlSlots();
        if (slots.size() < (size_t)instanceCount.load()) slots.resize(instanceCount.load());
        Slot& slot = slots[cacheId];
        int64_t key = ((int64_t)((uint64_t)(uint32_t)(pos.x >> 4) << 32)) ^ (uint32_t)(pos.z >> 4);
        if (slot.key != key) {
            // Java 语义：FlatCache 实例绑定生成 chunk，网格覆盖 [cx*16, cx*16+16]；
            // 边界点（x=cx*16+16，即下一 chunk 首列）命中现有网格 k=4，不重建（防嵌套递归）。
            int kc = (pos.x >> 2) - slot.cx * 4;
            int lc = (pos.z >> 2) - slot.cz * 4;
            if (slot.key == INT64_MIN || kc < 0 || lc < 0 || kc >= GRID || lc >= GRID) {
                slot.key = key;
                slot.cx = pos.x >> 4;
                slot.cz = pos.z >> 4;
                buildGrid(slot.cx, slot.cz, slot.grid);
            }
        }
        int k = (pos.x >> 2) - slot.cx * 4;  // 用网格的 chunk（Java: startBiomeX），非 pos 的 chunk
        int l = (pos.z >> 2) - slot.cz * 4;
        if (k >= 0 && l >= 0 && k < GRID && l < GRID) return slot.grid[(size_t)l * GRID + k];
        return arg->sample(pos);
    }
    double minValue() const override { return arg->minValue(); }
    double maxValue() const override { return arg->maxValue(); }

private:
    static constexpr int GRID = 5;  // horizontalBiomeEnd + 1 = 4 + 1
    struct Slot {
        int64_t key = INT64_MIN;
        int cx = 0, cz = 0;
        std::vector<double> grid;
    };
    int cacheId;
    static std::atomic<int> nextId;
    static std::atomic<int> instanceCount;  // 构造后固定（wg_create 单线程构建）
    static std::vector<Slot>& tlSlots() {
        static thread_local std::vector<Slot> slots;
        return slots;
    }
    static void updateInstanceCount() {
        int n = nextId.load(std::memory_order_relaxed) + 1;
        int cur = instanceCount.load(std::memory_order_relaxed);
        while (n > cur && !instanceCount.compare_exchange_weak(cur, n)) {}
    }

    // @anchor.test("FlatCacheDF 5x5 角点网格对齐 Java ChunkNoiseSampler.FlatCache（biome 坐标网格间距 4）", source="probe:block_probe!FLATCACHE#004")
    void buildGrid(int chunkX, int chunkZ, std::vector<double>& grid) const {
        if (wg_profEnabled) wg_profNoiseDF.fetch_add(1, std::memory_order_relaxed);  // [PROF] FlatCache 构建次数
        grid.assign((size_t)GRID * GRID, 0.0);
        NoisePos p;
        p.y = 0;  // Java: UnblendedNoisePos(blockX, 0, blockZ)
        for (int i = 0; i < GRID; i++) {
            p.x = (chunkX * 4 + i) * 4;
            for (int j = 0; j < GRID; j++) {
                p.z = (chunkZ * 4 + j) * 4;
                grid[(size_t)j * GRID + i] = arg->sample(p);
            }
        }
        if (wg_splineDebug) {
            std::fprintf(stderr, "[FLATCACHE] chunk=(%d,%d) cacheId=%d grid[0]=%.9f grid[1]=%.9f grid[5]=%.9f\n",
                         chunkX, chunkZ, cacheId, grid[0], grid[1], grid[5]);
        }
    }
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
        if (wg_profEnabled) {
            wg_profSpline.fetch_add(1, std::memory_order_relaxed);
            auto t0 = std::chrono::steady_clock::now();
            double r = sampleImpl(pos);
            wg_profSplineNs.fetch_add((int64_t)std::chrono::duration_cast<std::chrono::nanoseconds>(
                std::chrono::steady_clock::now() - t0).count(), std::memory_order_relaxed);
            return r;
        }
        return sampleImpl(pos);
    }
    // @anchor.test("FlatCacheDF 角点缓存命中/重建路径对齐 Java（5x5 网格 + 位置函数判定）", source="probe:block_probe!FLATCACHE#004")
    double sampleImpl(const NoisePos& pos) const {
        if (isLeaf) return fixedValue;
        double f = locationFunction->sample(pos);
        double r = apply(f, pos);
        if (wg_splineDebug) {
            const auto* fc = dynamic_cast<const FlatCacheDF*>(locationFunction.get());
            const auto* cn = dynamic_cast<const Cache2DDF*>(locationFunction.get());
            std::fprintf(stderr, "[SPLINE] pos=(%d,%d,%d) f=%.9f result=%.9f n=%zu locFn=%s%s%s locs=[",
                         pos.x, pos.y, pos.z, f, r, locations.size(),
                         fc ? "FlatCache" : (cn ? "Cache2D" : "other"),
                         fc ? (", cacheId=" + std::to_string(fc->getCacheId())).c_str() : "",
                         locationFunction == nullptr ? " NULL" : "");
            for (size_t li = 0; li < locations.size(); li++)
                std::fprintf(stderr, "%.4f%s", locations[li], li + 1 < locations.size() ? "," : "");
            std::fprintf(stderr, "]\n");
        }
        return r;
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

// InterpolatedDF：实例 id 分配（per-instance thread_local 缓存索引）
std::atomic<int> InterpolatedDF::nextId{0};
std::atomic<int> InterpolatedDF::instanceCount{0};
std::atomic<int> Cache2DDF::nextId{0};
std::atomic<int> Cache2DDF::instanceCount{0};
std::atomic<int> FlatCacheDF::nextId{0};
std::atomic<int> FlatCacheDF::instanceCount{0};

} // namespace wg


