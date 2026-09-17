---
id: paper-2608-25512:synthesis-draft
block: 260917-05
status: draft
worker: 主会话（综合；透镜产物 b1/b2/b3 = subagent）
paper: 《A Programming Paradigm for Spatiotemporal Composability》(Yifan Shi, Wei Zhang, Tianyi Cui; PKU + DeepSeek-AI)
paper_text: .tmp/paper-2608.25512.txt（92 页，PAGE 标记）
lens_files:
  - .investigations/paper-2608-25512/b1-coreswap.md
  - .investigations/paper-2608-25512/b2-ref-framework.md
  - .investigations/paper-2608-25512/b3-anchorlaw.md
---

# 论文 × CoreSwap / RE-Framework / Anchorlaw —— 主会话综合（draft）

> 本文件只做综合与裁决建议；三份透镜产物各自独立，细节与证据链以它们为准。
> 状态纪律：全部 draft；建议是否立项由用户拍板。

## A. 主会话独立发现（三个透镜任务书未预设，源自论文原文）

### A1. 论文把「逆是否真的回退了效果」显式划为**组件作者的义务**，runtime 不验证（§5.1.1，p.59）

原文：「What the operation does not check is the witness that 𝔈*Γ carries: the callback supplies an inverse, and that the inverse reverts the effect it accompanies is an **obligation on the component author rather than a property the runtime verifies**」；§6.1（p.70-71）进一步说明边界外（emission）只能 **withhold**（output commit）或 **compensation**（较粗等价，且「metatheory does not transfer」）。

- 对本项目的意义：我们的「回退/取代/降级」全部是**文本级 compensation**（§15.4 supersedes、append-only、知识库不改写），而论文明确说这类较粗等价**不被元理论覆盖**——即「我们回退了」这句话不因论文而获得结构保证，只获得**声明保证**。
- 缺口 = **验证域**：论文的 composability 假设逆正确，Anchorlaw 恰好是「验证声明」的协议。两者互补，且论文没写验证协议（B3 已声明「论文没有一句话谈验证协议」）。
- 建议（新条目，可并入 B3 建议 1）：把「逆的正确性」做成**可验证声明**——`@anchor.test` 的 source 指向「逆执行后的等价对照记录」（如回退后 hash 与基线逐位相同），而不是仅指向「动作做过」。

### A2. Confinement（Def.55，p.38）→ 「越界读写」的结构判据

论文要求 effect function 的每个 stage 只读写**它自己声明过的 key**（「What it may neither read nor write is a table outside the two declarations」）；越界 = 该组件违反形式前提。

- 对照本项目：接管组件的「声明」散落为 stageMask / ca_min / domainbatch / bulkwb / WG_* env 门控——**没有一个地方声明「这个接管允许读写哪些数据」**。知识库已多次付出代价：#98（修复依赖的邻 chunk 读在生产门控下恒 None → 修复整体 no-op）、#110（生产不可达路径）、#102 补充案例（消费点集合在出货门控下为空）。
- 判据可操作化：为每个接管面登记 `reads:` / `writes:` 两张表（数据域 + 路径），采集时机械核对「实际触达 ⊆ 声明」。这是一个**便宜且直接消解 #98 家族**的机制。

### A3. Theorem 80 的「终态规格性」（confluence，p.65）→ 判据/交接的终态定义

原文：「the quiescent state [is] a function of the final configuration alone: **whatever instantiations and retirements the loader performs on the way, and in whatever order**, the system quiesces where a load of the final configuration from scratch would have left it」。

- 这直接给出**「口径失传」类错误的病根**（#156）：交接把「抵达路径」（我这轮用了哪些开关、怎么跑的）当承载体，而正确承载体是「终态规格」（跑到什么状态算完成：开关集合 + world 身份 + 自证行 + 期望产物）。B2 建议 3 的 `rerun-entry.yaml` 正是这条的落地形态——两者独立命中同一结论（b2 从 §5.2.1 reconciliation 出发，本处从 Thm.80 出发），可作**交叉印证**。

## B. 三透镜交汇点（≥2 个透镜独立命中的高价值项）

| 交汇点 | 命中透镜 | 说明 |
|---|---|---|
| 副作用/回退契约缺口（文本级完备、副作用级空缺） | b2（框架装载体）+ b3（验证动作）+ A1（论文自认逆未验证） | 三方独立指向同一缺口 → **最高置信度** |
| 「装载规格/终态规格」取代「抵达路径」 | b2（`rerun-entry.yaml`）+ A3（Thm.80）+ b3（#156 归因） | 同一病根三种表述 |
| 前置集/依赖声明缺失 | b3（判据 preconditions）+ A2（confinement 声明）+ b2（触发点证据谓词） | 共同形态 = 声明缺失导致静默失效 |
| 触发点不应全机械化，但「触发后有无产物」可判 | b2（明确结论）+ b3（流程不变量 PI-1/PI-2） | 两者边界一致（语义判断 vs 机械判定） |

## C. CoreSwap 侧（b1 透镜已到位）

b1 交付（`b1-coreswap.md`，336 行，Degraded 静态分层）：6 条建议 = 立即可做 3（B1 副作用成对登记 / B2 coeffect 依赖声明 + 降级分类 / B3 开关生效机械对账）+ 需验证 2（B4 观测等价升级含置换/交错检验 / B5 组件粒度与 key 空间）+ 不建议 1（B6 照搬元理论——MC 生成管线天然耦合，pairwise independence 结构性不成立）。

**主会话 spot-check（对本块结论负责的部分）**：b1 引用的关键行号已复核属实——`ServerLightingProviderMixin.java:77`（单一 sysprop 门）、`:545-550` 与 `:718-724`（写回 + `setLightOn(true)` + `wgReleaseLightTicket` **无逆路径**）。b1 已自标：§4 元理论全文未读（B6 依据仅目录标题+前提句）；部分知识库条目仅读 INDEX 摘要行而非条目原文；B2 依赖链只覆盖 light←features 一侧（surface/CARVERS/FEATURES 未一手核对）；全部结论 Degraded、数字均为转述未复算。

**b1 与主会话 A1/A2/A3 的一致性**：B1 与 A1（论文自认「逆未验证」）、B2 与 A2（confinement 越界读写）、B3 与 A3（Thm.80 终态规格性）**三条各自独立同向** → 三方向交叉印证，不是单点推测。

## C2. 最终分档建议（合并 b2/b3 + 主会话 A1/A2/A3）

**立即可做（低风险、有现成挂钩点）**
1. 安装产物身份对账（b2 建议 1）：`install.ps1` 生成 `install-manifest.yaml`（sha256 + source_commit），`selfcheck.ps1` 第 3 段从「目录计数」升级为内容对账（MISSING/DRIFT/ORPHAN）——**已核实**：现第 3 段确实只判 `$count -lt 17`（RE-Framework `dsh/scripts/selfcheck.ps1:49-55`），而同脚本第 5 段已是 fail-closed 门（同构哲学未用到第 3 段）。
2. 副作用/回退契约清单（b2 建议 2 + b3 建议 1 + A1）：框架侧新增「界内/界外 + 逆或补偿」三列表；最小子项 = profile patch 改写先备份（3 行）。
3. 复测交接强 schema（b2 建议 3 + A3）：`rerun-entry.yaml`（switches 枚举必填 + world_identity + prev + expected_selfcert）+ 字段级 diff 门 → 直接消解 #156/#137/#161。
4. 判据前置集 preconditions（b3 建议 2）：把 SELFCERT 从脚本内联提升为判据 artifact 字段，前置失效 → 判据 suspended（**不得**自动改 status）。
5. 验证副作用的逆登记（b3 建议 1）：「验证的 temporality」——in-place 类副作用 MUST 登记逆或显式声明不可逆。

**需验证（先做 ≤1 轮廉价验证）**
6. 分层等价门 E-Full/E-Partial/E-Degraded（b3 建议 3）+ CoreSwap 并发交错档位（C-c）。
7. 流程不变量 PI-1/PI-2（b2 元理论对应 + b3 建议 4）——前置 = 判据依赖图无环性验证（论文自认 assumption）。
8. 接管面读写声明表（A2 + C-a）——先在一个接管面（如 light）试做，验证「实际触达 ⊆ 声明」的机械核对可行。

**不建议做（明确排除）**
9. 把论文演算/元理论形式对象搬进协议或框架（b2 建议 6 + b3 排除 1）。
10. 验证副作用**自动**回退（b3 排除 2——会删掉失败轮证据，与 #146 相反）。
11. 协议引用的「语义兼容性」检查（b3 排除 4，论文 §6.6 自述不可判定）。
12. 把四强制触发点完全机械化（b2 明确结论：触发是语义判断；只把「触发后有无产物」机械化）。

## C3. HOOK（需用户拍板）

- 是否把上面 1-5 中任一项**立项**（立项则各走 core-plan Phase 0）；
- 若立项 1/3/5，注意 **RE-Framework 是只读引用**（`E:\PYTHON\RE-Framework`）——改动须落在框架仓库自身并由 ref-maintain 走 sync/selfcheck；CoreSwap 侧只能落「运行时纪律条款」。

## D. 诚实声明

- 本综合基于：论文原文（主会话读 §1/§2/§3.1-3.4/§4.2.2/§4.3.4-4.3.5/§5.1.1/§5.2/§6.1-6.7/§8）+ 三份透镜 draft + 一次 spot-check（b3 的事实基础、b2 的 selfcheck 计数门禁）。
- 论文与三个目标的映射**全部是类比构造**，论文没有任何一句谈验证协议、工作流框架或 Minecraft worldgen（B3 已声明）。
- 未做：任何形式的实施改动、协议条款写入、框架文件修改。
