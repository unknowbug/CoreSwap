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

## 🔥🔥🔥 DFC C++ 移植方案（2026-08-16 立项，多轮大工程，MVP 先行）

### 现状
- **production SplineDF**（density.h）：虚调用递归树遍历（每节点 `locationFunctions[nd.locFn]->sample`，虚调用 + 可能嵌套另一棵密度树）——慢（15.8μs/op，并发 12×）。
- **DFC 基础设施**（dfc_gen.py）：
  - 已生成 **GLSL 版 spline_eval**（显式栈后序 + 数据驱 splineNodePack 表，L1309-1368）——无递归虚调用。
  - 已生成 **CpuBackend 数据表**（splineNodePack/Locs/Ders/ValF/ValKind/ValNode，gen_cpu L1714-1722）。
  - **缺**：C++ 版 spline_eval/spline_coord/eval_df 采样函数（这些只在 GLSL shader，CpuBackend 只有数据表）。

### 关键理解（移植复杂度）
- **spline_coord = 完整 DF 子树**（dfc_gen.py L1125-1132：`self.gen_node(coord)`/`self.gen(coord)` 生成完整 DF 表达式）——spline 的 coordinate 是 continents/erosion 等分量（本身含 interpolated/spline/noise 嵌套）。
- **所以 DFC C++ 移植 = 写「整个 finalDensity 树的 C++ 采样生成器」**（不只是 spline_eval——还有 spline_coord + eval_df + 嵌套 DF）。

### 执行方案（MVP 先行）
1. **MVP**：扩展 dfc_gen.py 的 `gen_cpu`，额外生成 **C++ 版 spline_eval + spline_coord**（聚焦 spline）——用现成 splineNodePack 数据 + gen_node 坐标链。对拍（DFC C++ vs SplineDF 现有输出）验证正确性。
2. **扩展**：生成整个 finalDensity 树的 C++ eval_df（复用 gen_shader 的 eval_df_glsl 逻辑 → C++）。
3. **接入**：production fillOneChunkCore 用 DFC C++ 采样替代 SplineDF 树遍历。
4. **验证**：8576/3200 零退化 + 吞吐对比（期望消除 11×）。

### 风险/待确认
- DFC C++ 直排是否真消除并发争用（CPU 直排代码仍有嵌套调用，但消除了动态虚调用/共享表指针跳转）——**需实测确认**。
- `node_mode`（D1：节点函数化，dfc_gen.py L1182-1184）vs `slot_mode`（方案1：corner slot 化）——选择哪种生成模式影响 C++ 结构。

### 记录
- 本方案已记 NEXT_SESSION 待办 6 + 本文档。MVP 实施记录随进度追加。

## 🎯 2026-08-16 技术细节确认（DFC C++ 移植真实规模）- **gen_shader（dfc_gen.py L1404+）已生成完整 DFC**：`gen_df` → `eval_df` 解释器（整个 finalDensity 树）+ `registry` 函数 + `spline_eval`（显式栈 L1309-1368）+ `spline_coord`（coordType 分派，`coord_glsl`）+ `normal_noise`/`interp`/`old_blended`（数据驱动单函数）。**GLSL 版完整存在**。
- **gen_cpu（L1649+）只生成数据**：噪声初始化 + split 行 + spline SSBO 表（NodePack/Locs/Ders/ValF/ValKind/ValNode，L1714-1722）+ perm。**无 C++ 采样函数**。
- **spline_coord = 完整 DF 子树**（`_spline_coord_type` L1125-1132：`self.gen_node(coord)` 生成完整 DF 表达式）——spline 的 coordinate 是 continents/erosion 等分量（含 interpolated/spline/noise 嵌套）。

### DFC C++ 移植 = 写完整 C++ DFC 后端（非仅替换 SplineDF）
需 gen_cpu 扩展生成：
1. **eval_df**（整个 finalDensity 树解释器，C++ 版）
2. **spline_eval**（显式栈 + splineNodePack 表）
3. **spline_coord**（coordType 分派 + 完整 DF 子树）
4. **normal_noise / interp / old_blended**（数据驱动单函数）
5. **接入** fillOneChunkCore + 对齐验证 + 吞吐对比

**结论**：这是**大型多轮实现**（相当于把 dfc_gen.py 的 GLSL 后端扩展为 C++ 后端），单轮无法完成。MVP 先行（先 C++ spline_eval + spline_coord，对拍 SplineDF 正确性 + 性能），再扩展到 eval_df 全树。

## 🔥🔥🔥 DFC 优化方向确认（dfc_gen.py 已有扁平 spline）——收益来源 = 消除嵌套密度树递归，不只是虚调用
- **SplineDF（当前 production）** 每节点：`locationFunctions[nd.locFn]->sample(pos)`（**虚调用，且 locationFunction 可能是 InterpolatedDF/SplineDF/NoiseDF 深层嵌套**）→ 每节点可能递归进入**整棵 density 树**，指数级膨胀 → spline 单次 15.8μs 高。
- **DFC spline_eval（dfc_gen.py，GPU shader）**：`spline_coord(ct, corner, sIdx, ix, iy, iz)`（**数据驱动直接算该节点坐标**）+ `spline_find_range`（二分）+ `spline_hermite` + **显式栈后序求值**（L1309，无递归无虚调用）——用 CpuBackend 的 `splineNodePack/splineLocs/splineDers/splineValF/ValKind/ValNode`（扁平表）。
- **本质差异**：SplineDF 每节点「调 locationFunction（可能递归子树）」→ DFC「直接算该节点坐标（数据驱动）」。**DFC 消除的是「整棵嵌套密度树递归」，不只是单层虚调用**——这是 spline 单次高的根源。
- **DFC 基础设施已存在**（dfc_gen.py L216-226：DFC 能生成 vanilla 完整 final_density 树 shader + CpuBackend；spline 已收编✅）：spline 扁平表 + 显式栈 spline_eval **GLSL 版已生成**。
- **优化路径**：把 GLSL 的 spline_eval（显式栈 + 数据驱动）**移植为 C++ 函数**（用 CpuBackend 扁平表），替换 production SplineDF 的虚调用递归——**消除嵌套密度树递归**。
- **关键验证**：SplineDF 的 locationFunction 是否真是 InterpolatedDF/SplineDF 深层嵌套（若是，DFC 收益大——消除指数递归；若只是 NoiseDF 单层，收益小）。见 MT10（6 SplineDF + 195 locFn，confirm 嵌套存在）。


## 后续（WG_PHASETICK 为可靠工具）
用 WG_PHASETICK 进一步拆分 density 内部（它可靠），定位 11× 的准确来源（squeeze vs InterpolatedDF grid 访问 vs 共享表）。

## 🔬 MVP 第 1 步结果（2026-08-16，显式栈算法正确性验证，路径 B）

**文件**：`.investigations/perf-rework/vulkan-proto/mvp_spline_eval.cpp`（独立 C++，内联 CpuBackend spline 表 245×5 + const 数组）

### 结果
```
[MVP] explicit-stack vs recursive: maxDiff=0.000e+00 @n=0
[MVP] N=200000 recursive=1.87ms stack=2.36ms acc=263382.1137 acc2=263382.1137
```

- **maxDiff = 0** —— DFC 显式栈 spline_eval 与递归版**逐位一致**（算法正确性 ✅）
- acc == acc2（263382.11）数值一致
- 性能：recursive **1.87ms** vs stack **2.36ms**（单线程，显式栈**略慢 26%**）

### 诚实解读
1. **算法正确性 ✓（MVP 核心）**：DFC 显式栈（与 GLSL 同源）与递归（production SplineDF 形态）输出逐位一致——**DFC C++ 移植算法正确**。
2. **性能对比不反映真实**：MVP 用 **const 数组**（无虚调用/shared_ptr），递归版也被编译器优化（const 数组 → 无虚调用 → 递归 vs 显式栈都很快）。**这不反映 production SplineDF 的真实开销**（虚调用 + shared_ptr + 递归，才是 density 11× 来源）。
3. **单线程显式栈略慢（26%）**——但在「无虚调用的递归」对比下，不具代表性。

### 关键结论
- **算法正确性已验证**（DFC 显式栈 = 递归）。
- **但性能收益未证实**——需第 2 步：用**真实 production SplineDF**（虚调用 + shared_ptr）作递归参照，对拍 DFC 显式栈，才显示「消除虚调用」的真实收益。
- **风险**：若 production SplineDF 的真实开销主要是「虚调用 + shared_ptr」而非「递归结构」，则 DFC（消除虚调用）有收益；若主要是「每点指令量」，则 DFC 收益有限。**MVP 第 2 步验证此关键假设**。
- 注：MVP 的「显式栈略慢」是单线程 const 数组对比；真实场景需在「虚调用递归」下测（production SplineDF）。

## 🔥🔥🔥🔥 2026-08-23 干净实验确证：11× 真实 + DFC 收益天花板 ~5% + 机制收窄

> 本小节**修订/推翻**前面多条结论：① 原「locationFunction 嵌套 SplineDF」= ❌ **误读**（已加权威 JSON 证据）；
> ② 原「DFC 是正确的优化方向」= ⚠️ **待重审**（收益天花板 ~5%）；③ 「density 11× 真实」= ✅ **保持成立**。
> 数据来源 = 今日（2026-08-23）**干净对照实验**（conc_density_probe + WG_PHASETICK，无探测污染）。

### 一、现象（数据）

**density 阶段单 chunk 延迟随线程数线性暴涨**：

| 测量（conc_density_probe，同批 chunk、无 warmup） | T=1 | T=8 | 倍率 |
|---|---|---|---|
| 平均 density（整批） | 39.31ms | 331.04ms | **8.4×** |
| 单 chunk (-6,-6) | 42.69ms | 391.41ms | **9.2×** |

**density 延迟随线程数线性增长**（共享资源可扩展争用特征，T=1→2→4→8）：

| T | density 耗时 | 相对 T=1 |
|---|---|---|
| 1 | 37.83ms | 1× |
| 2 | 74.01ms | 2× |
| 4 | 174.33ms | 4.6× |
| 8 | 341.79ms | 9× |

**关键区分——吞吐正常 vs 单 chunk 延迟暴涨**（AGENTS.md 早已警告的「吞吐均值 vs 每 chunk 延迟混淆」）：
- **每 chunk 延迟** = density 阶段耗时 = 42.69→391.41ms（**9.2×**）——真实暴涨；
- **整批吞吐** = wall/chunk = 69ms（T=1）vs 73ms（T=8）——**几乎不变**（bench_chunks [A]）；
- 文档早期把 bench `med/N=72ms` 当「并发正常」是**错误判法**——那是吞吐均值，掩盖了单 chunk 延迟。

### 二、根因（当前推断，机制收窄）

**文档原「locationFunction 嵌套 SplineDF」判断 = ❌ 误读**。权威 JSON 数据源
（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`，factor/offset/jaggedness）确证：
- **所有 spline 的 coordinate 都是纯噪声 DF，无一嵌套 SplineDF**：
  - `continents` = `flat_cache(shifted_noise(continentalness))` ✅（continents.json）
  - `erosion` = `flat_cache(shifted_noise(erosion))` ✅（erosion.json）
  - `ridges` = `flat_cache(shifted_noise(ridge))` ✅（ridges.json）
  - `ridges_folded` = `mul(-3, add(-1/3, abs(add(-2/3, abs(overworld/ridges)))))` —— 纯 mul/add/abs 链，引用 `ridges`（噪声），**不是嵌套 SplineDF** ✅（ridges_folded.json）
- **spline 嵌套真实存在，但只是「数据表结构」**：`points[].value` 层层嵌 spline（factor.json 可见，最多 3 层：erosion→ridges_folded→ridges）——这是 spline **采样点值的数据表**，**不是 coordinate 递归**。

**推论（核心）**：DFC 显式栈只消除虚调用/+shared_ptr（MVP 实测 ~5%），**无法消除主导的 shift_noise 噪声计算**。
⇒ **DFC 理论上不可能消除 11×**（收益天花板 ~5%）。优化重心应转向**噪声计算本身**，而非 DF 树递归/虚调用。

**已排除项**（干净确认，非猜测）：
- 跨线程共享缓存竞争（InterpolatedDF/FlatCacheDF/Cache2DDF 缓存全部 thread_local）❌
- noise sampler 共享状态（全 const）❌
- `g_curChunkX`（thread_local）❌
- 虚调用（~5%，非主导）❌
- DFC（收益天花板 ~5%，非 11× 来源）❌

**未定（待 scout / 进一步实验）**：
- 8 worker 同时读同一棵 finalDensity 树 + 同一套 noise perm 表（全局共享 const 对象）→ **LLC/L3 带宽争用**，导致每点延迟线性增长；
- 或 **I-cache 代码争用**；
- 或 **SMT（超线程）**。

### 三、定位（怎么测的）

- **conc_density_probe**：并发 density 探针（黑盒），T=1/2/4/8 同批 chunk 对照，无 warmup，测单 chunk density 阶段耗时。
- **WG_PHASETICK**：可靠阶段计时（QPC 单次 + 无 profiling 污染 + 单循环，density.h getter 同源）——确认 density 阶段真实耗时，排除探针污染（AGENTS.md「测量/探针污染铁律」）。
- **权威 JSON**：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json` —— 确证 tree 结构（coordinate 纯噪声 vs `points[].value` 嵌套）。
- **判据**：阶段耗时 × 并行批次 ≈ wall 自洽 → 工具对；吞吐（wall/N）与每 chunk 延迟（阶段耗时）**分开衡量**，不能用吞吐均值替代延迟。

### 四、修复（暂无，待机制确认）

- **暂无代码修复**。前一版「DFC C++ 移植消除嵌套密度树递归」方向**已推翻**——DFC 收益天花板 ~5%，非 11× 良药。
- 待机制确认后定方案（均为**待验证假设，未定**）：
  - 若为 LLC/L3 带宽争用 → 噪声重算 / 每线程 perm 表副本 / 采样数据分片；
  - 若为 I-cache → 代码紧凑化；
  - 若为 SMT → 调整线程数/clamp 策略。

### 五、教训

1. **吞吐 vs 延迟必须分开**：并行性能看「每 chunk 延迟（阶段耗时）」，不是 wall/N 吞吐均值——吞吐正常 ≠ 并发无问题。AGENTS.md 已警告，本文档仍犯一次（把 med/N=72ms 当「并发正常」）。
2. **静态排除不够，要干净实验**：靠 typeid 遍历 / 静态读 JSON 推断「locationFunction 嵌套 SplineDF」是误读——必须用**权威 JSON 数据源**（tree 结构）+ **干净探针**（无探测污染对照）才能定论。
3. **优化方向要先钉住「主导成本在哪」**：DFC 方案立项前，先确认成本是「虚调用/递归」还是「shift_noise 噪声计算」——用权威 JSON 树结构提前证伪「坐标递归」假设，用 MVP 收益天花板（~5%）提前排除 DFC，避免在错误方向投入大工程。

### 结论修订标注

| 旧结论 | 状态 | 依据 |
|---|---|---|
| 「locationFunction 嵌套 SplineDF」（density 11× 主因） | ❌ **误读** | 权威 JSON 证实所有 spline coordinate 为纯噪声，嵌套仅存在于 `points[].value` 数据表 |
| 「DFC 是正确优化方向」（C2ME 式 DFC 编译直排） | ⚠️ **待重审** | DFC 显式栈只消除虚调用/+shared_ptr（~5%），无法消除主导 shift_noise |
| 「density 11× 真实」 | ✅ **保持成立** | 今日干净实验复现 8.4×/9.2× |

## 🔴 2026-08-23 二：机制收窄（scout 勘探）+ 决定性对照实验（MVP 无法复现 11×）

> 承接「2026-08-23 一：干净实验确证」（density 11× 真实 + DFC 收益天花板 ~5% + 机制收窄）。
> 本节**新增两块今日确证**：① **scout 静态勘探收窄机制**（Tier-1 = SplineDF 递归虚调用链的共享内存延迟放大，Tier-2 = I-cache/code-fetch 争用）；
> ② **决定性对照实验**（mvp_spline_eval 并发线程扫描）——**MVP 无法复现 11×**。据此**修正**上一节「DFC ~5% 收益天花板」的定性（在这条 MVP 路径上既不能证实也不能证伪），并**推翻「放大 MVP 验证 DFC」路径**。
> 侦察报告：`.investigations/worldgen-mt-scaling/concurrent-density-probe-scout.md`（只读，未编译未改码）。

### 一、现象（数据）

**production 的 density 11× 真实**（上一节已确证，conc_density_probe + WG_PHASETICK 干净测量，8.4×/9.2×）——本节承接。

**关键对照——MVP 决定性实验未复现**（`.investigations/perf-rework/vulkan-proto/mvp_spline_eval.cpp`，加线程扫描 + 每样本成本，T=1/2/4/8，100000 点/线程，5 轮取 min）：

```
per-sample ns:  constRec          explicit(DFC)     virtual
    T=1            11.7              13.4             13.4
    T=8             2.1               2.2              2.4
 concurrency amplification (T=8 vs T=1): constRec=0.2x explicit=0.2x virtual=0.2x
```

| 每样本 | T=1 | T=8 | 并发放大 |
|---|---|---|---|
| **MVP（virtual 形态）** | 13.4 ns | 2.4 ns | **0.2×（快 5×）** |
| **production spline** | 15.8 μs | 190 μs | **12×（慢 12×）** |

- **MVP：spline 采样随线程数完美扩展**（0.2× = 8 线程快 5×），**未复现任何并发退化**；
- **production：单样本 15.8μs(T=1) → 190μs(T=8)**（12×，density 11× 的直接来源）；
- 两者方向**相反**——这是今日唯一「未复现」证据，且是**决定性**（不是「没测到」，而是干净对照后确认 MVP 层面不存在该机制）。

### 二、根因（当前定论：指针追逐 + 长链共享内存延迟）

**scout 静态勘探收窄**（来源 concurrent-density-probe-scout.md §三）：

1. **Tier-1（主因）= SplineDF 树递归虚调用链的共享内存延迟放大**
   - `sampleNode`（density.h:876-925）是**长串行依赖链 + 指针追逐**：递归至 ~90 节点/实例，每级 `locationFunctions[nd.locFn]->sample(pos)`（虚调用 + shared_ptr 间接，**locFn 对象散布堆**）+ 随机读 nodes/locations/derivatives/subIdx。
   - 这是**内存延迟受限**负载。8 线程把各依赖链的 load 流灌入同一 L1/L2/MSHR → 每线程有效 load 延迟↑ → 长链每题延迟乘法叠加 → **无需锁也单样本 10-12×**（15.8→190μs）。
   - 与「无锁、真并行（同时进出）、但单 chunk 膨胀」三者皆自洽。

2. **Tier-2（叠加）= I-cache / code-fetch 争用**：8 线程执行同一段冷递归代码，失同步时互相驱逐 L1I → L1I miss → 每层递归 + 每 virtual call 触发额外取指。单独难构成 10×，但与 Tier-1 叠加后放大显著。

3. **关键澄清：瓶颈不在「17KB 表」，在 locFn 指针追逐 + 长链**：17KB 表（17112B，驻留 L2）是**只读共享读（广播友好，不 ping-pong）**——排除 L3 容量/表容量问题。真正开销在 **locFn 对象指针追逐**（`std::vector<DF>`=shared_ptr<DensityFunction>，locFn 散布堆 + vtable）+ 长依赖链。

**明确排除（对观测 11×，干净确认）**：
- **a-1** run() yield 空转（每批一次，不在 density 区间）❌；
- **b** 超线程（T=8 ≤ 12 物理核，无 SMT 共享执行单元，GHz 恒 2.99 无降频）❌；
- **d** 硬锁（density 内循环 L784-801 无锁）❌；
- **e** Beardifier（null 门控，L797 不执行）❌；
- **c-3** 17KB 表共享读（L2 广播友好，非 ping-pong）❌（表本身非瓶颈，见上）。

**为什么 MVP 复现不了（本条目的关键辨析）**——微基准与生产负载的**结构性差异**：
1. **表太小**：MVP 245 元素全驻留 L1/L2 无争用（production 是 537 节点/17KB + 195 散布堆 locFn——指针追逐的基础）；
2. **locFn 轻量子类**：无真实 shared_ptr 散布堆对象指针追逐（MVP 用 const/连续数组，编译器优化掉虚调用 + 无指针追逐）；
3. **无「8 线程同读同一批共享对象」压力**：MVP 每线程读独立/驻留数据，不构成「8 线程同读同一批 cache-line 打满 load-store/MSHR」的延迟放大器。

### 三、定位（怎么测的 / 怎么诊断出「MVP 路径不通」）

1. **mvp_spline_eval.cpp 决定性对照**：加线程扫描（T=1/2/4/8）+ 每样本成本（100000 点/线程，5 轮取 min），三种形态（constRec / explicit(DFC) / virtual）同测——把「并发放大」从每 chunk 表象剥到「每 spline 采样」的决定性工具（scout 报告 §四 #1 变体 b）。
2. **scout 静态勘探**（concurrent-density-probe-scout.md，只读）：Tier-1/Tier-2 收窄 + 明确排除 a-1/b/d/e/c-3——**静态排除是资产，但只到「收窄方向」，定论要靠对照实验**。
3. **conc_density_probe + WG_PHASETICK**（上一节）：确证 production 的 density 11× 真实，排除探针污染（「测量/探针污染铁律」）。
4. **权威 JSON**（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`）：证伪「locationFunction 嵌套 SplineDF」，所有 spline coordinate 为纯噪声 DF，嵌套仅存于 `points[].value` 数据表。

### 四、修复（暂无；指出 DFC 非良药 + 路径闭合）

- **暂无代码修复**。
- **DFC 非良药**：DFC（显式栈）只消除虚调用/递归（~5% 量级），**不针对 11× 根（共享内存延迟/指针追逐）**，故不是 11× 的良药。上一节「DFC 收益天花板 ~5%」在这条 MVP 路径上**既不能证实也不能证伪**——「非良药」方向结论仍成立（机制不匹配，而非仅凭 ~5% 数字）。
- **「放大 MVP 验证 DFC」路径闭合（❌ 推翻）**：MVP 天然复现不了真实共享内存延迟，任何放大（扩表 + locFn 真递归）都复现不了 → 这条验证路径不可行。若需验证 DFC 或 11× 机制，必须回到**真实 production 树**（195 散布堆 locFn + 长链 + 并发读）上做，而非微基准外推。

### 被推翻假设（❌ 明确标注）

| 假设 | 状态 | 依据 |
|---|---|---|
| DFC 是 11× 的良药（C2ME 式 DFC 编译直排） | ❌ **推翻** | DFC 只消除虚调用/递归（~5%），不针对 11× 根（共享内存延迟/指针追逐）；~5% 在这条 MVP 路径上既不能证实也不能证伪 |
| 「locationFunction 嵌套 SplineDF」（density 11× 主因） | ❌ **误读**（见上一节） | 权威 JSON 证实所有 spline coordinate 为纯噪声 DF，嵌套仅存在于 `points[].value` 数据表 |
| 「放大 MVP（扩表 + locFn 真递归）来验证 DFC 能否消除 11×」 | ❌ **此路不通** | MVP 天然复现不了真实共享内存延迟（表小 + locFn 轻量 + 无共享对象并发压力），任何放大都复现不了 |

### 五、教训

1. **吞吐 vs 延迟必须分开**：并行性能看「每 chunk 延迟（阶段耗时）」，不是 wall/N 吞吐均值——吞吐正常 ≠ 并发无问题。（AGENTS.md 已警告，本课题仍反复犯。）
2. **静态排除不够，要干净实验**：靠 typeid 遍历 / 静态读 JSON 推断「locationFunction 嵌套 SplineDF」是误读；必须用**权威 JSON 数据源** + **干净探针对照**才能定论。scout 静态收窄给出方向，但**定论要靠决定性对照实验**（如 mvp_spline_eval 线程扫描）。
3. **微基准与生产负载特征不同，不能直接外推**：MVP（表小、无真实指针追逐、无共享对象并发压力）复现不了生产 11× 的共享内存延迟——**MVP「没复现」≠「11× 不真实」**（production 的 11× 已由 WG_PHASETICK 干净确证）。微基准只能验证**算法正确性**（显式栈=递归逐位一致 ✅），不能作为**性能/机制**结论的依据。
4. **先钉住主导成本在哪再立项**：DFC 立项基于「消除嵌套密度树递归」，但递归/虚调用只是 ~5%，主导成本是 shift_noise 噪声计算 + 指针追逐（11× 根）——用权威 JSON 树结构提前证伪「坐标递归」，用对照实验确认 MVP 路径不可行，避免在错误方向投入大工程（C2ME 式 DFC 大宗工程应缓）。

### 结论修订标注（相对上一节）

| 上一节结论 | 本节修订 | 依据 |
|---|---|---|
| DFC 收益天花板 ~5%（MVP 实测） | ⚠️ **下调为「既不能证实也不能证伪」** | 该 ~5% 源自 const-array 单线程 MVP（无真实指针追逐），此类 MVP 复现不了生产机制；~5% 非稳健量化结论 |
| DFC 非 11× 良药 | ✅ **保持成立**（依据从「~5% 天花板」改为「机制不匹配」） | DFC 只消除虚调用/递归，不针对 11× 根（共享内存延迟/指针追逐） |
| density 11× 真实 | ✅ **保持成立** | WG_PHASETICK 干净确证；MVP 未复现不构成否定（微基准结构性差异） |

## 🔥🔥🔥🔥🔥 2026-08-23 三：DFC C++ 消除 11× 并发放大实证（核心价值）

> 承接「2026-08-23 一/二节」（11× 真实 + DFC 收益天花板重审 + DFC C++ 立项拍板）。
> 本节记录 **DFC C++ 实现（CpuBackend 直排 + thread_local grid 缓存）的多线程并发放大实测**——这是 DFC 的**核心价值实证**：几乎消除了 production 的 8.4×/9.2×（density 11×）并发争用。
> 工具：`.investigations/perf-rework/vulkan-proto/dfc_cpp_conc.cpp`（thread_local grid 缓存，每线程 chunk 内采样；对标 conc_density_probe=production 的 density 11×）。

### 一、现象（数据：并发放大 T=8 vs T=1 仅为 1.30-1.31×）

**dfc_cpp_conc 每样本 per-sample μs（seed 同、N 同、thread_local grid）：**

| 版本 | T=1 | T=2 | T=4 | T=8 | **并发放大（T=8/T=1）** |
|---|---|---|---|---|---|
| 初版（每点整树 split） | 882.7 | 905.3 | 1021.5 | 1157.6 | **1.31×** |
| splitTop 优化后 | 251.7 | 260.3 | 296.0 | 327.8 | **1.30×** |
| 闭包优化后 | 238 | — | — | 314 | **1.32×** |

**对照 production（conc_density_probe + WG_PHASETICK，2026-08-23 一/二节）：**

| 测量 | T=1 | T=8 | 并发放大 |
|---|---|---|---|
| **production density** | 39.31ms | 331.04ms | **8.4×**（单 chunk 9.2×/11×） |
| **DFC C++** | per-sample 238μs | per-sample 314μs | **1.30-1.31×** |

### 二、结论（核心价值）

- **DFC C++ 的并发放大从 production 的 8.4×/9.2×（density 11×）降到 1.30-1.31×**——**几乎消除了并发争用**。
- **结论（✅）**：DFC 的核心价值被实证——它消除了 **SplineDF 指针追逐/共享延迟导致的 11× 并发放大**（即 2026-08-16 起整个 MT 课题的根因所对应的并发退化）。尽管 DFC 每点绝对耗时仍高（见未解问题），但**并发下几乎不再退化**，这正是「DFC 消除并发争用」的直接证据。
- **意义**：第三节的「DFC 收益天花板 ~5%（因不消除 shift_noise 噪声计算）」与本节「DFC 并发放大 1.3×」**不矛盾**——天花板讨论的是「每点绝对成本」能否被 DFC 大幅降低（不能），本节讨论的是「并发下每点的放大倍数」能否被 DFC 消除（能）——两者是独立维度。
- **对 11× 课题的实义**：production 的 11×（并发每点 15.8→190μs）在 DFC 上不复现（每点 238→314μs，仅 1.3×）。即**并发争用的来源（SplineDF 递归虚调用 + locFn 散布堆指针追逐）已被 DFC 直排消除**——佐证 Tier-1（指针追逐/共享内存延迟）是 production 11× 主因的判别方向。

### 三、未解问题（🔍 待重诊断，诚实声明）

1. **DFC 每点绝对耗时仍高**：DFC per-sample 238μs（T=1）**> production 0.4μs/点**（约 600×）——绝对值仍在数量级上慢于 production。并发放大好（1.3×）不等于绝对快（还没消除慢）。
2. **整 chunk 生成仍可能慢/超时**：每点 238μs × ~98k 点/chunk 仍可能是分钟级——**必须等「每点 238μs 真实主因」优化后再实测**整 chunk 生成 vs production（39ms/chunk）。
3. **每点 238μs 真实主因未明**（闭包优化只降 5% → 主因**不是**孤儿 delegate/节点分派）。候选：
   - ① **grid 构建摊销**（buildInterpGrid 每 chunk 首访建 5×768=768 个 cell 全量 split）
   - ② **sample() 每次仍调 splitTop**（每点仍算 25 条 @c0 split）
   - ③ **eval_density 结构成本**（顶层闭包/outer 非线性链）
   - **下轮重诊断**（单点绝对成本做主因定位，用干净无探针整批 wall + 调用次数计数，AGENTS.md 测量污染铁律）。

## 🚫 2026-08-23 四：DFC CPU 移植失败定论（拆误）——绕圈无果，作废

> 承接上文一/二/三节。本节给「DFC CPU 移植」方向**正式结案**：不是「性能待优化」，而是**方向不可行，作废**。
> 整个 DFC 移植绕了一圈回到「没作用」——本节为准。这是本次课题**最贵的教训（也是最大资产）：不要在「算法重写」上立项，除非先钉死主导成本**。
> 完整记录：`.investigations/perf-rework/vulkan-proto/dfc_cpp_conc.cpp`（并发放大 + 每点绝对耗时）+ `.investigations/perf-rework/gpu-accel-errors.md` D26（闭包化提速仅 5%）+ NEXT_SESSION.md §8 + **`.investigations/worldgen-mt-scaling/dfc-error-full-process.md`（DFC 错误过程完整台账，五段式逐错误步骤）**。

### 一、现象（数据：整 chunk 600× 慢，任何实际场景不可用）

| 测量 | DFC CpuBackend | production | 倍率 |
|---|---|---|---|
| **每点 sample()** | **238 μs** | **0.4 μs/点** | **~600×** |
| **整 chunk（98304 点）** | **≈ 23.4 s** | **39 ms** | **~600×** |

- DFC 每点 238μs（dfc_cpp_conc，T=1）→ 整 chunk = 98304 点 × 238μs ≈ **23.4s**；
- production 单 chunk density = **39ms**（98304 点，InterpolatedDF grid 复用 → 0.4μs/点）；
- **整 chunk DFC 慢 ~600×** → `dfc_fill_compare` 120s 超时 → **任何实际场景不可用**。

### 二、根因（为什么绕圈 / 为什么不可行）

**核心矛盾（净作用为负）**：DFC「消除并发放大 1.30-1.31×（vs production 8.4×/11×）」是用**更大的新问题（整 chunk 慢 600×）**换掉旧问题（并发 11×）——**净作用为负**。消除了一个「原则上可接受」的指标（并发放大倍数），代价是让「单点绝对成本」变成不可用（600×），总体仍然不可用。

**DFC 是 GPU 性质的设计搬到 CPU（用错工具）**——三个为 GPU「无 fp64 + 并行摊销 prefetch」定制的结构，CPU 串行每点付全额：

1. **split-precompute**：每点重算 splitTotal（8672 floats）+ interp 8 角点展开（初版 200 条 split-call；splitTop 后仍 25 条 @c0）。GPU 里 split 用并行工作组摊销；CPU 串行每点付全额。
2. **grid 构建摊销**：buildInterpGrid 每 chunk 首访建 5×49×5 网格（每节点调 `split()` 覆盖整个 splitCoord 8672 floats 全量重算），首访成本高，摊不到后续点。
3. **eval_density 结构成本**：splitTop 每点 25 条 + eval_df 顶层闭包/outer 非线性链逐点走。

这三项在 GPU 里被「无 fp64 妥协 + 并行摊销 prefetch」合理化；CPU 串行每点付**全额**——这就是每点慢的根源，也是 DFC 在 CPU 上结构性不可行（不是还可以再优化的「性能待提升」）。

### 三、立论证伪（关键——说明 DFC 方向从根上就错了）

核心假设「**虚调用是 11× 元凶**」今天已**证伪**：

- 权威 JSON（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`）证实所有 spline coordinate 均为纯噪声 DF（continents/erosion/ridges = `flat_cache(shifted_noise(...))`，ridges_folded = 纯 mul/add/abs 链），**无一嵌套 SplineDF**；
- DFC 消除虚调用 + shared_ptr 只 ~5%（复用 MVP/闭包化实测口径）——**主导成本是 shift_noise 噪声计算**，不是分派/虚调用；
- ⇒ **DFC 消除虚调用只 ~5%，不可能解决 11×**。整个 DFC 方向建立在「消除虚调用就能解决 11×」的错误前提上。

### 四、定位（怎么测的）

1. **dfc_cpp_conc**（thread_local grid 缓存，每线程 chunk 内采样）：per-sample μs，T=1/2/4/8 各版本对照——并发放大 1.30-1.31×，同时暴露每点绝对耗时 238μs（T=1）。
2. **干净无探针整批 wall + 调用次数计数**（AGENTS.md「测量/探针污染铁律」）：确认整 chunk 23.4s 量级，排除阶段计时探针污染。
3. **dfc_fill_compare**：120s 超时——整 chunk 生成在 DFC 下不可用，直接判死。
4. **D26（gpu-accel-errors）**：闭包化砍 87% 节点遍历只换 ~5% 提速——每点慢主因**不是**节点分派/孤儿 delegate；「正确但无用」的优化链收益差一个数量级。

### 五、修复（裁决：作废，而非继续优化）

- **DFC 性能方案 ❌ 作废**：DFC C++ 实现**保留**（对齐链成果），但**接入 WG_DFC_CPU 默认关**；**DFC 不作为性能方案**。
- 未来若再动 DFC，只能作为「正确性参照/对齐基线」用（对齐链：逐位对齐 9.57e-07 vs production / 2.06e-08 vs GPU 蓝本，证明 CpuBackend 正确），**不作为生产路径**。

### 六、真收获（✅ 非无用）与作废标注

**真收获（✅）**：

1. 证伪「虚调用是元凶」——避免未来继续错误方向（本次 DFc 方向的最大价值）。
2. 确认 **production 单点 0.4μs 很快（并发才是问题）**——把问题从「单点慢」重新定位到「并发争用」。
3. 完整 DFC 对齐链（逐位对齐达成，证明 CpuBackend 正确）——「正确的但无用」（每点慢使价值归零）。

**作废标注（❌）**：

| 项 | 状态 |
|---|---|
| DFC CPU 移植作为性能方案 | ❌ **正式作废**（绕圈无果） |
| DFC CPU 移植作为正确性/对齐参照 | ✅ **保留**（CpuBackend 正确，逐位对齐） |
| WG_DFC_CPU 接入 | ⚠️ **默认关**（保留代码，非生产路径） |

### 七、教训（判错经验——本课题最贵资产）

1. **不要用「算法重写」解决并发争用**：MC 的 density 树是 Java 语义的、已经是「一个对象 + 实例数据」形态；DFC 把它重写为 C2ME 式数据驱动直排，只是在 CPU 上制造了一个 600× 慢的「更正确」版本。**并发 11× 的战场在 production 自身的共享可变状态/争用点，不是算法重写。**
2. **先 benchmark 钉住主导成本再立项**（D26 直接教训）：闭包化砍 87% 节点遍历只换 5% 提速；DFC 立项前应先钉死「主导成本是可消除的分派/引用计数，还是不可消除的 shift_noise 噪声计算」——后者（噪声）是主导，DFC 天花板 ~5%，**先量化再立项**。
3. **正确性达成 ≠ 性能达成**：DFC 逐位对齐达成，但性能目标未达——两者独立衡量、分别验收，不能因「对齐保持」就认为优化到位。
4. **吞吐 vs 延迟必须分开**（AGENTS.md 已警告，本课题反复犯）：并行性能看「每 chunk 延迟（阶段耗时）」，不是 wall/N 吞吐均值。

### 🔍 指向下一真课题（承接，详见 10-timewise / NEXT_SESSION §8.2）

production 并发争用的**无损修复**——保留单点 0.4μs 快，修复 11× 并发（**不是 DFC**）。候选：SplineDF locFn 连续化/去 shared_ptr、thread_local grid 已做部分、找其余共享可变状态。**关键：不要再用「算法重写」。**

## 🔬 2026-08-23 五：locFn 连续化 A/B 非主导确认——真实主导=长串行依赖链

> 承接第三节（DFC 消除 11× 并发放大实证）与第四节（DFC CPU 移植失败定论）。本节为「下一真课题 = production 并发争用无损修复」的首个**决定性验证**：**最小 A/B（SERIAL locFn 连续化 vs BASE）**，实证 **locFn 连续化不能修复 11×（放大比持平）**，确认 scout 的候选判断——**真实 11× 主导 = 长串行依赖链（~90 节点/实例）的 load 延迟膨胀**，**不是 locFn 指针追逐**。
> A/B 代码：`density.h` SplineDF 加 SERIAL 路径（`WG_SERIAL_LOCFN` env 门控，BASE 不变）| 测量：conc_density_probe + WG_PHASETICK | 记录：locfn-serialization-ab.md / production-contention-scout.md。

### 一、现象（A/B 数据，conc_density_probe + WG_PHASETICK，12 固定 chunk 中位数）

| 变体 | T=1 | T=8 | 放大比（T8/T1） |
|---|---|---|---|
| **BASE**（`vector<DF>` 散布堆 locFn） | 35.11ms | 352.12ms | **10.03×** |
| **SERIAL**（locFn 全局按类型连续池 + 索引 + kind switch） | 34.76ms | 356.35ms | **10.25×** |

- **SERIAL 放大比（10.25×）与 BASE（10.03×）基本持平**——locFn 连续化**不能修复 11×**。
- **单线程 T=1 也仅微降**（34.76ms vs 35.11ms，<1%）——distinct locFn 对象只 4-6 个且 L2 热，连续化只去掉「每节点 ×1 次 L2 命中 deref」，绝对成本本来就小（与 scout §1.2 预测一致）。

### 二、结论（locFn 非主导，放大比持平）

- **locFn 连续化（Plan A）❌ 不做**：它只消除 A 类（每节点 ×1 次 L2 命中 deref），**不改变** B 类（串行依赖链）/C 类（I-cache 争用）。放大比不降 → **A 非主导**。
- **确认 scout 判断**：真实 11× 主导 = **长串行依赖链（~90 节点/实例 + 嵌套 spline 跳转）的 load 延迟膨胀**（只读共享广播，非 cache-line 写乒乓；scout §2.2 B 类）。
- **locFn 连续化从「11× 候选主修复」除名**——它仍是低风险、无损、有独立价值的小优化（绝对耗时微降），只是**不是 11× 的主方案**。

### 三、定位（怎么测的）

- **最小 A/B**（scout §6 预设判据）：BASE（原 `vector<DF>`）vs SERIAL（`LocFnRef`+按类型连续池，**保留递归+虚调用+thread_local 网格缓存+registry 共享 cacheId**，只去 deref + 池连续）。**其余原封不动**——这是隔离「A 贡献」的唯一可信判据（DFC 同时改 3 件事，无法从 DFC 1.3× 反推）。
- **测量入口**：conc_density_probe（12 固定 chunk + wg_fill_blocks_multi，读 `[PTICK] chunk(x,z): density=ms`）；**WG_PHASETICK**（QPC 单次零污染，AGENTS.md「测量/探针污染铁律」）——禁 WG_PROFILE/WG_STAGETIMER（并发污染）。
- **判据集**：12 个 chunk 的 `density=` 值取**中位数**；`放大比 = 中位数(T8 density) / 中位数(T1 density)`。
- **判读**（scout §6）：SERIAL 放大比显著低于 BASE（向 DFC 1.3× 靠拢）→ A 主导、值得落地；SERIAL 与 BASE 持平（仍 ~10×）→ A 非主导、**不做**。**结果 = 后者**。

### 四、修复（Plan A 不做，转向长链方向）

- **无代码修复**。locFn 连续化（Plan A）**不落地**作为 11× 主修复。
- **下 session 真方向 = 长串行依赖链**（B 类）：~90 节点/实例的每级 load 延迟串行叠加是 11× 主导。候选（待深挖，均为待验证假设）：
  - 提升 MLP / 打破依赖链形态（预取、分块、减少每级数据依赖）；
  - I-cache 争用（C 类叠加）；
  - **不再算法重写**（DFC 教训）——聚焦 production 自身的争用点/布局，保留单点 0.4μs 快（无损修复）。

### 五、教训（判错经验——「先钉死主导再动」有效）

1. **最小 A/B 验证主导，避免重蹈 DFC**：DFC 教训 =「静态推断（虚调用是元凶）→ 大投入 → 失败」；本次**先用最小 A/B（SERIAL 连续化，BASE 不变）在落地前实证「locFn 非主导」**——投入极小（存储布局 + env 门控）、风险极低、结论清晰。**DFC 教训第 4 条「先量化再立项」被成功应用（MT11 教训 4）。**
2. **放大比是 11× 判据，不是绝对耗时**：SERIAL 让 T1 绝对耗时微降，但放大比持平——**「消除一点绝对成本」≠「修好并发 11×」**。并行性能判据 = 每 chunk 延迟的放大比（T8/T1），不是绝对 per-sample 微降。
3. **隔离变量才有可信判据**：只改 locFn 存储、其余原封不动，才能从 A/B 隔离 A 的贡献——多变量同时改（如 DFC）无法归因。
