# review-001 — DFC 直排方向关闭结论审查（judge）

> 角色：core.judge（subagent，隔离审查）
> 日期：2026-08-28
> 审查对象：本 session「DFC 直排对 Rust 无意义，方向关闭」结论（commit `e3c4a09`）
> 触发点：重大方向决策（judge 强制触发点）
> 三源核对：① git 提交 ② 代码/探针 ③ 验证记录
> 状态：**只出审查意见，不改 status、不改代码**

---

## 一、三源核对结果

### 1. git 提交

| 提交 | 内容 | 核对 |
|---|---|---|
| `e3c4a09` | perf_quant（量化 density vs 管线成本） | ✅ 代码在，但**无输出记录**（见 §二.1） |
| `0059894` | Cache2D LRU 16→256 | ✅ 已提交，文档化 8 chunk 83×→2.2× |
| `f7c8663` | chunk 级预填回退 | ✅ 已提交，文档化 23.55 vs 24.74ms 无收益 |
| **（缺失）** | **Rust 方案 B（spline 显式栈）实验** | ❌ **无提交、无回退提交、无代码痕迹** |

**关键缺失**：背景声称「方案 B（spline 显式栈）fillbench 36.70ms vs 递归 34.68ms（噪声范围，无收益，已回退）」。但：
- `git log --all -- WorldgenRust/src/density.rs` 只有 6 个提交，**无任何 spline 显式栈实验提交**；
- `git diff WorldgenRust/src/density.rs` 工作区与 HEAD 一致（当前 density.rs 是**递归版** `SplineData::sample_node`）；
- **无回退提交**（`f7c8663` 回退的是 chunk 预填，不是 spline 显式栈）；
- 数字 `36.70`/`34.68` **在仓库任何文件都不存在**（grep 全仓 0 命中）。

`dfc-nosplit-explicit-stack.md` 是 **C++** 实验（WG_DFC_NOSPLIT，conc_density_probe），非 Rust fillbench。C++ 侧 NOSPLIT 数据是 34.91→345.78 = 9.9×（时间线 L1800），**不是** 36.70/34.68。

### 2. 代码/探针

- `perf_quant.rs`：纯树测试用**单 chunk (0,0) 逐列**（缓存命中）；fill_chunk 用 **8 chunk**。两测试 chunk 数不同（见 §二.1）。
- `density.rs`：`SplineData::sample_node`（L89-125）是**真递归**（自调用 + `loc_fns` Arc<DensityFunction> enum 分派）。编译器**无法**消除数据依赖递归。
- `fillbench.rs`：注释明确「这是 Spline 直排优化前的基线」——**无方案 B 版本**。

### 3. 验证记录

- perf_quant 输出数字（0.05/0.34μs/pt）**只在 commit message**，无 cmd-output 落盘。
- 方案 B 数字（36.70/34.68）**仓库内不存在**。
- C++ 侧 2026-08-23 已**证伪**「虚调用是 11× 元凶」（见 §二.3）。

---

## 二、逐项审查意见

### 审查点 1：量化结论是否成立 —— ❌ 有问题（测量方法缺陷）

**perf_quant 纯树 0.05μs/pt 不可信为「真实每点成本」**：

- 纯树测试用**单 chunk (0,0) 逐列**，finalDensity 含 Interpolated 节点。首列建 grid，后续 98304 点全命中缓存 grid（trilinear）。**grid 构建成本被摊薄进 0.05μs/pt**。
- commit `4635170` 明确文档化：**「bottleneck = Interpolated grid build (5 nested grids, 420ms/chunk), steady-state 0.07us/pt」**，并**明确警告**：
  > "caution: earlier ~237us/pt (mt_probe) was misleading (amortized grid build into per-point); perf_probe v1 0.1us/pt unreliable (same fixed point const-folded). Use varied points + fresh-vs-cached split."
- **perf_quant 的纯树测试正是踩了这个已文档化的坑**：单 chunk 缓存命中 + grid 构建摊薄。0.05μs/pt 是「稳态缓存命中」成本，**不是**真实每点 density 成本（真实成本含 grid 构建 420ms/chunk ≈ 4.27μs/pt，比 0.05 高 85×）。

**6.4×「管线开销」不干净**：纯树（单 chunk，grid 摊薄）vs fill_chunk（8 chunk，每 chunk 建 grid + 跨 chunk 缓存效应）。6.4× 混入了「8 chunk 的 grid 构建成本 + 跨 chunk 缓存 miss」，**不是**纯管线（aquifer/biome/surface）开销。commit `0059894` 已证明 8 chunk 有跨 chunk 缓存效应（83×→2.2×）。

**0.05μs/pt 是否「极快」**：C++ production density 单点 0.4μs（时间线 L1963）。Rust 纯树 0.05μs 是稳态缓存命中，**不可与 C++ production 直接比**（C++ 含完整树遍历）。若含 grid 构建，Rust 真实 density 成本远高于 0.05。

### 审查点 2：DFC 直排方向关闭是否合理 —— ❌ 有问题（关键证据缺失）

**方案 B 无收益的结论无证据支撑**：
- 方案 B（fillbench 36.70 vs 34.68）**无提交、无回退、无输出记录、数字仓库内不存在**。此实验**无法核实**。
- 唯一可核实的 Rust 相关实验是 `f7c8663`（chunk 预填 23.55 vs 24.74ms 无收益）——但那是**缓存预填**，不是 spline 显式栈。
- C++ 侧 NOSPLIT（spline 递归→显式栈）确实无收益（9.9× vs 10.38×，时间线 L1800）——但这是 **C++** 数据，不能直接外推到 Rust（Rust 是 enum-match 非虚调用，机制不同）。

**「DFC 对 Rust 无意义」的结论建立在不可核实的方案 B 上**，证据链不完整。

### 审查点 3：C++ 热点不转移的结论 —— ❌ 有问题（前提已证伪）

**「C++ 的 SplineDF 虚调用热点（11×）」是过时/错误前提**：
- C++ 侧 2026-08-23 已**证伪**「虚调用是 11× 元凶」：
  - DEVIRT（去 spline.locFn 虚分派）：10.05× vs 10.32×，降 2.6% 噪音内（时间线 L1816）；
  - 权威 JSON 证实所有 spline coordinate 均为纯噪声 DF，**无一嵌套 SplineDF**（时间线 L1657）；
  - 11× 归因 = wrapper 链争用 + latency QoS（时间线 L1848/L368），**非虚调用**。
- perf_quant 结论「Confirms the C++ virtual-call hotspot does not transfer to Rust」**基于已证伪的前提**。

**Rust enum-match 递归是否被编译器优化**：`SplineData::sample_node` 是**真递归**（数据依赖深度），编译器**无法**消除。但 vanilla spline coordinate 是纯噪声（C++ 已证伪嵌套 spline），所以 Rust 实际递归深度浅——**「递归指数级膨胀」担忧（scout 报告 L15）在 vanilla 下被高估**。scout 报告基于 C++ 已证伪的「嵌套 SplineDF」前提。

**Rust sample 在更大规模/多线程下是否仍慢**：**未测**。perf_quant 是单线程稳态测量，未覆盖多线程/冷缓存/大规模。C++ 的教训是「单点快 ≠ 并发快」（11× 是并发现象）。Rust 的并发争用**完全未量化**。

### 审查点 4：管线瓶颈判断 —— 存疑（需进一步量化）

- 6.4× 开销归因于 aquifer/biome/surface **方向可能对**（fill_chunk 确实含这些），但**测量不干净**（混入 grid 构建 + 跨 chunk 缓存效应）。
- **未量化哪个组件占大头**。fillprofile.rs 存在（可分解 density+aquifer vs biome），但**无输出记录**。
- 结论「瓶颈在管线非 density」**方向可信但证据不足**——需要 fresh-vs-cached 分离（perf_probe2 方法）才能干净归因。

### 审查点 5：风险 —— 有问题（方向关闭过早 + 遗漏路径）

**DFC 方向关闭过早**：
- 关键证据（方案 B）不可核实；
- 纯树测量方法有缺陷（grid 构建摊薄）；
- 多线程/冷缓存/大规模未测；
- C++ 侧已证伪虚调用前提，perf_quant 结论基于错误前提。

**遗漏的优化路径**：
- **多线程 fill**：C++ 教训是「单点快 ≠ 并发快」，Rust 并发争用完全未量化——这是 C++ 11× 课题的核心，Rust 侧**未做**；
- **grid 构建优化**：commit `4635170` 明确 grid build（420ms/chunk）是主导瓶颈，perf_quant 却把它摊薄掉了——**grid 构建才是 density 侧真瓶颈**，DFC 直排不解决它；
- **SIMD**：未评估；
- **缓存优化**：Cache2D LRU 已修（0059894），但 Interpolated grid 构建成本未优化。

---

## 三、产物契约核对

- ❌ **无 `.artifacts/dfc-direct-port/` 目录**，无 index.yaml 条目。「DFC 方向关闭」结论**未落盘为 artifact**。
- ✅ scout 报告在 `.investigations/dfc-direct-port/dfc-direct-scout-report.md`（draft）。
- ❌ perf_quant 输出无 cmd-output 落盘（数字只在 commit message）。
- ❌ 方案 B 实验无任何记录。

---

## 四、结论与建议

### 结论：DFC 方向关闭**不合理（证据不足，暂缓关闭）**

「DFC 直排对 Rust 无意义」的结论**不能成立**，因为：
1. 核心证据（方案 B 无收益）**不可核实**（无提交/无回退/无输出/数字不存在）；
2. 纯树测量方法**有缺陷**（grid 构建摊薄，踩了 4635170 已文档化的坑）；
3. 结论前提（C++ 虚调用热点）**已被 C++ 侧证伪**；
4. 多线程/冷缓存/大规模**未测**。

### 确认等级建议

- **保持 draft**（不升 candidate，更不升 confirmed）。
- 结论性 docs（时间线/主题篇）**不得写入**，直到证据链补齐。

### 必须修复 / 需补验证 / 可接受

**必须修复（证据链）**：
1. **补方案 B 实验记录**：若方案 B 真跑过，补提交/输出落盘；若没跑过，**不得声称「已回退」**——这是证据造假风险。
2. **补 perf_quant 输出落盘**：cmd-output 记录 0.05/0.34 数字 + 运行环境。
3. **补 `.artifacts/dfc-direct-port/` artifact + index.yaml**：结论必须落盘。

**需补验证（测量方法）**：
4. **用 fresh-vs-cached 分离**（perf_probe2 方法）重测纯树：区分 grid 构建成本 vs 稳态采样成本——0.05μs/pt 是稳态，真实 density 成本含 grid 构建。
5. **同 chunk 数对比**：纯树 vs fill_chunk 用**相同 chunk 数**（如都 8 chunk），才能干净归因管线开销。
6. **量化管线组件**：跑 fillprofile（density+aquifer vs biome）并落盘，确认哪个组件占大头。
7. **测多线程/冷缓存**：Rust 并发争用未量化——这是 C++ 11× 课题的核心，Rust 侧必须补。

**可接受**：
8. 「Rust 单点稳态 density 快」方向可信（0.05μs 稳态 vs C++ 0.4μs production），但**不能**据此关闭 DFC 方向。
9. 「管线（aquifer/biome/surface）是 fill_chunk 大头」方向可信，但需干净测量确认。

---

## 五、审查意见摘要

| 审查点 | 判定 | 理由 |
|---|---|---|
| 1. 量化结论 | ❌ 有问题 | 纯树单 chunk 缓存命中 + grid 构建摊薄，踩 4635170 已文档化的坑；6.4× 不干净 |
| 2. DFC 方向关闭 | ❌ 有问题 | 方案 B 无证据（无提交/无回退/无输出/数字不存在） |
| 3. C++ 热点不转移 | ❌ 有问题 | 前提（C++ 虚调用热点）已被 C++ 侧证伪；Rust 并发未测 |
| 4. 管线瓶颈 | 存疑 | 方向可信但测量不干净，组件未量化 |
| 5. 风险 | 有问题 | 关闭过早；遗漏多线程/grid 构建/SIMD 路径 |

**推荐状态**：保持 draft。**不授予 candidate**。**不写结论性 docs**。

> 本意见为建议，非命令。最终拍板权在宿主人类。
