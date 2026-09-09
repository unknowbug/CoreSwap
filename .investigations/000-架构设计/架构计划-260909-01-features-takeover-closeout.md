---
编号: 000
任务: 1.21.6 Rust features 接管收尾块（Phase 2.5 冻结序对拍 → judge → mask 翻转建议 → 知识库落盘 → 台账登记）
任务类型: swe（验证 + 审查收口）
模式档位: 轻量（延续已批准重量计划 架构计划-260908-16-features-takeover-1216.md 的 Phase 2.5/3）
状态: 已批准（用户 260909-01 批准，接 NEXT_SESSION 260909-01 开工点）
---

## 范围（含明确不做什么）
- 做：批次 D 后验证对拍（A 臂 mask 0b011 vs B 臂 -PrustStages=1，同 seed region 0,0 r=16，Chunky + region diff）→ 残差归因 → judge MUST → mask 翻转建议（confirmed 留用户）→ 知识库落盘（subagent 草稿）→ 缓装 5 项台账登记。
- 不做：缓装 5 项实装、carver、1.20.1 改动。

## 任务拆解
1. 环境复位（删 run\world、核 Chunky/RCON/参考脚本在位、Get-Date 取日期标签）
2. Phase 2.5 双臂对拍（#67 冻结序判据；残差 9 项 §9.7 单列；idk-5 最高优先观察）
3. 残差处置（成簇 ≥2 互斥候选 → fan-out .bN；否则归因记录）
4. judge MUST（三源核对；重点 ice_spike :46 / vegetation_patch 求值序 / root_system 双出口 / P1c 零回归）
5. mask 翻转建议（0b011→0b001，judge 后汇报，用户拍板）
6. 知识库更新（subagent 草稿，prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md；5 候选条目 + 10 时间线）
7. 台账登记（缓装 5 项 + c1-exemption-list.md B5/B6 注记）

## 验证方式
主载体 = Chunky 双臂 region diff（#26 载体）；判臂前核 flags/mask（#66）；features 固有非确定 → 冻结序生成判据（#67）。

## judge 预置
- candidate 授予前 MUST（三源：.artifacts 快照 / git diff / 对拍记录）
- 收尾交付 MUST

## fan-out 预置
- 对拍成簇偏差 ≥2 互斥候选（RNG 序 / 实现缺陷 / 数据解析）→ MUST .bN 并行，禁止主会话自推

## 知识库更新
- 结论性 docs/discovered：subagent 产出草稿（core.worker）+ 主会话应用 + INDEX 同步

## 子角色介入点
- scout: 无（勘探已完成，无机制未明面）
- worker: 知识库草稿（subagent）
- fan-out: 预置候选域见上
- judge: candidate 前 MUST + 收尾 MUST
- knowledge: 第 6 步，subagent 产出 + 主会话应用验证
