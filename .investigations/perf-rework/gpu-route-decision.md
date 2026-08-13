# GPU 加速路线决策（2026-08-13，用户拍板）

> 结论性决策记录：CoreSwap 世界生成 GPU 加速的技术选型 + 精度分层方案。
> 支撑依据见 fp32-experiment.md（FP32 有损探针）+ C2ME-OCL 调研。

## 一、API 选型：Vulkan compute shader

- **选 Vulkan**（用户拍板），不用 CUDA / OpenCL。
- 理由链：
  - 不用 CUDA：厂商绑定（NVIDIA only）。
  - 不用 OpenCL：驱动/生态兼容性差（C2ME-OCL 崩溃矩阵：Intel 集显、AMD 26.5.1 驱动、复杂 datapack 均崩溃）。
  - Vulkan 是跨厂商现代图形 API，MC 客户端侧可复用渲染/计算管线。
- **Vulkan 的已知短板（需绕开）**：
  1. Vulkan 不明确规定 FP64 精度（只保证「至少 FP32 的精度」）。
  2. Vulkan 至今缺 correctly rounded 的 `division` / `sqrt`。
  3. 无类型指针需 Vulkan 1.4（降低硬件兼容性）。
  → 通过「分层精度 + FP64 放 CPU」绕开（见下）。

## 二、分层精度方案（核心）

**目标：GPU 吃满 FP32（消费卡满血），FP64 只付少量税（CPU 一次算好，低频）。**

| 层 | 精度 | 位置 | 理由 |
|---|---|---|---|
| 坐标折叠 maintainPrecision | FP64 | **CPU 一次算好** | 远坐标精度命门；计算量极小（每采样 1 次除法+取整） |
| 宏观 2D 噪声（continentalness/erosion/ridges） | FP64 | CPU（flat_cache 低频缓存） | 每 chunk 每实例 25 角点，低频，算一两次不亏 |
| 高频 3D 噪声（base_3d_noise）+ 算术 + 插值 | FP32 | GPU | 高频（interpolated buildGrid 1225 角点 × 6 实例），性能关键；误差对方块判定鲁棒 |

**关键机制（绕开 Vulkan FP64 短板）**：
CPU 用 FP64 把大坐标折叠成 `[-2^24, 2^24]` 的小坐标，GPU 只接收折叠后的小坐标 → GPU 端坐标是 float 可精确表示的小值 → 既绕开「Vulkan FP64 精度不明确」，又绕开「远坐标 float ulp 爆炸（3000 万时 ulp≈512）」。

## 三、支撑依据

### 1. FP32 有损探针实验（fp32-experiment.md）
- **计算内部 float**（噪声/算术/插值舍入）：误差 ~1e-7，近坐标 block_probe 零新增 mismatch（99.9994%/99.9997% 保持）。
- **坐标 float**（`pos.x × scale` 舍入）：远坐标（块 3000 万）→ finalDensity 差异 ~1e-3（18/94 行），可能翻转边界块。
- **结论**：FP32 的「损」来自**坐标精度**（远坐标），不是**计算内部精度**；maintainPrecision 折叠在 FP32 下救不回（折叠前的坐标已舍入）。

### 2. C2ME-OCL 调研（有损方案，仅参考不照搬）
- C2ME 选 OpenCL 而非 Vulkan 的三理由：Vulkan FP64 精度不明确 / 缺 correctly rounded div+sqrt / 无类型指针需 1.4。
- C2ME 有损：官方声称「应当与原版相同」，实际 GPU 驱动 FP64 实现差异 + FMA 导致有损（兼容性矩阵大量「崩溃/损坏世界生成」）；**用户实测损得严重，勿以它为准**。
- C2ME 只实现「噪声 + 生物群系」阶段，主要靠 Chunky 批量预生成（1200~2500 cps）——印证「GPU 赢在批量预生成，非实时逐 chunk」。
- C2ME 死守 FP64（cl_khr_fp64），在消费卡吃的是 FP64 被阉割性能；本方案的「分层精度」是比它更聪明的路线。

## 四、数据流开销实测（CUDA 探针，2026-08-13）

> 用机器上已装的 CUDA/nvcc 实测同一块 GPU 的硬件数据（PCIe 传输 + kernel 启动是硬件层事实，与最终用 Vulkan 无关，可作参考）。
> 探针：`.investigations/perf-rework/gpu_probe.cu`。

| 项 | 实测值（RTX 4060 Laptop，PCIe 4.0 x8） |
|---|---|
| 单 chunk 往返（H2D 88KB + kernel + D2H 384KB + sync） | **144 µs** |
| D2H 384KB（density 输出） | 57 µs（实测带宽 6.9 GB/s） |
| kernel 启动延迟（pipeline 摊薄） | 7.1 µs |
| busy kernel（98304×200 乘加，模拟 FP32 负载） | 10 µs |

**对比 CPU density 47 ms/chunk（spline 扁平化后）**：GPU 单 chunk 往返 144 µs = **326× 快**。

**结论**：
1. **数据流开销（144 µs）远小于 CPU 计算（47 ms），不是瓶颈** —— 分层方案的传输量（in 88KB + out 384KB）完全可忽略。
2. **但单 chunk 的 GPU 计算（~10 µs）只占往返（144 µs）的 ~7%** → 单 chunk 场景 GPU 利用率极低，固定开销（PCIe 传输 57µs + 同步）主导。
3. **批量预生成才是 GPU 最优场景**（一次 H2D 批量坐标 + 一次 kernel 算多 chunk + 一次 D2H 批量结果 → 固定开销摊薄，GPU 满负荷）。实时逐 chunk 虽快 326× 但 GPU 空转。

## 五、下一步

1. 调研 Vulkan compute shader 的 DF 树扁平化（DFC）实现方式（compute pipeline + storage buffer + kernel 生成）。
2. 验证「CPU FP64 折叠 → GPU 吃小坐标」的数据流（每 chunk 传输量、kernel 启动延迟 vs 实时逐 chunk 的适配性）。
3. 明确 GPU 加速的适用场景：批量预生成（服务器/Chunky 式）优先，实时逐 chunk 需评估 kernel 启动 + 传输延迟。
