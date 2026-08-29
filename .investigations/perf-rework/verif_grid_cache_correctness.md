# Path-C grid 缓存正确性验证（phase-4a-1，Python 模拟层）

> 角色：验证 worker（只写 Python 模拟验证脚本，未改 C++ / 未编译 / 未改 production 源码）。
> 验证对象：**路径 C（全 chunk grid 缓存）**的两大前置假设——① split 数据是否可按网格节点组织并复现 production `InterpolatedDF`；② grid 节点值是否唯一（单一节点值可缓存）。
> 验证载体：`dbg_full_sim.py`（与 GPU 逐位对齐蓝本，maxdiff 5.7e-9）的 `eval_df_base` / `NOISE_SLOT_BASE` / `NORMAL` / `perm` + `dfc_gen.py` 的 `g.df_nodes` / `g.noise_slots` / `g.spline_ssbo_nodes` / `g.interp_roots` + `vulkan-proto` 的 `split_dump.bin` / `perm_dump.bin` / `coords_dump.txt`。
> 脚本：`.investigations/perf-rework/verif_grid_cache_correctness.py`（可直接运行）。
> 状态：**candidate**（验证证据产出；`confirmed` 仍归人类）。

---

## 0. 结论先行

| 前置假设 | 结论 | 证据 |
|---|---|---|
| **grid 节点值唯一性**（同一节点从不同 cell / 不同 corner 实例看，`eval_df_base` 值逐位一致） | **成立** | 5 个 interp 全部 94 个共享节点 max\|diff\|=**0.0**（双精度严格相等，非阈值近似） |
| **8 份同参角点实例等价性**（`noise_key@c0..c7` 的 dict 拷贝是否同参同 perm） | **成立** | 25 个 is_corner slot 的 8 份实例参数 + perm 内容全部逐位相同 |
| **跨 chunk / 跨 cell 边界一致性**（`edgeCol` 复用前提） | **成立** | x=16/32/48 边界 18 个共享节点、y=-56 边界 34 个共享节点，全部 0 不一致 |
| **按网格节点坐标求值**（dbg_full_sim 能否对任意节点坐标求 delegate 值） | **当前不能，需生成器改造** | Part 4/4b 证明 `eval_df_base` 取值**绑定 sIdx 的 split 位置**，实参 ix/iz 不参与 |

**裁定：路径 C 在语义层面成立**（grid 缓存单一值 + split 翻转不破坏逐位对齐），**但必须改生成器**（`_gen_split_lines` interpolated 分支需「节点模式」，把 split 从「按点+8角点展开」翻转为「按网格节点」）。这是实现改动，**非语义障碍**。

---

## 1. 验证设置

### 1.1 复用组件
- `dbg_full_sim.eval_df_base(root, corner, sIdx, ix, iy, iz)`：完整 delegate 求值器（含 spline 显式栈 + 边界嵌套递归），与 GPU 逐位对齐。
- `dbg_full_sim.NORMAL`（meta：n/octBase/splitBase/persistence/amplitude/amps）、`dbg_full_sim.perm`（256 元/octave × 356352）。
- `g.noise_slots`（slot → {kind, is_corner, base, stride}）、`g.interp_roots`=[134,140,148,151,155]（5 个 interp）。
- `split_dump.bin`：`SPLIT_TOTAL=8672` × 1024 样本（float）。
- `coords_dump.txt`：1024 个采样块坐标。

### 1.2 覆盖域（⚠ 局限，见 §5）
```
x 0..63 (4 chunk: 0,1,2,3)   y -64..-49 (gy 0..15)   z 0..0 (仅 1 个 z 面)
chunkX ∈ {0,1,2,3}, chunkZ 恒 = 0；cy ∈ {0,1}；cz 恒 = 0
```
域内网格节点 = 17(x)×3(y)×2(z) = **102** 个，**全部被覆盖**（每个节点都是某个采样 cell 的角点）。其中 94 个为多 cell 共享节点（内部节点），8 个为域边界节点（仅 1 cell）。

---

## 2. 验证结果：grid 节点值唯一性（核心）

方法：每 distinct cell 取一个代表样本（一个 cell 的 128 block 共享同一 8 角点），对该 cell 的 8 角点逐一 `eval_df_base(root, c, sIdx, ax, ay, az)`，按节点坐标分桶；**同一节点坐标出现 ≥2 个 (sIdx, c) 即做逐位比较**。

| interp | root | 覆盖节点 | 共享节点 | 最大共享度 | 命中角点索引 | 共享节点 max\|diff\| |
|---|---|---|---|---|---|---|
| interp_0 | 134 | 102 | 94 | 4 | {0..7} | **0.0** |
| interp_1 | 140 | 102 | 94 | 4 | {0..7} | **0.0** |
| interp_2 | 148 | 102 | 94 | 4 | {0..7} | **0.0** |
| interp_3 | 151 | 102 | 94 | 4 | {0..7} | **0.0** |
| interp_4 | 155 | 102 | 94 | 4 | {0..7} | **0.0** |

**解读**：
- `max|diff| = 0.0` 是**双精度严格相等**（`max(vals)-min(vals) == 0.0`），非阈值近似 → 同一 grid 节点从不同 cell、不同 corner 实例索引求值**逐位一致**。
- 命中角点索引覆盖 {0..7}：说明「跨 8 个角点实例索引的一致性」在共享节点集上被实际测到（虽然单个节点最大共享度=4，受限 z 覆盖，见 §5；但 8 个实例索引彼此由 Part 2 参数/perm 等价性 + Part 3 任意两两在节点处一致共同保证）。
- **结论：grid 可去重为单一节点值**——`buildInterpGrid` 每个节点一次求值，`sampleInterpGrid` 三线性插值即可逐位复现 production `InterpolatedDF`。

### 2.1 跨 chunk / 跨 cell 边界专项（edgeCol 复用前提）
- **x 方向跨 chunk 边界（x=16/32/48）**：18 个共享节点，0 不一致。左邻 chunk 的 gx=4 列（x=chunkX*16+16）与当前 chunk 的 gx=0 列（x=chunkX*16）世界坐标相同 → `eval_df_base` 值逐位相同 → **edgeCol 列跨 chunk 复用无损成立**。
- **y 方向跨 cell 边界（y=-56=cy0/cy1 交界）**：34 个共享节点，0 不一致。
- 二者共同确认：**节点值只由「世界坐标」决定（无角点/实例依赖）**——与 production `arg->sample(p)` 的「坐标决定一切」对齐。

---

## 3. 验证结果：8 份同参角点实例等价性

对每个 `is_corner` slot（base..base+7 共 8 份连续实例），比较：
- **参数**：amps / persistence / amplitude / n（来自 `NORMAL` meta）
- **perm 内容**：两段 octave 的 `perm[(octBase+k)*256 : +256]` 逐 256 元比较

**25 个 is_corner slot 全部 [OK]**（`params_eq=True perm_eq=True`）。含 continentalness、erosion、ridge、jagged、cave_entrance、spaghetti 系列、pillar 系列、noodle 系列等全部噪声。
- slot#4（old_blended）因 `NORMAL` 元数据不含 OLD 而跳过参数表对比，但其 8 份实例由 `_register_noise("old_blended", obbase+@c{c}, dict(params))` 以同 key/同参数拷贝注册（`_noise_slot` L190-198），且 Part 3 实测（含该 delegate 的 interp）节点值已逐位一致覆盖它。

**解读**：8 份同参实例 = **同一底层噪声函数**（同噪声 key → 同 PerlinNoiseSampler → 同 perm；同参数 → 同 octave 结构）。任意一份实例在「同一节点坐标」的采样值必相同 → grid 缓存**可用单一实例（如 corner=0）**统一求值，与 production 的单实例 `arg->sample(p)` 等价。

---

## 4. 验证结果：eval_df_base 坐标依赖 & 按节点坐标求值可行性

### 4.1 坐标依赖（Part 4）
对同 (sIdx, corner)，分别扰动 ix/iz 与 iy：
```
ix/iz 扰动 → diff = 0.000e+00   （噪声/样条读 sIdx 的 split 数据，不吃实参 ix/iz）
iy   扰动 → diff > 0             （仅 DF_Y / DF_Y_CLAMPED 用 iy）
```
**结论**：`eval_df_base` 的取值 = **corner 实例 sIdx 的 split 位置（编码全部坐标）** + **iy 实参（仅 y 参与）**。实参 ix/iz 完全不参与。→ **要得到节点 (nx,ny,nz) 的 delegate 值，必须让该节点的 split 数据落在 (nx,ny,nz)**。

### 4.2 取值绑定 sIdx 的 split 位置（Part 4b）
取位置相关节点 N=(0,-56,0)：用「其所在 cell」求值（正确）vs 用「相邻 cell（split 数据在 x=4）」求值：
```
interp_0 root=134: 正确=0.111084  错误cell(x=4)=0.267396  diff=1.563e-01
interp_1 root=140: 正确=0.577841  错误cell(x=4)=0.573998  diff=3.844e-03
interp_2 root=148: 正确=-0.067685 错误cell(x=4)=-0.067369 diff=3.169e-04
interp_3 root=151: 正确=0.656228  错误cell(x=4)=0.523663  diff=1.326e-01
interp_4 root=155: 正确=0.656815  错误cell(x=4)=0.752246  diff=9.543e-02
```
**结论**：`eval_df_base` 的取值**绑定 sIdx 的 split 位置**，不是绑定传入坐标。用错误 sIdx 求节点会得到"那个 sIdx 自己角点位置"的值（x=4 处），而非节点 (x=0) 的值。

### 4.3 可行性判定（[按网格节点组织 split]）
- **当前模型**：`_gen_split_lines` interpolated 分支（L1599-1613）对「**调用点 cell**」展开 8 角点，每个角点 c 以 `noise_key_suffix=@c{c}` 生成 delegate 的 split，坐标 = 该角点世界坐标。即 `splitCoord[sIdx]` 在**一次 `split()` 内**同时含「该 block 位置」+「其 cell 的 8 角点位置」的 split 数据，按噪声实例(base+c) 组织 → **sIdx 与「调用点 cell」绑定**。
- **因此**：在【当前 dump】下，`eval_df_base` **只能对「是某个采样 cell 的角点」的节点坐标**正确求值（那里 split 数据才在该位置）；对**任意节点坐标** (nx,ny,nz) 求值，需要 split 数据落在 (nx,ny,nz)，当前模型做不到。
- **必须的生成器改动**：`_gen_split_lines` interpolated 分支需提供「**节点模式**」——对每个 grid 节点**直接按节点坐标生成 delegate 的 split**（不做 8 角点展开），即 split 数据从「按点(sIdx)+8角点展开」翻转为「**按网格节点**」。同时 `splitTotal` / `NOISE_SLOT_*` 布局 / `perSample` / `base[]` / `valBuf` 分配都要随之重算（`_compute_val_layout` L414-483）。

**这是路径 C 的核心改造点**（与设计文档 §3.3 / §5 风险 #1 一致），**但不是语义障碍**：因为 §2/§3 已证「grid 节点单一值」与「8 实例等价」，故只要生成器按节点坐标产出 split，`eval_df_base`（用单一实例如 corner=0）在节点处求值 == production 的 `arg->sample(nodePos)`，逐位一致。

---

## 5. 关键发现 / 风险

1. **「对节点用 corner=0..7 各调 eval_df_base(root,c,sIdx,nx,ny,nz) 比较 8 值」的原始表述有误**：一个 cell 的 8 个角点是 **8 个不同位置**，用**同一 sIdx** 调 `c=0..7` 会取到 8 个不同节点（各 c 的 split 数据在其各自角点），**不是**「8 份实例在同一节点采样」。正确测法是：节点必须是「某 cell 的角点」，用该 cell 的 sIdx + 该节点的 corner 索引。跨 cell 时用各 cell 自己的 sIdx。**已在 §2 用正确测法验证**（共享节点两两逐位一致）。此坑值得写入错误台账。
2. **验证域局限（D23 回声）**：dump 只覆盖 `x∈[0,63], y∈[-64,-49], z=0`（4 chunk、cy 0/1、单 z 面）。虽已覆盖 x 跨 chunk（4 个 chunk）+ y 跨 cell（cy 0/1），并证 102 个域内网格节点全部一致，但**未覆盖**：z 方向跨 chunk、更高 y cell（cy≥2）、负坐标（x/z < 0）、跨 chunk 边界在 edgeCol 的真实遍历序。**路径 C 最终 C++ 必须按风险 #7 用 block_probe 在多 chunk / 多 cell / 负坐标 / 边界列对拍 production**，不能只信本小域。
3. **角点「实例选择」语义已解决**：grid 节点值唯一（§2）→ 可用**单一实例**求值，无需为不同 cell 锁死不同 corner。D23 教训（「角点值 ≠ finalDensity(角点坐标)」）在本验证下不构成障碍——因为 production `InterpolatedDF` 的 grid 值 = delegate 在节点采样，DFC 用该节点 delegate 的单一实例求值即等价。
4. **edgeCol 复用无损**：跨 chunk 边界节点逐位一致（§2.1）→ `edgeCol`（前 chunk gx=4 列 == 当前 gx=0 列）可无损复用，与 production 对齐。
5. **`minY`/`height` 通用化风险仍在**：当前 `minY=-64,height=384` 硬化（dfc_gen L1757/L2539）。若 future 支持 nether 等，grid GY 不同——生成器需参数化（与 §5 风险 #4 一致，本验证不缓解）。
6. **越界 clamp 差异**：production sample 有越界 clamp（density.h L517-522），DFC `interp_N` 没有。grid 缓存照搬需决定保留/去除（§5 风险 #6）。本验证基于现有（无 clamp）interp_N 对角点求值，与 production 在有 clamp 的 buildGrid 下**仅在越界 cell 有差异**；对正常域内节点无影响。

---

## 6. 产物 / 证据

- 验证脚本：`.investigations/perf-rework/verif_grid_cache_correctness.py`（主会话运行即可复现）。
- 辅助：`.investigations/perf-rework/_explore_coords.py`（覆盖域分析）、`.investigations/perf-rework/_probe_posvariant.py`（位置相关节点寻找）。
- 数据源：`vulkan-proto/split_dump.bin`（8672×1024 float）、`perm_dump.bin`（356352 u32）、`coords_dump.txt`（1024 坐标）。

---

## 7. 下一步建议（供主会话/judge 参考）

1. **路径 C 结论：可行**。C++ 侧执行路径为——① `_gen_split_lines` interpolated 分支加「节点模式」（按 grid 节点坐标生成 delegate split，不展开 8 角点）；② `CpuBackend` 增 per-interp thread_local grid（5×49×5）+ 生命周期；③ `buildInterpGrid`/`sampleInterpGrid`；④ 可选 edgeCol。插值公式零改动（已对齐）。
2. **验证顺序**：先按风险 #7 扩域（负坐标/z 向/更高 y）在 Python 层再证一次，再动 C++；C++ 落地后用 block_probe 多 chunk 对拍 production（Full 层）。
3. **需生成器改造的牵连面**：`splitTotal` / `NOISE_SLOT_*` 布局 / `perSample` / `base[]` / `valBuf`（`_compute_val_layout`）——建议先在 Python 模拟层把「按节点 split + eval_df_base + 三线性」端到端跑通并与 `interp_N`（已对齐 production）对拍，作为路径 C 的 feature 蓝本。
