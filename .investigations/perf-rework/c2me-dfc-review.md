# C2ME DFC 源码审查（2026-08-13）

> 审查对象：RelativityMC/C2ME-fabric（dev/26.2.0，master）+ C2ME-forge。
> 目的：① Forge 兼容性判定 ② DFC 实现质量（可否复用骨架、有无有损优化）。

## 一、仓库结构

- **C2ME-fabric**（主仓库，多模块）：`c2me-opts-dfc`（DFC 前端，DF 树 → AST → 代码生成）+ `c2me-opts-accel-opencl`（OpenCL 运行时：设备枚举/buffer 缓存/驱动 workaround）。
- **C2ME-forge**（独立仓库）：**停在 1.16.5（0.1-SNAPSHOT），单模块 `src`，无 dfc/accel-opencl**。DFC + OpenCL 加速只在 Fabric 版实现。

## 二、Forge 兼容判定（用户硬前提）

| 层 | 平台依赖 | Forge 可移植性 |
|---|---|---|
| DFC 前端（`common/ast/` + `common/gen/`，McToAst→AST→OpenCL C） | 纯 Java + **仅 Minecraft 原版类**（`net.minecraft.world.gen.densityfunction.*`、`ChunkNoiseSampler`、`Spline`、`InterpolatedNoiseSampler`） | ✅ 可移植（改映射名即可） |
| accel-opencl 运行时（mixin + duck 接口 `ChunkNoiseSamplerExtension` 等） | Fabric mixin 系统 + duck 注入 | ❌ Fabric 特有，移植需重写 |

**结论**：C2ME 的 DFC/OpenCL 方案**目前只 Fabric 能跑**（Forge 版停在 1.16.5 无此模块）。但我们只需要复用 **DFC 前端**（平台无关），运行时本来就要用 Vulkan 自己写 → **不受 Fabric 绑定影响，不构成否决**。

## 三、精度策略（关键）

1. **全 double（F64）**：所有 binary/unary/noise emitter 默认 `ValuesMethodDefF64`，Add→`+`、Max→`fmax`、Squeeze→`clamp`。
2. **显式关 FMA**：`c2me_opencl_ext_math.cl` 第 28 行 `#pragma OPENCL FP_CONTRACT OFF`——C2ME 知道 FMA 会引入与 vanilla 的精度差异，选择关闭保精度（而非开 FMA 提速）。
3. **噪声采样用 double**（`math_perlinFade`/`math_lerp` 全 double），而 vanilla `PerlinNoiseSampler` 内部是 **float** → C2ME 与 vanilla 有 ~1e-7 级差异（double 更精确 ≠ 对齐）。
4. **没有「宏观 F64 + 高频 F32」分层**：`ToF32Node` 仅在 spline 的 `FixedFloatFunction` 处用（`ConstantF32Node`），其余全 double。

## 四、有损/脆弱点（对应「C2ME 损得严重」）

1. **`fmax`/`fmin` vs `Math.max`/`Math.min`**：OpenCL `fmax(NaN,x)=x`，vanilla `Math.max(NaN,x)=NaN`——NaN 语义差异（density 里 NaN 少见，但存在）。
2. **flat_cache miss 即 `__builtin_trap()` 崩溃**：`CacheLikeNodeOpenCLCEmitter` 的 FLAT_CACHE/CACHE2D/INTERPOLATED 分支，缓存 miss 直接 trap（不重算）——flat_cache 必须 CPU 端预填充，否则崩。这是「复杂 datapack 崩溃」的根因。
3. 兼容性矩阵里大量 Intel/AMD 驱动 workaround（`workarounds/{intel,amd,mesa,nvidia}` blocklists）——印证 OpenCL 驱动生态差。

## 五、可复用 vs 需自研

| 项 | 判定 |
|---|---|
| DF 树 → AST 骨架（McToAst 的 DF 类型映射 + 语义保持的 opto passes） | ✅ 可参考（FoldConstants/BranchElimination 用 Java double 语义，无损） |
| flat_cache 宏观噪声「CPU 预填充 + GPU 只读」架构 | ✅ 与我们「宏观噪声 CPU 算」思路一致，可借鉴 |
| 精度分层（宏观 F64 + 高频 F32） | ❌ C2ME 没有，**要我们自己加** |
| Vulkan 后端（C2ME 只有 OpenCL C 后端） | ❌ 要自己写（clspv 或自写 GLSL） |
| 驱动 workaround 层 | ❌ 不继承（那是 OpenCL 的坑，Vulkan 不同） |

## 六、结论

1. **用户的直觉对**：C2ME 在「底层算法优化」上比我们的方案**保守**——它死守全 double + 关 FMA（所以要求 `cl_khr_fp64`，消费卡被 FP64 阉割拖累），**没有做「宏观 F64 + 高频 F32」的分层**。我们的分层方案正是它缺的那块，也是它在消费卡上吃满 FP32 的关键。
2. **复用边界明确**：拿 DFC 前端的「DF 树 → AST → 代码生成」骨架 + flat_cache「CPU 预填充」思路；精度分层 + Vulkan 后端自己写。
3. **Forge 兼容**：复用 DFC 前端（平台无关）不碰 Forge 雷区；Vulkan 运行时自研天然平台无关（LWJGL 3 同时支持 Fabric/Forge）。
