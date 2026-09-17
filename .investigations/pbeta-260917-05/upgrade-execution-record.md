---
id: pbeta-260917-05:upgrade-execution-record
block: 260917-05
status: draft
purpose: 框架升级后（Anchorlaw v0.22 / RE-Framework v2.6）待执行项 B1-B6 的执行记录
executed: B1 / B7 / B2 / B3 / B4 / B5 已执行；B6 仅出立项建议（未执行，待用户定优先级）
---

# 升级后执行记录（260917-05）

## B1 技能同步（已完成，全绿）

- `pwsh E:\PYTHON\RE-Framework\dsh\scripts\install.ps1`：首次因 profile patch 事务备份写入 `C:\Users\NDark\.dsh\profiles\web\cordis.patch.yml.bak-ref-install` 被沙箱拒绝（工作区外）→ escalation 后成功。**这是 v2.6 新增的事务备份首次在生产命令上撞沙箱边界**，记录在案。
- `pwsh E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1 -Project E:\PYTHON\CoreSwap`：11 anchor-* 更新至 v0.22。
- **项目级 17 ref-\* 手工同步**（无脚本覆盖，B1 复核发现的真实缺口）：8 个 drifted（core-artifact / core-fanout / core-judge / core-knowledge / core-plan / re-lift / ref-maintain / swe-guide）按显式文件清单 Copy-Item。
- **验证**：递归逐文件 sha256 对账 —— `ALL GREEN (17 skills, recursive)`；版本标记 36× v0.22；`selfcheck.ps1` 五段 `ALL CHECKS PASSED`，其中**新内容对账段**报告 `preset content reconciled: 37 artifacts (0 missing, 0 drift, 0 orphan)` + `user-global content reconciled: 17 artifacts (0 missing, 0 drift, 0 orphan)`。
- 注：`.dsh/skills` 不入 git（本地安装产物），故无提交；验证依据 = 内容 sha 对账 + selfcheck。

## B7 AGENTS.md 基线 v0.21 → v0.22（已完成）

五处编辑：① 头部协议段落（v0.22 五条款摘要）；② skill 安装记录段（d6d2ea7 / manifest / 内容对账）；③ 同步契约基线段（逐节 diff 数据 + **宿主侧待办三项**）；④ §〇 Anchorlaw 仓库行；⑤ §〇.1 用户级技能行。
**新增 §一 条款 9-12**：§9.8 验证副作用与逆 / 判据前置集 / halt 即终止（PI-1 + PI-2 前提）/ 交接 claim 证据分级。同时把 §一.6/.7 的 v0.20 引用更新为 v0.22 并补等价分层与无效声明清单。
注：`AGENTS.md` 在 CoreSwap 被 .gitignore（本地运行时文件，同 NEXT_SESSION.md 性质）→ 无提交，改动即生效。

## B2 PI-2 前提枚举（已完成，协议 MUST）

产物：`.investigations/pbeta-260917-05/pi2-dependency-enumeration.md`
- 枚举 4 份活跃判据 artifact 及其依赖边；逐条判定两条「判据/结论互引」边的方向（K2→K3 系、K4→K1 系），均为**单向后向引用**。
- **判定：当前活跃判据依赖图无环** ⇒ PI-2 可按协议原样采用（无需改为 "cycles permitted but registered"）。
- 登记的前提与盲区：覆盖面 = 4 份 artifact + verdict 引用面；未覆盖 = 无 criteria artifact 的历史课题与**未来新增判据**（未来不承担证明义务）；未用形式化图工具（cheap enumeration 即协议要求）。

## B3/B4 §9.8 + 判据前置集实操落地（已完成首实例）

产物：`.investigations/pbeta-260917-05/criteria-260917-05.md` 追加两节：
- **判据前置集**（5 项 key/expected/check：dll_sha256 / light_init / probe_armed / world_identity / seed）+ 前置失效处置声明；
- **验证副作用与其逆**（5 条 effect→inverse 表，含**两条「显式不可逆声明 + 理由」**：world 删除、server.properties 备份为首次快照而非本轮前状态 —— 按 §9.8「MUST NOT 伪造逆」的要求如实声明）。
- AGENTS.md §一.9/.10/.12 为条款面；本文件为该条款在本工作区的**首个实例**。

## B5 触发覆盖审计工具试跑（已完成，只读）

`node E:\PYTHON\RE-Framework\dsh\tests\audit_trigger_coverage.mjs --all .investigations`
- 本课题（pbeta-260917-05）单独跑：exit 0，四项均为「未触发/需确认」的咨询性输出。
- 全仓跑：约 90 课题 → **1 MISSING（exit 1）**，命中 `v5-residual`「存在 candidate 状态但无审查型产物」。
- **核实为假阳性**：该记录自述「candidate 已 judge：有条件 PASS 260902-04」，审查产物存放在**兄弟课题目录**（260902-04）而非课题自身目录。⇒ 与上游实测 FP 率 1.1% 一致；**FP 主因 = 产物跨目录存放**（工具按课题目录边界扫描）。
- 处置建议（不执行）：① 保持只读报告工具，**不接入 selfcheck**（上游自标 Unverified + 本仓 FP 已复现）；② 若未来要接线，先解决「审查产物跨目录」的归属识别（例如按 verdict 的 topic 标签反查），并在课题内用 `DECLARED-SKIP(reason)` 声明豁免。

## B6 项目侧接管面契约（未执行，出立项建议）

b1 透镜的 B1/B2/B3 属**生产代码副作用**面，协议不覆盖（协议只管验证动作的副作用）。优先级建议（按证据强度）：
1. **B6-1 开关生效机械对账**（b1 的 B3）：`build.gradle` 约 100 行手工 `-P`→`-D` 映射 × 各处 `System.getProperty` 消费点，两份事实源零机械核对（#8/#19/#47/#56 四家族已致灾）。可行性最高（纯静态 grep + 生成对照表），且与 v0.22 §1.3「presence ≠ satisfaction」同源。
2. **B6-2 接管面读写声明表**（b1 的 B2 + 主会话 A2 confinement）：为 light/density/carver/surface/feature 各接管面登记 `reads:`/`writes:` 数据域，采集时机械核对「实际触达 ⊆ 声明」→ 直击 #98/#110/#102 三族（生产门控下读取恒空使修复 no-op）。
3. **B6-3 接管副作用成对契约**（b1 的 B1 + 主会话 A1）：为每个接管开关登记「回退等价于什么」（inverse vs compensation），当前默认「关了就是回退」不成立（盘上 world 已写入内容不回退）。
- 三项均需走 core-plan Phase 0；建议 B6-1 优先（成本最低、与已致灾家族直接对应）。
