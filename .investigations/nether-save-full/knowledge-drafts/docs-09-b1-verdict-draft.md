# 草稿：docs/09 追加小节 —— B1（basalt deltas 三大宗石互换）机制定论

> 用途：主会话应用——**追加**到 `versions/1.20.1/docs/09-multi-dimension.md` 文件末尾（追加不覆盖）。
> 同时含两处就地标注动作：① L165「遗留课题更新」Hole 行的 supersedes 注记（不删原行）；② 口径澄清。
> 状态：机制定论 candidate（judge 审查通过建议），confirmed 留人类拍板。
> 依据：`.investigations/nether-save-full/`（facts / .b1 / .b2 / residual-interpretation / judge-review）+ `.artifacts/.b1-surface-depth/`、`.artifacts/.b2-nether-state-selector/`。

---

## 追加正文（从下面一行开始复制）

## B1 定论：basalt deltas 三大宗石互换 = feature 阶段产物在两种基底地形上的命中/形态差（candidate，260901-03）

> 承接上节「nether 存档写入口径 Full 化」B1 未闭合项（52,078 块 / 76.6%）。本轮三方实验 + fan-out 两候选裁决后机制定论。过程与被推翻假说见 10 时间线 260901-03 条；错误 E6 见 `.investigations/nether-save-full/nether-save-errors.md`。

### 机制定论（B1 主导，candidate）

- **架构事实**：cppReplace 模式下 Rust 只接管 populateNoise + buildSurface（NoiseChunkGeneratorMixin.java）；vanilla carvers + features 仍在 Rust 地形上运行。
- **机制**：nether 的 basalt_deltas / soul_sand_valley 宗石大宗（basalt_blobs / blackstone_blobs = netherrack_replace_blobs、large/small_basalt_columns、delta、basalt_pillar）**本是 feature 阶段产物**，不是 surface rule 产物。同一套 Java feature 在两种基底地形（vanilla surface vs Rust surface）上运行，命中/形态不同 → 大宗互换；叠加 Rust surface 薄带残差。
- **biome 源分配差排除**：互换块 100% 落在 vanilla basalt_deltas 列内（soul_sand_valley 家族单列）——feature 的 biome 源分配两侧一致，排除。

### 三方实验证据（数据直读）

| 口径 | 数字 | 判读 |
|---|---|---|
| 纯 Rust 输出（ctypes 直连 dll vs rlib 直跑）vs FULL 参照 | **77.43%**（basalt→netherrack 157k）；ctypes/rlib cell 级 **0 差异**（确定性） | Rust surface 薄带 + 纯 Rust 口径下 blobs/columns 缺失的叠加表现 |
| 存档（Rust noise+surface + Java carvers/features）vs FULL 参照 | **93.5508%** | feature 阶段产物补回大头 |
| WG_SKIP_SURFACE=1 重跑 | **55.18%**，且 blobs 不触发（stone 基底非 netherrack → blackstone=0、quartz/gold ore=0） | **surface 是实心块主来源**，且证明 blobs 是 feature 阶段依赖 netherrack 基底 |

### 对照口径澄清（v0.20 §9.7）

- 纯 Rust 口径（77.43%）与存档口径（93.55%）**不可比**（载体不同，§9.7）。
- B1 深挖的正确参照分两用：**BlockProbe SURFACE 口径**（无 carvers/FEATURE）测 Rust surface 残差；**存档口径**测端到端。
- **同 dll 非确定性容差（新过程事实）**：同 dll 两次完整 run 相差 369 块（93.5156% → 93.5508%）——Java feature 阶段邻块写入调度非确定性所致。**存档口径对齐指标必须声明该容差**（同 dll 重跑差 ≤ 百分级块数属正常，非实现回归）。

### 附带定论与遗留

- ✅ **Hole 语义不一致已闭合**（取代本篇前文遗留课题中的 Hole 行）：docs/09 前文「Rust Hole 用 surface_depth<=0」为 M6（2026-08-30）修复前的过时表述；当前 Rust surface_rules.rs L101 `Hole => stone_depth_above <= 0` 与 Java 一致，dll M17（sha C5AC5309）含修复。见下方 supersedes 标注。
- ❌ surface_depth 带厚机制（fan-out .b1）：不成立——带厚上限 ≤6 层，实测 40 层体块不可达（`.artifacts/.b1-surface-depth/` verdict）。
- ⚠️ nether_state_selector 恒 0.0（fan-out .b2）：**真实 bug**（`create_for_dim` step4 预加载表缺 nether 噪声：nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer → `unwrap_or(0.0)`），但只解释零星分支内翻转，**非 B1 主导**（`.artifacts/.b2-nether-state-selector/`）。修复值得做（一行预加载表补齐），预期闭合 soul_soil 子族等——**待修，修复后重测**。

---

## 就地标注一：L165 遗留课题 Hole 行 supersedes 注记（原行不删，其后追加一行）

原行（保留不动）：
`- Hole 语义不一致（Rust surface_depth <= 0 vs Java stoneDepthAbove <= 0，C++ L251 才对——Rust 注释声称对齐 Java 是错的，影响 nether lake/not(hole) 门控，单开课题）；`

在其后追加：

`  - **[supersedes 260901-03]** 本行已过时（M6 修复前表述）：当前 Rust surface_rules.rs L101 Hole => stone_depth_above <= 0 与 Java 一致，dll M17（C5AC5309）含修复——Hole 语义课题闭合。依据见本文「B1 定论」节（§15.4 取代链，原行保留不删）。`

## 就地标注二：上节「未闭合待查项」#2 B1 行追加状态注记

在「basalt deltas 大宗互换（B1）……」待查项后追加：

`  - **[已结案 260901-03]** 机制定论见本文「B1 定论」节：feature 阶段产物（blobs/columns/delta/pillar）在两种基底地形上的命中/形态差 + Rust surface 薄带残差；surface_depth 带厚候选被排除，nether_state_selector bug 另案（非主导）。`
