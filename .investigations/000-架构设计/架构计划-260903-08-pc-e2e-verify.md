---
编号: 000（260903-08）
任务: P-C 端到端验证——WG_GPU_CHANNELS 开/关 A/B vs Java（≥256 chunks）+ 0.61× 双线程无探针复测
任务类型: 验证（性能对齐 + 既有结论数据层复测）
模式档位: 轻量
状态: 待批准
session: 260903-08（实际 2026-09-03 16:05，锚 dc865fe@15:17）
---

## 范围（含明确不做什么）

**做**：
1. **P-C1 端到端 A/B**：WG_GPU_CHANNELS 开/关 × Java 原版三方端到端对比，≥256 chunks 大样本（WorldGenBench、充分预热、稳定中位数、排除冷启动 chunk）。§9.7 三要素预声明：端到端口径 = block_probe 级（judge D2 已定），非通道级 f32 / final 3.128e-07 口径。
2. **P-C2 0.61× 复测**：无探针整批 wall + 调用计数两步走，验 260903-05「双线程 0.61×」异常真伪；顺带补 P0「fill 全同步串行」Degraded 结论的数据层验证。

**不做**：
- GPU 性能优化（dispatch 复用/batch 合并——独立工作包，非阻塞）。
- N1/H3/glslc 欠账（P-C 后排，本包不排入）。
- 任何生产代码修改（零退化铁律：本包纯验证；若暴露 bug → 停，回 Phase 0 重评估）。

## 交接结论继承验证（2026-09-01 纪律，先做）

| 继承结论 | 验证动作（廉价，≤一轮） | 状态 |
|---|---|---|
| WG_GPU_CHANNELS confirmed（ch0 通道级） | 已 judge+用户 confirmed，无需重验 | ✅ 可继承 |
| 0.61× 双线程异常 | 本包 P-C2 即验证对象 | 🔍 待测 |
| P0 fill 全同步串行（Degraded） | P-C2 顺带数据层验证 | 🔍 待测 |
| 参照 seed 8576294172403134396 | 开测前 server.properties + 输出 header 三查 | 开工首步 |

## 任务拆解

1. **前置核对**：环境状态核对（工作树干净/server.properties seed/无残留 java 进程/各 WG_* 默认关）→ seed 三查落盘。
2. **P-C2 先行**（快、无 Java 依赖）：无探针整批 wall + 调用计数，M=1/2/4 线程 bench；判读 0.61× 真伪 + fill 串行验证。
3. **P-C1 主项**：Java WorldGenBench 大样本（预热 + 中位数）× C++/Rust WG_GPU_CHANNELS 关 × 开，三组数据落盘 cmd-output/。
4. **判读**：性能对比结论（draft → candidate），零退化核验（门控关 ≡ 主线）。

## 验证方式

- 端到端 wall 时间 + 稳定中位数；§9.7 载体/覆盖面/可比性同行声明。
- 遵守：端到端 vs Java 大样本铁律 / 测量探针污染铁律（只信无探针 wall + 计数）/ bench 吞吐 vs 每 chunk 延迟区分。

## judge 预置

- P-C1+P-C2 结论 candidate 授予前：SHOULD judge。
- 收尾交付：MUST judge（三源核对：artifacts 快照 + git diff + 验证记录）。

## fan-out 预置

- 0.61× 若复测仍异常且出现 ≥2 互斥机制候选（如 线程池竞争 vs 内存带宽 vs clamp 串行）→ MUST fan-out .bN 并行，禁止主会话自推。
- 单一结果（异常消失/复现且归因单一）→ 收敛分析主会话直接做。

## 知识库更新

- 结论性 docs（10 时间线 260903-08 节 + 相关主题篇追加）/ discovered（若有新判错模式）：subagent 产出草稿 + 主会话应用验证（prompt 含 SUBAGENT-KNOWLEDGE-GUIDE 行）。

## 子角色介入点

- scout: 否（机制已知，纯验证；0.61× 若复现且机制未明 → 补勘探点）。
- worker: 判读阶段（bench 数据解读）subagent 交叉核读 SHOULD；结论落盘 MUST subagent。
- fan-out: 见上（0.61× 多候选分叉点）。
- judge: candidate SHOULD + 收尾 MUST。
- knowledge: Phase 末尾 subagent 产出。
