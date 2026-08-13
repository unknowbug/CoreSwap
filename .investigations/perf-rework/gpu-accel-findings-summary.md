# GPU 加速预研：完整发现记录（2026-08-13）

> 本阶段（GPU 加速方向）所有关键发现的统一详细记录，按时间序。分散详情见各专篇。
> 专篇：fp32-experiment.md、coord-precision-finding.md、interpolated-fp64-finding.md、c2me-dfc-review.md、coreswap-vs-c2me.md、vulkan-dfc-research.md、vulkan-proto/README.md。

## 发现时间线

### F1. 数据流开销不是瓶颈（CUDA 探针）
- **测什么**：CPU FP64 折叠坐标 → GPU 算 → 读回，单 chunk 端到端开销。
- **数据**：单 chunk 往返 144µs（H2D 88KB + kernel + D2H 384KB + sync），D2H 384KB 57µs、kernel 启动 7.1µs、busy kernel 10µs；PCIe 实测带宽 6.9 GB/s（RTX 4060 Laptop，PCIe 4.0 x8）。
- **结论**：对比 CPU density 47ms/chunk，快 326×；但单 chunk 的 GPU 计算只占往返 ~7%，**批量预生成才是 GPU 甜点**（批量 256 chunk 摊薄后吞吐 ~10000 chunk/s vs CPU 14 chunk/s）。

### F2. FP32 的「损」来自坐标精度，不是计算内部精度
- **测什么**：density 计算路径加 `(float)` 舍入（模拟 GPU FP32），对比 double 逐位基线。
- **数据**：
  - 计算内部 float（噪声/算术/插值舍入）：误差 ~1e-7，近坐标 block_probe 零新增 mismatch（99.9994%/99.9997% 保持）。
  - 坐标 float（`pos.x × scale` 舍入）：远坐标（3000 万）→ finalDensity 差异 ~1e-3（18/94 行），可能翻转边界块。
- **结论**：FP32 的损 = 远坐标的坐标精度；计算内部 FP32 对方块判定鲁棒。

### F3. 坐标必须「int32 整数 + float 小数」拆分
- **触发**：发现此前方案假设「折叠后坐标 float 可精确表示」是错的——`maintainPrecision` 折叠后坐标仍 ~2^24，float ulp=2，整体 float 化丢小数。
- **数据**：整体 float 化误差 2.2e-1（翻方块）vs 拆分（int32 整数精确 hash + float 小数）误差 1.6e-7（远坐标）。
- **原理**：Perlin 采样对坐标只有两个用途——整数部分进 perm 表 hash（必须精确）、小数部分进 grad/fade（float 的 2^-23 够）。拆分让两者各取所需。

### F4. 对象搞混：base_3d_noise = InterpolatedNoiseSampler（不是 DoublePerlinNoiseSampler）
- **触发**：准备用「真实 base_3d_noise 参数」替换简化验证，查 vanilla 源码 `DensityFunctionTypes.java` L46：`register(registry, "old_blended_noise", InterpolatedNoiseSampler.CODEC)`。
- **影响**：此前 `octave.comp`/`octave_probe.cpp` 实现的是 DoublePerlinNoiseSampler（first+second×1.018 = NormalNoise），**对象错了**。「多 octave 叠加不放大误差」结论本身仍成立，但 base_3d_noise 有额外精度杀手（见 F5）。
- **真实结构**：16 lower + 16 upper + 8 interpolation octave，y 轴 smear，`scaledXzScale = 684.412×0.25 = 171.103`（正是 F2 实验用的系数）。

### F5. `/o` 放大结构要求 double（FP32 不可行的根因）
- **结构**：`for (r=0..15) { l += sample(coord×o)/o; o/=2; }`——高 octave（o=2^-r）噪声被 `1/o=2^r` 放大。
- **数据**（单 octave 诊断）：
  - r=0（o=1）贡献误差 3.3e-9；r=5 1.3e-6；r=10 9.0e-5；**r=15 1.9e-4**。
  - 完整 40 octave 累积：**maxDiff=1.03e-2**（翻方块）。
- **根因**：高 octave 要求 ~35 位坐标精度（2^-35），float 只有 24 位（2^-23）。**坐标拆分救不了**——拆分把小数精度固定在 float 的 2^-23，超不出 float 硬上限。

### F6. GPU fp64 完美解决（逐位一致）
- **测什么**：InterpolatedNoiseSampler 完整链路用 GLSL double（`GL_ARB_gpu_shader_fp64`），GPU fp64 vs CPU double。
- **数据**：`maxDiff=3.192e-16 avgDiff=1.167e-16`（double 机器精度，逐位一致）。
- **结论**：NVIDIA 驱动 fp64 是标准 IEEE double（52 位）；base_3d_noise 用 GPU fp64 完全可行，FP32 的 1e-2 误差改善 16 个数量级。
- **实现要点**：fp64 是 Vulkan 1.1 core feature（`VkPhysicalDeviceFeatures.shaderFloat64`，非扩展）；smear 常量 `1.0e-7f` 需 `double(1.0e-7f)` 对齐 vanilla `1.0E-7F`。

## 最终分层方案

| 层 | 精度 | 误差 | 位置 |
|---|---|---|---|
| 宏观 2D 噪声（flat_cache 25 角点） | FP64 | — | CPU（低频） |
| base_3d_noise（InterpolatedNoiseSampler） | FP64 | 3.2e-16 | GPU fp64 |
| 块级三线性插值 + 算术 + spline | FP32 | ~1e-7 | GPU fp32 |
| 坐标（base_3d_noise） | double 直传 | — | GPU fp64 内做 maintainPrecision |

## 关键踩坑清单

1. `uint32_t` 下溢：`(i%32)-16` 中 i 是 uint32，减 16 下溢成 2^29。
2. GLSL 保留字 `input` 不能作参数名。
3. 「折叠后坐标 float 可精确表示」假设错（~2^24 时 float ulp=2）。
4. `VK_KHR_SHADER_FLOAT64_EXTENSION_NAME` 宏不存在——fp64 是 Vulkan 1.1 core feature，用 `shaderFloat64`。
5. vanilla `PerlinNoiseSampler` 是 all-double（非 float）——此前多处误写「vanilla 是 float」。

## 关键事实（纠正过的）

1. vanilla 1.20.1 `PerlinNoiseSampler` 全是 double（`double grad`/`perlinFade`/`MathHelper.lerp3`）。
2. `old_blended_noise` = `InterpolatedNoiseSampler`；`NormalNoise` = `DoublePerlinNoiseSampler`。
3. `maintainPrecision` 折叠到 [-2^24, 2^24] 是 double 精度设计，float 下此范围 ulp=2（小数全丢）。
