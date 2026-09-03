---
编号: 000
任务: Q-AQ1 立项——Rust aquifer 段为何慢 ~37ms/chunk（vs Java）归因
任务类型: 性能归因（swe 收敛 + 数据层差分判别）
模式档位: 轻量
状态: 待批准
---

## 范围（含明确不做什么）

- **做**：定位 Rust aquifer 段 ~37ms/chunk 的慢因——先判「次数多」vs「单次贵」，再定位到具体机制。
- **做（顺手小修）**：`WorldgenRust/src/bin-diag/pc_e2e_bench.rs` L18 解析 WG_E2E_SEED 后 L22 恒用常量 SEED 的死代码（改用 seed 变量 + 运行时打印生效 seed）——Q-PD1 遗留。
- **不做**：本包不做优化实现（归因定论后另立优化包）；不动 gpu-batch-merge 优先级议题（需与用户重议，不在本包）；不动生产代码（诊断默认 env 门控关）。

## 任务拆解（子任务 → 预期产物）

1. **环境预检 + 基线锚定**：无残留 java 进程；确认 WG_* 计数器门控默认关；跑一次 Rust OFF 基线（复用 qpd1_stage_bench，64 chunks，对 08/09 日 70-77ms 带核对）→ cmd-output 原始输出。
2. **计数器采数（判别实验第一步）**：启用现成 WG_AQUIFERCOUNT/WL/BP 计数器采 split/sample/miss 频次（先自检 #20：确认计数器路径真被执行、自变量真生效）；与 Java 对应侧对比（Java 侧若缺对应计数器，则按需补探针或降级为 Rust 单侧频次 + 机制推理，声明分层）→ cmd-output + .investigations 过程记录。
3. **判读收敛**：「次数多」→ 差异在调用结构（split_xyz+random / 查找结构）；「单次贵」→ 差异在 barrier.sample / 数据访问模式。此处分叉 ≥2 互斥候选时 MUST fan-out（见下）。
4. **顺手小修**：pc_e2e_bench.rs L22 seed 死代码修复 + 编译 + 一次性运行验证打印生效 seed。

## 验证方式

- 基线对带核对（70-77ms）；计数器恒等式自检（总调用数 = 各路径计数和）；判别结论须两轮稳定或正反配置 A/B；§9.7 口径三要素声明（载体/覆盖面/与 Q-PD1 62 口径可比性）。

## judge 预置

- candidate 授予前 SHOULD judge（归因结论）；收尾交付 MUST judge（三源核对：artifacts 快照 + git diff + 验证记录）。

## fan-out 预置

- 若计数数据不能封闭到单一机制（候选：a 邻居随机偏移 split 多次调用 / b barrier.sample 采样次数多 / c 查找结构/hash 低效）→ MUST 并行 fan-out .bN worker 候选，禁止主会话逐个自推。

## 知识库更新

- 结论性 docs/discovered 写入：subagent 产出草稿（core.worker + SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用验证；新坑进 workflow-patterns / build-tooling。

## 子角色介入点

- scout: 否（管线摸底已完成，Q-PD1 已定位段位）——机制未明仅限 aquifer 内部，计数器采数属收敛实验。
- worker: 计数数据解读、Java 侧机制比对分析、knowledge 草稿产出（subagent）。
- fan-out: 判读收敛处三互斥候选分叉时（MUST）。
- judge: candidate 前 SHOULD + 收尾 MUST。
- knowledge: 课题末尾 subagent 产出。
