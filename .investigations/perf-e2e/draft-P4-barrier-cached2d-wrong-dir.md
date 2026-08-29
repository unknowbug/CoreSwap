# 草稿：P4 错误台账条目（barrier 加 Cache2D 方向被实测推翻）

> 载体：`.investigations/perf-e2e/perf-e2e-errors.md` 追加 P4（由 knowledge subagent 产出草稿，主会话应用）。
> 数据源：`cmd-output/aquifer_barrier_count.txt`（`aquifer_barrier_probe`，2026-08-29）、`cmd-output/aquifer_internal_precise.txt`。
> 本文档为草稿，最终写入 perf-e2e-errors.md 由主会话合并，格式对齐既有 P1-P3 五段式。

---

## P4. 「barrier 是 aquifer 大头 → 加 Cache2D 缓存」方向被实测推翻（barrier.sample 仅 0.1%）

### 现象
- 依据早前 aquifer 内部 profile（污染态）得出「calculate_density 52% = barrier.sample 无 Cache2D 缓存」的结论，并据此定优化方向「**aquifer 的 barrier.sample 跨点加 Cache2D 缓存**」。
- **精确无污染计数（`aquifer_barrier_probe`，2026-08-29）**：`barrier.sample` 调用 **346 次 / 4 chunks / 393216 点 = 0.1%**（每 chunk 仅 ~86 次）——barrier 采样**几乎不发生**，「barrier 是 aquifer 大头」不成立。
- 无污染精确构成（`aquifer_internal_precise.txt`）：get_fluid_level（含 estimate_surface_height）3.84ms（22%）、get_block_pos（3×3 邻域 18 次/点）2.57ms（14%）、calculate_density fluid 逻辑 ~0ms、get_water_level_at 小——**合计可解释 ~6.4ms，aquifer 总 17.5ms 剩余 ~11ms 未解释 = apply 每点 98304 次调用的固定开销**（函数调用 + 3×3 距离计算 + 分支 + 数组访问）。

### 根因（机制）
- **早前「calculate_density 52%」是污染态读数**：未区分 calculate_density 内部的 barrier.sample 与 fluid/提前返回逻辑，把整个 calculate_density 记作「barrier 慢」。
- **calculate_density 大多走提前返回**（lava_water / j==0 → 0.0），barrier 采样（3D Noise 树遍历）几乎不触发——0.1% 采样率。给根本采不到几次的 barrier 加缓存，是**优化了错误的目标**。
- aquifer 真实大头是**每点 98304 次的 apply 固定调用开销**（函数调用 + 3×3 距离计算 + 分支 + 数组访问），而非 barrier 采样；get_fluid_level + get_block_pos 合计 ~36% 是可解释的次大头，其余 ~11ms 是 apply 逐点成本本身。

### 定位（诊断方法）
- **计数类硬证据**：`aquifer_barrier_probe` 精确统计 `barrier.sample` **调用次数**（346/393216 = 0.1%），用「调用次数」而非「某函数耗时占比」判断热点——barrier 走提前返回，耗时占比会被稀释/误导，次数才是直接证据。
- **无污染 diag 定位**（非热路径 instrument）：`aquifer_internal_precise.txt` 用 chunk 级/统计型 diag 拆分 aquifer 内部分量，规避了 P2「诊断代码热路径每点执行」污染，得到可解释 ~6.4ms + 剩余 ~11ms 的精确构成。

### 修复
- **撤销「barrier 加 Cache2D 缓存」方向**（barrier 采样本就 0.1%，缓存无意义）。
- aquifer 优化方向修正为：**fill_chunk 宏观采样对齐 Java Interpolated 网格架构（降采样次数，~1225 网格交点 + 三线性插值 vs Rust 逐点 98304）**——根本解决 apply 每点 98304 次的固定开销，而不是在 barrier 上加缓存。需正确实现 Interpolated 网格避免「跨 chunk 雪崩重建」（P1 教训）。
- 早前污染态 profile（calculate_density 52%）在 07 篇作为历史读数保留标注，被本次精确数据取代。

### 教训（可复用判错经验）
- **定位瓶颈要用「计数类硬证据」（采样次数 / 调用次数），不要凭「barrier 是 density 树」推断它是热点**——走提前返回的路径耗时占比高，但实际调用次数极少，加缓存优化的是错误目标。
- **优化方向先量化再动手**：动手加缓存前先数清目标函数实际被调用几次（346 次/39 万点 = 0.1%），量化能直接否定「加缓存」这类方向，避免优化到几乎不执行的热点。
- **注意污染态读数**：早前 profile 把 calculate_density 整段记为 barrier 开销，混淆了 barrier.sample 与 fluid/提前返回。精确拆分要看内部计数，不能把父函数耗时全部归因于子调用。

---

## 速查表（并入 perf-e2e-errors.md 末尾附「错误 → 根因」速查表，新增一行）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| 「barrier 加 Cache2D」方向被实测推翻（P4） | 早前 calculate_density 52% 是**污染态读数**（把 barrier.sample + fluid/提前返回混记）；实际 barrier.sample 仅 **346/393216 = 0.1%**（走提前返回几乎不触发）；aquifer 真实大头 = **apply 每点 98304 次固定调用开销 ~11ms + get_fluid_level/get_block_pos ~36%** | **定位用计数类硬证据（采样/调用次数），别凭「barrier 是 density 树」推断热点**；**先量化再动手优化**（0.1% 采样率直接否定「加缓存」方向）；注意污染态 profile 把父函数耗时误归因于子调用 |
