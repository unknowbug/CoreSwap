---
编号: 000
任务: 1.21.6 移植遗留 P5 判定——核 ChunkStatus 邻居依赖语义是否被 Rust 侧消费
任务类型: swe / 验证（收敛型静态核对）
模式档位: 轻量
日期锚: Get-Date 2026-09-08 21:23；git 锚 9057871
状态: 已批准（用户 260908-13 拍板「仅 P5」）
---
## 范围（含明确不做什么）
- 做：静态核对 1.21.6 ChunkStatus 邻居依赖语义（Java 侧）与 Rust 消费侧，二选一产出：
  - 需要 → 产出「邻居依赖小表」并核 Rust 侧是否正确消费；
  - 不需要 → 记「不做」判据 + §9.7 覆盖面声明。
- 不做：P6 SPAWN barrier 探针（维持挂账）；F2 spread-replay（不排程）；I1 下钻（不做）。

## 任务拆解（子任务 → 预期产物）
1. Java 侧定位：1.21.6 ChunkStatus 邻居依赖语义（distance/依赖图生成逻辑）→ .investigations/mc-1216-port-260908-13/ 笔记
2. Rust 侧核对：worldgen-core / versions/1.20.1/rust（及 1.21.6 薄壳）是否消费邻居依赖语义
3. 判定落盘：.artifacts/mc-1216-port/p5-verdict-260908-13.md（draft）+ index.yaml 登记

## 验证方式
- 静态核对为主 → Degraded 降级声明；如可廉价加探针/复现点则升 Partial。

## judge 预置
- candidate 授予：SHOULD judge（subagent）
- 收尾交付：MUST judge（三源核对：artifacts 快照 + git diff + 验证记录）

## fan-out 预置
- 无预置分叉点；若核对中出现 ≥2 互斥机制候选即触发 fan-out（.bN 并行，禁止主会话自推）

## 知识库更新
- 结论性落盘（如发现可复用模式）：subagent 产出草稿（core.worker）+ 主会话应用验证；一次性结论不写知识库

## 子角色介入点
- scout: 否（语义范围明确，非机制未明大排查）
- worker: 判定为收敛分析，主会话直接做；知识库草稿 subagent 产出
- fan-out: 无预置（分叉即触发）
- judge: candidate SHOULD / 收尾 MUST
- knowledge: 结论性落盘 MUST subagent 产出
