# 00 — end 接管摸底 verdict（260906-04）

> 状态：draft（待 SHOULD judge → 用户批准）
> 依据：01-rust-status.md + 02-end-biome-rules.md（subagent）+ 03-resource-closure.md
> 验证分层：Partial（全部静态审查，无运行时 probe）——后续开发 Phase 2.5 按 §对拍计划补运行时证据

## 摸底结论：缺口定性 = 中小，建议轻量架构（3 要点）

对照 NEXT_SESSION 预期「摸底后按缺口大小定轻/重量」——摸底结果显示缺口**明确且有限**，无互斥方案分叉，无管线级重构，无需重量架构：

**已验证事实**（一手源码/数据核对）：
1. `create_for_dim` 全参数化已就绪，end.json 维度参数（min_y 0/height 128、aquifers false、legacy_random、单 block surface_rule、vein router 字段）全部被既有路径覆盖。
2. 资源层无缺失文件，end.json 引用闭包自包含。
3. end biome = 纯二维 section 级位置判定（非 MultiNoise，假设已验证），规则完整提取（阈值/坐标语义/种子链齐全）。

**缺口清单（全部开发项）**：
- G1 `SimplexNoiseSampler`（worldgen-core 缺失，~100 行）+ `EndIslands` DF（float 域逐行对齐，CheckedRandom(seed)+skip(17292)，worldSeed 直传无 split）+ density_builder 注册 `minecraft:end_islands` 分支。
- G2 `EndBiomeSource` 分类器（~30 行纯函数，与 MultiNoise BiomeClassifier 并列不复用）+ `create_for_dim` 按 settingsName=end 路由（含 biome 加载 5 key、sea_level=63 对 end 无害性确认）。
- G3 Java 侧 mixin 接管集加 `minecraft:end` + 端到端 vanilla 逐位对拍（Forge 生产环境，遵守 #55 生产命名环境验收判据）。

## 轻量架构 3 要点（待用户批准后另出正式开发计划细化）

1. **Rust 内核**：SimplexNoiseSampler + EndIslands 节点 + EndBiomeSource 分类器，create_for_dim 端到端接通（unit 对拍：simplex 置换表/erosion 值/阈值边界 section）。
2. **Java 接管**：mixin 接管集 + minecraft:end；生产环境行为日志验收（#55 判据）；`create_for_dim` L309-316 无条件 `BiomeClassifier::load(biome_params_file)`（end 无 params 文件）需在此分流到 EndBiomeSource（judge CONCERN-2，正式计划必列）。
3. **验证闭环**：同 seed Java↔Rust end chunk 逐位对拍（seed 三查 + 口径声明 §9.7），judge MUST 收尾。

## 风险与回退
- 最大风险 = SimplexNoiseSampler float 域精度（02 篇 §2.2 警告）→ 对拍前置（置换表先对）可隔离。
- end.json/Java 参照提取链版本未回查（02 篇 idk 声明）→ 开工首日廉价核验（对照 vanilla report）。
- 报告者 1.0.25 复测反馈到达则按 BUG 卡契约优先插队。

## 知识库
本摸底无通用新坑，不落 docs（价值门）；开发块结论另走流程。
