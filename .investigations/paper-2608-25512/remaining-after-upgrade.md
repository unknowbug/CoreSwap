---
id: paper-2608-25512:remaining-after-upgrade
block: 260917-05
status: draft
worker: 主会话
purpose: 框架升级后（Anchorlaw v0.22 / RE-Framework v2.6）复核论文改进建议——哪些已被上游吸收、哪些仍需在本工作区执行
upgrade_state:
  anchorlaw: v0.22 @8313097（2026-09-17 20:33，五条款吸收）
  re_framework: v2.6 @d6d2ea7（2026-09-17 22:02，吸收 v0.22 + 论文研究）
  user_global_skills: 已更新（36× v0.22，ref-* 17 + anchor-* 11）
  project_skills: **陈旧**（28 副本：13× v0.20 + 5× v0.21，新条款标记零命中）
  coreswap_agents_md: **基线仍写 v0.21**（待 bump）
verified_by: 主会话一手核对（git log / 技能正文版本 grep / install.ps1 参数 / 协议 §9.8/§15.4 PI 正文）
---

# 升级后仍待执行项（260917-05 复核）

## A. 已被上游吸收 —— 不再需要我执行（对照我原建议编号）

| 我的原建议 | 上游落点 | 状态 |
|---|---|---|
| b3 建议 1：验证的 temporality（副作用+逆） | Anchorlaw v0.22 **§9.8 Verification Temporality** | ✅ 已为规范条款（含「逆登记不并入 source=」「自动回退被禁止」两条精确化） |
| b3 建议 2：判据前置集 | v0.22 **§15.1/§15.4 criterion preconditions**（suspended / premise-expired / status 永不自动改） | ✅ 已为规范条款 |
| b3 建议 3：分层等价门 | v0.22 **§9.7 equivalence tiers**（E1/E2/E3 + key 集 S + 无效声明清单） | ✅ 已吸收（re-lift 技能同步） |
| b3 建议 4：流程不变量 | v0.22 **§15.4 PI-1**（halt is terminal，必须机械信号）；PI-2 登记为 **unverified** | ✅ PI-1 条款化；**PI-2 需宿主自证前提（见 B2）** |
| b3 建议 5：引用完整性 | v0.22 第五条（reference integrity） | ✅ 已吸收 |
| b2 建议 1：安装产物身份 | RE-Framework v2.6：`install-manifest.yaml` + `.re-framework-manifest.yaml`；selfcheck 第 3 段改为内容对账（MISSING/DRIFT/ORPHAN + 非零退出） | ✅ 已实现（本工作区需重跑安装以生效） |
| b2 建议 2 最小子项：profile patch 事务备份 | v2.6：`install.ps1:104-119` backup→restore | ✅ 已实现 |
| b2 建议 4：触发点证据谓词 | v2.6：`dsh/tests/audit_trigger_coverage.mjs`（假阳性率 1.1%，**Unverified、未接入 selfcheck**） | ⚠️ 工具已存在但未接线（见 B5） |
| b2「不建议 6」/b1「B6」/b3 排除项 | 上游均未吸收（一致） | ✅ 无冲突 |
| 主会话 A1（逆的正确性是作者义务，非 runtime 保证） | **被 v0.22 §9.8 rationale 原文引用** | ✅ 判断被采纳 |

## B. 仍需执行（本工作区 / 我这一侧）

### B1【立即可做】项目级技能副本同步到 v0.22 + 重跑安装
- 现状实证：`E:\PYTHON\CoreSwap\.dsh\skills` = 28 个，版本标记 13× v0.20 + 5× v0.21，`§5.6|preconditions|evidence grade|suspend` **零命中** → 陈旧。
- **脚本覆盖面实证（本次复核新发现）**：RE-Framework `dsh/scripts/install.ps1` **无 `-Project` 参数**，只写 user-global + preset（`:1-12, 137-146`）；Anchorlaw `-Project` 只写 **11 个 anchor-\*** 到项目 `.dsh/skills`（`:38-55`）。⇒ **项目级 17 个 ref-\* 副本无任何脚本覆盖**（正是 b2 建议 1 指出的缺口，AGENTS.md「5 技能副本本日补齐」= 手工步骤）。这正是 v2.6 manifest 对账要暴露的 ORPHAN/DRIFT 面。
- 动作（三步）：① `pwsh E:\PYTHON\RE-Framework\dsh\scripts\install.ps1`（host）② `pwsh E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1 -Project E:\PYTHON\CoreSwap`（11 anchor-*）③ 手工同步 17 ref-* 到项目 `.dsh/skills`（`Copy-Item` 明确清单，禁通配符防混入——§八.4）+ `selfcheck.ps1` 五段全绿。
- 风险：低；v0.22 技能正文已变（anchor-test 13 行 / anchor-judge 8 行 / anchor-write 5 行 diff）。

### B2【协议强制前置，MUST】PI-2 的前提枚举（≤1 轮）
- v0.22 §15.4 明写：**hosts adopting PI-2 MUST first perform a cheap enumeration of the active criteria dependency graph (≤1 round) and register the result**；有环则条款改为「cycles permitted but registered」。
- 动作：枚举当前活跃判据的依赖图（criteria artifact ↔ 所依赖的 source/前置事实），登记结果到 `.investigations/`。
- 注：这是**唯一一条协议点名要求宿主先做验证**的项，不做则不能采用 PI-2 的 fan-out 汇聚强制。

### B3【纪律落地】§9.8 副作用/逆登记进入本工作区实操
- 动作：CoreSwap `AGENTS.md` §一 增一条（验证副作用与逆：in-place → 归档/换标签/环境还原记录，或显式不可逆声明）；后续采集脚本/verdict 落地该字段（**独立字段，不并入 source=**）。
- 依据：v0.22 §9.8 + 三起真实证据灭失（#144/#146/K2-D5）。

### B4【纪律落地】判据前置集 + 交接证据分级
- §15.1/§15.4：判据 artifact 增 `preconditions`（key/expected/check）；本项目 SELFCERT 硬门已是满足性检查 → 提升为判据字段。
- §1.3（v2.6 吸收）：交接 claim 分 numeric（源可复算则可继承）/ qualitative（**MUST 重采样原始日志**，#162）/ anchor（**MUST 用属主工具自核**，#137）三档 → NEXT_SESSION 与 verdict 交接按此分级。

### B5【需验证】触发点覆盖审计工具接线
- v2.6 提供 `audit_trigger_coverage.mjs`（假阳性 1.1%），但**未接入 selfcheck、标 Unverified**。
- 动作：在 CoreSwap 上试跑一次（只读），看误报是否可接受；再决定是否接入工作区自检。**不建议**直接接入 selfcheck（作者自己标注为 Unverified）。

### B6【项目侧，论文 b1 透镜的未覆盖项】CoreSwap 接管面契约（原建议中唯一没被上游吸收的一类）
- b1 的 B1/B2/B3 是**项目代码层**建议（mixin 写回无逆路径 / light 依赖写死检查 / `-P`→`-D` 映射双事实源），协议层**不覆盖**（协议只管「验证动作的副作用」，不管「生产代码的副作用」）。
- 仍是 open：可作为**独立课题**立项（走 core-plan），或按价值排序逐条做。其中 B3（开关生效机械对账）与 v0.22 §1.3「presence ≠ satisfaction」（#162）同源，优先级最高。

### B7【文档同步】CoreSwap AGENTS.md 基线 v0.21 → v0.22
- 现状：AGENTS.md 顶部与「与 Anchorlaw 版本同步契约」仍写 v0.21 / RE-Framework 4895220。
- 动作：按同步契约重核（`git log` 对齐 Anchorlaw 8313097 + RE-Framework d6d2ea7），更新版本号、条款引用（§9.8/§15.4 PI-1/§1.3）与技能提交号。

## C. 建议执行顺序

1. **B1**（同步技能，使后续一切工作在 v0.22 语义下进行）→ 2. **B7**（AGENTS.md 基线对齐，含新条款引用）→ 3. **B2**（PI-2 前提枚举，协议 MUST）→ 4. **B3/B4**（纪律落地：副作用逆登记 + 判据前置集 + 交接分级）→ 5. **B5**（试跑审计工具）→ 6. **B6**（项目侧接管面契约，另立课题，需用户拍板优先级）。

## D. 诚实声明

- 本文件全部结论基于主会话一手核对（git/文件 grep/协议正文），未执行任何安装或修改（B1-B7 均**未执行**）。
- 未核：项目级技能重装后 Anchorlaw `-Project` 模式与 RE-Framework 无参安装的**叠加顺序**是否产生副本覆盖冲突（install.ps1 无 -Project 参数，项目副本由 Anchorlaw 侧管理 + RE 侧副本来源待核）——执行 B1 时先核。
