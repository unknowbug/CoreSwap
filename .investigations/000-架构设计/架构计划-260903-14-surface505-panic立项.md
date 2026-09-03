---
编号: 000
任务: surface_rules.rs:505 大 region panic（`missing noise sampler`）立项排查 + 顺带 block_probe 存档口径 Full 回归
任务类型: 机制未明排查（re-code 域）+ 验证补充
模式档位: 重量（机制未明 + 可能 fan-out + 生产稳定性课题）
状态: 待批准
日期: 260903-14（实际 2026-09-03 22:10）
---

## 1. 全局视图

- **主目标**：定位并修复 `WorldgenRust/src/surface_rules.rs:505` 的 panic `missing noise sampler`——64×64 大 region sweep 至 ~2304-2560 chunk 处触发（证据：estopt-sweep-260903-12.txt 尾部原文在案），生产稳定性课题，数据截止于此，4096 chunk 全程未完成。
- **顺带目标（可选，不阻塞主线）**：翻默认（WG_EST_SHARED/WG_EST_L2）后补 block_probe 存档口径 Full 回归（上轮 verdict 已诚实声明 Partial 口径）。
- **排除项**：不做新性能优化；不动 est 两开关语义；不改 Java 探针。

## 2. P0 交接结论廉价验证（开工前置，MUST）

按交接验证纪律，先独立验证「panic 现象」本身可复现（不把上轮日志当公理）：
- 复跑一次大 region sweep（64×64 或缩窗），确认 panic 复现 + 记录确切 chunk 坐标/调用栈回溯（RUST_BACKTRACE=full）。
- 若不复现 → 升级人类，回到环境差异排查（seed/世界状态/二进制版本三查）。

## 3. 角色分配 & 任务拆解

| Phase | 内容 | 角色 | 产物 |
|---|---|---|---|
| 1 | 勘探：sampler 预加载链路摸底——collect_noise_keys 收集范围 vs 运行时实际查询的 noise key 集合；panic 点 505 的 get(key) 来源；触发 chunk 的 biome/区域特征 | **recode-scout**（subagent，只读，MUST 前置——机制未明） | .investigations/panic-505/ 管线地图 + 候选缺失机制清单 |
| 2 | 收敛/对比分析：把 sweep 崩溃点 chunk 与 2304-2560 区间已成功 chunk 的 biome/规则树 diff，锁定缺失 key | worker（若单假设，主会话收敛；多假设→fan-out） | .artifacts/panic-505/ draft + index.yaml |
| 2F | fan-out（条件触发） | 各 worker 并行 .b1/.b2 | 候选产物 |
| 2.5 | 验证：修复后 4096 chunk 全程 sweep 复跑（panic 消失 + hash/输出与既有口径可比性声明 §9.7）+ 常规回归 | 主会话执行 | cmd-output + 验证记录 |
| 3 | judge 审查 | core.judge（subagent） | review-*.md |
| 4 | 知识库更新 | subagent 草稿 + 主会话应用 | docs/07 或 06 篇小节 + 10 时间线 + discovered 条目（如适用） |

## 4. 人工决策 HOOK 点

1. 架构批准（本文档，现在）。
2. fan-out 竞争裁决（若分叉）。
3. 修复方案方向（若涉及数据驱动边界，如噪声表加载策略）。
4. confirmed 授予（judge 通过后）。

## 5. 风险 & 回退

- 风险①：崩在运行时探针（estopt bin）路径而非生产路径 → 需先核「微测引用生产采样形态」（workflow-patterns #21），必要时换生产入口复现。
- 风险②：触发条件依赖大 region 顺序状态（缓存/预加载时序）→ 缩窗二分定位最小复现 chunk 区间。
- 回退：修复仅动 sampler 加载/兜底逻辑，不碰已 confirmed 的 est/角参数结论。

## 6. judge 步骤预置

- 节点：根因定论 | MUST（重大定论） | 审查对象：artifacts 快照 + git diff + 复现/修复验证记录
- 节点：收尾交付 | MUST | 三源核对
- 节点：修复 candidate | SHOULD

## 7. fan-out 步骤预置

- 分叉点：缺失 key 的机制候选 ≥2 互斥（如 a. collect_noise_keys 收集面不全（某 surface rule 嵌套分支未遍历）/ b. 运行时规则树按 biome 动态产生收集期不存在的 key / c. 预加载表被某路径绕过）→ MUST 并行 .bN，禁止主会话自推。

## 8. 知识库更新

- 结论性 docs/discovered：subagent（core.worker，prompt 带 SUBAGENT-KNOWLEDGE-GUIDE.md 指引）产出草稿 → 主会话应用 + 验证。
- 错误台账：.investigations/panic-505/panic-errors.md 五段式（本课题新建）。

## 9. 子角色介入点（全部预置）

- scout: Phase 1 MUST（机制未明勘探，管线地图）— subagent
- worker: Phase 2 分析解读 / 修复代码交付（如需隔离交付）
- fan-out: §7 分叉点条件触发
- judge: §6 各节点
- knowledge: §8 结论落盘 subagent 产出

## 10. 顺带任务（独立小包，主线空闲时）

- block_probe 存档口径 Full 回归（翻默认后证据补齐）：主会话执行 block_probe 逐位对比，产出登记。
