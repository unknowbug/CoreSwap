---
编号: 000
任务: end 接管课题 —— 半天摸底（Rust end 支持现状 / end biome 判定方案 / 资源完整性）→ 轻/重量架构决策
任务类型: 摸底勘探（re-code/swe 混合，结论决定后续开发架构）
模式档位: 轻量
状态: 待批准
session: 260906-04（命名前 Get-Date 锚定）
---

## 范围（含明确不做什么）

**做**（纯只读摸底，不写生产代码、不改 mixin）：
1. Rust 侧 `create_for_dim` / end 分支现状：aquifers、sea_level、surface_rule JSON 解析对 end.json（noise_settings/minecraft/end）的覆盖与硬编码点（对照 C++ `wg_create` 参数化维度先例）。
2. end biome 判定缺口：Java `TheEndBiomeSource` 中心/高地/边缘位置判定规则提取（yarn sources 只读参照），评估 Rust BiomeClassifier 需新写的内容。
3. 资源层 `density_function/end/*` 及 noise_settings/end.json 引用闭包完整性核对（缺哪些生成文件）。
4. 摸底产物汇总 → 轻/重量架构建议 → 用户批准。

**不做**：end biome 实际编码、Java 接管集加 minecraft:end、任何 1.0.25 之后的改动；1.0.25 复测反馈若到则按 §十 BUG 卡契约优先插队。

## 任务拆解（子任务 → 预期产物）

- T1 Rust end 分支现状盘点（worker 分析，主会话可做——收敛型单假设读码）→ `.investigations/end-takeover/01-rust-status.md`
- T2 TheEndBiomeSource 判定规则提取（re-code worker subagent，读 Java 参照）→ `.investigations/end-takeover/02-end-biome-rules.md`
- T3 资源闭包核对（脚本比对 end.json 依赖 vs 现有 data 文件）→ `.investigations/end-takeover/03-resource-closure.md`
- T4 汇总 + 轻/重量架构建议 → `.investigations/end-takeover/00-recon-verdict.md`

## 验证方式

摸底结论逐条附源码/文件行号证据（一手核对，不引交接文档转述）；方向性陈述标注「已验证事实」vs「待开发确认假设」。

## judge 预置
- 摸底汇总 verdict（含架构建议）：SHOULD judge（candidate 授予前）。
- 后续正式开发块另立计划，收尾 MUST judge 三源核对。

## fan-out 预置
- 摸底为只读盘点，无互斥假设分叉 → 本阶段无 fan-out；正式开发若出现 ≥2 互斥机制候选（如 end 噪声参数解析两种方案）即触发 fan-out .bN。

## 知识库更新
- 摸底本身是过程记录 → 只进 .investigations/，不派 subagent 写结论性 docs（价值门：无裁决结论不落 docs）。正式开发块的结论另按纪律走。

## 子角色介入点
- scout: T2 Java 判定规则勘探用 recode-scout/worker subagent（divergent 读参照源码）；T1/T3 主会话收敛读码直接做。
- worker: T2 判定规则解读 subagent 产出；T4 汇总主会话。
- fan-out: 无（摸底阶段）；开发阶段分叉即触发。
- judge: 摸底 verdict SHOULD judge。
- knowledge: 本阶段不写结论性 docs；若摸底踩出新通用坑按价值门即时入 discovered。

## 交接验证动作（纪律前置）
NEXT 中「create_for_dim 单世界 overworld 硬编码」「end.json 未覆盖」为上轮方向描述，T1 第一步直接读 `worldgen-core` 源码一手核实，不作为公理。
