---
编号: 000
任务: shared 臂 default 开销归因与修复——default(l2+shared) 37.67ms vs l2-only 27.50ms，中位 +10.2ms/chunk（+37%），min 不变尾部变重
任务类型: 性能归因 + 修复（swe 域，Rust CPU 管线）
模式档位: 轻量
状态: 已结案（前提撤销）——归因开工前代码阅读未发现可行机制，干净复跑（default 28.46 ≈ l2-only 27.50）+ qpd1 FULL(default) 25.87 证明 37.67 为并发 GPU bench 测量污染；shared 臂无罪，无代码修复。根因定论 judge PASS（review-260903-16-shared-arm-rootcause.md），confirmed 待用户拍板。
session: 260903-16
---

## 范围（含明确不做什么）
- 做：WG_EST_SHARED 翻默认后引入的 ~10.2ms/chunk 中位开销归因（机制定位）→ 修复 → 重测验证。
- 不做：GPU 算子课题（降为待重估——本课题修完后 CPU 基线预期 ~27.5ms，届时按 260903-12 重议条件重估）；density 底座 14.06ms 优化；aquifer 段（已 5.8ms）。

## 任务拆解
1. **归因**：shared 臂机制回顾（260903-12 estopt_ab / defaultflip-260903-13 产物）→ 定位开销来源（shared est 扫描路径/缓存失效/分支预测/内存布局），数据层证据（阶段差分复用 qpd1 / est_price_probe 形态探针）。
2. **修复设计**：单假设收敛则主会话直接修；若 ≥2 互斥机制候选 MUST fan-out（.bN）。
3. **修复实现 + 验证**：主会话改码 → build → 三臂 bench（l2-only / default / default+fix）+ 四臂 hash 零退化哨兵。

## 验证方式
- e2e 256 chunks median：default+fix 显著 < 37.67（目标 ≈27.5 或更好）；hash 与 fix 前逐位一致（零退化铁律）。
- §9.7 三要素同行声明；bench 交错防顺序效应（workflow-patterns #24）。

## judge 预置
- 收尾交付 MUST judge（三源核对）；根因定论属重大定论 MUST judge。

## fan-out 预置
- 分叉点：shared 开销机制 ≥2 互斥候选时 MUST fan-out；单假设收敛则不触发。

## 知识库更新
- 结论/错误链条：subagent 产出草稿（SUBAGENT-KNOWLEDGE-GUIDE.md 前置）→ 主会话应用。

## 子角色介入点
- scout: 否（shared 机制有 260903-12/13 产物底子，worker 回顾即可）
- worker: 归因数据解读 + 知识库草稿
- fan-out: 机制分叉 ≥2 时
- judge: 根因定论 MUST + 收尾 MUST
- knowledge: 结论性落盘 subagent 产出
