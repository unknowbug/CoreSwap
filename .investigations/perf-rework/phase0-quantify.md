# Phase 0 · 无损优化收益上限量化

> 数据源：单线程 WG_PROFILE（block_probe 8576 SURFACE 36 chunks）+ NEXT_SESSION 的 WG_SPLINEDEBUG 精确 spline 单次（1714ns）

## 一、density 阶段耗时构成（单线程，36 chunks，wall=1980ms=55ms/chunk）

| 组成 | 次数（36 chunk） | 单次 | 耗时（36 chunk） | 占比 |
|---|---|---|---|---|
| 块级三线性插值 | 3,538,944 | ~439ns | ~1,555ms | **~79%** |
| spline 树遍历 | 212,622 | 1,714ns（精确） | ~364ms | ~18% |
| base_3d_noise | 70,416 | 866ns（WG_PROFILE） | ~61ms | ~3% |

- spline 单次 1,714ns 取自 NEXT_SESSION WG_SPLINEDEBUG 精确统计（WG_PROFILE 的 8,730ns 是嵌套累加污染口径，×5.1）
- noise 61ms 是可靠值（WG_PROFILE 单次 866ns × 70416，无嵌套污染）
- 块级插值 = 余量（1980 - 364 - 61），每块 8 次 grid 读取 + 三线性插值浮点运算，~439ns/块合理

## 二、两个无损方向的收益上限

### 边界角点跨 chunk 复用（减 buildGrid 重复采样）
- 共享比例 = (5+5-1)/(5×5) = **36%**（InterpolatedDF grid x/z 边界角点与相邻 chunk 重合）
- 减 36% spline 调用（212,622→136,078）+ 36% noise（70,416→45,066）
- 单线程收益 = 364×0.36 + 61×0.36 ≈ **153ms（36 chunk）= 4.25ms/chunk**
- **不碰块级插值**（块级读的是 grid 产物，非 buildGrid 计算）

### spline 树扁平化（减间接寻址）
- 假设减 30% spline 耗时（虚指针链 → 扁平数组）
- 单线程收益 ≈ 364×0.30 = **109ms（36 chunk）= 3ms/chunk**

### 合计上限
- 单线程：55ms → ~48ms（**-13%**），总 wall（含 aquifer/surface）改善更小
- 多线程：spline 单次 10× 膨胀下，边界角点复用收益放大（省 ~1.3s 树遍历），**但块级插值 79% 依然是带宽争用大头，无损碰不到**

## 三、核心判断修正（用户 2026-08-12 纠正：内存是 DDR5 而非 DDR4）

**实测环境**：DDR5-5600 双通道（2×24GB，SK Hynix，SMBIOS=34），理论峰值 ~89.6GB/s（有效 ~75-85GB/s）；CPU Ryzen 9 7845HX **12 物理核**/24 逻辑核。

**旧定论失效**：NEXT_SESSION「8 线程 ~17.8GB/s ≈ DDR4 带宽上限 → 带宽饱和」基于**错误的内存类型假设**。17.8GB/s 是推演值（单线程 2.2GB/s × 8 线性外推），远低于 DDR5 双通道有效带宽（~85GB/s）→ **并非 bandwidth-bound**。

**重新定性：latency-bound（cache miss 延迟），证据**：
- 8 线程下 spline 单次 8730→87025ns（**10×**），noise 单次 866→1132ns（**仅 1.3×**）——**不对称膨胀**。若带宽饱和，两者应同比例排队；实际只有 spline（深递归指针链，cache miss 高）10×，noise（噪声参数表相对局部，cache miss 低）1.3× → 符合「随机指针链 cache miss 延迟」而非「带宽对称争用」。
- 物理核 12 > 8 线程，非核心争用。

**优化前景改善**：latency-bound 下，**减少 cache miss 直接减少延迟等待**，收益可能大于「带宽硬上限」判断下的估算：
- spline 扁平化（虚指针链 → 扁平数组）可显著减少树遍历的 cache miss → 单次 87025ns 的 10× 膨胀有望大幅回落
- 边界角点复用（-36% 树遍历）直接减少 cache miss 次数
- **多线程加速仍有可能**（latency-bound 可通过提升 cache 局部性缓解，不同于带宽硬上限）

## 四、方向（重新定性后，待用户拍板）

| 方向 | 收益（修正后） | 风险/代价 |
|---|---|---|
| A. 无损优化（spline 扁平化 + 边界角点复用） | 单线程 ~13%+；**多线程可能显著改善**（消除 spline 10× latency 膨胀，收益待实测） | 实现量中等；**零退化可保** |
| B. 接受现状收尾 | — | 明确边界 |
| C. 有损 CELL 增大（RQ-006） | 单线程 2.2×（已测） | 对齐 97.28%（用户此前拍板不做） |

## 五、不确定点（诚实声明）
1. spline 真实单次 1,714ns 依赖 NEXT_SESSION 数据，本次未重测（WG_PROFILE 污染口径不可直接引用）
2. 块级插值 79% 是「余量反推」，未经独立计数器直接测量（当前无块级插值专用计数器）
3. 多线程无加速「latency-bound vs L3 cache 容量争用」的最终区分未实测（需性能计数器 / VS 分析器采集 cache miss）
4. spline 扁平化能消除多少 cache miss、多线程能恢复多少加速，均需实现后实测
