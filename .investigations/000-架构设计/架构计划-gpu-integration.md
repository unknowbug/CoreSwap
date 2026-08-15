---
编号: 002
任务: GPU 密度引擎集成——DFC + CpuBackend + Vulkan 运行时接入 worldgen，8576/3200 零退化终验 + 吞吐对比
任务类型: 集成（C++ 运行时）+ 验证（正确性/性能）
模式档位: 重量
状态: 待批准（2026-08-15 用户拍板「收尾 → 进入 block_probe 集成立项」）
日期: 2026-08-15
---

# 架构计划：block_probe 集成立项（GPU 密度引擎接入 worldgen）

## 0. 背景与目标

- G4 A 方案已达标（pipeline 编译 903.4s → 67-102s，正确性 maxDiff=3.128e-07 与参照逐位一致，见 001 计划 §10 回填）。
- 下一步（NEXT_SESSION 待办 3 / 001 计划遗留项）：**DFC + CpuBackend + Vulkan 运行时接入 worldgen**，8576/3200 零退化终验 + 吞吐对比。
- 集成面摸底（2026-08-15 主会话）：
  - block_probe 采样入口 = `wg_router_sample(handle, "final_density", x, y, z)`（纯 C API，worldgen_api.h）；
  - 实际 density 批量求值 = `wg_fill_density`（每 chunk 网格 y→z→x 布局）→ 内部走 C++ 引擎（density.h 树解释器）；
  - **接入点 = wg_fill_density / wg_sample_density 内部**：把 CPU density 求值替换为「CpuBackend.split（CPU 拆分坐标）→ Vulkan kernel（GPU 求值）→ 读回」；
  - 验收载体 = block_probe 8576（99.9993% 对齐基线）+ 3200（干净参照零退化铁律）。

## 1. 范围（含明确不做什么）

**做**：
- I1：Vulkan 运行时封装（把 e2e 的 Vulkan 初始化/pipeline/descriptor/buffer 逻辑抽成可复用组件 `vulkan_runtime`，供 worldgen 链接）。
- I2：接入 `wg_fill_density`——final_density 分支走 GPU 路径（CpuBackend.split CPU 预拆分 → GPU kernel → 读回）；其余分量保持 CPU。
- I3：worldgen 构建集成——生成器产出（final_density.comp/spv + cpu_backend.h + spline 表）纳入构建产物管理。
- I4：8576/3200 零退化终验（block_probe 逐位对比 GPU vs CPU 引擎）。
- I5：吞吐对比（GPU vs CPU，批量 chunk 场景，复用 batch_probe 方法论）。
- I6：judge 审查 + 知识库更新（subagent 产出草稿）。

**不做（本阶段）**：
- ❌ 不改 CPU 侧参照（density_builder.h SplineDF / worldgen 现有路径）——零退化铁律（GPU 是新增路径，CPU 保持权威）。
- ❌ 不做多 chunk GPU 并行调度优化（I5 只测单 kernel 批量吞吐，调度优化后续立项）。
- ❌ 不改 JNI/Fabric 侧（纯 C++ 集成先行，mod 侧接入后续立项）。
- ❌ 不动 block_probe 参照数据（8576/3200 数据已验证，探针三查铁律）。

## 2. 任务拆解

| # | 子任务 | 产物 | 验证 |
|---|---|---|---|
| I1 | Vulkan 运行时封装（初始化/pipeline/descriptor/buffer 复用组件） | vulkan_runtime.h/.cpp | 编译通过 + e2e 复用组件回归一致 |
| I2 | wg_fill_density 接入 GPU 路径（CpuBackend.split + kernel + 读回） | worldgen GPU 路径代码 | 单 chunk 采样 vs CPU 参照一致 |
| I3 | 生成器产物纳入构建（spv/cpu_backend.h/spline 表） | CMake 集成 | 构建可复现 |
| I4 | 8576/3200 零退化终验 | block_probe 输出 | 8576 零新增 mismatch + 3200 零退化 |
| I5 | 吞吐对比（GPU vs CPU） | 计时记录 | 批量场景 GPU 收益量化 |
| T7 | 知识库更新（subagent 草稿）+ judge | docs diff + review | 一致性 |

## 3. 验证方式

- **正确性**：I2 阶段逐 chunk 对比 GPU vs CPU 引擎（采样点级 maxDiff）；I4 阶段 block_probe 8576/3200 零退化（探针三查：seed/坐标/文件）。
- **性能**：I5 用 WG_PROFILE 或 chrono 计时，对比 CPU density 基线（47ms/chunk 量级）vs GPU 批量（batch_probe ~10000 chunk/s 预估）。
- **门禁**：改动核心函数后 scripts/scan_cpp_anchors.py invalid=0；@anchor.test source 指向 block_probe 回归。

## 4. 风险 & 回退

- **R1 GPU 路径引入正确性回归**（浮点/布局差异）→ 逐 chunk 对比先行（I2），任何差异回退 CPU；8576/3200 终验兜底。
- **R2 Vulkan 初始化成本**（每 handle 一次，~70s pipeline）→ 单 chunk 场景不可接受 → I5 量化批量阈值；必要时 pipeline cache（001 计划 D 方案）正交叠加。
- **R3 构建集成破坏现有 worldgen** → 新增文件独立编译单元，CMake 增量；回退 = 移除 GPU 路径分支。
- **R4 吞吐不达预期** → 如实报告（诚实声明），不掩盖；GPU 甜点是批量预生成（F1 结论），单 chunk 不适用属预期。

## 5. 人工 HOOK 点

- **I2 完成**：GPU 路径单点正确性确认后，用户拍板是否进入 I4 终验（或先 I5 吞吐）。
- **I4/I5 数据落盘**：用户拍板 达标收尾 / 继续优化 / 回退 CPU-only。
- 重大方向变更：暂停回 Phase 0。

## 6. judge 步骤预置

- 节点：I2 GPU 路径正确性数据落盘后 | MUST | 三源核对（worldgen diff + block_probe 对比输出 + 计时）
- 节点：I4 终验 + I5 吞吐落盘后 | MUST | 三源核对（含 8576/3200 基线对比）
- 节点：交付闭环 | MUST

## 7. fan-out 步骤预置

- I2 若出现多互斥机制候选（GPU 差异来源：拆分/布局/插值）→ MUST fan-out 并行 .bN。
- I4 若 8576 新增 mismatch：分叉候选（GPU 语义差 / 参照污染 / 集成 bug）→ 先三查排除参照污染，仍互斥则 fan-out。
- 当前 I1/I3 无已知分叉（单一路径）。

## 8. 知识库更新（subagent 产出草稿）

- 10 时间线：2026-08-15/16 集成条目。
- gpu-accel-errors.md：新坑按「现象→根因→定位→修复→教训」追加（先读 SUBAGENT-KNOWLEDGE-GUIDE.md）。
- 主题篇（若产出集成结论）：versions/1.20.1/docs/ 对应篇。
- 通用模式：knowledge/discovered/（若发现跨项目模式）。

## 9. 子角色介入点（全部预置）

- scout：I2 前若「GPU 接入点内部机制未明」（wg_fill_density 现有实现细节）→ 勘探（管线地图），产物 .investigations/。
- worker：知识库草稿产出（subagent）；I2/I4 数据解读发散再按 fan-out 规则。
- fan-out：I2/I4 分叉点预置（见 §7）。
- judge：I2/I4/I5 数据落盘后 MUST；交付闭环 MUST。
- knowledge：T7 subagent 产出（prompt 带「先读 SUBAGENT-KNOWLEDGE-GUIDE.md」）。
