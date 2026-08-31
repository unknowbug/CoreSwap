# draft-multiworld-errors-m13 —— M13 追加 multiworld-errors.md（2026-08-31，status: candidate）

> 产出者：knowledge 落盘 subagent（2026-08-31）。数据源：主会话 M13 实测记录（nether 82.51%→96.0568%、bedrock 残差 10288→4011、分带数据）。
> 本条是同日后续，且是本文件此前所有 nether 对齐工作的收口性突破。
> 主会话应用方式：① 下面「A 部分」整段插入 **M12 补遗二 之后、`## 附：错误 → 根因 速查表` 之前**；② 「B 部分」表格行追加到速查表末尾。

## A 部分：追加小节正文

---

## M13. 数据驱动解析缺 `vertical_gradient` 支持：nether 82.51% → 96.06%（+13.5pp 一步修复）

### 现象
- nether 块级对齐长期卡在 82%（M12 修正链后 82.51%），**分带残差**：y0..31 79.8%、y32..63 65.8%、y64..95 55.2%、y96..127 61.0%（多个带同时偏低，非单带问题）。
- 混淆对特征：`netherrack→bedrock 10288@y96..`（**bedrock roof 整层缺失**）、`air→soul_sand/soul_soil`（涂布缺失）、`netherrack↔air` 双向（洞窟形状差，从属）。
- 前期定性曾指向「biome 判定错」（soul_sand_valley 误判）——后被 M11/M12 修正链澄清（❌ 该假说在 seed 修正后不再成立，真实根因见下）。

### 根因（机制）
- nether 的 surface_rule 走 **JSON 数据驱动解析**（`parse_surface_rule`，对齐 C++ 数据驱动架构铁律）——但解析器 `parse_surface_cond` 的条件类型支持清单**只有 not/biome/y_above/stone_depth，不支持 `vertical_gradient`**。
- `nether.json` surface_rule sequence 中 **bedrock_floor / bedrock_roof 两条 vertical_gradient 条件分支解析返回 None 被整条静默跳过**（[SURFACE-WARN] 有记录但被海量警告淹没，无人看）。
- 后果链：顶部 bedrock roof（y=123..127）、底部 bedrock floor（y=0..4）及整条表面规则链的涂布全部缺失——**不是逐带调参能解决的分带残差，是解析覆盖率缺口**。

### 定位（诊断方法）
1. **混淆对分带 top12 中 bedrock 缺口 10288**（特定块类整层缺失的签名）→ 读 `nether.json` bedrock_roof 规则结构（vertical_gradient + y_above NOT 组合）。
2. 对照 `parse_surface_cond` 支持清单（源码 L938 注释）→ 发现缺 `vertical_gradient`——一行「解析器不支持某条件类型」藏着 13.5pp。
3. **锚换算修正同步发现**：`below_top(N)` = `min_y + height - N` 的 height 必须传 **noise_height（128）** 而非 world_height（256）——否则 roof 锚落 251..256 越界（nether 逻辑高 128，与 M3 双高度教训同族）。

### 修复
1. `parse_surface_cond` 加 `vertical_gradient` 分支：`random_name` → `SurfaceCond::VerticalGradient { true_y = parse_anchor_abs_y(true_at_and_below), false_y = parse_anchor_abs_y(false_at_and_above) }`——复用既有 `vertical_gradient_test` 评估器（已支持反锚序/lerp/per-position random）。
2. 调用处 `parse_surface_rule(&sr, min_y, height)` 的 height 改传 `noise_height`（128）。

### 效果
- nether：**82.51% → 96.0568%**（+13.5pp 一步修复）；分带 y0..31 88.4%、y32..63 93.3%、y64..95 92.8%、y96..127 94.0%、y≥128 100%。
- bedrock 残差 10288 → 4011（roof 主层修复，剩随机层微差）。
- overworld **95.40% 零回归**（overworld 走代码规则，不经 JSON 解析器，不受影响）。

### 教训（⚠️ 重点，可复用判错经验）
- **数据驱动解析器的静默跳过是高危反模式**：解析失败的分支整条丢失且只留一行 WARN——新数据文件接入后 MUST 检查 [SURFACE-WARN]/[PARSE-WARN] 计数为 0（或解析覆盖率断言），**不能只看总分**（总分会被其它层部分对齐掩盖）。
- **「多带同时残差 + 特定块类缺失」优先查规则解析覆盖率**，而非逐带调参——本例 13.5pp 藏在「解析器不支持某条件类型」这一行。
- **锚基准随维度变**：below_top/above_bottom 的 height 必须用逻辑生成高度（nether 128），不能混用 world_height（256）——与 M3 双高度教训同族。

---

## B 部分：速查表追加行（插表末）

| nether 卡 82%、多带同时残差（y32..95 低至 55~66%）、bedrock roof/floor 整层缺失（混淆对 10288@y96..）（M13） | surface_rule JSON 数据驱动解析器不支持 `vertical_gradient` 条件类型 → nether.json 的 bedrock_floor/bedrock_roof 分支解析返回 None 被**整条静默跳过**（仅一行 WARN 被海量警告淹没）→ 顶部/底部 bedrock 层及规则链涂布全缺；附带：`below_top(N)` 的 height 误传 world_height(256) 致 roof 锚越界（须用 noise_height 128） | **「多带同时残差 + 特定块类缺失」优先查规则解析覆盖率，不逐带调参**——数据驱动解析器静默跳过分支是高危反模式，新数据文件接入 MUST 断言 [PARSE-WARN] 计数为 0，不能只看总分（总分被其它层掩盖，+13.5pp 一步修复）；**锚换算的 height 用逻辑生成高度（nether 128），不混用 world_height 256**（M3 同族） |
