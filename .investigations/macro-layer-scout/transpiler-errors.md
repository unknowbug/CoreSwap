# build-time transpiler 性能定位：错误与根因清单（重点记录）

> 载体：`.investigations/macro-layer-scout/transpiler-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「build-time transpiler 性能定位」里程碑（2026-08-29）中定位并处理/发现错误的条目。本 session 共 6 类（M1-M6）。
> 背景：build-time transpiler（把 density 树编译成 native 代码，避免运行时 enum match 解释）性能未达——transpiler cell grid 构建 41.79ms/chunk vs 运行时 Interpolated grid 构建 8.14ms/chunk（慢 5 倍）；transpiler 单次 fill 7μs vs 运行时热采样 171ns（慢 40 倍）。诊断链见 `cmd-output/transpiler_*.txt`（transpiler_perf → perf2 → fill_cost → grid_compare → single_compare → grid_calls）。
> 本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。数字来自 `cmd-output/` 实测记录。

---

## M1. transpiler cell grid 构建比运行时 Interpolated grid 构建慢 5 倍（性能未达主因）

### 现象
- transpiler cell grid 构建（`fill_cell_corner_densities` 采样 5 channels 完整树）：**41.79ms/chunk**（`transpiler_grid_calls.txt`）。
- 运行时 Interpolated grid 构建：**8.14ms/chunk**（冷 14.03 - 热 5.88，`transpiler_grid_compare.txt`）。
- 慢 **5 倍**。早期 `transpiler_perf.txt` 测 43.09ms vs 6.11ms（慢 7 倍，口径略异）。

### 根因（机制）
- **transpiler 的 `fill_cell_corner_densities` 每 corner 采样 5 channels 完整树，无缓存（缓存冷）**——每个 corner 都从根重新采样整棵 density 树。
- **运行时 Interpolated 有缓存复用**（noise 采样等中间结果在 cell 内复用），单次 grid 构建成本被摊薄。
- 即：transpiler 的 build-time 编译优势（消除 enum match）**被「无缓存」抵消**——编译成 specialized 函数但每点采样完整树（无缓存复用），比运行时（有缓存）慢。

### 定位（诊断方法）
- **逐层下钻的测量链**（`cmd-output/` 系列）：
  1. `transpiler_perf.txt`：transpiler 43ms vs 运行时 6.11ms（慢 7 倍）→ 定位到 cell grid 构建。
  2. `transpiler_perf2.txt`：NoiseSet 数组优化后 noise 采样 1.02x（排除 noise 查表）→ 主因锁定 cell grid 构建。
  3. `transpiler_grid_compare.txt`：transpiler cell grid 构建 43ms vs 运行时 Interpolated grid 构建 8.14ms（慢 5 倍）→ 确认差异在 grid 构建本身。
  4. `transpiler_grid_calls.txt`：corners=1225（与运行时相同采样量）→ 差异不是采样量，是单次 fill 成本。
- 关键判据：**「采样量相同（1225 corners）但单次成本不同」→ 差异在单次 fill 的缓存状态，不在采样量**。

### 修复
- **方向（未落地，待实现）**：transpiler 加缓存——noise 采样复用 / cell grid 复用，对齐运行时 Interpolated 缓存机制。
- 具体：`fill_cell_corner_densities` 内对共享子树（noise 采样、flat-cached 值）做 cell 级缓存，避免每 corner 从根重采完整树。

### 教训（可复用判错经验）
- **build-time 编译（消除 enum match）优势会被「无缓存」抵消**：编译成 specialized 函数 ≠ 快——若每点采样完整树（无缓存复用），可能比运行时（有缓存）更慢。**编译优化与缓存优化是正交的两条线，缺一不可**。
- **性能定位先排除「采样量」再查「单次成本」**：grid 构建慢，先确认 corners 数是否与参考一致（1225=1225），一致则差异必在单次 fill 成本（缓存状态），不在采样量。

---

## M2. transpiler 单次 fill 比运行时热采样慢 40 倍（无缓存 vs 有缓存）

### 现象
- transpiler `fill_cell_corner_densities` 单次（5 channels 完整树）：**7μs**（`transpiler_single_compare.txt`）。
- 运行时 `final_density.sample` 单次（热，缓存命中）：**171ns**。
- 慢 **40 倍**。

### 根因（机制）
- transpiler 单次 fill 采样 **5 channels 完整树**（每 corner 从根重采，无缓存）。
- 运行时热采样是**缓存命中**（Interpolated 每 chunk 建 grid 一次 + 块级插值，块内采样复用 grid 结果）。
- 即：**「单次采样成本」对比的是两种完全不同的执行形态**——transpiler 是「每点重采完整树」，运行时是「grid 建一次 + 块内插值复用」。

### 定位（诊断方法）
- `transpiler_single_compare.txt`：分别测 transpiler 单次 fill 与运行时热采样，直接对比单次成本 → 40 倍差距。
- 结合 `transpiler_grid_calls.txt`：cell grid 构建内单次 fill 实际 **34μs**（缓存冷），比 `transpiler_fill_cost` 测的 7μs（缓存热）还高——见 M3。

### 修复
- 同 M1：transpiler 加缓存（noise 采样复用 / cell grid 复用），对齐运行时 Interpolated 缓存机制。

### 教训（可复用判错经验）
- **「单次采样成本」对比必须标注缓存状态**：transpiler 单次 fill 7μs 是「缓存热」（连续调用），运行时热采样 171ns 是「缓存命中」——两者都是热态，但 transpiler 的热态仍比运行时慢 40 倍，说明 transpiler 的「热」没有运行时那种「grid 建一次 + 块内复用」的结构性缓存。
- **对比单次成本前先确认对比的是同一执行形态**：transpiler「每点重采完整树」vs 运行时「grid 建一次 + 块内插值」是两种形态，直接比单次数字会得出「transpiler 慢 40 倍」但掩盖「形态不同」这一根因。

---

## M3. 缓存热 vs 缓存冷混淆：`transpiler_fill_cost` 测的 7μs 是缓存热，cell grid 构建内是缓存冷 34μs

### 现象
- `transpiler_fill_cost.txt`：单次 fill 调用 **6954ns（~7μs）**，并据此估算「1225 corners × 7μs ≈ 8.5ms」——但实测 cell grid 构建是 **43ms**（`transpiler_perf.txt`），估算与实测差 5 倍。
- `transpiler_grid_calls.txt`：cell grid 构建内单次 fill 实际 **34μs**（缓存冷），是 `transpiler_fill_cost` 测的 7μs（缓存热）的 **~5 倍**。

### 根因（机制）
- **测量场景不同导致缓存状态不同**：
  - `transpiler_fill_cost` 测的是**连续调用**（同一 cell 内相邻 corner，中间结果缓存热）→ 7μs。
  - cell grid 构建是**不同 corner**（每 corner 采样 5 channels 完整树，缓存冷）→ 34μs。
- 用「缓存热的单次成本」× corners 数估算「缓存冷的 grid 构建总成本」→ **系统性低估**（7μs vs 34μs，差 5 倍，正好解释 8.5ms 估算 vs 43ms 实测）。

### 定位（诊断方法）
- **交叉核对估算与实测**：`transpiler_fill_cost` 的「1225 × 7μs ≈ 8.5ms」与 `transpiler_perf` 的实测 43ms 差 5 倍 → 触发怀疑「单次成本测错了场景」。
- `transpiler_grid_calls.txt` 在 cell grid 构建**内部**测单次 fill → 34μs（缓存冷），确认差异来自缓存状态。

### 修复
- **测量口径修正（必做）**：性能定位必须区分「缓存热 vs 缓存冷」——`transpiler_fill_cost` 的 7μs 是缓存热（连续调用），不能用于估算缓存冷的 grid 构建总成本。
- 方向（同 M1）：transpiler 加缓存，把「缓存冷」的 cell grid 构建变成「缓存热」的复用。

### 教训（可复用判错经验）
- **性能定位要区分「缓存热 vs 缓存冷」**：同一函数在不同调用场景（连续调用 vs 不同 corner）缓存状态不同，单次成本可差 5 倍。**用单次成本估算总成本前，先确认测量场景与目标场景的缓存状态一致**。
- **估算与实测差 5 倍是「测量场景错」的强信号**：8.5ms 估算 vs 43ms 实测，不是「更多 corners」就是「单次成本测错场景」——先查单次成本测量场景，再查 corners 数（M6 确认 corners 相同，故是单次成本场景错）。

---

## M4. noise 查表（HashMap）被误判为瓶颈，数组优化后证明不是

### 现象
- `transpiler_perf.txt` 曾列「NoiseSet 用 HashMap 查表（sample_noise 每次 HashMap.get）——比运行时直接字段访问慢」为性能未达原因之一。
- `transpiler_perf2.txt`：NoiseSet 数组优化后，noise 采样 **809 vs 793ns（1.02x，几乎相同）**——noise 查表**不是瓶颈**。

### 根因（机制）
- **归因错误**：把「HashMap 查表比字段访问慢」的直觉当成性能主因，但实测证明 noise 采样在总成本中占比可忽略（1.02x 无差异）。
- 真正主因是 cell grid 构建无缓存（M1/M3），noise 查表只是「看起来慢」的次要项。

### 定位（诊断方法）
- **对照实验**：NoiseSet 从 HashMap 优化为数组后，复测 noise 采样 → 1.02x 无差异 → 排除 noise 查表为瓶颈。
- 关键判据：**「优化后无差异」= 该项不是瓶颈**——若某项是瓶颈，优化它应有显著收益；无收益说明它不在关键路径。

### 修复
- **归因修正（必做）**：从「noise 查表慢」改为「noise 查表不是瓶颈（数组优化后 1.02x 无差异），主因是 cell grid 构建无缓存」。
- 数组优化本身保留（无害，且消除 HashMap 开销），但不作为性能修复的功劳。

### 教训（可复用判错经验）
- **「看起来慢」≠「是瓶颈」**：HashMap 查表直觉上比字段访问慢，但若它在总成本中占比可忽略，优化它无收益。**判断瓶颈用「优化后是否有显著收益」的对照实验，不用直觉**。
- **性能定位先做「排除法」再下结论**：把候选瓶颈逐个用对照实验排除（noise 查表 1.02x 排除），剩下的才是主因（cell grid 无缓存）。

---

## M5. build-time 编译优势（消除 enum match）被「无缓存」抵消——编译优化与缓存优化正交

### 现象
- transpiler 的 build-time 编译（把 density 树编译成 specialized native 函数，消除运行时 enum match 递归解释）**本应更快**，但实测 cell grid 构建 41.79ms vs 运行时 8.14ms（慢 5 倍，M1）。
- 即：**编译成 specialized 函数但每点采样完整树（无缓存复用）比运行时（有缓存）慢**。

### 根因（机制）
- **编译优化与缓存优化是两条正交的线**：
  - build-time 编译消除了「enum match 递归解释」的开销（这是编译的收益）。
  - 但 transpiler 的 `fill_cell_corner_densities` **每 corner 采样 5 channels 完整树，无缓存**（缓存冷）——这是「无缓存」的损失。
  - 运行时 Interpolated 有缓存复用（noise 采样等），把「每点重采」变成「grid 建一次 + 块内复用」。
- **净效果**：编译省下的 enum match 开销 < 无缓存带来的重采开销 → transpiler 反而慢。

### 定位（诊断方法）
- **诊断链收敛**（`transpiler_perf → perf2 → fill_cost → grid_compare → single_compare → grid_calls`）：
  1. transpiler 43ms vs 运行时 6.11ms（慢 7 倍）→ 性能未达。
  2. noise 查表排除（M4）→ 主因 cell grid 构建。
  3. 单次 fill 7μs（缓存热）vs 运行时热采样 171ns（慢 40 倍，M2）→ 单次成本差异。
  4. corners=1225 相同（M6）→ 差异不是采样量。
  5. 结论：差异是单次 fill 无缓存（缓存冷 34μs vs 缓存热 7μs，M3）。
- 关键判据：**「编译优势被无缓存抵消」是结论性判断**——编译省下的 enum match 开销不足以弥补无缓存的重采开销。

### 修复
- **方向（未落地）**：transpiler 加缓存（noise 采样复用 / cell grid 复用），对齐运行时 Interpolated 缓存机制——让编译优势真正兑现。
- 若缓存后仍慢，再评估「编译本身是否值得」（build-time 复杂度 vs 收益）。

### 教训（可复用判错经验）
- **「编译成 specialized 函数」≠「快」**：编译消除 enum match 只是消除一类开销；若引入「无缓存每点重采完整树」，可能比运行时（有缓存）更慢。**编译优化与缓存优化必须同时做，缺一不可**。
- **性能优化的收益要「端到端」验证**：不要因为「编译消除了 enum match」就预期快——用端到端对比（transpiler vs 运行时）验证，发现慢 5 倍再回头查缓存。

---

## M6. 采样量相同（1225 corners）但单次成本不同（缓存冷 34μs vs 热 7μs）——缓存是性能关键

### 现象
- transpiler cell grid 构建：**41.79ms/chunk, corners=1225**（`transpiler_grid_calls.txt`）。
- 运行时 Interpolated grid 构建：**8.14ms/chunk**。
- **两者都是 1225 corners（采样量相同）**，但 transpiler 慢 5 倍。

### 根因（机制）
- **差异不是采样量，是单次 fill 调用成本**：
  - transpiler 单次 fill **34μs**（cell grid 构建内，缓存冷）。
  - transpiler_fill_cost 测的 **7μs**（连续调用，缓存热）。
  - 运行时 Interpolated grid 构建 **6.6μs/corner**（有缓存复用）。
- 即：**采样量相同（1225）但单次成本不同（缓存冷 34μs vs 热 7μs）——缓存是性能关键**。

### 定位（诊断方法）
- `transpiler_grid_calls.txt`：在 cell grid 构建内统计 corners 数（fill 调用次数）= 1225，与运行时相同 → 排除「采样量更多」。
- 对比单次 fill 成本（缓存冷 34μs vs 缓存热 7μs vs 运行时 6.6μs）→ 确认差异在缓存状态。

### 修复
- 同 M1：transpiler 加缓存（noise 采样复用 / cell grid 复用），把单次 fill 从缓存冷 34μs 降到缓存热 7μs 甚至运行时 6.6μs 水平。

### 教训（可复用判错经验）
- **「采样量相同」是排除「采样量差异」的关键证据**：grid 构建慢，先确认 corners 数与参考一致（1225=1225），一致则差异必在单次成本（缓存状态），不在采样量。
- **缓存是性能关键**：同一采样量下，缓存冷（34μs）vs 缓存热（7μs）差 5 倍——**性能定位先查缓存状态，再查算法复杂度**。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| transpiler cell grid 构建 41.79ms vs 运行时 Interpolated grid 构建 8.14ms（慢 5 倍）（M1） | transpiler `fill_cell_corner_densities` 每 corner 采样 5 channels 完整树**无缓存（缓存冷）**；运行时 Interpolated 有缓存复用 | **编译优化与缓存优化正交**——编译消除 enum match ≠ 快，无缓存每点重采完整树可能比运行时（有缓存）慢。性能定位先排除采样量（1225=1225）再查单次成本 |
| transpiler 单次 fill 7μs vs 运行时热采样 171ns（慢 40 倍）（M2） | transpiler 单次 fill 采样 5 channels 完整树（无缓存）；运行时热采样是缓存命中（grid 建一次 + 块内插值复用） | **对比单次成本前先确认同一执行形态**：transpiler「每点重采完整树」vs 运行时「grid 建一次 + 块内复用」是两种形态，直接比单次数字掩盖形态差异 |
| `transpiler_fill_cost` 测 7μs 估算「1225×7μs≈8.5ms」但实测 43ms（差 5 倍）（M3） | **缓存热 vs 缓存冷混淆**：7μs 是连续调用（缓存热），cell grid 构建是不同 corner（缓存冷 34μs） | **性能定位区分缓存热/冷**：用单次成本估算总成本前先确认测量场景与目标场景缓存状态一致。估算与实测差 5 倍是「测量场景错」强信号 |
| NoiseSet HashMap 查表被列为性能主因，数组优化后 1.02x 无差异（M4） | **归因错误**：HashMap 查表直觉上慢，但 noise 采样在总成本占比可忽略，不是瓶颈；主因是 cell grid 无缓存 | **「看起来慢」≠「是瓶颈」**：判断瓶颈用「优化后是否有显著收益」的对照实验，不用直觉。先排除法再下结论 |
| build-time 编译（消除 enum match）本应更快，实测反而慢 5 倍（M5） | **编译优势被无缓存抵消**：编译省下的 enum match 开销 < 无缓存带来的重采开销 | **编译成 specialized 函数 ≠ 快**：编译优化与缓存优化必须同时做。性能收益要端到端验证（transpiler vs 运行时），发现慢再回头查缓存 |
| 采样量相同（1225 corners）但 transpiler 慢 5 倍（M6） | **差异不是采样量，是单次 fill 成本**：缓存冷 34μs vs 缓存热 7μs vs 运行时 6.6μs | **缓存是性能关键**：同一采样量下缓存冷/热差 5 倍。性能定位先查缓存状态，再查算法复杂度 |

> [DRAFT — knowledge subagent 产出，待主会话应用。] 主会话应用：保留本文件（错误台账独立成篇，符合 SUBAGENT-KNOWLEDGE-GUIDE §三），末尾速查表已含 M1-M6 各一行。修复方向（transpiler 加缓存）未落地，仅记录方向，不标 confirmed。
