# I1/I2/I3 集成记录（2026-08-15，block_probe 集成立项 002）——✅ **confirmed（用户 2026-08-15 拍板）**

> 架构：`.investigations/000-架构设计/架构计划-gpu-integration.md`（002，用户 2026-08-15 批准）。
> 目标：DFC + CpuBackend + Vulkan 运行时接入 worldgen，8576/3200 零退化终验 + 吞吐对比。

## I1：Vulkan 运行时封装（✅ 完成，已验证）

- 产物：`vulkan-proto/vulkan_runtime.h`（header-only 组件，复制到 `worldgen/src/vulkan_runtime.h`）
- 接口：init / createPipeline(spv) / createBuffer / upload / makeDescriptorSet<N> / dispatch / readback / destroy / destroyBuffer
- 语义与 e2e 内联版逐位一致：12 binding storage buffer layout（binding 2 已删 OriginBuf 但保留占位）、host-visible+coherent memory、单 command buffer + fence、256 work items/组
- **验证**：e2e 改用组件后 maxDiff=3.128e-07 / avgDiff=1.097e-08 与内联版逐位一致；pipeline 90.9s 达标

## I2：GPU 密度引擎 + worldgen 接入（✅ 引擎验证通过，接入待构建验证）

- 产物：
  - `vulkan-proto/gpu_density_engine.h/.cpp`（PIMPL，复制到 `worldgen/src/`）
  - 接口：GpuDensityEngine(seed, spvPath) / fill(coords, n, out) / sample / splitTotal / perSample / splineBindBase
  - 语义：CpuBackend.split（CPU double 预拆分）→ Vulkan kernel（GPU float）→ 读回；与 e2e GPU 路径一致
- **PIMPL 原因（D23 候选）**：cpu_backend.h → density.h 的 static 成员定义（InterpolatedDF::nextId 等 L937-942）**非 inline**，多 TU include 会 LNK2005（worldgen_core 恰好单 TU 持有定义未触发）；引擎引入第二 TU 暴露。
  - **修复**：density.h L937-942 static 定义加 `inline`（C++17 inline 变量，语义与单 TU 完全一致，零运行时影响）
- **引擎验证**（gpu_fill_probe）：maxDiff=3.128e-07 / avgDiff=1.097e-08 与 DensityBuilder 参照逐位一致；splitTotal=8672/perSample=352/splineBindBase=6 对齐生成器
- **worldgen 接入**（worldgen_api.cpp）：
  - WorldgenHandle 加 `gpu` 字段（`#ifdef CORESWAP_GPU_ENABLED` 条件）
  - wg_create 尾部：env `WG_GPU_FILL=1` 时构造引擎（spv 从 gpu-assets 读，缺文件 CPU fallback）
  - wg_fill_density：GPU 分支（批量坐标 → fill → float 转 double 输出）/ CPU 分支（默认，零退化）

## I3：生成器产物纳入构建（✅ 完成）

- 目录约定：`worldgen/gpu-assets/`（cpu_backend.h + final_density.spv）
- gen_final_density.py 同步 cpu_backend.h 到 gpu-assets（spv 由 glslc 编译后手动复制或脚本化）
- CMake：worldgen_core 加 gpu_density_engine.cpp/vulkan_runtime.h；`if(DEFINED ENV{VULKAN_SDK})` 条件加 Vulkan include/lib + CORESWAP_GPU_ENABLED 定义（无 SDK 时 CPU-only 构建）

## 待办

- [x] I4a：8576 CPU 路径零退化（density.h inline 修正后）——✅ 99.9994% 与基线一致（block_probe CPU 路径实测，2026-08-15 重跑落盘 cmd-output/blockprobe-I4a-cpu-20260815-191247.txt：TOTAL match=3538922/3538944 99.9994%）
- [x] I4b：GPU 引擎接入不破坏——✅ 块级生成（fillOneChunkCore）恒走 CPU finalDensity->sample，GPU 引擎（WG_GPU_FILL=1）仅构造 + wg_fill_density 批量接口生效，块级路径不受影响（fallback 机制 + WG_GPU_FILL=1 下 block_probe 运行不崩溃，2026-08-15 重跑落盘 cmd-output/blockprobe-I4b-gpu-*.txt）；**注意：I4b 不是「GPU 参与块生成的逐位验证」——块级正确性由 CPU 路径保证，GPU 引擎自身的逐位正确性由 e2e（3.128e-07）+ domain probe（9.9e-9）验证**（judge P1-2 修正表述）
- [x] I5：吞吐对比——✅ GPU **22.26x / 24.26x / 34.46x / 39.29x**（1/4/16/64 chunks，2026-08-15 重跑落盘 cmd-output/throughput-I5-*.txt）；**D23 修复后 maxDiff 1.06e-6 / 2.86e-6 / 4.42e-6 / 8.26e-6（正确性恢复，原 0.2-0.5）**
- [x] **D23 根因 + 修复（最重要）**：GPU/sim 大坐标域错值——根因 = spline_eval 边界外推遇嵌套 value 直接 0.0（未递归，vanilla Spline.apply L259/261 应递归）；GPU 修复（while 栈 stage 4/5 压子帧递归）后 (784,160,-408) 0.045→-0.458（diff 9.9e-9）、e2e 3.128e-07 零回归、I5 maxDiff 1e-6。完整记录 gpu-accel-errors.md D23。
- [x] sim 诊断脚本深层递归修复（dbg_full_sim.py stage 6/7 + outSlot 返回地址不被覆盖 + 回填不改父帧 stage）——✅ sim eval_df(784,160,-408)=-0.458333333、sim vs e2e-A5 全量对拍 maxDiff=5.7e-9；两 bug（outSlot 被 -1 覆盖 / 回填覆盖父帧 stage 跳 v1）见 gpu-accel-errors.md D23；**judge P1-1 追补：stage 6/7 完成路径同款 stage 覆盖 bug（L289/302 原 `stageStack[ps>>1]=2`）已修，递归版参照对拍 1344 组合 0 mismatch（verify_p11_recursive.py）**
- [x] judge 审查 + 知识库更新（subagent 草稿）——✅ judge 审查完成（4 P1 全闭合：P1-1 sim stage6/7 回填覆盖→已修+递归对拍 1344 组合 0 mismatch、P1-2 I4 表述修正+I4a/I4b 落盘、P1-3 I5 探针重跑落盘、P1-4 timewise+discovered+判错经验三载体应用）；**✅ 用户 2026-08-15 拍板 confirmed**（.artifacts 9 条升 confirmed）
