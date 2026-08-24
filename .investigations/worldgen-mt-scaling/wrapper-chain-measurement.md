# wrapper 链隔离测量 — 实测数据 + 排除法推理（2026-08-24 后续）

> 角色：主会话测量 | 状态：draft（排除法推理 = candidate；spine 1.2× 有线程模型局限，仅作辅证）
> 目的：把 11× 并发放大的争用锁定到 wrapper 链（spline 之外）。

## 0. TL;DR

- **次靶 ①（devirtualize spline.locFn）无效**（production 模型，可靠）：DEVIRT 10.05× vs BASE 10.32×。
- **三次 production 模型实验（SERIAL/NOSPLIT/DEVIRT）都只改 spline 内部 → 全部无效**（10.25×/9.9×/10.05×）→ **争用不在 spline 内部** → 在 wrapper 链（spline 之外）。
- spline 1.2×（std::thread）仅作辅证，**有线程模型局限**（见 §3），不作为主证据。
- **候选收敛 = wrapper 链（InterpolatedDF/min/blend_density/add/mul 等顶部 + buildGrid 链虚调用 + 寻址）**。

## 1. 次靶①（devirtualize spline.locFn）— production 模型，可靠

改动：density.h `sampleSerialLocFn` 去掉 `static_cast<const DensityFunction&>`（3 case），具体类型直接调 `.sample(pos)`（by-value 池，语义保证 devirtualize）。env 门控 `WG_SERIAL_LOCFN=1`。

测量（conc_density_probe，12 固定 chunk，WG_PHASETICK，median density ms）：

| 变体 | T=1 median | T=8 median | 放大比 |
|---|---|---|---|
| BASE | 33.54 | 346.26 | **10.32×** |
| DEVIRT（+WG_SERIAL_LOCFN=1） | 34.03 | 342.06 | **10.05×** |

放大比 10.05× vs 10.32×（降 ~2.6%，噪音/漂移内；未达 DFC 1.3×）。**① 无效**。

## 2. 排除法推理（主证据，全部 production 模型 = conc_density_probe，可靠）

三次实验**都只改动 spline 内部**、不碰 wrapper 链、均无效：

| 实验 | 改 spline 内部 | 放大比 | 结论 |
|---|---|---|---|
| BASE | — | 10.32× | 基线 |
| SERIAL（locFn 存储连续化） | locFn **存储** | 10.25×≈BASE | ❌ 无效 |
| NOSPLIT（spline 递归→显式栈） | **递归** | 9.9×≈BASE | ❌ 无效 |
| DEVIRT（locFn 虚分派 devirtualize） | locFn **虚分派** | 10.05×≈BASE | ❌ 无效 |

**逻辑**：spline 内部的候选（存储/递归/虚分派/寻址）已被三次生产模型实验逐个排除。11× 争用**残留于 spline 之外** = **wrapper 链**（finalDensity 顶部的 min/squeeze/interpolated/blend_density/add/mul...→spline，15-20 层纯委托虚调用 + buildGrid 链虚调用 + 寻址）。

## 3. spline 1.2×（辅证，有线程模型局限）

用 conc_sample_probe 直接采样单个 SplineDF（wg_sample_spline，绕 wrapper 链）测并发放大：

| SplineDF | 节点 | T=1 per-sample | T=8 per-sample | 放大 |
|---|---|---|---|---|
| [0] | 135 | 4139.9ns | 5039.0ns | **1.22×** |
| [2] | 254 | 4493.5ns | 5439.3ns | **1.21×** |

⚠️ **局限**：此测量用 conc_sample_probe 的 **std::thread** 模型（各线程独立循环采样），而 production 争用（10.32×）用 **wg_worker pool**（wg_fill_blocks_multi 填 chunk）。**两者线程模型不同**——std::thread 下多入口都低放大（noise 1.15×/spline 1.2×），无法排除「std::thread 模型本身无争用」的伪影。故 spline 1.2× **不能**独立证明「spline 在 production 下无争用」，仅作辅证。

**关键修正（测量）**：wg_sample_density（whole tree 单点）**无 grid 缓存（每点 buildGrid）→ 单点 ~6ms → std::thread 循环 20000 点 120s 超时**。故「同探针 strict 对照 density vs spline（std::thread）」不可行。

## 4. 结论（candidate）

- **11× 争用 = wrapper 链**（spline 之外），排除法推理可靠（3 次 production 模型实验均只改 spline 内部且无效）。
- spline 1.2× 是辅证（std::thread 局限）。
- 修复方向：针对 **wrapper 链的 buildGrid 链虚调用 + 寻址**（InterpolatedDF/min/blend 等顶部），即整棵 density 树 top 层数据驱动化（无 split DFC）。

## 5. 下一步（建议）

**production 模型下严格测 pure spline（绕 wrapper）**：在 wg_fill_blocks_multi/fillOneChunkCore 加 env 门控「density 只算某 spline（绕 wrapper 链）」，用 production 线程池测其放大 vs 全 tree（含 wrapper）。**消除线程模型混淆**，严格证实 wrapper 链贡献。

> 若此严格对照证实 spline（production 模型）也 ~10× → 则争用在 spline 自身（与排除法矛盾，需重审）；若 spline <<10× → wrapper 链是主争用（确认）。

---

## 6. ✅ production 模型严格证实（2026-08-24 后续，已执行 §5）

改动：`WorldgenHandle.splineOnlyIdx`（WG_SPLINE_FILL=which env）→ fillOneChunkCore density 采样绕 wrapper，直接 `spl[which]->sample(fpos)`（production 线程池 wg_fill_blocks_multi + conc_density_probe，同 conc_density_probe 全 tree 基线共用同一探针/线程池）。

| production 模型 | T=1 median | T=8 median | 放大比 |
|---|---|---|---|
| **全 tree（含 wrapper 链）** | 33.54ms | 346.26ms | **10.32×** |
| **spline-only[2]（绕 wrapper）** | 3.015ms | 4.895ms | **1.62×** |

**关键**（消除线程模型混淆，同一探针同一线程池，只差 wrapper 链）：
- spline 自身并发放大仅 **1.62×**（几乎无争用）。
- wrapper 链把 1.62× 拉到 **10.32×**（6.4× 放大贡献）。
- **wrapper 链占 density 时间 ~91%**（全 tree 33.54ms vs spline-only 3.015ms）。

### 结论（production 模型严格证实 = 强 candidate）
**11× 争用 = wrapper 链（InterpolatedDF/buildGrid 链虚调用 + 寻址），不是 spline。** spline 自身（含 locFn 噪声/递归/locFn 虚调用/寻址）并发放大仅 1.62×，几乎无争用。

**修正 scout 认知**：scout 原判「spline 宽递归是主争用」→ 实测 wrapper 链占 density 时间 91%（30.5ms/33.5ms）且贡献 6.4× 并发放大，spline 仅占 9% 时间 / 1.62× 放大。

### 具体争用机制
`InterpolatedDF` 每 chunk 首次采样触发 `buildGrid`（density.h L589-607），内部逐网格点调 `arg->sample(p)`（L607）→ arg 是下层 DF（InterpolatedDF/blend/min...）→ 递归建各层 grid → **buildGrid 多层虚调用链**。8 线程同时填 12 chunk 各触发自身 buildGrid → 该链虚调用 + 寻址被共享缓存层级放大约 10×。

### 修复方向（更新）
**优化 wrapper 链的 buildGrid 虚调用链**——即 InterpolatedDF（含 min/blend_density/add/mul 顶部）的 `arg->sample` 建 grid 虚调用，数据驱动化（去虚调用）。**不是 spline**（spline 已证 1.62× 无碍）。

> 注：wg_sample_density 单点无 grid 缓存 → std::thread 超时不可用；故用 WG_SPLINE_FILL（production 模型）作为严格对照，可靠。

---

## 7. ✅ warm vs cold 区分 buildGrid vs 顶层逐点（2026-08-24，修正 §6 后）

改动：fillOneChunkCore 加 `WG_WARM_GRID=1` → 预建 grid（对 chunk 中心点调 finalDensity->sample 触发 InterpolatedDF 懒建 grid），**排除 buildGrid 深链虚调用争用**，只剩顶层逐点 wrapper 包装虚调用。

| production 模型 | T=1 median | T=8 median | 放大比 |
|---|---|---|---|
| **cold（含 buildGrid 深链）** | 33.54ms | 346.26ms | **10.32×** |
| **warm（排除 buildGrid）** | 34.28ms | 346.12ms | **10.10×** |

**warm ≈ cold（差 0.22×）** → **buildGrid 深链虚调用争用贡献微乎其微**（<2%）。

### 修正 scout §6 结论（关键）
scout 断言「buildGrid 深链 = 91% 主争用」**有误**。warm（去 buildGrid）放大仍 ~10×。**11× 主争用 = 顶层逐点 wrapper 包装虚调用**（min→squeeze→mul→interpolated，每 chunk 98304 点 × 3-4 层虚调用 + 寻址）。

### 最终争用定位（production 模型，同探针三对照）
| 变体 | 覆盖 | 放大比 | 结论 |
|---|---|---|---|
| spline-only[2] | 绕全部包装（spline 自身） | 1.62× | spline 无碍 |
| warm | 去 buildGrid，留顶层逐点包装 | 10.10× | buildGrid 无碍 |
| cold | 完整 | 10.32× | — |
=> **争用 = 顶层逐点 wrapper 包装**（min/squeeze/mul/interp 每点 3-4 层虚调用+寻址，98304 点/chunk）。

### 修复方向（更新，更聚焦）
**优化顶层逐点 wrapper 包装虚调用**（finalDensity 树顶点 min/squeeze/mul/InterpolatedDF 的每点调用）。**不是** buildGrid（每 chunk 一次无碍），**不是** spline（1.62× 无碍），**不是**整棵树 DFC（过重）。
最轻量化：顶层 few 层 wrapper 的每点虚调用 → 数据驱动化（kind-switch 去虚调用），或合并纯委托层、减少每点调用层数。

---

## 8. ✅ WG_FLAT_TOP 数据驱动化（4→2 虚分派）— 负面结果（2026-08-24）

改动：`WG_FLAT_TOP=1` 识别 finalDensity = `min(squeeze(mul(0.64, InterpolatedDF)), noodle)`（worldgen_api.cpp dynamic_cast BinaryOperation MIN / UnaryOperation SQUEEZE / LinearOperation MUL），扁平化温暖 a 链：内联 `mul(0.64*)、squeeze(applyUnary)、min(da<bmin?da:min(da,b))`，温暖每点 4→2 虚分派（去 min/squeeze/mul 3 层）。

| 变体 | T=1 median | T=8 median | 放大比 |
|---|---|---|---|
| 生产（cold） | 33.54ms | 346.26ms | 10.32× |
| **WG_FLAT_TOP（4→2 虚分派）** | 34.34ms | 362.41ms | **10.55×** |

**WG_FLAT_TOP ≈ 生产（10.55× vs 10.32×，持平甚至略高）**。

### 关键结论（负面，排除 scout §5 candidate）
**减少虚分派层数（4→2）不降 11×**。scout 的「数据驱动化 min/squeeze/mul 降 11×」candidate **被证伪**。**11× 争用不是「虚分派层数多」导致**。

### 更新排除链
| 变体 | 改动 | 放大比 | 结论 |
|---|---|---|---|
| spline-only | 绕全部包装 | 1.62× | spline 无碍 |
| warm | 去 buildGrid 深链 | 10.10× | buildGrid 无碍 |
| WG_FLAT_TOP | 去 min/squeeze/mul 虚分派 | 10.55× | **虚分派数无碍** |
| 生产 | 完整 | 10.32× | — |
=> **11× 争用仍在 interp/noodle 采样内部**（grid 数组读 + 多层缓存 + 共享读），**非**虚分派数、**非** buildGrid、**非** spline、**非** min/squeeze/mul 虚分派。

> 注：WG_FLAT_TOP 性能结论（≈生产）强（值 T1 34.34ms≈生产 33.54ms）；**逐位一致性已 block_probe 对拍通过**（out_prod.bin vs out_flat.bin SHA256 完全一致 → WG_FLAT_TOP 保正确）。⇒ 「减少虚分派不降 11×」结论可信。
