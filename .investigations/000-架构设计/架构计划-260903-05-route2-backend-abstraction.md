---
编号: 000
任务: 路线② GPU 角点生产接线（env 门控）+ 双线路后端抽象（CPU/GPU 可切换）+ cpu_backend 越界读顺带修复
任务类型: swe（编程/接线）+ 验证（端到端 vs Java）
模式档位: 重量（跨 Rust/C++、含抽象层设计 + 端到端验证）
状态: 批准 → 执行中（260903-05 重大转向已用户批准，见 §11 追加包 X2）
编号来源: NEXT_SESSION 260903-04 → 本包 260903-05（实际时间以 git 时间戳为准）
---

## 1. 全局视图

- **目标**：terrain.rs 密度角点源接 GPU（GpuDensityEngine via gpu_ffi.dll），env 门控默认关；生产路径抽象出 DensityBackend 切换点，路线①（CpuBackend 扁平表）后续在同一接口下实现，最终用户可切换。
- **用户拍板（260903-05）**：路线②先做、路线①做准备；GPU 线路面向服务器/高性能客户端，纯客户端可用性存疑但必须实现；两条都要能跑，可切换。
- **范围外**：N1 取证、H3 ×16 重测、glslc 原子更新判据（遗留待办，不入本包）；WG 生产门控行为默认不变。

## 2. 开工前置：交接结论廉价独立验证（M14/M11 纪律，MUST）

- **待验假设**：「双线程同 handle Mutex=0.61× ⇒ GPU 异步流水（Mutex 未真串行化）」为 draft 机制解释。
- **验证动作（≤1 轮）**：读 `GpuDensityEngine::fill` / gpu_ffi.cpp 源码确认同步语义（是否 submit+readback 异步、Mutex 持锁范围）；必要时单线程 1 chunk vs 8 chunk 耗时对比复测。
- **验证通过才可继承**；不通过则修正接线并发设计（可能需 per-handle 队列或双 handle）。

## 3. 任务拆解 & 依赖

1. **P0 前置验证**（见 §2）→ 产物：`.investigations/lossless-accel/route2-260903-05.md` 验证节
2. **P1 后端抽象设计**：density 角点获取抽象（trait / enum dispatch，点在 terrain.rs 角点采样入口）；CpuBackend 现路径零改动语义
3. **P2 GPU 接线**：Rust 侧 GpuBackend——LoadLibrary + **handle 级缓存**（create ~75s 不可每 chunk 付，进程级 Mutex 缓存）+ env 门控 `WG_GPU_DENSITY`（计划原定 WG_GPU_CORNER，实现时更名：语义钉定后为逐块批量而非角点，见 route2-260903-05.md P1/P2 节）默认关
4. **P3 越界读修复**：`cpu_backend.h` sampleInterpGrid 行1022/1024 y=320 grid[49] 越界（iy<minY 同理）→ 边界钳制/扩容，改后 `scan_cpp_anchors.py` invalid=0 + block_probe 回归
5. **P4 端到端验证**：GPU 门控开 vs Java 原版 **≥256 chunks 大样本**（端到端铁律：充分预热、取稳定中位数、排除冷启动），零退化铁律 + §9.7 三要素声明；GPU 关门控行为与主线完全一致（默认路径零退化）
6. **P5 知识库更新**（subagent 产出草稿 + 主会话应用）：错误台账 LL 续条（如有）、build-tooling/workflow-patterns 新发现（如有）、10 时间线本包节

## 4. 并行计划

- 第一波：P0 验证（主会话读源码/复测）与 P3 越界读修复（主会话收敛闭环）并行
- 第二波：P1+P2 实现（收敛型 swe 主会话直接做；抽象接口设计稿可交 subagent 评审）
- 第三波：P4 验证采集（主会话执行）→ worker 解读（subagent）
- 第四波：P5 落盘

## 5. 人工 HOOK 点

- 本计划批准（当前）
- P0 验证若推翻「异步流水」假设 → 并发设计变更（重大转向，MUST judge + 用户知会）
- P4 结果：GPU 路径性能不达标/有退化 → 回 Phase 0 重评估（用户拍板是否转向路线①加速）
- confirmed 授予（最终）

## 6. 风险 & 回退

- create 75s 缓存失效/进程生命周期问题 → 回退：进程级 once + 显式 destroy 测试
- GPU 在目标机器不可用（驱动/设备缺失）→ 门控路径 graceful fallback 到 CpuBackend + 明确日志，生产零影响
- Mutex 串行化实际成立（0.61× 另有原因）→ GPU 路径单线程吞吐仍 5µs/pt，先交付单线程接线，并发优化后置

## 7. judge 预置

- P0 验证推翻假设时：MUST judge（重大转向）
- P4 端到端结论 candidate 授予前：MUST judge（三源核对：artifacts 快照 + git diff + 验证记录）
- 收尾交付：MUST judge

## 8. fan-out 预置

- 潜在分叉：若 P4 出现 GPU vs Java 系统性 diff 且 ≥2 互斥机制候选（如角点坐标语义差 / f32 舍入链差）→ MUST fan-out .bN 并行，禁止主会话自推

## 9. 知识库更新

- 结论性 docs/10 时间线/discovered：subagent（core.worker，prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md）产出草稿 → 主会话应用 + 验证

## 10. 子角色介入点

- scout: 否（机制已明，无勘探需求；P0 属廉价验证非勘探）
- worker: P4 数据解读（subagent）；知识库草稿产出（subagent）
- fan-out: §8 预置条件触发时
- judge: §7 三节点
- knowledge: P5

## 11. 追加工作包 X2（260903-05 重大转向，用户已批准；judge 审查进行中）

- 触发：P2 逐块接线语义 PASS 但性能 25.4s/chunk = D24 已判死形态（历史一致性验证，非 FFI 缺陷）。
- X2 目标：shader 暴露 **5 channels @ cell corners**（valBuf 已含，gen/FFI 改动）→ GPU 1225 角点/chunk（~6ms，22-39× 域）→ Rust trilerp channels + compute_final_density（代码已有）= 精确 Java 语义（插值在 combine 前，非线性红线不触）。
- 任务：① dfc_gen/gen_final_density 生成 channels 输出 shader（新 spv，独立文件不动现有）② gpu_density_engine/ffi 增加 fill_channels 出口 ③ Rust GpuChannelDensity（slices 布局对齐 TranspilerDensity，sample_interp 复用）④ 通道顺序对拍探针（GPU slices vs CPU fill_cell_corner 角点）⑤ P4 端到端（合并原 P4）。
- 验证判据：通道对拍 major_diff=0（f32 口径）；端到端 GPU 门控 vs Java ≥256 chunks；门控关零退化。
