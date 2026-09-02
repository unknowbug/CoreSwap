# 架构计划：lossless-accel —— 无损加速独立课题（260903-01 立项，重量级）

> 用户 2026-09-03 拍板：从结构/算法层面探索无损（或 GPU 算子利用型）加速，独立成课题；**轻有损路线明确排除在范围外**。
> 本文档为 Phase 0 产物；用户授权「写完不拍板，下 session 直接开工」——开工时按本文 Phase 序执行，**Phase 内首个重大分叉（X1）仍须 judge + 用户裁决**（预置见 §5）。

## 0. 背景与资产盘点（现状 = 三个半成品互不衔接）

| 资产 | 归属 | 状态 | 关键事实 |
|---|---|---|---|
| GpuDensityEngine（Vulkan + DFC shader + spline 6 表 SSBO） | C++ 侧 `vulkan-proto/` + `worldgen/src/`（`WG_GPU_FILL` 门控，默认关） | 已收官归档 | e2e 域内与 CPU 参照**逐位一致**（maxDiff 3.1e-07）；网格角点批量 **22-39×**；逐 block 方案 D24 定性不可行（8672 floats/点 → 3.4GB/chunk 带宽死局） |
| 正确方向：GPU 算 InterpolatedDF 角点 + CPU 插值 | 未实施（D24 遗留） | 🔍 待立项 | 768 角点/chunk = 27MB（数据量降 ~125×），**本课题 P2b 的核心** |
| TranspilerDensity（CPU 侧 transpile 生成代码） | Rust 侧 `terrain.rs`（`WG_TRANSPILER` 门控） | candidate，**已生产接线** | vs macro sampler 98304 点 max_diff=0；块级 99.30%；**vs vanilla 94.20% 比基线 95.40% 低 1.2pp，归因未做（欠账）** |
| spline 扁平化 + MT 膨胀根因 | `worldgen-mt-scaling` 课题挂起 | 🔍 未闭合 | 单线程 -24%；并发下 density 50→462ms（~11×），根因 = InterpolatedDF::buildGrid 1225 角点树遍历（spline 递归 + FlatCache + noise cache miss 叠加）；**待解方向 = DFC 整树扁平化** |
| MT3 clamp 结构性串行 | C++ `wg_fill_blocks_multi`（8/16 诊断） | 🔍 待修 | `if (threads > count)` clamp → count=1 时池恒 1 worker；**Rust 侧是否同款存在待核对** |

**结构性事实**：当前生产 dll = **WorldgenRust**；GPU 引擎在 **C++** 侧——存在实现语言鸿沟，是 X1 分叉的来源。

## 1. 课题目标

在不牺牲已验证对齐质量（零退化铁律）的前提下，把生成吞吐做上去：

1. **P2a（Rust DFC 化）**：把 density 采样的整树遍历（buildGrid 1225 角点 + spline 递归）编译为扁平直排，目标解掉 MT 膨胀 11× + 承接单线程 -24% 已证收益。
2. **P2b（GPU 角点方案）**：GPU 只算网格角点（768/chunk）+ CPU 照旧插值——插值路径与现状完全相同，角点逐位一致 ⇒ 端到端**无损**；结构级延伸 = split 派生逻辑上移 shader（int64 哈希 GPU 精确），上传量从 27MB/chunk 降为「seed+origin」级。
3. **前置欠账清偿（P0）**：见 §2。

**排除项（不进范围）**：轻有损近似（fp32 中间量化/spline 查表插值/跳 octave）；MT3 之外的历史性能遗留（H3 ×16 重测只随 P2a 顺带做，不单独立项）。

## 2. 前置欠账（P0，开工第一件事）

1. **transpiler 1.2pp 归因**：TranspilerDensity vs vanilla 94.20% 低于基线 95.40%——不归因则「加速前后对比」的基线本身有污点（加速优化可能吃掉或放大这 1.2pp，无法归责）。候选方向（07 篇 M12 已记）：cache_all_in_cell 点级缓存 vs Java cell 级语义差、channel 采样细微差。**归因未闭合前，P2a 的验收基线 = macro sampler 路径，不是 vanilla**。
2. **clamp 结构性串行核对**：Rust `wg_fill_blocks_multi` 是否存在同款 `if (threads > count)` clamp（C++ 侧 8/16 诊断的移植检查）——存在则一行修复（`&& count > 1`），直接影响实机验证口径。

## 3. 验证口径（验收判据，预注册）

- **零退化铁律**：新路径默认 env 门控关闭；seed 8576/3200 双口径，门控关闭时输出与基线 dll 逐位一致（sha256 级）。
- **无损证明（P2a）**：DFC 路径 vs 现路径全 chunk 98304 点 `max_diff=0.000000`（对齐 TranspilerDensity 验证范式，`{:.6}` 舍入内）；块级一致 ≥ 基线路径。
- **无损证明（P2b）**：GPU 角点 vs CPU 角点逐位一致（对齐 gpu_fill_probe 范式）；且**必须覆盖 algorithm-fingerprints #13 清单**——多 chunk（含 chunk 0 外）/多 cell（cy≥1、cz≥2）/多 y 层（含常数分支层）。I5 教训：e2e 域外系统性错值（maxDiff 2e-1）只有多域抽查才暴露。
- **性能口径**：端到端 vs Java 原版铁律（充分预热 + 大样本 ≥256 chunks 取稳定中位数 + 排除冷启动）；并行性能只信「无探针整批 wall + 调用次数计数」；阶段计时唯一可信 = WG_PHASETICK。**吞吐探针必带逐点 diff 抽查**（免费正确性门）。
- **§9.7 可比性声明**：任何加速比数字同行声明载体/覆盖面/与既有口径可比性（探针口径 22-39× 不得直接外推端到端）。

## 4. 已知硬约束（设计时绕不开）

1. **GPU 并发互斥**：共享 buffer 上传/dispatch 无锁 → 驱动级 0xC0000005（非错误返回）；多线程 fill 必须 mutex 或 per-thread 实例/批量提交。
2. **带宽账先算**：任何 GPU 方案先算「每点喂多少数据 × 点数 ÷ PCIe ~16GB/s」，不算「每点算多少」（D24 教训）。
3. **编译时间域**：spline/DFC 生成器结构改动后，「编译慢根因」结论必须重跑减法二分（algorithm-fingerprints #13/#V9——const 表 vs SSBO 结论有版本域）。
4. **诊断代码禁入热路径**：env 查询/atomic/时钟不得逐点执行（27% 退化前科）；门控必须 chunk 级判断一次。
5. **构建**：Rust 侧 cargo；C++ 侧如动用 build.ps1；诊断 bin 一律进 `WorldgenRust/src/bin-diag/` 隔离区。

## 5. 分叉点与角色预置（执行期只核对不补排）

### 分叉 X1：GPU 通道选择（P2b 开工前 MUST 裁决，fan-out + judge + 用户拍板）

- **候选 A：C++ GpuDensityEngine 经 FFI 借给 Rust**——复用已验证引擎（逐位一致 + 22-39× 实证），代价 = 跨语言边界（FFI 封装、调用 marshalling、构建链耦合 C++/Rust 双侧）。
- **候选 B：Rust 侧重写 GPU runtime**（wgpu 或 ash/Vulkan）——dfc_gen.py 已能产 GLSL，shader 资产可直接复用；代价 = runtime 重写 + 重新验证（但验证管线现成）。
- **候选 C：降级务实路线**——GPU 只用于 bench/离线批量，生产 Rust 走 CPU DFC（P2a）——若 X1 两候选工作量/风险超预期，此为回退位。
- **fan-out 预置**：X1 触发时派 2-3 个 worker subagent 并行产 `.bN` 候选评估（工作量/风险/验证成本量化），judge 对比后**用户拍板**（重大方向变更 HOOK）。

### 角色介入点预置

| 子角色 | 介入点 |
|---|---|
| scout | P1 开工必做：Rust 侧 `fill_chunk`/`TranspilerDensity`/`buildGrid` 热路径勘探 + GPU 通道候选的现状摸底（只读，subagent，产物 `.investigations/lossless-accel/`） |
| worker | P0 归因（transpiler 1.2pp 多候选 → 若 ≥2 互斥假设则转 fan-out）；P2a/P2b 实现（主会话收敛闭环，subagent 交付隔离件时用） |
| fan-out | X1（上述）；P0 归因若分叉；P2a 若 buildGrid cache miss 构成出现 ≥2 互斥归因 |
| judge | X1 裁决前；P2a 完成授 candidate 前；课题收尾交付（三源核对） |
| knowledge | 每个 Phase 末：subagent 产出 docs/10 时间线条目 + 可复用发现（GPU/DFC 相关进 vulkan-gpu-programming.md / algorithm-fingerprints.md）；主会话只应用 |

## 6. Phase 划分与回退

```
P0 前置欠账（clamp 核对 + transpiler 1.2pp 归因）        ← 下 session 开工点
P1 scout 勘探（Rust 热路径 + GPU 通道现状，只读）
P2a Rust DFC 化（整树扁平化，WG 门控，零退化 + max_diff=0 验证）
   └─ 顺带：H3 ×16 修复后重测（mt 侧 spline 单次成本定性）
P2b GPU 角点方案（X1 裁决后：角点 GPU + CPU 插值；延伸 = split 上移 shader）
P2.5 验证（§3 全项；evidence saturation 生效：3 轮无新数据层证据回数据层）
P3 judge 收尾审查 → 用户拍板 candidate/confirmed
```

**回退位**：任何阶段失败/超预期 → 门控默认关 = 生产零退化（I8 范式）；X1 可降级候选 C；P2b 可整体封存而 P2a 独立交付。

## 7. 预算与纪律

- 课题目录：`.investigations/lossless-accel/`（错误台账 `lossless-accel-errors.md`，五段式）+ `.artifacts/lossless-accel/`（结论 + index.yaml 登记）。
- retry cap = evidence saturation（§9.4）；C-gate halted escalation（同判据 3 次未满足 → halt + 人类裁决）。
- 性能数字一律带 §9.7 三要素声明；结论取代用 supersedes 双指针（§15.4）。
