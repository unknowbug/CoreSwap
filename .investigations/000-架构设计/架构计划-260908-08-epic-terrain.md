---
编号: 260908-08
任务: Epic Terrain——第三方 worldgen 数据包兼容（数据包入口 + Rust 接管集参数化）
任务类型: re-code + swe 混合（数据包注入路径勘探 + Rust 数据驱动参数化）
模式档位: 重量
状态: 待批准
日期锚: Get-Date 2026-09-08 13:12；上一块提交 ea4d435（260908-07 债务已清偿）
---

## 1. 全局视图

- **目标**：让 Rust worldgen 接管管线能消费第三方 worldgen 数据包（如 Terralith 类）注入的自定义 `worldgen/` 数据（biome / noise_settings / feature / structure 等），即：**数据包入口**（包内 JSON 如何被加载进 Rust 的 registry 数据集）+ **接管集参数化**（接管哪些维度/阶段/设置不再硬编码，跟随数据包内容）。
- **背景约束（AGENTS.md 数据驱动架构铁律）**：架构必须数据驱动——版本相关 worldgen 数据从 JSON 加载，代码固定；跨数据包只换数据，不改代码。`worldgen-core/data-driven-boundary.md` 已标注的硬编码升级点是本次工作底册。
- **范围排除项**：
  - 不做第三方 mod 的 Java 侧行为兼容（mod-compat 载体已另行结案）。
  - 不承诺任意数据包 100% 兼容；首期以「一个代表性第三方数据包端到端可生成」为验收。
  - 不改 Anchorlaw 协议 / RE-Framework。

## 2. 角色分配

| 角色 | 子任务 | 执行方式 |
|---|---|---|
| scout | Phase 1 勘探：数据包加载路径 / 接管集硬编码点清单 / data-driven-boundary 差距图 | subagent 只读，隔离 |
| worker | 数据包入口设计 + 参数化实现（收敛主会话做；发散候选交 worker） | 主会话 + subagent 按需 |
| judge | candidate 授予 SHOULD；收尾交付 MUST 三源核对 | subagent |
| knowledge | 结论性落盘 MUST subagent 草稿（含 SUBAGENT-KNOWLEDGE-GUIDE.md 指引） | subagent + 主会话应用 |

## 3. 任务拆解 & 依赖图

```
Phase 1 scout 勘探（只读，.investigations/epic-terrain-260908-08/）
  1a. 数据包入口现状：Minecraft datapack worldgen JSON 注入点（Java 侧 registry 加载时序）；
      Rust 侧当前数据集来源（blocks.json/light_data.json/generated 等加载路径）
  1b. 接管集硬编码清单：WorldgenHandle::create 单世界硬编码点、维度参数、
      build_overworld_rule 代码规则、stageMask 接线（已验证机制直接引用，不重验）
  1c. data-driven-boundary.md 差距图：已数据驱动 vs 硬编码（carver replaceable / feature tag 等）
  依赖：1a/1b/1c 可并行（fan-out 三个 scout 或单 scout 三段）

Phase 2 设计（收敛，主会话）
  2a. 选定代表性第三方数据包（需用户提供/确认目标包）
  2b. 数据包入口设计（JSON 从包目录 → Rust registry 数据集的通道与格式）
  2c. 接管集参数化设计（settingsName/维度/stage 参数化，对齐 C++ wg_create 形态）
  ★ 若入口层出现 ≥2 互斥方案（如 runtime 解析 vs build-time 转录）→ MUST fan-out .bN

Phase 3 实现（swe 收敛闭环，主会话）
  3a. 实现 + cargo build --offline 全量绿（#27：-p 单包绿 ≠ 全量绿）
  3b. bin-diag 归位（新诊断 bin 只进 bin-diag/）

Phase 4 验证（Phase 2.5）
  4a. e2e：目标数据包 + Rust 接管生成 region，与数据包+vanilla 臂对比
      （载体：Chunky 双臂法 #26；seed 三查 + 坐标语义核对前置）
  4b. 行为化证据：接管生效日志行为化 + 数据包内容哈希哨兵（#37/#53/#81 家族判据）
  证据饱和 / C-gate 按 Anchorlaw v0.20 §9.4/§9.7（对比口径三要素声明）

Phase 5 judge + 收尾
  5a. candidate SHOULD judge → 用户 confirmed 门
  5b. 知识库更新（subagent 草稿）+ docs/时间线追加 + 提交
```

## 4. 并行执行计划

- 第一波：Phase 1 scout（1a/1b/1c 并行段）。
- 第二波：Phase 2 设计收敛；分叉则 fan-out 并行。
- 第三波：Phase 3 实现 → Phase 4 验证（含 Java vanilla 臂准备可并行）。

## 5. 人工决策 HOOK 点

| 节点 | 触发 | 决策 |
|---|---|---|
| H1 | 架构批准 | 本文档用户批准（当前步骤） |
| H2 | Phase 2a | 代表性第三方数据包选型（用户指定） |
| H3 | 入口方案 ≥2 互斥 | fan-out 竞争结果用户拍板 |
| H4 | candidate 授予 | judge 意见 → 用户拍板 |
| H5 | 重大转向（范围/根因定论） | MUST judge + 用户确认 |

## 6. 风险 & 回退

- **R1 数据包内容超出现有数据驱动边界**（自定义 feature type / 结构等）→ 按边界文档声明缺口，逐项评估，不静默跳过（#37 降级声明纪律）。
- **R2 数据包 JSON 与 1.20.1 schema 漂移**（包可能面向多版本）→ 先核包目标版本，版本不符即停报用户。
- **R3 runtime 解析新代码引入性能回退** → 端到端对比铁律：优化类改动必须大样本 e2e 对比，热路径禁诊断代码。
- **R4 探针/对比口径污染** → seed 三查、坐标钉死、执行体三元组核对（#36 家族）前置。

## 7. judge 步骤预置

- 节点: Phase 2 设计定稿 | SHOULD | 审查对象: 设计文档 + boundary 差距图
- 节点: candidate 授予（Phase 4 后） | SHOULD | 审查对象: verdict + 验证记录
- 节点: 收尾交付 | MUST | 三源核对（.artifacts 快照 + git diff + 验证记录）

## 8. fan-out 步骤预置

- 节点: Phase 2b 入口方案 | 候选: runtime JSON 加载 vs build-time 转录扩展 | MUST .bN 并行（若两案均可行）
- 节点: Phase 4 残差归因 | 候选: 数据解析差 / 代码语义差 / 数据包固有非确定（#67 features 层）| MUST .bN（若出现多候选）

## 9. 知识库更新

- 结论性 docs/discovered：core.worker subagent 产出草稿（prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md 指引）+ 主会话应用验证；错误台账落 `.investigations/epic-terrain-260908-08/errors.md`（五段式）。

## 10. 子角色介入点（全部预置）

- scout: Phase 1 | 触发: 数据包入口/接管集机制未明 | 产物: .investigations/epic-terrain-260908-08/ 管线地图（subagent 隔离）
- worker: Phase 2/3 发散设计候选、Phase 4 数据解读、Phase 5 知识库草稿 | 产物: .artifacts draft / docs 草稿
- fan-out: §8 两节点 | 禁止主会话自推多候选
- judge: §7 三节点 | 只出意见不改 status
- knowledge: §9 | subagent 产出 + 主会话应用
