# Phase 0 · 内存访问热点分析（density 阶段）

> 数据源：WG_PROFILE 计数器（block_probe 8576 SURFACE，36 chunks，单线程 vs 8 线程，2026-08-12 修复后 HEAD）
> 基线吞吐（bench_chunks 8×8，WG_STAGETIMER）：单线程 density=50-62ms/chunk；8t=460ms(7.5×)；22t=1440ms(23×)

## 一、单线程 WG_PROFILE 计数器（36 chunks）

| 计数器 | 总次数 | 每 chunk | 说明 |
|---|---|---|---|
| base_3d_noise.sample | 70,416 | 1,956 | 底层 Perlin 噪声（spline 叶子 / shifted_noise） |
| spline.sample | 212,622 | 5,906 | spline 树 Hermite 插值（含嵌套 leaf） |
| interpGrid.fill | 238 | 6.6 | InterpolatedDF::buildGrid（每实例每 chunk 1 次） |
| aquiferDeep | 3,546,851 | 98,523 | aquifer 深度采样（≈每块 1 次） |
| biomeAt | 1,489 | 41 | 生物群系判定 |

- noise 累计 61.0ms（866ns/次）；spline 累计 1,856ms（8,730ns/次，**嵌套累加污染口径，勿作真实单次**）
- 对齐 TOTAL = 99.9994%（零退化基线）

## 二、多线程（-threads 8）对比 —— 调用数不变、单次 latency 膨胀

| 计数器 | 单线程 | 8 线程 | 变化 |
|---|---|---|---|
| base_3d_noise.sample | 70,416 | 70,416 | **不变** |
| spline.sample | 212,622 | 213,701 | 几乎不变（+0.5%） |
| interpGrid.fill | 238 | 238 | 不变 |
| aquiferDeep | 3,546,851 | 3,546,851 | 不变 |
| noise 单次 | 866ns | 1,132ns | 1.3× |
| spline 单次 | 8,730ns | 87,025ns | **10×** |
| density wall | 50-62ms | 300-500ms | 7-10× |
| 对齐 TOTAL | 99.9994% | 99.9994% | 不变 |

**结论**：多线程下内存访问**次数完全不变**（确定性生成），但单次访问耗时膨胀 10×（spline）、1.3×（noise）——不对称膨胀。**修正（2026-08-12 用户纠正内存是 DDR5）**：这不是 bandwidth-bound（17.8GB/s 远低于 DDR5 双通道 ~85GB/s），而是 **latency-bound（随机指针链 cache miss 延迟）**：spline 深递归指针链 cache miss 高 → 10×；noise 参数表相对局部 → 仅 1.3×。不是锁/伪共享/分配。

## 三、density 阶段内存访问构成拆解

每 chunk 三部分：

1. **块级三线性插值**：98,304 次 InterpolatedDF::sample，每次读 grid 8 个角点（=786,432 double 读取/chunk）。grid 为 per-instance 5×49×5=1225 doubles（9.8KB），cache 友好但总量大。
2. **buildGrid 树遍历**：~6.6 buildGrid × 1225 角点 ≈ 8,085 角点采样/chunk → 触发 spline 5,936 + noise 1,956 次/chunk。每次 spline/noise 读分散内存（locations/derivatives/subSplines 指针链 + 噪声参数表），cache miss 高，是带宽敏感大头。
3. **aquiferDeep**：98,523 次/chunk（aquifer 阶段，独立于 density 树）。

## 四、两个无损优化方向的收益定位

- **边界角点跨 chunk 复用**：InterpolatedDF grid 的 x/z 边界角点（gx=4 列 = 右邻 chunk gx=0 列，5×49=245 角点；gz 同理）与相邻 chunk 重复计算。共享比例 ≈ (5+5-1)/(5×5) = **36%** → 可减 ~36% 的 buildGrid 角点采样 → 相应减少 spline/noise 调用（约 -36%）。**块级插值不受影响**。
- **spline 树扁平化**：spline 5,936 次/chunk，每次递归 apply 经 locationFunction/subSplines 虚指针间接跳转（读分散内存）。扁平化减少间接寻址，降低单次访问的 cache miss。

## 五、待量化（下一步）

- 每次访问的真实 cache 行为（块级 grid 读取是否命中 L2 / buildGrid 树遍历的 cache miss 率）
- 减少 36% buildGrid 采样后，多线程 wall 能降多少（收益上限）
- 判断：多线程能否真正加速（wall_mt < wall_t1），还是只能降单线程 wall
