# DFC CPU 移植错误过程完整台账（五段式，逐错误步骤）

> 目的：**详细记录整个错误过程**（不只是结论）。覆盖 DFC 从立项误判 → 证伪 → 实现 → 逐轮优化失效 → 最终失败定论的**每条错误链**。
> 关联：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（缺陷四：DFC 失败定论，「四」节）+ `gpu-accel-errors.md` D26（闭包化提速 5%）+ `NEXT_SESSION.md` §8。
> 状态：本台账是「错误过程」详细记录（五段式逐错误），构成 DFC 课题的完整错误链资产。

---

## 错误链总览（DFC 绕圈的时间线）

```
[01] 立项误判（08-16）：虚调用/嵌套 SplineDF 是 11× 元凶（基于 6 SplineDF + 195 locFn 的 typeid 遍历）
  ↓（方向从根错）
[02] 立论证伪（今天 08-23）：权威 JSON 证 coordinate 全纯噪声 → 无嵌套 SplineDF → 虚调用只 5% → DFC 天花板 ~5%
  ↓（但已投入实现）
[03] MVP 预研：MVP 显式栈 maxDiff=0（正确性），但复现不了 11×（表小无指针追逐）——MVP 只能验证算法
  ↓（警示被忽视，继续完整实现）
[04] 完整 DFC 实现：gen_cpu 生成 C++ 采样函数 → 逐位对齐（2.06e-08 / 9.57e-07）✅ 正确性达成
  ↓（但每点慢）
[05] grid 缓存加 + corner=0 修正 → 正确性保持，并发放大 1.3×（看似核心价值），但每点 882μs
  ↓（绝对成本问题暴露）
[06] splitTop 优化（3.5×→251μs）：正确分配但每点仍慢
  ↓（收益递减）
[07] 闭包优化 D26（5%→238μs）：预估 2-4×，实际 5%——「移除叶子应大幅提速」预估失真
  ↓（主因不是孤儿 delegate）
[08] 接入 fillOneChunkCore：WG_DFC_CPU=1 整 chunk 超时（每点 238μs × 98304）→ 不可用
  ↓（关键洞察）
[09] 失败定论：DFC 是 GPU 设计搬 CPU（每点 split/grid 摊销是 GPU 妥协）→ 600× 慢 = 净作用为负 → 作废
```

---

## 错误 [01]：立项误判——虚调用/嵌套 SplineDF 是 11× 元凶（08-16 立论，方向从根错）

### 现象
- 立项时认为：density 11× 的根源 = SplineDF 每节点虚调用 `locationFunctions[nd.locFn]->sample(pos)` + locationFunction 是「InterpolatedDF/SplineDF/NoiseDF 深层嵌套」→ 递归进整棵 density 树（指数级膨胀）。
- 据此立项「DFC C++ 移植 = 消除虚调用 + 消除嵌套密度树递归」。

### 根因（为什么从根上错）
- **误读了 typeid 遍历结果**（2026-08-16 的 WG_SPLINESTATS/WG_SPLINESTATS 补全遍历：6 SplineDF + 195 locFn）——只证明了「有 SplineDF + 有 locFn」，**没证明「locFn 嵌套 SplineDF」**。
- 权威 JSON（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`）今天证实：**所有 spline coordinate 均为纯噪声 DF**（continents/erosion/ridges = `flat_cache(shifted_noise(...))`，ridges_folded = 纯 mul/add/abs 链），**无一嵌套 SplineDF**。
- spline 嵌套（points[].value 层层嵌，最多 3 层）**仅存在于数据表结构**，不是 coordinate 递归 → **「递归膨胀/指数级膨胀」假设不成立**。
- DFC 消除虚调用 + shared_ptr **只 ~5%**（MVP + 闭包化实测口径）——**主导成本是 shift_noise 噪声计算**，不是分派/虚调用。

### 定位
- 08-16 用 typeid 遍历找 SplineDF/locFn（未展开 registry 引用 / 未看 JSON coordinate 结构）。
- 今天 08-23 读到权威 JSON（density_function/overworld/*.json）逐一核对每个 spline 的 coordinate——发现全是噪声 DF。

### 修复
- **无代码修复**（方向本身错）。证伪「虚调用是元凶」。

### 教训
- **「有 X 类型 + 有 X 数量」≠「X 是根因」**——typeid 遍历只证明存在，不证明机制。**落到权威数据源**（JSON coordinate 结构）逐一核对才看到「coordinate 全噪声、无嵌套 SplineDF」。
- 先钉死主导成本（shift_noise 噪声）再立项——DFC 立项基于错误的「虚调用」主导假设。

---

## 错误 [02]：DFC 收益天花板 ~5%（虚调用不是 11× 主导）

### 现象
- MVP 实测：DFC 显式栈 vs 虚调用递归，收益只 ~4-5%（显式栈 2.22ms vs 虚调用 2.35ms，单线程 N=200000）。

### 根因
- DFC 显式栈只消除两样：虚调用（dispatch）+ shared_ptr 引用计数。但**主导成本 = shift_noise 噪声计算**（每点都在算），DFC 只消除虚拟分派（小头）。
- **虚调用的绝对成本小**（MVP 多态虚调用下两者 1.0x-1.2x）——虚调用本就不是主要成本。

### 定位
- MVP 线程扫描（const-recursive 11.7ns vs explicit 13.4ns vs virtual 13.4ns @T=1，并发 0.2x 全扩展）。
- 权威 JSON 证 coordinate 全噪声（噪声计算主导）。

### 修复
- 无（方向本身错）。DFC 收益天花板 ~5%，不可能解决 11×。

### 教训
- **「消除 X 能解决 Y」需先确认 X 占 Y 的份额**——虚调用只 ~5%，消除它不解决 11×。先量化（benchmark 主导成本）再立项。

---

## 错误 [03]：MVP 复现不了 11×（MVP 只能验证算法，不能外推性能）

### 现象
- MVP 线程扫描（mvp_spline_eval，3 形态 constRec/explicit/virtual，T=1/2/4/8）：**全部 amp=0.2x（完美扩展）**，未复现任何并发退化。
- MVP 显式栈 vs 递归：maxDiff=0（算法正确性 ✅）。

### 根因
- MVP 表小（245 元素全驻留 L1/L2 无争用）+ locFn 轻量子类（无真实 shared_ptr 散布堆对象指针追逐）+ 无「8 线程同时读同一批共享对象」压力 → **复现不了 production 的 11×（共享内存延迟/指针追逐）**。
- MVP 的访问足迹/表大小/指针追逐与 production **差一个量级**。

### 定位
- mvp_spline_eval 线程扫描（每形态每线程数取 min，100000 点/线程）。
- 对比 production 单样本 15.8→190μs（12×）——MVP 完全看不到。

### 修复
- 无（MVP 是算法验证工具，性能结论不可外推）。
- **警示应被重视**（说明 MVP 验证不了 DFC 的性能假设），但被「继续完整实现」覆盖。

### 教训
- **MVP（微基准）只回答「算法对不对」（maxDiff=0），回答不了「production 性能/机制」**。性能结论必须 production 数据，不可由 MVP 外推。
- 「MVP 没复现」≠「11× 不真实」（production 11× 已由 WG_PHASETICK 确证）——微基准与生产负载特征不同。

---

## 错误 [04]-[07]：完整 DFC 实现后逐轮优化「正确但无用」（收益递减）

> 这一段是「做了对的事但收益趋近零」——**正确性达成 ≠ 性能达成**。

### [04] 完整 DFC 实现（正确性 ✅，但每点慢）
- **现象**：gen_cpu 生成 C++ 采样函数（eval_density/eval_df/spline_eval 显式栈/spline_coord/normal_noise/interp）→ 逐位对齐 production（maxdiff 9.57e-07）✅。
- **根因**：DFC 用 split-precompute（每点重算 splitTotal 8672 floats）+ grid 摊销 + eval_density 结构成本——**GPU 设计（无 fp64 妥协 + 并行摊销）搬 CPU**，CPU 每点付全额 → 每点 882μs。
- **教训**：正确性达成（对齐）≠ 性能达成。算法对但性能不可用 = 无用。

### [05] grid 缓存 + corner=0 修正（正确性保持）
- **现象**：DFC 每点重算 → 加 grid 缓存（每 interp 每 chunk 5×49×5 网格 + 三线性）。**corner=0 修正**（grid 节点是 cell 的 (0,0,0) 角点 → corner 恒 0）→ 正确性保持。
- **根因**：直接采样被排除（会破坏 float32 对齐）→ 保留 split+perm 语义 → grid 缓存只是组织重构，不消除每点 split 成本。
- **教训**：grid 缓存消除「每点 8 角点重算」，但没消除每点 split()（分 t 仍每点重算整树）——**优化只针对结构，未针对主导成本（split）**。

### [06] splitTop 优化（3.5×→251μs，正确但每点仍慢）
- **现象**：sample() 每点整树 split()（200 条）→ splitTop（只 interp delegate 的 @c0，25 条 = 200 的 1/8）→ 882→251μs（3.5×）。
- **根因**：splitTop 只覆盖 interp 的 @c0（正确最小化），但**残余成本**（grid 构建摊销 + eval_density 结构）仍在。
- **教训**：splitTop 是「正确分配」的优化（对齐保持 + 3.5×），但只消除了 split 的一部分，未动 grid 摊销/eval_结构。

### [07] 闭包优化 D26（5%→238μs，预估失真）
- **现象**：eval_df_base/eval_df 从遍历全 DF_NODES(163) → 各用闭包子集（interp 1-4 只 ~17-21 节点，顶层 ~21）→ **251→238μs（仅 5%）**，预估 2-4×。
- **根因**：闭包化砍 87% 节点遍历只换 5% 提速 → **每点慢主因不是节点分派/孤儿 delegate**。「移除叶子计算应大幅提速」预估失真。
- **定位**：dfc_cpp_conc per-sample 对照（闭包化前后同工具同 seed）——delta <5% vs 预估 >2× 数量级差 → 反推主因 ≠ 孤儿 delegate。
- **教训**：「结构上砍掉大量计算 → 应大幅提速」是常见误判——**必须 benchmark 复核**。性能优化立项前先 benchmark 钉住主导成本。

### [07 汇总] 优化链收益对比（关键判错经验）
| 优化 | 改动 | 收益 | 对齐 | 结论 |
|---|---|---|---|---|
| splitTop | 每点整树 split → splitTop（1/8） | **3.5×**（882→251μs）| 保持 | ✅ 正确分配 |
| 闭包化 | 遍历 163 → 闭包子集 | **5%**（251→238μs）| 保持 | ⚠️ 正确但不显著 |
- **同样只改生成器、同样对齐保持，收益差一个数量级**——「每点慢」是结构性成本，不是单一优化点可解。

---

## 错误 [08]-[09]：接入失败 + 失败定论

### [08] 接入 fillOneChunkCore（整 chunk 超时）
- **现象**：WG_DFC_CPU=1 用 dfcBackend->sample（每点 238μs）→ 整 chunk 生成超时（98304 点 × 238μs ≈ 23.4s vs production 39ms，600× 慢）→ dfc_fill_compare 120s 超时、dfc_fill_compare 也不可用。
- **根因**：DFC 每点 238μs（split/grid_摊销/eval_结构）使整 chunk 不可行——**即使并发放大 1.3× 很好，但绝对成本 600× 慢**。
- **教训**：消除并发放大 ≠ 绝对快。DFC 用 600× 慢换掉 11× 并发——净作用为负。

### [09] 失败定论（D F C 是 GPU 设计搬 CPU，净作用为负）
- **现象**：DFC 每点 600× 慢 + 立论证伪（虚调用非元凶）+ 净作用为负 → **作废**。
- **根因**：三结构（split-precompute/grid 摊销/eval_density 结构成本）是 GPU「无 fp64 + 并行摊销 prefetch」的妥协；CPU 串行每点付全额。DFC 在 CPU 结构性不可行。
- **定位**：dfc_cpp_conc（并发放大 + 每点绝对耗时 T=1/2/4/8）+ dfc_fill_compare（120s 超时）+ D26（闭包化 5%）。
- **修复**：**DFC 性能方案 ❌ 作废**（实现保留作对齐参照，WG_DFC_CPU 默认关）。
- **教训**：
  1. **不要用「算法重写」解决并发争用**——MC density 树已是「一个对象 + 实例数据」，DFC 重写成 C2ME 数据驱动只是造了个 600× 慢的「更正确」版。并发 11× 战场在 production 自身争用点。
  2. **先 benchmark 钉住主导成本再立项**——DFC 立项前应先钉死「主导成本是可消除的分派/引用计数，还是不可消除的 shift_noise 噪声」——后者（噪声）主导，DFC 天花板 5%。
  3. **正确性达成 ≠ 性能达成**——逐位对齐达成但性能未达。

---

## 错误 [10]：关键洞察——DFC 是 GPU 设计搬 CPU（用错工具，最高层根因）

### 现象
- DFC（CpuBackend，GPU shader 移植）在 CPU 上每点 600× 慢，结构性不可行。

### 根因（最高层）
- **DFC 的设计初衷是为 GPU**：`split-precompute`（CPU 预拆分 float32 格点 + 小数，保精度）是 **GPU 无 fp64 的妥协**；grid 摊漪（每点 prefetch 并行工作组）是 **GPU 并行摊漪**。这些在 GPU 合理化（并行 amortize），**搬 CPU 串行每点付全额**。
- **CPU 的正确形态 = production 的 InterpolatedDF**：thread_local grid 缓存 + 共享实例直接采样（无 split 中间层）→ 单点 0.4μs 快。DFC 把 GPU 路径搬回 CPU，是**工具错位**。

### 定位
- 对比 production（0.4μs/点，无 split 直接采样）vs DFC（238μs/点，split+grid_摊销）——差在 split 中间层 + grid 摊销（GPU 妥协）。

### 修复
- DFC 作废。CPU 用 production 的 thread_local grid + 直接采样（单点快）。

### 教训
- **设计目标决定适用场景**：DFC（GPU shader 直排）用错场景 = 从 GPU 搬回 CPU。**先确认「优化目标是否与工具设计匹配」**——DFC 是为 GPU 精度的 split 预拆分，搬 CPU 反而增每点成本。

---

## 关键被自我纠正的点（错误过程的重要一环）

1. **corner=0 修正**（grid 缓存）：最初以为用 floorDiv corner 映射，实际 grid 节点是 cell 的 (0,0,0) 角点 → corner 恒 0。若照 floorDiv 映射（gx=1 读相邻角点 split）会破坏对齐。——**被 worker 发现并修正**（正确性关键）。
2. **直接采样被排除**：想在 CPU 直接调噪声对象（如 production），但会破坏 DFC 的 float32 对齐（已验证）→ 保留 split+perm。**被主会话排除**（避免破坏对齐）。
3. **闭包化预估失真**：预期 2-4×，实际 5%——**主因不是孤儿 delegate**（D26 教训）。

---

## 总结：DFC 错误链的最贵资产

1. **证伪「虚调用是元凶」**——避免未来继续在「消除虚调用」投入（最大价值）。
2. **确认生产单点 0.4μs 快（并发才是问题）**——把问题从「单点慢」重新定位到「并发争用」。
3. **先 benchmark 钉住主导成本再立项**——DFC 立项基于错误主导假设（虚调用），失败；后续用最小 A/B 先验证主导（见 rootcause「五」）。
4. **正确性 ≠ 性能**——逐位对齐达成但每点 600× 慢 = 无用。
5. **设计目标决定适用场景**——GPU 设计（split 预拆分）搬 CPU 反而增每点成本。
