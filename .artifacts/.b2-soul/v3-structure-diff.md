---
id: v3-structure-diff
topic: b2-soul
title: V3 静态结构对拍——nether surface_rule JSON vs Rust 解析（签名 B/C 归因）
status: draft
验证分层: Degraded（纯静态阅读对拍，无运行时证据；子代理分析，未编译/未执行）
可比性声明: 载体=JSON 数据树 vs Rust 解析器源码静态语义；覆盖面=nether.json surface_rule 全部节点类型 + surface_rules.rs parse_surface_rule/parse_surface_cond/SurfaceCond::test/build_surface 装配；与 V1/V2 动态探针口径不可比（本产物只回答「rule 结构差可否解释」，不回答「运行时输入差」）
date: 2026-09-08
---

# V3 静态结构对拍：nether surface_rule（JSON）vs Rust 解析

> **[supersedes 注 2026-09-09]** 本节「签名 B/C 结构差不可解释（需 V4 动态对照）」的处置方向已被
> `.artifacts/.b2-soul/v4-eval-conflict.md` 取代：矛盾最终定位为 **parse_surface_cond 布尔字段解析 bug**
> （as_f64 读 Bool → 恒 false，求值层 ≠ JSON 语义），非运行时输入差。本节「结构完整一致」结论仍成立；
> §1 的「参数全对拍」为假阴性——教训：静态对拍必须对拍**解析产物树**而非 JSON 原文（详见 v4-eval-conflict §4）。
> 原文不删不改，以本注记为准（§15.4 取代链）。

## 0. 对拍对象

- JSON：`versions/1.20.1/data/minecraft/worldgen/noise_settings/nether.json` surface_rule（L105-737）
- Rust：`WorldgenRust/src/surface_rules.rs`（parse_surface_rule L996 / parse_surface_cond L1030 / SurfaceCond::test L67 / build_surface L1096）
- 装配：`WorldgenRust/src/worldgen_handle.rs` L217-223（noise key 动态收集）+ L249-256（rule 装配，nether 走 `parse_surface_rule(sr, min_y, noise_height)`，min_y=0, noise_height=128）

## 1. 对拍表（JSON 节点路径 → Rust 解析行为）

JSON 树中出现的节点类型全集：`sequence / condition / block / vertical_gradient / not / y_above / biome / stone_depth / noise_threshold / hole`。**无 steep / water / temperature / surface / above_preliminary_surface 节点。**

| # | JSON 节点路径 | JSON 参数 | Rust 解析行为 | 与 B/C 因果关联 |
|---|---|---|---|---|
| 1 | root `sequence[7]` | — | `SurfaceRule::Seq`，逐项解析，**顺序保留** | 无差异；首中即返语义与 Java materialRule 一致 |
| 2 | [0] bedrock_floor：`vertical_gradient(above_bottom 0..5)` → bedrock | true_y=0,false_y=5 | 正确解析（absolute 换算：above_bottom=+min_y+v） | 无 |
| 3 | [1] `not(vertical_gradient bedrock_roof, below_top 0..5)` → bedrock | true_y=122,false_y=127 | 正确解析（below_top=min_y+height-1-v，height=128） | 无 |
| 4 | [2] `y_above(below_top 5, mult=0)` → netherrack | surface_depth_multiplier=0 | 正确解析；⚠️ parser **硬编码 mult:0**（L1051），JSON 的 surface_depth_multiplier 字段被忽略 | nether 全部 y_above 均 mult=0 → **本维度无影响**（跨版本风险点，非本次因果） |
| 5 | [3] basalt_deltas 分支：biome → seq[stone_depth(ceiling) → basalt; stone_depth(floor) → seq[patch/gravel 链, nether_state_selector→basalt, blackstone]] | — | 全部类型支持，完整解析 | 非 B/C 范围 |
| 6 | **[4] soul_sand_valley 分支（签名 B 目标）** | 见下 | **完整解析，结构与 JSON 一致** | **判定见 §2** |
| 6a | [4].ceiling：`stone_depth(add_surface_depth=T, surface_type=ceiling)` → seq[selector≥0→soul_sand; **soul_soil 兜底 block**] | offset 0, sec 0 | 参数全对拍（ceiling→stone_depth_below）；seq 末尾 soul_soil 无条件兜底 **存在** | B：若 entered 且 selector<0 走到此 seq，**必然**返回 soul_soil，不可能落空到 netherrack |
| 6b | [4].floor：stone_depth(floor) → seq[patch(-0.012)→y_above(30)→not y_above(35)→gravel; selector≥0→soul_sand; **soul_soil 兜底**] | — | 完整解析；y_above add_stone_depth=true 正确传递 | 同上 |
| 7 | [5] 通用 floor 段（lava hole 链 + warped/crimson 嵌套 biome 条件） | — | 完整解析；`hole`→stone_depth_above≤0 正确 | 无 |
| 8 | **[6] nether_wastes 分支（签名 C 目标）** | 见下 | **完整解析，结构与 JSON 一致** | **判定见 §3** |
| 8a | [6].floor+surface_depth：stone_depth(floor, add_surface_depth=T) → noise_threshold(**soul_sand_layer**, -0.012) → seq[not(hole)→y_above(30)→not y_above(35)→soul_sand; netherrack] | — | 完整解析；noise_threshold 的 noise key `minecraft:soul_sand_layer` 由 collect_noise_keys 递归收集（if_true/then_run/invert/sequence 全路径覆盖）→ 预加载表含该 key | C：分支**存在**，非「缺失」 |
| 8b | [6].floor：y_above(31) → not y_above(35,+stone_depth) → gravel_layer(-0.012) → seq[y_above(32)→gravel; not(hole)→gravel] | — | 完整解析 | 非 C |
| 9 | [7] 兜底 `block netherrack` | — | 正确解析（Seq 末项） | 无 |
| 10 | noise 参数源（soul_sand_layer 等 6 key） | — | 密度参数在 density_builder.rs L72 注册（soul_sand_layer 已见）；采样 (x,0,z) 与 Java NoiseThresholdCondition 一致 | 无静态差异 |

静态偏差全量清单（均判非本次因果）：
1. **AboveY/Water 的 multiplier 硬编码 0**（parser 忽略 JSON surface_depth_multiplier/负 mult）——nether 全为 0，无影响；overworld 代码规则不受影响；跨版本升级风险点。
2. parse_surface_cond 内 **vertical_gradient 解析块重复两份**（L1054 与 L1075，后者不可达）——无行为影响，代码卫生问题。
3. sequence 条目解析失败 → **静默跳过**（仅 stderr warn）；condition 解析失败 → 整分支跳过；整个 rule 解析失败 → **静默回退 overworld 规则**（L253 unwrap_or_else）——静态核对 nether 全部类型均在支持集内，**不触发**；但这是「结构性静默缺失」的潜在通道，建议后续把回退改为 fail-fast（改进建议，非本次因果）。
4. build_surface 的 default_block 匹配占位为 `minecraft:stone`（非 settings.default_block=netherrack）——占位语义：宏观 Rock→stone id，rule 命中后由 [7] 兜底写回 netherrack，净效果正确；但 ore_vein 在 surface 前替换出的非 stone id 点位不会被 rule 处理（nether ore_vein 禁用，无影响）。

## 2. 签名 B 判定（soul_soil 子分支：entered=true 且 selector<0 → applied=netherrack）

**判定：结构差不可解释（需 V4 动态对照）。**

理由链：
- soul 分支（含 ceiling/floor 两个子分支、nether_state_selector 阈值条件、soul_soil 兜底块）在 Rust 解析后**结构完整且顺序一致**；所有节点类型均在解析器支持集内。
- 语义推演：若生产运行时真的以 `biome=soul_sand_valley ∧ (ceiling||floor stone_depth 成立)` 进入该 Cond 分支，则 Seq 内 soul_sand(selector≥0) 不中后**必然**命中 soul_soil 兜底 block，返回值不可能是 None（不可能穿透到 [7] netherrack 兜底）。
- 因此「entered=true 且 selector<0 仍 netherrack」只能是：① 生产运行时该点**实际未进入**该分支（probe 复算的 entered 输入与生产 ctx 不一致——biome 取点/stone_depth_above/below 列扫描语义），或 ② probe 的 selector 与生产 sampler 实例/参数不一致。两者均为**运行时输入差**，静态结构对拍排除结构侧，转 V4 动态（建议：生产链路在 soul 分支入口加一次性诊断 dump 或复用 soul_selector_probe 直连生产 rule+ctx 构造路径）。

## 3. 签名 C 判定（nether_wastes floor 侧 soul_sand_layer 分支：组3 entered 0/60）

**判定：结构差不可解释（需 V4 动态对照）；且「分支缺失」假说被静态对拍否定。**

理由链：
- 该分支在 JSON L564-644 与 Rust 解析产物中**逐节点一一对应**（stone_depth(floor, add_surface_depth=true) → noise_threshold(soul_sand_layer, -0.012) → not(hole) 链 → soul_sand / netherrack），不存在缺失。
- 「entered 0/60」在结构完整前提下只能由输入侧导致：① 该 60 点 biome 实际非 nether_wastes（与签名 A 足迹偏移同源的可能性**不能排除**——但那是 biome 层差异，非 rule 结构差）；② soul_sand_layer 采样值 ≥ -0.012（噪声参数/实例差）；③ stone_depth 输入差。
- 附注（A 签名挂载点核对，非本对拍结论范围）：JSON 中 biome 条件挂载为三个顶层分支（basalt_deltas/soul_sand_valley/nether_wastes）+ warped/crimson 嵌套于通用 floor 段——Rust 结构与挂载点**一致**；Rust 侧 biome 谓词为纯字符串相等比较，无挂载差异。签名 A 的足迹偏移应归因 biome 分类层（build_surface 的 biome_at 走 `(x>>2)<<2` 对齐采样，L437），非 rule 结构。

## 4. 结论摘要（draft，建议）

- 签名 B：**结构差不可解释** → V4 动态对照（生产 ctx vs probe ctx 的 biome/stone_depth 输入）。
- 签名 C：**结构差不可解释**，「分支缺失」被否定；「同源于签名 A 的 biome 足迹偏移」升为 C 的并列主候选（biome 层，非 rule 层）。
- rule 解析器静态偏差 4 项均判非本次因果；1 项改进建议（解析失败回退改 fail-fast）。
- 本产物为 Degraded 静态对拍，不授予 candidate 以上状态；confirmed 留待用户。
