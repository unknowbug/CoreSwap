---
id: paper-2608-25512:b3-anchorlaw
block: 260917-05
status: draft          # 推荐 candidate 需 judge 审后由用户裁定；本 worker 不授予任何 status
worker: subagent
date: 260917-05
lens: anchorlaw       # 协议源 = E:\PYTHON\Anchorlaw（只读，基线 v0.21）
source_paper: 《A Programming Paradigm for Spatiotemporal Composability》(Yifan Shi, Wei Zhang, Tianyi Cui; PKU + DeepSeek-AI)
paper_text: .tmp/paper-2608.25512.txt（全文纯文本，含 ===== PAGE n ===== 页标记；本稿页码均为该文本页）
protocol_baseline: spec/protocol-v0.21.md（E:\PYTHON\Anchorlaw，只读）
mode: 提案（Anchorlaw 为只读协议源——本文件一切「MUST/SHOULD」措辞均为**建议条款**，非现行协议文本）
---

# 论文 × Anchorlaw v0.21 —— 可执行改进建议（B3 候选，anchorlaw 透镜）

## 0. 阅读与核验范围（一手 vs 未核实，先声明）

**一手读过（read 工具逐段读，页码可核）**
- 论文：p.1 摘要 / p.4 §1.1 两维定义 / p.9-11 §3.1.1 Effect Context（Def.1-7：twisted composition、$\partial\Gamma$、track/recover、Thm.7 soundness invariant）/ p.12 §3.1.2 Effect Functions（Def.8 $\mathfrak{E}_\Gamma$、$\mathfrak{E}^*_\Gamma$ witness $g(\delta)=\gamma$；per-state inverse）/ p.15 §3.1.3 Effect Iterators（Def.17-18、Thm.16 逆序回退）/ p.17-18 §3.2.1-3.2.2（Def.19-22 $\Sigma$、$\sigma\models d$、notify 三分 activating/deactivating/neutral）/ p.19-20 §3.2.3（Def.23 in-place vs derived realization、隔离/拦截）/ p.23-25 §3.3.2（Def.31 test over $\mathcal{A}$、$\simeq_k$、Def.33 $\simeq_S$、Def.34 沿类型构造子扩展、Def.36 $\mathfrak{E}^S_\Gamma$）/ p.31 Def.48（组件三元组 $(d,p,e)$）/ p.44-45 §4.3.2（Def.65 independence/entangled、Lemma 66-67）/ p.49-50 §4.3.4（Thm.73 Progress：no deadlock + termination，$\prec$ acyclic 为**假设**）/ p.51-54 §4.3.5（Def.74-76、Lemma 77-79、Thm.80 Confluence：quiescent 态 = 静态装配形态）/ p.64-68 §5.2（loader/reconciliation/HMR，按目录定位）/ p.75 §6.6（interface drift / key collision / 三种命名空间化与版本化方案）。
- 协议：§5.1/§5.2/§5.4/§5.5（含 v0.7 Source Artifact Requirement，`:697`）/ §6.1 / §9.1-§9.7（`:801-884`，含 §9.4 evidence saturation `:845-856`、§9.6 `:866`、§9.7 三要素 `:876`）/ §10 / §11（`:902-930` 全表逐行）/ §15.4（`:1266-1415`，含取代链语义、judge 三源基线、五触发点、A/B/C 终止门，C 的 mechanical fallback 3 轮 → halt）/ §16.1（`:1519-1559`，含 v0.13/v0.14 input-contract 语义收敛标准）/ §16.3（7 条，第 7 条 v0.20 host handover validation）。
- 落地：`AGENTS.md` §一（验证协议 8 条）§三（知识库铁律）§八.13/13a（临时区唯一隔离）§九（主会话/subagent 边界）§十/§十一；`versions/1.20.1/docs/10-timewise-archive.md` 260917-01/03/04/05 块（`:3282-3346`）；`.artifacts/pbeta-260917-05/verdict-260917-05.md`（全 71 行）+ `.investigations/pbeta-260917-05/criteria-260917-05.md`（全 35 行）；`knowledge/INDEX.md` 尾部 260917-01～260917-05 五行（`:180-189`，#156-#162 全文）；`knowledge/discovered/workflow-patterns.md` #144/#146 条目（`:2496-2544`）；`anchor-degrade`/`anchor-judge` SKILL.md 全文；`.artifacts/index.yaml` 头部 schema。

**未核实（推测，不作论据）**
- Cordis 实现细节（§5 core library 的 TypeScript 代码形状）——本稿只读机制叙述，未读实现代码。
- §6.1-6.5（系统边界/沙箱/粒度）仅由目录页定位，未逐段读；第 6 条建议涉及 §6.6 已读部分。
- 论文未给出 Anchorlaw（或任何验证协议）为宿主场景的形式化；**本稿所有映射是类比性构造，不是论文的原命题**——论文没有一句话谈验证协议。

---

## 建议 1（立即可做）：把「验证的 temporality」立为协议条款——验证副作用的成对逆

**论文机制（页码）**：p.9-11 §3.1.1（Def.3 `track`、Def.6 `recover`、Thm.7「被回退的效果不移动 recover 的结果」、soundness invariant $\varphi(\gamma)=\gamma_0$）；p.12 §3.1.2（**逆由**应用点**返回而非预先固定**——`Γ → Γ × (Γ→Γ)`，witness 只要求 `g(δ)=γ` 单点成立）；p.15 §3.1.3 Thm.16（**逆序回退**：reverting in reverse order 使每个逆恰好拿到自己应用时产生的状态）；p.19-20 Def.23（**in-place vs derived realization**：就地改并返回非平凡逆 / 保留输入派生新上下文且逆为 identity）。

**协议/落地现状**：协议对「结论」有完善的取代与降级机制，对「验证动作本身留下的副作用」**零契约**——
- §15.4（`:1268-1280`）只管状态的**单向**递进（draft→candidate→confirmed，「No downgrade below the evidence held」）与文本级 supersedes（v0.20 双指针，原结论不改写）；§9 只管**能力降级**的诚实声明；§9.4（`:845`）只管**轮次**的熵上限。三者都是「文本层/计数层」的约束，**没有一处要求验证动作在文件系统层可逆**。
- 落地现状是**三条零散纪律**、各自独立成因、无统一载体：`AGENTS.md` §八.13（临时文件唯一区：一次性脚本 → 仓库根 `.tmp/`；Rust 诊断 bin → `worldgen-core/src/bin-diag/`，不进默认构建）；workflow-patterns **#144**（复跑同 seed/同标签臂 MUST 先归档首轮日志再覆盖）+ **#146**（扩展为**所有臂**——成功/失败/作废一视同仁）。真实事故代价可核：260913-05 A2 首轮日志被复跑同路径覆盖，judge MUST-1 只能以「现象不可独立复核」的**诚实声明**收口（`workflow-patterns.md:2503`、`10-timewise-archive 260913-05`）；260917-03 K2-D5 半截日志归档为 `K2-D5-VOID2-partial.log`（`10-timewise-archive:3306`）；260913-06 失败首跑日志同样灭失（`1.21.6/10-timewise-archive:151`）。另一面：`NEXT_SESSION.md:32` 已明写「.tmp 不入库，用前核在盘；日志落 cmd-output/，同标签复跑先归档」——**纪律存在但住在交接文档里，而非住在协议里**（下轮丢失 = #156 家族重演）。
- **验证动作的副作用清单（从本项目一手记录归纳，这是本建议的核心交付物）**：① 新建探针代码（mixin 打点、`-Pxxx=1` 门控、`run_*.py` 运行台）；② 采集的日志/快照（`.cmd-output/*.log`、result json、snap json）；③ 写入/重建的 `.artifacts/index.yaml`（**注意**：`build-tooling #50` 实证 `merge_index.py` 是「解析 + 重建」，四字段白名单外一切静默丢失——写入本身是**破坏性**副作用）；④ 临时区运行台脚本（`.tmp/**`，不入库，天然易失）；⑤ 被覆盖的同标签日志（#144/#146）；⑥ **被污染的执行体/环境**（#42 家族：沙箱 TEMP 重定向 → 整臂静默 vanilla；`run/world`、`run/config/chunky` 残留；#113 卡死轮 dump 原文被成功复跑覆盖）；⑦ 门控/默认值的改动（#53「清空 env ≠ 恢复默认预期」）。

**缺口**：协议当前把「验证」当作**无副作用**的观察动作，因此①逆操作没有规范载体，②副作用的**存在性**在审查时不可见（judge 三源基线查不了不存在的清单），③纪律靠人记（#144/#146 已证明会漏：`#146` 的存在本身就是 `#144` 覆盖了一次失败臂的补丁）。

**具体改法（提案措辞）**
1. **§9 新增小节「§9.8 Verification Temporality（验证的 temporality，提案）」**：定义验证动作的**副作用二元组**——每个验证动作 MUST 声明 `effect`（它改变了什么：路径 + 变更类别）与 `inverse`（如何回到动作前的状态：归档路径 / 删除 / 换标签 / 还原 env），并区分 Def.23 的两种 realization：**in-place**（就地覆盖日志、重建 index.yaml、改动门控默认值——MUST 附逆）/ **derived**（新增独立命名的产物、新临时目录——逆为 identity，即丢弃）。条款文字建议只要求**逆存在且被登记**，不要求自动执行（机械自动化非协议职责，见 §16.2 边界）。
2. **判定式（机械、可检查）**：一个验证动作合规 ⟺ 对每个 in-place 副作用，存在一条可在 `source` 中寻址的逆（归档件路径 / 换标签后的新名 / env 还原命令）。此式与 §5.5 v0.7 Source Artifact Requirement（`:697`「source 引用的验证记录 MUST 在盘上可寻址」）**同形**——只是把「输出有记录」升级为「副作用的逆有记录」，可直接挂进同一个 scan gate（WARN 级）。
3. **`@anchor.test` 的 source 串加一个可选段（不破坏 v0.5+ 兼容，§13 语法向后兼容）**：建议形如 `source="probe:...; inverse=archived:.investigations/<task>/cmd-output/<arm>-r1.log#<sha8>"`。落地可行性已验证：CoreSwap 已在 source 里写自由文本描述（`AGENTS.md` §一.1 的 `source="probe:block_probe!xxx#NNN"`），扩一个分号段不需要改 stub（§5.5 implementation note 明言 source 为字符串、实现 MAY 校验格式但 MUST 逐字保留）。
4. **技能正文落点**：`anchor-degrade` SKILL.md 第 3-4 步之间插一步「登记副作用的逆」（该技能已管 `uncompilable_functions.yaml` 登记，载体现成）；`anchor-test` SKILL.md 加一条运行前置「同标签已有产物 → 先归档或换标签」（把 #144/#146 从项目纪律升格为技能动作）；`anchor-judge` 审查清单第 4 条（source on-disk artifact）**扩为**「source 的逆是否可寻址」。
5. **落地纪律（可先行，不必等协议）**：`AGENTS.md` §一新增第 9 条「验证副作用与逆」；把 #144/#146/#13 三条从 `workflow-patterns` **升级引用**（正文保留，§一 新增条款指向它们作为实例）。这一步纯宿主侧、零协议风险。

**收益**：①把「验证留下的痕迹」变成审查对象——judge 三源基线从「查快照/查 diff」扩到「查副作用的逆」，能抓出当前漏掉的证据灭失类问题（三起真实灭失已在案）；②#144/#146/#13 三条孤立纪律获得统一载体与成因叙述（Def.23 的 in-place/derived 二分正好是它们的抽象）；③`index.yaml` 重建式写入（build-tooling #50 的静默丢失）在「逆登记」下被迫显式——这是**当前正在发生的**数据丢失风险。
**成本与风险**：①**登记负担**——每个验证动作多写一段 inverse，粗估每轮采集 +1-2 分钟；缓解 = 只对 **in-place** 类强制（derived 类自动合规，占多数）。②**过度形式化风险**：验证动作的逆有时真的不存在（如「被外部工具改写的 world 目录」），若条款写成 MUST 会造成**伪造逆**（比没有更坏，直接违反 §7 行为准则「不假装验证过」）；因此建议措辞为「MUST 声明逆或显式声明不可逆 + 原因」（与 §9.5 诚实声明同构）。③与 §16.2「协议 MUST NOT 要求宿主重构自身」的边界：本条款只要求**登记**，不要求宿主改流程编排——但需在 §11 audit 表登记为「规范条款，实践数据待积累」，避免像 §5.1「每函数必有 anchor」那样长期停在 ⚠️ scoped（§11 `:909`）。

---

## 建议 2（立即可做）：判据的 coeffect 化——显式前置依赖集 + 前置失效触发判据降级

**论文机制（页码）**：p.17-18 Def.19（coeffect context $\Sigma = (k:K) \rightharpoonup \mathcal{V}k$，有限偏函数，**extension/restriction 带前置条件**：`k∉dom(σ)` 才能 bind、`k∈dom(σ)` 才能 revoke，违反前置 = 报错且**不产生转移**）；Def.22 `notify_d(σ,σ')` 三分 activating/deactivating/neutral（按**满足性状态是否被改变**分类，而非按「值是否变了」）；p.18 §3.2.2 结尾「两条被局部判据漏掉的东西」——**撤回必须在它引发的 deactivation 完成后**、**激活期间读到的绑定必须不被移动**；p.23 §3.3.2 关键推论（`:1188-1190`：related states 有相同 domain ⇒ 对 $\sigma\models d$ 与 `notify_d` 的结论一致 ⇒ **reactivity 是 $\Sigma/\simeq$ 的性质**）；p.31 Def.48（组件三元组 $(d,p,e)$：$d$ 声明依赖、$p$ 声明可提供）。

**协议/落地现状**：Anchorlaw 已有三层「前置」但**互不相连**：
- §5.5（`:654-699`）要求 `@pt` 带 source 且 source MUST 在盘上可寻址——**但 source 的失效不被检测**（`regression-record.md` 被删/被 supersede 后，引用它的 `@pt` 仍显示 healthy）。
- §15.4「Acceptance criteria first」（`:1331-1342`）+ 落地 #112（判据预登记必须与采集脚本同批）+ #150（判据读法必须预登记时写死）——判据有了「先于实现」的时序保证，**但没有「依赖什么前提成立」的登记**。
- §15.4 C（`:1399-1415`）的机械回退是**轮次**驱动（同一判据 3 次未满足 → halt）：它检测的是「反复未达成」，检测不到「判据的前提已悄悄失效」——后者往往一次就过，或根本不被触发。
- **真实事故（判据前提失效型，全部一手可核）**：① **M11 seed 错位**烧掉整条「Octave createLegacy 缺口」结论链（`AGENTS.md` 全局铁律「探针/参照数据采集核对铁律」明写 seed 类错误**已三犯**）；② **#161**（`INDEX.md:187`，最高价值）——G17c-L3「同 world 第三 boot」实为加载被 G17b-D60 覆盖重建的 domain world，`prev_after` 与盘上跨世界对比，200-changed 系混杂产物，**judge 三源核对也未捕获**，直接导致 verdict-260917-01 的 run3 层 legacy 证据被 §15.4 取代；③ **#156**（`INDEX.md:180`）——复测口径只列本块新增变量、总开关 `-PlightRust` 失传 → 两臂实为 vanilla 形态，**整臂 VOID**；④ **#160**（`INDEX.md:185`）——VOID 不断链，下一 boot 以污染产物为 prev 启动；⑤ 260917-05 verdict §1（`:18-26`）——主会话「46/46 = ABI 切换」的定性被 worker 日志抽样独立推翻（#162）。这五件的共同形态是：**判据本身的观测对象（world 身份 / 开关状态 / prev 来源 / 口径归属）失效，而判据的机械读法照常执行并给出一个「看起来正常」的数**。

**缺口**：协议有「判据的时序」（预登记）与「判据的终止」（3 轮 halt），**缺「判据的前置依赖集」**——因此前置失效无法触发该判据的自动降级/失效，只能靠人事后发现（#161 是 judge 三源都没抓到的形态，说明事后审查不足恃）。

**具体改法（提案措辞）**
1. **判据 artifact 的 schema 扩展（`acceptance-criteria` artifact，§15.1 stage 0 产物）**：每条判据 MUST 附 `preconditions:` 集，每项含 `key`（观测对象的身份）/ `expected`（取值或形态）/ `check`（可机械执行的核对动作）。直接映射 Def.19 的偏函数 + 前置条件：判据 = $d$，前置事实 = $\text{dom}(\sigma)$，**满足性谓词 $\sigma\models d$ 恰是本项目已在用的 SELFCERT 硬门**（`criteria-260917-05.md:24-27`：Done≥1 / lightInit ok / dll sha8 / probe armed / fallback=0 / domain_hook=0 + world 身份项）。
2. **notify 三分落到判据状态**（Def.22 的直接搬用，语义完全对齐）：前置集满足性 `σ⊭d → σ'⊧d` = **activating**（判据从 suspended 转 active，可执行）；`σ⊧d → σ'⊭d` = **deactivating**（判据 SHOULD 自动降级为 superseded/suspended，**而不是**继续被引用）；否则 neutral。关键收益：**「前置失效」与「判据被推翻」被区分开**——当前协议只有后者（§15.4 supersedes），前者被误当噪声吸收（#161 的 200-changed 就是被当成「legacy 不收敛」这个**错误命题**的证据吸收了）。
3. **机械挂钩点（现成）**：采集脚本的 SELFCERT 硬门已经在做 $\sigma\models d$ 检查且已实现「任一缺失 → 整轮 VOID + 非零退出」（#118 + #160）。改法只是**把 SELFCERT 的检查项从「脚本内联」提升为「判据 artifact 的 preconditions 字段」**，使 VOID 可以**回指到具体判据并触发其降级**——当前 VOID 只断链当前轮，不通知「此前引用同判据的结论」。
4. **技能正文落点**：`anchor-degrade` SKILL.md（已管「证据饱和」与降级声明）加一条「前置集核对」；`anchor-judge` 审查清单加第 8 条「判据 preconditions 是否被核 + 失效判据是否已降级」；`anchor-scout` 起草实施规范时产出 preconditions 草案（该角色正是 stage 1，§15.1）。
5. **落地纪律（可先行）**：`AGENTS.md` §一新增「判据前置集」条款，并规定「引用跨块判据结论前 MUST 核该判据的前置集当前是否仍满足」。这一条**直接覆盖 §16.3 第 7 条（v0.20 host handover validation）的盲区**——v0.20 要求继承前做一次廉价独立验证（方向性结论），但没要求核「该结论所依赖的判据前提是否还在」。260917-04→05 的实际执行（verdict §1 独立推翻）证明这条纪律有效，但它靠人挑；前置集把它变成机械项。

**收益**：①把三类事故（M11 / #161 / #156）从「事后发现」变为「事前拦截」；②$d$ 与 `@pt` 的 source 天然连通——source 就是判据前置的**证据指针**，protocol 现有结构足以承载；③与 §9.7 三要素互补：§9.7 管「这个数能不能跟那个数比」，前置集管「这个数还能不能成立」。
**成本与风险**：①**前置集写全很难**——#161 的教训恰是「世界身份」这种隐式前提最难枚举；缓解 = 要求前置集**只登记已发生失效的类别 + 采集脚本已检查的项**（起步用 SELFCERT 现状），不追求完备（避免变成「写不全就不合格」的假门）。②**降级语义可能与 §15.4 冲突**：§15.4 明写「No downgrade below the evidence held」且 confirmed 只能由人授予——故本建议的「降级」MUST 措辞为「**判据 suspended + 引用它的结论标注『前提已失效，待复核』**」，**不得**自动改写任何 status（与 §15.4 一致）。这是本建议最需措辞精确之处，若写成自动降级即违反 §15.4 与 §16.1 confirm hook。

---

## 建议 3（立即可做）：分层观察等价门——把「可比」从声明式升级为构造式

**论文机制（页码）**：p.23-25 §3.3.2——Def.31（对值 $v$ 的观测 = 对它的 key 的操作集 $\mathcal{A}$ 跑**有限字**（forward map 与 yielded inverse 交替），两值不可区分 ⟺ 每个 test 在两侧同定义且同结果）；Def.33（$\sigma \simeq_S \sigma'$：**只比 $S$ 内的 key**，$S$ 外的部分被遗忘——`:1183-1186` 明写「heap layout 与 generative name 落在关系之外，除非有 key 绑定它们」）；Def.34（关系**沿类型构造子扩展**：函数 `f≃g ≔ ∀γ≃γ'. f(γ)≃g(γ')`、积逐分量、Maybe 分情形）；**Lemma 35**（在函数与迭代器上 $\simeq_S$ 是**偏等价**——对称 + 传递，故两个 related 成员各自 respect 它；**自反性是唯一不被保持的性质**，「respect 是条件而非白给」）；Def.36（$\mathfrak{E}^S_\Gamma$：witnessed **up to $\simeq_S$**，把 §3.1 的等式全部换成 ≃ 后 Thm.7 等**原证明不变**——Lemma 38）；p.25 Lemma 39（$i$ 的每个 stage 只读写它自己声明的 key ⇒ 取 $S$ = 该成员声明的 key 即可满足假设）。

**协议/落地现状**：协议已有一个**声明式**的可比性契约——§9.7（`:876-884`）三要素（载体 / 覆盖面 / 与历史口径可比性）+ §9.5 降级声明。落地已相当扎实：判据文件与 verdict 都带 §9.7 三要素（`criteria-260917-05.md:29-31`、`verdict-260917-05.md:59-64`，含「emptySec/hash 口径为本块新立，无历史数字可比，禁止与 G3/G17 系 changed 数字直接混算」）；§11 家族已积累**噪声锚分级**（#111：同配置 run-to-run 锚 ≠ 跨实现代理基线；#148：载体噪声下限；#103：配对交错）、**内容指纹门**（#115：FNV-1a 位置敏感指纹逐 chunk 比对，4140/4140 ⇒ 二值强判据）、**同构建态单变量 A/B**（#150：同一 dll 零重编 + 运行时 flag 覆盖 + 每臂正负成对计数自证）、**等价性门**（#116：整文件/条目级 sha256，old=new 逐字节相同）。
**但**：这些全是**经验纪律**，协议层只有「你 MUST 声明可比性」，没有「什么条件下两个执行体**可以被判为观察等价**」的构造性判据。

**缺口**：①§9.7 是**自我声明**——声明错了没有机械拦截（#162 的教训正是「口径解释未经验证就是新伪差」：主会话的「46/46 = ABI 切换」就是一个**看起来合理的可比性解释**，被 10 行日志抽样推翻）；②**并发交错形态**从未被 §9.7 覆盖（三要素只讲载体/覆盖面/历史口径，不讲「两个观测是否在同一个交错下取得」）——而本项目已有相关事故（#113 自锁死「两臂都卡」、MT8/9/10 探针污染测量、#107 单车道并行度塌陷）。

**具体改法（提案措辞）**
1. **新增「分层等价门」三档（映射 §9.1 的 Full/Partial/Degraded，但维度是**关系强度）**：
   - **E-Full（同形态单变量）**：同一执行体 + 零重编 + 运行时 flag 覆写 = **#150 形态**。此时观察等价可直接**构造性地**断言（论文 Def.34 的 respect 条件由「同一份二进制」给出）——这是当前**唯一**能支撑量级结论的档。
   - **E-Partial（跨形态 / 跨实现）**：不同 dll 或不同门控形态。此时 MUST 取 $\simeq_S$ 的**restriction**读法——即论文 Def.33 的 $S$ = **两侧共同声明的观测 key 集**（本例 = changed 集合 / 指纹覆盖的 chunk 集 / 逐块对拍的分母）。禁止把 $S$ 外的分量（如 `common` 计数缺口、section 数语义 — 参见 #110「分母语义 `4096 × 两侧 section 并集`」）算进等价性判断。
   - **E-Degraded（并发/交错形态）**：两侧的观测不在同一交错下 ⇒ **不可判等价**，只能判「在已观测交错下未发现差异」。措辞要求与 §9.5 同构。
2. **偏等价性的诚实条款（论文 Lemma 35 的直接后果，这是本档最有价值的一条）**：$\simeq_S$ 在函数/迭代器上**只是偏等价，自反性不成立**——两个「看起来相同」的执行体，若其中一个在某 key 上分支（`:1238` 明写「A map that branches on a key outside $S$ respects $\simeq$ and fails to respect $\simeq_S$」），等价关系即不适用。**协议化**：声明等价前 MUST 先证明「两侧在 $S$ 之外无分支」——本项目里这恰是 **#115 内容指纹门**在做的事（指纹取在写回之前，明确声明「不覆盖写回/并发路径」`verdict-260917-05.md:63`）。建议把 #115 的边界声明**升格为等价门的强制组成部分**。
3. **反例条款（防「假可比」）**：#162 四条判据（定性抽样核验 / 存在前提 grep / 同口径≠无伪差 / **计数恒等≠集合恒等**）建议收为 §9.7 的**反例清单**——§9.7 现在只正面要求「声明三要素」，不列「什么声明是无效的」。把 #162 的「计数恒等 ≠ 集合恒等」写进条款，能直接拦住 verdict-260917-05 §2a 自己诚实标注的那个盲区（`:33`：emptySec 是计数不是集合）。
4. **技能正文落点**：`anchor-write`（source 合法性）加一句「跨执行体/跨形态的 source MUST 带 $S$ 声明」；`anchor-judge` 审查清单第 1 条（source 是否满足 §5.5）扩为「+ 可比性档位与 $S$ 是否声明」。
5. **落地纪律（可先行）**：`AGENTS.md` §一.7（可比性声明）从三要素扩为**三要素 + 等价档位 + $S$**；把 #111/#115/#116/#150 作为该条款的实例指针（避免重述，符合 `AGENTS.md` 引用纪律）。

**收益**：①把「可比性」从**叙述**变成**可机械核对的档位 + key 集**——§9.7 的自我声明从此有了可反驳的载体（#162 类事故的拦截点）；②并发交错形态**第一次**进入协议视野（当前 §9.7 三要素对它无话可说，而本项目 MT8/9/10、#107、#113 三族事故都在这个维度）；③与建议 2 的 preconditions 天然衔接（$S$ 就是前置集的 key 集）。
**成本与风险**：①**形式化 vs 可用性的张力**：论文的 $\simeq$ 建立在「操作集 $\mathcal{A}$ 已知」之上，而本项目的观测载体（探针 hash、快照 diff）**没有显式操作集**——直接搬形式定义会变成空转。故建议 1-3 全部采用**档位 + key 集**的操作化形式，**不搬**$\mathcal{A}$ 与 $\lambda$ 演算。②**E-Full 档适用面窄**：跨版本/跨实现（如 1.20.1 vs 1.21.6、Java vs Rust）永远落 E-Partial，若写成「E-Full 才行」会禁掉项目主线工作——条款须明写 E-Partial 是**正常档**（对应 §9.1 Partial 是正常模式而非失败）。

---

## 建议 4（需验证）：流程不变量——progress（无死锁）与 confluence（唯一裁决终点）

**论文机制（页码）**：p.49-50 **Thm.73（Progress）**：(1) **No deadlock**——`¬quiet_t ⇒ 某条 lifecycle 规则适用`；(2) **Termination**——$S(n) \le (K+3)(V(n)+1)$，且 $V(n)$、$\sum_n S(n)$ 有限；**推论：任何 maximal 序列都终止于 quiescent**。**证明的关键结构**：反证假设无规则适用 ⇒ 构造一条 $\prec$-递增的 `Unloading` 链（`:2616-2636`），**由 acyclicity 保证链有限** ⇒ 构造必然停止 ⇒ 矛盾。**注意 `≺` acyclic 是假设而非定理**（`:2576-2578`：「这是 assumption and not something the definition delivers，$n\prec n$ 对自提供自声明的组件成立」）；p.51-54 **Thm.80（Confluence）**：(1) **Canonical form**——任何到达 quiescent 的序列，都可以重排成「按 $\rhd$ 线性化、每个 $n_i$ 恰好一个 episode」的规范形；(2) **Confluence**——任意两条取相同 orchestration steps 的序列，其终态 $\simeq_K$ 相等。**前提**：每个组件 total on its provision（Def.76）+ $\prec$ 无环；**关键结构**：Lemma 78（transposition——相邻独立步可交换）、Lemma 79（deletion——删掉一个闭合 episode 不改变 quiescent 态，「The deleted steps leave the state where they found it」）。

**协议/落地现状**：
- **halt 语义存在**：§15.4 C（`:1399-1415`）——同一验收判据 3 轮未达标 → **pipeline MUST halt entirely**，judge MUST NOT 继续修复/重分类/re-review，MUST 提交详细报告交人类判定（判据可能错 → §12 挑战或人类修正）。落地即 v0.15 C-gate（`AGENTS.md` 顶部与 §一.6/§二）。
- **confluence 的对应物存在但不完整**：§15.4（`:1276`）「Transitions are one-way: draft → candidate → confirmed」+ v0.20 supersedes 双指针（结论取代链——原结论不可改写）+ judge 只出意见不改 status（`anchor-judge` SKILL.md:16-17，`:922` audit 行「Review gate MUST NOT change status directly」，有 `test_review_gate_is_opinion_only` 覆盖）。这与 Thm.80(2) **同构**：不同调度序（不同 agent 顺序）到达同一终态（同一 status + 同一 supersedes 链）。
- **真实事故（流程不变量破裂）**：**#160**（`INDEX.md:185`）——GATE 只打印不退出 ⇒ boot1 判 VOID 后**链未断**，boot2 已带上一轮污染产物为 prev 启动，kill 留半截日志。这**恰好是 Thm.73 假设被违反**：`quiet` 的定义要求「无 in-progress transition」而 VOID 态的 fiber 实际 in-flight，于是「maximal 序列终止于 quiescent」不再成立——**halt 没有成为真正的 quiescent 态**。另一侧：§15.4 说 halt 后「human decides」，但**没有条款说「halt 的结论不得被下一轮静默继承」**——#160 的污染正是通过「prev 传递」发生的。

**缺口**：协议有 Progress 的**动作**（3 轮 halt）与 Confluence 的**动作**（单向 status + supersedes），但**没有把二者写为流程不变量**，因此：①halt 后「不得静默继承」无条款（#160）；②fan-out 多候选的**汇聚义务**无条款——`core.fanout` 技能要求各产 `.bN` 候选交 judge 后用户拍板，但协议层无「候选必须汇聚到唯一裁决终点」的强制（260917-05 verdict §2e `:52` 记录了一次**未开 fan-out** 的裁量：数据面无互斥分叉所以不开——该裁量本身合规，但缺少「何时必须开、何时允许不开」的形式判据）；③**acyclicity 前提未被登记**——Thm.73/80 都**假设** $\prec$ 无环，论文诚实标注这是 assumption；协议侧的对应物是「判据之间的依赖不得成环」（判据 A 引用结论 B、B 引用 A），当前**无任何检查**。

**具体改法（提案措辞，标注「需验证」的原因见风险）**
1. **§15.4 新增「流程不变量（Process Invariants，提案）」两条**：
   - **PI-1（Progress / 无静默悬置）**：任何 halt 态 MUST 是**终止态**——①以非零退出码或等价机械信号表达（#160 已实现于脚本层，建议升为条款）；②**不得**作为下一轮的前置输入（禁止 prev 继承）；③MUST 有唯一的裁决去向（人类裁决 / §12 挑战 / 回规划，三选一，§15.4 C 已列，建议明确「必选其一」）。
   - **PI-2（Confluence / 唯一裁决终点）**：同一判据下的多条并行候选（fan-out 的 `.bN`）MUST 汇聚到**唯一**裁决终点；汇聚前任一候选**不得**被单独引用为结论（对应 Thm.80 的「相同 orchestration steps + 不同调度序 → $\simeq_K$ 等价终态」）。落地对应 `core.fanout` 技能 + 260917-04 的双 worker 收敛（b1/b2 交 judge 后收敛，`10-timewise-archive:3319`）。
2. **前提登记（直接照搬论文的诚实做法）**：Thm.73/80 把 $\prec$ acyclic 列为**假设**并说明「定义本身不提供它」。建议协议在引入 PI-1/PI-2 时**同样明写前提**：「本不变量假定判据依赖图无环；自引用判据（结论 A 的判据依赖 A 自己）不满足前提」——这既是诚实声明，也恰好给出一个可机械检查的**判据依赖环检测**（§11 audit 可登记为待实现）。
3. **技能正文落点**：`anchor-judge` 的 halt 段（SKILL.md:25、:57）补「halt 后 MUST NOT 被继承」；`anchor-scout`/`anchor-worker` 加「产出候选即登记汇聚点」。
4. **落地纪律（可先行）**：`AGENTS.md` §一新增「halt 断链 + 唯一裁决终点」条款（#160 与 260917-04 作为实例）。

**收益**：①#160 类事故从「教训」变为「条款」（且已有脚本级实现可引用，零额外成本）；②给出 fan-out 的**形式化汇聚判据**，把「何时必须开/何时允许不开」从裁量变为可辩论的判据（当前完全靠 AI 判断，260917-05 `:52` 的裁量虽合规但无判据载体）；③判据依赖环检测是论文诚实前提的**免费副产品**。
**成本与风险**：**本建议标注「需验证」的核心理由**——论文的 Progress/Confluence 建立在一个**形式化操作语义**上（lifecycle 规则表 + fiber 状态 + episode 定义），而协议的「流程」是**人的/agent 的工作流**，两者之间是**类比而非实例化**（论文第 4 章的形式对象与协议第 15 章的 judge/skill 之间没有映射，我未找到论文提到任何验证协议或 agent 工作流）。因此：①**不要把 Thm.73 的具体界（$(K+3)(V(n)+1)$）搬进条款**——它依赖 fiber/target view 等无对应物的结构；只搬「无死锁 + 终止于 quiescent + 唯一终点」三个**形态性结论**；②「quiet」的协议对应物需明确定义（当前没有；建议定义为「无 in-flight 验证动作 + 无未决意见 + 无未汇聚候选」），否则 PI-1 无法机械检查；③**需先做一次验证**：在 CoreSwap 现有流程上试跑「判据依赖环检测」（哪怕人工枚举当前活跃判据的依赖图），确认无环假设在真实项目里成立或找出反例——若不成立，PI-2 的措辞必须改为「允许环但须登记」而不是禁止。这条验证成本 ≤1 轮，建议列为落地前的前置。

---

## 建议 5（需验证）：引用完整性检查——协议 §6.6 的「接口漂移/键碰撞」的协议自用

**论文机制（页码）**：p.75 §6.6——形式模型里依赖链**纯由 key 身份建立**，类型族 $\mathcal{V}k$ 只在单一编译单元内保证类型一致；跨独立开发/构建时出现两类失效：**Interface drift**（provider 改了 $k$ 的接口——加字段/改签名/改行为契约——而 consumer 仍声明同一 $k$：依赖在 coeffect 层**被满足**（$k\in\text{dom}(\sigma)$），但运行时值不再符合 consumer 预期 → 类型错误 / method-not-found / **静默行为分歧**）；**Key collision**（两个独立开发的 provider 用同一个 $k$ 名字表示完全无关的接口，consumer 接受任一而不做兼容检查）。三种方案：**key namespacing**（$K \to K\times P$，最直接但最耦合）、**peer dependencies**（借宿主包管理器强制版本范围——**Cordis 当前采用**——但① 依赖 provider 遵守 semver 是**不可强制的约定**，② 包管理器通常解析到单一版本，**阻止同一应用内加载同包多版本**）、**structural compatibility**（把 $k\in\text{dom}(\sigma)$ 换成结构兼容谓词，类似结构子类型；记录类型（宽度子类型）可行，行为契约与受界量化下**不可判定**）。论文明确结论：三者结合的统一依赖模型**仍是 open problem**。

**协议/落地现状**：协议对**自身**的版本引用有纪律但无检查：
- §10 版本纪律（`:888-892`：minor MUST 向后兼容，旧 noise card 保持可读 —— v0.21 = pre-stable）。
- 落地 `AGENTS.md` 顶部「与 Anchorlaw 版本同步契约」：Anchorlaw 为**协议引用单一事实源**（只读），升级动作 = `git grep 'v0\.[0-9]'` 全仓 + 核版本号与引用条款；当前基线 v0.21（2026-09-17 核对，v0.20→v0.21 = 纯宿主适配登记、协议核心零变化，核对方法 = 归一化逐行 diff）。**但这条纪律无机械门禁**——`AGENTS.md` 明写「禁止反向修改」、靠人工 grep。
- **真实事故（正是 §6.6 的 interface drift 形态）**：**#156**（`INDEX.md:180`）——复测口径「使能开关失传」（`-PlightRust` 未随口径传递）⇒ 两臂实为 vanilla 形态 ⇒ **整臂 VOID**。这是**跨块的引用漂移**：一个「引用」在被传递过程中丢失了它的语义前提。另一形态：`AGENTS.md` 顶部自述 skill 已装 v0.21 引用，但项目级 `.dsh/skills` 副本是**本日才补齐**的（此前为 v0.20 → 引用漂移窗口）。**#13 家族**（转录失真）与 **#90**（交接清单条目转抄漂移）是同族的另外两形态。

**缺口**：协议 §6.6 精确描述了「引用完整性失效」的两种机制（drift / collision），**但协议没有把它用于自己的引用**——协议条款之间、协议与宿主文档之间的引用（如 `AGENTS.md` 引 §15.4、skill 正文引 `spec/protocol-v0.21.md#154-consistency-contract`）**没有完整性检查**。这构成一个漂亮的**自指机会**：论文 §6.6 给出的判据可以直接变成协议的自我检查项。

**具体改法（提案措辞，标注「需验证」）**
1. **新增协议版本引用完整性检查（`anchor-maintain` 技能 + CLI，提案）**：扫描宿主仓库全部 `.md`/skill 正文中的 `v0\.[0-9]+` 与 `§N.N` 引用，产出检查表：①引用的版本号是否为当前基线（stale 版本引用 = interface drift）；②引用的**条款号是否仍在位**（§N.N 被删除或重编号 = key collision 的条款级形态——同号不同义）；③引用的锚点是否可解析（如 `#154-consistency-contract`）。落地已有前置：RE-Framework `ref_manifest_validate`/`ref_status` 与 `dsh/scripts/selfcheck.ps1` 已做 manifest/安装产物核对，可挂为第 6 段；`Anchorlaw` 侧 §11 audit 表（`:902-930`）**逐行登记了每条通用声称的验证状态**——这正是承接本检查的现成载体。
2. **peer-dependency 方案的对应物与已知局限（照搬论文的诚实标注）**：协议的「peer dependency」= §10 的 minor 向后兼容承诺 + 宿主显式声明基线版本（`AGENTS.md` 顶部即此形态）。论文指出的两条局限**在本项目已各出现一次**：①「依赖遵守 semver 是不可强制的约定」→ v0.21 是 pre-stable，且 v0.18→v0.20 曾**一次引入三条新条款**（§15.4 取代链 / §9.7 / §16.3），远超「纯 bump」的预期——**minor 兼容承诺在实践中弱于字面**；②「包管理器常解析到单一版本，阻止同应用内多版本并存」→ 本项目同时存在 v0.20 与 v0.21 的引用窗口。建议协议**明写**这两条局限（借 §6.6 的措辞），而不是仅在 §10 说「MUST backward-compatible」。
3. **不建议做 structural compatibility（协议自用的那一半）**：论文自己指出行为契约与受界量化下**不可判定**。协议引用检查只应做**名义 + 结构可判定层**（版本号 + 条款号存在性 + 锚点可解析），**不要**尝试「条款语义兼容性」检查。这一条正是本建议「需验证」的边界。
4. **落地纪律（可先行）**：`AGENTS.md` 的同步契约条款增加「引用完整性检查输出入 judge 三源之一」（当前三源 = artifacts 快照 + git diff + verification records，可扩为四源）。

**收益**：①把 #156/#13/#90 三族「引用漂移」事故统一到一个判据下（论文 §6.6 提供机制命名）；②§11 audit 表是现成载体，检查成本低（文本 grep + 锚点解析）；③「同号不同义」检查恰好防住协议升级时最容易出的错（§15.4 在 v0.13-v0.20 间多次 amended，条款语义已变而引用文本未变）。
**成本与风险**：①**范围爆炸**风险——`git grep 'v0\.[0-9]'` 在 CoreSwap 全仓命中量巨大（历史归档按历史读、**不改写**，这是 `AGENTS.md` §八.11 与 260910-06 open 项 ⑦ 的明确纪律），若检查把历史归档也算进来会产生海量假阳性；条款 MUST 限定「活引用 vs 历史归档」两分（**#117 简记已立此区分**）。②**自指风险**：协议检查自己的引用，若检查规则也写在协议里，则规则本身需要版本化——需明确该检查属 `anchor-maintain`（协议维护）而非通用宿主义务，避免所有宿主被迫实现。③价值可能有限：本项目引用漂移的主因是「人没跑 grep」，加一个 CLI 不一定改变行为——**这条是本建议最弱的一条**，故列于第 5 位并标注需验证。

---

## 「不建议做」清单（明确排除，附理由）

1. **不建议把论文的完整演算（$\partial\Gamma$ / twisted composition monoid / $\mathfrak{E}^*_\Gamma$ witness）引入协议形式化**。论文 §2.3 明确动机是「static type systems 的运行时化」，其对象是**程序上下文**；协议的对象是**验证活动的状态**。搬形式对象会得到一套无法机械检查的符号体系（违反 §15.4「termination gates 必须机械」的精神），且与 §16.2「协议 MUST NOT 要求宿主重构自身」相抵触。
2. **不建议引入「验证副作用的自动回退」**。论文的 revert 由**运行时**持有并强制执行（`track`/`recover` 是运行时变换）；协议的宿主是 LLM agent + 人工，**自动回退文件系统**超出协议边界且危险（可能删掉本该保留的失败轮证据——恰与 #146「失败臂日志价值高于成功臂」相反）。建议 1 只做**登记 + 可寻址**，不做执行。
3. **不建议把 Thm.73 的具体数值界或 Thm.80 的 $\simeq_K$ 证明结构搬入条款**（理由见建议 4 风险）。
4. **不建议做协议引用的「语义兼容性」检查**（§6.6 自述不可判定，见建议 5 第 3 点）。

---

## 附：本稿的一手/推测分界自检与 open 项

- **@anchor.idk**：`@anchor.idk("论文 §5 Cordis 实现的机制细节（effect tracking 的实际数据结构、coeffect resolution 的算法）未读——本稿只依据 §1-§4 的形式机制与 §6 讨论；若实现层有与形式层不同的取舍，建议 1-3 的操作化可能需要调整", source="static: .tmp/paper-2608.25512.txt 目录页定位 §5.1-5.2 但未读正文；本次任务书未要求")`
- **@anchor.idk**：`@anchor.idk("建议 4 的『判据依赖图无环』在 CoreSwap 真实流程中是否成立未验证——论文明写 acyclic 是 assumption 而非定理，且自提供/自声明组件会违反它；落地前需一次人工枚举（≤1 轮成本）", source="static: 论文 :2576-2578 与 :2665-2670 的假设声明；本项目未做依赖图枚举")`
- **@anchor.idk**：`@anchor.idk("本稿对 Anchorlaw 是外部视角（只读、未跑 anchorlaw CLI 验证任何条款的可实现性）——建议 1/2/3/5 的 CLI/技能改动均为纸面设计，未编译未试跑", source="static: 任务约束「不改 Anchorlaw 仓库、只读 + 草稿」")`
- **§9.7 可比性声明（本稿自身适用）**：载体 = 论文纯文本（`.tmp/paper-2608.25512.txt`，PAGE 标记内页码）+ 协议 v0.21 正文 + CoreSwap 落地文档；覆盖面 = 论文 §3.1-3.3/§4.3.4-4.3.5/§6.6 与协议 §5/§9/§11/§15.4/§16 + 落地实例 260917-01/03/04/05 与 #156-#162；**可比性** = 本稿为**第一份论文×协议对照**，无历史同类件可比，禁止与既有 verdict 的数字口径混算。
- **status 纪律**：本文件 `draft`。建议授予 `candidate`（需 judge 审查意见）；`confirmed` 留用户。本 worker **未修改**任何既有 status、**未执行**任何命令、**未改动** Anchorlaw 仓库。
