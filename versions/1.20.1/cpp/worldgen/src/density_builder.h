#pragma once
// 从 vanilla worldgen JSON 构建 density function 树（1.20.1 overworld）
#include <map>
#include <string>
#include <memory>
#include <functional>
#include "json.h"
#include "density.h"

namespace wg {

class DensityBuilder {
public:
    using NoiseParamsMap = std::map<std::string, DoublePerlinNoiseSampler::NoiseParameters>;

    DensityBuilder(uint64_t seed, const NoiseParamsMap& noiseParams)
        : seed(seed), noiseParams(noiseParams) {
        // randomDeriver = XoroshiroRandom(seed).nextSplitter()
        XoroshiroRandom base(seed);
        randomDeriver = base.nextSplitter();
    }

    // 解析单个 density function JSON（registry 条目）
    DF parseFile(const std::string& key, const std::string& jsonText) {
        JsonParser parser(jsonText);
        JsonValue root = parser.parse();
        return buildNode(root, key);
    }

    // 构建 density function：数字/字符串引用/对象
    DF buildNode(const JsonValue& v, const std::string& selfKey = "") {
        if (v.isNumber()) {
            return std::make_shared<Constant>(v.numVal);
        }
        if (v.isString()) {
            std::string ref = v.strVal;
            if (ref == selfKey) {
                // 自引用（如 range_choice 引用自身）→ 惰性包装
                return makeLazyRef(ref);
            }
            return resolveRef(ref);
        }
        if (v.isObject()) {
            return buildObject(v, selfKey);
        }
        throw std::runtime_error("bad density node");
    }

    // 构建对象类型节点
    DF buildObject(const JsonValue& obj, const std::string& selfKey) {
        const JsonValue* t = obj.get("type");
        std::string type = t ? t->str() : "";
        if (type.empty()) {
            // 数字对象（minecraft:noise 简化？）或 spline 的嵌套结构
            throw std::runtime_error("density object without type");
        }
        auto arg = [&](const char* key, const char* key2 = nullptr) -> DF {
            const JsonValue* a = obj.get(key);
            if (!a && key2) a = obj.get(key2);
            if (!a) throw std::runtime_error(std::string("missing arg ") + key + " in " + type);
            return buildNode(*a, selfKey);
        };
        auto noiseSampler = [&](const std::string& key) -> std::shared_ptr<DoublePerlinNoiseSampler> {
            return getNoiseSampler(key);
        };
        auto refNoise = [&](const JsonValue& nv) -> std::shared_ptr<DoublePerlinNoiseSampler> {
            return getNoiseSampler(nv.str());
        };

        if (type == "minecraft:constant") {
            return std::make_shared<Constant>(obj.num("value", 0.0));
        }
        if (type == "minecraft:add") {
            return BinaryOperation::create(BinOp::ADD, arg("argument1"), arg("argument2"));
        }
        if (type == "minecraft:mul") {
            return BinaryOperation::create(BinOp::MUL, arg("argument1"), arg("argument2"));
        }
        if (type == "minecraft:min") {
            return BinaryOperation::create(BinOp::MIN, arg("argument1"), arg("argument2"));
        }
        if (type == "minecraft:max") {
            return BinaryOperation::create(BinOp::MAX, arg("argument1"), arg("argument2"));
        }
        if (type == "minecraft:abs") return UnaryOperation::create(UnaryOp::ABS, arg("argument"));
        if (type == "minecraft:square") return UnaryOperation::create(UnaryOp::SQUARE, arg("argument"));
        if (type == "minecraft:cube") return UnaryOperation::create(UnaryOp::CUBE, arg("argument"));
        if (type == "minecraft:half_negative") return UnaryOperation::create(UnaryOp::HALF_NEGATIVE, arg("argument"));
        if (type == "minecraft:quarter_negative") return UnaryOperation::create(UnaryOp::QUARTER_NEGATIVE, arg("argument"));
        if (type == "minecraft:squeeze") return UnaryOperation::create(UnaryOp::SQUEEZE, arg("argument"));
        if (type == "minecraft:clamp") {
            return std::make_shared<Clamp>(arg("input"), obj.num("min", 0.0), obj.num("max", 0.0));
        }
        if (type == "minecraft:noise") {
            const JsonValue* n = obj.get("noise");
            double xz = obj.num("xz_scale", 1.0);
            double y = obj.num("y_scale", 1.0);
            return std::make_shared<NoiseDF>(refNoise(*n), xz, y);
        }
        if (type == "minecraft:shifted_noise") {
            const JsonValue* n = obj.get("noise");
            double xz = obj.num("xz_scale", 1.0);
            double y = obj.num("y_scale", 1.0);
            DF sx, sy, sz;
            if (const JsonValue* v = obj.get("shift_x")) sx = buildNode(*v, selfKey); else sx = std::make_shared<Constant>(0.0);
            if (const JsonValue* v = obj.get("shift_y")) sy = buildNode(*v, selfKey); else sy = std::make_shared<Constant>(0.0);
            if (const JsonValue* v = obj.get("shift_z")) sz = buildNode(*v, selfKey); else sz = std::make_shared<Constant>(0.0);
            return std::make_shared<ShiftedNoiseDF>(sx, sy, sz, xz, y, refNoise(*n));
        }
        if (type == "minecraft:shift_a") {
            return std::make_shared<ShiftDF>(getNoiseSamplerFromObj(obj), ShiftDF::Mode::SHIFT_A);
        }
        if (type == "minecraft:shift_b") {
            return std::make_shared<ShiftDF>(getNoiseSamplerFromObj(obj), ShiftDF::Mode::SHIFT_B);
        }
        if (type == "minecraft:shift") {
            return std::make_shared<ShiftDF>(getNoiseSamplerFromObj(obj), ShiftDF::Mode::SHIFT);
        }
        if (type == "minecraft:range_choice") {
            return std::make_shared<RangeChoice>(
                arg("input"), obj.num("min_inclusive", 0.0), obj.num("max_exclusive", 0.0),
                arg("when_in_range"), arg("when_out_of_range"));
        }
        if (type == "minecraft:y_clamped_gradient") {
            return std::make_shared<YClampedGradient>(
                (int32_t)obj.num("from_y", 0.0), (int32_t)obj.num("to_y", 0.0),
                obj.num("from_value", 0.0), obj.num("to_value", 0.0));
        }
        if (type == "minecraft:weird_scaled_sampler") {
            const JsonValue* rv = obj.get("rarity_value_mapper");
            std::string rarity = rv ? rv->str() : "type1";
            WeirdScaledSampler::Rarity r = rarity == "type2" ? WeirdScaledSampler::Rarity::CAVES : WeirdScaledSampler::Rarity::TUNNELS;
            return std::make_shared<WeirdScaledSampler>(arg("input"), refNoise(*obj.get("noise")), r);
        }
        if (type == "minecraft:blend_alpha") return std::make_shared<BlendAlpha>();
        if (type == "minecraft:blend_offset") return std::make_shared<BlendOffset>();
        if (type == "minecraft:blend_density") return std::make_shared<BlendDensityDF>(arg("argument"));
        if (type == "minecraft:flat_cache" || type == "minecraft:cache_2d" || type == "minecraft:cache_once" ||
            type == "minecraft:cache_all_in_cell") {
            return std::make_shared<WrappingDF>(arg("argument"));
        }
        if (type == "minecraft:interpolated") {
            // NoiseChunk cell 插值（4×4×8）：高频噪声防 alias，块级采样时三线性插值
            return std::make_shared<InterpolatedDF>(arg("argument"));
        }
        if (type == "minecraft:old_blended_noise") {
            // InterpolatedNoiseSampler：random = randomDeriver.split(Identifier("terrain"))
            // 注意：split(Identifier) → split(toString) = split("minecraft:terrain")
            XoroshiroRandom rnd = randomDeriver.split("minecraft:terrain");
            return std::make_shared<InterpolatedNoiseDF>(
                rnd, obj.num("xz_scale", 0.25), obj.num("y_scale", 0.125),
                obj.num("xz_factor", 80.0), obj.num("y_factor", 160.0),
                obj.num("smear_scale_multiplier", 8.0));
        }
        if (type == "minecraft:spline") {
            return buildSpline(obj, selfKey);
        }
        throw std::runtime_error("unknown density type: " + type);
    }

    // 构建 spline
    DF buildSpline(const JsonValue& objIn, const std::string& selfKey) {
        // 解包：{"type":"minecraft:spline","spline":{coordinate,points}} 或直接 {coordinate,points}
        const JsonValue* obj = objIn.isObject() && objIn.get("spline") ? objIn.get("spline") : &objIn;
        const JsonValue* coord = obj->get("coordinate");
        const JsonValue* points = obj->get("points");
        auto spline = std::make_shared<SplineDF>();
        spline->isLeaf = false;
        spline->locationFunction = buildNode(*coord, selfKey);
        for (const JsonValue& p : points->arr) {
            spline->locations.push_back((float)p.num("location", 0.0));
            spline->derivatives.push_back((float)p.num("derivative", 0.0));
            const JsonValue* pv = p.get("value");
            if (pv->isNumber()) {
                auto leaf = std::make_shared<SplineDF>();
                leaf->isLeaf = true;
                leaf->fixedValue = (float)pv->numVal;
                spline->subSplines.push_back(leaf);
            } else {
                spline->subSplines.push_back(std::dynamic_pointer_cast<SplineDF>(buildSpline(*pv, selfKey)));
            }
        }
        return spline;
    }

    // 解析 registry 引用（"minecraft:overworld/continents" 等）
    DF resolveRef(const std::string& ref) {
        auto it = registry.find(ref);
        if (it != registry.end()) return it->second;

        if (ref == "minecraft:shift_x") {
            auto df = std::make_shared<ShiftDF>(getNoiseSampler("minecraft:offset"), ShiftDF::Mode::SHIFT_A);
            auto df2 = std::make_shared<WrappingDF>(df);
            registry[ref] = df2;
            return df2;
        }
        if (ref == "minecraft:shift_z") {
            auto df = std::make_shared<ShiftDF>(getNoiseSampler("minecraft:offset"), ShiftDF::Mode::SHIFT_B);
            auto df2 = std::make_shared<WrappingDF>(df);
            registry[ref] = df2;
            return df2;
        }
        if (ref == "minecraft:y") {
            // y = yClampedGradient(minY, maxY, minY, maxY)（恒等 y 映射，overworld -64..320）
            auto df = std::make_shared<YClampedGradient>(-64, 320, -64.0, 320.0);
            registry[ref] = df;
            return df;
        }
        if (ref == "minecraft:zero") {
            auto df = std::make_shared<Constant>(0.0);
            registry[ref] = df;
            return df;
        }
        // 惰性按需加载：minecraft:overworld/<name>
        if (ref.rfind("minecraft:overworld/", 0) == 0 && externalLoader) {
            std::string name = ref.substr(std::string("minecraft:overworld/").size());
            // 循环引用保护：先注册占位，externalLoader 期间若再引用 ref 会命中占位
            auto placeholder = std::make_shared<LazyRef>();
            registry[ref] = placeholder;
            DF df = externalLoader(ref, name);
            if (df) {
                placeholder->target = df;
                registry[ref] = df;
                return df;
            }
            registry.erase(ref);
        }
        throw std::runtime_error("unresolved density function ref: " + ref);
    }

    // 惰性自引用包装
    DF makeLazyRef(const std::string& ref) {
        auto it = lazyRefs.find(ref);
        if (it != lazyRefs.end()) return it->second;
        auto placeholder = std::make_shared<LazyRef>();
        lazyRefs[ref] = placeholder;
        placeholder->target = registry[ref]; // 可能尚未注册
        return placeholder;
    }

    class LazyRef : public DensityFunction {
    public:
        DF target;
        double sample(const NoisePos& pos) const override { return target->sample(pos); }
        // 未填充时按 Java RegistryEntryHolder 语义：±∞（保守范围）
        double minValue() const override { return target ? target->minValue() : -std::numeric_limits<double>::infinity(); }
        double maxValue() const override { return target ? target->maxValue() : std::numeric_limits<double>::infinity(); }
    };

    // 注册 registry 条目（density_function 文件）
    void registerFunction(const std::string& key, DF df) {
        auto old = registry.find(key);
        if (old != registry.end()) {
            // 占位符更新（循环引用）
            if (auto lr = std::dynamic_pointer_cast<LazyRef>(old->second)) {
                lr->target = df;
            }
        }
        registry[key] = df;
        for (auto& [k, lr] : lazyRefs) {
            if (k == key && lr->target == nullptr) lr->target = df;
        }
    }

    // noise sampler 缓存
    std::shared_ptr<DoublePerlinNoiseSampler> getNoiseSampler(const std::string& key) {
        auto it = noiseSamplers.find(key);
        if (it != noiseSamplers.end()) return it->second;
        auto pIt = noiseParams.find(key);
        if (pIt == noiseParams.end()) throw std::runtime_error("unknown noise params: " + key);
        auto rnd = randomDeriver.split(key);
        auto sampler = std::make_shared<DoublePerlinNoiseSampler>(rnd, pIt->second);
        noiseSamplers[key] = sampler;
        return sampler;
    }

    // 惰性加载器：(fullRef, shortName) -> DF（外部设置，用于按需加载 registry 引用）
    std::function<DF(const std::string&, const std::string&)> externalLoader;

    // 获取已注册的 registry 条目（供探针使用）
    DF getRegistryEntry(const std::string& key) {
        auto it = registry.find(key);
        return it != registry.end() ? it->second : nullptr;
    }

    // 暴露 randomDeriver（供探针复刻 NoiseConfig 派生链）
    const XoroshiroRandom::Splitter& randomDeriverPublic() const { return randomDeriver; }

private:
    std::shared_ptr<DoublePerlinNoiseSampler> getNoiseSamplerFromObj(const JsonValue& obj) {
        const JsonValue* n = obj.get("noise");
        if (!n) throw std::runtime_error("noise field missing");
        return getNoiseSampler(n->str());
    }

    uint64_t seed;
    const NoiseParamsMap& noiseParams;
    XoroshiroRandom::Splitter randomDeriver;
    std::map<std::string, DF> registry;
    std::map<std::string, std::shared_ptr<LazyRef>> lazyRefs;
    std::map<std::string, std::shared_ptr<DoublePerlinNoiseSampler>> noiseSamplers;
};

} // namespace wg
