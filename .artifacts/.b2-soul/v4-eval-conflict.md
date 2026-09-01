---
id: v4-eval-conflict
topic: b2-soul
title: V4 求值层矛盾裁决——parse_surface_cond 布尔字段解析 bug（add_surface_depth/add_stone_depth 恒 false）
status: draft
验证分层: Degraded 静态阅读 + Partial（bin-diag 复现：soul_tree_repro 解析产物树 dump + 定点 apply，2026-09-09，未逐位对拍）
可比性声明: 载体=nether.json（versions\1.20.1\data\worldgen\data\...\nether.json，sha256=80EE79C0*，四副本同哈希）× surface_rules.rs 当前工作区源码 × soul_tree_repro 实跑产物；覆盖面=nether surface_rule 全树解析产物 + 单点（3260,1,3200）apply + 生产 dump stderr 交叉核对；与 V1/V2 存档写入口径、V3 纯静态口径均不可比（本产物裁决「解析产物树 ≠ JSON 语义」，不重新度量对齐率）
date: 2026-09-09
---

# V4 求值层矛盾裁决：规则树求值层矛盾 = 解析器布尔字段 bug

## 0. 定论（draft）

**矛盾机制 = 候选 b（parser 产物树 ≠ JSON 语义），具体为 `parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段
（`add_surface_depth` / `add_stone_depth`），布尔值 `as_f64()` 返回 `None` → 字段恒解析为 `false`。**

- V3 的「soul_soil 无条件兜底存在」推读**没有错**（JSON 语义层面成立，L316-321）；
- 但解析产物树里 soul 分支 ceiling 条件的 `add_surface_depth` 被解析成 **false**（JSON L293 = `true`），
  StoneDepth 判定从 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1+0+0`——在 3260,1,3200（sdb=2, surface_depth=3）
  处 `2 ≤ 1` 为 false → **soul 分支根本没进入**（与 V4 dump 的 ceiling_ok=true 不矛盾：dump 镜像用的是
  JSON 正确语义，不是解析产物树的语义）→ 穿透到 [7] 兜底 `block netherrack` → applied=256。
- V3 §2 的「结构差不可解释」结论在该口径下正确；本产物以 **supersedes** 方式细化：结构差不在 JSON→树
  的「节点结构」层（V3 已排除，维持成立），而在「节点参数」层（V3 对拍表把参数核对标为「参数全对拍」，
  该项为 V3 静态对拍的漏检——布尔字段肉眼对拍「全对」，实际解析为 false）。

## 1. 证据链（逐节点求值路径 @ 3260,1,3200）

输入 ctx（生产 dump，soul-ctx-dump.stderr.txt）：biome=minecraft:soul_sand_valley, sda=22, sdb=2,
surface_depth=3, fluid_height=32, selector=-0.047268, y=1。

解析产物树（soul_tree_repro 实跑 dump，.investigations/soul-v4v5/cmd-output/soul-tree-repro.txt；JSON=versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\nether.json，Rust=surface_rules.rs）：

| # | 树节点（repro 实测参数） | JSON 原文（行号） | 偏差 |
|---|---|---|---|
| [0] | Cond VerticalGradient bedrock_floor true_y=0 false_y=5 → Block(31) | L108-126 | 无 |
| [1] | Cond Not(VerticalGradient bedrock_roof true_y=122 false_y=127) → Block(31) | L127-148 | 无 |
| [2] | Cond AboveY anchor=122 mult=0 asd=false → Block(256) | L149-165（anchor below_top 5→122 ✓） | 无 |
| [3] | basalt_deltas 分支：**StoneDepth asd=false**（JSON L181=`true`） | L166-277 | **bug** |
| [4] | soul 分支：ceiling **StoneDepth off=0 asd=false sdr=0 ceiling=true**（JSON L293=`true`）→ Seq[NoiseThreshold selector≥0→Block(257); **Block(258)**] | L278-403 | **bug（本次因果）** |
| [5] | 通用 floor 段 StoneDepth asd=false（JSON L408=`false` ✓，恰为假阴性）；warped/crimson 内 y_above asd=false（JSON L420/457→`false` ✓） | L404-563 | 偶合正确 |
| [6] | nether_wastes 分支 floor StoneDepth **asd=false**（JSON L564 段 `add_surface_depth: true`） | L564-644 | **bug（疑及签名 C）** |
| [7] | Block(256) 兜底 | L728-737 | 无 |

求值路径（repro + SurfaceCond::test L84-93 逐行）：
1. [0] y=1 ∈ (0,5) → gradient 随机（生产 splitter 判 false，repro 假 splitter 判 true；两值均可，不矛盾）。
2. [1] bedrock_roof true_y=122：y=1 ≤ 122 → gradient true → Not → false ✓（不写 roof bedrock）。
3. [2] 1 ≥ 122 false ✓。
4. [3] biome≠basalt false ✓。
5. [4] **biome=true** → then_run Seq → ceiling StoneDepth：`i=sdb=2 ≤ 1+offset(0)+j+k`，**j=0（asd 解析 false）** → `2 ≤ 1` = **false**（正确语义下 j=surface_depth=3 → `2 ≤ 4` = true）；floor StoneDepth 同病：sda=22 ≤ 1 false（正确语义 22 ≤ 4 也 false，不改变结果）→ **[4] 整体 None**。
6. [5] floor StoneDepth sda=22 ≤ 1 false → None；[6] biome≠wastes → None。
7. [7] Block(256) → **applied=256=netherrack** ✓ 与生产 dump 逐位一致。

根因行（surface_rules.rs parse_surface_cond）：
- **L1093**（stone_depth 的 `add_surface_depth`）：`j.get("add_surface_depth").and_then(|x| x.as_f64()).map(|x| x != 0.0).unwrap_or(false)` —— JSON `"add_surface_depth": true` 是布尔，`crate::json::JsonValue::as_f64()` 对布尔返回 `None`（json.rs 只对数字返回 Some）→ 恒 `false`。
- 同型三处：**L1079**（y_above `add_stone_depth`）、**L1116**（water `add_stone_depth`）、L1095/L1116 无此问题（字符串/其他）。y_above 的 asd bug 在 nether 的 gravel patch 链（anchor 30/35，JSON L220/232/349/361 = true）同样生效——gravel 高度带整体偏移，独立次生影响。
- 对照：`legacy_random_source` 的读取（worldgen_handle.rs L304 附近 `l.as_bool()`）用的是正确 API。

## 2. 3275,2,3201（签名 B 剔除项）裁决

**bedrock_floor 先中解释成立，该点从签名 B 证据集剔除。**
- ctx：y=2，biome=soul，sdb=3, surface_depth=4, selector=+0.2566。
- [0] VerticalGradient bedrock_floor true_y=0 / false_y=5（above_bottom 0..5，JSON L112-118）：y=2 ∈ (0,5) → `splitter("minecraft:bedrock_floor").split_xyz(3275,2,3201).next_float()` 随机判定 → 生产判 **true** → Block(31)=bedrock → **整规则首中即返**（SurfaceRule::Seq L263-269 首中语义）→ applied=id=31。
- 该点 y=2 处于 bedrock 随机带内，本就不该用于 soul 表面判定（vanilla 同点也先判 bedrock_floor）。剔除后签名 B 证据集 = 其余 soul 判定点（y 在 bedrock 随机带之上者）。

## 3. 修复方向（一行级定位，不附 patch）

1. **主修**：`WorldgenRust/src/surface_rules.rs` `parse_surface_cond` —— 新增/使用布尔读取助手
   `x.as_bool().or_else(|| x.as_f64().map(|f| f != 0.0)).unwrap_or(false)`，替换 **L1079（y_above add_stone_depth）/ L1093（stone_depth add_surface_depth）/ L1116（water add_stone_depth）** 三处 `as_f64()` 布尔误读。`JsonValue::as_bool` 已存在（json.rs L18）。
2. **回归**：修复后重跑 soul_tree_repro（产物树应显示 asd=true）+ soul_ctx_dump（3260,1,3200 应 applied=258）+ 180 点全量对照；nether_wastes 签名 C「entered 0/60」预期同源缓解（[6] floor StoneDepth 同为 asd 误读，修复后 sda 口径变宽 → 需重测，不在本产物下结论）。
3. **V3 对拍方法补丁（防复发）**：静态参数对拍不能只「肉眼核对 JSON 值」，须核对**解析产物**（本 repro 的树 dump 即最小工具）；建议把 parse 产物树 dump 固化为 bin-diag 常备诊断。
4. 次生（非本次因果，V3 已列）：AboveY/Water `mult` 硬编码 0（L1080/L1115）；解析失败静默回退 overworld 规则（worldgen_handle.rs L253 unwrap_or_else）建议 fail-fast。

## 4. 五段式（错误台账要素）

- **现象**：soul 判定点 biome=soul ∧ ceiling_ok=true ∧ selector<0 → applied=256（netherrack），probe 重组同 ctx 复跑同 rule 同样 256；生产 dump stderr 全程 0 条 SURFACE-WARN（无解析跳过迹象）。
- **根因**：parse_surface_cond 以 `as_f64()` 读 JSON 布尔字段，布尔→None→恒 false；StoneDepth 的 `1+offset+surface_depth` 退化为 `1+offset`，soul/wastes/basalt 分支条件面整体收窄，soul 分支该进未进，穿透至 [7] netherrack 兜底。
- **定位**：V3 静态对拍排除「结构缺失」→ V4 生产 ctx dump 否定「输入差」→ 本轮 bin-diag（soul_tree_repro）直接 dump 解析产物树，树参数 vs JSON 行号逐项对拍 → asd=false 与 JSON true 冲突即锁定；定点 apply 复现 256/31 两个签名值。
- **修复**：见 §3-1（三行布尔读取替换；未实施，待主会话应用+回归）。
- **教训**：①「参数全对拍」必须对拍**解析产物**而非 JSON 原文——中间层（解析器）本身是嫌疑对象；②数据驱动解析器对 JSON 标量类型（bool vs number）必须用类型感知 API，`as_f64` 万能读取是静默语义腐蚀通道；③probe「复算一致」只能证明 probe 与生产同源，不能证明与 JSON 规范同源——三方（JSON / 解析产物 / 运行时）对拍缺一不可。

## 5. 状态与可比性

- status: draft（AI 不授予 candidate 以上；confirmed 留待用户）。
- 证据饱和：本轮产生新数据层证据（解析产物树实跑 dump + 定点 apply 复现），retry 计数重置。
- 产物引用：.investigations/soul-v4v5/cmd-output/soul-tree-repro.txt（树 dump + apply）；bin-diag/soul_tree_repro.rs（临时诊断 bin，已按纪律留在 bin-diag 隔离区）。
