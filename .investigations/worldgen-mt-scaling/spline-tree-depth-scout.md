# SplineDF 树结构精确测量（production C++ worldgen）——深度 vs 节点数澄清

> 角色：scout 勘探 subagent（只读，未编译未改码）| 日期：2026-08 后续
> 目标：精确测量 production SplineDF 树的真实结构（递归深度 / 节点数 / 每级 load 依赖），
> 定量厘清「长串行依赖链 load 延迟膨胀」的准确含义，钉死「长链」= 深度 vs 节点数，
> 避免在错误机制上投入（不重蹈 DFC 覆辙）。
> 产出文件：`.investigations/worldgen-mt-scaling/spline-tree-depth-scout.md`
> 静态分析脚本（本次新建）：`spline_tree_static.py` + `spline_tree_reconcile.py`

---

## 〇、核心结论（一句话）

**production SplineDF 树是「浅而宽」的树，不是「深链」。** 单实例**最大递归深度 = 3 条边（4 级节点深度，
对应 MVP 表 node[28]/[55] 的 depth=4——MVP 计 root=1，故 4 级节点 = 3 条边）**，
「~90 节点/实例」指的是**节点数**（含大量 scalar leaf 节点），**不是深度**。**SplineDF 实例之间不互相嵌套**
（每个节点的 coordinate/locFn 解析为 `flat_cache` / 二进制运算，**从不解析为另一个 SplineDF**）。
因此「深链 load 延迟串行叠加」与「跨实例嵌套深链」两个机制在 vanilla overworld 数据上**均被排除**。

真正可解释 11× 的机制（候选，须实测确认）是：
**每采样点的「密度函数 wrapper 链 + spline 递归」整体是一个 latency-bound 依赖链**——
它不是 spline 树本身的深度（仅 4），而是 (a) 从 `InterpolatedDF.grid` 角点（每 chunk 1225 次）漏到底层 spline 的
**长 DF wrapper 链**（Interpolated→blend_density→add→mul→quarter_negative→…→flat_cache→cache_2d→spline，约 15-20 层虚调用）
外加 (b) spline 递归的**高节点访问量**（宽树，2^depth≈4-8 个 subnode 访问/采样，每节点一次坐标采样 + 二分）。
两者合计每采样点一串**依赖型内存读 + 虚调用**；8 线程并发把这一串依赖 load 灌入同一缓存层级，
inflate 每级有效延迟 → 每采样 12×（15.8→190μs）。

**关键：精度上，DFC 直排方向仍是对的，但收益来源不是「深链压扁」，而是「砍掉每采样点的高节点访问量 +
长 wrapper 虚调用 + 递归寻址」。若把机制误判成「深链 / 跨实例嵌套」，会修错地方。**

---

## 一、矛盾澄清：深度 4 vs ~90 节点

| 说法 | 出处 | 实测（本次静态重建） | 结论 |
|---|---|---|---|
| 「spline 递归深度只有 4（node[28]/node[55] depth=4）」 | MVP `analyze_spline_table.py`（CpuBackend 表） | 单实例**最大递归深度=3 边（4 层）** | ✅ MVP 深度正确，但那是**CpuBackend 扁平表**的子树深度（root=1 计到 4） |
| 「~90 节点/实例」 | 既有 scout / 时间线 | 生产 SplineDF `nodes` 数组**含 scalar-leaf 节点**：factor=135、jaggedness=44、offset=254（含叶）；**6 实例×~90 节点 ≈ 537** | ✅ **节点数**（含叶），**不是深度** |

**矛盾根本原因**：`buildSplineNode` 把**每个 scalar value 都 `addLeaf` 成一个独立 Node（n=0）**，
导致节点数被「每个标量叶」大幅膨大（offset 254 节点里 201 个是 scalar leaf）。
而**递归深度**只沿 `{coordinate,points}` 子节点链走（3 级），与节点数无关。
→「~90 节点」是**宽度/总量**，「深度 4」是**递归深度**，两者本就该分开，此前混为一谈。

---

## 二、production 树结构量化（每实例 {深度, 节点数, 叶数, 嵌套层数}）

### 2.1 生产 SplineDF 怎么构建（density_builder.h / density.h）

- `buildSpline`（density_builder.h:184）：**每个 `minecraft:spline` JSON 类型 → 恰好 1 个 `SplineDF` 实例**。
- `buildSplineNode`（:193）递归填充**同一个** SplineDF：
  - 每个 `{coordinate, points}` 对象 → 1 个 Node（n=len(points) > 0）。
  - 每个 **scalar value 点** → `addLeaf` → 1 个 Node（n=0，fixedValue）。
  - 每个**嵌套 `{coordinate, points}` value** → 递归子节点（**留在同一 SplineDF 内**）。
  - 节点 `coordinate` → `buildNode(*coord)`，为 **locFn**（locationFunctions 池）。
- **串行化**：`nodes`/`locations`/`derivatives`/`subIdx` 全连续数组（density.h:814-839），无递归 shared_ptr 树。
  - `Node{int locFn, locBegin, subBegin, n; float fixedValue}`，子节点用 `subIdx` 整数索引。

### 2.2 6 个实例的静态重建（overworld，非 amplified）

> 说明：标准 overworld 的 `final_density`（noise_settings/overworld.json → noise_router → final_density）
> 仅**静态可达 3 个 `minecraft:spline`**（factor / jaggedness / offset），见第三节。WG_SPLINESTATS 的「6 实例」
> 与 static 3 不一致（见第五节讨论），但**深度/宽/无嵌套结论与实例数无关**。

| # | 实例（JSON 文件） | 最大递归深度 | 节点数(含叶) | scalar-leaf(n=0) | 子节点数(n>0) | sum_points | 嵌套层数(叶计数) | ~表字节 |
|---|---|---|---|---|---|---|---|---|
| 1 | `overworld/factor.json` | **3**（4 层） | **135** | 86 | 49 | 134 | 34 层2 / 10 层3 | 4.3KB |
| 2 | `overworld/jaggedness.json` | **3**（4 层） | **44** | 28 | 16 | 43 | 6 层2 / 7 层3 | 1.4KB |
| 3 | `overworld/offset.json` | **3**（4 层） | **254** | 201 | 53 | 253 | 43 层2 / 4 层3 | 8.1KB |
| 合计(3 文件) | — | 3（4 层） | **433** | 315 | 118 | 430 | — | **13.8KB** |

- **深度分布（root=0 计）**：factor `{0:1, 1:4, 2:34, 3:10}`；jaggedness `{0:1, 1:2, 2:6, 3:7}`；offset `{0:1, 1:5, 2:43, 3:4}`。
  → **绝大多数节点在 depth=2**（34/43 个 subnode），**depth=3 已到叶**，**无 depth≥4 的更深链**。
- **最大分叉（单节点点数 n）**：factor/offset 到 **11**（子树宽），jaggedness 到 4。
- **节点 coordinate（locFn）类型**（全部为**外部 DF 引用**，无一为 spline）：
  - factor：continents×1, erosion×4, ridges×34, ridges_folded×10。
  - jaggedness：continents×1, erosion×2, ridges_folded×6, ridges×7。
  - offset：continents×1, erosion×5, ridges_folded×47。

> 静止解析见 `spline_tree_reconcile.py` 输出。**「嵌套层数」= 一个 SplineDF 内子节点链的最大深度 = 3**。

### 2.3 关键：坐标（locFn）为何不是 SplineDF（无跨实例嵌套）

每个节点的 `coordinate` 是**字符串引用**，经 `resolveRef` 解析为独立 DF：
- `overworld/continents` / `overworld/erosion` / `overworld/ridges` → `flat_cache(shifted_noise(...))` → **FlatCacheDF**。
- `overworld/ridges_folded` → `mul(-3, add(-0.333, abs(add(-0.667, abs(ridges)))))` → **BinaryOperation** 树（内部又引用 ridges=FlatCache）。
- 无任何 `coordinate` 指向 `minecraft:spline` 类型（overworld 数据中 0 处）。

→ **confirmed（静态，未实测）：spline 的 coordinate（locFn）永不等于另一个 SplineDF ⇒ 无跨实例 spline 递归深链。**
（若某坐标是 `minecraft:cache_2d(cache_all_in_cell(...spline))` 等包装，那也只是「spline 作为包装对象参数」，
其内部仍是**单** SplineDF；坐标本身不嵌套。）

---

## 三、finalDensity 可达的 SplineDF 实例数（3 vs 6 澄清）

从 `final_density`（overworld.json noise_router）静态追可达 `minecraft:spline`：

```
final_density = min( squeeze( mul(0.64, interpolated( blend_density( add(0.1171875, mul(yGrad, add(-0.117, add(-0.078, mul(yGrad256, add(0.078, range_choice(input=sloped_cheese, ...))))))) ) ) ), noodle )
sloped_cheese = add( mul(4, quarter_negative( mul( add(depth, mul(jaggedness, half_negative(noise:jagged))), factor ) ) ), base_3d_noise )
depth    = add( yClampedGradient, offset )           → offset.json spline   (instance #1)
jaggedness = flat_cache( cache_2d( add(0, mul(blend_alpha, add(0, spline)) ) ) )  → jaggedness.json spline (instance #2)
factor   = flat_cache( cache_2d( add(blend_offset*..., mul(-0.503, spline)) ) )     → factor.json spline   (instance #3)
```

→ **静态可达 3 个 `minecraft:spline` 类型 → 3 个 SplineDF 实例**。
- `caves/*`（noodle/pillars/spaghetti_2d/entrances/spaghetti_roughness）**无 spline**（grep 全树仅 factor/jaggedness/offset 3 文件含 `minecraft:spline`）。
- `base_3d_noise` = `old_blended_noise`（无 spline）。continents/erosion/ridges = flat_cache（无 spline）。

**「6」不一致**：WG_SPLINESTATS 报 splineInst=6/537 节点/17KB/195 locFn。静态 3 实例 = 433 节点/13.8KB。
若 6 实例则节点远超 537，故 6 实例**不是** 3 文件的 2 倍，说明「6」来源于**额外** spline 实例
（可能：probe 把 overworld/amplified/large_biomes/或跨维度混入；或 LazyRef 占位导致某 spline 重建；或 eager 注册路径）。
→ **留给主会话用 WG_SPLINESTATS probe 实测敲定**（第六节）。但**深度/宽/无嵌套结论与实例数无关**（每实例深度都≤4）。

---

## 四、每级 load 依赖（sampleNode，density.h L911-962）

`sampleNode(nodeId, pos)` 每级（非叶）的数据访问/依赖链：

1. **`const Node& nd = nodes[nodeId]`** — 连续数组随机读 Node（20B：locFn/locBegin/subBegin/n/fixedValue）。**依赖：上一级子节点索引**。
2. **`double f = locationFunctions[nd.locFn]->sample(pos)`** — **locFn deref**：
   - BASE 模式：`std::vector<DF>` = `shared_ptr<DensityFunction>` → **shared_ptr 指针追逐 → 堆对象 → vtable 虚调用**（locFn 对象散布堆，非连续）。
   - SERIAL 模式（WG_SERIAL_LOCFN=1）：`sampleSerialLocFn` 按 kind（FLAT_CACHE/CACHE_2D/BINOP/OTHER）分派到**连续池**（去 shared_ptr deref，仍保留虚调用；OTHER 回退 shared_ptr）。
   - locFn.sample 自身又是**一长串依赖**（FlatCache = 5×5 网格 bilinear / BinaryOp = 递归求 a、b → 底噪采样；这些都是**依赖型内存读**）。
3. **`const float* locs/derivs/subs = ... + nd.locBegin/subBegin`** — 指针算术，暂不取数。
4. **二分查找 `locs[lo..hi]`** — 对连续 float 数组做 **strided 依赖读**（~log2(n) 次，每次 mid 依赖上次比较结果）→ 找 bracket k/k+1。
5. **读 `ders[k]/ders[k+1]`、`locs[k]/locs[k+1]`** — 连续数组读。
6. **读 `subIdx[k]/subIdx[k+1]`** — 子节点索引（连续数组读）。
7. **递归 `sampleNode(subs[k]…)` / `sampleNode(subs[k+1]…)`**（一般 case 2 路）→ 回到步骤 1，依赖**子节点索引 load 的结果**。

**每级数据访问模式总结**：
- `nodes[]/locations[]/derivatives[]/subIdx[]` = **连续数组读**（表 13.8-17KB，驻留 L2，共享读广播友好）。
- locFn = **散布堆 shared_ptr deref + 虚调用**（BASE；SERIAL 部分缓解）。
- 递归并行度：一般 case **2 路分叉**，但两路**互为依赖**（都依赖本层 coordinate 的 f 与 bracket），且下钻后各自再 2 路 → **每采样的依赖链数 ≈ 2^depth 指数，但每条链深 ≈ 4**。

> **量化**：单采样点节点访问约 `~2^depth ≈ 4-8` 个 subnode（+对应 scalar leaf），每 node 访问 ≈ 4-6 个依赖 load
> （nodes 读 / locFn deref / 二分 locations / subIdx 读）+ 1 次虚调用。⇒ **每 spline 采样 ≈ 20-50 个依赖 load + 虚调用**。

---

## 五、「长链」机制判断（关键澄清）

### 5.1 排除的机制（本次钉死）

| 原假设 | 排除依据（本次实测） | 结论 |
|---|---|---|
| **深链**：spline 树递归深度大，每级 load 延迟串行叠加 | 单实例递归深度**仅 3-4**（4 层），无 depth≥4 的更深链 | ❌ **排除**（深度太小，不足以单独解释 11×） |
| **跨实例嵌套深链**：spline 的 locFn 是另一 SplineDF → 跨实例递归 | 所有 coordinate 解析为 `flat_cache`/`binary`，**无任何**坐标指向 SplineDF | ❌ **排除**（overworld 数据 0 处坐标=spline） |
| **"~90 节点/实例"即深链** | 90 是**节点数**（含大量 scalar leaf），非深度 | ❌ 概念混淆，已厘清 |

### 5.2 保留的机制（候选，须实测确认）

**每采样点的「DF wrapper 链 + spline 递归」整体 latency-bound**：
- **外层 wrapper 链（长）**：`InterpolatedDF.grid` 角点（每 chunk `5×49×5=1225` 次）→ `arg->sample(p)` 一路
  `blend_density → add → mul → quarter_negative → mul → add(range_choice) → … → flat_cache → cache_2d → add(mul) → SplineDF`。
  这是 **~15-20 层 DF 虚调用 + 若干 range_choice 二分**，每层都是**依赖型内存读 + 虚调用**，**每 grid 角点**走一遍。
- **spline 递归（宽）**：每层 wrapper 落到 spline 后，`sampleNode` 再走 3-4 深、4-8 个 subnode 的递归 + 坐标采样。
- **合并**：单采样点（wg_sample_density 一次性走到 spline）= 长 wrapper 链 + spline 递归，**latency-bound**。

**并发放大机制（不依赖锁）**：8 线程同时在 `fillOneChunkCore`（每线程处理不同 chunk）各自走上述整条链，
把所有依赖 load（nodes/table + locFn deref + wrapper 各层）+ 虚调用流**并发地灌入同一 L1/L2/MSHR 层级**：
- 大量**指针追逐/随机读**争用 load-store 单元与 MSHR → 每级有效 load 延迟↑。
- **I-cache 争用**：8 线程跑同一段递归/虚调用代码，失同步互逐 L1I → 取指变慢（Tier-2 叠加）。
- **无锁也能发生**（共享读 + 依赖链争 load 带宽，不争互斥）。

这解释了「无锁、真并行、但单 chunk density 11×」三者自洽；也与既有数据（spline 采样 15.8→190μs，12×）吻合。

> **「长链 load 延迟膨胀」的准确含义**：不是「一棵深树的级数」，而是**每采样点一串较长（wrapper 15-20 层）+
> 较宽（spline 4-8 个 subnode）的依赖链**，其**总依赖 load 数多（20-50/采样）且串行有依赖**，在并发下
> 被共享缓存延迟放大。放大系数由「依赖链总量 × 每级有效延迟膨胀」共同决定。

---

## 六、11× 的无损修复方向（基于准确机制）

### 6.1 会被「错误机制」误导的错误修法（应避免）
- ❌ **减少深度**：深度本就只有 4，无可减；减不到 11× 的量级。
- ❌ **扁平化跨实例嵌套**：无跨实例嵌套，无物可扁平。
- ❌ **仅做 locFn 连续化（WG_SERIAL_LOCFN）**：A/B 已证伪（SERIAL 10.25× vs BASE 10.03× 持平）——locFn 指针追逐**不是**主导，仅去 shared_ptr deref 不够。

### 6.2 正确方向（DFC 直排，但基于正确理由）
把「每采样点的高节点访问量 + 长 wrapper 虚调用 + 递归寻址」**编译直排**为**扁平化可迭代执行结构**：

1. **把 DF wrapper + spline 整棵树编译为 flat IR（显式栈迭代）**：消除递归调用帧链与依赖寻址，改为迭代，
   **打破递归帧间依赖**，提升隐藏并行（MLP/ILP）——这是「浅而宽」树真正的收益点。
2. **共享/合并子结构**：vanilla spline 里大量结构相同/相近的 ridges / ridges_folded 子节点（factor 34 个 ridges、
   offset 43 个 ridges_folded），可按 (coordinateType, 结构) 去重为**可复用子表达式**，把每采样访问量从几十
   降到个位数。
3. **去掉每采样点的虚调用 + shared_ptr deref**：直排为 switch/静态分派 + 连续数据，减少 I-cache 与指针追逐。
4. **坐标（locFn）FlatCache 粒度**：确认坐标的 5×5 网格是否每 chunk 恰好一个 cacheId（避免并发下重复 buildGrid）。

> 精度上这与既有 C2ME-DFC 方向一致（dfc_gen.py GLSL 版显式栈 + CpuBackend 扁平表已有），
> 但**收益论证应从「深链压扁」改为「砍掉每采样点高节点访问量 + 长 wrapper 虚调用 + 递归寻址 / 提升 MLP」**。

---

## 七、下一步验证设计（最小可执行）

1. **WG_SPLINESTATS 实测实例数与结构**（主会话运行）：
   - 在 `SplineDF` 构造/采样处加 env 门控 `WG_SPLINESTATS`（打印每实例 `nodesSize()/tableBytes()/locFnSize()`，
     这些方法已存在 density.h:842-847，只差一个触发点）。
   - 目标：敲定「6 vs 3」实例数，及每实例真实节点/字节（对比静态 433/13.8KB）。

2. **单采样点并发成本隔离（决定性）**：跑 `conc_sample_probe`（`conc_sample_probe.cpp` 已存在）：
   - `conc_sample_probe <seed> <worldgen> <T> density 20000` vs `... noise 20000`（对照纯噪声）。
   - 若 density（含 spline 表 + wrapper 链）随 T 线性放缓、纯噪声正常 ⇒ 定位到「spline 表 + wrapper 链」，坐实 latency-bound。
   - **对照 B（一石二鸟）**：用 DFC 显式栈版本（`.investigations/perf-rework/vulkan-proto/mvp_spline_eval.cpp` 同构）
     跑同并发——**若不随 T 放大 ⇒ 证实「递归+虚调用指针追逐」是放大器，且直接检验 DFC 直排方向**。

3. **深度 vs 节点访问量区分**（避免再混淆）：给 spline 采样加「节点访问计数 + 最大递归深度计数」，
   打印**每采样实际访问的 subnode 数分布**——若平均仅 ~4-8，则证明「深度小、靠宽与子结构复用来放大成本」，
   进一步坐实「非深链」。

4. **cache/miss 量测**（需运行环境）：T=1 vs T=8 下 `perf stat` 读 L1/L2/L3 miss + `dTLB-load-misses`，
   若 miss/延迟显著↑ ⇒ 支持「并发共享缓存延迟放大」。

---

## 八、置信度与候选状态

- **confirmed（静态，必须实测兜底）**：production SplineDF 单实例**递归深度 3-4（4 层）**、**宽树**、
  **无跨实例 SplineDF 嵌套**（坐标=flat_cache/binary）。
- **candidate（需 WG_SPLINESTATS / conc_sample_probe 实测确认）**：11× 的准确机制 =
  「每采样点 DF wrapper 链（15-20 层）+ spline 递归（4-8 subnode）的依赖 load 总量，在并发下被共享缓存延迟放大」。
- **open（待敲定）**：WG_SPLINESTATS「6 实例」vs 静态 3 实例；每采样点 15.8μs 的具体主导子项（wrapper 层 vs spline 递归 vs 坐标采样）。

> 采样污染注意：validation 用 **WG_PHASETICK**（QPC 单次，无 profiling）或 `conc_sample_probe`（无计数器），
> **禁 WG_PROFILE/WG_STAGETIMER**（并发下耗时列被污染，只信计数）。
