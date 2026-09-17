```yaml
id: paper-2608-25512:b1-coreswap
block: 260917-05
date_anchor: "2026-09-17 19:55 (Get-Date 实核)"
status: draft              # 草稿；不自我授予 candidate/confirmed
worker: subagent (core.worker)
lens: coreswap
paper: "A Programming Paradigm for Spatiotemporal Composability (Yifan Shi, Wei Zhang, Tianyi Cui; PKU + DeepSeek-AI) — 全文 .tmp/paper-2608.25512.txt(92页)"
scope: 论文机制 → CoreSwap 现状对照 → 具体可执行改进建议（仅草稿，不改任何 status、不写 knowledge/ 或 docs/）
reading_discipline: |
  一手读过（read 工具，非摘要）：论文 1-6 页导论/贡献、§2、§3.1（Definition 1-18 + Theorem 4/5/7/10/11/13/15/16）、
  §3.2（Definition 19-27）、§3.3.1/3.3.2（Definition 28-33，含 Algorithm 6）、§3.3.3-ish（Definition 30 迭代器）、
  §3.4（Definition 40-46 + Theorem 43/45/47）、§5.2.1（Definition 81 + 对账 dispatch 清单 + Algorithm 7）、
  §5.2.2（Algorithm 8/9 + Phase 3 起首）、§5.1.4、§6.1/§6.2/§6.3 起首、§6.5、§6.6、§6.7；
  CoreSwap 侧一手：AGENTS.md、knowledge/INDEX.md（全）、knowledge/discovered/workflow-patterns.md #46/#47/#100/#102/#110/#111/#112/#138/#142/#158-161、
  NEXT_SESSION.md、ServerLightingProviderMixin.java（789 行全读）、LightDomainBatch.java（全读）、
  java-core/src/main/java/wg/bench/CppBridge.java（:96-200/:440-560 等）、worldgen-core/src/light/mod.rs（:1-355）、
  versions/1.20.1/java/build.gradle（-P→-D 映射全表）、docs/12-lighting.md（全读）、docs/07-block-pipeline.md（形态审计节 + 章节索引）、
  .artifacts/pbeta-260917-05/verdict-260917-05.md（全读）。
unverified: |
  明确标注「未核实」项见各条 ⑤ 与文末「诚实声明」节；凡未一手读过的 file:line 一律以知识库条目号转述并标注转述来源。
```

# b1-coreswap：论文机制 → CoreSwap 改进建议草稿（260917-05）

> **本文件性质**：对照分析草稿。**不改任何 status、不执行命令、不写 knowledge/ 或 docs/**（那属主会话 + 知识库 subagent + judge 流程）。
> 论文页码/小节定位均以 `.tmp/paper-2608.25512.txt` 的 `===== PAGE n =====` 标记为准（读的是纯文本页标记，非 PDF 页脚）。

## 0. 总览

论文的两个机制（revertible effects / reactive coeffects）在本项目的**最有价值落点不是"照抄元框架"**，而是三处**结构性缺口**：

1. CoreSwap 的"接管"是**有开有关、无一等回退契约**的——开关是 sysprop 读一次，副作用（写回的 chunk、票据、缓存、盘上 world）没有成对登记（→ 建议 B1，价值最高）；
2. CoreSwap 的阶段间依赖（light 依赖 features 完成 + 3×3 邻域到位）写死在**代码检查 + 注释**里，没有可机械对账的**声明面**（→ 建议 B2）；
3. `-P`→`-D` 映射表与运行时消费点是**两套手工维护的事实源**，漏一行静默失效已三犯（→ 建议 B3，与 B1 是同一缺口的两个面）。

**一句话**：论文对本项目最有用的一条 = 「**移除/回退组件时,MUST 有 runtime 持有的逆操作；副作用不登记逆操作 = 不可回退**」——它把 CoreSwap 已有的"自证/噪声锚/身份门"纪律升级为**成对契约**（每一次接管写入，都要有一份机械可执行的撤销/核对记录），直接命中 #110/#102/#161 三族残留态问题的共同根因。

---

## B1（价值最高 · 立即可做）阶段接管的「副作用成对登记」契约——把 revertible effects 的"逆操作"落到接管面清单

### ① 论文机制
- §3.1.1 Definition 2（PAGE 10）：**effect context** `∂Γ ≔ Γ × (Γ→Γ)`——第二个投影 𝜑 是**累加器 = 迄今所有 effect 的逆的复合**，"the function that recovers the context to its initial state"；初始态 `(γ0, idΓ)`。
- §3.1.1 Definition 3 + Theorem 7（PAGE 10-11）：`trackΓ(f,g) = (γ,φ) ↦ (f(γ), φ∘g)`；**recover 读的状态只有 𝜑(γ)**，`𝜑(γ) = γ0` 称 **soundness invariant**。
- §3.1.2（PAGE 12，动机段）：track/recover 两个 limitation 之一 = "`recoverΓ` is **all-or-nothing**: it cannot selectively undo one effect while retaining others"→ 故升级为 effect function `𝔈∗Γ ≔ (e: Γ→Γ×(Γ→Γ)) × 见证 (g(δ)=γ)`（Definition 8）。
- §6.1 System Boundary（PAGE 70）：**边界判据**——"A location lies inside when the system is able to modify it **exclusively** and to **restore the state before that modification** … A location lies outside when **either ability fails**, so an operation on it acts as **idΓ** and is therefore **neither tracked nor reverted**."并给出 **acquisition / emission 两阶段**：acquisition（获得访问并**在边界内安装一条记录**，如 open→descriptor、malloc→block）是**可回退 effect**；emission（把数据推出边界）"acts as idΓ"——**跨边界的发射天然不可回退**，只能靠 withhold（output commit）或 compensation（补偿动作，按 LIFO 复合但元理论不覆盖）。

### ② 本项目现状（证据）
- 接管 = **单一 sysprop 读入的静态门**，无逆操作概念：
  - `ServerLightingProviderMixin.java:77-79`：`LIGHT_RUST = System.getProperty("coreswap.light.rust") != null`、`LIGHT_DOMAIN = ...("coreswap.light.domainbatch") != null`——**进程级 final，读一次定生死**。
  - `CppBridge.java:102-107 resolveStageMask()`：`prop==null||isEmpty ⇒ 0b011`；`:115/:165/:185` 三处句柄 `setFlags(handle, resolveStageMask())`。**这就是"接管面开关"的全部状态**。
- 接管写入的**产物无成对撤销/核对记录**（四类）：
  1. **写回 chunk 光照**：`ServerLightingProviderMixin.java:716-724`（legacy）与 `:543-551`（domain）——`enqueueSectionData` × 48 + `chunk.setLightOn(true)` + `wgReleaseLightTicket(chunkPos)`。**一旦写入，无任何"撤销/回滚"路径**；注释自述"复刻原 light() 尾部语义"——复刻的是**正向**语义，逆操作不存在。
  2. **票据状态**：`:550/:724` `wgReleaseLightTicket`——**释放即不可逆**（这正是 #161 陷阱的机理面：状态机推进一步就不可回）。
  3. **缓存**：`worldgen-core/src/light/mod.rs` 的 scratch 已改为 per-call 局部（`:189-190`、`:319`），**这一侧实际是干净的**；但 #102「缓存三面核对」（谁写/谁读/clear 波及面）与 #102 补充案例（260911-01「只写不读」写入侧门控）证明项目**已有**这类残留态家族问题。
  4. **盘上 world / 运行环境**：**#161（260917-04，最高价值·错误优先）**——"每臂 rmtree + 复用脚本不删 world + 自证门无 world 身份项 ⇒ 「同 world 第 N 轮」前提静默失效"，且"**已 confirmed 的结论 + judge 三源核对也未捕获**"。这是**同一缺口的实锤实例**：接管/实验动作在盘上留下了副作用，而**没有任何逆操作记录**，导致"这轮跑在哪个 world 上"这一前提无法机械判定。
- 已有**半成品**纪律（说明缺口是"缺机械契约"而非"缺意识"）：
  - #118 自证硬门（含 `ServerLightingProviderMixin.java:28-29` 自述的负证据面 `WG_DOMAIN_INLINE`）——核的是**执行体身份**（lightInit/hook/dll sha）；
  - #161 判据 3 指出："**自证门必须含数据身份项，不止执行体身份**"——**该判据已写出，但未机械化**（仍是纪律文字，不是代码/脚本里的强制项）。
- #110（260910-05）证明同一缺口的反向形态：**"路径不可达（空集）"的断言需要"证明三件套" + 复活出口穷尽 + 前置条件登记**——即项目已经在为"副作用的可达性"付审查成本，只是没有把它变成声明面。

### ③ 缺口
**CoreSwap 的接管面有"接线清单"（哪些开关 → 哪些路径），但没有"副作用清单"（这条路径一旦执行，在共享环境上改了哪些位置、每个位置的逆操作或等价核对方式是什么）。** 论文的判据直接给出两个可机械化的划分：
- **acquisition 侧（边界内、可回退）**：`enqueueSectionData` 写入的 section 数据、`setLightOn` 标志、票据状态、进程内缓存——**都应当是"有逆"的**，现状是"无逆且无登记"。
- **emission 侧（跨边界、不可回退）**：盘上 `run/world`（chunk 落盘）、`chunkStorage` 的持久化、`versions/.../data` 下探针 dump 产物——**按论文应显式声明为 "acts as idΓ, not reverted"**，并改用 **withhold（output commit）或 compensation（补偿）** 处置。#161 正是"未声明为 emission、却当成可继承状态"的事故。

### ④ 具体改法（可落地到文件/流程级）
1. **新增接管面「副作用登记表」**（建议落 `versions/1.20.1/docs/` 或 `worldgen-core/` 旁的清单文件，形如 `takeover-effects.yaml`；位置请主会话定，本草案不写 docs/）——**每行一个副作用点**，字段照论文 §6.1 二分：
   - `id` / `owner`（如 `light.legacy` / `light.domain` / `cs.fill` / `cs.writeback`）
   - `gate`（哪个 sysprop/mask 位；**必须给 file:line**，如 `CppBridge.java:102-107`）
   - `class: acquisition | emission`
   - `inverse`（acquisition）：逆操作是什么（**可执行描述**，不是散文），或"等价核对方式"（若逆操作在 vanilla 语义下不可行，论文 §6.1 允许用 compensation）
   - `verification`（该行如何被机械验证：正/负成对自证行、hash 门、world 身份行）
2. **把 #161 判据 3「数据身份项」机械化**：在任何复用盘上 world / 复用运行台的脚本入口，**强制打印一行 world 归属自证**（world 目录创建时间戳 + 上一臂标签 + region 文件指纹，三者取一，见 #161 判据 1）+ **在脚本里做硬门禁**（mismatch ⇒ 臂判 VOID 非零退出，对齐 #160「VOID/GATE 裁决必须非零退出断链」）。**这一条是立即可做且收益最直接的**——#161 已 confirmed 是真实事故，且修复方向已在条目里写好，只是没有落地到脚本模板。
3. **对 emission 类副作用补"withhold / compensation"字段与动作**：例如 `run/world` 落盘 → 在探针协议里默认 `rmtree`（withhold 的极端形式）或**显式声明"本轮继承上一臂 world"并记录继承链**（二者必居其一，#161 的根因正是"既没删也没声明"）。
4. **`enqueueSectionData` / `setLightOn` / `releaseLightTicket` 三点的逆语义确认**：论文要求逆操作"在应用的那个状态上"成立（Definition 8 见证 `g(δ)=γ`）。对本项目这意味着：**切换 `-PlightRust` 从 on→off 后，已经跑过的 chunk 光照状态与没跑过的是否可区分/可复原？** 这是可一轮探针回答的**明确问题**（见 ⑤ 成本），当前项目**从未问过这个问题**（`docs/12-lighting.md` 的 round-trip G3 判据量的是"重载收敛"，不是"关掉接管后的状态复原"）。

### ⑤ 预期收益与成本/风险
- **收益**：① 直接把 #110/#102/#161 三族"残留态"问题从**事后审查**转为**声明期机械检查**；② 为"开关翻转回退"提供**可判定的验收判据**（现状只有"关掉后行为等于 vanilla"的行为等价，没有"关掉后共享环境复原"的状态等价）；③ 副产品——接管面清单本身就是 §6.6「依赖类型与版本化」的键空间基础（见 B5）。
- **成本**：登记表 = 一次性梳理（估 1 个工作块内可完成首版：接管面已知 = light(legacy/domain/blockabi/oldcollect) × 3 维 + cs.fill × 3 维 + cs.writeback(bulk/perblock/skipair) + stageMask 三态）；world 身份硬门 = 脚本级改动（低成本）。
- **风险**：**最大风险是"清单腐化"**——手工维护的清单会像 `-P`→`-D` 映射一样漂移（#8/#19/#47/#56 家族已四犯）。**缓解**：清单条目**必须带 file:line 且由脚本核对该行仍存在**（对齐 #118「自证行 mismatch 即 VOID」的硬门思路），否则本建议会变成第 N 个静默失效的手工表。**若不做这一步自动核对，本建议价值减半，应降为「需验证」。**
- **不确定性**：`enqueueSectionData` 的逆语义是否在 vanilla 语义下**根本不存在**（论文 §6.1 的 emission 情形）——**未核实**，需一轮源码核对（`LightStorage.enqueueSectionData` 再传播语义本项目已标 `@anchor.idk`，见 `docs/12-lighting.md:17`）。若确为 emission，则该项按 §6.1 走 compensation（等价核对）而非逆操作，**结论形态会变**。

---

## B2（高价值 · 立即可做）阶段接管的「coeffect 依赖声明 + 自动降级」——把写死在代码里的前置检查升级为声明面

### ① 论文机制
- §3.2.1 Definition 19（PAGE 17）：coeffect context = 依赖偏函数 `Σ ≔ (k:K) ⇀ 𝒱k`；`get`/`set` 带**前置条件**（`set` 要求 `k∉dom(σ)`，违反 = **error 且不产生转移**）。
- §3.2.2 Definition 21/22（PAGE 18）：**specification** `𝔇Σ ≔ Set(K)`；**notify** 把任意转移分为 `activating`（σ⊭d ∧ σ′⊧d）/ `deactivating`（σ⊧d ∧ σ′⊭d）/ `neutral`——"An **activating** transition triggers the execution of the component's effects … a **deactivating** transition triggers **recovery by applying the accumulator**."
- §3.2.2（PAGE 18）判据："a component **activates only at a state satisfying its specification**, so it **never reads a binding that is absent**, and every change to the context is classified against that specification, so a **loss of satisfaction is detected where it happens and drives a deactivation**."
- §5.1.4 Algorithm 6（PAGE 64）：`resolve(ctx,key)` 沿 fiber 链上溯——`key ∈ fiber.inject` 且未 commit ⇒ `throw INACTIVE_ACCESS`；到 root 无声明 ⇒ `UNDECLARED_ACCESS`。**"Reading the view rather than the store"** 是 Theorem 70 的基础。

### ② 本项目现状（证据）
- 前置条件**以命令式检查散落在代码里**，不是声明：
  - `ServerLightingProviderMixin.java:290-293`（`wgLightNeighbor`）：`nc.getStatus()==null || !nc.getStatus().isAtLeast(ChunkStatus.FEATURES)` ⇒ 记 detail 返 null——**"light 依赖邻 chunk 到 FEATURES"这条 coeffect 写在一个 if 里**，注释（`:285`）自述"D4 只需 blocks（FEATURES 起就绪）"。
  - `:228-231`（`wgLightCollectBlocks`）：高度守卫 `center.getBottomY()!=-64 || !wgLightHeightOk(world)` ⇒ false——**"依赖 1.20.1 主世界高度参数"这条 coeffect 也是一个 if**。
  - `:494-503`（`wgLightDomainReady`）：注释自述"**提交前廉价预检**（.b1 §3.3-2/边界语义）：高度参数 + 3×3 邻域全部到 FEATURES"——**依赖清单以自然语言写在注释里，检查以循环写在函数体里**。
- **降级路径存在但分类模糊**（这正是论文 notify 三分要解决的问题）：
  - `:510-531 wgLightDomainSubmit`：预检不过 ⇒ `return false` ⇒ 调用方 `:760-767` 落到 legacy 内联路径 ⇒ 再不过 ⇒ `:775-777` 不 cancel ⇒ **vanilla**。三级降级 + `WG_DOMAIN_INLINE`（`:94`,`:519`,`:766`）与 `wgLightFallback`（`:162-171`）两组计数——**但"降级"与"未激活"在语义上没有被区分**：预检不过（**specification 从未满足 = 从未激活**，论文的 neutral/永不 activating）与任务期失败（**激活后失去满足 = deactivating**，`:617-628` "降级重放"）**走的是不同的代码路径但共享同一个"回退"叙事**。
  - 这一混淆**有实锤代价**：#158 简记（260917-01）"机制推演候选分解地基 = 内核输入通道清点——**结构性排除法定位循环依赖只能活在回退子路径**"；#110 的"前置条件登记"与 260916-01 的"不可达路径，loud fail"注释（`:618`）都是同一问题的不同侧面。
- **依赖面已被审计过，但结论是"6 行 open 错配"而非"依赖声明缺失"**：`docs/07-block-pipeline.md:1540`（形态审计 C 段）列出 L1-L6 六项 open，其中 **L5+L6 = "3×3 全量重算 ×9 + 无跨 chunk 缓存"**、**L1/L2 = 同步粘线/线程放置**——**这些都是"与外部供给形态的依赖"**，但在审计矩阵里被表述为"错配"，没有升格为可声明的依赖。
- #150（260915-01，最高价值）："**事实串行化会掩盖其下方的并发缺陷**……解除串行化的修复与被掩盖缺陷修复 MUST 同批"——**这正是论文 Theorem 47 的工程对偶**：依赖/串行的隐式性使"独立"无法被证明。

### ③ 缺口
**CoreSwap 有"前置检查"，没有"依赖规范"**。差别是可机械化的三点：(a) 检查失败时**分不清**"从未激活"（前置不满足，论文 neutral）与"激活后失效"（deactivating，须走 recovery）；(b) 依赖清单散在代码/注释里，**无法在立项前枚举**（论文 §6.5 说依赖环"predictable from the dependency declarations alone, so a runtime can **report it when components are loaded**"）；(c) 降级链的每一级**没有对应的"失去满足"语义**，因此降级后的状态归属无法判定——**这与 B1 的残留态缺口是同一个洞的两面**。

### ④ 具体改法
1. **为每个接管面写一份 coeffect 规范**（复用 B1 的登记载体，加一个 `requires:` 段）。以 light 为例，把 `:228-231`/`:290-293`/`:494-503` 三处检查**反向提取**为声明：
   ```yaml
   light.legacy:
     requires:
       - key: world.height_profile
         satisfies: "bottomY==-64 && botSec==-4 && span==24"   # 现 :228-231 + :195-198
       - key: chunks.neighborhood3x3
         satisfies: "∀(dx,dz)∈[0,2]²: neighbor(dx,dz).status ≥ FEATURES"   # 现 :290-293 / :494-503
     on_loss: fallback            # 失去满足时的动作（当前：return false → 下一级）
   light.domain:
     requires: [world.height_profile, chunks.neighborhood3x3]
     on_loss: degrade_to(light.legacy)   # 现 :510-531 + :617-628
   ```
2. **区分两类失败并分开计数**：新增两个计数（或复用现有计数但**改名/分列**）——`PRECONDITION_UNMET`（**从未激活**：预检不过，论文 neutral）vs `DEACTIVATED_MIDFLIGHT`（**激活后失去满足**：任务期失败 `:617-628`，论文 deactivating，须走 recovery 语义）。**当前 `WG_DOMAIN_INLINE` 与 `DEGRADED` 已部分承载这个区分**（`:94` vs `LightDomainBatch.java:27`），**但没有在文档/判据层面声明为两类**——升格为声明即可，成本近零。
3. **立项前置检查：依赖图穷尽**（论文 §6.5 的"load 时报告"）：任何新接管面（如 CP-1 增量化的 5×5 域、未来的跨 chunk LightStorage）**在立项前 MUST 给出 requires 集合**，并机械检查 `P₁∩S₂ = P₂∩S₁ = ⌀`（论文 Theorem 47 的唯一遗留条件，PAGE 30）——即"**我消费的键，不被我的供给者同时消费**"。这一条对 CP-1 尤其关键：L5/L6 的"无跨 chunk 缓存"本质是**同一 chunk 被多个中心重复消费**，用论文语言就是**违反了 disjointness**。

### ⑤ 预期收益与成本/风险
- **收益**：① 降级链的每一级有明确语义归属（B1 的判据基础）；② 让"接管面依赖"从代码内部可被**跨面审阅**（论文 §6.5 的环检测前置）；③**直接服务 CP-1 立项**——CP-1 的核心风险（L5/L6 冗余 ×9）在 coeffect 语言下就变成一条可检查的 disjointness 条件，而不是"感觉冗余"。
- **成本**：light 面首版规范 = 半天级（三处检查已定位）；全接管面（density/carver/surface/feature/light）随 B1 一并梳理。
- **风险**：**过度形式化风险**——本项目是性能工程不是框架工程，若把每条 `if` 都升格为 YAML 规范，收益不抵维护成本。**建议边界**：只对**跨组件/跨阶段**的依赖做声明（light⟵features、surface⟵fill、写回⟵section 状态），**面内私有检查不动**。
- **不确定性**：论文的 notify 机制建立在"**每次 context 变化都对 spec 分类**"上，而 CoreSwap 的关键变化源（chunk status 推进、邻 chunk 完成）**不是通过一个统一 context 中介的**，而是 MC 自己的状态机。因此**只能借用"依赖声明 + 分类"的工程形态，无法借用其元理论保证**（preservation/progress/confluence 在 CoreSwap 无对应物）。此为**结构性不可迁移**，必须诚实声明——本建议是**类比落地，不是形式化移植**。

---

## B3（高价值 · 立即可做）「开关生效自证」= 声明式配置 ↔ 运行时状态的对账（configuration reconciliation 的工程化移植）

### ① 论文机制
- §5.2.1（PAGE 64-65）："the orchestrator specifies the desired composition as a **persistent data structure**, and the loader translates changes to this specification into the corresponding imperative fiber operations."
- **Definition 81 Entry**（PAGE 65）：`id`（**stable identifier, used as the reconciliation key**）/ `url` / `isolate` / `intercept` / `config` / `disabled`——**"An entry can serve as a faithful specification because **what supports a fiber is exactly what an entry records**."**
- **Reconciliation 的 per-field dispatch**（PAGE 66）："the loader **dispatches on which of the entry's fields changed** and applies the **least disruptive operation** for each"——`id,url`→rebuild；`isolate`→reassign realms；`intercept`→in-place；`config`→handed to component（typically **diffing against the previous one**）；`disabled`→unload/reload。
- Theorem 80（PAGE 65）：**quiescent state 只是最终 configuration 的函数**——"whatever instantiations and retirements the loader performs **on the way, and in whatever order**, the system quiesces where a load of the final configuration from scratch would have left it."

### ② 本项目现状（证据）
- **事实源双份且靠人手工对齐**：
  - **声明侧** = `versions/1.20.1/java/build.gradle` 的 `benchVmArgs` 闭包（`:68` 起）：约 **100+ 行 `if (project.findProperty('X') != null) run.vmArg "-DY=..."`** 手工映射；`:150-151` 注释自述："⚠️ 必须写在 benchVmArgs 闭包内……且**缺行 = gradle -P 静默不生效（#8/#19 家族，本工程已三犯）**"。
  - **消费侧** = 各处的 `System.getProperty(...)`（如 `ServerLightingProviderMixin.java:77-79/82/104-105/115/121/128` 共 10 个门控、`CppBridge.java:103 resolveStageMask()`、`WgCompat.flag(...)` `CppBridge.java:621/642`）。
  - **两份表之间没有任何机械核对**。
- **该缺口的已付出代价（知识库明载）**：
  - #8（build-tooling）："gradle -P→-D 手工映射清单遗漏**静默不生效**"——**已三犯**（`:151` 注释自述）；
  - #19（build-tooling）："**-PblockProbe.full 点分名静默不生效**，映射实为 blockProbeFull，#8 家族**三犯**"；
  - #47（build-tooling）："gradle -P→-D 映射的**作用域**坑——`run` 只在 `loom.runs` 闭包内可见"，#8/#19 家族**第五形态**；
  - #56（build-tooling）："**UP-TO-DATE 假绿**"家族（另有 #1）；
  - #32（build-tooling）："**生产/开发口径参数缺失**……`-Dcpp.blockRegister` 门控 dev 经 gradle -P→-D 映射自动带入、**生产裸 java 无携带**"——#28 家族；
  - #28（build-tooling）："**冒烟口径 ≠ 存档口径**……带错参数集 handle=0 全 -1"。
- **已有"单侧自证"但无"双侧对账"**：
  - #81："「**A=B 恒等」不证 env/覆盖分支生效**——两臂可能同走旧路径，闭环 = **行为化分支日志 + A/B hash 双向变化**"；
  - **#37/#81/#53 家族的三查升级（260905-10）**："env 门控默认值当公理……强制三查：**消费点直读 / NEXT_SESSION 声明对照 / 行为化哈希哨兵**"；
  - `CppBridge.java:117-119` 打印 `[CppBridge] init ... stageMask=` **且** `stageMask` 值来自 `CppWorldgen.getFlags(handle)`（**JNI 回读**，见 #110 判据 3 原文）——**这就是双侧对账的雏形，但只覆盖 stageMask 一个键**。
- **本项目独有的额外复杂度**（论文没有）：开关有**三个通道**（gradle `-P` → JVM `-D` sysprop → native `WG_*` env），且 #22/#25/#32/#152 家族记录"**daemon 复用吞掉客户端 env**"、"sysprop 名 ≠ env 名两个命名域"——**对账必须跨三通道**。

### ③ 缺口
**没有一处机械核对能回答"我这次跑，声明的开关集合里哪些真的生效了"。** 现状是**逐项人工自证**（#37/#81/#53 家族的"行为化哨兵"要**每个开关单独设计一条日志**），且**遗漏是静默的**（#8/#19/#47/#56 四家族反复）。论文的 reconciliation 给出的正是这个形状：**声明侧是 persistent record，运行时侧是实际状态，对账 = 逐字段 diff**。

### ④ 具体改法（这是本草案中**最容易落地且收益可量化**的一条）
1. **建立"门控注册表"**（单一事实源）：每个门控一行 = `{prop 名, -D 名, 消费点 file:line, 默认值, 生效自证行格式}`。**声明侧（build.gradle 映射行）与消费侧（getProperty 点）由脚本双向核对**：
   - 正向：`build.gradle` 里每个 `run.vmArg "-Dxxx"` 必须有 registry 条目 + 消费点；
   - 反向：每处 `System.getProperty("xxx")` / `WgCompat.flag("xxx")` 必须有 registry 条目 + 映射行（或**显式标注"生产裸 java 带入，不经 gradle"**，如 #32 的 `cpp.blockRegister`）。
   - **判据形态**（对齐 #118 硬门）：**孤儿条目 ⇒ 非零退出**（不是警告），对齐 #160"VOID/GATE 裁决必须非零退出断链"。
2. **启动期一行机械对账（取代逐项人工哨兵）**：在每个 handle 初始化点（`CppBridge.java:117-119` 已有雏形）**扩展为一次性打印"本次进程读到的全部 coreswap.* 门控及其取值"**（遍历 `System.getProperties()` 前缀 `coreswap.` + `cpp.` + `wg.` + native env 通道），并**与 registry 的"本臂预期集合" diff**。差异 ⇒ 打 `[GATE-MISMATCH]` 且**臂判 VOID**。
   - **为什么这是关键**：#118 的教训是"脚本打印了 executed vs want 却不判定，空跑静默通过并外泄无效结论"——**打印不是门禁，判定才是**。
3. **对齐论文 Theorem 80 的"顺序无关"性质**：论文保证"无论 loader 中途如何实例化/退役，静止态只由最终配置决定"。本项目的对偶是：**开关集合的最终生效状态应当与"哪些开关是先设的、哪些后设的"无关**。可检验形态 = **同一开关集合、两种设置顺序（含缺省 + 显式）⇒ 启动期对账行逐字节相同**。这是一条**零成本、可自动化**的新判据（现项目只有"单开关翻转"的 A/B，没有"设置顺序无关性"检验）。
4. **与 B1 合并载体**：registry 与 B1 的副作用登记表**共用同一份 YAML 的不同段**（`gates:` / `effects:`），避免再造第二个会腐化的手工表。

### ⑤ 预期收益与成本/风险
- **收益**：① **直接消灭 #8/#19/#47/#56 家族的静默失效**（四家族、至少六形态）；② 把"每个开关单独设计行为化哨兵"（`#37/#81/#53` 三查纪律的当前形态）降级为"注册表自动覆盖 + 少数需要语义哨兵的项手工补"；③ 生产口径（#32，裸 java 无 gradle）的缺参问题**自动暴露**（registry 需显式标注通道）。
- **成本**：registry 首版 = 逐条抄 `build.gradle:79-219` 约 100 行（机械）+ 核对消费点（可脚本化）；对账行 = `CppBridge`/mixin 侧十几行改动。**估 1 个工作块内可完成首版并自证。**
- **风险**：① **registry 与代码不同步**（与 B1 同风险，同缓解：file:line 存在性由脚本核）；② **对账行的打印量**——必须一次性（进程级），**不得进热路径**（对齐 260911-05「诊断门控 flag 在初始化器内唯一置位」#11 与"诊断代码绝不能放热路径每点执行"铁律）；③ 三个通道（gradle/sysprop/env）的对账语义需分清"声明但未送达"（通道断）与"送达但未消费"（消费点改名）——**两类必须分开报**，否则又是一个模糊结论。
- **不确定性**：`WgCompat.flag(...)`（`CppBridge.java:621/642`）的读取语义（是否读 env、默认值来源）**未核实**——`WgCompat` 未一手读。落地前须核（否则 registry 的"默认值"列可能错，重复 #53「默认值当公理」家族）。

---

## B4（中高价值 · 需验证）「观测等价」的判据升级：从"逐位/hash 门"到"操作序列不可区分"（并补并发交错形态）

### ① 论文机制
- §3.3.2 Definition 31（PAGE 23）：**观测等价是"没有任何观察者能区分"**——"A test over 𝓐 is a finite word whose letters are **forward maps and yielded inverses** of the effect functions … Values v, v′ are **indistinguishable** when **every test** over 𝓐 is defined at both or at neither and **yields the same outcomes at both**." `≃k ≔ ≈𝓐k`。
- Lemma 32（PAGE 23）："≈𝓐 is an equivalence that every operation of 𝓐 respects, and it is **the coarsest such relation**"——**证明原理**：要证两值等价，**只需给一个被所有操作尊重、且包含该对的等价关系**。
- Definition 33（PAGE 24）：context 层等价 `σ ≃S σ′ ≔ dom(σ)∩S = dom(σ′)∩S ∧ ∀k∈dom(σ)∩S. σ(k) ≃k σ′(k)`——**按 key 集合 S 切片比较**。
- §3.4.1 Definition 42（PAGE 27）：**independence 要求"每一个变换与每一个变换交换"，forward map 与 yielded inverse 都在内**（`f∘g = g∘f`），且"**neither one's transformations disturb what the other yields**"。Theorem 43（PAGE 28）：**pairwise independent ⇒ 逆操作按任意置换顺序施加仍回到 γ0**。
- §3.4.2 Definition 31 + 讨论（PAGE 29）："≃k is **indistinguishability under the tests the operations of k generate**, so an interface **publishing fewer outcomes admits fewer tests and coarsens the relation**, and **withholding an outcome its callers do not need can carry a key from one side of a division to the other**."（引 scalable commutativity rule）

### ② 本项目现状（证据）
- **等价门已相当强，且项目自己已建"形态阶梯"**：
  - #142（260913-01）："**等价性证据的形态阶梯**——逐 chunk **多重集**判等 → **完整无缺格网格** → **整文件全 64 位 sha256 同一**（最强，二值）"；且给出**口径窄化**：fp 文件是排序件 ⇒ "判据域 = **排序域**"。
  - #47（260905-04）："性能优化 golden 逐位不变对照法——pre 冻结 + post 复跑 + hash 等值"；#115：位置敏感 FNV-1a 指纹跨形态逐 chunk 比对（4140/4140 全等）。
  - #116（260910-07）："结构性迁移的**等价性门 = 产物全量条目级 sha256**"；#138：**按"是否改变产物字节"分档**（V1-strict / V1b）。
- **但形态覆盖面的缺口是明说的**：
  - 12-lighting.md:121："对拍抽样 = **4 chunk/轮**（须含位流节）……**大样本正确性依赖 fallback=0 + 结构性论证，非全量逐位**"；
  - #142 边界："指纹取在**写回之前**，不覆盖写回/并发路径"；
  - 12-lighting.md:94（round2 归因）被 round3 **取代**："round2 减法口径把 Java 段记 ≈0.15ms/chunk，被 round3 分段探针实测证伪——collect 串行 4.5ms/chunk（30×）"→ #147（"减法归因口径作优化排序依据前 MUST 分段直接计时"）。
- **并发/交错形态的等价门**：
  - #107（最高价值）：**"同步接管阻塞单车道调度器"**——"接管类优化必须对齐原实现的**同步/异步形态**，不能只对齐语义正确性"；
  - #129（最高价值·错误优先）：**"判据只有「满足 / 未满足」两种终态"**——"**变更臂自身的同实现对照（`new × new`）不可用未变更臂对照（`old × old`）替代**"；
  - #161："跨臂 run3 复用型协议的盘上 world 身份陷阱"；
  - 12-lighting.md:144-145：G3 首载漂移 7× 的**两个候选 (i) 邻域时机形态 / (ii) UB 面"当前不可分"**。
- **#161 与 B4 的直接关联**：论文 Theorem 43 的"**逆操作任意置换顺序仍回原点**"是本项目**从未检验**的性质——本项目检验的是"**两条执行路径产物相同**"（跨形态 A/B），没有检验"**同一组操作以不同交错顺序执行，产物是否相同**"。而 #161 的事故**恰恰是交错顺序问题**（G17b-L60 → G17b-D60 → G17c-L3 的顺序造成了 world 覆盖）。

### ③ 缺口
**本项目的等价门是"产物态等价"，论文的判据是"操作序列不可区分"。** 二者的差距落在**交错/顺序**这一维：
- 现门：给定路径 P，跑出的产物 hash；比较 P₁ 与 P₂ 的产物。
- 论文门：给定操作集合，**任意交错顺序**的产物是否可区分（Theorem 43），以及**逆操作在"外来 effect 已经移动过的状态"上是否仍撤销自己的那一份**（§3.4.1 开篇："An inverse may be run while later effects are still in place, which is **what removing one component from a running system amounts to**"）。

对 CoreSwap 这意味着两条具体的新判据（当前都没有）：
1. **顺序无关性（置换检验）**：同一组接管动作（如"3 个 chunk 的光照接管"）以不同完成序执行 ⇒ 产物应当等价（或差异必须落在已量化的 run 级噪声带内）。**这正是 23 线程 worldgen 的天然风险面**，而项目当前只做了"跨形态比较"（#107 家族）与"run-to-run 噪声锚"（#111），**没有做过置换检验**。
2. **逆操作在外来状态上的可撤销性**：对应 B1 的第 4 点（切换 `-PlightRust` 后的状态复原）——论文给出的正是这个问题的判据形态（§3.4.1）。

### ④ 具体改法
1. **新增"置换检验"判据类别**（写入项目探针协议，非 knowledge/）：对任一接管面，取 N 个 chunk 的操作集合，**至少跑两种完成序**（自然序 + 强制逆序/随机序），比较：
   - **产物层**：逐 chunk 指纹（#115/#142 阶梯，取在**写回之后**——修补 #142 的"边界 = 指纹取在写回之前"）；
   - **内部态层**：若可观测（如 `LightStorage` 的 section 状态），比较 §3.3.2 Definition 33 式的**按 key 切片**比较（本项目 key = chunk 坐标，S = 本次操作涉及集合——**切片比较比全量比较强**，这是可以直接照搬的形式）。
   - **判据**：不等的**定量分解**必须区分"置换导致"与"run 级非确定"——现项目有噪声锚（#111 分形态/分维度），**置换臂必须复用同锚**（否则重复 #111 的教训："用代理基线会把真实分量判成噪声内 ⇒ 因果指认错"）。
2. **补"等价门与变更同层"的机械检查**：#14 补充案例（260912-01）"等价门必须与被测变更**同层**——`[WG-CONTENT]`（Rust buf 层）对 Java 侧写回变更**结构性不敏感**"。论文的判据给出**判层的形式化理由**：观测等价的粒度由"**该 key 的 operations 集合**"决定（Definition 29/31）——**如果等价门观察的不是该 key 的操作，它就不是该 key 的观测者**。建议在判据文书里**显式写出"本门对应哪个 key 的哪组 operations"**，作为 `#14 补充案例` 的可复用形态。
3. **"发布更少 outcome 使等价关系更粗"的主动利用**（§3.4.2 PAGE 29）：本项目已有实例（`docs/12-lighting.md` 的缺键语义 #46：vanilla 缺 SkyLight=隐式 15 / rust 缺键=flag1 全 0，统一填充制造 1.47M 假差异）。**论文正面表述了这个技巧**：`≃k` 由 k 的 operations 生成的 tests 决定 ⇒ **"发布更少 outcome ⇒ 更粗的等价 ⇒ 更多实现被视作等价"**。可落为一条**判据设计原则**：跨实现比较前，先声明"比较域（published outcomes）"，**未声明的维不进判据域**（#46 的教训正是"未声明的省略语义进入了判据域"）。

### ⑤ 预期收益与成本/风险
- **收益**：① 补上并发/交错形态的等价门（当前**唯一的空档**，且 #161 已证明交错顺序能造成已 confirmed 结论被推翻）；② 为 CP-1（增量化）提供**比"跨形态 hash 同"更强的判据**——增量化的正确性本质是"**交错无关**"，正是 Theorem 43 的形状；③ "判层声明"成本近零却能防 #14 家族复发。
- **成本**：置换检验 = 探针协议级改动（需驱动支持"强制完成序"，**本项目已有先例**：#155（260915-03）"虚拟玩家票据驱动法——自建 ChunkTicketType + addTicketWithLevel 分级 + 固定路径跳位"，说明顺序可控的载具**已经存在**）。
- **风险**：① **置换检验的噪声底可能吞掉信号**——run 级非确定 overworld ~0.0133%（#111 新口径），而交错差异量级未知；**必须先测"同序双跑"作本底**（论文 Lemma 32 的"coarsest equivalence"在工程上的对偶 = 先建立**噪声生成的等价关系**，再看置换差是否超出）。**若本底 ≥ 信号，本建议不可判**，须降级声明（#148 家族："boot Done 载体噪声下限……缺口 <2s 不可判"）；② **over-fitting 风险**：置换检验会引入新的一次性判据，须过"记录价值门"（AGENTS.md 三.2），只保留**可复用**的形态。
- **不确定性**：论文的 Theorem 43 假设"pairwise independent"，而 CoreSwap 的接管面**显然不满足**（光照 3×3 邻域天然耦合、#107 的调度形态耦合）——**因此置换检验预期会 FAIL**，其价值在于**把"耦合在哪一维"定量化**，而非宣称收敛。**这一点必须在立项文里写死**，否则会被误读为"又一个未满足的判据"（C-gate 3 次未满足 ⇒ halt，见 AGENTS.md 验证协议 §6）。

---

## B5（中价值 · 需验证）组件粒度与版本化：接管面划分 vs §6.5/§6.6 的分解原则

### ① 论文机制
- §6.5（PAGE 74）：**依赖环"predictable from the dependency declarations alone, so a runtime can report it when components are loaded"**（与调度无关的死锁 vs 可预测的永不激活）；**分解总是可能的**——"every bidirectional interaction can be factored into independent unidirectional bindings"，代价是组件数**平方级增长**（n 个互相作用的组件 → 集成组件数 quadratic）；缓解手段 = **package bundling / convention-based wiring / scaffold tooling**，"These strategies preserve the formal guarantees of the acyclic model while reducing the authoring burden to something closer to the monolithic case."
- §6.6（PAGE 75）：**key 身份只有 nominal linking，没有 versioned/structural linking**；两个问题 = **Interface drift**（provider 改接口，consumer 仍声明同 key ⇒ "satisfied at the coeffect level (k∈dom(σ)), yet the runtime value no longer conforms"→"type errors, method-not-found failures, or **silent behavioral divergence**"）与 **Key collision**（两个独立 provider 用同名 key 表示无关接口 ⇒ "no compatibility check"）。三种方案：**Key namespacing**（K→K×P）/ **Peer dependencies**（Cordis 当前采用，依赖 semver 惯例 + 单版本解析）/ **Structural compatibility**（结构子类型，语言无关但有界多态下不可判定）。
- §6.7（PAGE 76）：co-design 可让"**dependency cycle 在编译期报告**"、"dependency 按**类型结构**而非 key 身份比较（row types）"。

### ② 本项目现状（证据）
- **接管面粒度 = 阶段级，且已有版本薄壳**（AGENTS.md §〇.1 / §八13a）：`worldgen-core`（跨版本共享引擎）+ `versions/<ver>/rust`（薄壳 cdylib）；数据驱动架构铁律（AGENTS.md 四）要求"版本相关 worldgen 数据尽量从 JSON 加载"。**这正是 §6.5 说的 "package bundling" 形态**——把细粒度组件打成单一可安装单元。
- **但 1.21.6 移植已经实测出 §6.6 的两个问题**：
  - **Interface drift 家族**：`algorithm-fingerprints #26`（最高价值，260912-02）"跨版本**分块容器 storage 写帧**契约差——1.20.1 `readPacket` 走 `readVarInt` 长度前缀 vs 1.21.6 `readFixedLengthLongArray` 定长无前缀；失配症状 = **不是帧格式错而是整段移位** ⇒ `EntryMissingException`"；`build-tooling #53`（E5 复算）"手写 NBT reader 长度字段按「tag 类型 → 载荷长度类型」对表核规范"；#39（"多版本 dll 同名冲突裁决——薄壳包名带版本 + processResources rename 回 worldgen.dll，**版本隔离在构建产物层**"）。
  - **Key collision 家族**：`compiler-idioms` 的"**跨层 id 域错位 raw block id vs state id**（2026-09-01）"、#21（"模拟 mod 方块注册探针 raw id 必须取 `Registries.BLOCK.size()`——**vanilla 域 id 是 blocks.json 既有属主**"）、#22（"跨语言 id 域注册时**同域化优于运行时映射表**——映射表=第二真相源"）、`build-tooling #27 家族第三形态`（260910-03）"existence-only marker 跨版本陈旧缓存——**1.21.6 dll 读 1.20.1 blocks.json，982/1003 id 域错位 → 大规模错块 → 重力/流体级联 → O(n²) + 16GB OOM 冻结**"——**这是 Key collision 的教科书级实例，且后果是灾难性的**。
  - **#39 补充**（260908-08）"多版本 dll 同名冲突裁决——**薄壳包名带版本**（worldgen1216.dll）+ processResources rename 回 worldgen.dll，**版本隔离在构建产物层**"——**项目已经在用"Key namespacing"的构建层变体**（论文方案一）。
- **粒度问题的具体形态（从形态审计提取）**：`docs/07-block-pipeline.md:1535` 覆盖面声明 = "生产 **3+1 段**（A NOISE 含 A' Beardifier / B SURFACE / D 写回 + C 光照）+ executor **横切层**"；而 `stageMask` 只有 **2 位**（bit0=SKIP_CARVER bit1=SKIP_FEATURES，`CppBridge.java:99-101`）——**接管开关的粒度（按 bit）≠ 审计面的粒度（按段）≠ 论文的组件粒度（按 effect/coeffect 组合）**。**三套粒度并存且无映射**。

### ③ 缺口
**"接管面"作为组件单位缺乏定义**：现在的划分是历史堆积（`stageMask` 两位 + 若干独立 sysprop + executor 横切），既不是论文意义上的"effect+coeffect 组合"，也不与审计矩阵的分段对齐。§6.6 诊断的两类问题（interface drift / key collision）在本项目**都已实际发生且造成过严重事故**（#27 家族 16GB OOM），说明**这不是理论关切而是已付代价的缺口**。

### ④ 具体改法
1. **建立"接管面 = 组件"的定义表**（复用 B1/B2/B3 的同一载体）：
   ```
   component: light.domain
     provides: [chunk.light.block, chunk.light.sky]        # 论文的 key（K）
     requires: [...]                                        # B2 段
     gate:     coreswap.light.domainbatch                   # B3 段（声明侧键）
     effects:  [...]                                        # B1 段（副作用/逆）
     audit_segment: C                                       # 与 07 篇矩阵分段对齐
     stage_mask_bit: (none)                                 # 与 stageMask 对齐（若无则显式写 none）
   ```
   **关键**：把"`stageMask` 两位 ↔ 审计 3+1 段 ↔ 独立 sysprop"三套粒度**显式映射**，`none` 也必须写（防"没写 = 没这回事"的静默，对齐 #110"复活出口穷尽"）。
2. **对 key 采用论文方案一（Key namespacing）的工程变体**：现有 `K`（"chunk 光照"/"方块 id 域"/"帧格式"）应升级为 **`K × P`（P = 接口定义方：`1.20.1` / `1.21.6` / `core`）**——**项目已有此实践**（薄壳包名带版本 `worldgen1216.dll`、`WgCompat`、`:99-101` 的 1.20.1/1.21.6 分版缝注释），只是**没有形式化为 key 空间**。形式化后的直接收益：`#27 家族第三形态`（1.21.6 dll 读 1.20.1 blocks.json）在 key 空间下是**类型不匹配**，可在**加载期**拒绝而非在**生成期**爆 OOM。
3. **粒度建议（回答任务书方向 5）**：
   - **不建议**把接管面切得更细（如把 light 的 collect/native/writeback 切成三个组件）——论文 §6.5 明确说分解代价是平方级，且本项目已有"组件轻量但配置/认知成本高"的约束；**论文自己推荐用 bundling 缓解**。
   - **建议**按"**是否共享可变状态**"切分：共享态是耦合的根源（light 的 `LightStorage`/scratch、写回的 section、`pending_cross_writes`），**当前切分已经沿着这个线**（light 独立成面、写回独立成面）。真正缺的是**把"共享哪个 key"写出来**（B2 的 requires/provides），而不是切得更碎。
   - **1.21.6 侧的薄壳策略经 #27/#39/#26 三次事故检验，方向正确**（版本隔离在构建产物层 + 分版缝收口），**唯一缺口是"数据缓存也要带 P"**——#27 家族第三形态的根因正是"existence-only marker 跨版本陈旧缓存"，与代码键的 namespacing **不对称**。

### ⑤ 预期收益与成本/风险
- **收益**：① 把 #27 家族（16GB OOM 级事故）的防线从"生成期爆炸"提前到"加载期拒绝"；② 为 B1/B2/B3 提供共同的**组件单位**；③ 回答"接管面怎么切"这个反复出现的问题（形态审计、CP 池、粒度讨论都隐含它）。
- **成本**：定义表 = 与 B1/B2/B3 同批（**边际成本低，这是把它排在 B5 的原因**）；key 空间形式化 = 需要一次 1.20.1/1.21.6 双版交叉核对（#94 判据："任务简报字段名/分类词不进一手源核对不可用"）。
- **风险**：① 论文 §6.6 自己说"Designing a **unified** dependency model … remains an **open problem**"——**不要试图在本项目里解决这个开放问题**，只用它的**诊断面**（interface drift / key collision 两类命名）来分类已有事故；② 形式化过度会重蹈"手工表腐化"（同 B1/B3）。
- **不确定性**：`WgCompat`（`CppBridge.java:621/642`）与分版缝（#26 提及"写端 MUST 收敛到单一分版缝函数"）的**现状边界未一手核实**——需读 `java-core/` 分版缝实现 + 1.21.6 侧对应文件后才能给出准确的 key 空间方案。

---

## B6（中价值 · 不建议做）照搬论文的元理论/形式化保证到 CoreSwap

### ① 论文机制
§4 全套：component/fiber 演算（§4.1-4.2 orchestration/lifecycle/confinement）+ 元理论（§4.3.1 Preservation / §4.3.2 Temporal Composability / §4.3.3 Spatial Composability / §4.3.4 Progress / §4.3.5 Confluence）+ §4.4 Extensions。

### ② 本项目现状
- 项目**已有自己的等价判据体系**（#47/#115/#116/#138/#142/#129），且**反复证明形式化/静态断言不足**：#42（"静态机制断言未实测当公理，一轮 dump 证伪"）、#25（"静态调研结论失真两例"）、#149（"静态审计 MUST 附立项前预验证探针清单、禁引绝对秒数"）。
- 论文的元理论**前提在 CoreSwap 不成立**：Theorem 47 要求 `P₁∩S₂ = P₂∩S₁ = ⌀` 且"每个 key commutative"（PAGE 30）；Theorem 43 要求 **pairwise independence**（PAGE 27）。CoreSwap 的接管面（光照 3×3 邻域、共享 section 写、executor 横切）**结构性违反**这两条——不是实现问题，是**问题域不同**（MC worldgen 是强耦合的生成管线，不是插件生态）。

### ③ 缺口
无缺口——**是"不该做"**。论文 §2.3 自己承认"classical effect and coeffect systems are **static instruments**"，本文的贡献是**把它们 reify 成运行时机制**；而 CoreSwap 需要的是**运行时机制的形状（登记/对账/分类）**，不是**静态类型系统的保证**。

### ④ 具体改法
**不做**。若一定要用，只用 §3.1/§3.2/§5.2 的**工程形状**（B1/B2/B3 已提取），**不引 §4 的定理作为本项目结论的依据**。

### ⑤ 预期收益与成本/风险
- **收益**：无直接收益；**避免损失**——防止后续会话拿"论文证明了 progress/confluence"当公理去推断 CoreSwap 的行为（这正是 #42/#25/#95 家族的形态：**外部权威当公理**）。
- **成本/风险**：若做，成本极高（形式化建模 MC 管线），且**产出的保证在前提不成立时无意义**——属于"看起来很强但不可用"的资产，且会污染知识库权重（违反"记录价值门"）。

---

## 三档分类汇总

| 档位 | 条目 | 一句话 |
|---|---|---|
| **立即可做** | **B1** 副作用成对登记契约（含 #161 数据身份硬门） | 论文的核心判据；#161 已是 confirmed 事故，修法已写好未落地 |
| **立即可做** | **B2** coeffect 依赖声明 + 降级分类 | 把散落的 if 检查升格为声明；直接服务 CP-1 立项 |
| **立即可做** | **B3** 开关生效机械对账（registry + 启动期 diff） | 消灭 #8/#19/#47/#56 四家族静默失效；`CppBridge.java:117-119` 已有雏形 |
| **需验证** | **B4** 置换检验 / 判层声明 | 补并发交错形态空档；**但置换差异预期会 FAIL（前提不成立），价值在定量化** |
| **需验证** | **B5** 组件粒度与 key 空间（K→K×P） | 边际成本低（与 B1-B3 同载体）；需先核 `WgCompat`/分版缝现状 |
| **不建议做** | **B6** 照搬 §4 元理论 | 前提（independence/disjointness）在 MC 生成管线结构性不成立 |

**若只能做一条**：**B1**（并**立即**落地 #161 判据 3 的 world 身份硬门——那是零成本、已 confirmed 的事故修复）。

---

## 诚实声明（未核实 / 一手边界 / idk）

### 一手读过（可据以立论）
- 论文：PAGE 1-8（导论/贡献/§2 全部）、PAGE 9-16（§3.1 全部含 Definition 1-18 与 Theorem 4/5/7/10/11/13/15/16）、PAGE 16-20（§3.2 全部含 Definition 19-27）、PAGE 21-24（§3.3.1/3.3.2 含 Definition 28-33）、PAGE 26-30（§3.4 含 Definition 40-46 + Theorem 43/45/47）、PAGE 64-69（§5.1.4/§5.2.1/§5.2.2 含 Algorithm 6/7/8/9）、PAGE 70-76（§6.1/§6.2/§6.3 起首/§6.5/§6.6/§6.7）。**未读**：§4 全部（元理论证明，任务书许可略读）——故 B6 对 §4 的引用**仅来自目录标题 + §3.4 末尾的指引句**，**其定理编号与内容未一手核对**。
- CoreSwap：`ServerLightingProviderMixin.java`（789 行全读）、`LightDomainBatch.java`（全读）、`CppBridge.java`（:96-200 / :440-560 读；**其余部分仅 grep 行号，未逐行读**）、`worldgen-core/src/light/mod.rs`（:1-355 读；**355 行以后未读**）、`versions/1.20.1/java/build.gradle`（:68-253 读）、`docs/12-lighting.md`（全读）、`docs/07-block-pipeline.md`（:1531-1617 读 + 章节索引 grep）、`knowledge/INDEX.md`（全读）、`workflow-patterns.md`（#110/#111/#112 附近 + #161 全读）、`NEXT_SESSION.md`（全读）、`.artifacts/pbeta-260917-05/verdict-260917-05.md`（全读）、`AGENTS.md`（工作区指令全文）。

### 未核实（依据二手转述，标注来源）
1. **`WgCompat.flag(...)` 的读取语义与默认值**（`CppBridge.java:621/642` 调用点一手；**类本体未读**）——B1/B3/B5 的"默认值"列依赖它。→ **未核实**。
2. **`java-core` 与 `versions/1.20.1/java` 的关系**——我按 AGENTS.md「共享核抽取（260912-01 D3）」理解为共享核在 `java-core/`，但**未一手核对构建路径如何接线**（`versions/1.20.1/java/build.gradle` 是否 sourceSets 引用 `java-core/` **未读**）。→ **未核实**，影响 B3 落地位置。
3. **分版缝实现**（#26 提到的"写端 MUST 收敛到单一分版缝函数"）——**未一手读**，B5 的 key 空间方案据此为粗粒度。→ **未核实**。
4. **`enqueueSectionData` / `LightStorage` 的再传播语义**——项目已自标 `@anchor.idk`（`docs/12-lighting.md:17`）；B1 的"逆语义是否根本不存在"**未核实**。→ 直接引用项目自己的 idk 状态。
5. **知识库条目 #8/#19/#47/#56/#27/#26/#39/#21/#22/#115/#116/#129/#142/#147/#148/#149/#150/#155/#158/#160 的内容**——我一手读了 **#110/#111/#112/#161**；其余**来自 `knowledge/INDEX.md` 的分类摘要行**（INDEX 是索引不是原文）。→ **转述来源 = INDEX.md 摘要行**，条目标题与摘要可信，**条目正文细节未核**（尤其 #27 家族的具体数字 982/1003、16GB OOM 来自 INDEX 摘要）。
6. **`docs/06-surface-rules.md` / `docs/11-features-stage.md` / 07 篇的 carver/feature 段**——**未读**。故 B2 的"阶段间依赖"举例**只覆盖 light←features**，**surface/CARVERS/FEATURES 之间的依赖声明缺口未做一手核对**（任务书方向 2 提到"worldgen→carvers→surface→features→light→装饰"，我只核实了其中 light 一侧）。→ **覆盖面缺口，如实声明**。
7. **23 线程 worldgen 的具体池实现与完成序**——我引用了 #107/#122/#153 的 INDEX 摘要与 `docs/07-block-pipeline.md:1538` 的"自有池 N=logical/2-2"，**未一手读线程池实现**。→ B4 的置换检验可行性据此为推断。

### @anchor.idk 措辞（不确定处，供主会话裁决）
- `@anchor.idk("B1 的『逆操作』在 CoreSwap 光照接管面上是否根本不存在——enqueueSectionData/setLightOn/releaseLightTicket 三点的可撤销性未核实；项目自身对 enqueueSectionData 再传播语义已标 idk（docs/12-lighting.md:17），本草案未能推进该边界", source="static: ServerLightingProviderMixin.java:543-551/716-724 调用点 + docs/12-lighting.md:15-17 转述；无 trace/probe")`
- `@anchor.idk("B4 置换检验的噪声本底是否低于交错差异——overworld run 级非确定 ~0.0133%（#111 新口径，转述自 INDEX 摘要）已接近典型信号量级，置换差量级完全未知；未做任何采集", source="none: 纯推演，无 trace/probe")`
- `@anchor.idk("B3 门控注册表的完整性前提——build.gradle 映射行与 System.getProperty 消费点的『全量』清点未做；本草案只抽样核对了 light 面（mixin 10 个门控）与 stageMask，其余 ~90 行映射行未逐个核对消费点", source="static: build.gradle:68-253 抽样 + ServerLightingProviderMixin.java 门控点穷举（该类内）；未做全仓 grep 穷尽")`
- `@anchor.idk("论文 §4 元理论与 CoreSwap 的不可迁移性判断——§4 全文未读（任务书许可略读），B6 的依据仅为目录标题 + §3.4 末尾指引句 + 定理 43/47 的前提在 CoreSwap 明显不成立；若 §4.4 Extensions 含弱化假设的版本，B6 的『不建议做』可能过强", source="static: 论文 PAGE 2-3 目录 + PAGE 27-30 前提；§4 正文未读")`

### 方法边界（本草案**没有**做的事）
- **未执行任何命令**（无 shell 采集、无探针、无构建）——全部结论建立在静态阅读之上，**验证分层 = Degraded（静态审查）**，对齐 Anchorlaw §9。
- **未改任何 status**、**未写 knowledge/ 或 docs/**、**未派 subagent**（本文件自身即 subagent 产物，但其产物按任务书要求只落 `.investigations/`，不进知识库）。
- **未引用任何绝对秒数作为立项量化依据**（对齐 `docs/07-block-pipeline.md:1560` 的 C-1 条件"实测前禁止引用绝对秒数 0.69s/1.3s 做立项量化依据"与 #149"禁引绝对秒数"）。
- 所有百分比/hash 数字**均为转述**（来自 INDEX 摘要或 docs 正文），**未一手复算**——若主会话要采用，MUST 按 §9.7 三要素重新声明口径（#127："口径换代后旧基线必须同口径复算才可继续作锚"）。
