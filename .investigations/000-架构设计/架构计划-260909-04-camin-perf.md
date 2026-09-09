---
编号: 000
任务: WG_CA_MIN 翻默认开后 features 阶段 +68% 成本的性能优化（成本回收）
任务类型: 性能优化（swe 域，Rust worldgen-core）
模式档位: 轻量
状态: 待批准
会话编号: 260909-04（实际 2026-09-09 17:32 起）
基线: git 2bea71c；背景 = .investigations/mc-1216-features-takeover/t3-rerun-record-260909-03.md（CA_MIN 定位）+ NEXT_SESSION 260909-03
---

## 范围（含明确不做什么）

- 做：定位 CA_MIN +68% 成本结构（features 阶段），提出并实现优化（算法/数据结构/惰性化/按需构建），恢复大部分成本，**行为逐位不变**（terrain/veg 输出 hash 级等价）。
- 不做：mask 翻转、tree placer 实装（B13 族）、CA_MIN 语义回退（默认开为用户拍板，不动）、GPU 方向（已三次否决，#82）。
- 优化验收双门：① 端到端 vs Java 原版（AGENTS §四 端到端铁律，大样本）；② CA_MIN on 与 off 输出逐位等价（行为恒等哨兵，#76 同代码双臂模式）。

## 任务拆解

1. **Phase 1 勘探（scout，subagent 只读）**：worldgen-core 内 CA_MIN 通路地图——数据结构、构建时机（per-chunk 全量 vs 惰性）、读写调用点、+68% 成本候选（构建成本/缓存失效/锁竞争/冗余拷贝），产物 .investigations/camin-perf/ 管线地图。
2. **Phase 2 成本测量（主会话，遵守测量污染铁律）**：无探针整批 wall + 调用次数计数，CA_MIN on/off 分臂串行 bench（噪声基线先行，#28 串行铁律），定位成本主导项。测量数据交 worker/subagent 解读。
3. **Phase 3 优化实现（收敛，主会话）**：单假设逐项收敛改；若分叉 ≥2 互斥优化方向且需竞争裁决 → fan-out。
4. **Phase 4 验证**：行为等价（确定性 dump/输出 hash 对比 on vs off 改前改后）+ 性能对比（同口径大样本，目标：CA_MIN on 成本回落至接近 off，量级判据开工点由 Phase 2 实测定）。
5. **Phase 5 judge + 知识库落盘**。

## 验证方式

- 行为等价：优化前后 CA_MIN=on 输出 hash 逐位等价（golden 冻结对照，#47）。
- 性能：串行分臂 wall 中位数 + 端到端 vs Java（≥256 chunks 大样本，#83 分场景声明：本课题目标口径 = 吞吐摊销）。

## judge 预置

- 收尾交付 MUST judge（三源核对：快照 + git diff + 验证记录）。
- Phase 2 成本定位结论 candidate 授予 SHOULD judge。

## fan-out 预置

- 分叉点：Phase 2 后若成本归因出现 ≥2 互斥候选（如「构建成本主导」vs「读放大主导」）→ MUST fan-out（.bN 并行），禁止主会话自推。

## 知识库更新

- 结论性 docs/discovered 写入：subagent 产出草稿（先读 knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用验证；时间线 → versions/1.21.6/docs/10-timewise-archive.md 260909-04 块。

## 子角色介入点

- scout: 是 — Phase 1 CA_MIN 通路勘探（机制未明，禁止直接跳单点定位），subagent 隔离。
- worker: Phase 2 测量数据解读 / Phase 5 知识库草稿（core.worker）。
- fan-out: Phase 2→3 分叉点（见上）。
- judge: 收尾 MUST；成本定位 candidate SHOULD。
- knowledge: 结论性落盘 MUST subagent 产出。
