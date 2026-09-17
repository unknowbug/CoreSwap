```yaml
id: pbeta-260917-05:discovered-upgrade-draft
block: 260917-05
status: draft          # 草稿供主会话应用；编号/载体归属最终由主会话定
worker: subagent (core.worker)，工作块 260917-05
scope: 框架升级（Anchorlaw v0.21→v0.22 / RE-Framework v2.5→v2.6）+ 论文研究落地 这一轮的可复用发现
载体建议: 主载体 = knowledge/discovered/workflow-patterns.md（框架/门禁/纪律类）；次载体 = knowledge/discovered/build-tooling.md（安装链/沙箱/工具类）
```

# discovered 草稿 · 框架升级执行轮（260917-05）

> **编号占位说明**：`workflow-patterns.md` 现最大 = **#162**；`build-tooling.md` 现最大 = **#154**（末尾为「#8/#118 家族补充案例」与「#42 家族补充案例·第三犯」两个补充案例段）。下方编号一律写「暂标 #16x / #15x」，**实际编号由主会话按应用时刻的现网最大值定序**（本文件不改任何既有编号，也不改任何 status）。
> **一手来源**（全部逐条读过并核对数字）：① `.investigations/pbeta-260917-05/upgrade-execution-record.md`；② `.investigations/paper-2608-25512/remaining-after-upgrade.md`；③ `.investigations/pbeta-260917-05/pi2-dependency-enumeration.md`；④ `.investigations/pbeta-260917-05/criteria-260917-05.md`（末两节）；⑤ `E:\PYTHON\RE-Framework\dsh\scripts\install.ps1:104-119`、`selfcheck.ps1:64-118`；⑥ `E:\PYTHON\Anchorlaw\spec\protocol-v0.22.md` §9.8（:950-964）/ §15.4 PI-1·PI-2（:1606-1629）；⑦ `E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1:1-60`（-Project 模式）。

---

## §0 价值门判定与合并/否决决策（先判价值，再定载体）

| 候选 | 判定 | 载体 | 处置 |
|---|---|---|---|
| **A 内容对账 vs 计数门（silently-green 门）** | **高价值（必记）** | workflow-patterns | **写**——判错方法/可复用判据 + 有 CoreSwap 侧漂移实证 + 上游同构自述 |
| **B 多副本安装树「无脚本覆盖面」** | **高价值（必记）** | workflow-patterns（附 build-tooling 指针） | **写**——与 A 同族但**根因不同**（A = 门判什么；B = 谁写谁验），不合并，做交叉指针 |
| **C 升级路径上的沙箱边界首次碰撞** | **中-高（错误链，五段式必写）** | workflow-patterns 或 build-tooling（主会话定） | **写**——错误/失败链条 + 环境坑，五段式；与既有 #42 tmpdir 家族、AGENTS「沙箱 escalation 工作区外先拒」并列 |
| **D PI-2 前提枚举（边对象分类法）** | **高价值（必记）** | workflow-patterns | **写**——协议强制前置的**首次执行形态**，方法可复用（依赖图无环判定） |
| **E §9.8「不可逆声明」形态（首实例）** | **高价值（必记）** | workflow-patterns | **写**——反模式的对偶面（不许伪造逆），有协议 rationale 一手背书 |
| **F 触发覆盖审计工具假阳性** | **中-低** | **不单独成条** | **合并**进 D 条目末尾「配套：工具首用 FP 形态」一行；不写进 build-tooling |

**合并说明（须理由）**：
1. **D + F 合并**：两者同属「PI-2 / 审计工具首用」这一轮的动作面，且 F 的价值门只到中-低（一次性工具试跑 + 假阳性率 1.1% 与上游一致），独立成条会稀释高价值权重。合并后 F 作为 D 条目的「配套观察」保留可复用判据（**按目录边界扫描的覆盖工具遇跨目录存放产物必假阳性**）。
2. **A 与 B 不合并**：虽然同属「多副本安装树」这一轮发现，但根因面正交——A 的根因是**门禁判据的语义**（存在/数量 ⊂ 内容），B 的根因是**覆盖面归属**（谁写、谁验）。合并会把两个不同判据压成一条，下次遇到只能复用一半。
3. **C 独立成条**：它是唯一的**错误链**（install 中止），按错误优先原则详实度要求最高，且五段式格式与判据条目不同（须与既有 build-tooling #42/#46 沙箱家族交叉引用）。

**不写入知识库的（低价值，只留 .investigations/）**：
- 具体数字快照（`.dsh/skills` 28 个目录 / 8 drifted / 36× v0.22 / preset 37 + user-global 17 / 全仓 ~90 课题 1 MISSING / 判据 4 份）——这些是**本轮对齐状态快照**，价值门明列「不记」；它们在正文中只作**证据行**出现（判据的支撑），不单独成条。
- B6 项目侧接管面契约（未执行，只是立项建议）——不属本轮发现，留 `.investigations/` 待立项。

---

## §1 workflow-patterns 新增发现草稿

### 发现 #16x（暂标；最高价值·错误优先）: 门禁只判「存在/数量」就是 silently-green 门——内容对账门与计数门的语义边界（260917-05）

- **发现时间 / 发现者 / 置信度 / module**：260917-05；主会话执行 B1/B7 时实证 + 上游 v2.6 同步落地，core.worker subagent 起草；**candidate**（本工作区侧 = 内容 sha256 递归对账 `ALL GREEN (17 skills, recursive)` + selfcheck 五段全绿实证；上游侧 = selfcheck 源码自述，非本工作区复现）；workflow-patterns / 门禁设计与安装产物核验（**#105「载体偏差」家族的门禁形态 + #118「自证行硬门禁」的判据语义维**）。
- **来源定位**：`.investigations/pbeta-260917-05/upgrade-execution-record.md` §B1（8 drifted 清单 + `ALL GREEN (17 skills, recursive)` + selfcheck 两段对账报告）+ `.investigations/paper-2608-25512/remaining-after-upgrade.md` §B / §B1（项目级 `.dsh/skills` = 28 个、版本标记 13× v0.20 + 5× v0.21、`§5.6|preconditions|evidence grade|suspend` **零命中**）+ 上游实现 `E:\PYTHON\RE-Framework\dsh\scripts\selfcheck.ps1:56-58`（旧段：`$count -lt 17`）vs `:64-118`（新段：MISSING/DRIFT/ORPHAN 逐件 sha256）。

- **观察（现象）**：项目级 `E:\PYTHON\CoreSwap\.dsh\skills` 的**目录计数一直是「绿」的**（28 个目录，ref-\* 17 + anchor-\* 11，数目从未少过），但内容层面 **8 个 ref-\* 已 drift**（core-artifact / core-fanout / core-judge / core-knowledge / core-plan / re-lift / ref-maintain / swe-guide）——残留 v0.20/v0.21 版本标记、v0.22 新条款关键词（`§5.6` / `preconditions` / `evidence grade` / `suspend`）**零命中**。即：**门是绿的，产物是陈旧的**，且这一状态**无任何机制能发现**，只能靠人手工逐文件 diff/grep 才暴露。上游 v2.6 恰好把 selfcheck 第 3 段从**目录计数**（`$count -lt 17`）升级为**内容对账**（install-manifest.yaml / .re-framework-manifest.yaml 逐件 sha256，报 MISSING/DRIFT/ORPHAN 并非零退出），其源码注释自述这与 preset 行解析门禁是**同一失败类**：*"a gate that silently goes green when it cannot execute is the same failure class as the incident it guards against"*。

- **根因（机制，为什么错）**：**「存在」与「数量」是内容的必要非充分条件**——一个门禁若其判据落在目录存在性/条目计数上，则「产物被替换为旧版本」「产物被局部编辑」「产物被同名占位」三类退化**全部落在判据的盲区**，门恒绿。这不是实现 bug，是**判据语义的层级错配**：被保护对象的真实语义是「产物 = 安装源的内容」，而门问的是「产物 = 若干条目」。与 preset 行解析门禁同构：后者在**无法执行**时静默绿（缺口被吞），前者在**产物不可信**时静默绿（漂移被吞）——共同点是**门的判据没有触达被保护语义本身**，于是门的「绿」只证明门跑了，不证明被保护对象没坏。

- **定位（怎么发现的，可复用）**：① **内容级 grep 反证**——不数目录，直接 grep 新条款关键词（`§5.6` / `preconditions` / `evidence grade` / `suspend`）→ 零命中，而 user-global 侧同一批文件命中，两副本对比一眼看出 drift；② **逐文件 sha256 递归对账**——用显式文件清单（非通配，防混入）做递归 sha 对账，得 `ALL GREEN (17 skills, recursive)`，与 grep 结论互证；③ **核门禁判据本体**——读上游 selfcheck 第 3 段前后版本（`:56-58` 计数 vs `:64-118` 内容对账），确认「绿」的来源是计数而非内容。

- **修复 / 如何利用（判据，MUST）**：
  1. **判据（MUST）**：凡门禁判据只判「存在 / 数量 / 条目数」，**一律按 silently-green 门处理**——补内容对账（逐件 sha256 或内容指纹）后该门才可作结论依据；对外的「全绿」声明 MUST 同时声明**门测的是什么**（数量门 / 内容门），否则「全绿」不构成产物可用证据。
  2. **安装产物核验的正确形态** = `install-manifest.yaml`（source_commit + 逐件 sha256）+ 对账报 MISSING/DRIFT/ORPHAN + **非零退出**——「数量对」只能当**前置必要条件**，不能当结论。
  3. **门禁哲学（跨领域）**：门禁的判据必须触达被保护语义本身；**「门跑了」≠「门守住了」**。同一失败类的两种表现：① 门无法执行 → 静默绿（preset 行解析门禁上游自述）；② 门可执行但判据层级过低 → 静默绿（本条）。
  4. **反向操作提示**：升级门禁时，旧计数门**不要删**——保留为廉价前置（数量不对时连内容都不用查），内容门作为后置强判据；两者**判据域不同、结论强度不同**，MUST 分别声明。

- **家族索引**：#118（自证行硬门禁——「只打印不判定 = 没有门禁」；本条补「判定了但判据层级过低 = 门恒绿」）、workflow-patterns #105（载体偏差——「载体全绿」对真实故障零判别力；本条为其**门禁判据**维）、#27/#50（静默退化/静默销毁家族：退出码 0 + 无警告）、#16/#18/#23（「产物在盘 ≠ 当前语义」家族的门禁形态）、build-tooling #2/#27（marker 语义超载——同属「一个判据承载了它不承载的语义」）。

---

### 发现 #16x+1（暂标；最高价值）: 多副本安装树的「无脚本覆盖面」——逐副本 MUST 给出「谁写、谁验」两问（260917-05）

- **发现时间 / 发现者 / 置信度 / module**：260917-05；core.worker subagent 起草（主会话 B1 复核一手发现 + 手工同步执行）；**candidate**（脚本参数面 = 一手读码实证：RE-Framework install.ps1 无 `-Project` 参数、Anchorlaw `-Project` 只复制 `skills\*` 即 11 个 anchor-\*；项目级 17 ref-\* 副本 = 手工 Copy-Item 完成并内容对账全绿）；workflow-patterns / 部署覆盖面与副本漂移（**#100「谁写谁读」双侧核对家族**的部署副本形态 + **#105「载体偏差」**的副本维）。
- **来源定位**：`.investigations/paper-2608-25512/remaining-after-upgrade.md` §B1「脚本覆盖面实证（本次复核新发现）」（一手定位 = RE-Framework `install.ps1:1-12, 137-146`；Anchorlaw `install.ps1:38-55`）+ 本轮一手复核 `E:\PYTHON\RE-Framework\dsh\scripts\install.ps1`（**函数签名无 `-Project`**，头部注释的安装目标仅 `~/.dsh/.agent-presets/re-framework/` 与 `~/.dsh/skills/` 两处）+ `E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1:22-29, 38-55`（`param([string]$Project)`；`Copy-Item -Path (Join-Path $srcRoot 'skills\*')` = 仅 11 个 anchor-\* 落到 `<proj>/.dsh/skills`）+ `upgrade-execution-record.md` §B1（项目级 17 ref-\* 手工按显式清单 Copy-Item）。

- **观察（现象）**：同一框架/协议的技能产物在本机存在**三份副本树**，但脚本覆盖面**只有两份**：
  | 副本树 | 谁写它 | 谁验它 |
  |---|---|---|
  | `~/.dsh/.agent-presets/re-framework/skills/`（17 ref-\*，preset 内嵌） | RE-Framework `install.ps1`（无参） | selfcheck 第 3 段（preset manifest 内容对账） |
  | `~/.dsh/skills/`（17 ref-\* + 11 anchor-\*，user-global 共享树） | RE-Framework `install.ps1` + Anchorlaw `install.ps1`（无参） | selfcheck 第 3 段（user-global manifest 内容对账，命名空间限定 `^(core\|re\|recode\|swe\|ref)-`） |
  | `<proj>/.dsh/skills/`（**28 个**，Rank 100，本工作区会话最高优先命中） | **anchor-\* 11 个**由 Anchorlaw `-Project` 写；**ref-\* 17 个无任何脚本覆盖** | **无**（不在任何 manifest 的对账范围内） |

  ⇒ 项目级 17 个 ref-\* 副本是**漂移黑洞**：脚本不写它、manifest 不含它、selfcheck 不验它，其陈旧只能靠人肉 grep/对账发现——本轮 8 个 drift 正是这样暴露的（且是被**别的工作**顺带撞见，不是被机制发现）。

- **根因（机制）**：**「脚本可写面」与「运行时可见面」不是同一张地图**。DSH 的技能发现排名是 项目级（Rank 100）> `.agents/skills`（200）> preset 内嵌（300）> 用户级（400）——即**最高优先命中的那一份，恰是脚本从不写的那一份**。两个上游各自只覆盖自己的命名空间（Anchorlaw 管 anchor-\*、RE-Framework 管 ref-\* 但只写 user-global/preset），**没有任何一方对「项目级树」负责**：职责边界按命名空间切，不按副本树切，于是副本树成了职责的补集。更本质的是：项目级副本**不参与任何 manifest 的身份登记**，因此它连「内容对账」这类强门禁都没有输入可用——**没有身份登记就没有对账**（这也说明 A 条的内容对账门只覆盖「有 manifest 的副本」）。

- **定位（怎么发现的，可复用）**：① 读两个 install.ps1 的**参数面**（`param()` 块 + 头部注释的目标清单）→ 一眼看出无 `-Project`；② 读 `-Project` 分支的 `Copy-Item` 源（`skills\*`）→ 只有 anchor-\*；③ 对照运行时副本清单（`Get-ChildItem .dsh\skills -Directory` = 28 个）→ 差集 = 17 个 ref-\*；④ 复核 selfcheck 的 manifest 路径（两条：preset 内、user-global）→ 项目级不在其中。**四步全为读文件，零执行成本**。

- **修复 / 如何利用（判据，MUST）**：
  1. **判据（MUST）**：**多副本部署 MUST 逐副本给出「谁写、谁验」两问**——答不出「谁验」的副本即**漂移黑洞**，必须补（a）纳入某方 manifest 的对账，或（b）在部署脚本中显式声明「手工同步 + 同步后跑内容对账」，或（c）删掉该副本（若其可见性收益不值得维护成本）。**只答「谁写」不足以合规。**
  2. **覆盖面必须按「副本树」核对，不能按「命名空间」核对**——命名空间切分会让副本树成为职责补集（本条实例）；核对动作 = 列出运行时**实际可见**的每一个副本路径，逐个问「哪个脚本写它 / 哪个门验它」。
  3. **手工同步是不可接受的长期形态，但可以是合规的过渡形态**——前提是**同步动作本身有对账**（本例 = 显式文件清单 Copy-Item（禁通配，防混入）+ 递归 sha256 对账），且该对账 MUST 记入执行记录；「手工同步 + 无对账」= 最坏形态（既无自动门，又无一次性验证）。
  4. **副本优先级倒挂是风险放大器**：副本的**可见性排名越高，越容易被消费，越必须被门禁覆盖**（Rank 100 的项目级副本排在第一位，却唯一无门）——部署设计时按「可见性排名」而不是按「写入便利性」分配门禁强度。

- **家族索引**：**#16x（silently-green 门——本条为其覆盖面维：内容门只覆盖有 manifest 的副本）**、workflow-patterns #100（「谁写谁读」双侧核对——本条为部署副本的「谁写谁验」形态）、#105（载体偏差——最高优先命中的副本恰是无门的那份）、#77（回滚 ≠ 引用面清理——同属「结构动作的完整性检查」）、#54（目录级规则作用域 MUST 按「里面住了什么」核——同属「作用域/覆盖面必须先枚举再声明」）。

---

### 发现 #16x+2（暂标；错误优先，五段式）: 升级路径上的沙箱边界首次碰撞——上游新增的「更安全」机制会引入新的环境前提，升级后首次运行 MUST 视为 escalation 窗口（260917-05）

- **发现时间 / 发现者 / 置信度 / module**：260917-05；主会话 B1 首次生产调用实测（install 中止 → escalation 后成功），core.worker subagent 起草；**candidate**（一手：失败原文 + escalation 后命令全绿 + 上游源码定位到 `:104-119` 事务备份段）；workflow-patterns 或 build-tooling（**build-tooling #42 tmpdir 家族 / #46 沙箱可用性家族**的「升级引入新环境前提」形态 + AGENTS.md 全局铁律「沙箱 escalation 凡涉及 workspace 外路径，用户一律先拒绝再人工重审」）。
- **来源定位**：`.investigations/pbeta-260917-05/upgrade-execution-record.md` §B1 首条；上游实现 `E:\PYTHON\RE-Framework\dsh\scripts\install.ps1:104-119`（`$patchBackup = "$patchPath.bak-ref-install"` + `Copy-Item` 备份 → 失败时 `Copy-Item $patchBackup $patchPath` 还原 → `throw`）；注释自述动机 = *"The patch file is host-shared state: other frameworks' rows live in it too, so a bad rewrite is not recoverable by re-running this script. Back up first."*（同源 = 论文 §5.2.2 Algorithm 10 transaction 形态）。

- **五段式（错误优先）**：
  - **现象**：`pwsh E:\PYTHON\RE-Framework\dsh\scripts\install.ps1` 首次执行**中止**——写入 `C:\Users\NDark\.dsh\profiles\web\cordis.patch.yml.bak-ref-install` 被 DSH 沙箱拒绝（该路径在 session workspace `E:\PYTHON\CoreSwap` **之外**）；escalation 后同一条命令成功。**注意这个组合**：该脚本**本来就要写 DSH home**（`~/.dsh/.agent-presets/`、`~/.dsh/skills/`，本就在工作区外，历史上从无问题——因为那些写操作此前已在允许面内或以既有方式通过），而**新增的一步备份**落在了一个此前从未被写过的具体路径上，于是首个失败点是**新机制**而不是老命令。
  - **根因（机制）**：**「更安全」的机制会扩大写入面**。事务备份的语义要求「先复制一份到可寻址路径」，这一动作把写入面从「目标文件」扩展到「目标文件 + 备份路径」——而备份路径（`cordis.patch.yml.bak-ref-install`）是**首次出现的新对象**，其父目录/命名的可写性**从未被历史命令验证过**。⇒ 上游把「不可恢复的坏重写」风险换成了「首次运行撞环境边界」风险，**这是一次风险转移而非纯收益**；而转移后的风险**只在首次运行暴露**（一旦备份文件已存在，`Copy-Item -Force` 覆盖同一路径的行为是否仍被拒，取决于是「创建新文件」还是「覆盖已有文件」的沙箱策略，**本块未单独复现，属 idk**）。
  - **定位（怎么发现的，可复用）**：错误文本本身就指名了**新路径**（`.bak-ref-install` 后缀是这次升级新引入的命名）——**判错动作 = 读失败路径，并与「这次升级改了什么」对照**：失败路径里的命名是升级前不存在的 ⇒ 先怀疑新机制而不是老命令的回归。辅证：读上游 `install.ps1:104-119` 确认该段是 v2.6 新增的事务备份段（注释与论文 Algorithm 10 同源）。
  - **修复（处置）**：escalation 后重跑成功（本会话批准面）；**纪律层修复**（本条的真正价值）= 把「升级后的第一次运行」显式列为 **escalation 窗口**：
    1. **升级动作分两步走**：先在一个可 escalation 的窗口里跑**一次**安装脚本（允许首次撞边界），确认新增机制在目标环境可用后，再把「重跑安装脚本」当作例行命令；**不要**把首次运行放在无人值守/批处理链里。
    2. **对上游新增的每一个「更安全」机制，问一句「它新增了哪些写入对象」**——新增的备份/临时/日志/锁文件路径都是新的环境前提，MUST 在升级记录里逐个登记（本条的登记物 = `cordis.patch.yml.bak-ref-install`）。
    3. **失败要中止而非降级**——本例 install 中止是可接受的（宁可停也不要「跳过备份继续改共享 patch 文件」，那正是备份段要防的事）；**升级后的失败模式检查 = 「失败时是中止还是静默降级」**，静默降级的「更安全」机制比没有更危险。
  - **教训（可复用判错经验）**：
    1. **「老命令照跑」不成立**——命令没变，但它依赖的**环境前提变了**（新写入对象）；环境边界类故障在升级窗口的暴露率被系统性低估。判据：**升级后首次运行 MUST 视为需要 escalation 的窗口**，且失败路径里的**新命名**是第一线索。
    2. **沙箱类失败先核「这次多写了什么」**，不要先怀疑权限配置回归（AGENTS.md 全局铁律：「沙箱 escalation 凡涉及 workspace 外路径，用户一律先拒绝再人工重审」——本条是这条护栏的**正常运作实例**，不是事故）。
    3. **同族**：build-tooling #42（JNA tmpdir 拒访——同为「环境前提未被满足」）、#46（沙箱能力必须实测、被拦形态可能是阻塞而非异常）、#153（沙箱 taskkill /I 组级 kill 拒绝——**沙箱能力面是分粒度的**，同一类操作的不同粒度可能一放一拒）；三者共同上位原则：**沙箱能力清单 MUST 实测维护，不凭常识推断**，且**升级动作是刷新该清单的强制时点**。

- **家族索引**：build-tooling #42（tmpdir 环境前提）/ #46（沙箱能力实测）/ #153（沙箱能力粒度）；workflow-patterns #105（环境/载体变化先核，勿按历史结论续推）、#18（跨 session/跨块结论不可直接续推——升级即一次「环境换代」）。

---

### 发现 #16x+3（暂标；最高价值）: 判据依赖图的无环判定 MUST 区分「外部事实边」与「判据结论边」——且「无环」是**假设**，须与前提登记同构（260917-05）

- **发现时间 / 发现者 / 置信度 / module**：260917-05；主会话执行 B2（协议明文 MUST 前置）+ core.worker subagent 起草；**candidate**（方法首次执行，覆盖面 4 份活跃判据 artifact 全量扫描 + 两条可疑边逐条判向；判定「无环」为**协议要求登记的产物**，非定理）；workflow-patterns / 判据依赖管理与协议前置（**#161「自证门须含数据身份项」的同族**：都是「前提的成立条件本身必须被核对」）。
- **来源定位**：`.investigations/pbeta-260917-05/pi2-dependency-enumeration.md`（§1 枚举 4 份判据 K1-K4 / §2 依赖边表 / §3 环检测结论 + 前提登记 / §4 PI-2 采用状态 + §5 诚实声明）+ 协议原文 `E:\PYTHON\Anchorlaw\spec\protocol-v0.22.md:1618-1629`（PI-2 条款：*"Hosts adopting PI-2 MUST first perform a cheap enumeration of the active criteria dependency graph (≤1 round) and register the result; if cycles exist, the clause MUST be amended to 'cycles permitted but registered' rather than enforced as a ban."*）+ `:245` / `:1047`（§11 审计表：acyclicity 是 assumption，非 definition 的交付物）。

- **观察（现象）**：PI-2（fan-out 候选汇聚唯一裁决终点）在协议中登记为 **unverified**，其前提 = **判据依赖图无环**；采用前宿主 MUST 跑 ≤1 轮枚举。本轮首次执行：枚举出 **4 份活跃判据 artifact**（K1 `c1-c6-260917-04` / K2 `cp1-light-form-260916-01` / K3 `g3-drift-basis-260917-01` / K4 `pbeta-260917-05`）及其边，其中**两条边指向「另一条判据的结论」**（K2 → K3 系的 G3 drift 阈值；K4 → K1 系 260917-04 已 confirmed 的新基线），其余边全部指向**外部事实**（载具身份 / 执行体三元组 sha / SELFCERT 自证日志 / seed / 运行序）。逐条判向两条可疑边 → 均为**单向后向引用**（K2 引用 K3 的已定论数值，K3 不回引 K2；K4 引用 K1 的已 confirmed 结论，K1 前置不含 K4）⇒ **当前活跃判据依赖图无环**，PI-2 可按协议原样采用（无需改为 "cycles permitted but registered"）。

- **根因（机制，判据的判据）**：**环的风险只存在于「判据结论边」上**——若边指向**外部事实**（sha / 日志 / seed / 运行序），该事实不依赖任何判据的满足性，无论引用多少条都不会成环；只有当边指向**另一条判据的结论（满足性）**时才可能形成 A 依赖 B、B 又依赖 A 的形态。⇒ 「无环判定」的正确操作 = **先对边做对象分类，再只对「判据结论边」判向**；把两类边混在一张图里跑算法（或凭「看起来都是引用」下结论）会把**结构性不可能成环的图**误报为有环。补充机制：K2 → K3 那条边当前是**历史边**——K2 的 2.5% 阈值已随载体回炉并被 260917-01/02 以 §15.4 **取代**收口，被取代的边不再承载强制（这解释了「活跃图」必须按**当前状态**枚举，而不是按文件里出现过的所有引用）。

- **定位 / 方法（怎么做的，可复用）**：
  1. **枚举活跃判据**：全量扫描 `.investigations/**/criteria*.md` 得 4 份（**全量非抽样**，须在产物里写清覆盖面）；
  2. **抽边**：对每份判据，枚举其「观测对象 / 前置事实 / 所引结论」；
  3. **边分类（本方法的核心判据）**：边指向**外部事实** ⇒ 无环风险；边指向**另一条判据的结论** ⇒ 有环风险，须判向；
  4. **判向**：对每条风险边逐条读两侧正文，确认是单向（A 引 B 的定论、B 前置不含 A）还是互引；
  5. **登记前提与盲区**：写清覆盖面（哪些 artifact 在图内）与未覆盖面（**无 criteria artifact 的历史课题**、**未来新增判据**）。

- **修复 / 如何利用（判据，MUST）**：
  1. **判据（MUST）**：判据依赖图的无环判定 MUST **区分外部事实边与判据结论边**，且只对后者判向；报告 MUST 给出「风险边逐条判向表」，不能只给结论「无环」。
  2. **「无环」是假设而非定理，MUST 显式登记**——协议 §11 审计表已自述 acyclicity 是源分析里的 *assumption*；宿主登记物 MUST 含：**覆盖面**（本轮 = 4 份 artifact + 其 verdict 引用面）/ **未覆盖面**（无 criteria 的历史课题、**未来新增判据不承担证明义务**——与 #143「未来代码不承担证明义务」同形）/ **判定方法**（边对象分类 + 逐条判向，未用形式化图算法，符合协议「cheap enumeration」要求）。
  3. **「活跃」图 = 当前状态图**：被 §15.4 取代收口的边属**历史边**，不承载强制，枚举时 MUST 按当前状态重判，不得把历史上出现过的引用一律计入。
  4. **新增判据 = 重跑枚举的触发点**——新判据可能首次引入环；落地动作 = 把「新增 criteria artifact 时重跑 ≤1 轮枚举」写进判据预登记流程（本工作区当前尚无此自动触发，属待办）。
  5. **主要盲区（自陈）**：存在**未在 artifact 中写明的隐式依赖**（某判据的 check 动作引用了他课题的采集产物但未登记）——此类边本枚举无法发现，判定须修正；这与论文「acyclicity 是 assumption 而非 theorem」同性质，**必须写进登记产物而不是留在心里**。
- **配套（原候选 F，合并至此；中-低价值，仅作形态记录）**：上游 v2.6 提供触发覆盖审计工具 `dsh/tests/audit_trigger_coverage.mjs`（**未接 selfcheck、作者自标 Unverified、实测假阳性率 1.1%**）。本工作区全仓试跑（约 90 课题）得 **1 MISSING（exit 1）**，命中 `v5-residual`「存在 candidate 状态但无审查型产物」；**核实为假阳性**——该记录自述「candidate 已 judge：有条件 PASS 260902-04」，审查产物存放在**兄弟课题目录**（260902-04）而非课题自身目录。⇒ **判据（形态）**：**按目录边界扫描的覆盖工具，遇到「产物跨目录存放」必假阳性**；接线前 MUST 先解决「产物归属识别」（如按 verdict 的 topic 标签反查），或提供课题内 `DECLARED-SKIP(reason)` 显式豁免。**处置建议：保持只读报告工具、不接入 selfcheck**（上游自标 Unverified + 本仓 FP 已复现）。

- **家族索引**：**#118（自证行硬门禁）/ #16x（silently-green 门——同属门禁家族：本条为「工具未接线 + Unverified」的采用门槛）**、#143（全称否定断言 MUST 声明「未来代码不承担证明义务」——本条「未来判据不承担证明义务」同形）、#161（跨臂 world 身份——同属「前提的成立条件本身须核对」）、#137（引用外部锚须属主工具自核——同属引用完整性家族）。

---

### 发现 #16x+4（暂标；最高价值）: 逆登记 MUST NOT 为了表格好看而编造「可还原」——「显式不可逆 + 理由」才是合规形态（260917-05，§9.8 首实例）

- **发现时间 / 发现者 / 置信度 / module**：260917-05；主会话执行 B3/B4（§9.8 副作用逆登记 + 判据前置集落地）+ core.worker subagent 起草；**candidate**（本工作区**首个实例**，5 条 effect→inverse 表，其中 2 条为显式不可逆声明；协议条款一手核对 `:950-964`）；workflow-patterns / 验证副作用登记与诚实声明（**#129「判据只有满足/未满足两种终态，写不可判 MUST 先证明测量不可分辨」的 §9.8 对偶面**——本条是「逆只有可寻址/显式不可逆两种终态，禁止伪造中间态」）。
- **来源定位**：`.investigations/pbeta-260917-05/criteria-260917-05.md` 末节「验证副作用与其逆」（5 行表）+ 协议原文 `E:\PYTHON\Anchorlaw\spec\protocol-v0.22.md:950-964`（§9.8 Verification Temporality；关键句 = *"Writing an inverse MUST NOT be fabricated to satisfy the clause: an inverse that does not actually restore the prior state is worse than a declared irreversible effect, because it makes a false claim"*，以及 *"Automatic reversion is forbidden precisely because a failed round's artifacts are the more valuable evidence"*）。

- **观察（现象）**：本轮登记 5 条 in-place / derived 副作用，其中 **2 条如实声明为显式不可逆**，而不是给一个看起来能还原的逆：
  1. **删除 `run/world`（run1 前强制重新生成）** → 逆 = **显式不可逆声明**（world 为生成产物，逆 = 删除，重生成即等价；原 world 无保留价值）；
  2. **改 `server.properties` 的 `level-seed`** → 逆 = 备份件 `server.properties.bak-formprobe`（脚本 `:36-43` 首次运行前创建，可还原）**但附注**：该备份为**首次快照**、非本轮前状态 ⇒ **已声明不可逆面 = 「本轮前的 seed 值未单独留档」**。
  其余 3 条为常规合规形态（derived 产物 identity / 源码 git revert / 覆盖风险由驱动拒绝同标签日志而「机制上不发生」）。

- **根因 / 判据（机制，为什么这样才对）**：逆登记的用途是**让副作用可被 containment 核对**（judge 的三源基线要能查「有没有该登记的逆」），不是让表格齐整。**伪造逆的危害等级高于声明不可逆**：不可逆声明是**诚实的缺口**（下游知道这块没有还原路径，会据此调整处置），伪造的逆是**虚假的保证**（下游以为可还原，实际不能）——协议 rationale 明写其「makes a false claim」，属 §7 行为准则级错误。第 2 条的形态尤其值得记：**「有备份文件」不等于「备份的是本轮前状态」**——备份是在首次运行时创建的快照，之后所有轮次的改动都**不在**它的还原范围内；不把这个边界写出来，备份件就会变成一个**看起来合规、实际部分失效**的逆。

- **定位 / 怎么落地的（可复用操作）**：逐条列出本轮所有会改盘上状态的动作 → 对每条问三问：**① 它改了哪个可寻址对象？② 有没有一个盘上可寻址的逆（归档件路径 / 换标签后的新名 / 环境还原记录）？③ 若没有，缺口具体是什么（写清「哪一段状态无法还原」）？** 第 ② 问答案为「没有」时，**允许**写「显式不可逆 + 理由」，**禁止**用「重生成即等价」「再跑一次就回来了」这类**未经验证的等价性叙事**顶上（除非该等价性本身成立且被声明条件，本条 world 一条即属此类：删除 = 逆，因为对象是纯生成产物——**该理由本身是判据，不是措辞**）。

- **修复 / 如何利用（判据，MUST）**：
  1. **判据（MUST）**：逆登记 MUST NOT 为了让表格好看而编造「可还原」；**不可逆 + 理由 才是合规形态**。每个 in-place 副作用的逆只有两种合法终态——**① 盘上可寻址的逆**；**② 显式不可逆声明 + 具体缺口说明**。中间态（「大致能还原」/「重跑即可」而无条件声明）视为伪造逆。
  2. **逆为独立字段，不得并入 `source=`**（协议明文）：`source` 答「哪份记录背书」，inverse 答「副作用如何被 containment」——**两个不同事实合并会把语义与扫描门一起弄脏**。
  3. **备份件的逆效力 MUST 声明其快照时点**——「备份在首次运行时创建」⇒ 该逆的还原范围 = **首次运行前的状态**，不是「本轮前状态」；不写这一行，逆就是部分失效的。
  4. **禁止自动回退**（协议明文）：失败轮的产物证据价值高于成功轮；**逆是登记义务，不是 runtime 机制**——协议只要求「登记并可寻址」，不要求任何自动回滚。逆的**正确性是作者义务，非 runtime 保证**（协议 rationale 引用论文边界；主会话 A1 判断被 v0.22 原文引用采纳）。
  5. **落地形态（本工作区）**：采集脚本/verdict 增 `inverse` 字段（与 `source=` 并列，不合并）+ 判据 artifact 增 `preconditions:` 字段（key/expected/check，见同批 criteria-260917-05.md 首实例）。

- **家族索引**：**#129（判据终态二分——本条为其 §9.8 对偶面：逆也只有二终态）**、#27/#50（静默退化/静默销毁——伪造逆属另一类静默：**虚假保证**，比静默更坏因为它主动声明）、#143（全称否定断言 MUST 声明残留边界——同属「声明必须把边界写出来」）、#137（引用完整性——逆 = 副作用的引用面）。

---

## §2 build-tooling 侧指针（不新增独立条目，避免与 §1 重复）

- **#42 家族（JNA/tmpdir 沙箱坑）**：本轮**无**新犯（pbeta05b 的第三犯已由同批草稿 `discovered-draft.md` 落盘），本节不重复。
- **#46 / #153 家族（沙箱能力必须实测）**：§1 发现 #16x+2（升级撞沙箱边界）为其**升级窗口形态**——若主会话判定该条入 build-tooling 而非 workflow-patterns，则**并入 #46/#153 家族作补充案例**（内容不变，仅换载体）；两处 MUST 择一，禁双写（避免同一事实两个编号）。
- **安装链身份登记**：`install-manifest.yaml`（source_commit + 逐件 sha256）+ `.re-framework-manifest.yaml`（user-global）为 §1 发现 #16x 的**工具面实现**，属 RE-Framework 上游产物，CoreSwap 侧只作消费者——**不在本项目知识库重复记载工具内部结构**，只记判据（§1 已记）。

---

## §3 自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] **先过价值门**：A/B/D/E 高价值（详写判据 + 来源定位 + 家族索引）；C 中-高（错误链，五段式完整）；F 中-低（**合并**进 D 条目末尾，不单独成条，理由见 §0）；一次性数字快照（28/8/36×/37+17/~90/4 份）**不单独成条**，只作证据行。
- [x] **错误优先**：C 条五段式完整（现象含失败动作与路径 / 根因含机制 = 更安全机制扩大写入面 / 定位含「读失败路径 + 对照升级改动」可复用手法 / 修复含纪律层处置 / 教训含判错经验）；无「只记修复」条目。
- [x] **根因是机制层**，非现象复述（A = 判据语义层级错配；B = 职责按命名空间切分致副本树成补集；C = 风险转移；D = 边对象分类；E = 伪造逆 = 虚假保证）。
- [x] **定位含诊断方法/工具**（逐条均可复用：grep 新条款关键词 + 递归 sha 对账 / 读 install.ps1 参数面 + Copy 源 + selfcheck manifest 路径四步 / 读失败路径的新命名 / 边对象分类 + 逐条判向 / 逆登记三问）。
- [x] **判错经验已沉淀**：C 条「升级后首次运行 MUST 视为 escalation 窗口 / 失败路径里的新命名是第一线索」；A 条「门跑了 ≠ 门守住了」；B 条「可见性排名越高越必须被门禁覆盖」；E 条「有备份 ≠ 备份的是本轮前状态」。
- [x] **被推翻/被否决的候选有标注**：F 明标「中-低 → 合并，不单独成条」；B6 明标「未执行 → 不写入知识库」；C 条内含 ⚠️ idk（备份路径已存在时是否仍被拒，本块未单独复现）。
- [x] **写入载体正确**：通用可复用判据 → `knowledge/discovered/workflow-patterns.md`（编号由主会话定）；build-tooling 侧只留**载体择一指针**不双写。
- [x] **低价值结论未写**：对齐状态快照（副本数/版本标记分布/FP 率）只留 `.investigations/`。
- [x] **数字与一手材料一致**：17 ref-\* / 11 anchor-\* / 28 副本 / 8 drifted / 36× v0.22 / preset 37 + user-global 17（0 missing, 0 drift, 0 orphan）/ ~90 课题 1 MISSING / FP 1.1% / 4 份判据 / 5 条 effect→inverse（2 条不可逆）/ `$count -lt 17` / `:104-119` / `:56-58`/`:64-118` / 协议 `:950-964`、`:1618-1629` —— 全部逐条读文件核对，无编造、无占位符（**编号占位 `#16x` 除外，且已显式声明由主会话定序**）。
- [x] **格式与目标文件末尾现状对齐**：先读 `workflow-patterns.md`（2728 行，末条 #162）与 `build-tooling.md`（1284 行，末条为 #42 家族第三犯段）末尾再起草；追加不覆盖。
- [x] **status 全部 draft**，未改任何既有 status、未写进 `knowledge/` 或 `docs/`（应用归主会话）。
