# Phase 1 · 优化方案设计（测量后修正）

> 依据：phase0-interp-measurement.md（buildGrid 树遍历 86.5% 大头，纯插值仅 13.5%）

## 一、测量后的事实

- density 阶段（单线程 ~55ms/chunk）构成：buildGrid 树遍历 **86.5%**（1557ms/36chunk）+ 纯块级插值 13.5%（242ms）+ 其他
- 多线程 8t：buildGrid 膨胀 **8.8×**（latency-bound），纯插值仅 2.8×
- 两个无损方向（边界角点复用 / spline 扁平化）都直接减少 buildGrid 树遍历 → **方向正确**

## 二、方案优先级调整（关键决策）

原计划「Phase 1 边界角点复用 → Phase 2 spline 扁平化」。测量后发现：

| 方向 | 收益 | 复杂度 | 单/多线程 |
|---|---|---|---|
| **spline 扁平化** | 减树遍历 cache miss（spline 单次 8t 73311ns 的 latency 膨胀有望大幅回落） | 中（改 SplineDF 结构，无跨线程同步） | **单+多线程都受益** |
| 边界角点复用 | -36% buildGrid 采样（单线程 -28% density） | 高（跨 chunk 全局缓存 + 同步） | **主要单线程**（多线程并行生成时相邻 chunk 同时 buildGrid，复用率低） |

**建议：spline 扁平化优先（收益/复杂度比更高，且同时改善多线程 latency），边界角点复用其次（单线程收益大但多线程受限）。**

## 三、spline 扁平化设计（优先）

**现状**（density.h SplineDF）：`sampleImpl` 递归 `apply`：`locationFunction->sample`（虚调用）→ 二分查 locations → `subSplines[k]->sample`（虚调用，递归嵌套）→ Hermite 插值。每层 2 次虚指针间接跳转 + 二分。

**扁平化**：把递归 spline 树展开为线性数组/紧凑节点（locations/derivatives/subSpline 索引内联，去虚调用），减少指针追逐的 cache miss。对齐 Java Hermite 插值公式（`lerp(kd, nv, ov) + kd(1-kd)lerp(kd, p, q)`）逐位不变。

**风险**：纯结构重排，采样值逐位不变（BK-001 零退化）；需逐行对拍 Java Spline.apply。

## 四、边界角点复用设计（其次）

**机制**：InterpolatedDF grid 5×49×5 的 x/z 边界角点（gx=4 列 = 右邻 gx=0 列，共 441/1225=36%）与相邻 chunk 坐标重合。

**方案**：每实例全局 grid 缓存 `unordered_map<chunkKey, grid>` + mutex；buildGrid 时边界角点从邻居缓存读（未就绪则自算）。

**限制（诚实声明）**：多线程并行生成时相邻 chunk 同时 buildGrid → 复用率低；单线程顺序生成时复用率高（-36%）。同步开销可能部分抵消收益，需实测。

## 五、验收判据（同 Phase 0）
- BK-001 零退化：8576 SURFACE 99.9994% / 3200 99.9997%
- spline 单次（WG_PROFILE 口径）单/多线程下降；多线程 8t density wall 回落
- scan_cpp_anchors.py invalid=0
