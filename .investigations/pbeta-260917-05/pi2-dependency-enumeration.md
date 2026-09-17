---
id: pbeta-260917-05:pi2-dependency-enumeration
block: 260917-05
status: draft
purpose: v0.22 §15.4 PI-2 采用前的强制前置——判据依赖图 ≤1 轮枚举（协议明文 MUST，未做则不得据 PI-2 强制 fan-out 汇聚）
protocol_baseline: Anchorlaw v0.22（8313097）§15.4 PI-2
method: 枚举活跃判据 artifact → 提取其「观测对象/前置事实」→ 判定边方向（判据 → 前置事实；结论 → 判据）→ 检测环
scope_note: 覆盖面 = 本工作区 .investigations/ 下全部 criteria 类 artifact（4 份）+ 其 verdict 对判据的引用面
---

# PI-2 前提枚举：判据依赖图（260917-05）

> 协议要求原文（v0.22 §15.4）：*Hosts adopting PI-2 MUST first perform a cheap enumeration of the active criteria dependency graph (≤1 round) and register the result; if cycles exist, the clause MUST be amended to "cycles permitted but registered" rather than enforced as a ban.*

## 1. 枚举范围（一手）

活跃判据 artifact（4 份，`.investigations/` 全量扫描 `criteria*.md`）：

| # | 判据 artifact | 课题 | 判据主体 |
|---|---|---|---|
| K1 | `c1-c6-260917-04/criteria-260917-04.md` | C-1(R1) 载体换轨 + C-6 性能 A/B | changed(run2→run3) 带判定 + C6 折算 |
| K2 | `cp1-light-form-260916-01/criteria-260916-01.md` | CP-1 域批形态 | G3 drift 阈值（C-1 <2.5%）+ 形态门 |
| K3 | `g3-drift-basis-260917-01/criteria-260917-01.md` | G3 漂移基底归因 | C-B1..C-B4（settle 持久性 / run3 分裂 …） |
| K4 | `pbeta-260917-05/criteria-260917-05.md` | P-β/P-path 分辨探针 | C-β（inputDiff23 × changed23）+ SELFCERT |

## 2. 依赖边（判据 → 其前置事实 / 判据 ← 其依赖的结论）

关键区分（本枚举的核心判据）：**边的对象是「外部事实」还是「另一条判据的结论」**——前者不构成环风险（事实不是判据），后者才是。

| 边 | 类型 | 对象 | 环风险 |
|---|---|---|---|
| K1 → 载具身份（snap_light.py + Done+60s + dll 6F7FA3AE + seed 8576294172403134396） | 外部事实 | 执行体三元组 | 无 |
| K1 → `[SELFCERT]` 自证项（lightInit ok / hook armed / fallback=0 / dll sha / **world 身份**） | 外部事实（行为化日志） | 采集产物 | 无 |
| K2 → G3 阈值（<2.5%） | **判据间引用** | K3 系结论（G3 drift 数值） | ⚠️ 需判向 |
| K3 → 载具（snap_light.py 同源）+ run1/2/3 世界身份链 | 外部事实 | 采集序 | 无 |
| K4 → 载具（同 K3/K1 同源）+ SELFCERT（含 world 身份项 #161） | 外部事实 | 采集序 | 无 |
| K4 → 260917-04 §15.4 取代后的新基线（legacy 干净链 run2→run3=0） | **结论引用** | K1 系 verdict 结论（已 confirmed） | ⚠️ 需判向 |

## 3. 环检测结论

**两条「判据/结论互引」边的方向判定**（逐条一手核对）：

1. **K2 → K3 系（G3 阈值）**：方向 = K2 引用 K3 产出的**已定论数值**（G3 drift），K3 不回引 K2。K3 的判据（C-B1..C-B4）前置全部是外部事实（采集产物），不含「K2 是否满足」。⇒ **单向后向引用，无环**。且该引用已被 260917-01/02 以 **§15.4 取代**收口（K2 的 <2.5% 阈值随 G3 载体一并回炉，#157）——即这条边在当前状态是**历史边**（被取代后不再承载强制）。
2. **K4 → K1 系（260917-04 新基线）**：方向 = K4 引用 K1 的**已 confirmed 结论**作为背景基线；K1 的判据前置（载具身份/SELFCERT）不含 K4。⇒ **单向，无环**。

**总体判定：当前活跃判据依赖图无环（acyclic）**——4 份判据的全部前置可归约为「外部事实 + 已有定论的单向引用」，不存在 A 的满足性依赖 B、B 的满足性又依赖 A 的形态。

**前提登记（照协议要求显式写出）**：
- 本判定覆盖面 = 上述 4 份 artifact + 其 verdict 引用面（全量扫描所得，非抽样）；
- 未覆盖 = 尚未产出 criteria artifact 的历史课题（其判据未登记，故不在图内）；以及未来新增判据（**未来代码不承担证明义务**，#143 残留边界的同族声明——新增判据可能首次引入环，届时须重跑本枚举）；
- 判定方法 = 边对象分类（外部事实 vs 判据结论）+ 两条可疑边逐条判向，未做形式化图算法（本枚举即协议要求的「cheap enumeration」）。

## 4. PI-2 采用状态（登记）

- 因**无环**，PI-2 可按协议原样采用（**无需**改为 "cycles permitted but registered"）。
- 但 PI-2 在协议中仍标 **unverified**（缺 conformance test）；本工作区的落地形态 = `AGENTS.md` §一.11 已写「PI-2 采用前 MUST 先跑 ≤1 轮枚举」+ 本文件为**首次枚举登记**。
- 落地动作（待办）：把「PI-2 汇聚」写入 fan-out 收尾纪律——**任一 `.bN` 候选在汇聚到唯一裁决终点（verdict/judge 结论）前不得被单独引用为结论**；配 fail 判据 = 存在 `{bN | 无判决 且 无 rejected_reason}` 即视为未汇聚（此判据来自论文研究建议，属**待验证方向**，非协议强制）。

## 5. 诚实声明

- 全部结论基于本工作区一手文件（4 份 criteria + 相关 verdict），**未执行形式化图分析工具**；「无环」判定依赖第 2 节的边分类，若存在未被本枚举发现的隐式依赖（如某判据的 check 动作引用了他课题的采集产物但未在 artifact 中写明），判定需修正——**这是本枚举的主要盲区**，与论文「acyclicity 是 assumption 而非定理」同性质。
- 未改动任何 status；本文件 = 协议要求的**登记产物**（register the result）。
