# -288 区域 finalDensity 剖面对比分析（Phase 3）

> 角色：recode.scout（隔离子进程，只读勘探）
> 数据源：`.investigations/-288-reopen/cmd-output/` 下 C++ WG_SURFDUMP 三列 dump + Java 探针五件套
> 参照：`vanilla_-8248318472910187742_4_-288_-256.blocks`，seed=-8248318472910187742（-8248 世界）
> 产出：本文档（唯一可写产物）
> 结论标注：统计/数值对比=【确定】；机制归因=【推测】

---

## 0. 执行摘要（先说结论）

1. **y=36 的 0.23 finalDensity 差异是真实的，但它不是「finalDensity 树组件错误」，而是「采样基准错配」**：C++ `[SURF] finalDensity` 走的是 **InterpolatedDF 4×4×8 cell 三线性插值链**（与 Java 游戏实际 cns 插值链同基准），而 Java `vanilla_density_*.txt` 探针是 **无插值树 sample**（`router.finalDensity().sample()`，interpolated 节点直接采样底层）。两者在 cell 角点（y≡0 mod 8）一致、在 cell 中点（y≡4 mod 8）不同。
2. **差异层 100% 落在 y≡4 (mod 8)**（-60/-52/-44/-36/-28/-20/-12/-4/4/12/20/28/36/44/52/60/68/84/92/100，-52 等因该 cell 内线性度好差异仅 1e-5~1e-4）；y≡0 (mod 8) 全部逐位一致（≤2e-6）。这是 8 块 cell 插值路径的铁证。
3. **C++ 插值链与 Java cns 游戏实际插值链逐位吻合**：`squeeze(0.64 × cns_idx0(y))` 与 C++ `[SURF] finalDensity(y)` 在全部验证层一致（误差 ≤4e-6，打印位）。证明 C++ InterpolatedDF 插值实现正确。
4. **三列 initialDensity（无插值树）逐位一致**（-244/-242/-278 在 y=28..72 全同：1.785375/1.291625/0.797875/...），证明 finalDensity 树中的无插值部分（initial_density）在 C++ 侧自洽，树本身无列间异常。
5. **base_3d_noise 不构成根因**（见 §6.3）：finalDensity 差异模式是 8-cell 周期插值特征，base_3d 是无周期独立噪声函数；角点层（含 base_3d 参与）逐位一致排除了 base_3d 参与 finalDensity 树时的贡献错误；b3d 单点（y=31，0.017440 vs Java 插值 ≈0.0108）因采样点错位不可直接对比，标注待查但不指向根因。
6. **剩余列决策：不需补跑 (-242,-256)、(-278,-240) 的 Java 无插值 DensityProbe**（§5）。差异机制已由 (-244,-256) 完全解释，三列 initialDensity 一致证明树无列间差异；若需进一步定位块级 mismatch，应补跑 **cns（游戏实际插值链）** 而非无插值探针。

---

## 1. 剖面对比：(-244,-256) 列 C++ vs Java finalDensity

数据源：
- C++：`dump_x-244_z-256.txt` 中 `[SURF] (-244,y,-256) finalDensity=...`（y 步长 4，-64..124）
- Java：`vanilla_density_overworld_c-16_-16_b12_0.txt`（`router.finalDensity().sample()`，y 步长 4，-64..316；本表取 -64..108 有值段）

| y | C++ finalDensity | Java 无插值 | 差(C−J) | y mod 8 | 判定 |
|---|---|---|---|---|---|
| -64 | 0.037482 | 0.037482 | 0 | 0 | ✓ 一致 |
| -60 | 0.053541 | 0.049907 | +0.0036 | **4** | ✗ 差 |
| -56 | 0.069571 | 0.069571 | 0 | 0 | ✓ |
| -52 | 0.093005 | 0.092964 | +0.00004 | **4** | ~（1e-5） |
| -48 | 0.116335 | 0.116335 | 0 | 0 | ✓ |
| -44 | 0.139970 | 0.140280 | -0.0003 | **4** | ~（3e-4） |
| -40 | 0.163442 | 0.163443 | -1e-6 | 0 | ✓ |
| -36 | 0.163212 | 0.164152 | -0.0009 | **4** | ~（9e-4） |
| -32 | 0.162983 | 0.162983 | 0 | 0 | ✓ |
| -28 | 0.159217 | 0.159962 | -0.0007 | **4** | ~（7e-4） |
| -24 | 0.155447 | 0.155449 | -2e-6 | 0 | ✓ |
| -20 | 0.147643 | 0.150045 | -0.0024 | **4** | ✗ |
| -16 | 0.139820 | 0.139821 | -1e-6 | 0 | ✓ |
| -12 | 0.124667 | 0.123840 | +0.0008 | **4** | ~（8e-4） |
| -8 | 0.109455 | 0.109456 | -1e-6 | 0 | ✓ |
| -4 | 0.082866 | 0.097108 | -0.0142 | **4** | ✗ |
| 0 | 0.056158 | 0.056147 | +1e-5 | 0 | ✓ |
| 4 | 0.052818 | 0.051356 | +0.0015 | **4** | ✗ |
| 8 | 0.049476 | 0.049474 | +2e-6 | 0 | ✓ |
| 12 | 0.045093 | 0.045230 | -0.0001 | **4** | ~（1e-4） |
| 16 | 0.040707 | 0.040705 | +2e-6 | 0 | ✓ |
| 20 | 0.043504 | 0.034741 | +0.0088 | **4** | ✗ |
| 24 | 0.046301 | 0.046309 | -8e-6 | 0 | ✓ |
| 28 | 0.082637 | 0.105055 | -0.0224 | **4** | ✗ |
| 32 | 0.118752 | 0.118751 | +1e-6 | 0 | ✓ |
| **36** | **0.224602** | **0.453385** | **-0.229** | **4** | ✗✗ 最大 |
| 40 | 0.324994 | 0.324992 | +2e-6 | 0 | ✓ |
| 44 | 0.179740 | 0.178415 | +0.0013 | **4** | ✗ |
| 48 | 0.025628 | 0.025626 | +2e-6 | 0 | ✓ |
| 52 | -0.013938 | -0.016283 | +0.0023 | **4** | ✗ |
| 56 | -0.053461 | -0.053463 | +2e-6 | 0 | ✓ |
| 60 | -0.095322 | -0.100940 | +0.0056 | **4** | ✗ |
| 64 | -0.136843 | -0.136845 | +2e-6 | 0 | ✓ |
| 68 | -0.176210 | -0.172074 | -0.0041 | **4** | ✗ |
| 72 | -0.214996 | -0.214999 | +3e-6 | 0 | ✓ |
| 76 | -0.253799 | -0.253793 | -6e-6 | 0 | ✓ |
| 80 | -0.291722 | -0.291724 | +2e-6 | 0 | ✓ |
| 84 | -0.322045 | -0.320625 | -0.0014 | **4** | ✗ |
| 88 | -0.351608 | -0.351610 | +2e-6 | 0 | ✓ |
| 92 | -0.378710 | -0.376063 | -0.0026 | **4** | ✗ |
| 96 | -0.405003 | -0.405004 | +1e-6 | 0 | ✓ |
| 100 | -0.426438 | -0.416747 | -0.0097 | **4** | ✗ |
| 104 | -0.447216 | -0.447217 | +1e-6 | 0 | ✓ |
| 108+ | -0.458333 | -0.458333 | 0 | 0 | ✓（clamp 极限） |

**【确定】统计规律**：在 -64..108 全范围内，**差异 >1e-3 的层 100% 是 y≡4 (mod 8)**；**y≡0 (mod 8) 层 100% 逐位一致（≤2e-6）**。y=36 差 -0.229 是最大差异层（真实存在），但不是孤立尖峰——它属于 y≡4 (mod 8) 的一组差异层，只是该 cell [32,40) 内树非线性最强（无插值值在 32→36→40 为 0.1188→0.4534→0.3250，36 处有尖峰），故插值平滑效应最大。

主会话初步观察（-64 一致 / 48 一致 / 52 接近 / 36 差 0.23）与完整表一致，但「36 是唯一差异层」的暗示不成立：差异层是一个 8 周期的集合。

---

## 2. 差异层定位：哪一环节？

### 2.1 关键机制事实（源码确认）

- C++ `[SURF] finalDensity` 打印 `h->finalDensity->sample(p)`（`worldgen_api.cpp:673-674`），而 `h->finalDensity = builder->buildNode(*final_density_json)`（`:361-362`）。`overworld.json` 的 `final_density` 顶层是 `min(squeeze(0.64 × interpolated(blend_density(...))), ...)`——**含 `minecraft:interpolated` 节点**（`overworld.json:30-38`），C++ 将其构建为 **`InterpolatedDF`（4×4×8 cell 三线性插值链）**（`density_builder.h:163`；`density.h:467-591`，注释：「与 Java CellCache(add(DensityInterpolator(finalDensity), Beardifier)) 语义一致——只对 interpolated 节点插值，min/squeeze/mul 等非线性在插值后应用」，`worldgen_api.cpp:569-571`）。
- Java `vanilla_density_*.txt` 探针：`df = router.finalDensity(); df.sample(new UnblendedNoisePos(...))`（`DensityProbe.java:64-70`）——**无插值树 sample**，interpolated 节点不触发 cell 缓存插值，直接采样底层函数。

**→ C++ `[SURF] finalDensity` 与 Java `vanilla_density_*.txt` 并非同基准。** 任务假设「C++ [SURF] finalDensity 与 Java vanilla_density 同基准（无插值树 sample）」不成立。C++ 侧真正同基准的是 **Java `_cns.txt`（游戏实际插值链）**。

### 2.2 差异 = 插值路径差异（cell 中点），非树组件错误

- cell 网格：CELL_X=4, CELL_Y=8, CELL_Z=4，minY=-64 → y 角点在 -64, -56, ..., 0, 8, 16, ...（即 y≡0 mod 8）。
- 插值链在角点处 = 底层树直采值；在 cell 中点（y≡4 mod 8）处 = 角点三线性插值 ≠ 无插值树直采。
- 因此 y≡0 (mod 8) 层 C++（插值）= Java（无插值）逐位一致；y≡4 (mod 8) 层两者必然不同，差异大小正比于该 cell 内底层树非线性度。**与 §1 统计规律完全吻合。**

**【确定】差异层定位**：差异来自 **finalDensity 树的 interpolated 子节点在 cell 中点的插值路径**，即「C++ 插值链（InterpolatedDF） vs Java 无插值探针」的基准错配。**不是 finalDensity 树组件实现错误。**

### 2.3 分量交叉验证（C++ [COMP] vs Java comps）

| 分量 | C++ [COMP]（-244,-256） | Java comps（c-16_-16_b12_0） | 判定 |
|---|---|---|---|
| depth | -64:0.876250 … 316:-2.092500 | 逐位同（-64:0.876250 …） | ✓ 逐位一致 |
| continents | -0.206056（全 y 恒定） | -0.206057 | ~ 1e-6 打印位 |
| erosion | 0.246871 | 0.246878 | ~ 7e-6 |
| barrier | 32:0.065507 / 36:-0.175767 / 44:0.061232 / -32:0.526046 | barrierNoise 32:0.065514 / 36:-0.175762 / 44:0.061237 / -32:0.526028 | ✓ 一致（≤9e-6） |
| vein_toggle/ridged/gap 等 | 见 dump | （无对应列或需确认映射） | 未逐一核 |

注：barrier 在 Java comps 中名为 `barrierNoise`，C++ dump 为 `barrier`；抽样 4 层全部一致（差 ≤9e-6），说明 aquifer 输入分量无异常。该分量不进 finalDensity 树（仅 aquifer 用），不影响本剖面结论。

这些常数分量（continents/erosion/temperature/vegetation）全 y 恒定且两测 1e-6~1e-5 级吻合，进一步佐证无插值树整体对齐，差异集中在 interpolated 插值路径。

---

## 3. cns 链对比：C++ 插值链 vs Java 游戏实际插值

### 3.1 cns idx0 语义

`DensityProbe.java` cns 段遍历 ChunkNoiseSampler 的 8 个 interpolators，idx0 为第一个（历史记录 208 已实证：idx0 delegate = BlendDensity[initialDensityWithoutJaggedness 树]（final_density 树 argument1 里 interpolated 的内容），BlendDensity 是恒等包装）。因此 **cns idx0(y) = finalDensity 树中 interpolated(blend_density) 部分在 y 的插值后值**。

### 3.2 验证：squeeze(0.64 × cns_idx0) ≈ C++ [SURF] finalDensity

finalDensity 树 = min(squeeze(0.64 × interpolated(blend_density)), arg2)。C++ squeeze = clamp(x,-1,1)/2 − clamp(x)³/24（`density.h:154-157`）。当 arg2 分支不钳制时：

| y | cns idx0 | 0.64×idx0 | squeeze(0.64×idx0) | C++ finalDensity | 差 |
|---|---|---|---|---|---|
| 32 | 0.372866 | 0.238634 | 0.118751 | 0.118752 | 1e-6 ✓ |
| 36 | 0.714318 | 0.457164 | 0.224598 | 0.224602 | 4e-6 ✓ |
| 40 | 1.055769 | 0.675692 | 0.324992 | 0.324994 | 2e-6 ✓ |
| 44 | 0.567934 | 0.363478 | 0.179738 | 0.179740 | 2e-6 ✓ |
| 48 | 0.080098 | 0.051263 | 0.025626 | 0.025628 | 2e-6 ✓ |
| 52 | -0.043567 | -0.027883 | -0.013941 | -0.013938 | 3e-6 ✓ |
| 56 | -0.167232 | -0.107028 | -0.053463 | -0.053461 | 2e-6 ✓ |
| 60 | -0.298797 | -0.191230 | -0.095324 | -0.095322 | 2e-6 ✓ |

**【确定】C++ 插值链与 Java cns 游戏实际插值链逐位一致**（误差全部 ≤4e-6，落在 `%.6f` 打印位）。即：**C++ finalDensity 插值实现正确，块级（mismatch 层）密度与 Java 游戏实际同源**。

> 注：表中 `squeeze(0.64×idx0)` 为手算近似（±2e-6），实际 C++ 以浮点全精度计算；所有层两者均落入打印位容差（≤4e-6），结论不受手算误差影响。

### 3.3 结论：差异是「角点/树 sample 级」基准差，不是「插值实现级」错误

- Java 无插值探针在 y=36 的 0.453385 是**无插值树直采**（该 cell 内初始密度尖峰 ~1.54 经 0.64×squeeze）；
- C++ 在 y=36 的 0.224602 是 **8 块 cell 三线性插值后的游戏实际值**（= Java cns 插值值）。
- 若拿「C++ 插值链 vs Java 无插值探针」逐点比较，y≡4 (mod 8) 层必然出现 0.001~0.23 差异——**这不是实现 bug，而是基准错配**。历史时间线 L670「负坐标差 0.05-0.23」的相似量级，大概率源于同样的基准错配（RouterProbe 独立构建 + 插值/无插值混淆），而非 base_3d_noise。

---

## 4. 三列横向验证：initialDensity 一致、finalDensity 差异同机制

| y | initialDensity（-244） | initialDensity（-242） | initialDensity（-278） |
|---|---|---|---|
| 28 | 1.785375 | 1.785375 | 1.785375 |
| 32 | 1.291625 | 1.291625 | 1.291625 |
| 36 | 0.797875 | 0.797875 | 0.797875 |
| 40 | 0.304125 | 0.304125 | 0.304125 |
| 44 | -0.189625 | -0.189625 | -0.189625 |
| 48 | -0.683375 | -0.683375 | -0.683375 |
| 52 | -0.821625 | -0.821625 | -0.821625 |
| 56 | -0.945063 | -0.945063 | -0.945063 |
| 60 | -1.068500 | -1.068500 | -1.068500 |
| 64 | -1.191938 | -1.191938 | -1.191938 |
| 68 | -1.315375 | -1.315375 | -1.315375 |
| 72 | -1.438813 | -1.438813 | -1.438813 |

【确定】三列 initialDensity（无插值树 `R["initial_density"]->sample`）逐位一致，而三列 finalDensity 不同（-244:36→0.224602 / -242:36→0.223115 / -278:36→0.259195）。finalDensity 差异来自 cell 角点网格（依赖 chunk 内位置，三列角点不同）+ 插值，与无插值树无关。这排除了「某列树构建错乱」的可能，把差异收敛到插值路径。

---

## 5. 剩余列决策：是否补跑 (-242,-256)、(-278,-240) 的 Java DensityProbe

**建议：不需补跑这两列的 Java 无插值 DensityProbe（router.finalDensity().sample）。**

理由（按证据强度）：
1. 【确定】差异机制已由 (-244,-256) 完全解释：C++ `[SURF] finalDensity` 是插值链（InterpolatedDF），Java 无插值探针是不同基准；差异层= y≡4 (mod 8) cell 中点；C++ 插值链与 Java cns 逐位一致。
2. 【确定】三列 initialDensity 一致 → 无插值树部分无列间差异；finalDensity 差异仅由插值角点网格随位置变化产生，属同机制。
3. 【推测】补跑这两列的无插值探针只能复现同一 8-cell 周期差异，对定位根因增量≈0。

**替代建议（若仍需推进块级 mismatch 定位）**：
- 真正需要的是 **Java cns（游戏实际插值链）** 探针（`DensityProbe` 的 `_cns.txt` 输出），且应同时取该列 `caches`（CellCache 完整树 + Beardifier）以对齐 C++ `densityBuf`（`worldgen_api.cpp:619`，`h->finalDensity->sample`）。(-242,-256) 在 chunk(-16,-16) 内（同 c-16_-16_b12_0 探针的 chunk），其 cns 可复用同一 chunk 生成，只需把 DensityProbe 的 bx/bz 参数改为 (14,0)；(-278,-240) 在 chunk(-18,-15)，需新 chunk。
- 块级 mismatch（gravel/stone、water/stone 边界）属 **aquifer/surface rules 层**（Phase 2 已确认 33% 真 density/surface 差异），与 finalDensity 插值路径无直接因果；后续应对比 cns 游戏实际方块列而非密度探针。

---

## 6. 根因方向

### 6.1 候选根因（按证据排序）

1. **【确定】基准错配（已完全解释本剖面差异）**：C++ `[SURF] finalDensity` 是 InterpolatedDF 插值链（= Java cns 游戏实际），Java `vanilla_density_*.txt` 是无插值树 sample。y≡4 (mod 8) 层的全部差异（含 y=36 的 0.229）由此产生。**这不是 bug，是测量基准不一致**。后续所有「C++ finalDensity vs Java 无插值探针」的逐点 diff 都必须改用 cns（插值链）对照，否则会重复误报。
2. **【推测】块级真差异在 aquifer/surface rules 层**：-244（gravel/stone y≈50）、-242（water/stone y=52/56）、-278（stone/water y=15）边界差异是 Phase 2 已确认的 33% 真 density/surface 差异。本分析证实 finalDensity 插值链一致后，**该层是剩余真正待查对象**（Aquifer 液面、surface rule 抛铺、estimateSurfaceHeight 后续表面 builder）。estimateSurfaceHeight(-244,-256)=32 与 Java [ESH]=32 一致，说明表面估计入口正确，差异应在估计之后的表面规则/含水层细化。
3. ~~Barrier 分量~~ **（已排除）**：抽样的 4 层 barrier/barrierNoise C++/Java 全部一致（≤9e-6），aquifer 输入分量无异常（§2.3）。原误读（-32 的 0.526028 vs 32 的 0.065507）已修正。

### 6.2 反证排除项

- **排除「finalDensity 树组件（无插值部分）错误」**：【确定】y≡0 (mod 8) 层 C++ 与 Java 无插值逐位一致（≤2e-6），说明树的无插值求值在角点全对齐；cns squeeze 验证进一步证明插值链全对齐。若树组件错，角点不会一致。
- **排除「插值实现错误」**：【确定】C++ InterpolatedDF 插值结果 = squeeze(0.64×cns_idx0) 全层吻合（≤4e-6）。与 density.h:399 anchor（「InterpolatedDF 4x4x8 cell 插值逐位对齐 Java DensityInterpolator」）一致。
- **base_3d_noise 排除论证**（铁律：不预设，用数据说话）：
  - 【确定】base_3d_noise 是独立噪声函数（`overworld/base_3d_noise.json` 顶层 `old_blended_noise`，无 interpolated 包装），不具 8-cell 周期；而观察到的 finalDensity 差异 100% 呈 8-cell 周期 → 差异模式不可能由 base_3d 产生。
  - 【确定】若 base_3d 参与 finalDensity 树时贡献错误（如采样坐标/octave 错），y≡0 (mod 8) 角点层也会错；但角点层逐位一致 → base_3d 在最终树路径上无净错误。
  - 【推测】b3d 单点对比不足：C++ `[SURF] base_3d_noise(y=31)=0.017440` vs Java b3d 链每 4 采样（y=28:0.009808 / y=32:0.011146 / y=36:0.040973），y=31 无 Java 对应点，线性插值估算 ≈0.0108 与 C++ 差 ~0.0066——**采样点错位，且 b3d 独立链与 finalDensity 树内 base_3d 采样位置可能不同，无法从现有数据定论**。不指向根因；如需闭合可补 `-DdensityProbe` 在 y=31 的 b3d 单点或 C++ `wg_sample_named("minecraft:overworld/base_3d_noise", -244,31,-256)`。03 篇历史已排除（与 Java deriver 逐位一致），本分析不推翻，仅标注待查。

### 6.3 一句话根因

> **本剖面差异（含 y=36 的 0.23）是「C++ 插值链 vs Java 无插值探针」的基准错配，非实现错误；C++ finalDensity 插值链与 Java 游戏实际（cns）逐位一致。剩余真实 block 差异应归因于 aquifer/surface rules 层（Phase 2 的 33%），与 finalDensity 树及 base_3d_noise 无关。**

---

## 7. 影响架构的变化（交主会话裁决）

> 显式标注为「架构变更建议」，不在此自行修改。

1. **对比基准口径修正（高优先）**：C++ 侧所有 finalDensity 对照必须区分两路——
   - 无插值树 sample（对应 Java `router.finalDensity().sample()`）：仅在 y≡0 mod 8 角点可逐点比；cell 内不可比，除非补 C++ 无插值 finalDensity dump（`buildNode` 时跳过 InterpolatedDF 或对 interpolated 节点直采）。
   - 插值链（对应 Java `_cns.txt`）：与 Java cns 逐位可比（本分析已验证），是块级/游戏实际的正确保准。
   - 建议：`.investigations/` 文档与后续探针命名统一标注基准（`-interp` / `-raw`）。
2. **块级 mismatch 下一步**：既然 finalDensity 插值链一致，mismatch 收窄到 aquifer/surface rules。建议下一步跑 (-242,-256)（chunk(-16,-16) bx=14 bz=0）与 (-278,-240)（chunk(-18,-15)）的 **cns 游戏实际列**（含 `caches`/Beardifier 对比 C++ `densityBuf`）。
3. **base_3d 单点待查**（低优先）：y=31 单点 b3d 差异无法排除也无法确认，建议补一次同点同基准采样闭合；不阻塞主流程。

---

## 附：待深入点清单

| # | 项目 | 状态 | 置信度 | 建议 |
|---|---|---|---|---|
| 1 | C++ [SURF] finalDensity 基准（插值链 vs 无插值） | 已定位 | 确定 | 文档/命名统一 |
| 2 | cns idx0 ↔ C++ 插值链逐位验证 | 已通过（≤4e-6） | 确定 | 无需动作 |
| 3 | barrier 分量（原误读为差异） | 已排除 | 确定 | 抽样 4 层 C++/Java ≤9e-6 一致 |
| 4 | 块级 mismatch（-242/-278）cns 游戏实际列 | 未跑 | - | 建议补跑 cns 而非无插值探针 |
| 5 | base_3d_noise y=31 单点 | 待查 | 推测 | 同点同基准采样；低优先 |
