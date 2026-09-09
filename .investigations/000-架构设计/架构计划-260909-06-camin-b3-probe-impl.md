# 架构计划-260909-06：ca_min 残差 +30% —— B3 变体 a 前置探针轮 + 实现

---
编号: 000
任务: ca_min +30% 优化——前置探针轮（P1/P2 判别）→ B3 变体 a（3×3 邻 chunk 列 Arc 预取快照）实现 → 验证 + judge
任务类型: 性能优化（swe 域，收敛型主会话闭环）
模式档位: 轻量
状态: 已批准（用户 260909-06，实际 2026-09-09 21:51，git 锚 fc9c4fe）
上游: 架构计划-260909-05-camin-memo.md（评审裁决 confirmed：有条件立项 B3 变体 a，探针 MUST 前置）
---

## 范围（含明确不做什么）

- 做：① WG_CA_MEMODIAG 扩展探针（一轮采齐）；② 按判据分流后实现 B3 变体 a（3×3 邻 chunk 列 Arc 预取快照 + neighbor_terrain 兜底 + B1 risk-3 不变量注释）；③ on/off hash 逐位不变验证 + 无探针 wall A/B（目标 on≈off±5%）；④ #101 §15.4 取代记录落盘。
- 不做：B1 点级 memo 主体实现（仅探针分流命中 P1 主导时转 B1 补充评估，回用户拍板）；不改 vanilla 行为；不发布。

## 任务拆解

1. **Step 0 交接验证（廉价独立验证）**：核对 fc9c4fe 诊断代码在位 + WG_CA_MEMODIAG 行为化输出正常；交接判据（98% 重复读/唯一列 ≈288）不作公理，探针轮数据本身即验证。
2. **Step 1 探针轮（一轮采齐）**：
   - ① 越界读目标 chunk 偏移 (tcx-cx,tcz-cz) 直方图（验 3×3 覆盖率）
   - ② neighbor_terrain miss 计数 + fill_terrain_column 计时（验 P2）
   - ③ WG_CA_CAP=2048 判别臂
   - 产物：`.investigations/camin-perf/probe-260909-06.md`
3. **Step 2 判据分流**：
   - miss≈288 且 CAP 臂回落 → P2 实锤（E2b 需 §15.4 重审）
   - miss 高但 CAP 臂无差 → 重生成另有结构 → **潜在 fan-out 分叉点（≥2 互斥候选 MUST fan-out .bN）**
   - miss<<288 → P1 主导转 B1 补充评估（回用户拍板 = 重大方向变更 HOOK）
4. **Step 3 实现 B3 变体 a**：3×3 邻 chunk 列 Arc 预取快照 + neighbor_terrain 兜底 + risk-3 不变量注释（「缓存列永不更新+写不可见于越界读」）。
5. **Step 4 judge + knowledge**：judge MUST（收尾三源核对）；#101 §15.4 取代记录 subagent 草稿 + 主会话应用。

## 验证方式

- golden 逐位不变对照法（发现 #47）：on/off hash 双臂，hash 报告带 hash↔文件名对应，任何不等即 FAIL。
- wall A/B 串行 bench、无探针口径；§9.7 口径声明随行（载体/覆盖面/可比性）。
- workspace 全量绿：`cargo build --offline`（全量，非 -p 单包，#27 判据）。

## judge 预置

- 节点: Step 4 收尾交付 | MUST | 审查对象: .artifacts 快照 + git HEAD/worktree diff + 验证记录（hash A/B + wall A/B）
- 节点: Step 2 分流结论 candidate | SHOULD

## fan-out 预置

- 节点: Step 2 判据分流 | 触发: 「重生成另有结构」且 ≥2 互斥候选 | .bN 并行 | 禁止主会话自推

## 知识库更新

- 结论性 docs/discovered（#101 §15.4 取代记录 + 本块高价值发现）: subagent 产出草稿（先读 knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用验证。

## 子角色介入点（全部预置）

- scout: 否（机制方向已有 scout 地图，本轮探针为验证型）
- worker: 知识库草稿 subagent ×1
- fan-out: Step 2 分流触发（条件见上）
- judge: MUST ×1（收尾）+ SHOULD ×1（分流 candidate）
- knowledge: 结论性落盘 MUST subagent 产出
