# 提案：知识库压实机制（事件溯源 + 两级降压 + 事务性压实）

- 提案对象：RE-Framework v2（`spec/engineering-framework-v1.md` §4.5 执行强制链 + `dsh/skills/core-knowledge` + `dsh/skills/core-artifact`）
- 提案人：CoreSwap multiworld-port session（2026-09-01）
- 状态：draft（待维护 agent 评估）
- 实证来源：CoreSwap 错误台账 M11/M14/M16（`.investigations/multiworld-port/multiworld-errors.md`）+ DSH `packages/compaction/*` 设计借鉴

## 1. 问题陈述

CoreSwap 实战暴露的知识库结构性两难：

1. **覆盖式修订有丢失风险**：错误优先原则（AGENTS.md §三.2）要求错误链全文保留；任何"修订旧条目"的操作都可能烧掉判错过程（M11 的 seed 错位链条若被后续修正覆盖，第 3 次复发就无从诊断）。
2. **纯追加导致熵增**：台账线性增长（M1-M16 已 550 行），速查表/INDEX 等视图靠手维护开始失步（M7 行格式已串行）；原则层（discovered）混入未验证结论的风险随条目数上升。
3. **跨 session 交接的假设污染**：NEXT_SESSION/台账里的「机制方向」类未验证结论被下轮当公理续推（M14 方向错误绕圈一轮、M11 seed 错位三犯的共同根因）。

## 2. 借鉴源：DSH compaction 的三个可移植机制

| DSH 机制 | 本质 |
|---|---|
| 事件溯源（append-only events，compaction 产生持久化 summary 事件而非改写历史） | log 不可变，消费视图用派生摘要 |
| 两级降压（tool-result-pruner 机械修剪先行，不足才 LLM summarization） | 便宜的机械操作优先于昂贵的语义重写 |
| 事务性压实（region 选择 + span 变更即拒绝提交 + start/summary/end 事件对） | 压实是事务，只在安全边界执行 |

## 3. 具体条款变更

### 3.1 core-knowledge 新增：条目结构化 front-matter（机械层）

台账/错误账本条目（`errors/error-<NNN>-<slug>.md` 及项目级指定载体）头部 MUST 带结构化字段：

```yaml
---
id: M16
status: open | closed | superseded
supersedes: M14        # 可选：本条推翻/取代的条目
superseded_by:         # 可选：被取代时由压实/后续条目回填
signature: "不相干方块成片 + 数量跨环境精确复现"   # 一行现象签名（速查表生成源）
verdict: "写入路径 id 域错位"                      # 一行根因（速查表生成源）
lesson: "跨层 id 必须显式声明域"                    # 一行教训（速查表生成源）
---
```

正文五段式不变（错误优先原则不动）。**front-matter 是生成视图的数据源，不是给人读的**。

### 3.2 core-knowledge 新增：派生视图禁止手维护（机械层）

- 速查表（「错误→根因」表）、INDEX.md 的分类索引行、（可选）主题篇的结论清单 = **由脚本从 front-matter 生成**（`gen_cheatsheet.py`，~50 行，每框架/项目各一份模板）。
- 手改生成产物视为无效操作（下次生成即覆盖）。纪律依赖 → 机械生成，同源消除视图失步。

### 3.3 core-knowledge 新增：压实 pass（语义层）

- 触发：每累计 **N=10** 条新台账条目，或一个课题结案时。
- 动作：同根因/同判据的条目簇 → 合并为 1 条 discovered 原则（含回指 `source_entries: [M14, M16]`）；被合并条目标 `status: consolidated`（**原文保留**，不删）。
- 执行者：subagent 产出合并草稿 → 主会话应用（复用现有知识库更新强制触发点流程，不新增执行者角色）。
- 合并产生的原则条目标 `confidence: candidate`，走既有 confirmed 状态机。

### 3.4 core-plan / 宿主 AGENTS 模板新增：交接结论验证纪律 + 切换建议职责

写入 spec §4.5（执行强制链）或 Phase 0 前置：

1. **交接结论验证**：交接文档/台账里的「机制方向/待查假设」类结论不得当公理续推，开工前 MUST 廉价独立验证一次（≤1 轮成本）。反例实证：M14（feature 污染方向错误）、M11（seed 错位三犯）。
2. **切 Session 三态建议 = AI 职责**：在每个闭合点（结论闭合/judge 通过/修复验证完成/污染信号出现），AI MUST 主动给出「建议切（附已外化清单 + 下轮开工点）/ 建议继续（附未闭合理由）/ 无差异」三态建议；污染信号（同假设两轮无新证据、重复旧论证、旧结论引用含糊）出现时即建议切，切前执行「已验证事实与未验证假设分离落盘」。

### 3.5 与既有条款的关系

- **不冲突**错误优先原则：压实合并的是「知识表示」，原始证据链在台账层永久保留，原则条目带回指。
- **不冲突**记录价值门：价值门管「写不写」，本提案管「写了之后怎么组织」。
- 指定载体优先原则（INDEX.md 错误台账载体声明）不受影响：front-matter/生成脚本对项目级指定载体同样适用。

## 4. 迁移与验收

- 迁移：存量条目补 front-matter 可渐进（新条目 MUST，旧条目惰性补齐）；生成脚本首跑后速查表与手维护版 diff 应零语义损失。
- 验收判据：① 手改速查表 → 重生成后语义一致；② M14/M16 用 supersedes 链表达后，任一条目可回答"当前有效结论是什么、被谁取代"；③ 一次压实 pass 后原则层条目数不增（合并 ≥2 → 1）。
