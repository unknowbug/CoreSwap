# review-002 — biome 性能优化（分桶+SearchTree）+ 多线程并发验证 + DFC 方向评估（judge）

> 角色：core.judge（subagent，隔离审查）
> 日期：2026-08-29
> 审查对象：本 session 后半段交付
>   - `c03e8ef` biome depth 分桶（扫描 54→30μs/pt）
>   - `f73c005` biome SearchTree（KD-tree，扫描 30→2.9μs/pt）
>   - `a46e9a9` mt_fill 多线程并发验证（4.6× 扩展）
>   - 前置：`0059894`（Cache2D LRU）、`a9ba91a`（MultiNoise 最近邻）、`e3c4a09`（perf_quant）
> 触发点：收尾交付（judge 强制触发点）+ 重大方向（DFC 关闭评估）
> 三源核对：① git 提交 ② 代码/探针 ③ 验证记录（cmd-output）
> 状态：**只出审查意见，不改 status、不改代码**

---

## 一、三源核对结果

### 1. git 提交

| 提交 | 内容 | 核对 |
|---|---|---|
| `c03e8ef` | biome depth 分桶（7593→~3800 行/桶） | ✅ 代码在，biome_scan_cost.txt 落盘（30.783μs/pt） |
| `f73c005` | biome SearchTree（KD-tree） | ✅ 代码在，但 **cmd-output 未更新**（见 §二.1） |
| `a46e9a9` | mt_fill 多线程验证 | ✅ 代码在，mt_fill.txt 落盘（T=1 790ms→T=8 170ms，mismatch=0/16） |
| `0059894` | Cache2D LRU 16→256 | ✅ 已提交 |
| `a9ba91a` | MultiNoise 最近邻 | ✅ 已提交 |
| `e3c4a09` | perf_quant | ✅ 代码在，perf_quant.txt 落盘（0.05/0.34μs/pt） |

### 2. 代码/探针

- `biome.rs`：SearchTree 实现（TreeBranchNode/Leaf + build_search_tree + get_resulting_node）
- `mt_fill.rs`：Arc<Ctx> 共享 + N 线程 fill_chunk 完整管线
- `perf_quant.rs` / `perf_fresh.rs` / `biome_scan_cost.rs` / `biome_rows.rs` / `biome_fill.rs` / `fillbench.rs`
- vanilla 参照：`mc_src_extract/.../MultiNoiseUtil.java`（SearchTree 权威）

### 3. 验证记录（cmd-output）

- `biome_scan_cost.txt`：30.783μs/pt（depth 分桶版，**SearchTree 前**）
- `mt_fill.txt`：T=1 790ms / T=2 483ms / T=4 256ms / T=8 170ms，mismatch=0/16
- `perf_quant.txt`：0.05 vs 0.34μs/pt（6.2×）
- `perf_fresh.txt`：fresh 0.10 / cached 0.06μs/pt（1.6×）
- `fillprofile.txt`：density+aquifer 17.3ms / +biome 32.2ms / biome 46%
- `biome_fill.txt`：no-biome 18.9ms / real-biome 34.1ms / biome 45%
- `biome_hot.txt`：54.1/54.6μs/pt（线性扫描基线）
- `biome_perf.txt`：384.6μs/pt（**异常值，见 §二.5**）

---

## 二、逐项审查意见

### 审查点 1：SearchTree 正确性 —— 存疑（结构对齐，但关键性能数字无落盘证据）

**与 vanilla MultiNoiseUtil.SearchTree 逐行对拍**（`MultiNoiseUtil.java` L379-604）：

| 项 | vanilla | Rust | 判定 |
|---|---|---|---|
| 7 维（6 参数+offset） | `HYPERCUBE_DIMENSION=7`，`getNoiseValueList()` 第 7 位=0L | `NDIMS=7`，`vals[6]=0.0` | ✅ 对齐 |
| offset 距离 | `ParameterRange(offset,offset).getDistance(0)` = offset（offset≥0） | `params[6]=[offset,offset]`，`get_squared_distance` 得 offset | ✅ 对齐 |
| getDistance | `l=noise-max; m=min-noise; l>0?l:max(m,0)` | `range_distance` 同式 | ✅ 对齐 |
| getSquaredDistance | 7 维平方和 | `get_squared_distance` 同式 | ✅ 对齐 |
| enclosing 剪枝 | `getResultingNode`：`l>m` 才递归（m=子树 enclosing 距离） | `d < best_dist` 才递归 | ✅ 对齐（严格小于） |
| size<=6 排序 | 按 7 维 `|(min+max)/2|` 之和 | 按 **6 维**（NPARAMS=6）之和 | ⚠️ 结构差异（不影响正确性） |
| 分桶 | `6^floor(log6(n-0.01))` 桶 | `sqrt(n).ceil()` 桶 | ⚠️ 结构差异（不影响正确性） |
| sortTree | 多键排序（dim j, j+1, j+2...循环） | 单键排序（仅 dim） | ⚠️ 结构差异（不影响正确性） |
| previousResultNode 缓存 | 有（thread-local） | 无 | ⚠️ 性能差异（不影响正确性） |

**结论**：SearchTree 的**剪枝逻辑正确**（enclosing 距离是叶子距离的下界，`d < best_dist` 严格剪枝保证找到真最近邻）。结构差异（排序键、桶大小、previousResultNode 缓存）只影响树形/性能，**不影响最近邻正确性**。7 维 + offset 语义与 vanilla 对齐。

**但**：SearchTree 的**性能数字（2.9μs/pt）无落盘证据**。`biome_scan_cost.txt` 是 depth 分桶版（30.783μs/pt），`f73c005` 只改了 biome.rs，**未更新 cmd-output**。2.9μs 只存在于 commit message 和 artifact index.yaml。这是与 review-001 对 perf_quant 相同的证据链违规（数字只在 commit message，无 cmd-output）。

**正确性验证缺口**：SearchTree 是否与线性扫描产生**完全相同**的最近邻，**无直接对拍记录**。blocks_cmp 93.49% 不变是间接证据（见审查点 2），但未覆盖所有区域/所有 depth 值。

### 审查点 2：分桶正确性 —— ❌ 有问题（depth 分桶有真实正确性 bug）

**depth 分桶逻辑**（biome.rs L203-206）：
```rust
let depth_mid = (ranges[4][0] + ranges[4][1]) / 2.0;
if depth_mid < 0.5 { rows_depth0.push(entry); } else { rows_depth1.push(entry); }
```
查询（L229）：`let tree = if d < 0.5 { &self.tree_depth0 } else { &self.tree_depth1 };`

**正确性 bug**：分桶按 depth range **中点**，查询按采样 depth **值**。若某 biome 的 depth range 中点 ≥0.5（进 depth1 桶）但 range 下界 <0.5，则当采样 `d < 0.5` 时该 biome 被排除在 depth0 桶外——**即使它可能是最近邻**。

**实测**（biome_params.json）：
- `minecraft:dripstone_caves` depth **[0.2, 0.9]**，中点 0.55 → depth1 桶
- `minecraft:lush_caves` depth **[0.2, 0.9]**，中点 0.55 → depth1 桶

当采样 depth `d ∈ [0.2, 0.5)` 时，查询走 depth0 桶，这两个 biome 被排除。若某点 humidity∈[0.7,1.0]（lush）或 continentalness∈[0.8,1.0]（dripstone）且 depth∈[0.2,0.5)，这两个 biome 本应胜出但被错误排除 → **返回错误 biome**。

**blocks_cmp 93.49% 不变是否证明分桶正确？** **否**。93.49% 是 badlands 区（chunk 168-175 x -192）的 blocks_cmp 结果。badlands 区是地表区域，depth 通常 >0.5（地表 depth 高），**不覆盖 depth∈[0.2,0.5) 的洞穴场景**。dripstone/lush_caves 是洞穴 biome，出现在地下 depth<0.5 处，badlands 区对拍**不覆盖此场景**。因此「blocks_cmp 不变」**不能证明分桶正确**。

**这是必须修复的正确性 bug**，不是性能取舍。分桶必须保证：任何 biome 只在其可能成为最近邻的 depth 区间内被搜索。正确做法：分桶边界应保证每个 biome 的 depth range 完全落在单桶内（即按 range 下界/上界分桶，或对跨 0.5 的 biome 双桶复制）。

### 审查点 3：多线程验证 —— 通过（mt_fill 可信，但范围有限）

**mt_fill 设计**（mt_fill.rs）：
- `Arc<Ctx>` 共享，Ctx 含 `Arc<DensityFunction>` 树 + `MacroBiome`（BiomeClassifier）
- 每线程独立 `thread_local` 缓存（density.rs L224/321/361 确认 Interpolated/Cache2D/FlatCache 均 thread_local）
- `XoroshiroSplitter` `#[derive(Clone)]`，每 chunk clone（无共享可变状态）
- 每 chunk 独立 `Aquifer`（不共享）

**并发正确性**：✅ 设计正确。所有共享数据（树、BiomeClassifier）构建后只读；可变状态（缓存、splitter、aquifer）均 per-thread/per-chunk。`Arc<Ctx>` 的 Send+Sync 由编译保证（mt_fill 编译运行成功）。mismatch=0/16 实证确认 16 chunk 的 rock 计数在 T=1/2/4/8 下完全一致。

**扩展性**：T=1 790ms → T=8 170ms = **4.65×**。亚线性（非 8×）对内存带宽受限的 fill 工作负载是合理的。mismatch=0/16 证明确定性。**可信**。

**范围限制**：mt_fill 只验证了 16 chunk（40..55, -30..-15 区域），单 seed（8576294172403134396）。未覆盖多 seed、更大规模、不同区域。但作为「Rust 并发无争用」的验证，已足够支撑「Rust 无 C++ 11× 争用」的结论。

**与 C++ 对比**：C++ 11× 热点（density 阶段并发慢）在 Rust 侧未复现——Rust thread_local 缓存 + Arc 只读树避免了 C++ 的缓存争用。这是有意义的结论，但**注意**：C++ 11× 的根因（2026-08-23 DEVIRT 已证伪虚调用，归因 wrapper 链争用 + latency QoS）与 Rust 的 thread_local 设计不同，不能简单说「Rust 避免了 C++ 的问题」——两者机制不同。

### 审查点 4：DFC 方向评估 —— 存疑（方向可关闭，但方案 B 证据仍缺失）

**review-001 的 4 项补验证**：
1. ✅ **fresh-vs-cached**：perf_fresh 0.10/0.06μs/pt（1.6×）——grid 构建影响小，density 稳态即快。**补上了**。
2. ✅ **组件量化**：fillprofile biome 46% + biome_fill 45%——**biome 是真正热点，非 density**。**补上了**。
3. ✅ **Rust 多线程**：mt_fill 4.6× 扩展，无争用。**补上了**。
4. ❌ **方案 B（spline 显式栈）实验**：**仍未补**。无提交/无回退/无输出记录。review-001 明确「不得声称已回退」，本 session 未再跑或记录方案 B。

**DFC 方向能否关闭？** **方向可关闭，但理由需修正**：
- 原结论「DFC 直排对 Rust 无意义」基于「density 已快（0.05μs/pt）」+「方案 B 无收益」。
- 补验证后：density 稳态确实快（perf_fresh 0.10/0.06），**biome 才是真热点**（46%），且 biome 已通过分桶+SearchTree 优化。**「DFC 直排优化 density 无收益」的方向判断成立**。
- **但**「方案 B（spline 显式栈）无收益」的具体实验证据**仍缺失**。若关闭 DFC 方向，应明确：关闭理由是「density 已快 + biome 是热点」，**不是**「方案 B 无收益」（后者无证据）。

**建议**：DFC 方向可关闭（density 非瓶颈），但 artifact 应修正关闭理由，删除/标注「方案 B 无收益」的不可核实声明。方向关闭 ≠ 方案 B 结论成立。

### 审查点 5：性能优化价值 —— 存疑（fillbench 20% 无落盘证据 + 测量不一致）

**fillbench 28.76→22.98ms（20%）**：**无任何 cmd-output 记录**。全仓搜索 fillbench 输出，只有 biome_fill.txt（34.1ms real-biome，**SearchTree 前**）和 fillprofile.txt（32.2ms，**SearchTree 前**）。fillbench 20% 数字只存在于 commit message 和 artifact index.yaml。**证据链缺失**。

**测量不一致**：
- `biome_hot.txt`：54.1/54.6μs/pt（线性扫描基线，合理）
- `biome_scan_cost.txt`：30.783μs/pt（depth 分桶版，合理）
- `biome_perf.txt`：**384.6μs/pt**（异常，比线性扫描基线高 7×）——biome_perf 与 biome_hot 同为 biome_of 测量，结果差 7×，**测量方法不一致**（可能 debug build 或不同采样点）。此异常未解释。

**biome 优化价值方向可信**：biome 占管线 45-46%（fillprofile/biome_fill 一致），扫描成本 54μs/pt 是主因（biome_scan_cost 30μs 扫描 + biome_hot 54μs 全量）。分桶减半 + SearchTree 剪枝**方向正确**。但**具体提升幅度（fillbench 20%）无落盘证据**，且 depth 分桶有正确性 bug（审查点 2）。

### 审查点 6：产物契约 —— 有问题（部分落盘，关键数字缺失）

- ✅ `biome_scan_cost.txt` / `mt_fill.txt` / `perf_quant.txt` / `perf_fresh.txt` / `fillprofile.txt` / `biome_fill.txt` / `biome_hot.txt` / `biome_breakdown.txt` / `biome_perf.txt` 已落盘
- ✅ `.artifacts/dfc-direct-port/index.yaml` 已更新（含补验证结论）
- ❌ **SearchTree 2.9μs/pt 无 cmd-output**（biome_scan_cost.txt 是分桶版，未更新）
- ❌ **fillbench 28.76→22.98ms 无 cmd-output**（全仓无 fillbench 输出记录）
- ❌ **blocks_cmp 93.49% 不变无重跑记录**（93.49% 是 13dbadc 的 badlands 结果，优化后未重跑 blocks_cmp 落盘）
- ❌ **方案 B 实验仍无记录**（review-001 已要求，未补）

---

## 三、审查意见摘要

| 审查点 | 判定 | 理由 |
|---|---|---|
| 1. SearchTree 正确性 | 存疑 | 剪枝逻辑与 vanilla 对齐（正确），但 2.9μs 性能数字无落盘；无与线性扫描的直接对拍 |
| 2. 分桶正确性 | ❌ 有问题 | **depth 分桶有真实正确性 bug**：dripstone/lush_caves depth[0.2,0.9] 中点≥0.5 进 depth1 桶，但 d<0.5 时被排除；blocks_cmp 93.49%（badlands 地表）不覆盖洞穴 depth<0.5 场景，不能证明分桶正确 |
| 3. 多线程验证 | 通过 | mt_fill 设计正确（thread_local 缓存 + Arc 只读树 + per-chunk splitter），mismatch=0/16 实证一致，4.65× 扩展可信；范围有限（单 seed 16 chunk） |
| 4. DFC 方向 | 存疑 | 方向可关闭（density 非瓶颈，biome 是热点），但方案 B 证据仍缺失；关闭理由应修正 |
| 5. 性能优化价值 | 存疑 | 方向可信（biome 45-46% 热点），但 fillbench 20% 无落盘；biome_perf 384μs 异常未解释 |
| 6. 产物契约 | 有问题 | 部分落盘，但 SearchTree 2.9μs / fillbench 20% / blocks_cmp 重跑 / 方案 B 均无记录 |

---

## 四、结论与建议

### 整体确认等级建议：**保持 draft**（不升 candidate，更不升 confirmed）

理由：
1. **depth 分桶有真实正确性 bug**（审查点 2）——这是阻断性问题，必须修复后才能谈 candidate。
2. SearchTree 性能数字（2.9μs）和 fillbench 20% **无落盘证据**——证据链不完整。
3. blocks_cmp 93.49% 不变**不能证明分桶正确**（不覆盖洞穴场景）。

### 必须修复

1. **depth 分桶正确性 bug**：dripstone_caves/lush_caves depth[0.2,0.9] 跨 0.5 边界，d<0.5 时被错误排除。修复方案：① 按 depth range 下界分桶（保证每 biome 只进其 range 覆盖的桶）；② 或对跨 0.5 的 biome 双桶复制；③ 或放弃 depth 分桶，仅用 SearchTree（SearchTree 本身已 10×，分桶的 2× 收益可牺牲以换正确性）。修复后**必须重跑 blocks_cmp 覆盖洞穴区域**（depth<0.5）验证。

### 需补验证

2. **SearchTree 与线性扫描直接对拍**：跑一个 probe 对比 SearchTree vs 线性扫描（a9ba91a 的线性版）在大量随机点上的 biome 结果，确认完全一致。落盘 cmd-output。
3. **补 SearchTree 性能落盘**：重跑 biome_scan_cost 落盘 2.9μs 数字。
4. **补 fillbench 落盘**：跑 fillbench 落盘 28.76→22.98ms（或当前实际值）。
5. **补 blocks_cmp 重跑**：优化后重跑 badlands + 洞穴区域 blocks_cmp，落盘确认数值。
6. **解释 biome_perf 384μs 异常**：与 biome_hot 54μs 差 7×，需说明测量方法差异。
7. **方案 B 实验**：若真跑过，补记录；若没跑过，从 artifact 删除「方案 B 无收益」声明。

### 可接受

8. **mt_fill 多线程验证**：设计正确，mismatch=0/16，4.65× 扩展可信。可作为「Rust 并发无争用」的 candidate 级证据（但整体仍 draft）。
9. **SearchTree 剪枝逻辑**：与 vanilla 对齐，正确性方向可信（待对拍确认）。
10. **DFC 方向关闭**：方向判断（density 非瓶颈）可信，但需修正关闭理由（删除方案 B 声明）。

---

## 五、与 review-001 的关系

review-001 要求补的 4 项验证中，3 项已补（fresh-vs-cached、组件量化、多线程），1 项未补（方案 B）。但本 session 新增的 **depth 分桶引入了新的正确性 bug**（review-001 时是纯线性扫描，无分桶）。因此整体仍不能升 candidate——不是证据不足，而是**引入了新的正确性问题**。

> 本意见为建议，非命令。最终拍板权在宿主人类。
