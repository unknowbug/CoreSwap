# 每 chunk 并发下慢——根因定位：density 阶段 11× 真实（2026-08-16 WG_PHASETICK 确认）

> 状态：**确认 density 11× 真实**（WG_PHASETICK 干净测量）| 承接 per-chunk-concurrent-slow-mtrace.md
> 修正链：WG_PROFILE → WG_DENSITYTICK(bug) → WG_PHASETICK（最终可靠）。

## ✅ 最终可靠结论（WG_PHASETICK，QPC 单次 + 无 profiling 污染 + 单循环）

| 阶段 | T=1 | T=8 | 放大 |
|---|---|---|---|
| **density** | 34-42ms | **400-412ms** | **11×（主犯）** |
| aquifer+ore | 8ms | 25-28ms | ~3× |
| surface | 7ms | 25-38ms | ~4× |
| total | 50ms | **462ms** | **9×** |

- **自洽验证**：462ms × 8 并行（64 chunks = 8 批）≈ 3696 + 批间 = 4618ms = wall ✅
- **每 chunk 真实 462ms（T=8）vs 50ms（T=1）= 并发下慢 9× 真实**
- **density 11× 是主犯**（aquifer/surface 仅 ~3-4×）

## 概念澄清（关键，之前混淆）
- **bench `med/N`（wall/N）= 吞吐均值（72ms/chunk）**——不是每 chunk 耗时
- **每 chunk 真实耗时 = 462ms**（8 worker 并行，wall 4618ms 处理 64 chunks = 8 批）
- **wall/N 是平均吞吐，每 chunk 耗时是延迟**——多线程下吞吐均值（72ms）掩盖单 chunk 延迟（462ms），差 6.4×（并行度）
- 之前「wall+8% → 并发正常」是**把吞吐均值误当每 chunk 耗时**的错

## 修正链（为什么前两次错了）
| 测量 | 结果 | 判定 |
|---|---|---|
| WG_PROFILE density | 34→400ms（11×） | ✅ **真实**（WG_PHASETICK 印证）|
| WG_STAGETIMER density | 34→400ms | ✅ 真实 |
| WG_DENSITYTICK density | 6.95ms 不变 | ❌ **bug**（重复循环，6.95ms 假象）|
| WG_MTTRACE dur | 470ms | ⚠️ fprintf 锁竞争（但 462ms 量级对，考虑锁竞争）|
| **WG_PHASETICK** | 34→409ms | ✅ **最终可靠** |

- **WG_DENSITYTICK 的 bug（重复循环）**误导我得出「并发正常」——初稿 MT8 是错的，已修正。
- **概念混淆**：wall/64=72（吞吐）被误当每 chunk 耗时 → 误判「只慢 8%」。

## density 内部待定位（下一步）
density 11× 真实（squeeze(InterpolatedDF) 阶段）。候选：
- squeeze 非线性对 InterpolatedDF 网格输出的变换
- InterpolatedDF::sample 每点访问 thread_local grid（8 角点）+ arg 链
- 全局共享（WG_PROFILE 计入 spline 34K 次——需澄清 spline 在 density 阶段的触发）

## 🔥 最新定位（2026-08-16，WG_SPLINESTATS 补全遍历）——spline 真实存在！
- **finalDensity 树含 6 个 SplineDF**（splineInst=6）、**537 节点、17KB 表**（splineBytes=17112）、**195 locationFunction**
- **之前误判「无 spline」是错的**——最初 typeid 遍历漏了 BlendDensityDF/WrappingDF（spline 经 blend_density 引用 continents/erosion/depth 分量）；WG_SPLINESTATS 补全遍历后确认 6 实例。
- **关键**：
  - spline 表 **17KB（很小，驻留 L2）**——**不是 L3 miss 容量问题**（远小于 16MB L3）
  - spline 单次 **34μs（T=1）→ 52μs（T=8）= +51%**——spline 树每点遍历 90 节点（537/6）+ 递归 sampleNode + 195 locationFunction 虚调用
  - **density 11× 核心 = spline 树递归 + 虚调用 + 多实例的每点成本**（不是 L3 miss——表太小）
- **修正**：之前「L3 miss 放大」假设**不成立**（17KB 表驻留）。真正是 **spline 单点计算开销（递归 90 节点 + 虚调用）+ 并发下 cache-line/1-cache 争用**。

## 后续（spline 为 density 11× 主因）
定位 spline 单次 34μs 的构成：递归深度（90 节点/实例）× 每节点操作。优化方向：
- **SplineDF 节点紧凑化/去虚调用**（locationFunctions 195 个虚调用是主要开销）
- spline 表格化（C2ME DFC 编译直排）——消除每点树递归遍历

## 🔥🔥 决定性反推（2026-08-16 最新）——spline 单次并发下慢 12×
用 **WG_PHASETICK（干净 density，无 profled 采样计时）** + **spline 计数（可靠）** 反推 spline 真实单次成本：
- **T=1**：density 34ms / spline 2154 次 ≈ **15.8μs/spline**（真实，密度是 WG_PHASETICK 干净的）
- **T=8**：density 409ms / spline 2160 次 ≈ **190μs/spline**
- **spline 单次并发下慢 12×**（15.8→190μs）——**这是 density 11× 的直接来源**

### 关键澄清
- **spline 表 17KB（驻留 L2）**——不是 L3 miss 容量。**慢在 spline 树递归（90 节点/实例）+ 195 locationFunction 虚调用 + 并发下 I-cache/cache-line 争用**
- **spline 单次 15.8μs（T=1 真实）已经很高**——正常 MC shape spline 单次 <1μs。**「每块树遍历 + 虚调用」是固有膨胀**（C2ME 用 DFC 编译直排消除）
- 并发下 15.8→190μs（12×）= **虚调用/递归的并发争用**（8 线程同时遍历同一棵 spline 树，I-cache 被稀释 + 递归栈 cache-line 共享）

### 结论
density 11×（=每 chunk 并发下慢 9×）**根源 = SplineDF 树遍历（递归 + 虚调用）在并发下的 I-cache/争用放大**。表小（17KB）但递归深（90 节点）+ 195 虚调用。**C2ME 式 DFC 编译直排**（消除树遍历虚调用）是正确优化方向。


## 后续（WG_PHASETICK 为可靠工具）
用 WG_PHASETICK 进一步拆分 density 内部（它可靠），定位 11× 的准确来源（squeeze vs InterpolatedDF grid 访问 vs 共享表）。
