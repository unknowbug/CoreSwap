# 草稿：07 篇「aquifer 内部精确定位」补充小节文本

> 载体：`versions/1.20.1/docs/07-block-pipeline.md`「2026-08-29 Rust worldgen 端到端性能定位」小节内**补充/修正**（追加不覆盖原则下，新增小节而非改写既有污染态小节）。
> 数据源：`cmd-output/aquifer_internal_precise.txt`、`cmd-output/aquifer_barrier_count.txt`（aquifer_barrier_probe，2026-08-29），承接 perf-e2e-errors.md P1-P4。
> 本文档为草稿（subagent 产出），由主会话应用进 07 篇。

---

### 无污染 aquifer 内部精确定位（diag 方法，2026-08-29）——修正早前污染态构成

> ⚠️ **修正**：下表取代下上一小节「aquifer 内部 profile（4 chunks）」的污染态读数。早前「calculate_density 52% = barrier.sample 无 Cache2D」是**污染/粗糙**归因（把 barrier.sample + fluid/提前返回混记），已被计数类硬证据推翻（见 perf-e2e-errors.md P4）；该小节历史读数保留但**勿再引用为当前构成**。

| aquifer 内部部分 | 耗时/chunk | 占比/备注 |
|---|---|---|
| get_fluid_level（含 estimate_surface_height） | 3.84ms | 22% |
| get_block_pos（3×3 邻域 18 次/点） | 2.57ms | 14% |
| calculate_density（fluid 逻辑） | ~0ms | **barrier.sample 仅 0.1%（346/393216），走提前返回几乎不触发** |
| get_water_level_at | ~0.9ms（污染态） | 小 |
| **合计可解释** | **~6.4ms** | — |
| **剩余 ~11ms（未解释）** | = apply 每点 98304 次调用的**固定开销**（函数调用 + 3×3 距离计算 + 分支 + 数组访问） |

- **barrier.sample 几乎为 0（0.1%，346/393216 点）**——「barrier 加 Cache2D 缓存」方向已被实测推翻（错误方向，见 perf-e2e-errors.md P4）。
- **aquifer 慢的根本** = apply **每点 98304 次**调用的累积成本（~11ms 固定开销）+ get_fluid_level/get_block_pos（~36%），**不是 barrier 采样**。

### 根本洞察：Java 宏观网格采样 vs Rust 逐点采样（~80× 差）

- **Java**：宏观用 Interpolated 网格缓存（~1225 网格交点 + 三线性插值），aquifer/density 采样次数大幅减少。
- **Rust**：逐点采样（98304 点/ chunk）——采样次数比 Java 多 **~80×**（1225 vs 98304），这是 **Java 8-9ms vs Rust 44.9ms 慢 5 倍的根本**。
- 早前「外层网格采样探针」实测 2000ms 是**探针实现缺陷**（走 internal interpolated 雪崩重建，P1 教训），**不是方向错误**；正确对齐 Java 网格架构（避免雪崩）是正解方向。

### 优化方向（candidate，修正）

1. **（修正，替代原「barrier 加 Cache2D」）fill_chunk 宏观采样对齐 Java Interpolated 网格架构**（~1225 网格点 + 三线性插值，降采样次数约 80×）——**直接消除 apply 每点 98304 次的固定开销**（~11ms 未解释大头 + get_block_pos/get_fluid_level 的每点成本）。**需正确实现避免跨 chunk 雪崩重建**（P1 教训），且对齐 MC「本就该插值」的语义（单层插值是固有精度产物，非不可用）。
2. **density**：单层 Interpolated 对 SplineDF 实测加速 70×（judge 已验证），是密度优化正解；需单层生产化验证（保留）。
3. **carver / surface**：相对小头，后置（保留）。

### 域/边界

> 本节「aquifer 内部精确构成」为 **Partial** 快照，来自无污染计数/diag 探针；「~11ms apply 固定开销」为减法估计（17.5 - 6.4），随优化变化。端到端必须用充分预热的 Java 基准。
