// gpu_ffi.cpp —— C-ABI shim：把 GpuDensityEngine 借给 Rust FFI（lossless-accel 路线② X1=A）
// 导出面（稳定 C ABI，Rust 侧声明对应）：
//   gpu_ffi_create(seed, spvPath) -> handle（NULL=失败）
//   gpu_ffi_fill(h, coords, n, out)
//   gpu_ffi_destroy(h)
//   gpu_ffi_last_error() -> const char*（线程局部，诊断用）
// 注意：GpuDensityEngine 非线程安全（共享 buffer，驱动级 0xC0000005 前科）——
// 互斥策略在 Rust 侧（Mutex<handle> 或串行调用），shim 不加锁（保持与 C++ 生产同语义，实测 fillMtx 行为）。
#include "gpu_density_engine.h"
#include <cstdint>
#include <cstring>
#include <string>

static thread_local std::string t_lastError;

extern "C" {

__declspec(dllexport) void* gpu_ffi_create(uint64_t seed, const char* spvPath) {
    t_lastError.clear();
    if (!spvPath) { t_lastError = "spvPath is null"; return nullptr; }
    try {
        return static_cast<void*>(new GpuDensityEngine(seed, std::string(spvPath)));
    } catch (const std::exception& e) {
        t_lastError = std::string("create failed: ") + e.what();
        return nullptr;
    } catch (...) {
        t_lastError = "create failed: unknown exception";
        return nullptr;
    }
}

// X2（260903-05）：channels 引擎创建——outPerSample=5 时 fill 输出 n*5 float
// （interleaved：out[s*NCH+ch]，ch 序 = channels_map.json interp_order）。
__declspec(dllexport) void* gpu_ffi_create_ex(uint64_t seed, const char* spvPath, int32_t outPerSample) {
    t_lastError.clear();
    if (!spvPath) { t_lastError = "spvPath is null"; return nullptr; }
    try {
        return static_cast<void*>(new GpuDensityEngine(seed, std::string(spvPath), outPerSample));
    } catch (const std::exception& e) {
        t_lastError = std::string("create failed: ") + e.what();
        return nullptr;
    } catch (...) {
        t_lastError = "create failed: unknown exception";
        return nullptr;
    }
}

__declspec(dllexport) void gpu_ffi_fill(void* h, const int32_t* coords, int32_t n, float* out) {
    t_lastError.clear();
    if (!h || !coords || !out || n <= 0) { t_lastError = "bad args"; return; }
    try {
        static_cast<GpuDensityEngine*>(h)->fill(coords, n, out);
    } catch (const std::exception& e) {
        t_lastError = std::string("fill failed: ") + e.what();
    } catch (...) {
        t_lastError = "fill failed: unknown exception";
    }
}

__declspec(dllexport) void gpu_ffi_destroy(void* h) {
    delete static_cast<GpuDensityEngine*>(h);
}

__declspec(dllexport) const char* gpu_ffi_last_error() {
    return t_lastError.c_str();
}

// 布局参数查询（诊断用，与 shader 布局一致性核对）
__declspec(dllexport) int32_t gpu_ffi_split_total(void* h) {
    return h ? static_cast<GpuDensityEngine*>(h)->splitTotal() : -1;
}
__declspec(dllexport) int32_t gpu_ffi_per_sample(void* h) {
    return h ? static_cast<GpuDensityEngine*>(h)->perSample() : -1;
}

} // extern "C"
