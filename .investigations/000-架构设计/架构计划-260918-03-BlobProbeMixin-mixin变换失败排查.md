---
编号: 000
任务: 1.21.6 树 BlobProbeMixin mixin 变换失败排查（[BLOB-PROBE] stats read failed: Mixin transformation failed）
任务类型: 定位/排查（机制未明 → 含 scout 勘探）
模式档位: 轻量
状态: 待批准
---

## 交接前提验证状态（core-plan 前置纪律）

| 前提 | 状态 | 处置 |
|---|---|---|
| 「BlobProbeMixin mixin 变换失败」现象（NEXT_SESSION 定性，qualitative） | ⚠️ 未验证 | T0 重采样原始日志（属主日志核读，≤1 轮），失效则课题重定义 |
| 本块改动无因果（配置层 vs 类变换层） | 推断，非公理 | 仅作线索，不续推 |

## 范围（含明确不做什么）

- 做：1.21.6 树 BlobProbeMixin 装载失败定位（boot 日志 APPLY FAILED 段 → 目标方法/签名漂移 → 与 1.20.1 同名 mixin diff → 修复或定性「本就不可用」）。
- 不做：1.20.1 树任何改动；B6 线其余遗留；门禁强化；switch 映射（B6-2 已闭合）。

## 任务拆解

- T0 前提验证：重采样哨兵实跑原始日志，确认失败签名原文（[BLOB-PROBE]/APPLY FAILED 段），归档 cmd-output。
- T1 scout 勘探（subagent，机制未明 MUST）：定位 1.21.6 树 BlobProbeMixin 源码 + mixin json 注册项 + 目标方法签名；与 1.20.1 同名 mixin diff（json 条目数 vs 类清单 #40 家族签名）；产出 .investigations/b61x-blobprobe-260918-03/ 勘探地图。
- T2 收敛分析（主会话，单假设线性推进）：按勘探地图定位根因候选（json 失同步 / 目标方法签名漂移 / AP 警告前兆 #25）；若 ≥2 互斥候选并存 → fan-out（预置见下）。
- T3 修复 + 验证（主会话 swe 闭环）：修复 → 重跑哨兵直证 [BLOB-PROBE] 正常输出或明确「生产路径本就不可用」定性 + 修复建议；验证证据落盘。
- T4 judge（subagent，MUST）：三源核对（artifacts 快照 / git diff / 验证记录）。
- T5 知识库更新（subagent 产出草稿 → 主会话应用）：#40 家族补充案例或新发现 + 时间线块。

## 验证方式

- 直证法：修复前后 boot 日志中 mixin APPLY 状态 + [BLOB-PROBE] 行为化输出（正/负成对自证，#81）。
- 若定性「本就不可用」：给生产/探针两口径的存在性结论 + coverage 声明，不外推。

## judge 预置

- 收尾交付 MUST judge（三源核对）；candidate 授予 SHOULD judge。

## fan-out 预置

- 潜在分叉点：T2 若「json 失同步 / 签名漂移 / 目标类重命名」≥2 互斥候选并存 → core.fanout .bN 并行，禁止主会话自推。

## 知识库更新

- 结论性 docs/discovered 写入：subagent 产出草稿（先读 knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用验证。

## 子角色介入点

- scout: T1 | 触发: 机制未明勘探（mixin 装载链/目标签名摸底）| 产物: .investigations 勘探地图
- worker: T2/T3 收敛分析主会话直接做；解读类需求再派
- fan-out: T2 分叉点（见上）
- judge: T4 MUST（收尾三源核对）
- knowledge: T5 MUST subagent 产出

## 人工 HOOK 点

- H1 本计划批准（当前）；H2 修复方案确认（若涉行为面变更）；H3 confirmed 拍板。
