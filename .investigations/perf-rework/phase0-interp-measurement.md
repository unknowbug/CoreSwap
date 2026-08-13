# Phase 0 · 前置测量结果：块级插值构成（临时计数器）

> 测量手段：InterpolatedDF::sample 临时加 interp.sample / interp / interpGrid 计数器（纯计时，采样值不变），WG_PROFILE 输出。测后需恢复。

## 一、单线程（36 chunks，8576 SURFACE，对齐 99.9994% 零退化）

| 计数器 | 次数 | 耗时 | 单次 |
|---|---|---|---|
| interp.sample（InterpolatedDF::sample） | 8,935,405 | 1,799.7ms | 201ns |
| interpGrid（buildGrid 树遍历） | 238 | **1,557.3ms** | 6.5ms/次 |
| 纯插值（= interp - interpGrid） | — | **242.5ms** | — |
| spline.sample | 212,622 | 1,723.8ms（污染口径） | — |
| base_3d_noise | 70,416 | 58.2ms | 826ns |

**构成**：buildGrid **86.5%**（1557.3ms）+ 纯插值 **13.5%**（242.5ms）。

## 二、多线程（-threads 8，对齐 99.9994% 零退化）

| 计数器 | 单线程 | 8 线程 | 膨胀 |
|---|---|---|---|
| interp.sample 次数 | 8,935,405 | 8,935,405 | **不变** |
| interp 总耗时 | 1,799.7ms | 14,394.5ms | 8.0× |
| interpGrid（buildGrid） | 1,557.3ms | 13,707.8ms | **8.8×** |
| 纯插值 | 242.5ms | 686.7ms | 2.8× |
| spline 单次 | 8,107ns | 73,311ns | 9.0× |
| noise 单次 | 826ns | 1,018ns | 1.2× |

## 三、关键结论（推翻 Phase 0 之前的错误估算）

1. **真正的耗时大头是 buildGrid 树遍历（单线程 86.5%、多线程 95.2%），不是「纯块级插值」**。之前 phase0-quantify.md「块级插值 ~79-87% 大头」是**余量反推错误**——把 buildGrid 首次触发的树遍历误归入「块级插值」。
2. **多线程膨胀集中在 buildGrid（8.8×），纯插值只 2.8×**——印证 latency-bound 定性：buildGrid 是随机指针链树遍历（cache miss 高），纯插值读 grid（相对局部）。
3. **原计划两个无损方向正中靶心**：
   - **边界角点复用**（-36% buildGrid 采样）→ 减 ~36% × 1557ms = 560ms（36 chunk）= **-15.6ms/chunk（-28% density）**
   - **spline 扁平化**（减 buildGrid 树遍历 cache miss）→ 缓解 8.8× latency 膨胀
4. 「先测量再定方案」价值兑现：避免了在错误方向（纯块级插值优化）投入。

## 四、收益上限重估（修正后）

- 单线程 density：55ms → 边界角点复用 -36% buildGrid ≈ **-28%**（55→40ms），spline 扁平化额外减 cache miss
- 多线程：buildGrid 8.8× 膨胀是 latency 瓶颈，两方向都直接减 buildGrid 树遍历 → 多线程 wall 有望显著回落
