# v5 残差 fan-out 候选：nether biome 选择链偏差机制

- 日期标签：260902-04（worker 只读分析，无 shell；原 260902-05 系笔误，judge P5 修正）
- 置信度：全体 draft（静态精读，Degraded 分层）
- 现象：残差列 100% = Rust 判 warped_forest，vanilla 判 basalt_deltas(94.4%)/soul_sand_valley(5.6%)

> **[supersedes 标注，260902-04]** 本文档前提「残差列 100% = warped_forest」已被探针坐标 bug 修正推翻——wBiome 误用 chunk 局部坐标（见 finding-mid-260902-04.md / workflow-patterns 发现 #13 / judge P5）。修正后：96.32% biome 一致，.b2「单向落入 warped 碗」前提不成立；.b1/.b3/.b4 机制分析仍可作 B1 下钻参考（维持 draft）。

## 链路事实（静态确认）

1. `biome_params_nether.json`（实际路径 `versions/1.20.1/data/biome_params_nether.json`，不在 worldgen/ 下）：
   5 条目全部是**点参数盒**（[v,v]），无 NaN 维，offset 明确：warped=0.375、basalt=0.175、其余 0.0。与 vanilla 1.20.1 `MultiNoiseBiomeSource.Preset.NETHER` 的 `Climate.parameters(...)` 逐值一致。**疑点 3 排除：warped 不是兜底 biome（无全 NaN 盒）。**
2. nether.json router：continents/erosion/depth/ridges 全为常量 0.0；temperature 与 vegetation 均为 `shifted_noise(noise=minecraft:temperature|vegetation, shift_x=shift_x, shift_z=shift_z, xz_scale=0.25, y_scale=0.0)`。**⇒ biome 只随 (x,z) 变化、整列常量**——「残差是整柱级」与该结构自洽，残差必然产生于 2D (t,h) 场的采样值差异或判定差异，不可能是 y 相关机制。
3. 判定数学（全常量退化后）：warped 胜 ⇔ `(h−0.5)² + 0.375² < (t+0.5)² + 0.175²`，即 `(t+0.5)² − (h−0.5)² > 0.11`。vanilla 判 basalt 而 Rust 判 warped ⇒ Rust 侧 t 显著高于 vanilla（或 h 显著偏高），**边界量级 ≥ ~0.25**——这是符号级/结构性输入差，不是浮点平局。

## 候选

### .b1 offset 维语义偏差（L426 vals[6]=0.0）
- 机制：`biome.rs:426` 查询点 offset 恒 0.0；条目侧 offset 经 `BiomeClassifier::load` (L274) 读为点范围 [offset,offset] 并入第 7 维距离。vanilla `MultiNoiseSampler.sample` 构造 TargetPoint 时 offset 同样固定 0.0，条目 offset 只作为「固定惩罚距离」参与。
- 评估：静态逐点对拍一致，**疑点 1 基本排除**；列此候选仅为形式完备（offset 读取 as_f64 已确认 0.375/0.175 正确入树）。
- 裁决探针：WG_BIOMEDUMP 残差列输出 `biome_of_debug` 的 best_dist；手工按 (t,h)+offset 重算 5 距离，若手算最近 = biome_of 输出则 offset 维无误。
- 置信度：draft（低可疑）

### .b2 shift_x/shift_z（Legacy 随机源 ShiftA/ShiftB Perlin）采样偏差【最可疑】
- 机制：nether `legacy_random_source=true`（nether.json L13，Rust 侧 `worldgen_handle.rs:135-144` 走 Legacy 分支）。temperature/vegetation 的 shifted_noise 需要 shift_x/shift_z 两个特殊噪声（vanilla NoiseConfig 里由 `LegacyRandomSource` 派生的 PerlinNoise，种子派生路径与普通 noise 分开）。若 Rust 的 shift_a/shift_b 种子派生（`RandomState` ctor 中 legacy 源的 at/derive 顺序）或采样坐标（vanilla ShiftedNoise：先乘 xz_scale=0.25 再加 shift 采样值）与 vanilla 有差，(t,h) 场整体错位 ⇒ 边界带翻转，方向性表现为某些区域 warped↔basalt 互换——与 94.4% 单向分布吻合（错位场一边倒落入 warped 碗）。
- 涉及代码：`worldgen_handle.rs:183-188`（tempf/humf 从 router 建）、`density.rs` shifted_noise/shift 采样实现、noise.rs Legacy Perlin 种子链。
- 裁决探针：主会话采 ①WG_BIOMEDUMP（Rust：残差列 6 维 t/h + 判定）②Java RouterProbe 的 temperature/vegetation 行（同 seed、floor 对齐坐标）——先按探针三查铁律核 seed 与坐标口径（三套坐标不可直接比）。期望观测：**残差列 t 或 h 的 |Δ| ≥ 0.1 且 t 符号跨 ±0.25 边界**；非残差列 Δ≈0。若所有列 Δ≈0 → 排除 .b2，转 .b4。
- 置信度：draft（高可疑）

### .b3 pick-cell 种子/坐标偏差（biome_pick_cell 的 seed 语义）
- 机制：`worldgen_handle.rs:534/661` 用 `self.seed`；`biome.rs:231-260` 为 Java BiomeAccess.getBiome 8 邻域复刻。若 `self.seed` ≠ vanilla BiomeAccess seed（worldSeed vs worldSeed±/^ 某常量），picked (px,pz) 会偏 ±1 cell（4 块），t/h 采样点错位 → 同样产生整列翻转。y 项因 y_scale=0 无影响（疑点 4 的 y 分量自豁免，xz 分量仍有效）。
- 裁决探针：WG_BIOMEDUMP 打印 picked (px,py,pz) vs Java SURFBIOME 的 bp 坐标（注意口径不同：SURFBIOME 打印 bp 对齐坐标、判定输入是原始 BlockPos；WG_BIOMEDUMP 是 (px<<2,pz<<2)）。期望观测：残差列 picked px/pz 相差 ±4 而非相同。
- 置信度：draft（中）
- 旁证：该函数与 C++ biomePickCell 对齐且已被存量验证覆盖，直接错位可能性低于 .b2。

### .b4 Rust SearchTree 移植缺陷（无 Rust 侧单元测试）
- 机制：`biome.rs:50-76` KD 剪枝/平局（严格 `<`，先到先赢）静态看与 vanilla 一致、且 enclosing 界剪枝不改变最近邻（构造无关正确性）。**但 st_bug_test.cpp 只验证了 C++ 侧移植，Rust 版 SearchTree 从未被单元测试过**——尤其 build_search_tree 的 NaN 初始化/enclosing 聚合在 5-entry 小树上的退化路径。
- 裁决探针：纯数据裁决——用 .b2 探针采到的残差列 6 维值，离线线性扫描 5 条目重算最近邻；若离线胜者 ≠ biome_of 输出（或 ≠ vanilla 胜者而 vanilla 输入相同）→ 树缺陷；若离线 = biome_of 输出且输入 Δ≈0 → 该候选排除，矛盾回推输入差（.b2）。可加 Rust 侧 5-entry 穷举单测（枚举 (t,h) 网格 × 树 vs 线性扫描）。
- 置信度：draft（中低；「全列单向 warped」更符合输入场错位而非树裁剪错误——树错应产生空间碎斑而非整列一致性）

### .b5 tempf/humf 密度函数同源性（疑点 5）
- 机制：`worldgen_handle.rs:183-188` 直接取 nether.json router 的 temperature/vegetation/continents/erosion/depth/ridges——与 vanilla router 同源同键，文件已核对无替代键。剩余风险仅在该 router 子树在 Rust density_builder 中的展开（shifted_noise 节点语义、xz_scale 乘法次序、shift_y=0 分支），已并入 .b2 的探针观测范围（若 t/h Δ 非零，继续下钻到 shifted_noise 节点级 A/B）。
- 置信度：draft（结构同源已确认，实现级风险归入 .b2）

## 主会话裁决顺序建议

1. 一次 WG_BIOMEDUMP + RouterProbe 对照采集（同 seed！三查铁律）同时裁决 .b1/.b2/.b3/.b4：残差列 10-20 列即可。
   - 输入 Δ 大 → .b2（下钻 shifted_noise/shift 节点 A/B）
   - 输入 Δ≈0 但 picked 坐标差 → .b3
   - 输入与坐标全同但胜者异 → .b4（离线线性复算 + Rust 树单测）
   - offset 手算不一致 → .b1（几乎不可能）
2. 任何探针前先核对 seed 三处一致（worldSeed 纪律，M11 三犯教训）。
