---
id: b2-ref-framework
status: draft          # 草稿——AI 不自我授予 candidate/confirmed（spec §1.1）
worker: subagent       # 本文件由 subagent 产出（b2 分支，lens: re-framework）
date: 260917-05
lens: re-framework      # 透镜 = RE-Framework v2 工作流框架现状对照
paper: "A Programming Paradigm for Spatiotemporal Composability (arXiv 2608.25512, PKU + DeepSeek-AI, Cordis)"
inputs_read_firsthand:
  - "E:\\PYTHON\\CoreSwap\\.tmp\\paper-2608.25512.txt（§1-§6 分段读完；元理论 §4.3 略读）"
  - "E:\\PYTHON\\RE-Framework\\spec\\engineering-framework-v1.md（全文 514 行）"
  - "E:\\PYTHON\\RE-Framework\\dsh\\skills\\{core-plan,core-artifact,core-version,core-fanout}\\SKILL.md"
  - "E:\\PYTHON\\RE-Framework\\dsh\\scripts\\{install.ps1,selfcheck.ps1}"
  - "E:\\PYTHON\\CoreSwap\\AGENTS.md（项目级运行时落地）"
  - "E:\\PYTHON\\CoreSwap\\knowledge\\discovered\\workflow-patterns.md（结构 + #155-#162 尾部）"
  - "E:\\PYTHON\\CoreSwap\\knowledge\\INDEX.md"
  - "实测：三处安装位（upstream dsh/skills、~/.dsh/skills、项目 .dsh/skills、preset 内嵌）逐文件 sha 对比"
not_read_firsthand:   # 明确区分：以下为**未一手核实**的推断
  - "core-judge / core-worker / core-knowledge SKILL.md 正文（本文件只在 spec §4.5 与 AGENTS.md 层面引用其职责）"
  - "论文 §4 演算与 §4.3 元理论证明细节（§9.4 saturation 与 progress/confluence 的对应为**我的类比**，非论文原话）"
  - "Anchorlaw 协议正文（经 RE-Framework spec §3 引用面转述，未直读 protocol-v0.21.md）"
---

# b2 — 论文《A Programming Paradigm for Spatiotemporal Composability》× RE-Framework v2 对照：可执行改进建议

## 0. 读法与结论摘要

本篇以**论文机制（页码）→ 框架现状（file:line / 技能名 / 知识库条目号）→ 缺口 → 改法 → 收益与成本**五段式，给出 **6 条**建议，分三档。

**总判断**：论文最有价值的**不是**它的形式化（§4 演算 + 元理论对 RE-Framework 这种 Markdown-based 方法论框架的可移植性有限），而是它的**三条工程不变量**——① 每个 context 变换自带逆（§3.1）；② 每个依赖声明带 satisfaction 谓词 + 变化分类（§3.2.2）；③ 装载规格是**声明式持久记录**、由 loader 与运行态双向对账（§5.2.1）。**RE-Framework 当前恰好在这三条上都是「粗粒度 workaround + 纪律」**：安装靠全量重装、回退靠文本级 .vN/supersedes、触发靠人工预置表格。这三处正是可机械化的地方。

**一句话回答**：论文对 RE-Framework 最有用的一条是——**把「装载规格」提升为可对账的声明式记录（§5.2.1 的 entry + §5.2.2 的 reconciliation），使安装产物的身份/版本/归属成为机械可判事实，而不是靠重跑 install.ps1 的自觉**；这是唯一一条能同时消解 #137（外部锚未核）、#156（口径失传）、#161（世界身份失传）三个已实证错误的机制，且不依赖任何形式化能力。

---

## 1. 「立即可做」档

### 建议 1（最高价值）：安装产物「身份清单」对账——用 sha 内容比对取代数量计数

**论文机制**：§5.2.1 Declarative Configuration（p.65）——"Everything an instantiation needs can be declared"；entry 记录 `id / url / isolate / intercept / config / disabled` 六字段（Definition 81），"An entry can serve as a faithful specification because what supports a fiber is exactly what an entry records"。§5.2.1 Reconciliation（p.65）按**字段级 dispatch**（id/url→rebuild、isolate→reassign、intercept→in-place、config→diff、disabled→unload）应用最小破坏操作。

**框架现状（file:line 一手核实）**：
- `dsh/scripts/install.ps1:126-131`——技能同步是**全量擦除重拷**：`Remove-Item $presetSkills -Recurse -Force` 后 `Copy-Item -Path (Join-Path $srcRoot 'skills') -Destination $presetSkills -Recurse -Force`；用户级则 `Copy-Item ... -Force`（盲覆盖）。全脚本**无任何内容比对**。
- `dsh/scripts/selfcheck.ps1:49-55`——第 3 段只校验**目录计数**：`$count = @(Get-ChildItem -Path $presetSkills -Directory).Count`，判定 `if ($count -lt 17)`；用户级同理。**注释写的是 "OK embedded skills: 17 directories"**——即它保证的是「17 个目录存在」，不是「这 17 个目录的内容 == 上游」。
- **实测证据（本 session 亲自跑，非推测）**：逐文件 sha256 对比 upstream `dsh/skills` vs `~/.dsh/skills` vs `E:\PYTHON\CoreSwap\.dsh\skills` vs `~/.dsh/.agent-presets/re-framework/skills`——**四处全绿、零 DRIFT、零 MISSING**。这是**好消息**，但它同时暴露缺口：这份绿色是**手工维护的偶然结果**，不是任何机制保证的（`install.ps1` 只管前两处；项目级 `.dsh/skills` 由 Anchorlaw `-Project` 模式 + 副本合并而来，**无脚本覆盖**，AGENTS.md 明示「项目级 `.dsh/skills` 5 技能副本本日补齐」，即人工补的）。
- 对照 `selfcheck.ps1:89-111` 第 5 段（preset 行解析门禁）——**这一段恰恰做对了**：`audit_preset_rows.mjs` fail-closed，且注释里写明「a gate that silently goes green when it cannot execute is the same failure class as the incident it guards against」（2026-09-09 upstream 改名漂移事件）。**同构的哲学没有用到第 3 段**——第 3 段正是「silently goes green」的那种门。

**缺口**：技能/插件/preset 的安装产物是**无身份**的——没有任何落盘记录说明「此刻 `~/.dsh/skills/core-plan/SKILL.md` 的 sha 应为 X」。因此：
1. 上游改了 skill 正文但没重跑 install.ps1 → **静默陈旧**，selfcheck 数量门照绿；
2. 项目级 `.dsh/skills` 副本无脚本覆盖 → 漂移只能靠人肉发现；
3. 与知识库 #137（外部锚 MUST 用该锚的属主工具自核——引用不存在的对象 = 幻觉签名）**同族**：这里引用的是「已安装技能」，而「已装」这个事实无锚可核。

**改法（可落地）**：
1. **`install.ps1` 生成 manifest 而非盲拷**：安装完写 `~/.dsh/.agent-presets/re-framework/install-manifest.yaml`（以及 `~/.dsh/skills/.re-framework-manifest.yaml`），字段对齐 Definition 81 的 entry 形态：
   ```yaml
   schema_version: 1
   source_root: E:\PYTHON\RE-Framework\dsh        # = entry.url 的类比（来源身份）
   source_commit: <git rev-parse HEAD>            # 装载规格的可追溯锚
   installed_at: <ISO8601>
   artifacts:
     - id: skills/core-plan            # entry.id（稳定标识，用于对账键）
       target: ~/.dsh/skills/core-plan/SKILL.md
       sha256: <hash>
     - id: preset/agent.cordis.yml
       target: ~/.dsh/.agent-presets/re-framework/agent.cordis.yml
       sha256: <hash>
   ```
2. **`selfcheck.ps1` 第 3 段从「计数」升级为「内容对账」**（这是最小改动、最大收益的一步）：对每个 artifact 重算 sha256 与 manifest 比对；`MISSING` / `DRIFT` / `ORPHAN`（磁盘有、manifest 无 = 上游已删除的残留）三类分别报错。**保留计数检查作为二级断言**（防 manifest 本身被删）。
3. **项目级 `.dsh/skills` 纳入同一对账**：新增 `-Project <dir>` 参数或独立脚本，使「项目副本 == 上游」成为**可机械判定**事实，替代 AGENTS.md 里「本日补齐」这类人工叙述。

**收益**：把「装没装、新不新、干净不干净」从**人工状态叙述**变成**机械可判事实**。直接消解：上游改了 skill 未重装（静默陈旧）；项目副本漂移；上游删除某技能后旧副本残留（**orphan——当前 install.ps1 对用户级 `~/.dsh/skills` 只做 `Copy-Item -Force`、从不清除已删技能**，这是已存在的真空洞，非推测）；以及 #137 同族——「已安装」这一断言的属主工具可核。

**成本/风险**：改动集中在 `install.ps1`（新增写 manifest 段）+ `selfcheck.ps1`（第 3 段重写），约 60-90 行 PowerShell，无新依赖。风险：① sha 对账会把「有意的人工微调」判为 DRIFT——需在 manifest 里支持 `allow_local_override: true` 白名单；② `install.ps1` 需访问 `$HOME\.dsh`（工作区外）——按现有脚本已如此，无新增边界；③ 需要 `git rev-parse` 取 source_commit，仓库非 git 时降级为 `<unknown>` 并诚实声明（不 fail-open）。

---

### 建议 2：副作用回退契约——给「无逆操作」的框架副作用补上逆，或显式声明域外

**论文机制**：§3.1.1 Definition 2-3（p.10）——effect context `∂Γ ≔ Γ × (Γ→Γ)`，`track_Γ(f,g)` 把逆 `g` 复合进累加器 `φ`；"all effects performed on ∂Γ can be tracked and the context can be recovered"。§3.3.1（p.22）明确 plug-in 隐喻："Loading a component corresponds to executing its effects (plugging in); Unloading ... reverting its effects (unplugging, without affecting other running components)"。§5.1.1 Algorithm 1（p.59）实现——`ctx.effect` 是**唯一**原语，"coeffect provision, component instantiation, and every other context-mutating operation reduces to a ctx.effect call"。**§6.1 System Boundary（p.70）是本节关键**：位置**在界内** ⟺ 系统能①独占修改**且**②恢复到修改前状态；两者任一失败 → 该操作 = `idΓ`，**既不追踪也不回退**——这是**诚实声明的形式化**，不是缺陷。§6.1 并区分 acquisition（界内、可逆）与 emission（跨界、只可 withhold/compensate）。

**框架现状（一手核实）**：框架的回退纪律**全部是文本级**，且**已相当完备**：
- `spec §7` + `core-version` 技能——`.yaml` 活跃 / `.vN` 历史 / `.bN` 候选，「禁止无用户确认删除旧产物」；
- `spec` 引用 Anchorlaw v0.21 §15.4——supersedes 双指针 + 一行理由 + 原文永不删除/改写；
- `spec §6`——知识库 append-only + 压实 `consolidated` 原文保留；
- `.artifacts/index.yaml`——产物索引留痕。

**但以下副作用当前无回退契约**（逐条给出证据）：

| 副作用 | 现状证据 | 论文视角的定位 |
|---|---|---|
| 用户级 `~/.dsh/skills/*` 副本 | `install.ps1:130` 只 `Copy-Item -Force`，**无逆操作**；删技能 = 手动 `Remove-Item` | §3.1 effect **无逆** → 落在 §6.1 **界外**（`idΓ`），但**从未声明** |
| 项目级 `.dsh/skills/*` + `.dsh/plugins/*` | 无脚本；AGENTS.md 记载「本日补齐」为人工动作 | 同上，且**无 owner**（属主不明） |
| preset 内嵌技能/插件 | `install.ps1:128-129` 先 `Remove-Item -Recurse` 再拷——**重装即销毁**（粗粒度 workaround 的教科书形态） | §1.2.3 "coarse-grained workaround"（p.5-6）——"reconfigure only by restarting, discarding runtime state" |
| profile `cordis.patch.yml` 行 | `install.ps1:77-114` 用内联 Python 删除 `re-framework-tools-global` 行——有删除逻辑，**但无写回/回滚**（改错即毁其他行） | 有逆但**未做事务**（论文 §5.2.2 Algorithm 10 的三阶段事务性 reload 是正解） |
| 知识库 INDEX.md / docs 追加 | append-only 纪律**在位**（`spec §6`） | ✅ 已有契约 |
| `.artifacts/index.yaml` 写回 | `core-artifact` 有契约，`ref_merge_index` 有冲突保护（"Conflicts are reported but never silently overwritten"） | ✅ 已有契约 |
| 探针代码留在 mixin / 临时 `.tmp` 产物 | **实测**：`E:\PYTHON\CoreSwap\.tmp` 当前 **348 个文件、120.6 MB**；AGENTS.md §八.13 立「临时文件唯一区纪律」但只规定**往哪放**，未规定**何时清、谁负责、是否可回退** | §6.1 emission 类比——产物一旦写出即跨界，只能 compensate。当前**连 compensate 都没定** |
| `.dsh` 之外的 `versions/1.20.1/java` 等被安装/生成的运行环境 | AGENTS.md §四记载 runtime 目录、world 状态等 | 明确界外，且已有「world 身份」惨案（#161） |

**缺口**：框架有**一套文本级回退纪律**（很扎实），但**没有「副作用清单」这个概念**——即：哪些副作用在界内（有逆、可回退）、哪些在界外（无逆、只能补偿或声明）、界外副作用的**补偿动作**是什么。后果可实证：知识库 **#161（最高价值·错误优先）** 记录「跨臂 run3 复用型协议的盘上 world 身份陷阱」——`run_g3_settle.py:91-92` 每臂 `rmtree(world)`、`run_g3_run3.py` 不删 world，**副作用（删/不删 world）无契约、无身份自证**，导致一个已 confirmed 结论建立在「跨世界对比」上，**judge 三源核对也未捕获**（三源核的是证据管理面，不是实验设计面）。这正是「界外副作用未声明」的代价。

**改法（可落地）**：
1. **在 `spec` 新增一节「副作用边界与回退契约」（§1.7 或 §9 前）**，给出框架自身的 `Γ` 清单——**三列强制**：
   | 副作用 | 界内/界外 | 逆或补偿 |
   |---|---|---|
   | 写入 `.investigations/` / `.artifacts/` | 界内 | 逆 = 删除该产物文件（有 owner = 该 subtask） |
   | 追加 `docs/` / `knowledge/` | 界内（append-only） | 逆 = supersedes 双指针（**已有**） |
   | 安装 `~/.dsh/skills/*` | **界外**（宿主共享） | 补偿 = manifest orphan 清理（配合建议 1） |
   | 安装项目 `.dsh/*` | **界外** | 补偿 = 项目级 manifest 对账 |
   | 写 profile `cordis.patch.yml` | **界外** | 补偿 = **先备份后改**（当前无备份——**立即可做的最小修复**） |
   | 生成/删除 runtime world、探针产物、`.tmp` | **界外** | 补偿 = 显式声明「本次运行是否可复用」+ 身份自证行（#161 判据 1） |
2. **最小可执行子项（不依赖新框架节）**：给 `install.ps1` 的 profile patch 改写段加**事务性**——改写前 `Copy-Item $patchPath "$patchPath.bak-<timestamp>"`，失败即 `restore`（对齐论文 §5.2.2 Algorithm 10：`backup ← invalidate_caches(...)` → `try ... catch → restore_caches(backup)`）。这是 3 行改动，消除「改坏 profile 无回滚」的现实风险。
3. **`core-artifact` 技能加一行**：产物落盘时若在界外（非 `.investigations/`/`.artifacts/`），MUST 在产物内标注「界外副作用：<位置> | 补偿：<动作或 idk>」。

**收益**：把「哪些东西能干净撤回」从**隐含假设**变成**逐条显式声明**。这是论文 §6.1 最直接可移植的内容——**「界外」不是失败，是必须声明的正常状态**（论文原话：an operation on it acts as `idΓ` and is therefore neither tracked nor reverted）。同时给 #161 类事故一个**结构性**（而非判据式）的防御面。

**成本/风险**：写清单是一次性成本（约半天），维护成本低。风险：清单可能腐烂（写了不更新）——缓解：把它挂在 `install.ps1`/`scan` 类脚本的**输出**里（副作用发生时打印其契约行），使其「被使用」而非「被阅读」。**不建议**追求论文级的自动逆追踪（见建议 6）。

---

### 建议 3：交接/口径的「声明式 entry + 字段级 diff」——把 #156 类口径失传做成机械可判

**论文机制**：§5.2.1 Reconciliation（p.65）——"the loader dispatches on which of the entry's fields changed and applies the least disruptive operation for each"，并按字段分类：`id,url → rebuild`、`isolate → reassign`、`intercept → in-place`、`config → handed to the component, which ... typically by diffing it against the previous one and reloading only on a material change`、`disabled → unload/reload`。§5.2.1 另明确 "the binding runs in both directions: the loader responds to a change in an entry's fields by adjusting the fiber, and a component that revises its own configuration or disables itself has the change written back to its entry"（**双向**）。Theorem 80（p.65）："the quiescent state is a function of the final configuration alone"——**与路径无关**。

**框架现状（一手核实）**：交接的唯一权威载体是 `NEXT_SESSION.md`（AGENTS.md §三.4），格式为**自由 Markdown 叙述**。相关纪律已很丰富但全在**判据层**：
- `spec §4.5` 第五条（v0.21 §16.3）：「交接文档/台账中的『机制方向/待查假设』类结论 MUST 廉价独立验证 ≤1 轮后才可作前提」；
- `core-plan` 技能「交接结论验证（Phase 0 前置）」三步骤（区分验证状态 / 廉价独立验证 / 结果落盘）；
- AGENTS.md STEP 1 的「交接结论验证纪律」+ 切 Session 三态建议。

**已实证的缺口**：知识库 **#156**（260917-01）——「复测口径『使能开关失传』」：NEXT_SESSION 复测口径只转录本块**新增**变量（`-Pdomainbatch=1`），**既有总开关（`-PlightRust`）在交接转录中失传**，两臂实为另一执行形态，整轮读数无效。其根因被明确写成：**"不是参数映射错（#8 形态），而是复测口径条目清单不完整：交接者默认『没变的开关不用写』，复测者默认『口径里没列的就是不需要』"**。判据被立为（#156 判据 1）："任何复测/重跑口径 MUST 三件齐备：全使能开关清单 + 每臂行为化形态自证行 + SELFCERT 硬门 mismatch 即整臂 VOID"。

**缺口**：这仍然是**判据（该写什么）+ 纪律（该核什么）**，不是**结构（写不全就机械不通）**。而论文 §5.2.1 给出的是结构：**entry 是一组具名字段，diff 按字段名做**——「未列 = 默认」的歧义在**类型层面不可能出现**，因为规格是**完整记录**（Theorem 80：quiescent state 是 final configuration 的函数，不依赖路径）。#156 的表述「交接者默认没变不用写 / 复测者默认没列不需要」正是**无字段 schema 的必然产物**。

**改法（可落地）**：为复测/实验类交接定义**强 schema 的 entry**（对齐 Definition 81 的六字段形态），落盘为 `.investigations/<课题>/rerun-entry.yaml`：
```yaml
id: G17c-L3                       # 稳定标识 = 对账键（对应 entry.id）
executor:                          # 执行体身份（#156 的引擎身份 + #161 的数据身份，双身份齐备）
  dll_sha256: <hash>
  switches:                        # 全使能开关清单——**枚举、非叙述**；缺项 = schema 违约
    - {name: "-PlightRust",         value: "1",  role: "光照接管总开关"}
    - {name: "-Pdomainbatch",       value: "1",  role: "本块新增"}
  world_identity:                  # #161 判据 1 要求的盘上 world 归属自证
    origin_arm: G17b-L60
    generated_at: <ISO8601>
prev: <上一轮 entry 的 id 或 artifact path>   # 对应 entry 的"双向绑定"
expected_selfcert: ["[SELFCERT]", "[GATE]", "lightInit_ok=1"]
```
配套**机械门**（新增 `dsh/scripts/audit_rerun_entry.mjs` 或并入 selfcheck）：
1. `switches` 必须**非空且含上一轮 entry 的全部 name**（缺项 → fail-closed，正是 #156 的病）；
2. `prev` 指向的 entry 必须存在且 `id` 可解析（#137 的属主工具自核，此处机械完成）；
3. `expected_selfcert` 非空（#118/#156 的行为形态自证前置）；
4. 与上一轮 entry 做**字段级 diff**（完全复用论文 §5.2.1 的 dispatch）：`switches` 变化 → 必须显式标 `change_reason`；未变字段不重复叙述（消除 #156 的"没变不用写"歧义）。

**收益**：直接消解三个已实证错误——#156（口径条目失传 → schema 必填，缺项机械 fail）、#137（外部锚未核 → `prev`/`dll_sha256` 由门自核）、#161（world 身份失传 → `world_identity` 成必填字段）。并顺带解决 AGENTS.md §三.5「日期漂移三犯」的同构问题（日期作为**字段**由 `Get-Date` 生成，而非叙述里手填）。

**成本/风险**：schema + 一个 ~80 行 node 门，成本小。风险：① 给所有交接加 schema 会**过重**——建议**只覆盖复测/实验类**（正是 #156/#161 的高发区），普通分析交接仍用 Markdown；② 门可能被绕过（不生成 entry 就直接跑）——缓解：门挂在采集脚本入口（对齐论文 §5.2.2 的做法：loader 是唯一入口），或按 #160 的教训「裁决必须非零退出断链」。

---

## 2. 「需验证」档

### 建议 4：把四强制触发点做成「reactive coeffect 式」可机械判定的声明 + 自动调度

**论文机制**：§3.2.2（p.18）——coeffect specification `𝔇Σ ≔ Set(K)`（Definition 21）；satisfaction predicate `σ ⊧ d ≔ ∀k∈d. k∈dom(σ)`（式 22，**可判定**，因 `dom(σ)` 有限）；`notify_d(σ,σ') ≔ activating / deactivating / neutral`（Definition 22，式 24）。关键句（p.18）："Since all mutations to σ pass through effect functions (whose inverses recover the previous domain), changes to satisfaction are detectable at each effect boundary. **This is the algebraic basis of reactivity: the effect system guarantees that every coeffect change is observed.**" §5.1.2 Algorithm 3（p.61）是 `notify` 的实现，按 `key ∈ fiber.inject` 过滤后 `refresh(fiber)`；并由 `refresh` 的**幂等性**"renders a neutral change harmless"（p.60）。

**框架现状（一手核实）**：
- 四强制触发点定义为 `spec §4.5` 的四张表（judge / scout / fan-out / knowledge 落盘），级别 MUST/SHOULD 分层；
- 落地形态 = AGENTS.md 的子角色介入点表 + `core-plan` 技能里**模板化的预置段**（轻量模板第 57-69 行、重量模板第 88-101 行，逐项列 `judge 预置 / fan-out 预置 / 知识库更新 / 子角色介入点`）；
- 执行强制链依赖 `todo_write` 建计划 + `complete_step` 签核（AGENTS.md「执行强制链」段）。

**关键判断（诚实标注）**：**触发点的「条件」目前是自然语言 + AI 自觉，不是可判定谓词。** `core-plan` 模板让 AI 在 Phase 0 **手填**这些字段——这已经比纯纪律好，但**填什么由 AI 决定**，所以：
- 模型可以（且已发生）预置一个**空/含糊**的 judge 项而形式上「预置了」；
- `spec §4.5` 的 MUST 条件（如「判定树分叉 ≥2 个互斥候选」）是**语义判断**，AI 可以在「这看起来可收敛」的自我说服下不触发——**AGENTS.md 自己记载了此反模式**：「-288 未闭合课题 B3 (b) 子候选主会话自推……**即使每个看起来「可收敛/简单」**」。

**可对照的知识库反模式**：#112（判据未预登记——读法须写死）、#154（预登记数值判据的分支必须覆盖实数轴全域或显式声明灰区）、#159（追加预登记判据的「事后逐臂展开」合法性三条件——因为预登记**天然锚定预期形态**，实测分裂无法穷举）、#157 判据 3。这一族错误**全部是「预登记设计」问题**，而 #159 的教训尤其刺中要害：**"预登记判据的分支枚举天然锚定『预期行为形态』，实测形态分裂无法在预登记时穷举"**。

**因此我的判断是：不能、也不应该把「谁该在什么条件下介入」完全机械化。** 论文的 `notify_d` 之所以可判定，是因为 `σ ⊧ d` 建立在**有限、可枚举的 `dom(σ)`** 上（式 22 明说 "This predicate is decidable (since dom(σ) is finite)"）。而框架的触发条件（「机制未明」「互斥候选」「结论性 docs」）**不是有限集上的成员判定，是语义判断**——强行机械化会退化成「关键词匹配」，并制造 #159 式的**新盲区**（预登记穷举不到的分支 → 静默展开）。

**但有一条**是真正可机械化的：**「触发后有没有留下产物」是有限集上的可判定谓词**。这是论文 reactive coeffect 精神的正解——**不要判定「该不该触发」，判定「触发了没有」**。

**改法（建议先做小规模验证）**：
1. **把四触发点从「预置表格」升级为「证据谓词 + 机械门」**（`notify` 的类比：不判 satisfaction，只判 transition 是否被观察）：
   | 触发点 | 可判定谓词（有限集成员判定） |
   |---|---|
   | scout | Phase 1 存在 `.investigations/<课题>/管线地图*.md` 或 `任务.md` 标注 `scout: skipped(reason)` |
   | fan-out | 若 `candidates/` 目录存在 → 必有 ≥2 个 `.bN`；若无 → `架构计划.md` 必须有 `fan-out: not-triggered(reason)` 行 |
   | judge | 每个 `status:` 从 draft→candidate 的**变更点**必须有对应 `judge-review-*.md` |
   | knowledge | 每个含结论的 `.artifacts/**/verdict-*.md` 必须有对应 `docs/` 或 `knowledge/` diff（AGENTS.md 已要求，但**无门**） |
2. **门的形式**：`dsh/scripts/audit_trigger_coverage.mjs`，扫 `.investigations/<课题>/` 目录结构 + frontmatter，输出 `MISSING / DECLARED-SKIP / OK` 三态；`DECLARED-SKIP` **必须带 reason 字符串**（非空）——这就是「neutral 也要被观察」的落地。
3. **fail-closed**：`MISSING` 非零退出（对齐 #160 教训：**"裁决的价值在执行不在打印"**，VOID/GATE 类裁决 MUST 以进程退出码表达）。

**为什么归「需验证」而非「立即可做」**：谓词的可判定性依赖**目录/frontmatter 约定**，而当前 `.investigations/` 结构**并非严格统一**（本次任务本身写入的就是一个自由命名的 `b2-ref-framework.md`，无 frontmatter 之外的强 schema）。**必须先做一次真实课题的回填审计**（用历史 3-5 个已结课题跑门，看误报率）再决定是否写入 spec —— 否则会造出「门绿了但课题没做对」的假安全（对齐 `selfcheck` 注释里那个 2026-09-09 教训）。

**收益**：把「AI 有没有自觉预置」变成「**产物里有没有对应的东西**」——一个**事后可核**、**不依赖自我说服**的判据。且 `DECLARED-SKIP(reason)` 保留了语义判断的空间（论文里 neutral 也产生一次分类），避免 #159 式静默展开。

**成本/风险**：门 ~120 行 node。风险：① **误报**——把合法的「本课题无需 fan-out」判为 MISSING（缓解：`DECLARED-SKIP` 是合法出口）；② **形式主义**——AI 学会写空 reason 骗过门（缓解：reason 长度 + 关键词下限，但这是军备竞赛——**这正说明不能纯机械化，需要 judge 抽查**）；③ 与现有 `core-plan` 模板可能双写——需明确单一事实源。

---

### 建议 5：subagent 交接的「强类型契约」——输入契约 / 输出契约 / 依赖前置

**论文机制**：§3.3.1 Definition 28（p.21）——unified context `Γ∞ ≔ μΓ. Γ × (Γ→Γ) × Σ`，"Every interaction between a component and its environment passes through this single entity"；Definition 29（p.21）——key 携带 `(𝒱_k, 𝒜_k)`，值类型 + **操作集**；Definition 30（p.22）——context-mediated iterators `ℑ^𝒜_Σ(S,P)`，"Membership in this class is the formal content of mediating every interaction through the context"。§3.2.1 Definition 19（p.17）——coeffect context 是**依赖偏函数** `Σ ≔ (k:K) ⇀ 𝒱_k`，扩展/限制带**前置条件**（"a dependency cannot be provided twice nor revoked if absent ... A violated precondition is signaled as an error and produces no transition"）。

**框架现状（一手核实）**：
- `spec §4.4`「subagent 写码强制自检清单」——五条 checkbox（类型宽度 / move 语义 / 异常路径 / 对拍点 / 自检声明）；
- `spec §4.3`「命令委托模式」——**这是全框架最接近「强类型交接」的已有设计**：命令模板 = `命令 + 参数 + 输出落盘路径`（`.investigations/<任务>/cmd-output/<NNN>.<ext>`），且规定「主会话执行时**不解读**；原始输出**必须落盘**」——**契约明确**（输入 = 命令模板；输出 = 指定路径的文件；依赖前置 = 落盘路径存在）；
- `AGENTS.md` §九——命令委托契约 + subagent 写码交付自检 + judge 审查基线；
- `core-worker` / `core-judge` / `core-plan` 的技能正文（**未一手读**，本文件不据其断言）。

**缺口（对照转抄家族错误）**：主会话与 subagent 之间、以及 subagent 与 subagent 之间的**语义性**交接（不是命令委托），载体是**自由文本 prompt + 产物文件引用**。已实证的失败模式：
- **#90 转抄漂移**（机制措辞在转抄中失真）；
- **#94 简报字段名失真**（字段名在简报中走样）；
- **#137 外部锚未核**（引用的 commit/sha/产物路径未经属主工具自核——「引用不存在的对象 = 幻觉签名」）；
- **#156 口径失传**（交接条目清单不完整，本 file 建议 3 已处理）；
- **#162（最高价值·错误优先）**——「汇总交叉定性 MUST 回原始日志抽样」：**"主会话机械交叉『inputDiff23=46/46 = packed↔blocks9 ABI 切换』被 verdict worker 10 行日志抽样一票推翻"**。这条极关键：**数字可以机械继承，定性不行**——「汇总交叉产物 = 数字 + 定性，数字可复算继承，定性 MUST 抽样回原始日志一手验证后才可进 verdict」。

**缺口本质**：框架对**命令委托**（机械面）有强契约，对**语义交接**（自然语言面）**没有契约**。#162 给出的判据其实已经是**契约的雏形**：区分「可机械继承的数字」与「必须一手复核的定性」。这与论文 Definition 29 的 `(𝒱_k, 𝒜_k)` 同构——**光有值（数字）不够，还要有「这个值上允许什么操作」**（能不能直接继承？还是必须重新验证？）。

**改法（建议形态，需先小规模验证）**：
1. **给交接产物加「证据等级」字段**（落盘为 frontmatter，非叙述）：
   ```yaml
   claims:
     - text: "inputDiff23 = 46/46"
       kind: numeric            # numeric | qualitative | anchor
       source: ".investigations/<课题>/cmd-output/017.txt:23-45"
       inheritable: true        # 数字且 source 可复算 → 可直接继承
     - text: "46/46 = packed↔blocks9 ABI 切换"
       kind: qualitative        # #162：定性 MUST 回原始日志抽样
       source: <idk 或 抽样记录路径>
       inheritable: false       # 未抽样前不得进入 verdict
     - text: "verdict 见 commit a1f9336"
       kind: anchor             # #137：MUST 用属主工具自核
       source: "git cat-file -t a1f9336 → commit"
       verified: true
   ```
2. **接收侧机械检查**（`core-worker`/`core-judge` 的操作手册新增一步）：prompt 里传入的 `claims` 中，`kind: qualitative && inheritable: true` → **拒收**（fail-closed）；`kind: anchor && !verified` → **拒收**。这正是 #137 + #162 判据 ① 的机械形态。
3. **依赖前置**（论文 §3.2.1 的前置条件语义）：交接声明 `requires:`（如 `requires: [".artifacts/xxx/index-entry.yaml"]`），落盘时检查文件存在——缺失即失败**且不产生 transition**（对齐论文："A violated precondition is signaled as an error and produces no transition"）。

**为什么归「需验证」**：① 我**未一手读** `core-worker`/`core-judge` 正文，无法确认是否已有类似字段；② 加 frontmatter schema 会**显著增加每一次交接的书写成本**，必须先用 2-3 次真实交接试跑，测量「多写这些字段的边际成本 vs 拦下的错误数」——#162 本身说明这类问题的**实际杀伤力大**，但**发生频率未量化**；③ 存在与 `core-artifact` 现有 schema 重复的风险（需先做一次字段去重设计）。

**收益**：把 #90/#94/#137/#162 这**四类转抄家族问题**从「判据 + 抽查」升级为「**结构上不可传**」——定性声明在缺抽样记录时**物理上无法进入下游**。这是论文 unified context「一切经其中介」最贴近本框架的落地：**不是统一上下文类型，而是统一「声明的证据等级」**。

**成本/风险**：schema 设计 + 两个技能正文改动 + 可能的落盘工具。风险：**过度工程**——若实际错误频率低，这套 schema 会成为纯负担（对齐 `spec §6` 记录价值门的精神：**只记「若再遇到不想重新想一遍」的东西**）。**因此必须先测量频率再决定是否铺开**。

---

## 3. 「不建议做」档

### 建议 6（不建议）：把 RE-Framework 改造成论文意义上的「revertible effect runtime」

**论文机制**：§3.1 + §5.1.1——`ctx.effect` 作为唯一原语，"coeffect provision, component instantiation, and every other context-mutating operation reduces to a ctx.effect call"，自动追踪 + LIFO 逆复合（Algorithm 1）。§5.2.2 HMR（p.67-69）——三阶段（模块分类 / stale-entry 检测 / **事务性 reload**），Algorithm 8 的 accept/decline 不动点 + Algorithm 10 的 `backup → try → catch → restore`。

**为什么不建议**：
1. **论文自己划了边界，且 RE-Framework 的大部分关键副作用恰在界外**。§6.1 System Boundary（p.70）：界内要求**独占修改 + 可恢复**。而框架的产出面向 `E:\PYTHON\CoreSwap` 工作区文件、`~/.dsh` 宿主目录、以及**人类拍板（confirmed 授予）**——最后一项在**任何**形式化里都不可能自动化（`spec §1.1`：confirmed 只有用户亲自拍板）。强制把框架塞进 `Γ` 会导致大量 `idΓ`，形式化**空转**。
2. **论文的核心机制依赖「运行时能拦截一切 mutation」**，而框架的「运行时」是 **LLM 的 token 流 + Markdown 文件**——没有 interception point。§5.1.1 明说 `ctx.effect` 覆盖一切是因为"coeffect provision, component instantiation, and every other context-mutating operation **reduces to** a ctx.effect call"——这个 reduction 需要语言/库级的入口控制，本框架**没有也不可能有**（它不是一个 runtime）。
3. **成本收益严重倒挂**：论文 §5.3 自己承认 "**measuring the abstraction's overhead and its effect on developer productivity against a baseline remains future work**"（p.69）——连论文的实装都还没有量化收益，让一个方法论框架去追它的形式化更无依据。
4. **本框架已有的文本级回退纪律覆盖面其实很好**（`.vN`/`.bN` + supersedes 双指针 + append-only + merge_index 冲突保护），缺的是**界外界面的显式声明**（建议 2 处理），不是**自动逆追踪**。

**但论文有两点值得**独立**吸收（不构成第 7/8 条独立建议，并入上文）**：
- **§5.2.2 Algorithm 10 的事务性 reload**（backup → try → catch → restore）→ 已合入**建议 2 的最小可执行子项**（profile patch 改写前备份）。
- **Theorem 80 的「quiescent state 是 final configuration 的函数」（p.65）**（与路径无关）→ 这是**建议 3** 的理论依据：交接 entry 应当只记录**终态规格**，而不是**抵达路径**（后者正是 #156「只写本块新增变量」的病根——路径式记录必然失传）。

---

## 4. 元理论三定理 ↔ 框架保证（追问方向的回答）

> 用户问题 5：progress / confluence / preservation 分别对应框架里的什么保证？框架缺哪个的**可检验形式**？
> **诚实标注**：以下为**我的类比**（`not_in_paper`），论文未做此对应；且我**略读**了 §4.3 的证明，只读了三节的结论段。
> 论文的原文（§4.3 各节，p.43/49/51）—— Preservation 保存类型/状态合法性；Progress 不停在无主状态；Confluence 并发分支汇聚一致。

| 论文定理 | 框架对应物 | 现有可检验形式 | 缺口 |
|---|---|---|---|
| **Preservation** | 置信度状态机（`spec §1.1`）：draft→candidate→confirmed 不越权 | ✅ **有**——`core-judge` 只出意见不改 status；AI 绝不写 confirmed（`spec §8` 禁止行为明列） | 基本闭合 |
| **Progress** | Phase 0→3 不得停在无主状态 | ⚠️ **部分**——`todo_write` + `complete_step` 签核（AGENTS.md 执行强制链）；C-gate halted escalation（Anchorlaw v0.15）规定「同一验收判据 3 次未满足 → halt + 人类裁决」 | **halt 之后的「谁接手」无机械判据**。C-gate 保证会 halt，但 halt 态本身是「无主」（等人类），而框架**没有**「halt 态必须显式落盘 + 标注 waiting-on-human」的产物要求 → 与建议 4 的 `DECLARED-SKIP(reason)` 同构解 |
| **Confluence** | fan-out 多候选**最终必须汇聚到一个裁决** | ⚠️ **弱**——`core-fanout` 规定「全部完成后 → Judge 对比所有 .bN → 用户拍板 → 优胜者复制为活跃」（`core-fanout` 技能第 36-40 行）；`spec §7` 规定 `.bN` 淘汰后仍保留 | **无机械保证 `N` 个候选都被对比过**。若 fan-out 派出 3 个 worker、只有 2 个返回，`core-fanout` 的流程描述（"全部完成后"）**没有对应的门**——这正是论文 Confluence 要排除的情况（`spec §7` 只说"证据同时支持多假设 → 不强选，标 candidate 等更多证据"，但这**不是** confluence 的检验形式）。**这是框架三定理中唯一缺「可检验形式」的**。 |

**可检验形式的建议（并入建议 4 的门）**：fan-out 的 `candidates/` 目录下每个 `.bN` 必须有一个 `verdict:` 字段指向裁决记录，或一个 `superseded_by:` / `rejected_reason:`——即 **每个候选都必须被"处置过"**（不是被删除，对齐 `core-version`「淘汰后仍留审计追溯」）。这样 confluence 从「流程描述」变成「目录可枚举判定」：`orphan_candidate = {bN | 无 verdict 且无 rejected_reason}`，非空即 fail。**这是我建议 4 的一个具体实例，成本极低（~15 行），但我不单独列为一条建议，因为它属于建议 4 的谓词集。**

---

## 5. 不确定项（@anchor.idk 形态的诚实声明）

- `@anchor.idk("E:\\PYTHON\\RE-Framework\\dsh\\skills\\core-judge|core-worker|core-knowledge 正文是否已含 claims/entry 类结构化字段——本 session 未一手读，建议 1/3/5 的落地需先核这三份正文")`
- `@anchor.idk("#162 类『定性转抄』错误的实际发生频率——未量化；建议 5 的 schema 铺开与否取决于该频率，需 2-3 次真实交接试跑测量")`
- `@anchor.idk("建议 4 的谓词对历史课题的误报率——未回填测试；须用 ≥3 个已结课题跑 audit_trigger_coverage 后再决定是否写入 spec")`
- `@anchor.idk("Anchorlaw protocol-v0.21.md 原文——本文件对 §15.4/§16.3 的引用经 RE-Framework spec §3 转述，未直读协议正文；若建议需改 Anchorlaw 侧条款，须另做一手核对")`
- `@anchor.idk("论文 §4 演算（§4.2 三条规则）与 §4.4 Extensions 全文——仅读目录与 §4.3 结论段；若存在与本框架「混合任务分流」直接对应的 extension，本文件未覆盖")`
- 说明：本次任务为**只读 + 草稿**，未执行任何写框架文件、未改任何 status、未执行安装/同步命令（唯一的写操作是本文件与 `.investigations/paper-2608-25512/` 目录，均在 CoreSwap 工作区内）。
