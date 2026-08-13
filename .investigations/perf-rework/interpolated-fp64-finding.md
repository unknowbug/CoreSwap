# 重大发现：base_3d_noise 的 /o 放大结构要求 double，FP32 不可行（2026-08-13）

> 推翻此前「高频 3D 噪声 FP32 方块零影响」的核心假设。探针：`interpolated_probe.py` + `diag_interpolated.py`。

## 一、两个对象搞混了

查证 vanilla 源码（`DensityFunctionTypes.java` L46）确认：**`old_blended_noise` = `InterpolatedNoiseSampler`**（16+16+8 个 octave + y 轴 smear + 插值），**不是** DoublePerlinNoiseSampler（first+second×1.018，那是 NormalNoise）。

此前 `octave.comp`/`octave_probe.cpp` 实现的是 **DoublePerlinNoiseSampler**——对象错了。虽然「多 octave 叠加不放大误差」的结论本身仍成立，但 base_3d_noise 的真实结构有额外的精度杀手。

## 二、真正的精度杀手：`/o` 放大

`InterpolatedNoiseSampler.sample` 的结构：

```java
for (int r = 0; r < 16; r++) {
    ...
    l += pn.sample(maintainPrecision(d*o), maintainPrecision(e*o), maintainPrecision(f*o), j*o, e*o) / o;
    o /= 2.0;   // o = 1, 1/2, 1/4, ..., 1/2^15
}
```

**高 octave（r 大，o=2^-r 小）的噪声贡献被 `1/o = 2^r` 放大**。这要求高 octave 的坐标有极高的小数精度，否则误差被放大 2^r 倍。

## 三、float 的硬限制

| 项 | float（24 位） | vanilla double（52 位） |
|---|---|---|
| 高 octave（r=15）坐标 ~2^17 的小数精度 | 2^-23（f32 小数精度） | 2^-35（ulp(2^17)） |
| 除以 o=2^-15 后单 octave 贡献误差 | **~1.9e-4** | ~9.5e-7 |
| 40 octave 累积（实测） | **maxDiff=1.03e-2** | （基线，误差 ~1e-6 量级） |

**坐标拆分（int32 整数 + float 小数）解决不了这个问题**——拆分让小数精度固定为 float 的 2^-23，而高 octave 需要 2^-35（超 float 24 位硬上限）。float 无论怎么拆都不够 35 位。

## 四、实测

- 完整 InterpolatedNoiseSampler 拆分 float vs double：`maxDiff=1.032e-2 avgDiff=1.489e-3`（远坐标 + y 小数，N=200）。
- 单 octave 诊断（`diag_interpolated.py`）：r=15 贡献误差 1.9e-4，40 octave 累积到 1e-2。
- 对照：此前单 octave Perlin（低 octave o=1）4.9e-7、DoublePerlinNoiseSampler 1.4e-7——都**没有 /o 放大结构**，所以 FP32 成立；但 base_3d_noise 有，FP32 不成立。

## 五、修正后的分层方案

| 层 | 精度 | 理由 |
|---|---|---|
| 宏观 2D 噪声（flat_cache 25 角点） | FP64（CPU） | 低频，划算 |
| **base_3d_noise（InterpolatedNoiseSampler，7350 角点×40 octave）** | **FP64（GPU fp64 或 CPU）** | /o 放大要求 35 位精度，float 不够 |
| 块级三线性插值（98304 块） | FP32（GPU） | 最高频、无 /o 放大，插值 FP32 足够 |
| 算术 + spline | FP32（GPU） | 无 /o 放大 |

**关键**：base_3d_noise 必须 FP64（NVIDIA 驱动 fp64 = 标准 IEEE double，精度足够；Vulkan fp64 规范虽只保证 ≥fp32，但实际 NVIDIA 实现是完整 double）。

**成本重估**：RTX 4060 fp64 ~0.24 TFLOPS（1/64 阉割），base_3d_noise ~14.7M flops/chunk → GPU fp64 算 ~61µs/chunk，加数据流 144µs ≈ 200µs/chunk，仍比 CPU 47ms 快 ~235×。

## 六、对之前结论的影响

1. 「高频 3D 噪声 FP32」→ **修正为「块级插值 FP32 + base_3d_noise FP64」**。
2. `octave.comp`/`octave_probe.cpp` 实现对象错误（DoublePerlinNoiseSampler ≠ base_3d_noise），需改成 InterpolatedNoiseSampler。
3. 坐标拆分方案仍正确（低 octave / DoublePerlinNoiseSampler 适用），但对 base_3d_noise 高 octave 不够，需 FP64。
