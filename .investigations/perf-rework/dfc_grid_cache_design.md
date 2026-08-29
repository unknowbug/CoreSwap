# DFC「cell grid 缓存」设计依据（仿 production InterpolatedDF）

> 作者角色：勘探 worker（只读，未编译未改码）。
> 目的：为「DFC C++（CpuBackend，gen_cpu 生成）加 cell grid 缓存，仿 production `InterpolatedDF`」产出设计依据，供后续 C++ 实现选型。
> 已确证前提：DFC C++ 采样函数与 production 逐位对齐（maxdiff 6.52e-07）；DFC 当前 interp 是**每点重算 8 角点（无缓存）**。
> 结论核心：**DFC 加 grid 缓存 = 把「每点 8 角点」重构为「buildGrid（存 5×49×5 角点/实例）+ sampleGrid（三线性）」**，与 production `InterpolatedDF` 结构一一对应；关键难点不在插值公式（已逐位一致），而在 **DFC 的 split-precompute 数据模型是「按点（sIdx）+ 8 角点展开」，需改造为「按网格节点」**。

---

## 1. production `InterpolatedDF` 的 grid 缓存结构（density.h L482-620，事实）

### 1.1 cell 尺寸 / grid 尺寸

| 量 | 值 | 推导 |
|----|----|------|
| `CELL_X` | **4** | `static constexpr int CELL_X = 4`（L485）|
| `CELL_Y` | **8** | `static constexpr int CELL_Y = 8`（L485）|
| `CELL_Z` | **4** | `static constexpr int CELL_Z = 4`（L485）|
| `GX` | 5 | `16 / CELL_X + 1 = 16/4+1 = 5`（L511/L590）|
| `GY` | **49** | `height / CELL_Y + 1 = 384/8+1 = 49`（L511/L590）|
| `GZ` | 5 | `16 / CELL_Z + 1 = 16/4+1 = 5`（L511/L590）|
| grid 元素数 | **1225** | `5×49×5 = 1225` 个 double |
| chunk 角点数（含跨 chunk 复用） | 1225 | 每实例每 chunk 一次 buildGrid |

⚠️ **注意**：任务描述写「4×4×8」，但代码实际是 **CELL_X=4, CELL_Y=8, CELL_Z=4**（x/z=4、y=8）。这是 vanilla `DensityInterpolator` 的事实，DFC 的 interp_N 与之同构（`cx=gx/4, cy=gy/8, cz=gz/4`）。cell 网格：xz 平面 4×4、y 方向 8（垂直方向更密的网格 = 垂直噪声频率更高时保持采样精度）。

`minY = -64`（overworld 基准，L486 默认），`height = 384`（overworld，L487 默认，`GY=49` 由此来）。

### 1.2 ThreadLocal 缓存结构

- **per-instance per-thread**：`static std::vector<Slot>& tlSlots()`（L576-579）。每线程一个 `std::vector<Slot>`，按 **cacheId**（实例 id，构造时 `nextId.fetch_add(1)`）索引，`slot = slots[cacheId]`（L505）。
- `instanceCount` 静态原子（L575），构造后固定（wg_create 单线程构建）；`sample` 里 `if (slots.size() < instanceCount) slots.resize(...)`（L504）——**一次性扩到实例总数，buildGrid 内不再 resize，外层 slot 引用不悬垂**（L502 注释）。
- `Slot`（L565-572）：
  - `int64_t key`（初始 `INT64_MIN`）——当前 chunk 键。
  - `std::vector<double> grid` —— 5×49×5 角点值。
  - `int edgeCX/edgeCZ` + `std::vector<double> edgeCol` —— 边界列复用（见 1.4）。
- chunk 键（L500）：`key = ((int64_t)((uint64_t)(uint32_t)chunkX << 32)) ^ (uint32_t)chunkZ`；`chunkX = floorDivP(pos.x,16)`，`chunkZ = floorDivP(pos.z,16)`（L498-499，floor 语义负坐标）。
- **命中/失效**：`if (slot.key != key) { slot.key = key; buildGrid(...); }`（L506-510）。命中即直接读 grid；miss 才 buildGrid。每线程每实例每 chunk 最多 buildGrid 一次。

### 1.3 buildGrid 角点坐标公式（L589-619）

```
for gy in [0, GY):
  for gz in [0, GZ):
    for gx in [0, GX):
      if (gx==0 && reuseLeft): grid[idx] = edgeCol[gy*GZ+gz]; continue;
      p.x = chunkX*16 + gx*CELL_X;      // chunkX*16 + gx*4
      p.y = minY    + gy*CELL_Y;        // minY    + gy*8
      p.z = chunkZ*16 + gz*CELL_Z;      // chunkZ*16 + gz*4
      grid[idx] = arg->sample(p);       // delegate 实采样
```
其中 `idx = (gy*GZ + gz)*GX + gx`。**每个 grid 节点 = delegate 在该节点世界坐标的实采样值**（`InterpolatedDF::arg->sample(p)`），无「角点」概念（见 §2.4 与 DFC 的差异）。

`wg_profInterpGrid` 计数（L509）即 buildGrid 触发次数（phase0 实测约 **6.6/次/点**，每实例每 chunk 一次）。

### 1.4 边界列跨 chunk 复用（edgeCol，无损）

- 几何事实：左邻 chunk `(chunkX-1,chunkZ)` 的 **gx=4 列**（`(chunkX-1)*16 + 5*4 = chunkX*16`）与当前 chunk 的 **gx=0 列** x 坐标相同 → 采样值逐位相同（L568-569 注释）。
- 复用条件：`slot.edgeCX == chunkX-1 && slot.edgeCZ == chunkZ`（L596）。
- 当前 chunk 构建后，把自身 **gx=GX-1 列** 存入 `edgeCol`（L614-618），供右邻 chunk 复用其 gx=0 列。
- 无损性：同一列同世界坐标 → delegate 值相同 → 逐位一致（`review-aae119d` 已判语义无损成立）。
- 节省量：x 方向边界列共享比例 ≈ `(5+5-1)/(5×5) = 36%`（phase0-quantify），× 每实例每 chunk。

### 1.5 sample 三线性插值链（L534-548）

```
fx = (gx%CELL_X)/CELL_X; fy = (gy%CELL_Y)/CELL_Y; fz = (gz%CELL_Z)/CELL_Z;   // gx..gz 为 chunk 内坐标
g(dx,dy,dz) = grid[((cy+dy)*GZ + (cz+dz))*GX + (cx+dx)]                      // cy/cz/cx 为 cell 索引
d000=g(0,0,0) d100=g(1,0,0) d010=g(0,1,0) d110=g(1,1,0)
d001=g(0,0,1) d101=g(1,0,1) d011=g(0,1,1) d111=g(1,1,1)
d00=d000+(d100-d000)*fx; d10=d010+(d110-d010)*fx;
d01=d001+(d101-d001)*fx; d11=d011+(d111-d011)*fx;      // x 方向
d0 =d00+(d10-d00)*fy;  d1 =d01+(d11-d01)*fy;            // y 方向
rr =d0 +(d1 -d0 )*fz;                                    // z 方向
```
越界保护（L517-522）：`cx/cy/cz` 越界时 clamp 到 `GX-2/GY-2/GZ-2`（POC 与 Java `DensityInterpolator` 直接采样略有差异，稳定性优先，见 L516 注释）。

### 1.6 buildGrid 触发时机

**每 chunk 每实例一次**——某实例在某 chunk 的首次采样时触发（miss）。但注意：**每个 InterpolatedDF 是独立实例、独立 cacheId、独立 grid**，所以同一 chunk 若有 K 个 InterpolatedDF，则 buildGrid 触发 K 次（每实例一次）。finalDensity 树有 5 个 InterpolatedDF → 每 chunk 5× buildGrid（每实例 1225 节点采样）。生产 buildGrid 的 `wg_profInterpGrid=238 ≈ 6.6/chunk` 是单线程 POC 测量值。

---

## 2. DFC 的 interp 结构与 production 的对应（关键问题 1/2）

### 2.1 DFC 有 5 个 interp（`interp_roots`），逐项对应

实验测定（`DfcGen.gen_df(overworld.json noise_router.final_density)`，df_nodes=163，interp_roots=[134,140,148,151,155]）：

| DFC interp | 节点类型 | delegate 根 | delegate 闭包长度 | 对应 production 的 InterpolatedDF |
|-----------|---------|------------|------------------|----------------------------------|
| `interp_0` | root=134 `BLEND_DENSITY` | final_density 的**顶层 interpolated**，argument = `minecraft:blend_density`（sloped_cheese / 内容树） | 134 节点（含 SPLINE/NOISE/OLD_BLENDED/WEIRD/算术全谱） | **final_density 顶层 InterpolatedDF**（内容树 = 整棵 sloped_cheese） |
| `interp_1` | root=140 `RANGE_CHOICE` | noodle/cave 插值 1 | 21 节点 | cave/noodle 的 `minecraft:interpolated` #1 |
| `interp_2` | root=148 `RANGE_CHOICE` | noodle/cave 插值 2 | 20 节点 | cave/noodle #2 |
| `interp_3` | root=151 `RANGE_CHOICE` | noodle/cave 插值 3 | 17 节点 | cave/noodle #3 |
| `interp_4` | root=155 `RANGE_CHOICE` | noodle/cave 插值 4 | 18 节点 | cave/noodle #4 |

**关键结论（回答「production finalDensity 树有几个 InterpolatedDF？」）**：

- **DFC 的 5 个 interp 全部对应 production finalDensity 树里的 5 个 InterpolatedDF**（`JSON` 采样证实：final_density 顶层类型是 `minecraft:min`，内含 1 个**内联** interpolated（`fd.argument1.argument.argument2`，arg=blend_density）；其余 4 个来自 **registry 引用**（`resolve_ref` 展开的 cave/noodle 函数，每个都是 interpolated））。
- 上述 JSON 直接走 `walk` 只数到 1（未展开 registry 引用）；展开后总数 = **5**（static-audit-c2me-steel L67 并列证据：`final_density 1 + caves/noodle 4 = 5`）。
- ⚠️ **生产构建计数 `InterpolatedDF instances=6` 与 DFC 5 的差异**（static-audit L68）：生产 `density_builder.h` 对每个 interpolated **字面量**机械构建（**无去重**），同一内容被多次引用会构建多份实例；DFC 的 `_df_node`/`_noise_slot` **按结构去重** → DFC 合并为 5 个唯一 interp。即：生产 6 份中至少有 1 份与其余 5 份内容重复（或属非 finalDensity 组件）。**对 grid 缓存无语义影响**（重复内容共享同一 delegate 结构，缓存按唯一内容建一份即可，反而少算一次）。

### 2.2 树内 interp 有几层？——**全部在顶层闭包，无嵌套**

实验证实（对每个 interp 求闭包）：

- 每个 interp 的 delegate 闭包 `nested_interp=[]`（**所有 delegate 树都不含 DF_INTERP**）。
- 顶层闭包（21 节点）包含全部 5 个 DF_INTERP 节点（索引 135,141,149,152,156，a1=0,1,2,3,4）。

即：**interp 节点只出现在 finalDensity 树的最外层（顶层闭包），5 个互不嵌套**——`eval_df_base`（DFC）因此可以安全地**不含 DF_INTERP 分支**（与 production「interpolated 的 delegate 不会再是 interpolated」语义一致；mapping 文档 L80/L83 同述）。这个性质是 grid 缓存设计成立的前提：`eval_df_base` 是完整的 delegate 求值器，可直接用于每个 grid 节点的 val。

### 2.3 每个 InterpolatedDF 的 cell 尺寸是否相同？——**完全相同（4×8×4）**

- production：**单一 `InterpolatedDF` class，`CELL_X/CELL_Y/CELL_Z` 是 `static constexpr`**（L485）→ **所有实例共享同一 cell 尺寸**，不存在「树内不同 interp 不同 cell」。
- DFC：5 个 interp 全部用 `cx=gx/4, cy=gy/8, cz=gz/4`（L2269）→ 同为 4×8×4。

**→ grid 缓存网格尺寸对所有 interp 统一（5×49×5），无需按 interp 区分 cell；复杂度不因 interp 数放大。**

### 2.4 关键差异：production grid 值 vs DFC interp 角点值的「角点语义」

- **production**：`grid[i] = arg->sample(p)` —— delegate 直接在节点世界坐标 p 采样。delegate 里的噪声是**单一实例**（无角点概念），节点值唯一、与「从哪个 cell 看它」无关。
- **DFC（当前）**：`interp_N` 的 8 角点 `d[c] = eval_df_base(root, c, sIdx, ax, ay, az)`。`c`（corner 0..7）不仅传**角点世界坐标**（`ax=chunkX*16+(cx+dx)*4` 等），还作为**噪声实例选择**：`normal_noise(NOISE_SLOT_BASE[slot] + corner*NOISE_SLOT_STRIDE[slot], sIdx)`。`is_corner=True` 的 slot 有 **8 份连续噪声实例**（`base+c`，stride=1）——**这就是 DFC grid 缓存的核心冲突点**（见 §3.3）。

---

## 3. DFC grid 缓存设计

### 3.1 结构映射（与 production 一一对应）

| production | DFC 对应（建议） |
|-----------|-----------------|
| `Slot.grid`（5×49×5 值/实例） | 每 interp 一份 `float grid[5][49][5]`（thread_local，按 interp idx 索引），或直接复用 `Slot` 模式 |
| `Slot.key`（chunk 键） | 每 interp 一份 chunk 键（同 `((chunkX<<32)^chunkZ)`；DFC 的 `chunkX=floorDiv(ix,16)`） |
| `buildGrid(chunkX,chunkZ,grid)` | DFC `buildInterpGrid(interpIdx, chunkX, chunkZ, grid)`：对 5×49×5 节点，逐节点 `eval_df_base(root, c, sIdx, node_x,node_y,node_z)` |
| `sample` 三线性 | DFC `sampleInterpGrid(interpIdx, ix,iy,iz)`：`fx=(gx%4)/4, fy=(gy%8)/8, fz=(gz%4)/4` + 三次 lerp |
| `edgeCX/edgeCZ/edgeCol` 跨 chunk 复用 | 可移植（每 interp 存 gx=4 列，供左邻 chunk gx=0 列复用）|
| `cacheId`/`tlSlots()` | DFC 用 interp idx（0..4）替代，或沿用 thread_local vector |

### 3.2 buildGrid 怎么算（核心步骤）

对 interp `i`，每 chunk：
```
for gy in [0,49): for gz in [0,5): for gx in [0,5):
  node = (chunkX*16 + gx*4, minY + gy*8, chunkZ*16 + gz*4)     // 与 production buildGrid 角点公式一致
  grid_i[gy][gz][gx] = eval_df_base(interp_root[i], <corner>, node_x, node_y, node_z)
```
- `eval_df_base` = **内容树（无 interp）**求值器，直接用（已在 CpuBackend 生成 L2231）。这正对应 production 的 `arg->sample(p)`（delegate = 内容树）。
- 结果 grid_i 再在 `sampleInterpGrid` 里做三线性 → 逐位等价于 production（前提：DFC 已逐位对齐）。

### 3.3 ⚠️ 核心难点：split-precompute 的「按点」模型 vs 「按网格节点」

这是 DFC grid 缓存与 production 的**本质差异**，也是最大风险点。

- **production**：`arg->sample(p)` 直接用真实噪声对象（`DoublePerlinNoiseSampler` 等）在 p 采样，**无 split 中间层**。
- **DFC**：`eval_df_base` 里的噪声读的是 **split-precompute 缓冲**：`normal_noise(NOISE_SLOT_BASE[slot]+corner*STRIDE, sIdx)` → `int b = sIdx*splitTotal + splitBase + i*6; read splitCoord[b..]`（L2072）。**split 数据是按「采样点 sIdx」组织的**，每个采样点 sIdx 有 `splitTotal`(8672) floats（D24 实测）。
- DFC 当前 `split(x,y,z,out)`（`_gen_split_lines` L1541-1631 + `split` L1837）**对 interpolated 做 8 角点展开**（L1599-1613）：它算 `(x,y,z)` 所在 cell 的 8 个角点，对每个角点 c 以 `noise_key_suffix="@c{c}"` 递归生成 delegate 的 split 行——即 split 缓冲里按 `base+c`（8 份实例）存的是**该 block 所在 cell 的 8 个角点**的 split 坐标。

**结论**：DFC 现在的 split 数据流是「**每 block → 该 block 所在 cell 的 8 个角点 split**」。这正好是**每 block 重算 8 角点**的根因（也是 task 背景说的「DFC 每点重算」）。要加 grid 缓存，就必须把 split 数据的组织**从「按 block（带 8 角点展开）」翻转为「按网格节点」**。

**grid 缓存下，buildInterpGrid 需要**：对每个 grid 节点，`eval_df_base` 在该节点坐标下的 delegate 值。这要求 split 缓冲里**有该节点坐标的 delegate 噪声 split 数据**。当前 split() 做不到（它展开的是「调用的那个位置的 cell 角点」，不是「节点本身」）。

### 3.4 三条实现路径（复杂度递增 / 保真度递增）

| 路径 | 做法 | 保真度 | 工作量与风险 |
|------|------|--------|-------------|
| **A. 逐 cell 8 角点值缓存（最简，最小改动）** | `interp_N` 内缓存「最近一个 cell 的 8 角点值」：命中（同 cell）直接复用，miss 才算 8 角点。cell = 4×8×4 = 128 block，同 cell 的 128 block 共享同一 8 角点 → 8 角点重算次数 ≈ 每 chunk 768 cell 而非 98304 point。 | **非 production 完全对应**（不跨 cell 去重角点；每 cell 仍算 8 次而非去重的网格节点）| 低：只改 interp_N 内部 + 每 interp 一组 8 值缓存。**但 split() 仍每 block 重算**（见下）→ 只能省 `eval_df_base` 的 delegate 求值，省不了 split 预处理 |
| **B. 逐 cell split+角点缓存（折中）** | 缓存**某 cell 的 8 角点 split 数据 + 8 角点值**（每 interp 一份当前 cell 缓存）→ 同 cell 的 128 block 复用 split + 角点值。 | 较接近；仍不跨 cell 去重（768 cell × 8 = 6144 次角点 vs production 1225）| 中：把 `split()` 的 8 角点展开结果和 `eval_df_base` 结果一起缓存到「当前 cell」粒度；需处理 cell 边界（cell key） |
| **C. 全 chunk 网格缓存（最贴 production，推荐终极形态）** | 每 interp 每 chunk 一次性建 5×49×5 **去重**网格（1225 节点），随后每 block O(1) 三线性。需**重构 split 为「按网格节点预 Compute」**：新增 `buildInterpGrid` 前先对每个网格节点算该 interp delegate 的 split（node 坐标），再 `eval_df_base` 建网格。 | **production 完全对应**（1225 节点去重 + edgeCol 跨 chunk 复用）| **高**：需改生成器（`_gen_split_lines` 增「网格模式」：interpolated 不展开 8 角点，而是按节点坐标生成 delegate split）+ CpuBackend 增 grid 成员 + `buildInterpGrid`/`sampleInterpGrid` + 生命周期/thread_local |

**推荐路线**：先用 **B（逐 cell）** 拿一个「网格缓存存在」的公平对拍版本（改动小、风险可控），再视性能缺口上 **C（全 chunk，production 完全对应）**。若目标是「与 production 逐位 + 同构的性能对比」，最终必须落 **C**（B 仍比 production 多算 ~5× 角点，会低估 DFC 消除 spline 指针追逐的价值）。

### 3.5 跨 chunk 复用（edgeCol）是否也做？

- **建议做**（production 有，且 phase0-quantify 实测共享比例 36%）。对 DFC，若走 C，gridi 的 `gx=4` 列存到 `edgeCol[i]`（每 interp 一份 `[GY*GZ]=[49*5]=245`），当前 chunk 的 `gx=0` 列复用左邻。
- **无损性**成立：同一列世界坐标 → `eval_df_base` 值逐位相同（坐标决定一切，无角点依赖——见 §3.3 第 1 点的「节点值唯一」）。
- **若不先做 edgeCol**：每个 chunk 仍 1225 节点全算，跨 chunk 没有重复利用但也没有错误；仅是每次多算 245×2（x/z 边界列不到 1225，实际 ~36%）。初版可先不做，性能对比时注明「未启用 edgeCol，DFC 仍多算 36% 角点」——但为了与 production 公平对比，建议启用。

---

## 4. 关键数据汇总

| 项 | production | DFC（当前） | DFC 加 grid 缓存后（目标） |
|----|-----------|------------|--------------------------|
| interp 数量 | 5（finalDensity 树内；构建 count≈6 因无去重重复/late-vein） | 5 | 5 |
| cell 尺寸 | 4×8×4（统一 static const） | 4×8×4 | 4×8×4 |
| grid 尺寸 | 5×49×5 = 1225 节点/实例 | 无 grid（每点算 8 角点） | 5×49×5 = 1225 节点/实例 |
| 每 chunk 角点采样（delegate eval） | 实例数 × 1225 = 5×1225 = 6125 | 98304 点 × 8 角点 × 相关 interp 路径 | 5×1225 = 6125（+edgeCol 复用后更少）|
| 角点去重（跨 cell 共享） | 有（网格结构天然去重） | 无 | 有（网格结构）|
| 跨 chunk 边界复用 | 有（edgeCol gx=4 列，~36%）| 无 | 可选（建议启用）|
| 缓存载体 | 每实例每线程 `Slot`（thread_local，cacheId 索引）| 无 | 每 interp 每线程 grid（thread_local）|
| 插值 | 每次 `sample`：8 角点读 + 3 lerp | 每次 `interp_N`：8 次 delegate 求值（重） | 每次 `sampleInterpGrid`：grid 读 + 3 lerp |
| **split 数据组织** | 无 split（直接噪声采样）| **按点（sIdx）+ 8 角点展开** | **需按网格节点**（改造点）|

---

## 5. 风险 / 难点（重构点清单）

1. **split-precompute 数据模型翻转**（最大难点，见 §3.3）：`_gen_split_lines` 的 interpolated 分支（L1599-1613）当前做 8 角点展开；grid 缓存需要**按网格节点**生成 delegate 噪声的 split。需新增生成模式或重构该分支 → `splitTotal` / `NOISE_SLOT_*` 布局随之变化，`perSample`、`base[]`、`valBuf` 分配都要重算（`_compute_val_layout` L414-483）。**风险高**（牵连面广），建议先在 Python 模拟层（`dbg_full_sim.py`）验证新 split 组织与逐位一致性，再动 C++。
2. **DFC interp 角点的「实例选择」语义处理**：$c$ 既传坐标又选噪声实例。grid 节点值必须**唯一**（同 production）。正确性取决于一个前提：**同一 grid 节点，无论从哪个 cell、哪个 corner 索引看，`eval_df_base` 的值都相同**——因为 8 份实例参数相同（`dict(params)` 复制）、且在同一节点坐标下 split 数据应一致。**必须验证此前提**（grid 节点值唯一性），否则需按「节点坐标」固定选取一个实例并锁死（否则 grid 缓存会破坏逐位对齐——这正是「角点值 ≠ finalDensity(角点坐标)」教训 D23 所在）。
3. **grid 生命周期 / 线程安全**：DFC `CpuBackend` 是**实例内共享**结构；grid 缓存必须**thread_local**（与 production 一致）。注意 `CpuBackend` 现有 `splitCoord` 成员是**非 thread_local 的可变 shared buffer**（L1953/L2326 `splitCoord.assign`）——**若多线程 fill，grid 缓存与 splitCoord 都要 per-thread**，否则并发污染（这与 production 的 thread_local slot 设计不同，需在 DFC 侧显式补）。参考 `i-integration-record` 的「multi-TU LNK2005」「PIMPL」教训——static 成员加 `inline`。
4. **`minY`/`height` 通用化**：DFC `minY=-64`、`height=384` 目前写死（L1757/L2539）。若 future 支持 nether（minY=0,height=128）等维度，grid `GY=49/17` 不同——生成器需把 `minY/height` 作为参数，或至少注释清楚「当前仅 overworld 硬化，与 production 一样按维度构造」。
5. **性能对比口径**：除非用路径 C（+edgeCol），否则 DFC 实测仍会**多算**（B 多 5× 角点；无 edgeCol 多 36%）→ 会低估 DFC 消除 spline 指针追逐的 11× 价值。**公平对比必须用路径 C + edgeCol**。
6. **越界 clamp 差异**：production sample 有越界 clamp（L517-522，与 Java 直接采样略异）；DFC `interp_N` 没有。grid 缓存若照搬 production 语义，需决定**是否保留 clamp**（保留则维持「已对齐 production 行为」；去掉则更贴 Java 但需重验）。
7. **验证覆盖**：D23 教训（gpu-accel-errors L340）——验证域必须**跨 chunk / 跨 cell（cy≥1）**，不能只信单一小域（`x≤63,y∈[-64,-49],z≤4`）。grid 缓存后必须用 block_probe 在多 chunk、多 cell、负坐标、边界列上对拍 production。

---

## 6. 关键问题显式回答

**Q1. production finalDensity 树有几个 InterpolatedDF？** → **5**（DFC `interp_roots=5`，interp_0=顶层 blend_density 内容树，interp_1..4=cave/noodle）。生产构建打印 `instances=6` 是**无去重 + 重复引用/late-vein** 的统计口径差异，非语义差异。**5 个 DFC interp 全部对应 production 树里的 interp 节点**（2.2 证实无嵌套、全在顶层闭包）。

**Q2. 每个 InterpolatedDF 的 cell 尺寸是否相同（4×8×4）？DFC 5 个 interp 是否各自不同 cell？** → **全部相同**：production 是单一 class 的 `static constexpr CELL_X=4, CELL_Y=8, CELL_Z=4`；DFC 5 个 interp 全部 `cx=gx/4, cy=gy/8, cz=gz/4`。**→ grid 网格对所有 interp 统一 5×49×5，复杂度不随 interp 放大。**

**Q3. DFC 加 grid 缓存的最佳形态？** → **路径 C（全 chunk 网格缓存，逐位对应 production：每 interp 每 chunk 建 5×49×5 去重网格 + 三线性 + edgeCol 跨 chunk 复用），用 `eval_df_base`（内容树，无 interp）算各节点角点**。作为降低风险的过渡，先做路径 B（逐 cell 缓存 8 角点值 + split）。路径 C 是「公平性能对比」的必要条件（否则 DFC 仍多算，低估其消除 spline 指针追逐的价值）。

**Q4. DFC 加 grid 缓存的关键决策点 + 复杂度评估？** → 见 §3.4 三路对比 + §5 风险清单。**复杂度：中→高**（主要在 split-precompute 数据模型翻转 + grid 生命周期/线程安全 + 逐位重验证），插值公式本身零改动（已对齐）。

---

## 7. 附：证据 / 产物对照

- production `InterpolatedDF`：`versions/1.20.1/cpp/worldgen/src/density.h` L482-620（`sample` L497-558，`buildGrid` L589-619，`Slot`/`tlSlots`/`edgeCol` L563-619）。
- DFC 生成器：`.investigations/perf-rework/dfc_gen.py`（`_df_interp_node` L385-411，`eval_df_base` L2231-2265，`interp_N` L2266-2284，`_gen_split_lines` interpolated 分支 L1599-1613，`split` L1837-1839）。
- DFC 映射表：`.investigations/perf-rework/dfc_cpu_mapping.md` §2.3/§2.9（interp 8 角点 + eval_df_base 无 interp）。
- DFC Python 参考：`.investigations/perf-rework/dbg_full_sim.py` L349-372（`interp_N`）。
- 生产 interp 计数/网格：`.investigations/perf-rework/static-audit-c2me-steel.md` L64-68；`phase0-quantify.md` L20；`phase0-interp-measurement.md`。
- split 数据量（8672 floats/点）：`.investigations/perf-rework/gpu-accel-errors.md` D24（L598/L608）。
- 验证域教训（D23）：`.investigations/perf-rework/gpu-accel-errors.md` L340（跨 chunk/cell 覆盖）。

> 状态：**draft**（本 worker 勘探产出，未编译未改码；「grid 节点值唯一性」与「split 翻转后逐位一致」两项待 Phase 2 验证，需与 production 对拍确认）。

---

## 8. 2026-08-23 追加：split 布局设计关键发现（design_split_grid.py，主会话分析）

> 背景：路径 C 的 split 翻转核心矛盾是「grid 1225 节点若各占 splitTotal 会爆量」。本分析用 `split_dump.bin`（1024 采样点 × 8672）实测 split 布局，得出**关键设计**：**split 组织应从「按采样点 sIdx」翻转为「按 cell」**。

### 8.1 实测数据（design_split_grid.py 输出）

| 项 | 值 | 含义 |
|----|----|------|
| 采样点数 | 1024 | split_dump 样本数 |
| **不同 cell 数** | **32** | 1024 点映射到 32 个 cell |
| 每 cell 采样点 | 32（全部均匀） | cell = 4×8×4，128 block/cell |
| **同一 cell 的 32 采样点 split 段** | **完全相同**（8/8 全部） | **cell 内 split 冗余：同一 cell 的 32 点重复存同样 split** |
| 不同 cell 的 split 段差异 | ~26% 不同（2268/8672） | cell 间需局部独立，但大部分共享（flat_cache/共享噪声） |

### 8.2 关键设计结论

1. **split 布局冗余巨大**：当前「每采样点一份 splitTotal」= 8672 × 1024 = 888 万 floats；但实际只有 **32 个不同 cell** 的有效 split（32 × 8672 = 28 万 floats）。**每个 cell 的 128 block 共享同一 split**。
2. **split 翻转的正确定义**：把 split 组织从「**按采样点 sIdx**」改为「**按 cell**」——每个 cell 一份 split（含该 cell 的 8 角点展开），供该 cell 的 128 block 复用 + 8 角点求值。这与 production buildGrid「每 chunk 每实例建网格（节点=cell 角点）」对应。
3. **grid 缓存实现**：每 chunk 每 interp 建网格（1225 节点），节点的 delegate 值用「该 cell 的 split + 节点 corner 索引」经 `eval_df_base` 求值；128 block 三线性共享。**不再需要每 grid 节点一份 split**（cell 内共享）。
4. **复杂度大幅降低**：split 翻转从「1225 节点各 splitTotal 爆量」→「每 cell 一份 split（32 cell/chunk 量级）」。牵连仍涉及 `_gen_split_lines`（改 cell 粒度）+ splitTotal 布局（从 sIdx 到 cell 索引）+ `_compute_val_layout`，但**可行性明确**。

### 8.3 下一步（Phase 4a-2 实现前置）
- 用 Python 模拟验证「每 cell 一份 split」组织下，grid 缓存能否复现 production InterpolatedDF（逐位）——在 `dbg_full_sim` 层用「cell 一份 split + eval_df_base」对拍现有 interp_N。
- 确认后改生成器（`_gen_split_lines` interpolated 分支改 cell 粒度 + splitTotal 按 cell 组织）+ C++ `buildInterpGrid`/`sampleInterpGrid` + edgeCol。
- 分析脚本：`.investigations/perf-rework/design_split_grid.py`（可运行）。

### 8.4 2026-08-23 追加：grid 三线性验证结果 + sIdx 绑定关键认知（verif_grid_trilinear.py）

用 `dbg_full_sim` 验证「grid 缓存三线性（预存网格节点值）== interp_N（每点重算 8 角点）」：

| block | grid 三线性 | interp_N | diff |
|---|---|---|---|
| (0,-64,0) | 0.117187500 | 0.117187500 | **0** |
| (2,-60,2) | 0.121866318 | 0.121866318 | **0** |
| (4,-56,0) | 0.131783286 | 0.267819368 | 1.36e-01 |
| (7,-52,3) | 0.269803923 | 0.209757537 | 6.0e-02 |

**结论**：
1. **grid 缓存三线性结构正确**：cell 内（用对 sIdx）时 diff=**0**（block (0,-64,0)/(2,-60,2) 同一 cell）→ grid 预存节点值 + 三线性 = interp_N。
2. **跨 cell 不一致（diff>0）的根因 = sIdx 绑定**：grid 节点 (4,-56,0)/(7,-52,3) 属于**不同 cell**，但验证脚本对全 chunk 网格用了 **sIdx=0**（错误 split 位置）→ `eval_df_base` 读到错误 cell 的 split → 值错。**grid 节点必须用「其 cell 的 sIdx」求值**。
3. **这证实 split 翻转的精确含义**：**每个 grid 节点用「所在 cell 的一份 split」**（而非全局 sIdx）——即 split 从「按采样点 sIdx」翻转为「按 cell」，grid 节点用其 cell 的 split。**这是路径 C 关键正确性来源**，也解释了为什么不能简单「每点重算」或「全局 sIdx」。
4. **grid 节点值唯一性（verif_grid_cache_correctness）已证**——但唯一性是「同一节点从不同 cell/corner 看值相同」，需该节点是「各 cell 的角点」+「各 cell 用自己 sIdx」。我的验证脚本用 sIdx=0 全局 → 违反了这个前提，才出现跨 cell diff。

**修正验证方向**：正确验证 = grid 节点用「其 cell 的 sIdx」+「节点 corner」求值，然后三线性 vs interp_N。但现有 split_dump 是按采样点组织（非 cell 粒度）——**正确验证需先做 split 翻转（改生成器）**。此实验证实了「sIdx 绑定是 grid 缓存正确性关键」，为 split 翻转设计提供了精确约束（每 cell 一份 split，节点用其 cell）。

---

## 9. 2026-08-23：grid 缓存最小正确版实现 + 正确性验证通过（✅ 里程碑）

> phase-4a-2 worker（f5085be0）在 `dfc_gen.py` 的 `gen_cpu_sampling` 实现 path-C grid 缓存（最小正确版），主会话重生成 + 编译 + 验证。

### 9.1 实现（CpuBackend 新增）
- `float gridCache[N_INTERP][49][5][5];` + `int64_t gridKey[N_INTERP]`（INT64_MIN 哨兵）
- `buildInterpGrid(interpIdx, chunkX, chunkZ)`：对 gy/gz∈[0,5), gx∈[0,5) 的 5×49×5 网格节点，`split(nx,ny,nz)` 覆盖 splitCoord + `eval_df_base(root, 0, 0, nx,ny,nz)`（sIdx=0 读最新 split），存 gridCache；`splitCoord.swap(saved)` 还原
- `sampleInterpGrid(interpIdx, ix,iy,iz)`：gridKey 命中检查（不中则 buildInterpGrid）+ 5×49×5 三线性
- `interp_N` 开头：`if (sIdx == 0) return sampleInterpGrid(...)`（sIdx!=0 保留原 8 角点 batch 语义）

### 9.2 关键设计修正（worker 发现，重要）
**grid 节点是 cell 的 (0,0,0) 角点 → corner 恒为 0**。因节点落在 4/8 网格线，`split(nodeX,nodeY,nodeZ)` 对它选出的 cell 使节点恰为 (dx,dy,dz)=(0,0,0) 角点 → `eval_df_base(root, 0, 0, nx,ny,nz)` 读到节点自身 split → 正确（= production `arg->sample(nodePos)`）。原任务提示的 floorDiv corner 映射**不自洽**（gx=1 会读相邻角点 split → 破坏对齐），worker 用 corner=0 **已修正**。

### 9.3 正确性验证（✅ PASS）
| 验证 | 结果 |
|---|---|
| dfc_cpp_vs_prod（DFC 含 grid 缓存 vs production） | n=768（跨多 cell/chunk），**maxdiff=9.57e-07**，>1e-5:0，PASS |
| dfc_cpp_verif（grid 缓存 vs 蓝本） | maxdiff=**2.061e-08** PASS |

**结论**：grid 缓存（最小正确版）**不破坏逐位对齐**——DFC C++ 现在用 grid 缓存（每 interp 每 chunk 5×49×5 网格 + 三线性 + gridKey 命中），正确性保持（与 production 9.57e-07，与 GPU 蓝本 2.06e-08）。

### 9.4 已知局限（待优化，phase-4a-2 第二步）
- **慢**：buildInterpGrid 每节点调 `split()`（覆盖整个 splitCoord 8672 floats）——每 chunk 每 interp 1225 次 split 全量重算（纯浪费，节点只需它 cell 的 split）
- edgeCol 跨 chunk 复用未做（gridKey 触发全 chunk 重算）
- thread_local 未做（gridCache/gridKey/splitCoord 是实例成员，需 per-thread 并发安全）
- 优化设计 worker（38bdb543）进行中：量测性能瓶颈 + 设计 split 组织优化（每 cell 一份 split）+ edgeCol + thread_local
