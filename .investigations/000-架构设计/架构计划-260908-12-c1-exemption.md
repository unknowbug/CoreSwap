---
编号: 260908-12
任务: C1 豁免清单候选追加收口（fallen_tree + place_on_ground）
任务类型: 文档收口 + 数据核对（swe 收敛闭环）
模式档位: 轻量
状态: 待批准
会话块: 260908-12（Get-Date 2026-09-08 21:06 实锚）
---

## 范围（含明确不做什么）

- **做**：把 260908-10 verdict §4 已记录的 C1 豁免候选落到权威清单载体，可核对、防再挂账：
  1. **数据核对**：在 versions/1.21.6/data 与一手 1.21.6 数据源核对 `minecraft:fallen_tree`（5 变体）与 tree decorator `minecraft:place_on_ground` 的存在性 + Rust/Java 接管侧未支持现状（交接结论廉价验证——NEXT_SESSION 该条为挂账描述，动手前先验）。
  2. **权威清单落地**：新建 `.artifacts/mc-1216-port/c1-exemption-list.md`（单一权威载体），汇总 260908-10 verdict §4 既有项（DimensionPadding/TrialChambers padding、Shipwreck Y、4 项 feature、SPAWN barrier）+ 本次两项候选，每项标：机制/影响域/B 区「维持旧行为」/首次取证来源（§15.4 不可改写，仅追加）。
  3. **index.yaml 登记 + 知识库评估**：结论进 index（draft→candidate 走 judge）；知识库按价值门评估——本项为一次性结论为主，预期不派 discovered 草稿（如核对中出现新坑则升级）。
- **不做**：F2 spread-replay 字级验证（上轮已判低价值不排程）；P5/P6/③ 遗留（默认不动，见下方 HOOK 问询）；任何引擎/数据/探针代码改动（纯文档收口，除非核对推翻挂账描述）。

## 验证方式

- fallen_tree/place_on_ground：数据文件 grep + feature 解析/展开代码路径核对（静态，Full 不适用——文档收口）；核对结果与 260908-10 verdict 描述逐条对上，不一致即记错误台账。

## judge 预置

- 收尾 | SHOULD | 清单内容 vs verdict 原文忠实度 + index 登记（纯文档块，SHOULD 足够；若核对推翻原记录升 MUST）。

## fan-out 预置

- 核对若出现 ≥2 互斥机制候选（如「feature 确未支持」vs「已支持但数据未引用」冲突证据）→ MUST fan-out .bN；预期不触发。

## 知识库更新

- 价值门评估（预期：一次性结论不写知识库；有新坑才 subagent 草稿）。

## 子角色介入点

- scout: 否（已知路径文档收口）
- worker: 核对为收敛单假设，主会话直接做
- fan-out: 预置触发点见上（预期不触发）
- judge: SHOULD（收尾，subagent）
- knowledge: 价值门评估后定（预期不派）

## 人工 HOOK 点

- **H-A 本计划批准**（现在）。
- **H-B 范围问询**：P5（ChunkStatus 邻居依赖判定）/P6（SPAWN barrier 探针）/③ 是否纳入本块——默认不纳入。
