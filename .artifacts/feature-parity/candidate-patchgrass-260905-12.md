# FEA-11 · candidate：patch_grass 系空 id generate_nested miss 修复（260905-12）

> 状态：**confirmed（用户拍板 260905-12）**；judge APPROVE-WITH-CONDITIONS 260905-12，条件已核销
> 过程：`.investigations/feature-parity/260905-12-patch-grass-inline-fix.md`

## 缺陷与根因

`[feature-loader] generate_nested: unknown id (placed+configured miss): `（空 id）。
根因 = 18 个 `patch_*.json` + 6 个 `flower*.json`（共 **24 内联点位**）的内嵌 `feature` 字段为**内联 configured feature 对象**（Java `Holder.direct`，无 id），`PlacedFeature::parse_inline` 只支持字符串形态 → 静默吞成空串 id → cache 双 miss。

## 修复

- `placement.rs`：`PlacedFeature.inline_configured: Option<Box<ConfiguredFeature>>`（Box 断 CF→PlacedFeature→CF 递归环）；parse_inline 枚举三形态（id 字符串 / 内联对象 / 缺失）。
- `feature_loader.rs`：generate_configured random_patch 分支 inline 直发（优先于 id 查 cache）；generate_nested placed 路径同判（**死防御注记：当前数据 placed JSON 内联对象 0 个，分支不可达**——judge WARN #2，残差专项开工前如需启用须镜像 patch 分支走 pf.generate placement 链）；3 处构造补 None。

## 验证（features_probe release × recheck 6×6 参照，stash/pop 单变量）

| 臂 | miss | match |
|---|---|---|
| 修复前 | 8766 | 94.99% |
| 修复后 | 0 | 94.78% |

RNG 中性（judge 专项核对）：Simple/Weighted StateProvider 消费 0/1 均同 Java；修复前 miss 路径与修复后 simple 路径均 0 消费。

## 残差边界（§9.7 + judge #3/#4 修订）

修复后该参照区 rust 新放 7241 grass + 261 tall_grass、vanilla 全 0——**因果标注：这是修复激活 patch 实际放置后暴露的上游差异（修复前同点位 miss 短路=不放），非本修复语义错误**。RNG 消费中性已证，排除直发时机/消费差。归「patch feature 选择 / biome 门」域，fan-out 候选集：
1. biome feature 列表数据/映射差（badlands 列表）
2. dripstone_caves step9 含 patch_grass_plain（vanilla 事实）+ biome 门差
3. patch 选型差（patch_grass vs patch_grass_badlands）
4. unsupported placement modifier（noise_threshold_count 等，rel log 大量）+ biome features 顺序/global_index 差（judge #4）

口径：features_probe vs vanilla FULL 6×6，与 v18（2193 chunks treediag）不可比。
