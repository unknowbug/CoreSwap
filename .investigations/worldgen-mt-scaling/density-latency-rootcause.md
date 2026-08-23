# 每 chunk 并发下慢 7.5×——根因阶段定位：density 阶段内存延迟放大（2026-08-16）

> 状态：draft（主会话临时排查记录）| 承接 `per-chunk-concurrent-slow-mtrace.md`
> 上一阶段：MTTRACE 铁证 8 worker 真并行 + density 11× 慢（`per-chunk-concurrent-slow-mtrace.md`）

## 阶段级定位（WG_PROFILE，无 WG_MTTRACE 互斥）

### T=1 vs T=8 各阶段耗时（16 chunks 同 seed 8576）
| 阶段 | T=1 | T=8 | 放大 |
|---|---|---|---|
| density | 34-42ms | **400-450ms** | **~11×（主犯）** |
| aquifer+oreVein | 8-8.9ms | 20-58ms | ~4× |
| sh4+surface | 6.5-7ms | 22-36ms | ~4× |

→ **density 阶段 11× 远超其他 4×**（不是单纯带宽，否则各阶段同比例；density 有独特竞争）。

### 决定性计数（WG_PROFILE 全量计数，16 chunks）
| 指标 | T=1 | T=8 | 变化 |
|---|---|---|---|
| spline.sample 调用数 | 34,466 | 34,734 | +0.8%（**不变**） |
| interpGrid.fill 次数 | 762 | 762 | **不变** |
| aquiferDeep 次数 | 12,596,808 | 12,596,808 | **完全不变** |
| biomeAt 次数 | 27,936 | 27,936 | **完全不变** |
| **spline 单次耗时** | 28,141ns | 40,787ns | **+45%（核心元凶）** |
| noise 单次耗时 | 1,114ns | 1,188ns | +6.6% |

→ **调用次数全不变**（spline/网格/aquifer/biome 全稳定）→ **唯独 spline 单次耗时 +45%**（28.1→40.8μs）、noise 单次 +9%。**并发慢核心 = spline 单次内存延迟放大**。spline 单次 T=1 已 28μs（偏高，正常 <1μs），占 density 34ms 主体；并发下 28→40μs = 共享只读表 L3 miss 延迟放大——**每 chunk 并发下慢的直接元凶**。noise 单次 +9% 是伴随（non 主导）。

## 共享只读表（所有线程并发访问）
- **SplineDF**：`nodes` / `locations` / `derivatives` / `subIdx`（density.h L822-826）+ `locationFunctions` 池——每点随机访问
- **InterpolatedDF / FlatCacheDF**：per-instance `thread_local std::vector<Slot>`（每线程独立 grid，跨线程不共享——不是伪共享源）

## 根因假设（候选，C6 方向实锤）
**共享只读表（spline 表 + nodes 数组）跨核同时随机访问 → L3 miss 延迟放大**（非带宽饱和——C7 已排除 540MB/s = DDR 1-2%；是**延迟**：每点随机 miss 的 latency 因并发核争用 L3/memory controller 而翻倍）。

- 排除：C2 睿频（GHz 恒 2.99）、C7 带宽（1-2%）、C1 堆分配（C1 已排除，T=8 依旧）
- ⚠️ **C3「LLC 容量」结论需推翻（2026-08-16 复测修正）**：scout-map L93「8T 活动集 10.4MB < 16.5MB LLC → C3 排除」**估算有误**——漏算每线程 thread_local slot 完整 grid 驻留 + spline 表全量，且**未考虑 CCX L3 分片**（Zen4 12C = 2×16MB 分片，非单一 32MB）。实测单调性（见下）证明 **L3 在 T=2 已压力增大**，C3 大概率未真正排除。
- **剩余**：C6 共享只读表 L3 随机访问延迟放大（与 spline 单次 +30%、density 11× 吻合）+ L3 工作集超 CCX 分片失效。
- **先验修正**：nodes 数组 20B/节点（5×int/float，非 28B）——但 locations/derivatives float 连续数组访问仍可能缓存行争用。

### ✋ 2026-08-16 重要修正（typeid 铁证）——「spline 表是元凶」**排除**
- **density 阶段的 `finalDensity->sample(fpos)`（worldgen_api.cpp L790）遍历的树不含 SplineDF**：
  - typeid 递归遍历 finalDensity 树（WG_DENSITYSTATS 诊断）：53 个节点全是
    `BinaryOperation / UnaryOperation / LinearOperation / InterpolatedDF / BlendDensityDF / RangeChoice / YClampedGradient / NoiseDF / Constant`，**无 `wg::SplineDF`**。
  - JSON 铁证：overworld.json 的 `noise_router.final_density` = `minecraft:min`，argument1 = `minecraft:squeeze`，**全文件 `minecraft:spline` 出现 0 次**。
- **推论**：WG_PROFILE 全局 `spline.sample=34,566` **不在 density 阶段**（finalDensity 无 spline）——spline 计数来自 router **其他分量**（continents/erosion/depth 等，在 aquifer/surface/其他阶段触发）。**「spline 单次 +45%」是全局平均（含 aquifer），不是 density 元凶。**
- **density 阶段真实结构**：`min(squeeze(InterpolatedDF 网格角点差值), ...)` —— **squeeze 非线性 + InterpolatedDF 网格访问**。并发慢 11× 候选：
  - **InterpolatedDF::sample 网格访问**（thread_local grid + arg 链 → squeeze 触发每个角点）
  - **squeeze 非线性**（对 InterpolatedDF 输出变换）
  - **WG_PROFILE 探针污染仍可能**（density 阶段的 noise/spline 计时被探针原子竞争污染）
- **待定**：density 内部需**无探针**细分（InterpolatedDF 网格 vs squeeze）。MTTRACE dur 470ms 是无探针整 chunk（可靠），但未细分 density。
- **⚠️ 探针污染结论**：WG_PROFILE 的「spline 单次 +45% / noise 单次 +9%」在并发下被探针原子竞争污染（L854/L857 原子 fetch_add + steady_clock，T=8 下 8 线程竞争）——**这些单次耗时数字不可信**。真实依据 = MTTRACE dur（无探针）+ WG_STAGETIMER（阶段边界 once，污染小）：**density 11× 真实**，但内部哪个子操作慢未锁定。
- **推论**：spline 表（locations/derivatives/nodes/subIdx）**不是** density 阶段的每点随机访问元凶——splineDF 不在 finalDensity 树里（spline 可能只在 aquifer/oreVein 等非 density 阶段）。
- **元凶应在**：`InterpolatedDF`（网格插值 buildGrid + 角点三线性）+ `NoiseDF`（perlin 求值）的并发内存竞争。但：
  - NoiseDF 的 perm 表仅 256B（全驻留 L1/L2），noise 单次 T=1 1.11μs / T=8 1.19μs（+6.6%，基本不变）——**noise 非元凶**
  - interpGrid 重建次数不变（762）
- **待定**：并发下慢 11× 的真正来源需在**真并行 + 无探针**下逐阶段细分（InterpolatedDF buildGrid / 角点插值 / aquifer 共享表）。当前结论收敛为「density 阶段内存竞争」，**具体表未锁定**。

### ✋ 重大修正（2026-08-16，探针污染疑点）——「spline 单次 +45%」可能被 WG_PROFILE 探针竞争污染
- **WG_PROFILE 的 spline 单次/耗时测量在并发下被探针原子竞争污染**：L854 `wg_profSpline.fetch_add` + L857 `wg_profSplineNs.fetch_add`（原子 RMW）+ 每 spline 采样 `steady_clock::now()`——**T=8 下 8 线程同时原子递增 + 计时 → 原子总线/缓存行争用**，拖慢被测对象本身 + 计时失真。
- **反例证据**：typeid 遍历 finalDensity 树**无 SplineDF**，但 spline.sample=34,566 非零且单次 28μs（T=1）——**spline 大概率不在 density 阶段**（在 aquifer/surface 或独立 DF），density 阶段的 11× 与 spline 无直接关系。
- **真实（MTTRACE 无探针确认）**：density 阶段 11×（34→400ms）真实（MTTRACE dur 470ms ≈ WG_STAGETIMER density 400 + aquifer 58 + surface 30 = 488ms 吻合）。
- **density 11× 且 noise/interpGrid 调用数不变** → density 阶段每点慢但调用数不多 → **真凶 = InterpolatedDF::sample 网格访问的并发内存延迟**（grid thread_local，但每点 8 角点访问 + arg 链），**非 spline**（spline 在别阶段）。
- **待定**：density 阶段内部（InterpolatedDF 角点插值 / buildGrid arg 链 / BinaryOperation 虚调用）的并发慢，需无探针方式细分。

## 关键单调性数据（WG_STAGETIMER，干净阶段计时）
| T | density/chunk | 相对 T=1 |
|---|---|---|
| T=1 | 34ms | 基线 |
| T=2 | 71.1ms | **2.1×** |
| T=4 | 170.5ms | **5.0×** |
| T=8 | 382ms | **11.2×** |

→ **超线性增长**（T=2 已 2.1×，随 T 加速恶化）——典型共享资源竞争（L3 容量/内存控制器争用），非纯延迟翻倍。T=2 即 2.1× 说明 **2 线程就触发 L3 压力**（每线程工作集 > L3 分片/2）。

## 待定（下一步）
1. **定位并发慢的具体阶段**：真并行 + 无 WG_PROFILE 探针下，用 WG_STAGETIMER（阶段级）细分 density 内部——是 InterpolatedDF buildGrid 慢还是角点插值慢
2. 确认 InterpolatedDF 的 arg 链是否真的无 spline（若 aquifer 有 spline 表则另论）
3. 优化方向候选：
   - **InterpolatedDF 网格插值/角点缓存优化**（若此为主）
   - spline 表格化（仅在 aquifer 阶段若证实 spline 参与并发）
   - 降虚调用（DensityFunction 树遍历虚调用）

## 结论
> 「每 chunk 并发下慢 7.5×」根因 = **density 阶段并发内存竞争**（具体表未锁定；**非 spline 表**——typeid 证实 finalDensity 树无 SplineDF）。C3「LLC 容量」结论被推翻（scout-map 估算遗漏每线程 slot 驻留 + CCX 分片）。**notify bug 与内存竞争是两个独立问题**。
