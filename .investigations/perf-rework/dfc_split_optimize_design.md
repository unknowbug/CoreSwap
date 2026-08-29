# DFC split 组织优化设计（phase-4a-2 第二步：量测瓶颈 + 设计 per-cell split / edgeCol / thread_local）

> worker：phase-4a-2 优化 worker（只读 + 写设计，不改 C++/production，未运行）。
> 状态：本文档 = **candidate**（设计 + 静态量化，未落 C++、未编译、未跑 benchmark）。
> 前置：`cpu_backend.h`（path-C grid 缓存最小正确版，§9）、`dfc_gen.py`（生成器）、`density.h`（production 蓝本）、`dfc_grid_cache_design.md`（§8 split 翻转关键发现 + §9 最小正确版）。
>
> **grounded 事实（采信，源码级）**：
> - `splitTotal = 8672`（cpu_backend.h L19）；`perSample = 352`（L21）；`DF_NODES = 163`（L577）；`minY = -64`（L34）。
> - grid 缓存：`float gridCache[N_INTERP][49][5][5]`（846），`int64_t gridKey[N_INTERP]`（847），`N_INTERP=5`（587），`INTERP_ROOTS = {134,140,148,151,155}`（588）。
> - `buildInterpGrid`（856-879）：对 `gy∈[0,49) gz∈[0,5) gx∈[0,5)` 共 **5×49×5=1225** 网格节点，每节点 `split(nx,ny,nz, splitCoord.data())`（覆盖全部 8672 floats）+ `eval_df_base(root, 0, 0, nx,ny,nz)`（corner=0，§9.2 已证实节点是 cell 的 (0,0,0) 角点）。
> - `split()`（314-538）是**整树单调**：一次调用遍历 `root_df` 全树，对每个 noise/old_blended/interpolated 的 **8 个角点（@c0..@c7）** 全部展开；即一次 `split()` = 整棵树（含 5 个 interp 的 8 角点）的拆分 = 8672 floats。
> - production `InterpolatedDF::buildGrid`（density.h 589-619）：1225 节点 × `arg->sample(p)`（无独立 split 阶段，直接 DFS 求值 delegate），+ edgeCol gx=4 列复用（614-618），+ thread_local slot（565-579）。

---

## 1. 性能瓶颈量化

### 1.1 当前 buildInterpGrid 单 chunk（单 interp）构建成本

`buildInterpGrid(interpIdx, chunkX, chunkZ)` 的工作 = 每个网格节点做两件事：

| 项 | 每节点动作 | 每 chunk 单 interp 次数 | 每次成本 |
|---|---|---|---|
| **split()** | 对整棵树（5 个 interp × 8 角点 + 全部 noise/oldBlended）做拆分，**写满 splitCoord = 8672 floats** | **1225** | 8672 次 float 写 + 每 octave 若干 `splitOctave`/`split7`（floor、origin 加、坐标写）+ 若干 `shiftNoises.at(...).sample`（递归 octave perlin 采样） |
| **eval_df_base** | 对 interp 根 delegate 做 163 节点解释执行 | **1225** | `for i in 0..DF_NODES(163)` 逐节点分支 + 读 splitCoord（`normal_noise`/`interp_noise`/`spline_eval` 读 `NOISE_SLOT_BASE[a1] + corner*STRIDE`） |

**单 chunk 单 interp 汇总**：
- split() 调用 = **1225 次**，每次写 **8672 floats** → **1225 × 8672 ≈ 10.62M float 写**。
- eval_df_base 调用 = **1225 次**，每次 163 节点。

**单 chunk 全部 5 个 interp**（sampleInterpGrid 逐 interp gridKey 命中，fillOneChunkCore 内 5 个 interp 各建一次网格）：
- split() 调用 = **5 × 1225 = 6125 次**，写 = **6125 × 8672 ≈ 53.1M float 写**。
- eval_df_base 调用 = **5 × 1225 = 6125 次**，每次 163 节点。

### 1.2 冗余来源（为何这是纯浪费）

`split()` 是**整树单调**生成（`_gen_split_lines(root_df, "x","y","z")`，dfc_gen.py L1698），一次调用就把**整棵 DFC 树的所有噪声**拆好——包括 **5 个 interpolated 节点的全部 8 角点（@c0..@c7）** 以及所有顶层 noise/oldBlended。而 `buildInterpGrid(k)` 只要 interp k 的 delegate 根（`INTERP_ROOTS[k]`）的拆分。于是：

1. **跨 interp 冗余**：一次 split() 计算了 5 个 interp 的 8 角点（5×8 = 40 套角点展开）+ 顶层噪声，但给定 interp 的 `eval_df_base(root_k, 0, 0, ...)` 只消费 interp k 的 delegate 闭包引用到的噪声槽。其余 4 个 interp 的角点展开**全部白算**。→ 约 **5×** 的 interp-corner 拆分工作是浪费的。
2. **节点 vs cell 冗余**：`buildInterpGrid` 对每个网格节点各调一次 `split()`，而一次 `split()` 计算的是**该节点所在 cell 的 8 角点**。节点 (gx,gy,gz) 是 cell (gx,gy,gz) 的 **(0,0,0) 角点**，用得着 8 角点里 1 个；但相邻节点（(gx+1,gy,gz) 等）又会在**自己的** `split()` 里重复计算那 7 个共享角点。一个 interior 网格节点是 8 个相邻 cell 的公共角点（在各 cell 的 (dx,dy,dz) 组合里各出现一次）→ 该节点的 split 值在贪心 per-node 下被 **~8× 重复计算**。
3. **角点失真**：一次 `split()` 就算出了 8 角点，其中只有 corner=0（节点自身）被 eval_df_base 用；其余 7 角点是给三线性插值用的，但那属于 interp 的**别**的节点，grid 阶段根本不用。

### 1.3 vs production buildGrid 的成本差

| | DFC buildInterpGrid（当前） | production InterpolatedDF::buildGrid |
|---|---|---|
| 每节点代价 | split() 整树 8672 floats + eval_df_base 163 节点 | `arg->sample(p)`：delegate 树的直接 DFS 求值（**无独立 split 阶段**，无 8672-float 中间缓冲） |
| split 冗余 | **有**（整树 + 8 角点 + 跨 interp 5× + 角点共享 ~8×） | **无**（delegate 采样按需求值） |
| 每 chunk 节点数 | 5×1225 = 6125 | 每 interp 实例 1225（5 实例 × 1225 = 6125 采点，但**无** split 冗余） |
| edgeCol / thread_local | 未做（gridKey 触发整 chunk 重建；实例成员非线程安全） | edgeCol 复用 gx=4 列（~36% 省）；thread_local slot |

**成本差定性**：DFC buildInterpGrid 每节点的**叠加常数** = split() 的整树成本，而 production 每节点只有 delegate 采样本身。在「每节点极小的 split 重用度」下，DFC 的 split() 摊销远大于 production 的 `arg->sample`（后者直接求值且被 arg 内部各级 interpolated 的网格缓存摊薄）。**核心差 = split() 的冗余放大，不是 eval_df_base。**

### 1.4 split 占总成本比例（估算）

> 说明：**未运行**，以下为源码级保守估算。split() 与 eval_df_base 的单位成本无法绝对量化，但可排序。

- `eval_df_base`：163 节点，但 interp delegate 闭包实际活跃节点 << 163（多数是浅层加/乘/常数，读 splitCoord），单节点 ~数次 float op。相对便宜。
- `split()`：写 8672 floats（`splitOctave` 每次写 6 floats + floor + origin；`split7` 写 7 floats），外加每 interp 8 角点引用的 `shiftNoises.at(...).sample()`（**递归 octave perlin 采样，是重头**——shift noise 每次 `sample` 走 N 个 octave 的 perlin）。角点越多，shift/spline coordinate noise 采样越多。

**估算 split() 单节点成本 ≥ eval_df_base 的 5~10×**（因 shift noise 递归 perlin + 整树 8672 writes vs 163 节点只读）。故：
- **split 冗余占 buildInterpGrid 总成本 ≈ 80%~90%**（保守下界 60%，上界 95%）。
- **若能消除 split 冗余 -> 单 chunk 构建成本降至 ~1225 × eval_df_base 级别**（production 同级）。

**收益量级（粗）**：split() 调用 `6125 → 768`（每 cell 一次，见 §2），且每次 split 不再整树（只拆该 cell 的 interp 角点）。若 split 占 85%，则整体构建成本 ≈ 15%（不 split）+ 85%×(768/6125)≈11% → **降到 ~26%**；进一步按需只拆 1 角点（§2.6）可更低。**绝对须 benchmark 复核（§5 步骤 5）。**

---

## 2. split 组织优化设计（每 cell 一份 split）

### 2.1 目标

把 split 从「按节点整树重算（1225×/interp）」翻转为「**按 cell 组织**」：每个 cell（`cell = (chunkX, chunkZ, cx, cy, cz)`，尺寸 4×8×4 = 128 block）计算**一份** split（含该 cell 的 8 角点展开），供该 cell 的 8 个 grid 节点（cell 角点）复用。网格节点值 = 「该 cell 的 split」+「节点在 cell 内的 corner 索引」经 `eval_df_base` 求值。

### 2.2 网格节点 ↔ cell 映射（grounded）

- 网格节点 (gx,gy,gz)，`gx,gz ∈ [0,5)`，`gy ∈ [0,49)`，坐标 `nx = chunkX*16 + gx*4`、`ny = minY + gy*8`、`nz = chunkZ*16 + gz*4`。
- `split(nx,ny,nz)` 内（dfc_gen.py 1599-1603）算 `_gx = nx - chunkX*16`、`_cx = _gx/4 = gx`（同理 `_cy=gy`、`_cz=gz`）；`dx = nx - (chunkX*16 + cellX*4)`，`cellX=gx` → `dx = 0`（恒为 0，§9.2 已证 corner=0）。
- **因此 grid 节点 (gx,gy,gz) 就是 cell (gx,gy,gz) 的 (0,0,0) 角点**。cell 索引 = (gx,gy,gz)。
- cell 数量：`gx∈[0,4)`（4）、`gy∈[0,49)`（其 cell 索引 0..48 但真实 cell 是 0..47，gy=48 是顶边界列）、`gz∈[0,4)`（4）→ **interior cell = 4×48×4 = 768**（gy=48 那层 = 顶边界列，属跨 chunk，`minY+height-...`，见 edgeCol）。

### 2.3 设计：按 cell 一份 split

新组织：**iter 每个 cell (cx,cy,cz)，算一次 split(cell)，用其 8 角点填 8 个网格节点**，去重共享角点。

**数据组织（splitTotal 布局翻转）**：
- **现在**：`splitTotal = 8672` 是**单点**整树拆分（含 5 interp × 8 角点 + 顶层噪声）。`splitCoord[idx]` 按 sIdx 组织（`b = sIdx*splitTotal + splitOffset`）。
- **目标**：把「角点维度」从 splitTotal 里**拿出来**，改由「cell → 8 角点」索引。即 splitTotal 变为**单 cell 的委托拆分大小**（`split_total_cell`），`splitCoord[cellIdx][corner]` 存各角点的拆分；cellIdx 用 chunk 内 cell 序号（0..767），corner 0..7。
- 但注意：interp 的 delegate 求值（§9.2）只需 **corner=0**（节点自身）。**三线性插值需要的 8 角点**其实已经由「相邻 cell 的角点节点」覆盖，不需要在 splitCoord 里额外展开 8 角点。

**关键认知（§8.4 已证）**：grid 节点值 = `eval_df_base` 用「该节点所在 cell 的 split」+「节点在该 cell 的 corner」。节点是 cell 的 (0,0,0) 角点 → corner=0。所以**每个 grid 节点只需它 cell 的那一份 split 的 corner=0**。

### 2.4 具体改动

**A. 生成器 `_gen_split_lines` 的 interpolated 分支（dfc_gen.py L1599-1613）**：
- 当前：对「调用点 cell」做 8 角点展开，`@c{c}` suffix（每角点独立注册 normal/old_blended，独立 splitBase）。
- 改为 **cell 粒度**：`split()` 分裂时按 cell 组织——为每个网格节点只生成「该节点 cell 的 corner=0 拆分」，**不再展开 8 角点**（或把 8 角点改为「cell 的 8 个角点值」供节点复用）。
- 方案 a（**窄改，推荐先行**）：保留单 `split()` 但**按 cell 去重**——`buildInterpGrid` 先对每个 cell 调一次 `split(cellCorner0)` 并把 `splitCoord` 存到 per-cell 槽，再逐节点从 per-cell 槽读 corner=0。**不改生成器接口**，只改 C++ 侧缓存结构。
- 方案 b（**深改，生产对齐**）：生成器新增「grid 模式」——`_gen_split_lines` 在 interpolated 分支不再展开 8 角点，而是输出「按 cell」的拆分 + splitTotal 布局按 cell 组织。牵连 `_compute_val_layout`（L414）的 `per_sample`、`bases`、`val_slots` 全要重算（§3 风险高）。

**B. splitTotal 布局（dfc_gen.py `gen_cpu` L1655-1676 + `split` L1837 + `splitTotal`）**：
- `split_total_cell` = 单 cell 的委托拆分大小（远 < 8672，因为不再 × interp 数 × 角点数）。
- `splitCoord` 索引 = `cellIdx * split_total_cell + ...`（cellIdx ∈ [0, 768)）。
- `perSample`（L21, 352）= 相应更新（与 shader PER_SAMPLE 对齐，若 shader 也改；否则保持 shader 旧布局、DPC 用新布局——**需同步**）。

**C. CpuBackend `buildInterpGrid`（cpu_backend.h L856-879）**：
- 目标伪码：
  ```
  buildInterpGrid(interpIdx, chunkX, chunkZ):
    for cell (cx in 0..3, cy in 0..47, cz in 0..3):
        cellIdx = (cy*4 + cz)*4 + cx         // 768 cells
        split(cellCorner0Coord)              // 一次 split，只拆该 cell corner=0
        splitCoord_cell[cellIdx] = splitCoord // 存该 cell 的拆分
    for node (gx, gy, gz):
        cellIdx = node 所在 cell
        // 从 splitCoord_cell[cellIdx] 读 corner=0
        gridCache[interpIdx][gy][gz][gx] = eval_df_base(root, 0, cellIdx_base, nx,ny,nz)
  ```
- **grid 节点值唯一性**（verif_grid_cache_correctness.md）：同一节点可作不同 cell 的角点，值相同。用「该节点所在 cell 的 split + corner=0」求值即可保证唯一（§8.4 已证正确姿势 = 每节点用其 cell 的 split）。

### 2.5 复杂度评估

| | 当前（贪心 per-node） | 优化（per-cell） |
|---|---|---|
| split() 调用 /interp/chunk | 1225 | **~768**（4×48×4 interior cell）|
| split() 调用 /chunk（5 interp）| 6125 | **~3840**（若 split 按 interp 拆；若 interp 共用则 768）|
| split 每次是否整树 | 是（5 interp × 8 角点）| **否**（只拆该 cell；方案 a 仍整树但按 cell 去重调用 → 需再改生成器才消除整树；方案 b 直接按 cell）|
| eval_df_base /interp/chunk | 1225 | 1225（不变）|
| 顶/边界列 | 1225 节点含 gx=4 边界 | gx=4 列由 edgeCol 复用（§3）|

> **口径澄清**：这里的 **768** = 单 chunk 网格的 interior cell 数（4×48×4，对应 grid 节点去重维度）；与 design_split_grid.py 的「1024 采样点 → 32 个不同 cell」（采样分布维度）是**两个不同 scope**，勿混用——前者用于量化 per-chunk 构建，后者用于观察「同 cell 共享 split」。

**净增益**：方案 a（纯 C++ 按 cell 去重，不改生成器）把 split() 调用从 1225 → 768/（interp），且**消除了跨节点共享角点的 ~8× 重复计算**（因为每 cell 一份，8 角点复用）；但仍保留「整树 5 interp」的浪费。方案 b 再消除跨 interp 5× 浪费。

### 2.6 是否破坏已验证的 DFC 对齐（逐位重验清单）

**验证要求**：★ 必须逐位重验（§9.3 的 maxdiff 9.57e-07 / 2.06e-08 基线）。

1. **corner=0 绑定**：优化后节点仍须用「其 cell 的 split + corner=0」。若改成「cell 的 8 角点、节点按 (dx,dy,dz) 选 corner」——**注意**节点是 cell 的 (0,0,0) 角点，故 corner 恒 0；**不可**沿用原任务提示的 floorDiv corner 映射（§9.2 已判其不自洽，会读相邻角点 split 破坏对齐）。**维持 corner=0。**
2. **sIdx 绑定（§8.4 铁证）**：grid 节点必须用「所在 cell 的 split」，不可全局 sIdx。这是 grid 缓存对齐的关键。per-cell 存储天然满足。**不可回退到全局 sIdx=0。**
3. **splitCoord 还原 / minY**：buildInterpGrid 当前 `splitCoord.swap(saved)` 还原外层（block 位置）split。per-cell 后需**确保外层 eval_df 的非 interp 路径所需的（block 位置）split 不被破坏**——per-cell split 槽必须与「当前点 split`（sIdx 主槽）隔离，或构建后照旧还原。
4. **verif_grid_cache_correctness + dfc_cpp_vs_prod + dfc_cpp_verif** 三张表全跑通，maxdiff 保持 ≤ ~1e-6 级。

---

## 3. edgeCol 设计（跨 chunk 列复用，对齐 production density.h）

production `buildGrid`（density.h L595-618）：
```cpp
const bool reuseLeft = (slot.edgeCX == chunkX - 1 && slot.edgeCZ == chunkZ);
for (gx...) if (gx==0 && reuseLeft) { grid[..gx=0] = edgeCol[..]; continue; }
... 构建 gx=0..GX-1 ...
slot.edgeCX = chunkX; slot.edgeCZ = chunkZ;
slot.edgeCol = grid[.. gx=GX-1 列];
```

**对应 DFC**：
- 网格节点 `gx=4`（即 `nx = chunkX*16 + 4*4 = chunkX*16+16`）== **chunkX+1 的 gx=0** 列（x 相同）。故每 chunk 把 gx=4 列存为 edgeCol（per interp，`[49][5]` = 245 floats），构建右邻 chunk（gx=0）时复用。
- 复用条件：`gridCache` 需记录每个 interp 的 `edgeCX/edgeCZ`（当前 chunk 键）+ `edgeCol`。复用须在 `gridKey` 命中检查之外增量判断左邻标记。
- **仅水平方向（x）复用**（production 只做 gx 列，不做 gz 行——保持蓝本一致，避免额外正确性风险）。
- **收益**：gx=4 边界列是 1225 节点中的 49×5=245 个节点（20%）——但真正跨 chunk 共享的是 x 边界 `gx=4` 一列（245 节点）/chunk。设计文档应精确为：gx=4 列（245 节点）被复用，节省 ~20% 节点构建；若也做 z 边界行则可再省（但生产未做，建议先只做 x，与蓝本公平）。

**实现注意**：
- edgeCol 值必须是**未三线性插值前的网格节点原始值**（`gridCache[interpIdx][gy][gz][gx=4]`），构建时存，右邻 gx=0 直接拷贝。
- 必须在「gridKey 命中 + 左邻标记」同时满足才复用；否则退回全建（正确性优先）。

---

## 4. thread_local 设计（并发安全，对齐 production thread_local slot）

production（density.h L563-579）：`static thread_local std::vector<Slot> slots` + `Slot& slot = slots[cacheId]`（per 实例缓存，O(1) 按实例 id），slot 存 `key + grid + edgeCol`；`tlSlots()` 一次性扩到实例数（构造后固定，`instanceCount` 原子）。

**DFC CpuBackend 现状（问题）**：
- `float gridCache[N_INTERP][49][5][5]`（846）、`int64_t gridKey[N_INTERP]`（847）、`std::vector<float> splitCoord`（1955）全部是**实例成员**——多线程 fill 时**共享可变 buffer**，并发污染。
- `buildInterpGrid` 动 `splitCoord`（`assign`/`swap(saved)`）→ 非线程安全。
- 若 `fillOneChunkCore` 多线程并行 fill 不同 chunk，各线程会互相踩 `splitCoord` / gridCache / gridKey。

**设计**：
1. **gridCache/gridKey** → `thread_local`。形态：`thread_local` per-interp 数组，按 interpIdx + chunkKey 索引。可选方案：
   - 方案 i（对齐 production）：`static thread_local std::vector<GridSlot> slots;`（`GridSlot{ int64_t key; float grid[49][5][5]; int edgeCX,edgeCZ; float edgeCol[49][5]; }`），`slots.resize(N_INTERP)`，`GridSlot& s = slots[interpIdx]`。**结构与 production 完全同构**，最干净。
   - 方案 ii（微省）：`thread_local float gridCache[N_INTERP][49][5][5]; thread_local int64_t gridKey[N_INTERP];`。
2. **splitCoord** → `thread_local`。`buildInterpGrid`/`split()`/`eval_df` 读写的 `splitCoord` 必须 per-thread；否则 buildInterpGrid 的 `splitCoord.swap(saved)` 与采样线程冲突。**这是并发安全的关键**——production 的 `arg->sample` 无共享 split 缓冲，故天然安全；DFC 因 split 缓冲共享必须显式补。
3. **static 成员加 `inline`**（multi-TU LNK2005 教训，AGENTS.md 五 §8）——若引入 `static thread_local` 成员，须 `inline`。

**风险**：per-thread 的 splitCoord/gridCache 内存 = 每线程 × (8672×4B + 5×49×5×5×4B + ...) ≈ 每线程 ~（34.7KB + 24.5KB）≈ 60KB。多线程开销可接受，但需在无探针 wall 下验证（AGENTS.md 测量污染铁律：并发只信「无探针整批 wall + 调用次数计数」）。

---

## 5. 建议执行顺序（收益/风险比）

> 每步独立可验证、可回滚。**先做低风险高收益，后做高风险。**

| 步 | 优化 | 改动面 | 收益 | 风险 | 验证 |
|---|---|---|---|---|---|
| **1** | **thread_local 化 splitCoord + gridCache/gridKey**（§4） | CpuBackend 成员 → static thread_local（+inline）| 解锁并发（消除 11× 的前提）| 低（纯存储迁移，不改数值）| `dfc_cpp_vs_prod` 单线程 maxdiff 不变（9.57e-07）；多线程 wall 对比 |
| **2** | **edgeCol 跨 chunk 复用（§3）** | cpu_backend.h `buildInterpGrid` 加 edgeCX/edgeCZ/edgeCol | 省 20% 节点构建 + 与 production 公平口径 | 低（复用条件加左邻标记，copy 原始值）| `dfc_cpp_vs_prod` + gridKey 命中计数（edgeCol 命中 >0）|
| **3** | **per-cell split 去重（方案 a：纯 C++ 按 cell 存 1 份）**（§2.4 A 方案 a） | 只改 `buildInterpGrid`：先按 cell 算 split 存 per-cell 槽，再逐节点读 | split() 1225→768 + 消角点共享 8× | 中（存储布局 + 外层 split 还原需保）| 重跑三张验证表（§9.3 基线）|
| **4** | **生成器 split 布局翻转（方案 b：split_total_cell + 不展开 8 角点）**（§2.4 A 方案 b + B） | `_gen_split_lines` interpolated 分支 + `splitTotal` + `_compute_val_layout` | 消除跨 interp 5× + 整树浪费（最优）| **高**（牵连 perSample/bases/val_slots；须与 shader PER_SAMPLE 同步）| 先在 Python 模拟层（dbg_full_sim）验证逐位，再改 C++ |
| **5** | **重生成 + 编译 + benchmark** | 主会话 | 量测实际收益 | — | `dfc_cpp_verif` + 无探针整批 wall + split() 调用计数 |

**推荐**：先 1 → 2（低风险，把「并发安全 + 跨 chunk 复用」落地，即可接入 fillOneChunkCore 测 11×），再 3（per-cell，中风险，把纯浪费砍掉），最后 4（生成器翻转，高风险，需 Python 模拟层前置验证）。**4 应作为独立 task 而非 3 的延伸**。

---

## 6. 风险清单

1. **split 组织翻转破坏对齐（最高风险）**：`_gen_split_lines` interpolated 分支改 cell 粒度 + `splitTotal` 布局翻转，会重排 `NOISE_SLOT_BASE`/`NORMAL_PACK`/`perSample`/`bases` → **若不逐位验证，会静默破坏 DFC 对齐**。§8.4 已证「grid 节点必须用其 cell 的 sIdx」是正确性来源，稍微改动 sIdx 绑定即错。**必须**：Python 模拟层（dbg_full_sim）先验证「cell 一份 split + eval_df_base」逐位 == interp_N，再动 C++。
2. **corner 映射回退陷阱**：改 split 组织时，**不可**改用「floorDiv corner 映射」（§9.2 已判其不自洽，会读相邻角点 split 破坏对齐）。任何改动须保持「节点=cell 的 (0,0,0) 角点 → corner=0」。
3. **thread_local 的 CpuBackend 改动**：`splitCoord`/`gridCache`/`gridKey` 从实例成员 → thread_local（+ inline static），可能引入 multi-TU 链接问题（LNK2005 教训）；且每线程内存 ~60KB。
4. **minY/height 硬化（未做）**：`minY = -64`（cpu_backend.h L34），`height` 未显式（grid 49 行断言 overworld 384）。若 nether（minY=0/height=128）或其它维度 DF 进入 DFC，grid 尺寸/ny 映射全错。当前 grid 缓存**只验证 overworld**。**风险**：接 multi-dimension 时需把 minY/height 参数化（对齐 production InterpolatedDF 的 minY/height 成员），并把 grid 维度改为运行时算（GX=16/CELL_X+1、GY=height/CELL_Y+1、GZ=16/CELL_Z+1）。当前硬编码 49/5/5 在非 overworld 下越界。
5. **edgeCol 正确性**：复用条件必须 `edgeCX == chunkX-1 && edgeCZ == chunkZ`（严格左邻）+ gridKey 未命中，否则退全建。若 fill 顺序非「x 递增」则命中率低，但**不破坏正确性**（只是少省）。
6. **测量污染（科学口径）**：并发性能只信「无探针整批 wall + 调用次数计数」（AGENTS.md 测量污染铁律）；不用 WG_PROFILE/WG_STAGETIMER 的耗时列；可靠工具 = WG_PHASETICK。

---

## 7. 结论（candidate）

- **瓶颈 = split()` 的冗余放大**（整树 5 interp × 8 角点 + 跨节点角点共享 8×），非 eval_df_base。占 buildInterpGrid 成本 ~80-90%。
- **优化路径**：thread_local（§4，先做，解并发）→ edgeCol（§3，省 20% 且对齐 production）→ per-cell split 去重（§2，方案 a 中风险 / 方案 b 高风险需模拟层前置）。
- **正确性红线**：grid 节点必须用「其 cell 的 split + corner=0」，不可回退全局 sIdx 或 floorDiv corner 映射；每步重跑三张验证表。
- **未运行**：量化数值为源码级估算，实际增益（split 占比、11× 消除、每步 wall）须由主会话在步骤 5 benchmark 复核。

> 待 main 会话复核：是否把方案 b（生成器翻转）作为独立 task；是否引入 minY/height 参数化（多维度）；per-cell split 槽布局细节。
