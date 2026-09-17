```yaml
id: pbeta-260917-05:timeline-upgrade-draft
block: 260917-05
status: draft          # 追记块草稿 → versions/1.20.1/docs/10-timewise-archive.md（追加到既有 260917-05 块之内/之后）
worker: subagent (core.worker)，工作块 260917-05
append_target: E:\PYTHON\CoreSwap\versions\1.20.1\docs\10-timewise-archive.md（现 3346 行结束）
conflict_note: 该文件**已有 260917-05 块**（3325-3346 行，P-β/P-path 探针面，含 `> 承接…` 引言 + 4 条 `- ` 条目 + `- 过程产物 …` 收尾行）。本条为**同一块的第二次追记**（框架升级执行面），MUST NOT 重复建块——两方案见下，推荐方案 A
```

# 10-timewise-archive.md 追记块草稿（260917-05 · 框架升级执行面）

## 落位方案（主会话择一，**禁重复建块**）

**方案 A（推荐）：追加到既有 260917-05 块之内**——在 3346 行（`- 过程产物 .investigations/pbeta-260917-05/…` 收尾行）**之后**、作为同块的新分组插入，用一行小标题区分面：

```markdown
> **（同块追记：框架升级执行面 — Anchorlaw v0.21→v0.22 / RE-Framework v2.5→v2.6 + 论文研究落地）**
```

理由：① 标题行已写「260917-05」且本块的两面（探针 / 升级）**时间重叠、来源同一目录**（`.investigations/pbeta-260917-05/`），拆成两个同年同块编号的 `##` 会制造「两个 260917-05」的假象；② 现网体例里同块的补充以 `> 追记（<块号>）` 或 `> 2609xx-xx 追加` 形式呈现（如 3297 行「✅ **追记（260917-02）**」），追记不用新 `##`；③ 该文件 260917-04 块同样用了 `> 追记（同日晚）` 形式。

**方案 B（备选）：新建 `## 260917-05b` 块**——仅在主会话判定「升级面与探针面须块级分离」时使用；此时块标题须带 `b` 后缀以免编号撞车。

**下文给的是方案 A 的正文**（直接粘贴到 3346 行之后；若选方案 B，只需把首行小标题改成 `## 260917-05b（实际 2026-09-17；框架升级执行面…）`）。

---

## 正文（粘贴内容，方案 A）

```markdown
> **（同块追记：框架升级执行面 — Anchorlaw v0.21→v0.22 / RE-Framework v2.5→v2.6 + 论文研究落地，B1→B7）** 🔍 candidate（内容 sha256 对账 + selfcheck 五段全绿；judge 待走 / confirmed 留用户）

- ✅ **B1 技能同步（全绿）**：`pwsh E:\PYTHON\RE-Framework\dsh\scripts\install.ps1` —— **首次因 v2.6 新增的 profile patch 事务备份写 `C:\Users\NDark\.dsh\profiles\web\cordis.patch.yml.bak-ref-install` 被沙箱拒绝（工作区外）→ install 中止 → escalation 后成功**（本块最重要环境事实，见下「过程错误」）；`pwsh E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1 -Project E:\PYTHON\CoreSwap` → 11 anchor-\* 更新至 v0.22；**项目级 17 ref-\* 手工同步**（无脚本覆盖，见下）：8 个 drifted（core-artifact / core-fanout / core-judge / core-knowledge / core-plan / re-lift / ref-maintain / swe-guide）按**显式文件清单** Copy-Item（禁通配，§八.4）。**验证** = 递归逐文件 sha256 对账 `ALL GREEN (17 skills, recursive)` + 版本标记 36× v0.22 + `selfcheck.ps1` 五段 `ALL CHECKS PASSED`（新内容对账段：`preset content reconciled: 37 artifacts (0 missing, 0 drift, 0 orphan)` + `user-global content reconciled: 17 artifacts (0 missing, 0 drift, 0 orphan)`）。注：`.dsh/skills` 不入 git（本地安装产物），无提交，验证依据 = 内容 sha 对账 + selfcheck。
- ✅ **B7 AGENTS.md 基线 v0.21 → v0.22**：五处编辑（头部协议段 / skill 安装记录段 / 同步契约基线段含**宿主侧待办三项** / §〇 Anchorlaw 仓库行 / §〇.1 用户级技能行）+ 新增 §一 条款 9-12（§9.8 验证副作用与逆 / 判据前置集 / halt 即终止（PI-1 + PI-2 前提）/ 交接 claim 证据分级）+ §一.6/.7 引用更新为 v0.22 并补等价分层与无效声明清单。注：`AGENTS.md` 在 CoreSwap 被 .gitignore（本地运行时文件）→ 无提交，改动即生效。
- ✅ **B2 PI-2 前提枚举（协议 MUST 前置，首执行）**：产物 `.investigations/pbeta-260917-05/pi2-dependency-enumeration.md`——枚举 **4 份活跃判据 artifact**（K1 `c1-c6-260917-04` / K2 `cp1-light-form-260916-01` / K3 `g3-drift-basis-260917-01` / K4 `pbeta-260917-05`）及其依赖边；两条「判据/结论互引」边（K2→K3 系 G3 阈值、K4→K1 系 260917-04 新基线）**逐条判向均为单向后向引用** ⇒ **当前活跃判据依赖图无环**，PI-2 可按协议原样采用（无需改 "cycles permitted but registered"）。方法 = **边对象分类法**（边指向「外部事实」还是「另一条判据的结论」——只有后者有环风险）。登记的前提与盲区：覆盖面 = 4 份 artifact + verdict 引用面；未覆盖 = 无 criteria artifact 的历史课题 + **未来新增判据（不承担证明义务）**；未用形式化图工具（cheap enumeration 即协议要求）。
- ✅ **B3/B4 §9.8 + 判据前置集实操落地（首实例）**：`criteria-260917-05.md` 追加两节——**判据前置集**（5 项 key/expected/check：`dll_sha256` / `light_init` / `probe_armed` / `world_identity` / `seed` + 前置失效处置声明）+ **验证副作用与其逆**（5 条 effect→inverse 表，含 **2 条显式不可逆声明 + 理由**：world 删除（生成产物、删除即等价）/ `server.properties` 备份为**首次快照而非本轮前状态**）。AGENTS.md §一.9/.10/.12 为条款面；本文件为该条款在本工作区的**首个实例**。
- ✅ **B5 触发覆盖审计工具试跑（只读）**：`node E:\PYTHON\RE-Framework\dsh\tests\audit_trigger_coverage.mjs --all .investigations` —— 本课题（pbeta-260917-05）单独跑 exit 0（四项均为咨询性「未触发/需确认」）；全仓跑约 **90 课题 → 1 MISSING（exit 1）**，命中 `v5-residual`「存在 candidate 状态但无审查型产物」。**核实为假阳性**：该记录自述「candidate 已 judge：有条件 PASS 260902-04」，审查产物存放在**兄弟课题目录**（260902-04）而非课题自身目录 ⇒ 与上游实测 FP 率 **1.1%** 一致；**FP 主因 = 产物跨目录存放**（工具按课题目录边界扫描）。处置（不执行）：保持只读报告工具、**不接入 selfcheck**（上游自标 Unverified + 本仓 FP 已复现）；若未来接线，先解决「审查产物跨目录」的归属识别 + 课题内 `DECLARED-SKIP(reason)` 豁免。
- ❌→✅ **过程错误（本块升级面唯一错误链，五段式）**：**现象** = `install.ps1`（RE-Framework）首次执行写 `.bak-ref-install` 被沙箱拒绝（路径在 workspace 外）→ install 中止；**根因（机制）** = v2.6 新增的 profile patch **事务备份**（`install.ps1:104-119`，backup→restore，注释自述 *"The patch file is host-shared state … Back up first."*，与论文 §5.2.2 Algorithm 10 同源）把写入面从「目标文件」扩到「**备份路径**」——而该路径是**首次出现的新对象**，其可写性从未被历史命令验证过 ⇒ **「更安全」的机制引入新的环境前提，是风险转移而非纯收益**；**定位** = 读失败路径，其命名（`.bak-ref-install` 后缀）是升级前不存在的 ⇒ 先怀疑**新机制**而不是**老命令回归**；辅证 = 上游源码定位到新增段；**修复** = escalation 后重跑成功 + **纪律层**：升级后首次运行 MUST 视为 **escalation 窗口**（不放进无人值守/批处理链）；对上游每个新增「更安全」机制 MUST 问「它新增了哪些写入对象」并登记（本条登记物 = `cordis.patch.yml.bak-ref-install`）；升级后 MUST 检查「失败时是中止还是静默降级」（本例中止是**可接受**的——静默降级的「更安全」机制比没有更危险）；**教训** = **「老命令照跑」不成立**（命令没变、环境前提变了）——沙箱类失败先核「这次多写了什么」，不先怀疑权限配置回归。⚠️ **idk**：备份路径**已存在**时 `Copy-Item -Force` 覆盖是否仍被拒（创建新文件 vs 覆盖已有文件的沙箱策略差异）本块未单独复现。
- 📌 **通用模式 → 本轮新增五条可复用判据**（草稿见 `.investigations/pbeta-260917-05/knowledge-drafts/`，编号由主会话定）：① **门禁只判「存在/数量」= silently-green 门**（28 个目录一直绿而 8 个 ref-\* 内容 drift；上游 v2.6 恰把 selfcheck 第 3 段从 `$count -lt 17` 升级为逐件 sha256 内容对账，其注释自述与 preset 行解析门禁属**同一失败类**："a gate that silently goes green when it cannot execute is the same failure class as the incident it guards against"）；② **多副本安装树的「无脚本覆盖面」**（RE-Framework `install.ps1` **无 `-Project` 参数** ⇒ 项目级 17 ref-\* **无脚本写、无门验** = 漂移黑洞，而它恰是 Rank 100 最高优先命中的副本；判据 = 逐副本问「谁写、谁验」，答不出「谁验」即黑洞；覆盖面按**副本树**而非命名空间核对）；③ **逆登记 MUST NOT 伪造「可还原」**（§9.8：不可逆 + 理由才是合规形态）；④ **判据依赖图无环判定 MUST 分类边对象**；⑤ 配套 FP 形态：**按目录边界扫描的覆盖工具遇跨目录存放产物必假阳性**。
- 🔍 **遗留 / 待办（承接 AGENTS.md 宿主侧待办三项）**：① **PI-2 采用前的枚举已做（B2），但「新增 criteria artifact 时重跑枚举」尚无自动触发**——落地动作待排（PI-2 的 fan-out 汇聚纪律「`.bN` 候选在汇聚前不得被单独引用为结论」亦待写入 fan-out 收尾）；② **判据 artifact 的 `preconditions:` 字段**已在 criteria-260917-05.md 首实例化，采集脚本/verdict 的 `inverse` 字段**尚未全量接线**；③ **B6 项目侧接管面契约**（b1 透镜 B1/B2/B3 = 生产代码副作用面，协议不覆盖）**未执行**，仅出立项建议（优先级建议 B6-1「开关生效机械对账」：`build.gradle` 约 100 行手工 `-P`→`-D` 映射 × 各处 `System.getProperty` 消费点两份事实源零机械核对，#8/#19/#47/#56 四家族已致灾；与 v0.22 §1.3「presence ≠ satisfaction」同源）。三项均需走 core-plan Phase 0。
- 📌 **产物路径**：`.investigations/pbeta-260917-05/{upgrade-execution-record.md, pi2-dependency-enumeration.md, criteria-260917-05.md（末两节）, knowledge-drafts/（本轮 subagent 草稿三份）}` + `.investigations/paper-2608-25512/remaining-after-upgrade.md`（升级后复核：已吸收 vs 待执行）+ 上游一手 `E:\PYTHON\RE-Framework\dsh\scripts\{install.ps1, selfcheck.ps1}` 与 `E:\PYTHON\Anchorlaw\spec\protocol-v0.22.md`（§9.8 :950-964 / §15.4 PI-1·PI-2 :1606-1629）。
```

---

## 自检

- [x] **先读文件尾部确认**：该文件 3346 行结束，**已有 260917-05 块**（3325-3346 行，P-β/P-path 探针面）——本条**不重复建块**，给两方案（推荐 A：块内追记；B：`260917-05b` 新块），并说明理由；两条路径正文均可直接粘贴。
- [x] **内容聚焦「框架升级执行」这一面**，不重述探针面（P-β/P-path、46 inputDiff、SELFCERT 三 boot 等**不复写**，只在「承接」处不涉及）。
- [x] 体例对齐现网：`## <块号>（实际 …；<主题>）<状态标记>` + `> 引言` + `- ✅/❌→✅/📌/🔍 条目` + `- 📌 产物路径`；追记用 `> **（同块追记：…）**` 小标题（同 260917-04 块的 `> 追记（同日晚）` 体例）。
- [x] 状态标记：升级面无 confirmed（用户未拍板），标 🔍 candidate 并注明「judge 待走 / confirmed 留用户」；错误链标 `❌→✅`（已处置）且保留 idk。
- [x] **数字逐条一手核对**：8 drifted 名单 / `ALL GREEN (17 skills, recursive)` / 36× v0.22 / preset 37 + user-global 17（0/0/0）/ 4 份判据 K1-K4 / 5 条 effect→inverse（2 不可逆）/ ~90 课题 1 MISSING / FP 1.1% / `install.ps1:104-119` / `selfcheck.ps1:56-58` 与 `:64-118` / 协议 `:950-964`、`:1606-1629` —— 全部读文件核对，无编造、无占位符。
- [x] **§15.4 无关**：本轮无结论被取代（B2/B3/B5 为首实例或试跑），未写取代记录；不涉及任何既有 status 变更。
- [x] status: draft；未写入 `versions/1.20.1/docs/`（应用归主会话）。
