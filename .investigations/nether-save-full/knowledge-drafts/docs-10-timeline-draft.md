# 草稿：docs/10 时间线 260901-03 条目

> 用途：主会话应用——**追加**到 `versions/1.20.1/docs/10-timewise-archive.md` 文件末尾（追加不覆盖）。
> 状态：按时间线纪律（含被推翻假说记录，每条一行排除证据）。

---

## 追加正文（从下面一行开始复制）

## 260901-03 B1 定论：basalt deltas 三大宗互换 = feature 产物 × 两种基底地形（candidate）

> 承接 260901-03 nether 存档条目 B1 未闭合项（52,078 块 / 76.6%）。结论 → 09 篇「B1 定论」节；错误 E6 → nether-save-errors.md。

### ✅ 一、前置验证推翻交接假设（Hole 语义）
- 上轮遗留「Rust Hole 用 surface_depth<=0」为 M6（2026-08-30）修复前过时表述；开工前廉价独立验证：Rust surface_rules.rs L101 当前为 `Hole => stone_depth_above <= 0` 与 Java 一致，dll M17（sha C5AC5309）含修复——Hole 语义课题闭合（§15.4：09 篇原行加 supersedes 注记，不删）。
- 教训印证 AGENTS.md 交接结论验证纪律：交接里的「方向/待查假设」开工先验，本轮第一动作即排除一条假赛道。

### ✅ 二、机制定论（三方实验）
- 架构：cppReplace = Rust 只接管 populateNoise+buildSurface；vanilla carvers+features 仍在 Rust 地形上跑。宗石大宗（basalt_blobs/blackstone_blobs、large/small_basalt_columns、delta、basalt_pillar）本是 feature 阶段产物。
- 三方数据：纯 Rust（ctypes 直连 dll vs rlib 直跑 cell 级 0 差异）vs FULL = 77.43%（basalt→netherrack 157k）；存档（+Java carvers/features）= 93.5508%；WG_SKIP_SURFACE=1 = 55.18% 且 blobs 不触发（stone 基底非 netherrack → blackstone=0、quartz/gold ore=0）。
- 判读：互换主因 = 同一套 Java feature 在两种基底地形上的命中/形态差 + Rust surface 薄带残差；biome 分桶（互换 100% 落 vanilla basalt_deltas 列）排除 biome 源分配差。

### ❌ 三、fan-out 两候选裁决
- ❌ **.b1 surface_depth 带厚机制不成立**：带厚上限 ≤6 层，实测 40 层体块不可达（排除证据：`.artifacts/.b1-surface-depth/` 最终 verdict）。
- ⚠️ **.b2 nether_state_selector 恒 0.0 是真实 bug 但非主导**：`create_for_dim` step4 预加载表缺 nether 噪声（nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer → `unwrap_or(0.0)`）——只解释零星分支内翻转（证据：`.artifacts/.b2-nether-state-selector/`）。**修复待做**：一行预加载表补齐，预期闭合 soul_soil 子族等。

### 🔍 四、新过程事实与口径纪律
- **同 dll 非确定性容差**：同 dll 两次完整 run 相差 369 块（93.5156% → 93.5508%）——Java feature 阶段邻块写入调度非确定性；存档口径对齐指标 MUST 声明该容差。
- **对照口径澄清（§9.7）**：纯 Rust 口径（77.43%）与存档口径（93.55%）载体不同不可比；B1 深挖参照分两用——BlockProbe SURFACE 口径测 Rust surface 残差，存档口径测端到端。

### 📌 记录指引
- 结论 → 09 篇「B1 定论」节（草稿 knowledge-drafts/docs-09-b1-verdict-draft.md，含 L165 supersedes 标注文本）。
- 错误 E6（对照口径误置）→ `.investigations/nether-save-full/nether-save-errors.md` 追加。
- 通用模式 → knowledge/discovered/workflow-patterns.md 发现 #10（三阶段归因法）。
- 状态：机制定论 candidate（judge 审查通过建议），confirmed 留人类。
