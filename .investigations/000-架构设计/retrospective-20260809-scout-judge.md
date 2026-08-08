# 复盘：worker 熟练但 scout/judge 缺位（2026-08-09 -288 课题教训）

> 触发：用户在 -288 课题收尾后指出「worker 你用得很熟练了，但是侦测和 judge 缺位很严重，尤其是 judge」——复盘确认属实，本文件为反模式记录 + 改进机制依据。

## 一、量化（-288 课题 session 实际 subagent 使用）

| 角色 | 次数 | 用途 | 评价 |
|---|---|---|---|
| worker（recode.scout 解读/分析） | 12 次解读 + 2 次 patch 设计 | analysis-phase2~13 解读、noise_blk/beardifier patch | ✅ 熟练 |
| scout（勘探） | **0 次** | 初始勘探/管线依赖摸底/大范围扫描 | ❌ 全缺 |
| judge（审查） | **1 次（收尾补的）** | 用户提醒「你的审计呢」之后才跑 | ❌ 严重缺位 |

## 二、缺位表现

### judge 缺位（最严重）
- 工作流要求「Phase 2 分析 → **Phase 3 审查（core.judge）** → 用户拍板」——**Phase 3 全程跳过**，所有关键结论主会话自评：
  - 「8/8 结案不成立」（phase2）
  - 「Beardifier 排除」（phase6-8）
  - 「noodle 高频丢失」（phase11）——**方向是错的**（phase13 证反：noodle 低频 firstOctave=-8）
  - 「C++ 核心无 bug」（AQF-APPLY 铁证）
- **补位 judge 立刻抓到 5 项问题**：① 差异构成表不闭合 23%（海底边界/gravel/表面规则未归类）② phase13 归纳失真（「强候选·未闭合」被写成「✅ 一致」）③ AQF-APPLY 高位垃圾值未说明 ④ index.yaml 覆盖缺口 ⑤ retry cap 超限未记录——**缺位的真实代价**

### scout 缺位
- -288 是「质疑结案重开」，入口看似明确（mismatch 明细），**早期若做一次管线阶段摸底（NOISE→CARVERS→SURFACE→FEATURE）**，carvers（洞穴雕刻）很可能提前多轮被发现，省掉 phase10-13 的 noodle/caves 树绕圈（4 轮超 retry cap）
- 主会话直接跳入单点定位（密度层→aquifer→Beardifier→caves 树），缺少「子系统/阶段全景」视角

## 三、原因分析

1. **v0.8 收敛门误解**：「编程=主会话直接闭环」被理解为「自评即可」——漏了「**+judge 审查门**」的强制部分。收敛门 = 主会话闭环 **+** judge 审查门，不是二选一
2. **judge 触发点未制度化**：什么节点必须 judge 不明确（confirmed 前？重大转向？各阶段结论？）→ 全凭自觉 → 全漏
3. **scout 触发条件未区分**：默认「收敛型分析主会话直接做」，没有区分「**入口已明**」（可收敛直做）与「**机制未明**」（应先 scout 勘探管线/子系统全景）

## 四、改进机制（已写入 AGENTS.md 执行强制链 + knowledge/discovered/）

### judge 强制触发点（MUST）
1. **confirmed 授予前 MUST judge**（本次教训：只有用户提醒才补）
2. **重大转向 MUST judge**：结案重开、根因定论（如「C++ 无 bug」）、范围决策（如 FEATURE 范围）
3. **各阶段结论（candidate 授予）也应 judge**
4. **随 todo 计划预置 judge 步骤**（计划阶段就排 judge 项，不是事后补）

### scout 勘探触发条件（MUST）
1. **「机制未明」类大排查初期 MUST scout 勘探**（管线阶段/子系统依赖摸底），禁止主会话直接跳入单点定位
2. 勘探产物（.investigations/ 管线地图）作为定位前置

### 验证
- 下次大排查（8576 21 块课题 / carvers 实现）按新机制跑：计划预置 scout + judge 步骤，验证不重演

## 五、附：-288 破案回顾（本次复盘对象）

- 14 轮调查（worker 解读 12 次）最终破案：C++ 核心无 bug（AQF-APPLY 铁证 + chunk status=carvers 铁证），差异 = 范围外 FEATURE（岩石替换 49% + carvers 17% + 结构 3.6%）
- 若无 judge 补位，差异构成不闭合（23% 未归类）会随结论一起归档——judge 审查门的价值实证
