// gpu_density_engine.h —— GPU 密度引擎（I2：DFC + CpuBackend + Vulkan 运行时，worldgen 集成用）
// 语义：CpuBackend.split（CPU 预拆分坐标，double 精度）→ Vulkan kernel（GPU float 求值）→ 读回。
// 与 dfc_final_backend_e2e.cpp 的 GPU 路径逐位一致（同一 shader + 同一 CpuBackend 数据）。
// PIMPL：本头不 include cpu_backend.h/density.h（其 static 成员定义非 inline，多 TU 会 LNK2005），
// 实现细节全在 .cpp（worldgen 集成同样只需链接 gpu_density_engine.cpp）。
#pragma once
#include <cstdint>
#include <string>
#include <memory>

class GpuDensityEngine {
public:
    // seed: 世界种子（与 wg_create 相同）；spvPath: final_density.spv 绝对路径
    // outPerSample（X2 260903-05）：每采样点输出 float 数——1=final（默认）；5=channels @ corners
    //   （final_density_channels.spv，interleaved 布局 out[s*NCH+ch]，judge C3 已写死声明）
    GpuDensityEngine(uint64_t seed, const std::string& spvPath, int outPerSample = 1);
    ~GpuDensityEngine();
    GpuDensityEngine(const GpuDensityEngine&) = delete;
    GpuDensityEngine& operator=(const GpuDensityEngine&) = delete;

    // 批量求值：coords = n×3 int32（块坐标 x,y,z），out = n*outPerSample float
    void fill(const int32_t* coords, int n, float* out);

    // 单点采样（便捷；内部走 fill，N=1 低效仅诊断用）
    float sample(int x, int y, int z);

    // 布局参数（生成器产出，与 shader 一致；host 侧可查）
    int splitTotal() const;
    int perSample() const;
    int splineBindBase() const;

private:
    struct Impl;
    std::unique_ptr<Impl> m;
};
