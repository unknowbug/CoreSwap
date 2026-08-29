# 草稿：10 时间线「aquifer 内部精确定位」补充条目文本

> 载体：`versions/1.20.1/docs/10-timewise-archive.md` 2026-08-29 记录补充（在既有「2026-08-29 Rust worldgen 端到端性能定位」条目后追加一个 sub-point）。
> 数据源：`cmd-output/aquifer_internal_precise.txt`、`cmd-output/aquifer_barrier_count.txt`（aquifer_barrier_probe，2026-08-29），承接 perf-e2e-errors.md P4。
> 本文档为草稿，由主会话应用进 10 时间线。

---

#### ✅ 五-b、aquifer 内部精确无污染定位（2026-08-29，修正污染态构成）

- **barrier.sample 实测仅 0.1%**（346/393216 = 每 chunk ~86 次）——「barrier 是 aquifer 大头 / 加 Cache2D 缓存」方向被计数类硬证据**推翻**（错误方向，见 perf-e2e-errors.md P4）。
- **无污染精确构成**（diag 方法，非热路径 instrument）：get_fluid_level（含 estimate_surface_height）3.84ms（22%）+ get_block_pos（3×3 邻域 18 次/点）2.57ms（14%）+ get_water_level_at 小 + calculate_density fluid ~0ms → **合计可解释 ~6.4ms，剩余 ~11ms 未解释 = apply 每点 98304 次调用的固定开销**。
- **修正早前污染态**：07 篇「calculate_density 52% = barrier.sample」为粗糙/污染归因（把 barrier.sample + fluid/提前返回混记），已被取代。
- **根本洞察**：Java 宏观用 Interpolated 网格（~1225 交点 + 三线性插值）vs Rust 逐点 98304 次 → **采样差 ~80×**，是 Java 8-9ms vs Rust 44.9ms 慢 5 倍的根本。
- **优化方向（candidate）**：fill_chunk 宏观采样对齐 Java Interpolated 网格架构（降采样次数 ~80×，消除 apply 每点固定开销）；需正确实现避免跨 chunk 雪崩（P1 教训）。早前外层网格探针 2000ms 是探针实现缺陷，非方向错误。

#### 📌 记录指引（更新）
- **错误台账（新增 P4）**：`.investigations/perf-e2e/perf-e2e-errors.md` P4——「barrier 加 Cache2D 方向被实测推翻」五段式 + 速查表一行（计数类硬证据定位、先量化再动手教训）。
- 结论：07 篇「Rust worldgen 端到端性能定位」小节补充「无污染 aquifer 内部精确构成」+ 优化方向修正（fill_chunk 宏观采样对齐 Java Interpolated 网格）。
- 域边界：精确构成 = Partial 快照；~11ms apply 固定开销为减法估计；填「fill_chunk 宏观采样对齐」方向 = candidate 待立项验证。
