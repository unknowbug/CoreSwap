#pragma once
// 最小 JSON 解析器（仅支持 density function JSON 所需子集）
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <stdexcept>
#include <cctype>
#include <cmath>

namespace wg {

class JsonValue {
public:
    enum class Type { Null, Bool, Number, String, Array, Object };
    Type type = Type::Null;
    bool boolVal = false;
    double numVal = 0;
    std::string strVal;
    std::vector<JsonValue> arr;
    std::vector<std::pair<std::string, JsonValue>> obj; // 保持顺序

    bool isNull() const { return type == Type::Null; }
    bool isNumber() const { return type == Type::Number; }
    bool isString() const { return type == Type::String; }
    bool isObject() const { return type == Type::Object; }
    bool isArray() const { return type == Type::Array; }

    const JsonValue* get(const std::string& key) const {
        for (auto& [k, v] : obj) if (k == key) return &v;
        return nullptr;
    }
    double num(double def = 0.0) const { return isNumber() ? numVal : def; }
    double num(const std::string& key, double def = 0.0) const {
        const JsonValue* v = get(key);
        return v && v->isNumber() ? v->numVal : def;
    }
    std::string str(const std::string& def = "") const { return isString() ? strVal : def; }
};

class JsonParser {
public:
    explicit JsonParser(std::string s) : src(std::move(s)), pos(0) {}

    JsonValue parse() {
        skipWs();
        JsonValue v = parseValue();
        skipWs();
        if (pos != src.size()) throw std::runtime_error("JSON trailing data at " + std::to_string(pos));
        return v;
    }

private:
    std::string src; // 按值持有（避免临时对象悬垂）
    size_t pos;

    void skipWs() { while (pos < src.size() && std::isspace((unsigned char)src[pos])) pos++; }
    char peek() { return pos < src.size() ? src[pos] : '\0'; }
    char next() { return pos < src.size() ? src[pos++] : '\0'; }
    void expect(char c) {
        if (peek() != c) throw std::runtime_error(std::string("expected '") + c + "' at " + std::to_string(pos));
        pos++;
    }

    JsonValue parseValue() {
        char c = peek();
        switch (c) {
            case '{': return parseObject();
            case '[': return parseArray();
            case '"': return parseString();
            case 't': return parseLiteral(true);
            case 'f': return parseLiteral(false);
            case 'n': return parseLiteralNull();
            default: return parseNumber();
        }
    }

    JsonValue parseObject() {
        expect('{');
        JsonValue v; v.type = JsonValue::Type::Object;
        skipWs();
        if (peek() == '}') { pos++; return v; }
        while (true) {
            skipWs();
            JsonValue key = parseString();
            skipWs();
            expect(':');
            skipWs();
            JsonValue val = parseValue();
            v.obj.emplace_back(key.strVal, std::move(val));
            skipWs();
            char c = next();
            if (c == ',') continue;
            if (c == '}') break;
            throw std::runtime_error("bad object at " + std::to_string(pos));
        }
        return v;
    }

    JsonValue parseArray() {
        expect('[');
        JsonValue v; v.type = JsonValue::Type::Array;
        skipWs();
        if (peek() == ']') { pos++; return v; }
        while (true) {
            skipWs();
            v.arr.push_back(parseValue());
            skipWs();
            char c = next();
            if (c == ',') continue;
            if (c == ']') break;
            throw std::runtime_error("bad array at " + std::to_string(pos));
        }
        return v;
    }

    JsonValue parseString() {
        expect('"');
        JsonValue v; v.type = JsonValue::Type::String;
        std::string out;
        while (true) {
            char c = next();
            if (c == '"') break;
            if (c == '\\') {
                char e = next();
                switch (e) {
                    case '"': out += '"'; break;
                    case '\\': out += '\\'; break;
                    case '/': out += '/'; break;
                    case 'b': out += '\b'; break;
                    case 'f': out += '\f'; break;
                    case 'n': out += '\n'; break;
                    case 'r': out += '\r'; break;
                    case 't': out += '\t'; break;
                    case 'u': {
                        int code = 0;
                        for (int i = 0; i < 4; i++) {
                            char h = next();
                            code = code * 16 + (h >= '0' && h <= '9' ? h - '0' : (h >= 'a' && h <= 'f' ? h - 'a' + 10 : (h >= 'A' && h <= 'F' ? h - 'A' + 10 : 0)));
                        }
                        if (code < 0x80) out += (char)code;
                        else if (code < 0x800) { out += (char)(0xC0 | (code >> 6)); out += (char)(0x80 | (code & 0x3F)); }
                        else { out += (char)(0xE0 | (code >> 12)); out += (char)(0x80 | ((code >> 6) & 0x3F)); out += (char)(0x80 | (code & 0x3F)); }
                        break;
                    }
                    default: out += e;
                }
            } else if (c == '\0') {
                throw std::runtime_error("unterminated string");
            } else {
                out += c;
            }
        }
        v.strVal = out;
        return v;
    }

    JsonValue parseNumber() {
        JsonValue v; v.type = JsonValue::Type::Number;
        size_t start = pos;
        while (pos < src.size() && (std::isdigit((unsigned char)src[pos]) || src[pos] == '-' || src[pos] == '+' ||
                                    src[pos] == '.' || src[pos] == 'e' || src[pos] == 'E')) pos++;
        v.numVal = std::stod(src.substr(start, pos - start));
        return v;
    }

    JsonValue parseLiteral(bool b) {
        JsonValue v; v.type = JsonValue::Type::Bool; v.boolVal = b;
        const char* lit = b ? "true" : "false";
        size_t len = b ? 4 : 5;
        if (src.compare(pos, len, lit) != 0) throw std::runtime_error("bad literal");
        pos += len;
        return v;
    }
    JsonValue parseLiteralNull() {
        JsonValue v; v.type = JsonValue::Type::Null;
        if (src.compare(pos, 4, "null") != 0) throw std::runtime_error("bad null");
        pos += 4;
        return v;
    }
};

} // namespace wg
