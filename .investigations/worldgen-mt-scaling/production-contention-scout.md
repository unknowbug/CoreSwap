# production SplineDF locFn 连续化 — 并发争用准确来源勘探（只读）

> 勘探角色：本 subagent（只读，未编译未改码）| 日期：2026-08 后续
> 课题：production (C++ worldgen) 多线程 density 11× 并发放大的**准确争用来源**，评估「locFn 连续化」（保留多态+直接采样，去散布堆指针追逐）能否无损修复。吸取 DFC 教训：**先钉死争用来源再动，不重蹈「算法重写」覆辙**。
>
> 本文所有结构性事实均直接核对了 `density.h` / `density_builder.h` / `overworld.json` 及 overworld 各 spline JSON（行号内联引用）。**这是「准确机制」判断 = candidate（非 confirmed）**——关键争议点（指针追逐是否占 11× 主导）静态证据微弱，需一节最小实验钉死，禁止未验证先改。

---

## 0. 结论先行（TL;DR）

| 问题 | 判断 | 置信度 |
|---|---|---|
| production locFn 对象是否散布堆（指针追逐存在）？ | **是**。`locationFunctions` = `std::vector<DF>`（`DF=shared_ptr<DensityFunction>`），locFn 对象独立 `make_shared` 堆分配，散布内存 | 确证（density.h:822/846） |
| SplineDF 内部表已连续化吗？ | **是**。nodes/locations/derivatives/subIdx 已扁平连续（17KB L2 驻留）。**唯一残留散布堆 = locFn 池** | 确证（density.h:822-827） |
| 散布 locFn 的 distinct 对象多吗？ | **很少（≈4-6 个）**，被 195 个 shared_ptr 条目重复引用（registry 缓存去重）。指追逐目标是 L2 热对象 | 确证（density_builder.h:220-222 registry 缓存 + JSON 结构） |
| 并发争用的**主导**机制是「cache-line 冲突（写 ping-pong）」还是「内存延迟膨胀（只读共享）」？ | **只读共享 + 长串行依赖链的 load 延迟膨胀**，**不是** cache-line 写乒乓 | 确证（表只读无写；先前 scout c-3 已排除表 ping-pong） |
| locFn 连续化能修复 11× 吗？ | **candidate：大概率不能主导修复**。它只去掉「每节点 ×1 次 L2 命中 deref」，不改变串行依赖链 / 每级 load 延迟膨胀 / I-cache 争用。**绝对耗时微降，放大比（T8/T1）几乎不变** | candidate（需 A/B 钉死） |
| locFn 连续化方案 | **Plan A（全局按类型连续池 + 索引 + 类型标签，保虚调用）可行且无损**；Plan B（std::variant）**破坏 registry 共享 + 内存爆炸，不推荐** | 结构可行性：确证 |

---

## 1. production SplineDF locFn 分布 & 是否散布堆

### 1.1 数据结构（确证）

`density.h:811-941` SplineDF。locFn 池：

```cpp
std::vector<DF> locationFunctions;   // L822  DF = std::shared_ptr<DensityFunction>
```

- 每个 spline 节点的坐标函数存储在 `locationFunctions`，是 `std::shared_ptr<DensityFunction>`。
- `addNode`（L844-846）：`locationFunctions.push_back(std::move(locationFn))` —— 每个非叶子节点 push 1 条 shared_ptr。
- `sampleNode`（L879）：`double f = locationFunctions[nd.locFn]->sample(pos);` —— 每节点经 `shared_ptr` 间接 + 虚调用访问。

**散布堆确认**：locFn 对象自身（pointee）是 `make_shared` 独立堆分配，对象在内存中**不连续**（每个对象带 vtable + 各自成员）。这是「指针追逐」。✅

**但关键要点**：SplineDF 的**内表已连续化**（L823-827：`locations/derivatives/subIdx/nodes` 全扁平连续数组），先前扁平化（2026-08-12）已完成。**唯一残留的散布堆/指针追逐 = `locationFunctions` 池本身**。这是「locFn 连续化」要消灭的最后一个散布源。

### 1.2 locFn 实际类型 & distinct 对象数（确证）

从 overworld 的 spline JSON 与 `density_builder.h` 建模路径推出，terrain spline 的 locFn（spline coordinate）实际解析为：

| spline 引用 | 注册表顶层类型（buildNode/resolveRef 结果） | 用作 locFn 时的类 |
|---|---|---|
| `overworld/continents` | `flat_cache`（wrapper `ShiftedNoiseDF`） | **FlatCacheDF** |
| `overworld/erosion` | `flat_cache` | **FlatCacheDF** |
| `overworld/offset` | `flat_cache` | **FlatCacheDF** |
| `overworld/factor` | `flat_cache` | **FlatCacheDF** |
| `overworld/jaggedness` | `flat_cache` | **FlatCacheDF** |
| `overworld/ridges` | `flat_cache` | **FlatCacheDF** |
| `overworld/ridges_folded` | `mul`（包裹 spline） | BinaryOperation(mul) → 内含嵌套 SplineDF |
| `overworld/depth` | `add`（内含 spline） | BinaryOperation(add) → 内含嵌套 SplineDF |

**distinct locFn 对象**：由 `density_builder.h:220-222`（`resolveRef` 先查 `registry`，命中即返回同一 shared_ptr）+ `registerFunction`（L284-296）缓存 —— **同一 registry 条目（如 continents）全树共享同一实例**。所以 195 个 `locationFunctions` 条目指向的 **distinct 堆对象只有 ≈4-6 个**（continents/erosion + ridges/ridges_folded 等），其余为对同一对象的重复 shared_ptr 引用。

**分布含义**：
- `locationFunctions.size()`（WG_SPLINESTATS 基线 195）= **非叶子节点数**（每个非叶子节点 push 1 条；叶子 n=0 且 `locFn=-1` 不 push）。6 个 SplineDF 合计 195 非叶子节点、537 总节点（342 叶子）。
- distinct locFn **对象** ≈ 4-6 个，且始终 L2 热。

→ **结论**：指针追逐在静态结构上**确实存在**，但其目标对象**极少且 L2 热** —— 每节点指针追逐 ≈「1 次 L2 命中 deref + 虚调用」，而非「DRAM/L3 miss 级散列」。

### 1.3 单点 0.4μs 快源（确证，连续化不得破坏）

production 单点快 = **InterpolatedDF 懒建 grid + thread_local per-instance 缓存**直接在 `arg->sample` 上摊销：
- InterpolatedDF::sample（L497-558）：按 chunk 懒建 5×49×5 网格（L589-619），之后每点只做 8 角点读 + 三线性（L534-548）。
- 每采样缓存都是 `thread_local` per-instance：InterpolatedDF::tlSlots（L576-578）、Cache2DDF::tlSlots（L710-712）、FlatCacheDF::tlSlots（L776-778）、`g_curChunkX/Z`（L40-41）。**无跨线程可变共享**。
- 关键别名 `cacheId`（per-instance，构造时 `nextId` 分配）：thread_local Slot 按 cacheId 索引。**同一 locFn 实例 → 同一 cacheId → 各线程独立 Slot，互不污染**。

→ **连续化绝不可破坏**：①thread_local 网格缓存结构；②per-instance `cacheId` 索引（否则 Slot 错位或重复建 grid）；③同 locFn 实例全 chunk 只建 1 次 grid（registry 共享语义）。

---

## 2. 并发争用准确来源判断（关键）

### 2.1 机制本质（确证）：不是「cache-line 冲突（写乒乓）」

数据全是**只读共享**：spline 表（17KB）、locFn 对象（vtable + 成员）、noise sampler（只读 perm 表）。无写共享。**多只读线程共享同一 cache line = 广播（broadcast），无 invalidation、无 ping-pong、无 coherence 流量**。先前 scout `concurrent-density-probe-scout.md` c-3 已确证「17KB 只读小表共享读广播，不 ping-pong，瓶颈不在表」。

→ 真正的并发放大机制是 **memory-subsystem 延迟 / 吞吐 QoS**：多个线程同时灌入「长串行依赖链」的 load 流 → L2/MSHR/load-store 队列排满 → **每 load 的 latency 膨胀** → 长依赖链（每级 load 延迟串行叠加）的链延迟被**非线性放大**。这发生在**任何锁都没有**的情况下，与「无锁 + 真并行 + 单 chunk 膨胀 11×」三条件完全自洽。

### 2.2 三类候选的归属（连续化能修什么，不能修什么）

| # | 候选 | 机制 | locFn 连续化能否修复？ | 证据 |
|---|---|---|---|---|
| **A** | 散布堆 locFn 指针追逐 | 每节点 1 次 shared_ptr deref → L2 热对象 + vtable 间接 | **能修**（去散布、去 deref、池连续可预取） | density.h:879/822 |
| **B** | Spline 递归串行依赖链（~90 节点/实例，含嵌套 spline 跨实例跳转） | 每级有数据依赖，load 延迟串行叠加 = latency-bound 核心 | **不能修**（依赖链结构不变，仅节点边界 deref 少 1 次） | density.h:876-924 |
| **C** | I-cache / code-fetch 争用（8 线程同跑递归+虚调用冷路径） | 失同步互驱逐 L1I → 取指延迟 | **不能完全修**（Plan A 保留虚调用；Plan B 可去虚调用但引入大额内存/破坏共享） | density.h:876-925 / scout c-1 |

**归属结论**：locFn 连续化**只修复 A**（且 A 的绝对值因为对象少且 L2 热而偏小）；**B 是 latency-bound 主导**，连续化**不动它**；C 最多部分缓解。→ **单靠 locFn 连续化，10-12× / 8.4× 的放大比预期几乎不降**，只能把 T1 与 T8 的**绝对**耗时等比例微降（放大比≈不变）。

### 2.3 locFn 连续化对 11× 的期望收益（candidate，关键争议）

**静态侧证据（偏向「指针追逐非主导」）**：
- locFn distinct 对象 ≈4-6 个 + L2 热 → 每节点指针追逐是 L2 命中 deref（~4-8 周期），非 DRAM/L3 miss。
- spline 递归 ~90 级，每级有若干**连续表 load**（nodes/locs/ders/subs）+ 嵌套 spline 跨实例跳转 —— 这才是 load 延迟叠加的主体，连续化不动它。
- 先前 scout 已确认「瓶颈不在表（17KB 广播），在 locFn 指针追逐 + 长链」——但指针追逐的**单次成本**远低于长链的**每级连续表 load × 90 级**。

**但**：先前 scout Tier-1 **把**「递归虚调用链 + 指针追逐」**合并命名**为共享内存延迟放大，未单独量化「纯 locFn 指针追逐」占比。DFC（8.4×→1.3×）同时消除三件事（连续表 + 无递归 + 无虚调用），**locFn 连续化只解决其中一件**（连续表那一件里 locFn 部分）。**故不可从 DFC 结果直接外推「locFn 连续化 ≈ DFC」**。

**判断（candidate）**：locFn 连续化是**低风险、无损、进步**的小优化，能把绝对 per-sample 降低一个小的常数；但对**放大比**（11×/8.4× 这类）的削减**预计有限**。**在未验证「A 占主导」前，不应把 locFn 连续化当成「无损修复 11×」的主方案** —— 这正是 DFC 教训要求先钉死再动的地方。

---

## 3. locFn 连续化可行性（Plan A vs Plan B）

### 3.1 前提：必须满足的「无损」约束（确证）

1. **不算法重写**：spline Hermite 插值公式逐位不变（BK-001 零退化），只是存储布局变。
2. **保留多态/直接采样**：sampleNode 的递归逻辑与虚调用形态可保留（Plan A）或不保留（Plan B），但采样值必须与现状逐位一致。
3. **不破坏 thread_local 网格缓存 & per-instance cacheId 别名**（§1.3）。
4. **不破坏 registry 共享语义**（同一 locFn 全 chunk 只建 1 次 grid）。

### 3.2 Plan A：全局按类型连续池 + 索引 + 类型标签（保虚调用）—— **推荐**

- 设计：建一个**全局按类型连续池**，把全树所有 distinct locFn 对象（continents/erosion FlatCacheDF、ridges/ridges_folded 嵌套 SplineDF 等）**实体**放入各自 `std::vector<T> pool`；SplineDF 的 `locationFunctions` 从 `std::vector<DF>` 改为 `std::vector<LocFnRef>`，`LocFnRef = {Kind kind; int index;}`。
- `sampleNode`：`switch (locFn.kind) { case FLAT_CACHE: return flatCachePool[index].sample(pos); ... }` —— 池内对象**实体**（非指针），连续，可预取；仍走 `sample`（Plan A 保留虚调用，或改为非虚 `sample_impl`）。
- 优点：
  - **保留 registry 共享**：池是全局单例，continents 只在池里出现 1 次 → cacheId 不变 → thread_local Slot 正确、grid 每 chunk 只建 1 次 → 单点 0.4μs 结构不破坏。✅
  - **无损**：公式/语义不变，只去 deref + 使池连续。✅
  - **不算法重写**：spline 求值数学不变。✅
- 复杂性：**中高**（非算法级，但触及构造架构）。需在 `density_builder` 里把 registry 的 `shared_ptr` 反转为「全局池 slots + 索引」，并在 `buildSplineNode`（density_builder.h:193-217）填充时登记 `(kind,index)`，spline `addNode`（density.h:844-852）改存 `LocFnRef`。**这是所有权重构，非求值重写**。

### 3.3 Plan B：std::variant 联合（无虚调用） —— **不推荐**

- `using LocFnVariant = std::variant<FlatCacheDF, SplineDF, NoiseDF, Constant, ...>`（~20 类全列举）；`locationFunctions` = `std::vector<LocFnVariant>`；`samp` 用 `std::visit`。
- 优点：无虚调用（`visit` = switch），全内联连续。
- 缺点：
  - **破坏 registry 共享**：variant 持有对象**拷贝**，同一 continents 会在每个 spline 里各有 1 份 → cacheId 各自分配 → thread_local Slot 各自构建 → **每 chunk 重复建 25 点 grid（每份一员）** → 单点快被破坏 + 浪费。❌
  - **内存爆炸**：`sizeof(LocFnVariant)` = 最大成员大小。`InterpolatedNoiseDF`/`SplineDF` 含多个 vector / OctavePerlinNoiseSampler 数组 → variant 巨大（几百字节），`std::vector<variant>` 内存/拷贝/缓存全恶化。❌
  - 需编译期全类型列举，且现有 `sampleNode` 递归逻辑（L876-924）要整体改写为 visit 分发，**更接近算法重写**，违背 DFC 教训。❌
- **结论：Plan B 不可行**（破坏无损约束 + 引入新问题）。

### 3.4 方案选型

**Plan A 是唯一满足「无损 + 不算法重写 + 保多态/直接采样 + 保 thread_local 快」的方案**。但它修的是 A（指针追逐），不修 B/C —— 对 11× 放大比的预期收益有限（§2.3）。

---

## 4. 风险 / 边界

1. **是否破坏多态**：Plan A 保留虚调用（或改用非虚 `sample_impl` + kind switch），多态语义保留。✅ 但若直接用 kind-switch 而非虚调用，需确保 switch 覆盖全部分支且与现有 `dynamic_cast` 诊断（density.h:913-919）等价。
2. **20 类连续池内存**：各池**只放 distinct 实例**（≈4-6 个），非 195 条目 → 内存极小（几十字节级），不是问题。关键是**引用去重**（registry 缓存已天然去重）。
3. **构建时如何连续化**（density_builder）：
   - `buildSplineNode`（density_builder.h:193-217）当前用 `buildNode(*coord)` 返回 shared_ptr → 需改为「查全局池 / 建池 / 返回 (kind,index)」。
   - 嵌套 spline（ridges/ridges_folded 作为 locFn）需在全局池里也登记一个 SplineDF 实体 slot。
   - 需要**两遍构建**或**池先建后引用**处理循环引用（ridges_folded 引用 ridges，后者引用 continents）——用现有 registry 占位（LazyRef）机制扩展。
4. **风险**：全局池引入**构建期顺序依赖**（池必须先于引用拉满），且 owner 从 shared_ptr 改为池 slot 引用计数，需小心悬垂（池在销毁前不可变）。构造仍是单线程（`wg_create`），无并发构建风险。
5. **收益不确定性（最大风险）**：连 §2.3 判别正确后，若 A 非主导，Plan A 白做（DFC 教训重演）。**必须先做 §6 验证，再决定是否落地 Plan A**。

---

## 5. 结论

**locFn 连续化是不是「无损修复 11×」的可行方向？**

- **结构可行性：是**（Plan A 无损、不算法重写、保多态/直接采样、保 thread_local 快）。
- **对 11× 的修复能力：candidate（大概率不够）**。它只消除 A（散布堆指针追逐）。静态证据强烈偏向 B（spline 递归串行依赖链的每级 load 延迟膨胀）才是 11× 的**主导**放大器，而：
  - locFn distinct 对象 ≈4-6 个且 L2 热 → 指针追逐是 L2 命中 deref（绝对成本小）。
  - spline 内表已连续化（17KB L2 驻留），连续化无法再优化它；嵌套 spline 跨实例跳转的连续表 load 才是 load 延迟主体。
  - 连续化**不动**递归依赖链（B）与 I-cache（C）。
- **期望收益**：绝对 per-sample 微降（T1 与 T8 等比例降），**放大比（11×/8.4×）预计几乎不变**。若把 locFn 连续化当「无损修复 11×」主方案，ID=RISK 重复 DFC 教训。
- **真正能修 11× 的方向**（另立）：必须是**改变依赖链形态 / 提升 MLP** 的东西（如 C2ME DFC 式全扁平直排 + 显式栈 + 无虚调用），但那属于 DFC 已证「绕圈」的算法重写 —— 所以**本课题的出路可能不是 locFn 连续化**，而要先钉死 A/B/C 各自占比再定向。

**一句话**：locFn 连续化 = **值得做但预期收益不高**的低风险优化，**不能作为 11× 的无损主修复**；先验证「A 是否主导」，不验证就动会重蹈 DFC 覆辙。

---

## 6. 下一步验证建议（钉死 A 是否主导，再决定 Plan A 值不值得）

**决定性实验（最小、无损、不改最终语义）**：A/B 测试 `locationFunctions` 存储布局，其余**原封不动**（同一棵 SplineDF 树、同递归、同虚调用、同 InterpolatedDF grid、同 thread_local 缓存），在 **T=1 vs T=8** 各测固定样本：
- **变体 BASE**：现在的 `std::vector<DF>`（shared_ptr → 散布堆）。
- **变体 SERIAL**：`std::vector<LocFnRef>` + 全局按类型连续池（Plan A 原型），`LocFnRef={kind,index}`，`sampleNode` 经 kind-switch 访问池实体。**保留递归与虚调用**（只去 deref + 池连续）。

**判据**：
- 测每样本绝对成本（T1）与**放大比**（T8/T1）。
- 若 SERIAL 的放大比显著**低于** BASE（向 DFC 的 1.3× 靠拢）→ **A 主导**，Plan A 是真主修复，值得落地。
- 若 SERIAL 的放大比**与 BASE 基本持平**（仍 ~8-11×，仅绝对耗时微降）→ **A 非主导**，Plan A 收益有限，**不做**；转向 B/C（依赖链 / I-cache），另立方向。

**为何这是唯一可信判据**：DFC 同时改了 3 件事，无法从 DFC 1.3× 反推「只改 locFn 存储」。只有**只改 locFn 存储、其余不动**的 A/B，才能隔离 A 的贡献。

**注意（测量纪律，AGENTS.md 八）：** 禁用 WG_PROFILE/WG_STAGETIMER（并发下污染）；用 WG_PHASETICK（QPC 单次 + 无 profiling 污染）；对比「吞吐均值（wall/N）」与「每 chunk 延迟（阶段耗时）」要分开；线程池正确性先核（worker 就绪/通知竞争已修复 0a781e1）。

**一次实验可顺带检验的**：SERIAL 变体若同时发现单点 T1 也明显变快（如 >10%），说明 locFn 存取本身（即使单线程）是热点 —— 佐证 A 有独立价值，但仍需放大比判据确认它是否修 11×。

---

> **本文件状态**：candidate（需 §6 实验钉死 A/B 占比后 upgraded）。结构性事实（locFn 散布堆、内表已连续、表只读广播、Plan A 可行/Plan B 破坏共享）为确证；「A 是否主导 11×」为候选判断，静态证据薄弱。产出依据：density.h:811-941 / density_builder.h:193-217,220-222 / overworld JSON spline 坐标 / concurrent-density-probe-scout.md。
