# Phase 0 · 架构设计（多线程加速优化，DDR5 修正后）

> 状态：draft（待用户拍板）。整合基线/热点/量化三阶段发现 + 用户 DDR5 纠正。

## 一、现状总结

| 项 | 值 | 来源 |
|---|---|---|
| 单线程 density wall | 50-62ms/chunk | WG_STAGETIMER（本 session 复测） |
| 多线程 8t density wall | 400-500ms/chunk（膨胀 7-10×） | WG_STAGETIMER |
| 多线程 22t | 1440ms/chunk（23×） | WG_STAGETIMER |
| 吞吐 | 1t=92、8t=71、22t=80ms/chunk（无加速） | bench_chunks |
| 内存 | **DDR5-5600 双通道**（~85GB/s 有效） | 本 session 核实 |
| CPU | Ryzen 9 7845HX **12 物理核**/24 逻辑核 | 本 session 核实 |
| 对齐基线 | 8576/3200 SURFACE 99.9994%/99.9997% | 零退化铁律 |

## 二、关键发现（修正后）

1. **旧「带宽饱和（DDR4 上限）」定论失效**：17.8GB/s 是推演值（单线程 2.2GB/s × 8），远低于 DDR5 有效带宽。真实瓶颈是 **latency-bound（cache miss 延迟）**，证据：8t 下 spline 单次 10×（指针链 cache miss 高）vs noise 1.3×（局部参数表）——不对称膨胀，符合延迟而非带宽特征。
2. **density 耗时构成（估算，余量反推，有不确定度）**：
   - 块级三线性插值 **~79-87%**（98304 次/chunk 读 grid + 插值 + 虚函数调用）
   - spline 树遍历 ~9-18%（非 leaf ~109k 次 × 1.7µs）
   - base_3d_noise ~3%（70k 次 × 866ns）
3. **原计划两个无损方向只碰树遍历（~13-21%），碰不到块级插值大头**。块级插值 439ns/块偏高（8 读 + 14 浮点 + 虚调用，若全 cache hit 应 ~50-100ns），说明**含 cache miss 或虚函数开销，可能有无损优化空间**——但未直接测量。

## 三、方案设计

### 方案 A（无损，原计划）：spline 扁平化 + 边界角点复用
- **spline 扁平化**（优先，latency-bound 下收益大）：SplineDF 递归 `apply` 的 `locationFunction/subSplines` 虚指针链 → 扁平数组，减少树遍历 cache miss → 有望回落 8t 的 spline 10× latency 膨胀
- **边界角点复用**：InterpolatedDF grid x/z 边界角点与相邻 chunk 重合 36%，跨 chunk 复用减少 buildGrid 树遍历
- 收益上限：单线程 ~13%；多线程若 spline 扁平化消除 latency 膨胀，收益可能更大（待实测）
- 风险：**零退化可保**（纯缓存/布局，采样值逐位不变）

### 方案 B（无损，新增）：块级插值去虚 + grid 访问优化
- 针对 79-87% 大头：InterpolatedDF::sample 虚调用去虚化 / grid 连续内存布局 / 预取
- 收益：若块级插值 439ns 大头是 cache miss/虚调用，优化空间大（可能 > 方案 A）
- 风险：零退化可保；但方向需先测量块级插值构成确认

### 方案 C（有损，重新评估）：CELL 增大（RQ-006 8/16/8）
- 直接减少 grid 角点（1225→225，-82% 采样），单线程 2.2×（已测）
- 代价：对齐 97.28%（用户此前拍板不做）

## 四、推荐路径（先测量再定方案，避免误投入）

1. **【前置】轻量测量块级插值构成**：临时给 InterpolatedDF::sample 加计数器/计时，确认真实占比（消除 79-87% 余量反推的不确定性），并判断 439ns 中 cache miss vs 浮点 vs 虚调用的比例。
2. 依测量结果：块级插值确为大头 → 方案 B（无损去虚/cache）优先；树遍历占比高 → 方案 A 优先。

## 五、验收判据
- **BK-001 零退化**：8576 SURFACE 99.9994% / 3200 99.9997% 逐位不变
- 单线程 density wall 下降 + 多线程 8t wall < 单线程 wall（真加速）
- `scan_cpp_anchors.py` invalid=0

## 六、决策点（待用户拍板）
1. 是否接受「先做前置测量（块级插值构成）」再定方案，还是直接按原计划 A 实施？
2. 若测量确认块级插值是大头，是否投入方案 B（块级插值无损优化）？
3. 有损 CELL 增大（方案 C）是否维持「不做」？
