# Cache2D LRU 打穿：跨 chunk 逐列采样 83× 退化（WorldgenRust 性能课题，2026-08-28）

> 本文件是 WorldgenRust（Rust worldgen 重写）性能课题的错误台账草稿，按五段式（现象/根因/定位/修复/教训）记录。
> 结论已实测验证（ordercmp / fillbench 探针 + git 提交 0059894 / 9adbc90 / f7c8663）。
> 载体：`.investigations/perf-rework/`（过程性中间产物，主会话可写；本文件为 subagent 产出草稿，待主会话应用/验证）。
> 状态：**draft**（未过 judge，未 confirmed）。

---

## 一、主错误：Cache2D 16 槽 LRU 被跨 chunk 逐列访问打穿

### 现象（具体数据）
- **ordercmp 探针**（纯树采样，`WorldgenRust/src/bin/ordercmp.rs`）：
  - 单 chunk(0,0) 逐列：**0.05 μs/pt**
  - 8 chunk(0,0)-(3,3) 逐列：**4.16 μs/pt** → **慢 83×**
- **fillbench 探针**（走 `fill_chunk` 完整管线，`WorldgenRust/src/bin/fillbench.rs`）：
  - 16 chunk per-chunk：首次冷启动 **236.67 ms** → 预热后 **23.55 ms**（冷启动慢 ~10×）
- 同一棵树、同一 chunk，仅遍历顺序/范围不同，per-sample 成本差 83×——**纯缓存命中率问题，不是算法复杂度问题**。

### 根因（机制层面）
- `Cache2DDF`（`density.rs`）是 **16 槽 LRU**（`CACHE2D_CAP = 16`），key 是 `(x,z)` 笛卡尔坐标（`((x as u32) as u64) << 32 ^ (z as u32 as u64)`）。
- 8 chunk 逐列访问 **2048 个不同 (x,z) 坐标**（8 chunk × 16×16 列），全部塞进 16 槽 LRU → **每列都 miss** → 每次 miss 重新采样深子树 `arg`（`self.arg.sample(pos)`）。
- 深子树 `arg` 是 InterpolatedDF/SplineDF 深层嵌套，单次采样成本极高 → 16 槽 LRU 形同虚设，退化为「每点全量重算」。
- **计数器掩盖**：`GRID_ARG_SAMPLES` 计数器（`density.rs` L16/L268）**只统计 Interpolated 的 `build_grid` 采样**，**不统计 Cache2D 的 `arg` 采样**。所以「42875 次网格构建正常」这个指标掩盖了 Cache2D 的级联重算——Cache2D 的 miss 重采样深子树 arg 时，arg 内部 Interpolated 的 build_grid 会重新触发（计数器会涨），但 Cache2D 自身 miss 的「每列重采样」次数没有独立计数，无法从该指标看出 Cache2D 在打穿。

### 定位（诊断方法/工具）
- **ordercmp 探针**（决定性实验）：同一棵树、同一 chunk，对比「模式A y-外层（perf_probe5 式）」vs「模式B 逐列（fill 式）」vs「模式C 8 chunk 逐列」的 per-sample 成本 → 模式C 4.16μs/pt vs 模式A/B 0.05μs/pt，锁定「多 chunk 逐列」是退化触发条件。
- **GRID_ARG_SAMPLES 计数对比**：ordercmp 打印模式C 的 grid arg samples delta，与单 chunk 对比——发现计数器无法解释 83× 退化（delta 正常），反证「有别的缓存（Cache2D）在打穿，计数器没覆盖」。
- **代码审查**：读 `Cache2DData::sample`（L342-355）确认 16 槽 LRU + 每 miss 重采样 `self.arg.sample(pos)`；读 `GRID_ARG_SAMPLES` 计数位置（L268，只在 `build_grid` 内）确认只统计 Interpolated。

### 修复（改了什么 + 为什么能修）
- **`CACHE2D_CAP` 16 → 256**（`density.rs` L318），并同步改 `Cache2DSlot` 的 `keys/values/stamps` 三个数组长度（L320）——LRU 槽位从 16 扩到 256，8 chunk 的 2048 坐标中，同一 chunk 内 256 列可全部驻留，跨 chunk 访问不再每列 miss。
- 顺带把缓存 backing 从 HashMap 改为 Vec-by-id（`C2D_CACHE` 用 `Vec<Option<Box<RefCell<Cache2DSlot>>>>` 直接下标），消除每 sample 的 hash/entry 开销。
- **效果**：ordercmp 8 chunk 从 **4.16 μs/pt → 0.13 μs/pt（快 32×）**，几乎追平单 chunk 0.06 μs/pt（剩 2.2× 差距）。
- 提交：`0059894 perf(density): raise Cache2D LRU capacity to 256 to fix cross-chunk thrash`。

### 教训（可复用判错经验）
1. **缓存容量/LRU 槽位不足是跨 chunk 性能退化的常见根因**——先查缓存容量再怀疑算法。跨 chunk 逐列访问会引入大量不同 key，小容量 LRU 必然打穿；「同一 chunk 内正常、跨 chunk 慢 83×」是缓存容量不足的典型指纹。
2. **计数器指标可能只统计部分缓存节点**——`GRID_ARG_SAMPLES` 只统计 Interpolated 的 build_grid，不统计 Cache2D 的 arg 采样，会掩盖其他缓存（Cache2D）的抖动。**「指标正常」≠「无退化」**，指标覆盖范围必须与怀疑的缓存节点对齐。
3. **LRU 容量与 key 空间要匹配**：key 是 (x,z) 笛卡尔坐标，跨 chunk 时 key 空间 = chunk 数 × 每 chunk 列数；容量必须 ≥ 单次工作集（单 chunk 列数），否则退化为每点全量重算。

---

## 二、反模式：SteelMC 式「chunk 级预填 grid」对 fillbench 无显著收益（已回退）

### 现象（具体数据）
- 尝试给 `DensityFunction` 加 `prefill_chunk` 递归遍历，`fill_chunk` 开头预填所有缓存节点（Interpolated/Cache2D/FlatCache）的 grid。
- **fillbench 实测**：prefill 启用 **23.55 ms** vs 禁用 **24.74 ms**（per-chunk，预热后）——**几乎一样，无显著收益**。
- 该改动已回退：`f7c8663 Revert "perf(density): add chunk-level grid prefill to eliminate cross-chunk cache thrash"`（revert `9adbc90`）。

### 根因（为什么无收益）
- `fill_chunk` 是**逐 chunk 调用**，单 chunk 内 Interpolated/Cache2D 的**懒建缓存本来就命中**（key 不变，同一 chunk 内坐标 key 稳定）。
- prefill 只是把「采样时懒建」提前到「chunk 开头」，**对单 chunk 内命中率无提升**——懒建和预填在单 chunk 内命中率相同，只是把工作提前，不减少总工作量。
- 真正的跨 chunk 问题（ordercmp 那种一个循环扫 8 chunk）**已被 `CACHE2D_CAP=256` 修复**——prefill 想解决的正是这个已被容量修复解决的问题，属于「用错工具打已死的靶」。

### 定位（诊断方法/工具）
- **fillbench A/B 对比**：prefill 启用 vs 禁用，per-chunk 耗时几乎相同（23.55 vs 24.74 ms）→ 直接证伪「prefill 有收益」。
- **与 ordercmp 结论对照**：ordercmp 已证明跨 chunk 退化根因是 Cache2D 容量不足（容量修复后 32× 提速），prefill 不改变单 chunk 内命中率 → 逻辑上必然无收益。

### 修复（改了什么）
- **回退**：`git revert 9adbc90`（`f7c8663`），删除 `prefill_chunk` 递归遍历 + `fill_chunk` 预填逻辑（density.rs 78 行 + terrain.rs 84 行删除）。
- 保留 `CACHE2D_CAP=256` 修复（`0059894`）——那是真正解决跨 chunk 退化的改动。

### 教训（反模式沉淀）
1. **「chunk 级预填」只在「跨 chunk 共享/复用」场景有意义**——预填的价值是把「跨 chunk 重复计算」提前/去重；对「逐 chunk 独立处理」（fill_chunk 逐 chunk 调用）无收益，因为单 chunk 内懒建缓存本来就命中。
2. **预填不改变单 chunk 内懒建缓存的命中率**——懒建和预填在单 chunk 内命中率相同，只是把工作提前，不减少总工作量。**先确认「要优化的场景是否真的跨 chunk 共享」再决定是否预填**。
3. **先量化根因再选优化手段**：ordercmp 已定位跨 chunk 退化 = Cache2D 容量不足，正确手段是扩容量（32× 收益）；prefill 是「没先量化就试的优化」，实测无收益后回退——**优化前先确认瓶颈是「容量」还是「预填时机」**。

---

## 三、错误 → 根因 速查表（一页索引）

| 错误/现象 | 一句话根因 |
|---|---|
| ordercmp 8 chunk 逐列 4.16μs/pt vs 单 chunk 0.05μs/pt（慢 83×） | Cache2D 16 槽 LRU 被 2048 个不同 (x,z) key 打穿，每列 miss 重采样深子树 arg |
| fillbench 16 chunk 冷启动 236.67ms vs 预热 23.55ms | 冷启动缓存全空，逐 chunk 首次全量重算；预热后单 chunk 内懒建缓存命中 |
| 「42875 次网格构建正常」但跨 chunk 仍慢 | `GRID_ARG_SAMPLES` 只统计 Interpolated 的 build_grid，不统计 Cache2D 的 arg 采样——指标掩盖 Cache2D 打穿 |
| SteelMC 式 chunk 级预填 grid 对 fillbench 无收益（23.55 vs 24.74ms） | fill_chunk 逐 chunk 调用，单 chunk 内懒建缓存本来就命中；prefill 只提前工作不提高命中率；跨 chunk 问题已被 CACHE2D_CAP=256 修复 |
| 跨 chunk 逐列慢的通用指纹 | 缓存容量/LRU 槽位 < 单次工作集 key 数 → 先查缓存容量再怀疑算法 |

---

## 附：相关提交

| commit | 内容 |
|---|---|
| `0059894` | `perf(density): raise Cache2D LRU capacity to 256 to fix cross-chunk thrash`（修复，保留） |
| `9adbc90` | `perf(density): add chunk-level grid prefill to eliminate cross-chunk cache thrash`（反模式，已回退） |
| `f7c8663` | `Revert "perf(density): add chunk-level grid prefill..."`（回退 9adbc90） |
