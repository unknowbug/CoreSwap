# 候选 b2 —— initial_density 宏观粗化查表（coarse-table）设计文档

- **status: draft**（fan-out 候选 .b2，未经 judge/用户拍板）
- 置信度：**candidate（设计层）**，核心可行性判定基于静态代码阅读 + 已实测数字推算，Degraded/静态审查分层；未运行任何验证。
- 日期标签：260903（随父 session Q-AQ1 / lossless-accel 课题）
- 前置调研：`java-est-cache-semantics.md`（P1，candidate）
- **一句话结论先行：「以插值/拟合为核心的粗化查表在逐位一致硬约束下不成立；bit-exact 框架下 b2 只能退化为（i）与 b1 同域的精确值缓存，或（ii）树内插值噪声角点精确值 memoization（高风险深改）。本候选按主形态判定为不成立，备选形态降格为 b1 的补充而非替代。」**

---

## 1. 核心思路与可行性判定（硬约束：同 bit 输出）

### 1.1 被优化的计算

est 冷扫描（`WorldgenRust/src/aquifer.rs:282-299`）：每量化列（`(bx>>2)<<2, (bz>>2)<<2`）自 `min_y + height` 向下按固定步长扫到 `min_y`，逐点调 `self.initial_density.sample(&NoisePos{...})`（aquifer.rs:294），首个 `> 0.390625` 的 y 即返回，全空返回 `i32::MAX`（:288,296-298）。实测 7342 次全价采样/chunk × 2117ns/sample ≈ 15.5ms/chunk。

`initial_density_without_jaggedness` 在 Rust 侧构建于 `worldgen_handle.rs:230`（`db.build_node(router.get("initial_density_without_jaggedness"))`），是 seed+坐标的纯函数（无 chunk 局部可变状态）——**纯函数性成立，这是任何缓存类方案的前提，b2 满足。**

### 1.2 粗化查表为何不满足逐位一致（主判定）

est 的输出不是密度值本身，而是**首穿越 y 的离散索引**。逐位一致要求：扫描路径上**每一个被比较的 y 级**的精确比较结果（`f(bx, l, bz) > 0.390625`）与现行实现完全相同。由此推出两个不可绕过的结构性约束：

1. **首穿越之上每级都必须精确求值**。扫描从顶部开始、返回首个 > 阈值的 y；其上每一级都必须被真实求值并比较（值 ≤ 阈值）。不存在「用粗表跳过一段 y」的自由度——跳过区间内任何一级的值若实际 > 阈值，返回的 y 就会不同，est 差 1 格即改变 aquifer fluid level 判定（AquiferSampler.java:364-382 的 `o = n + 8` 比较、:432 的 `Math.min(surfaceEstimate, q)`），进而改变地形。
2. **粗网格插值不可能逐位复现原函数**。`initial_density_without_jaggedness` 树内部已含多层 InterpolatedDF/SplineDF 插值（其自身网格精度由 `size_horizontal/size_vertical` 定义）；任何外部粗网格（如 4×4 列 / 8 格 y）上采样再插值，得到的是另一个函数 f̃ ≠ f。f̃ 在扫描格点上的值与 f 逐位相等的概率为 0（f64 插值系数非平凡时），比较结果在 f 接近 0.390625 的各级上必然翻转。**粗化查表 = 换一个近似函数 = 放弃逐位一致。**

3. **二分/单调性剪枝同样不成立**：f 沿 y 无单调性保证（地形树含 mountains/caves 项），不能保证「顶上某级 ≤ 阈值 ⇒ 其上所有级 ≤ 阈值」之外的任何剪枝；而约束 1 已说明顶部逐级精确比较不可避免。

**结论：主形态（粗网格预计算 + 精化/插值）在「同 bit 输出」硬约束下不成立。** 原因清单：
- R1 首穿越语义要求扫描路径逐级精确比较，无区间跳过自由度（结构性，见 1.2-1）；
- R2 任何插值引入的 f64 偏差会在阈值附近的比较上翻转为不同 est 值（精度层面）；
- R3 唯一能做到逐位一致的「查表」是**在精确采样格点上存精确值**——而扫描格点集（列 × 全部 y 级）上的预计算总量 ≥ 冷扫描本身的计算量（冷扫描平均 ~34 次/列即停，全表需 96 级/列，见 1.4），预计算反而更贵。

### 1.3 剩余的 bit-exact 变体（不是「粗化」，如实列出）

- **V-a 精确值跨列/跨 chunk 缓存**：est 结果按 (worldSeed, noiseParams, 量化列) 缓存。这就是 P1 §4「方案 a」+ b1 的领地，**b2 在此无增量**——见 §4 重叠分析。Java est 采样用 `UnblendedNoisePos`（ChunkNoiseSampler.java:234），即 init 采样本身旁路 blend density，跨 chunk 缓存的 blend 污染风险天然较低；⚠️ 但 Rust 侧 `init.sample(&NoisePos)` 是否等价于 Unblended 语义需先对拍确认（P1 闸门 3 同源）。
- **V-b 树内插值噪声角点精确 memoization**：冷扫描每列自顶向下访问固定 y 格点，相邻 y 级（Rust 步长见 §6 疑点）落在 InterpolatedDF 单元内时**重复计算相同的角点值/样条求值**；在 DF 求值引擎层对角点精确值做 memoization（存原始 f64，不插值），理论逐位安全。但这改的是 SplineDF/InterpolatedDF 热点引擎本身（density-latency-rootcause 定位的主热点），改动面与回归风险远超 est 课题边界，且**没有任何已实测数字支撑其命中率**——按「禁止自由参数凑数」纪律，本档不给收益数字，只标注为独立候选方向。

### 1.4 「同 bit 输出」证明策略（若强行推进 V-a/V-b 的验收形态）

- 差分 oracle：优化前后同 seed 大样本（≥256 chunks）`initial_density.sample` 在扫描格点集上逐点 f64 位相等断言 + est 值全等断言 + block_probe 逐位（Full 层）。
- 边界集定向：阈值附近（|f−0.390625| < 1e-9）采样点专项枚举；`i32::MAX` 空列路径（P1 G4 对拍点）。
- 任何一步位不等 → 方案即刻判死，不允许「误差在容忍内」的降级叙述。

---

## 2. 查表结构（仅对假设成立的 V 形态有意义；主形态不适用）

> 主形态（插值粗表）因 §1.2 判死，本节仅记录设计推演结论，防止后续重走。

- **粒度**：插值粗表无论列级（4×4）还是区域级（chunk/16×16），都绕不开 §1.2-R2；精确值表的粒度即 est 缓存现有粒度（量化列，aquifer.rs:283-287，CACHE_DIM=32 / CACHE_OFF=12,4，aquifer.rs:77-79）。
- **容量/内存界**：精确值表每列一个 i32；跨 chunk 持久化需 LRU + key 含 (worldSeed, noiseRouter 参数)；容量界与 b1 相同，无独立设计价值。
- **失效策略**：est 纯函数 → 参数不变即永不失效；blend 键控若 V-a 证实 init 无 blend 输入则可豁免（待对拍）。
- **blend chunk 交互**：Java init 采样经 UnblendedNoisePos 旁路 blend（ChunkNoiseSampler.java:234）；若 Rust 语义一致，blend 旁路不是障碍；若 Rust init 树内含 blend 输入，则跨 chunk 表必须按 blend 状态键控/旁路——这是 V-a 的第一对拍点，也是主形态的又一否决加权项（主形态连本 chunk 内列间都无法一致，blend 只是雪上加霜）。

---

## 3. 预期收益量级（只用已实测数字）

| 项 | 数值 | 来源 |
|---|---|---|
| est 冷扫描全价采样 | 7342 次/chunk | confirmed 实测（父 session） |
| 单次 initial_density 采样 | 2117ns | confirmed 实测 |
| est 冷扫描总成本 | 7342 × 2117ns ≈ **15.5ms/chunk** | 上两行乘积 |
| est 调用/chunk | 214 次 | confirmed 实测 |
| 平均每 est 采样次数 | 7342/214 ≈ 34 次（= 平均扫描 ~34 级后首穿越，上限 96 级） | 前两行除法 |

- **主形态（插值粗表）：收益 0**——方案不成立（§1.2），无收益可计。
- **V-a（=b1 域）**：上限即 15.5ms/chunk 中可被「避免重复」消除的部分；单独上限见 §4。
- **V-b（树内 memo）**：无实测命中率数据，**按纪律不给数字**；其理论上限同样被 15.5ms/chunk 封顶（est 侧），但代价可能波及主 density 路径的收益（负面风险，无法量化）。

---

## 4. 与候选 b1（缓存生命周期对齐+共享，P1 G5）的收益重叠分析

- **b1 机制**：消除同 chunk 内 NOISE 路径（worldgen_handle.rs:446）与 carver 路径（:547）各建一份 Aquifer 导致的重复冷扫描（P1 G5——Java sampler 挂 Chunk 跨阶段共享，Rust 双份）。b1 **不减少采样次数上限，只消除重复**。
- **b2 主形态**：目标是用查表替代采样本身——若成立则把 15.5ms 直接砍掉，b1 变得无意义（没有采样了就没有重复）。**但主形态已判死。**
- **b2 V-a 与 b1 关系：同一机制的两种覆盖面，可叠加但收益同池**：
  - b1 单独：节省 = 双 Aquifer 造成的重复冷扫描部分（carver 路径重复量；具体占比无独立实测，上限 15.5ms/chunk，实际远小于此——因为 NOISE/carver 两路径的 est 查询列集合重叠度未测）。
  - V-a（跨 chunk/跨阶段持久）单独：上限 15.5ms/chunk（若全部列命中，热世界近似可达）。
  - 叠加：V-a 落地后 b1 的增量收益 → 0（b1 的共享缓存成为 V-a 的子集）；b1 先落地则 V-a 在其上仍能吃到跨 chunk 部分。**结论：算法上可叠加、收益上同池且 b1 ⊂ V-a；互斥性不存在，边际收益互斥。**
- **各自单独上限**：b1 = ≤15.5ms/chunk（实际 = carver 重复量，待测）；V-a = ≤15.5ms/chunk；b2 主形态 = 0（不成立）。

---

## 5. 实现风险清单 + 工作量估计

### 风险清单（按主形态判死 → V 形态残余风险）

| # | 风险 | 等级 | 说明 |
|---|---|---|---|
| K1 | 主形态位偏差（结构性） | **致命（已触发）** | §1.2-R1/R2，插值必翻比较位 |
| K2 | Rust init 是否等价 UnblendedNoisePos | 高（待对拍） | 决定 V-a 的 blend 键控；aquifer.rs:294 用通用 NoisePos，与 Java :234 Unblended 语义对拍点 |
| K3 | 扫描步长疑点：Rust `l -= 8`（aquifer.rs:295）vs Java `verticalCellBlockCount`（ChunkNoiseSampler.java:233；overworld 常规为 4）；P1 文档 G3 称「一致（4 步进）」与代码字面不符 | 高（待核） | 若为真差异，est 已不一致——但项目整体逐位 confirmed，更可能 Rust 侧 min_y/height 语义换算不同或 P1 文档笔误；**b2 任何形态开工前必须先核清**（不影响 b2 判死结论） |
| K4 | V-b 触碰 SplineDF/InterpolatedDF 引擎 = 既有主热点（density-latency-rootcause 定论） | 高 | 引擎级改动，回归面 = 全部 density 输出；多线程下 memo 表竞争需要 sharded/lock-free 设计，与 density 阶段并发 11× 慢的背景相互干扰 |
| K5 | 跨 chunk 表的多线程竞争（V-a）：worldgen 线程池并发 fill，全局表需并发结构 | 中 | 参考 CoreSwapPool 竞争教训（AGENTS.md 线程池铁律） |
| K6 | 内存无界/键污染 | 低 | LRU + (seed, params) 键控 |

### 工作量估计（对照 b1）

- **b1（参照面）**：改 Aquifer 生命周期/共享 surface_cache，改动集中在 worldgen_handle.rs 两处 Aquifer::new 与 aquifer.rs 缓存载体——小改动面（估计 1-2 天含对拍）。
- **b2 主形态**：不实施（判死）。若强行推进：需要新粗表数据结构 + 精化逻辑 + 全量位对拍，且按 K1 必然失败——工作量无限趋近于「证明不可行」（本文档已完成）。
- **V-a**：约等于 b1 + 跨 chunk 表（key/LRU/并发）≈ b1 的 2-3 倍改动面，但收益增量 = b1 之外的跨 chunk 部分（未测）。
- **V-b**：DF 引擎级改造（SplineDF/InterpolatedDF memo 化 + 并发表），独立大课题（数天-周级），且与 C2ME 式 DFC 编译直排方向（NEXT_SESSION 待办 6）重叠——**建议并入 DFC 直排课题统一评估，不单独立项**。

---

## 6. 判定汇总

| 形态 | 逐位一致 | 判定 |
|---|---|---|
| 主形态：插值/拟合粗表 + 精化 | ❌ 不可能（结构性+精度） | **不成立，不实施** |
| V-a：精确值跨列/跨 chunk 缓存 | ✅（条件：K2/K3 核清） | 成立但 = b1 超集，**并入 b1 课题**，b2 无独立身份 |
| V-b：树内角点精确 memo | 理论 ✅ | 改动面/风险超界，**并入 DFC 编译直排课题**评估 |

b2 作为独立候选**不成立**；其可取残余已分别归并 b1 与 DFC 直排两个既有课题。

## 来源索引

| 结论 | 来源 |
|---|---|
| est 扫描实现/步长/阈值/缓存 | aquifer.rs:282-299, 77-79, 123-124; worldgen_handle.rs:64, 230 |
| init DF 构建 | worldgen_handle.rs:230 |
| Java est 语义/UnblendedNoisePos/verticalCellBlockCount | ChunkNoiseSampler.java:222-241, 129-132 |
| est 成本数字（7342/2117ns/214/15.5ms） | 父 session confirmed 实测（Q-AQ1） |
| b1/G5/blend 闸门 | java-est-cache-semantics.md（P1）§3-G5, §4 |
