# Vulkan DFC 调研（2026-08-13）

> 调研目标：Vulkan compute shader 如何做 DF 树扁平化（DFC）——即把 vanilla 的 noise_settings JSON 密度树编译成 GPU kernel。

## 一、Vulkan compute 基础（DFC 的地基）

- compute shader 用 GLSL/HLSL 写，编译成 SPIR-V（`glslc` / `glslangValidator`）。
- 数据流：storage buffer（大数组，如折叠坐标 in / density out）+ descriptor set + `vkCmdDispatch` + memory barrier。
- 分层精度：FP32 在 Vulkan shader 是满血；FP64 精度不明确（前已记），故 FP64 放 CPU。

## 二、DFC 的两条实现路径

### 路径 A（推荐）：复用 C2ME DFC 前端 + clspv 转 SPIR-V

- C2ME 已有 **DFC 前端**（DF 树 → OpenCL C 源码生成，含 optimization passes）。
- **clspv**（Google/Khronos，基于 Clang/LLVM）把 OpenCL C 编译成 Vulkan-flavor SPIR-V：
  - 自动生成 descriptor mapping（`__global float* input` → set=0 binding=0 Storage Buffer）。
  - **AOT**：运行时只需加载小 SPIR-V 二进制，无需带 OpenCL 编译器。
  - 指针：clspv 用 `VK_KHR_variable_pointers`（core 1.1）模拟 OpenCL 指针算术；**Vulkan 1.4 的 Buffer Device Address 让它更高效**。
- 代价：clspv 只支持 **OpenCL C 子集**，C2ME DFC 输出的 OpenCL C 需适配（去掉子集外特性）。

### 路径 B：自写 DF 树 → GLSL 编译器

- DF 树 → GLSL 源码 → `glslc` → SPIR-V。需自己实现 DF 树到 GLSL 的代码生成（工作量大，但无 clspv 子集限制）。

## 三、现成先例：GPU Land mod（Vulkan worldgen，但非 DFC）

- **GPU Land**：用 Vulkan compute shader 做 MC worldgen 的 alpha mod（`modrinth.com/mod/gpuland`）。
- 关键事实：
  - 作者自述「I don't even know much about noise and vanilla world generation」——**手写单个 GLSL noise shader，不复刻 vanilla DF 树**（非 DFC）。
  - 单个 shader-pass，把 blockpos 翻译成 ≤256 的 block palette 索引。
  - **要求 Vulkan 1.4** + `GL_EXT_shader_explicit_arithmetic_types_int8` + `GL_EXT_shader_8bit_storage`。
  - 单机 only（不支持专用服务器）、最小多线程（依赖 C2ME）。
- 意义：证明「Vulkan compute shader 做 MC worldgen」可行，但它**不是我们的目标**（它不复刻 vanilla，我们有 vanilla DF 树要对齐）。

## 四、关键约束（调研确认）

1. **Vulkan 1.4 是硬要求**（clspv 的 Buffer Device Address 优化 + GPU Land 的 8bit 扩展都依赖它）——印证 C2ME 作者「无类型指针要 1.4」的说法。
2. **FP64 精度不明确** → 分层方案（FP64 放 CPU）是正确绕法。
3. **DFC 是运行时 JIT 还是 AOT**：C2ME 是运行时 JIT（带 OpenCL 编译器）；走 Vulkan 更现实是 **AOT**（预编译 vanilla DF 树 → SPIR-V，随 datapack 分发；自定义 datapack 需重新编译）。

## 五、推荐路径 + 下一步

- **推荐路径 A**：复用 C2ME DFC 前端（DF 树 → OpenCL C）+ clspv（→ SPIR-V）→ Vulkan compute pipeline。
- 下一步验证：
  1. C2ME DFC 输出的 OpenCL C 是否落在 clspv 支持的子集内（决定是否要裁剪 DFC 前端）。
  2. 分层数据流映射：CPU 折叠坐标 + 宏观噪声（FP64）→ storage buffer in；GPU 算高频 3D 噪声 + 算术 + 插值（FP32）→ storage buffer out。
  3. 场景定调：批量预生成（Chunky 式）优先，实时逐 chunk 需评估 GPU 空转。
