# 260905-12 patch_grass 系空 id generate_nested miss 修复 · 过程记录

> status: 修复完成，**judge APPROVE-WITH-CONDITIONS（260905-12，条件已核销）→ confirmed（用户拍板 260905-12）**。锚定 Get-Date = 2026-09-05 22:27 开工。
> 架构计划：.investigations/000-架构设计/架构计划-260905-12.md（方案 C 已批准）

## 现象

NEXT_SESSION 未闭合课题 #3：`[feature-loader] generate_nested: unknown id (placed+configured miss): `（**空 id**）。
本轮复现载体：features_probe（release）× recheck 参照 vanilla_8576294172403134396_6_720_-432.blocks（6×6，chunk 45..50,-27..-22）→ 修复前 **miss = 8766**。

## 根因

`patch_grass`（及同形态共 **24 内联点位** = 18 个 `patch_*.json` + 6 个 `flower*.json` random_patch/flower 系 configured feature——judge #1 计数更正，实测 260905-12）的内嵌 `feature` 字段是**内联 configured feature 对象**（`{"type":"minecraft:simple_block",...}`），不是 id 字符串。

- `PlacedFeature::parse_inline`（placement.rs:503）对 object 形态走 `as_str().unwrap_or("")` → `configured_feature = ""`。
- 运行时 `generate_nested("")` → cache.placed miss → cache.configured miss → 空 id 告警 ×N。
- Java 语义 = `PlacedFeature(Holder.direct(...))`：内联 configured 无 id，直发；非查表。

## 修复（worldgen-core）

1. `placement.rs`：`PlacedFeature` 增 `inline_configured: Option<Box<ConfiguredFeature>>`（Box 断 CF(selector)→PlacedFeature→CF 递归环）；`parse_inline` 对 object 形态直接 `ConfiguredFeature::parse("", obj, blocks)` 持有实体。
2. `feature_loader.rs`：
   - 3 处显式 PlacedFeature 构造补 `inline_configured: None`；
   - `generate_configured` random_patch 分支闭包：`inline_configured` Some 时直发（优先于 id 查 cache）；
   - `generate_nested` placed 路径同判（防 placed JSON 内嵌对象形态miss）。

## 验证（Phase 2.5）

| 臂 | miss | match（region 6×6） |
|---|---|---|
| 修复前（git stash 基线） | **8766** | 94.99% |
| 修复后 | **0** | 94.78% |

- 修复前后同 seed 同参照同二进制载体（features_probe release），stash/pop 单变量切换——基线对照合规（#20 死参数判别：miss 确实在此参照发生）。
- debug 构建会在 chunkrandom.rs:169 溢出 panic（debug 算术检查），生产语义 release 正常——非本修复引入。

## §9.7 口径声明

- match 数字载体 = features_probe（rust 全管线含 features）vs vanilla FULL 6×6 参照，与 v18 口径（2193 chunks 三臂 treediag）**不可比**。
- **−0.21% 残差已定位边界**：修复后 rust 在该参照区新放 7241 grass（id 123）+261 tall_grass（501），vanilla 全 0。**因果标注（judge #3）**：修复前同点位 miss 短路 = 不放置，修复后 patch 实际放置激活了此处暴露的上游差异——残差是「修复让放置真实发生」才可见，非修复引入的语义错误；RNG 中性已证（BlockStateProvider Simple/Weighted 消费 0/1 均同 Java 一手源，tree.rs:49-63，judge 专项核对）。该参照区为 terracotta/badlands 系（8576-24blocks 记录：(812,73,-337) terracotta）。判读：**不属于本修复语义域**——inline dispatch 语义本身与 Java 一致；多放 grass 的机制属「patch feature 选择 / biome 门」域（互斥候选 ≥2：① badlands biome feature 列表数据/映射差 ② vanilla 事实 dripstone_caves step9 含 patch_grass_plain + biome 门差 ③ patch 选型差 patch_grass vs patch_grass_badlands ④ unsupported placement modifier（noise_threshold_count 等，rel log 大量）+ biome features 顺序/global_index 差（judge #4 补）→ 按分叉即 fan-out 纪律**不在主会话自推**，留待专项（可与 jungle_l 采集共用 java 侧证据）。

## 环境注意

- rlib mtime 早于 dll 的红旗本轮为 **#6 假阳性**（fs::copy 保留 mtime），内容指纹（行为差异 + dll 22:41 重链）已核。
- runServer 侧 worldgen.dll 已重链新鲜；runtime/ mixin chunk 过滤（本块任务 1）编译绿，运行时效果验证并入下轮 java 采集。

## 教训（候选知识库条目，待 subagent 草稿）

- parse_inline 三形态枚举不全（字符串 id / 内联对象 / 缺失）→ 内联对象被静默吞成空串 id——「id 字段可能是对象」与 compiler-idioms #8（JSON 布尔 as_f64 恒 false）同族：**JSON 字段类型形态枚举不全 = 静默语义腐蚀家族**。
