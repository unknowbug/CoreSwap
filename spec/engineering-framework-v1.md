# Engineering Framework v1 — 通用工程方法论协议（多领域模块化）

> **定位**: 从逆向到编程的通用工程方法论框架。领域无关核心（core）+ 按需加载的领域模块（re-binary / re-code / swe）。
> **来源**: 由 RE-Framework v1（42KB 单文件 CLAUDE.md，BitWarden 方法论）工程化、Skills化、SubAgents化拆分而来；验证协议继承自 [Anchorlaw Protocol v0.13](https://github.com/unknowbug/anchorlaw)（其前身 Practify 即 v1 引用的验证协议）。
> **状态**: candidate（本协议自身遵守置信度状态机——待实际项目验证后由用户拍板 confirmed）
> **当前版本**: v1.0

---

## 0. 关键概念

| 概念 | 定义 |
|------|------|
| **模块 (Module)** | 独立安装单元 = 一组 skill + 触发表 + 依赖声明 + 卸载保证。可单独启用/禁用 |
| **领域 (Domain)** | 模块的分类：`re-binary`（二进制逆向）/ `re-code`（代码逆向）/ `swe`（软件工程） |
| **层 (Layer)** | L0-L4（对齐 Anchorlaw §14.2）：L0 概念参考 / L1 静态审查 / L2 执行动作 / L3 反馈循环 / L4 协议维护 |
| **角色 (Role)** | scout / worker / judge — subagent 隔离执行（对齐 Anchorlaw §15），不占 manifest 名额 |
| **锚 (Anchor)** | `@anchor.test` / `@anchor.idk` — 声称的可验证载体（Anchorlaw §13，语言无关） |
| **声称 (Claim)** | 任何 AI 产出的结论性表述，必须挂 anchor 或标注来源 |

**核心设计原则**: 框架是**接口**不是**内容仓库**。任何模块的正文都不应常驻上下文——AGENTS.md 是探测器，按任务类型加载对应模块的 skill。

---

## 1. 核心铁律（领域无关，所有模块强制）

以下铁律不因模块而异，是框架的基线：

### 1.1 置信度状态机
- 任何 AI 产出默认 `draft`，经证据支撑可升 `candidate`。
- `confirmed` **只有用户亲自拍板后才能标记** — AI 永远不能自己写 confirmed。
- 审查角色只出审查意见，不直接改 status。
- 读他人产出时带怀疑态度，检查置信度标记。

### 1.2 产物必须落盘
- 分析结果 → `.artifacts/`（index.yaml 主索引 + 类/方法/函数产物）
- 思维链/推理过程 → `.investigations/`（任务简报、假设、发现、结论）
- 不要只留在对话里。

### 1.3 防幻觉（可追溯性）
- 每个结论要么有可追溯来源的验证，要么诚实标注 `@anchor.idk`。
- test anchor 的 source 字段**必填**，来源类型 `trace` / `memory` / `probe`（v0.7）/ `static`（**仅限 `@anchor.idk`**）；缺失 → 审查直接驳回（视为凭空编造）。
- **source 证据落盘**：source 指向的验证记录 **MUST 有可引用的落盘证据**（`.investigations/*/regression-record.md` 条目 + 命令 + 输出摘要）——judge 按此核对「验证是否真的跑过」（实战项目实证：无此机制时 source 只能靠事后补证）。
- **order-dependent 语义**：还原点涉及排序/缓存/平局/tie-break/遍历序时（三语言等价的结果等价不足以覆盖），@anchor 描述 MUST 标注 order-dependence，并验证「确定性 + 与参照实现查询序列对齐」（实战项目实证：生物群系平局 tie-break + Java ThreadLocal 缓存依赖查询序列）。
- `@anchor.idk` 必须具体到可验证的条件，不许模糊标注。

### 1.4 上下文隔离
- 单个任务过长时主动拆分子任务，每个子任务独立 investigation 目录。
- 避免在同一个对话里塞入过量原始数据（汇编/大段代码/超长日志）。

### 1.5 可验证的正确性
- 一切走 [Anchorlaw 协议](https://github.com/unknowbug/anchorlaw)（§3 接口面）。
- 不确定的部分显式标注，不许闷在注释里。
- 因外部依赖无法独立验证 → 诚实声明（degraded 路径，Anchorlaw §9），不可假装验证了。

### 1.6 模块边界
- 模块之间禁止隐式耦合：skill 只能引用自己的模块 + core + Anchorlaw 接口，**不得引用其他领域模块的 skill 正文**。
- **豁免**：core 模块的执行角色 skill（如 `core.worker`/`core.judge`）允许**引用领域 skill 名称**作操作手册/路由（引用 = 提及名称与职责，非拷贝正文/依赖实现）——协调角色必须知道领域 skill 存在才能路由。
- 违反 = manifest 校验失败（§2.5）。

---

## 2. 模块化接口契约（本协议核心）

### 2.1 模块定义

每个模块是一个**独立安装单元**，由四部分组成：

```
<module>/
├── SKILL.md 或 skills/          # 该模块的 skill 集（每个 skill 有 frontmatter）
├── 触发表                        # 触发场景 → 调用哪个 skill（AGENTS.md 中注册）
├── 依赖声明                      # 外部工具链/协议依赖（如 MCP、Anchorlaw CLI、反编译器）
└── 卸载保证                      # 说明：删除模块目录后，其余模块不受影响
```

模块契约：
- **自包含**: 模块内 skill 只依赖自身 + core + Anchorlaw 接口面。
- **可探测**: 模块注册自己的触发关键词/文件格式特征，由 AGENTS.md 探测器调用。
- **可卸载**: 删除模块 = 该领域不可用，其余模块与 core 照常工作（Anchorlaw §2 Uninstall Guarantee 的模块级延伸）。
- **单一职责**: 一个模块只覆盖一个领域的工作流，不跨域。

### 2.2 模块清单（v1.0）

| 模块 | 名称 | 覆盖场景 | 依赖 | 默认 |
|------|------|---------|------|------|
| `core` | 通用核心 | 置信度/落盘/架构设计/审查门/知识库 | 无（必装） | ✅ 必装 |
| `re-binary` | 二进制逆向 | 汇编/机器码/vtable/RTTI/xref/trace | IDA/Frida/x64dbg/CheatEngine MCP + Anchorlaw degraded | 可选 |
| `re-code` | 代码逆向 | 字节码/反混淆映射/类层次/行为验证（如 Minecraft） | 反编译器 + JVM/语言工具链 + Anchorlaw | 可选 |
| `swe` | 软件工程 | 常规开发/验证/审查 | anchorlaw CLI + anchorlaw-scanner | 可选 |

> `re-binary` 与 `re-code` 的边界：输入是机器码还是字节码/源码。两者共享 core 的产物 schema（§5）与验证协议，差异仅在分析流程 skill。

### 2.3 探测与自适应（任务类型 → 模块加载）

AGENTS.md 在任务开始时运行**探测器**，按以下决策树只加载匹配的模块：

```
用户给出任务
    │
    ├── 输入/目标是二进制（.dll/.exe/.so/.bin）、汇编、IDA/Ghidra、
    │   vtable、RTTI、xref、脱壳、反调试 → re-binary 模块
    │
    ├── 输入/目标是字节码/源码（.jar/.class、Java/Kotlin/C#）、反混淆
    │   映射、类层次还原、mod 逆向（如 Minecraft）→ re-code 模块
    │
    ├── 任务是编写/修改/审查代码、协议设计、常规开发 → swe 模块
    │
    ├── 混合任务（如"逆向一个 API 并复刻其行为"）→ 主模块 + 按需附加
    │   （re-code/re-binary 产出 → swe 复刻，各自走各自 skill）
    │
    └── 无法判断 → 询问用户目标格式，不猜
```

加载规则：
- 只加载匹配模块的 skill，**不加载不匹配模块**（卸载保证的运行时形态）。
- 模块内 skill 仍按 L0-L4 按需加载，不一次性全量读入。
- 探测器自身不产生分析结论，只做路由。

### 2.4 卸载保证

```
删除 <module>/ 目录 → 该领域全部 skill 从可用集消失：
  - AGENTS.md 触发表中该模块的行失效（不报错，只是不再触发）
  - core 与其他模块照常工作
  - 该模块曾产出的 .artifacts/ 产物保留（历史数据不因模块卸载而清除）
```

- 卸载一个模块**不允许**破坏其他模块的 manifest 一致性（§2.5 校验器守护）。
- 产物目录与模块目录解耦：`.artifacts/` 属于项目，不属于任何模块。

### 2.5 manifest 校验规则

每个模块安装/更新后运行校验器（实现: `scripts/validate_manifest.py`，对齐 Anchorlaw `test_skills.py` 守护 manifest 的思路）：

| 规则 | 说明 |
|------|------|
| R1 frontmatter 完整 | 每个 SKILL.md 有 `name` / `description` / `layer` / `execution` |
| R2 无孤儿引用 | skill 引用的其他 skill 必须存在于 自身模块 ∪ core ∪ Anchorlaw 接口面 |
| R3 层合法 | layer ∈ {L0, L1, L2, L3, L4, role}，role 必须声明 `runAs: subagent` |
| R4 触发表一致 | AGENTS.md 注册的每个 skill 都有实体文件，反之亦然 |
| R5 无跨模块引用 | 引用其他领域模块的 skill 正文 → 校验失败（§1.6）；**豁免**：core 角色 skill（worker/judge）引用领域 skill 名称作路由 |

---

## 3. Anchorlaw 四接口面引用

本框架的验证/执行/集成语义**不自行维护**，全部引用 [Anchorlaw Protocol v0.13](https://github.com/unknowbug/anchorlaw)（MIT），保持单一事实源：

| 接口面 | Anchorlaw 章节 | 本框架的使用方式 |
|--------|---------------|-----------------|
| **Claim（声称）** | §13 Anchor Abstraction + §5 Anchor Semantics | `@anchor.test`（description + test_fn + source 必填）/ `@anchor.idk`（具体未知项）；source 类型 `trace`/`memory`/`probe`（v0.7）/`static`（仅限 idk）；source artifact requirement（v0.7——source 必须有磁盘证据）；staleness（90 天）、健康状态（healthy/unverified/degrading/stale_unknown/skeleton/uncompilable） |
| **Knowledge（知识）** | §14 Agent Skill Manifest | 本协议 §0 的 Layer 模型、模块 skill 的 frontmatter 格式、manifest 一致性守护（本协议 §2.5） |
| **Execution（执行）** | §15 Execution Topology | 角色隔离：scout/worker/judge 在 subagent 子进程中运行，只返回最终答案 + 产物引用；**域收窄（v0.13）**——"programming is constructive" 限于 input-contract 域（已注册 §11 audit），**逆向工程明确域外**（探索型、验证无界），两模式互补；retry cap = **evidence saturation（v0.13，§9.4）**：3 轮**无新数据层证据**（trace/probe）强制回数据层/升级人类，产生新证据的轮次重置 streak 不消耗 cap，工程修复不计数；verification termination gates（外部测试集 + 三档审查意见，blocking 仅限 test/compile/claim 矛盾）；执行模式选择（收敛任务 inline / 发散任务 MAY subprocess） |
| **Host（宿主）** | §16 Host Integration Contract | AGENTS.md 作为宿主集成点：触发表注册、confirm hook（confirmed 仅人类授予）、产物契约（落盘 + 索引更新）；hosts 自供执行者（本框架用 core.worker/core.judge）；**RE handover 标准（v0.13，§16.1）**——RE host 的 input contract 仅在 vanilla 行为模型**收敛**后算 confirmed，混合探索-构造任务按子部分收敛状态分流（确定子部分进流水线，未收敛部分由 RE host 自理）；三协议闭环：requirements → Anchorlaw → RE framework |

**与 Anchorlaw 的版本同步契约**：本框架对 Anchorlaw 为**协议引用**（单一事实源），升级时对照其 changelog 核对引用条款：
- 当前引用基线：**Anchorlaw v0.13**（2026-08-10）
- 已知同源双写：本框架 §4.5 执行强制链 ↔ Anchorlaw §15.4 judge institutionalization（同一批实战反馈双侧落地）——升级 Anchorlaw 时须对照其 changelog 同步核对
- 检查动作：Anchorlaw 发新版本后，重新 `git grep 'v0\.[0-9]'` 本仓库，核对版本号 + 引用条款是否仍成立

**验证协议三态（Anchorlaw §9）** 在各模块的应用：
- `re-binary`：.cpp Lift 产物常因外部符号不可编译 → degraded 路径（uncompilable_functions.yaml + 诚实声明）
- `re-code`：反编译/重写产物通常可编译 → 全功能路径（模式 A）+ 行为测试（运行时 hook → trace source）
- `swe`：全功能路径（模式 A），scanner 静态审查

---

## 4. 通用工作流（core 提供）

所有模块共享同一工作流骨架（各模块 skill 填充其中的"分析"环节）：

```
Phase 0: 强初始化（每个 session 强制，按序，未经完成不得开始分析——AGENTS.md 〇段）
    ├── STEP 1 读知识库: 查 knowledge/INDEX.md + 相关条目（core.knowledge，先于一切）
    ├── STEP 2 规划工作: 架构设计（轻量 ≤3 要点 / 重量完整文档），用户确认后才开始
    └── STEP 3 预置子角色介入点: scout/worker/fan-out/judge/knowledge 全部介入时机随计划落盘
        （执行只核对不补排——执行强制链 §4.5 的初始化形态）
    │
Phase 1: 勘探（scout 角色，subagent）
    ├── 只读：入口定位 / 交叉引用 / 依赖摸底 / 粗略分类
    └── 产物 → .investigations/（不写 .artifacts/）
    │
Phase 2: 分析（worker 角色，subagent）
    ├── 按模块加载分析 skill（re-binary → lift/classify；re-code → deobfuscate/classmap）
    └── 产物 → .artifacts/（status: draft）
    │
Phase 2.5: 验证（Anchorlaw 协议）
    ├── 全功能（可编译）→ anchorlaw test 运行 @anchor.test
    └── 降级（不可编译）→ uncompilable_functions.yaml + 诚实声明
    retry cap = evidence saturation（Anchorlaw §9.4, v0.13）: 连续 3 轮无新数据层证据（trace/probe）→ 强制回 Phase 1 数据层采集/升级人类；产生新证据的轮次重置计数；工程修复/代码迭代不计数
    验证执行者分离: 分层标注（Full/Partial/Degraded）以实际执行为准——谁跑的、什么环境；
                    分析产物在无运行时证据前不得升 candidate（除非显式声明降级）
    │
Phase 3: 审查（judge 角色，subagent）
    ├── 证据完整性（source 字段）/ 置信度合法 / 产物契约 / 噪声卡历史 / retry cap
    └── 只出审查意见，不改 status
    │
用户拍板 → confirmed → 归档
```

人工 HOOK 点（任何模块通用）：
- 架构批准（Phase 0 后）
- 重大方向变更（发现与架构预期不符 → 暂停回 Phase 0）
- 多假设竞争（fan-out 候选对比后）
- confirmed 授予（Phase 3 审查后）

### 4.1 执行模式路由（任务类型 → 执行模式）

执行模式**随任务类型切换**，不是一刀切 subagent（实战项目验证：编程在隔离执行下不可达——无编译/运行闭环）：

| 任务类型 | 模块 | 执行模式 | 理由 |
|---------|------|---------|------|
| 逆向（二进制/代码） | re-binary / re-code | **subagent 为主** | 深度分析需隔离上下文（防污染主会话/防注意力稀释）；工具执行走命令委托（§4.3） |
| 编程（写码/重构/验证） | swe | **主会话为主** | 工程迭代需直接闭环（写码→编译→运行→调试→验证）；隔离执行阻断迭代 |
| 审查 | core.judge | subagent（隔离） | 独立视角，防锚定 |

规则：
- **逆向任务**：分析/解读/审查/代码交付 → subagent（scout/worker/judge）；编译/回归/工具采集/崩溃调试/git/签核 → 主会话（只执行不解读）。
- **编程任务**：写码/重构/编译/运行/调试/验证 → 主会话直接闭环；subagent 仅用于隔离审查（如独立 code review），**不派 subagent 做核心写码/迭代**。
- **混合任务**（如"逆向 API 并复刻其行为"）：按子任务类型分别路由——逆向部分 subagent，编程部分主会话。

**与 Anchorlaw v0.13 的两域对齐**：Anchorlaw v0.13 将 "programming is constructive" 收窄到 input-contract 域（§11 audit 注册），**逆向工程明确协议域外**（探索型、验证无界），两模式互补。本框架的执行模式路由与两域结构**同构**：
- **RE 探索域（re-binary/re-code）** = Anchorlaw 域外——由本框架自理（subagent 勘探/分析 + 主会话命令委托闭环）；探索收敛后按 §16.1 RE handover 标准产出 confirmed input contract
- **构造域（swe）** = Anchorlaw input-contract 域内——接 Anchorlaw 流水线语义（判据先行 + evidence saturation + Judge 门禁），但执行模式保留主会话为主（§15.3 SHOULD 允许 host 自选；RE 复刻的"确定的子部分"是混合任务切片，非纯构造域）

> 注：v0.13 之前评估的「编程=主会话为主」与 v0.10「并行 Worker 写码」范式冲突已由域收窄**消解**——并行 Worker 仅约束已收敛构造域，RE 域外由本框架自理，无需偏离声明。

### 4.2 职责边界（按任务类型两套）

**逆向任务职责边界**（实战项目 第九章实证）:

| 执行者 | 可做 | 禁做 |
|-------|------|------|
| 主会话 | 编译 / 回归 / 工具采集 / 崩溃调试 / git / 签核 | 解读分析结果（**只执行不解读**） |
| subagent | 分析 / 解读 / 审查 / 知识库 / 代码交付 | 执行环境敏感命令（无 shell 时走命令委托 §4.3） |

**编程任务职责边界**:

| 执行者 | 可做 | 禁做 |
|-------|------|------|
| 主会话 | 写码 / 编译 / 运行 / 调试 / 验证（直接闭环） | 把核心写码/迭代派给 subagent（隔离阻断闭环） |
| subagent | 隔离审查 / 独立验证（可选） | 无环境命令执行（除非宿主放行工具链） |

**「主会话只执行不解读」原则**：主会话执行命令委托后不自行解读原始输出——落盘回传 subagent 解读，防止主会话结论与分析 subagent 冲突/污染。

### 4.3 命令委托模式（subagent 无执行环境时的标准路径）

当分析 subagent 沙箱无 shell / 只读白名单拦截可执行程序（探针、编译器、运行测试）时，采用命令委托：

```
subagent（分析侧）                    主会话（执行侧）
    │  ① 提交命令模板                       │
    │  （命令 + 参数 + 期望输出）  ────────►  │
    │                                       │  ② 执行（不解读）
    │                                       │  ③ 原始输出落盘
    │  ◄──── ④ 落盘路径回传  ─────────────  │
    │  ⑤ 读取落盘输出并解读                  │
```

约定：
- **命令模板格式**：`命令` + `参数` + `输出落盘路径`（`.investigations/<任务>/cmd-output/<NNN>.<ext>`）
- 主会话执行时**不解读**；原始输出**必须落盘**（core.artifact），subagent 基于落盘文件解读
- 失败/异常输出同样落盘（不掩盖）；输出格式与预期不符 → 如实记录偏差

### 4.4 subagent 写码强制自检清单

subagent 交付代码（逆向还原/移植/修复）前**必须逐项自检**并在交付物中附勾选结果：

- [ ] **类型宽度**：明确整数宽度（int32/int64/size_t）；跨语言移植检查 long 宽度差异（如 MSVC long=32 位截断是真实 bug 源）
- [ ] **move/拷贝语义**：显式区分移动与拷贝，无悬垂引用/迭代器失效
- [ ] **异常路径**：throw/catch 覆盖，错误分支有返回；与参照实现异常行为对齐
- [ ] **对拍点**：与参照实现（Java/其他语言）的对拍点明确标注（输入/输出可比对）
- [ ] **自检声明**：交付时附清单勾选结果；无法独立编译的如实声明（uncompilable_functions.yaml），不假装自检通过

### 4.5 执行强制链（judge/scout/fan-out/knowledge 强制触发点）

**「编程=主会话直接闭环」≠「自评即可」**（实战项目实证：收敛门被误读为自评后，judge 全程缺位，关键结论靠主会话自评——「高频方向」结论错误到后期才被证反，收尾补位 judge 一次抓到 5 项问题）。审查门是强制项，独立 subagent 审查不可因主会话闭环而省略。

**judge 强制触发点**（core.judge，MUST/SHOULD 分层）：

| 触发点 | 级别 | 说明 |
|--------|------|------|
| confirmed 授予前 | **MUST** | 只有用户拍板能授 confirmed，但拍板前 MUST 有 judge 意见 |
| 重大转向 | **MUST** | 结案重开、根因定论（如"无 bug"）、范围决策（如扩展分析范围） |
| candidate 授予 | SHOULD | 各阶段结论（分析产物升 candidate）应过 judge；至少留审查意见 |
| 收尾交付 | **MUST** | 交付前 judge 核对三源（§4.3 三源核对） |

**judge 步骤预置**：judge 随 todo 计划**预置**（core.plan 架构设计阶段就排 judge 项，不是事后补）；工作流允许无 judge 的探索（draft 阶段），但不允许跳过 judge 直接给结论。

**scout 强制触发条件**（re.scout / recode.scout）：

| 场景 | 级别 | 说明 |
|------|------|------|
| 「机制未明」类大排查初期 | **MUST scout** | 管线阶段/子系统依赖摸底（如逆向目标的多阶段处理链），**禁止主会话直接跳入单点定位** |
| 入口明确但路径未知 | MUST scout | 已知入口（如 mismatch 明细）→ 先勘探全链路再定位 |
| 入口明确且机制已知 | 可跳过 | 收敛型单点分析（用户已明确路径）允许主会话直接做 |

勘探产物（`.investigations/<任务>/管线地图/依赖图`）作为定位前置——先有地图再下钻。

**fan-out 强制触发点**（core.fanout，与 judge/scout 并列第三条）：

| 场景 | 级别 | 说明 |
|------|------|------|
| 多疑点冲突 / 多互斥假设并存（判定树分叉 ≥2 个互斥候选） | **MUST fan-out** | 并行派 worker 各验一分支（core.fanout 产 .bN），**禁止主会话逐个自推** |
| 同一现象多机制候选 | MUST fan-out | 如「e 翻转 / pocket / 结构」多机制并存 |
| 旧结论 vs 新证据冲突 | MUST fan-out | 如已归档结论与新增证据矛盾——并行重验各分支 |
| 子假设再分叉 | MUST fan-out | 候选内部再拆出互斥子候选（(a)/(b) 级）同样适用 |

原则：
- **不因候选小/看起来简单自推**——主会话逐个自推的判断成本远高于派 worker 的隔离成本。
- **自检提示**：主会话深钻到「第二轮仍无定论」时自查是否已分叉——是则立即 fan-out。
- **三触发点并列独立**：scout（机制未明勘探）→ fan-out（多假设分叉）→ judge（结论审查），任一触发即执行；互斥假设才 fan-out（互补假设应合并探索，core.fanout）。

**知识库/结论落盘强制触发点**（第四条，core.knowledge + core.worker）：

| 场景 | 级别 | 说明 |
|------|------|------|
| 结论性 docs（主题篇/时间线/结论文档）写入前 | **MUST subagent 产出草稿** | core.worker 解读产出 → 主会话只应用 + 一致性验证 |
| knowledge/discovered 结论条目写入前 | MUST subagent 产出（或至少标注产出者） | 结论性内容走 worker；单条简单发现标注产出者 |
| 过程性记录（.investigations 中间产物/排查日志） | 主会话可写 | 非结论，不进强制链 |

**过程/结论分界**：结论性 = 主题篇、时间线、discovered 结论条目、结论文档（沉淀后会被复用的知识）；过程性 = 临时排查记录、中间产物（一次性的工作痕迹）。

**映射与验证**：`core.worker`（解读产出草稿，按 core.knowledge 格式）→ 主会话应用落盘 + scan/一致性验证（INDEX.md 同步、格式合规）。

**反模式**：主会话直接写结论性 docs = **自评污染**（结论未经独立解读，与「主会话只执行不解读」同理）；「主会话可做 AGENTS.md/工具链文档维护」不得扩大解释到结论性知识库。

---

## 5. 产物 schema 通用约定（语言无关）

产物格式与模块无关。定位字段支持三种形式，按模块选用：

| 字段 | re-binary | re-code |
|------|-----------|---------|
| `address` | 虚拟地址（`0x1400077c0`） | 字节码偏移（`0x1A`）或方法句柄 |
| `name` | 符号名/推测名 | 反混淆名/成员名 |
| `evidence` | 寄存器快照/汇编引用 | 源码引用/映射文件条目 |

### 5.1 主索引 `.artifacts/index.yaml`

```yaml
schema_version: 1
project: <project 名>
module: <core | re-binary | re-code | swe>   # 记录产出模块（卸载时提示产物归属）
entries:
  - id: '<module>:<name>:<locator>'
    path: '<产物相对路径>'
    kind: class | method | function | xref
    status: draft | candidate | confirmed
```

### 5.2 类产物 `class.yaml`（通用版）

```yaml
schema_version: 1
class_name: <name>
locator: <地址 | 字节码偏移 | 类全名>          # 语言无关定位
status: draft | candidate | confirmed
members:
  - offset: <偏移>
    type: <type>
    name: <推测名>
    evidence: '<观察证据>'
methods: [<method 名列表>]
```

### 5.3 方法/函数产物 `method.yaml` / `function.yaml`（通用版）

```yaml
schema_version: 1
name: <方法名/函数名>
locator: <地址 | 字节码偏移 | 签名>
status: draft | candidate | confirmed
signature: <签名>
return_type: <类型>
anchor_path: <对应 _anchors.py 或等效 anchor 载体路径>
retry:                          # 逆向假设验证轮次记录（judge 核对用，spec §4.5）
  count: <N>                    # Lift→Verify 轮次（工程修复迭代不计数，§4.1）
  over_cap: false               # 是否超限（>3）；超限必须声明 + 回勘探取新证据
  history: []                   # 可选：每轮摘要（如 "noodle 高频假设 phase11 被证反"）
uncertain_areas:            # 来自 @anchor.idk
  - '<具体未知项>'
dependencies: []
notes: |
  <还原过程备注>
```

### 5.4 跨引用 `xref.yaml`

```yaml
编号: xref_<NNN>
状态: draft | candidate | confirmed
从: {binary: <来源>, 定位: <locator>}
到: {目标: <目标>, 类型: <引用类型>}
关系类型: 调用 | 继承 | 数据引用 | 事件订阅
证据: <可追溯描述>
```

### 5.5 降级产物 `uncompilable_functions.yaml`（re-binary 常用）

```yaml
- function: <name>
  locator: '<locator>'
  uncompilable_reason: "<原因>"
  missing_deps: []
  suggested_path: "<建议路径>"
```

---

## 6. 知识库生长机制

- 知识库分两层：`knowledge/builtin/`（预置，不随项目变化）+ `knowledge/discovered/`（项目运行中 AI 自动写入）。
- `knowledge/INDEX.md` 是总入口，分析前先查。
- **自动贡献**: 任何阶段发现可复用知识 → 立即写入 `discovered/`，更新 INDEX.md。
- 触发条件（模块无关）：编译器/语言惯用法、还原工具误译模式、算法/协议指纹、反逆向/混淆手法、跨模块通用模式。
- 每条发现标注: 时间、来源定位、发现阶段、置信度、如何利用。
- 知识条目分模块归属（`module:` 字段），与产物解耦——跨模块知识标注 `module: core`。

---

## 7. 版本管理

- `<name>.yaml` — 当前活跃版本（主文件）
- `<name>.vN.yaml` — 时间线历史版本（旧主文件改名 .vN）
- `<name>.bN.yaml` — fan-out 候选分支（多假设并行探索）
- 升级流程: 旧主文件改名 .vN → 新候选确认为活跃 → 复制为 .yaml
- **禁止在没有用户确认的情况下删除旧产物**（.vN 保留，.bN 淘汰后仍留审计追溯）

### Fan-out 并行

多假设互斥时（如"这个函数是 CRC32 还是 djb2？"）：
```
主会话 → WorkerA(假设A) + WorkerB(假设B) + WorkerC(假设C)  [并行 subagent]
       → Judge 对比所有 .bN → 审查意见 → 用户拍板 → 优胜者复制为活跃版
```
- 适合互斥假设，不适合互补假设（互补应合并）。
- 证据同时支持多假设 → 不强选，标 candidate 等更多证据。

---

## 8. 行为准则

### 启动时
- 已有 CLAUDE.md/AGENTS.md + `.artifacts/` → 从上次断点继续；新项目 → 先初始化骨架（index.yaml 空索引 + 目录）。

### 工作时
- 分析前先查 `knowledge/INDEX.md`。
- 每个独立任务先建 `.investigations/<任务>/任务.md`。
- 产出严格标记 `status: draft | candidate`。
- 遇到重大发现或方向决策主动打断用户，不闷头跑偏。
- 发现可复用模式 → 立即写入知识库。

### 禁止行为
- ❌ 直接把单一工具输出（F5/反编译器/LLM 自身）当最终答案
- ❌ 产物只留在对话里不落盘
- ❌ 自己把 status 改成 confirmed
- ❌ 在没有用户确认的情况下删除旧产物
- ❌ 跨模块引用其他领域模块的 skill 正文（§1.6）
- ❌ @anchor.test 缺 source 字段（视为凭空编造，审查直接驳回）
- ❌ @anchor.idk 写得模糊（必须具体到可验证的条件）
- ❌ 连续 3 轮假设验证仍无新数据层证据（evidence saturation 违规，Anchorlaw §9.4 v0.13）— 必须回勘探取新证据，不得继续盲迭代

---

## 9. Maturity / 诚实声明

| 组件 | 状态 | 说明 |
|------|------|------|
| 模块化接口契约（§2） | **CONJECTURE** | 接口已定义，尚无项目同时使用多模块验证其独立性 |
| core 通用工作流（§4） | **VERIFIED 于 RE 场景** | 承自 RE-Framework v1（BitWarden 方法论，实际 RE 项目使用） |
| re-binary 模块 | **VERIFIED 于 RE 场景** | Lift/Class-identify 流程在 v1 中经实际项目验证 |
| re-code 模块 | **CONJECTURE** | 新增设计，等待 Minecraft 等代码逆向项目实践 |
| swe 模块 | **VERIFIED** | 直接继承 Anchorlaw 协议（scanner/anchor 已实测） |
| 探测与自适应（§2.3） | **CONJECTURE** | 决策树尚无实际项目验证路由准确性 |
| 执行模型/职责边界（§4.1-4.2） | **VERIFIED（实战项目实证）** | 第九章「主会话/subagent 职责边界」本地落地形态实战运行（主会话可做编译/回归/工具采集/崩溃调试/git/签核；「主会话只执行不解读」） |
| 命令委托（§4.3） | **VERIFIED（实战项目实证）** | 临时手动版（worker 下模板 → 主会话执行不解读 → worker 解读）实测「效果可行」；框架级契约本次正式化 |
| judge 三源核对 | **VERIFIED（实战项目实证）** | 审查基于过期快照导致 64 位误报 → 三源核对落地后消除 |
| retry cap 区分 | **VERIFIED** | 用户明确拍板：≤3 只约束逆向假设验证轮次，工程修复不计数 |
| order-dependent 语义 | **VERIFIED（实战项目实证）** | 生物群系平局 tie-break + Java ThreadLocal 缓存依赖查询序列，实证 + 修复闭环 |
| merge_index 工具（R-2） | **CONJECTURE** | 冒烟测试通过，尚未实战（替代主会话手动合并） |

> 诚实声明：标注 CONJECTURE 的部分是工作假设，价值待实践检验。用实际项目验证它们，而不是假设它们有效。
