---
编号: 000
任务: shared 臂 Java 逐位裁决（WG_EST_SHARED 是否真修 surface est 错位 bug）+ est L2 翻默认评估 + gpu-batch-merge 天花板重议
任务类型: 验证裁决 + 性能评估 + 决策
模式档位: 重量
状态: 已批准（用户选定 W1+W2+W3，W4 b4 carver 另立 session）
session: 260903-12（实际 2026-09-03 20:45）
---

## 1. 全局视图（目标/范围/排除项）

**目标**
1. **W1**：裁决 WG_EST_SHARED（b1-a）开启后的输出变化（hash `8bff4087…`）是「修正既有 surface est 错位 bug」还是「引入新偏差」——Java 逐位对比定论。若真修 bug：评估翻默认（对齐率免费提升）。
2. **W2**：est L2（b1-b）翻默认前置三件套：① mt_fill Mutex 争用基线 ② 大 region 淘汰行为 ③ e2e l2 stats 落盘（judge C1）；附带归因 e2e 收益（−48ms/chunk）超 est 微测上界（15.5ms）的剩余差（#21 working set 失配签名，观察未归因）。
3. **W3**：gpu-batch-merge 天花板重议——est 优化落地后 e2e 27.69ms/chunk，决策立项/降级/搁置。

**明确不做（本 session）**
- W4 b4 carver 状态列反直觉机制定位（机制未明，另立 session，届时 MUST recode-scout 勘探前置）。
- 任何生产默认值翻动——本 session 只产证据与决策建议，翻动本身属后续独立变更。
- 逐位对齐率大盘点、L1-L3 block_probe 分层用例（judge C3，另行安排）。

## 2. 角色分配

| 角色 | 承担项 | 说明 |
|---|---|---|
| 主会话 | 全部命令执行（探针/构建/bench）、收敛型数据解读、决策建议 | subagent 沙箱无 shell（八.12） |
| scout（subagent） | W1 若需 Java 侧 ChunkNoiseSampler/SurfaceBuilder est 调用链摸底（共享语义对拍点定位） | 仅在静态对拍需要时触发；非默认必开 |
| worker（subagent） | bench/对比原始数据解读确认（发散/多假设时）；知识库草稿产出 | 命令委托契约：主会话只执行不写结论 |
| fan-out | 见 §8 预置分叉点 | |
| judge（subagent） | 见 §7 预置 | |
| knowledge（subagent） | 结论性 docs/discovered 草稿 | 主会话只应用+验证 |

## 3. 任务拆解 & 依赖图

```
P0 交接验证（W1 前置，廉价）
 ├─ P0.1 复跑四臂 A/B（estopt_ab.rs）确认 off/l2/shared 三臂 hash 现象仍复现
 │        （off=74f5dfc4…，l2 一致，shared=8bff4087…；不复现则 W1 重开）
 └─ P0.2 环境四查：删 run\world（#19）/ seed 核对 / WG_* 默认关确认
        ↓
P1 W1 shared 臂裁决
 ├─ P1.1 锁定 Java 参照点：ChunkNoiseSampler.java:222-226 est 共享语义 + SURFACE 阶段
 │        est_at 调用形态（scout subagent 摸底，只读）
 ├─ P1.2 Java 参照采集：同 seed/坐标 SURFACE est 值导出（五要素核对：seed/size/
 │        origin/dim/stage，#10/#11）
 ├─ P1.3 Rust off 臂 vs shared 臂 vs Java 三方逐点对比（坐标钉死律 #17）
 └─ P1.4 裁决：shared 变化=修正 or 偏差 → .artifacts 结论（draft→candidate）
        ↓
P2 W2 翻默认三件套（与 P1 无数据依赖，环境允许可交错）
 ├─ P2.1 mt_fill Mutex 争用基线（多线程 off/l2 A/B，区分争用收益）
 ├─ P2.2 大 region 淘汰行为（FIFO 131072 条上限下的命中率/退化曲线）
 ├─ P2.3 e2e l2 stats 落盘（C1：256-chunk e2e 命中率+收益，可溯源落盘）
 └─ P2.4 剩余差归因：e2e −48ms vs 微测上界 15.5ms（working set 复刻，#21）
        ↓
P3 W3 决策 + judge
 ├─ P3.1 gpu-batch-merge 天花板重算（基于 27.69ms/chunk 新基线）→ 决策建议
 ├─ P3.2 judge 审查（MUST，收尾三源核对）
 └─ P3.3 知识库更新（subagent 草稿 → 主会话应用）
```

## 4. 并行执行计划

- 第一波：P0.1+P0.2（主会话串行，快）
- 第二波：P1.1 scout（后台 subagent）与 P2.1/P2.2 bench（主会话）并行
- 第三波：P1.2→P1.3→P1.4（主会话）；P2.3/P2.4
- 第四波：P3 收尾

## 5. 人工决策 HOOK 点

| 节点 | 触发 | 决策内容 |
|---|---|---|
| P1.4 后 | shared 裁决完成 | 是否推进翻默认建议（用户拍板 confirmed） |
| P2 后 | 三件套数据齐 | b1-b 是否翻默认（建议 → 用户拍板） |
| P3.1 后 | 天花板重算 | gpu-batch-merge 立项/降级/搁置（用户拍板） |
| 任何重大转向 | 裁决推翻交接假设 | 暂停回 Phase 0（MUST judge 前置） |

## 6. 风险 & 回退

- **参照污染**：run\world 未删 / seed 错位 → 采集前四查铁律（#19 + seed 三查）；SURFACE 参照 stage 要素核对（#10/#11）。
- **坐标错位**：探针坐标语义三套不同 → 对比前坐标钉死（#17），打印坐标≠采样坐标即结论无效。
- **P0.1 现象不复现**：stash/target 重建可能影响 → 不复现即停，重查环境再判，不硬推。
- **多臂顺序效应**：A/B 必须 chunk 粒度交错（#24）。
- **量化声明**：所有对齐/性能指标同行声明 §9.7 三要素；每量化声明可溯源到落盘原始输出（C1 复训）。

## 7. judge 步骤预置

| 节点 | 级别 | 审查对象 |
|---|---|---|
| P1.4 shared 裁决结论 candidate 授予前 | SHOULD | .artifacts 快照 + 原始对比输出 + Java 参照五要素记录 |
| P2 三件套结论 candidate 授予前 | SHOULD | bench 落盘 + stats 数据 + §9.7 声明 |
| P3 收尾交付 | MUST | 三源核对（.artifacts 快照 + git diff + 验证记录） |

## 8. fan-out 步骤预置

| 分叉点 | 候选 | 处置 |
|---|---|---|
| P1.4 若三方对比呈现混合结果（部分点修正/部分点偏差） | 互斥假设 a=D1 修正真实但局部 / b=共享引入新偏差域 | MUST fan-out .b1/.b2 并行 worker，禁止主会话自推 |
| P2.4 剩余差归因若多机制候选并存 | 如 L2 命中外的内存/分支/内联因素 ≥2 互斥候选 | MUST fan-out |

## 9. 知识库更新

- 结论性 docs（10 时间线 + 主题篇小节）与 discovered/ 发现：**subagent 产出草稿**（prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md 指引行）→ 主会话应用 + 验证。
- 预期候选沉淀：P1.4 裁决结论（若推翻/确认 D1）、P2.4 working set 归因补充（#21 家族）、错误台账条目（五段式）。

## 10. 子角色介入点（全部预置，执行不临时起意）

- scout: P1.1 | 触发: Java est 共享语义调用链摸底（机制未明面） | 产物: .investigations/est-shared-java-map/（只读）
- worker: P1.3 数据解读确认 / P2 bench 解读 / §8 fan-out 候选各一 | 产物: .artifacts draft
- fan-out: §8 两预置分叉点，触发即并行
- judge: §7 三节点（P3.2 MUST）
- knowledge: P3.3 | subagent 草稿 + 主会话应用
