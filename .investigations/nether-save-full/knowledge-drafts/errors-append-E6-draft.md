# 草稿：nether-save-errors.md 追加 E6 条目

> 用途：主会话应用——追加到 `.investigations/nether-save-full/nether-save-errors.md`：
> ① 在 E5 之后、「未闭合待查项」节之前插入 E6 正文；
> ② 在文末「错误→根因 速查表」表尾追加 E6 一行。
> 格式：五段式（现象/根因/定位/修复/教训），与 E1-E5 对齐。

---

## E6 正文（从下面一行开始复制）

## E6. 对照口径误置：把存档口径残差直接归因 surface rule 条件链，未先做阶段消融

- **编号**：E6（B1 定论轮复盘，建议 candidate）
- **现象**：260901-03 轮把 seed B 残差大头 B1（basalt deltas 三大宗石互换，52,078 块 / 76.6%）的机制候选直接写成「surface rule 条件链系统性偏差（biome 判定 / noise 阈值 / Hole 语义下游表现）」并列为深挖优先级 #1——本轮三方实验证明该归因方向错位：互换主因 = **feature 阶段产物**（blobs/columns/delta/pillar）在两种基底地形上的命中/形态差 + Rust surface 薄带残差，宗石大宗根本不是 surface rule 产物；连带把已修复的 Hole 语义（M6 后 Rust `stone_depth_above <= 0` 与 Java 一致）仍当未闭合疑点继承。
- **根因**：**对照口径误置 + 归因未先做阶段分解**——残差来自存档口径（Rust noise/surface + Java carvers/features 端到端），其中混着 Java feature 阶段产物，却在未做任何阶段消融的情况下把差异整体对到替换方（Rust surface rule）的条件链上；且把上一轮交接文档里的「方向性待查假设」（Hole 语义不一致）当公理直接继承，未做廉价独立验证（该假设 M6 修复时已过时）。
- **定位**：三方实验 + fan-out 两候选裁决：① 纯 Rust 口径（ctypes 直连 dll vs rlib 直跑 cell 级 0 差异）vs FULL = 77.43%（basalt→netherrack 157k = surface 薄带 + 纯 Rust 下 blobs/columns 缺失叠加）；② 存档口径（+Java carvers/features）= 93.5508%——feature 补回大头；③ WG_SKIP_SURFACE=1 重跑 = 55.18% 且 blobs 不触发（stone 基底非 netherrack → blackstone=0、quartz/gold ore=0）——证明 blobs 是 feature 阶段、依赖 netherrack 基底；④ biome 分桶（互换 100% 落 vanilla basalt_deltas 列）排除源分配差。两候选：.b1 surface_depth 带厚（❌ 带厚上限 ≤6 层，40 层体块不可达）、.b2 nether_state_selector 恒 0.0（⚠️ 真实 bug 但只解释零星翻转，非主导）。
- **修复**：B1 机制定论改写为「feature 产物 × 两种基底地形」结论（→ 09 篇追加小节草稿，candidate）；Hole 语义遗留行做 supersedes 标注（§15.4，原行不删）；对照口径澄清（纯 Rust 77.43% 与存档 93.55% 载体不同不可比；B1 参照分两用：BlockProbe SURFACE 口径测 Rust surface 残差、存档口径测端到端）；.b2 的 nether_state_selector 预加载表缺 nether 噪声列为待修（一行补齐，非 B1 主导）。
- **教训**：
  1. **替换模式存档口径残差必须先做三阶段归因（noise/surface = 替换方 vs carvers/features = 存续方），再定位机制**——「残差 → 某层条件链」的归因出手前必须已有阶段消融（如 WG_SKIP_SURFACE）或直连基线（如 ctypes 直连）证据，否则只能是 draft（已沉淀为 workflow-patterns 发现 #10）。
  2. **交接假设开工先验再继承**（AGENTS.md 交接结论验证纪律）：本轮第一动作即用 L101 源码核对推翻 Hole 假设——若沿用上轮归因直接深挖 surface rule 条件链，整轮工作量将投入不存在的 bug。
  3. **「大宗块差」先问产物阶段归属**（与发现 #2/#4 同族）：vanilla 宗石/涂布类块面多为 feature 阶段产物，见到成片互换先查 feature 依赖（基底块条件），再查 surface/noise。

## 速查表追加行（表尾）

| E6 B1 大宗互换被归因 surface rule 条件链（实为 feature 产物 × 基底差） | 对照口径误置：存档口径混 Java feature 阶段产物，未先阶段消融就归因替换方条件链 | **先消融/直连基线后归因**（发现 #10 三阶段归因法）；交接方向性假设开工先验 |
