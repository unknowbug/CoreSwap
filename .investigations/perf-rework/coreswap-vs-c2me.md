# CoreSwap GPU 方案 vs C2ME-OCL 差异对比（2026-08-13）

> 汇总本阶段（GPU 加速预研）发现的 CoreSwap 与 C2ME-OCL 的全部关键差异。
> 前置依据：fp32-experiment.md、gpu-route-decision.md、vulkan-dfc-research.md、c2me-dfc-review.md。

## 一、总览

| 维度 | CoreSwap（我们） | C2ME-OCL | 谁更优 |
|---|---|---|---|
| 计算 API | **Vulkan compute** | OpenCL | 我们（跨厂商 + 驱动生态较稳） |
| 精度策略 | **分层：宏观 F64（CPU）+ 高频 F32（GPU）** | 全 double + 关 FMA | 我们（消费卡吃满 FP32） |
| FMA | 待测（用户：先跑测试） | `#pragma OPENCL FP_CONTRACT OFF` 关闭 | 待定 |
| 噪声精度 | 高频 3D 噪声 FP32（相对 double ~5e-7，方块零影响） | 噪声 double（对齐 vanilla double） | 我们（FP32 吃满） |
| 坐标折叠 | FP64 放 CPU + 拆 int32 整数/float 小数给 GPU | double `maintainPrecision`（GPU 内） | 我们（绕开 Vulkan FP64 短板） |
| DFC 后端 | 自写（DF→GLSL 或复用 C2ME 前端+clspv） | DF→OpenCL C→运行时 JIT | 平（我们 AOT 更轻） |
| 平台 | **Fabric/Forge 双平台** | 只 Fabric（Forge 版停 1.16.5） | 我们（Forge 是硬前提） |
| 适用场景 | 批量预生成（分层精度吃满 FP32） | Chunky 批量（被 FP64 阉割拖累） | 我们 |

## 二、精度策略差异（核心差异）

这是最本质的一条。C2ME 为了对齐 vanilla **死守全 double**：

- C2ME：所有 binary/unary/noise emitter 默认 `ValuesMethodDefF64`，噪声采样 double，`#pragma OPENCL FP_CONTRACT OFF` 关 FMA → 要求 `cl_khr_fp64`，**消费卡 FP64 被阉割到 1/64，性能吃不满**。
- CoreSwap：**「宏观 F64 + 高频 F32」分层**——FP32 实验证明：
  - 计算内部 float 误差 ~1e-7，方块判定零影响（近坐标 block_probe 零新增 mismatch）。
  - 唯一的损在「远坐标的坐标精度」（坐标 float 化 → density 差异 ~1e-3），而这通过「坐标折叠放 FP64 CPU」绕开。
  - → GPU 端高频噪声/算术/插值全 FP32，**消费卡满血**；只有低频宏观噪声 + 坐标折叠付 FP64 税（CPU 算，量小）。

**C2ME 缺的正是这层分层**——它连 F32 分层都没敢碰（`ToF32Node` 只在 spline 固定值用了一次）。

## 三、API 差异（Vulkan vs OpenCL）

- C2ME 选 OpenCL 的理由：① 作者熟悉 ② Vulkan 无类型指针要 1.4 ③ Vulkan 不明确 FP64 精度 ④ Vulkan 缺 correctly rounded div/sqrt。
- CoreSwap 选 Vulkan 的理由（用户拍板）：跨厂商、MC 客户端可复用渲染管线、不用厂商绑定的 CUDA。
- **差异的关键**：Vulkan 的 FP64 短板（精度不明确 + 缺正确舍入 div/sqrt）→ 我们靠「FP64 放 CPU」绕开；C2ME 靠「改投 OpenCL」绕开。两者殊途同归，但我们的路线保留了 Vulkan 的跨厂商 + 双平台优势。

## 四、有损性差异

| 有损点 | C2ME | CoreSwap |
|---|---|---|
| 噪声精度 | double（对齐 vanilla double，非差异） | 高频 FP32（相对 double ~5e-7，方块零影响） |
| fmax/fmin NaN 语义 | `fmax(NaN,x)=x` ≠ `Math.max(NaN,x)=NaN` | 待定（GPU 语义需逐一核对） |
| flat_cache miss | `__builtin_trap()` 崩溃（脆弱） | 宏观噪声 CPU 算，无 miss 问题 |
| 远坐标 | double 精度可保 | 坐标折叠 FP64 可保 |
| 驱动 workaround | 大量 Intel/AMD blocklists（OpenCL 生态差） | Vulkan 驱动较稳（待验证） |

## 五、平台差异

- C2ME：DFC + OpenCL 加速**只在 C2ME-fabric**；C2ME-forge 停在 1.16.5（0.1-SNAPSHOT，无 dfc/opencl 模块）。
- CoreSwap：DFC 前端（DF→AST→代码生成）是纯 Java 平台无关；**Vulkan 运行时用 LWJGL 3 自研，天然同时支持 Fabric/Forge** → 满足 Forge 硬前提。

## 六、复用边界（结论）

| 项 | 判定 |
|---|---|
| DF 树 → AST 骨架（McToAst + 无损 opto passes） | ✅ 参考/复用 |
| flat_cache「CPU 预填充 + GPU 只读」架构 | ✅ 借鉴（= 我们「宏观噪声 CPU 算」） |
| 精度分层（宏观 F64 + 高频 F32） | 🆕 自己写（C2ME 没有） |
| Vulkan 后端（GLSL 或 clspv） | 🆕 自己写 |
| 驱动 workaround 层 | ❌ 不继承（OpenCL 的坑） |
| Fabric mixin 运行时 | ❌ 不继承（换 LWJGL 3 自研） |

## 七、一句话结论

> C2ME 的 DFC 前端是「正确但保守」的参考——骨架可借鉴、flat_cache 思路可复用；但它死守全 double + 关 FMA，**没做「宏观 F64 + 高频 F32」分层**，所以消费卡被 FP64 阉割拖累。我们想出的分层精度（宏观噪声 + 坐标折叠放 FP64 CPU，高频噪声/算术/插值吃 FP32 GPU）正是它缺的关键一块，也是消费卡吃满 FP32 的前提。
