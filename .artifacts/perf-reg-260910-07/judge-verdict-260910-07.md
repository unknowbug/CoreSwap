---
status: draft
kind: review
role: judge（core-judge / anchor-judge §15.4 隔离审查者）
区块: 260910-07（Java mod 工程迁出 runtime/）
审查对象:
  - .investigations/000-架构设计/架构计划-260910-07.md
  - .artifacts/perf-reg-260910-07/verdict-260910-07.md（candidate）
  - .investigations/perf-reg-260910-07/migration-errors-260910-07.md
  - .investigations/perf-reg-260910-07/cmd-output/{equivalence-260910-07.txt,ignore-checks-260910-07.txt}
  - .tmp/perf-reg-260910-07/{results.txt,logs/oa-r1.log,logs/oa-r2.log,run_arms_1201.ps1}（驱动/原始运行证据副本，不入库）
  - 工作区实际状态：versions/<ver>/java/**、runtime/<ver>/java/**、.gitignore、AGENTS.md、NEXT_SESSION.md、.artifacts/index.yaml、git HEAD/worktree
审查时点锚: 初审快照 23:00:11（HEAD=d5151a1）/ 复审 23:03（HEAD=43076b3）/ **收尾核验 23:06（HEAD=b8f91b4）** —— 审查期间主会话连推 3 次提交并应用了 C1-C12（见 §0/§0.1/§0.2）
结论等级: PASS-with-conditions（C1-C12 已闭合并经我逐点复核；**剩 must-fix = C13**（两支决定性 `.log` 仍因 `.gitignore:7 *.log` 未入 git）、**will-fix = C14**（等价性脚本 3 行假绿门禁）；二者修完本 judge 侧无阻塞）
推荐状态: 保持 candidate；C13/C14 修复后**可建议人类拍板 confirmed**（confirmed 只能由人类授予，且受 verdict 自身实机前提约束）
---

# judge 审查意见 — 260910-07「Java mod 工程迁出 runtime/」

> 本角色只出审查意见，**未修改** verdict / index.yaml / errors 台账 / 计划 / 证据文件的任何 status 字段。
> 沙箱内**无构建/运行授权**：本审查以「读一手文件 + 自行重算」为限，凡未核项在 §9 逐条声明。
> **审查时点固定声明**：本次审查与父会话收尾**并发**进行 —— 审查期间观测到 HEAD 由 `bd23b9a`(22:56:59) 被 amend 为 `d5151a1`(22:57:13，.gitignore 回调白名单链 6 行)，其后 `.artifacts/index.yaml`(22:57:47)、`NEXT_SESSION.md`(22:58:24)、`knowledge-draft-260910-07.md`(23:00) 与 `cmd-output/` 补档(23:00–23:02) 陆续落盘。**初审快照 = 23:00:11（HEAD=d5151a1）；复审定稿快照 = 23:02**，§1/§2 的「不一致/未覆盖」描述皆为**初审快照**下的实况，现状见 §0。若其后 `.gitignore`/verdict/证据文件再变动，见 C10。

## 0. 复审补记（审查期间主会话已应用的部分 —— 均已由本 judge 实测核验，非转述）

审查进行到 §1-§7 后、写盘时重取状态，发现主会话**在我出意见前已自行应用**了若干条件，故逐条实测复核（结论：数据层面全部为真）：

| 原条件 | 主会话动作（23:00–23:02） | ★ 我的核验 | 处置 |
|---|---|---|---|
| **C1（blocking）** | 把 `results.txt`、`logs/oa-r{1,2}.log(.err)`、`run_arms_1201.ps1`、`rcon_one.py` 复制进 `.investigations/perf-reg-260910-07/cmd-output/`，并新增 `MANIFEST-sha256.txt`（8 行）；verdict 头部 L6 已补「复审补档」说明 | **5 个归档文件与 `.tmp/` 原件 sha256 逐一 MATCH**（results.txt `48b5d616…`、oa-r1.log `a2a9c016…`、oa-r2.log `ca793c10…`、两个 .err `ef72fc6c…/f1465f82…`）；MANIFEST 在场；`NEXT_SESSION.md:6` 的路径引用**现为真** | ✅ **闭合**（blocking 解除） |
| **C3** | verdict §7.5 增补「**未覆盖**：… **1.21.6 的运行回归**（V2 只跑 1.20.1 一臂）」 | 与实测一致（1.21.6 `run/` 的 world mtime 19:36 早于迁移、无 1.21.6 臂日志） | ✅ **闭合**（已显式登记为缺口） |
| **C4** | `ignore-checks-260910-07.txt` 增补**第 9 格**（两版本 `worldgen-data/data/**/overworld.json`）+ **`git ls-files` 计数段**（src 49/49、worldgen-data 1019/1739、`build\|run\|.gradle\|native` 已跟踪 **0**、含 `0fc44d3` 与 11.1 天）；verdict §3 V4 增补第 9 格说明 | 两文件确在库（`git ls-files` 命中）；`build/run/.gradle/native` 四类 **0** 复核一致；白名单链（`.gitignore:109-110`）我另用假想新文件实测「未忽略」 | ✅ **闭合**（仅 §2 的 .gitignore 行仍未列 `!` 两行，属文字完整性 → 并入 C11） |
| — | verdict §1 新增**离库窗口更正**：`0fc44d3`(2026-08-30 20:29:53) → `d5151a1`(2026-09-10 22:56:59) = **11.1 天，非「5 周」** | 我用 `git log --date=iso` 独立复算 = 11 天 2h27m ≈ **11.10 天**，**更正正确** | ✅ 采纳；但 `NEXT_SESSION.md:26` 仍写「5 周无 git 史」→ **新增 C12** |
| — | verdict §3 新增「臂名复用说明」（results.txt 内 2 条 `FAIL_no_done` 与 1 条 66 s 测量行同名，引用须注轮次） | 与 `results.txt:1-3` 实况一致（此点正是我 §2.3 的备注） | ✅ 闭合 |

### 0.1 第三轮（23:01:35 提交 `43076b3` 之后）——新闭合项与新发现

| 项 | 实况 | ★ 我的核验 | 处置 |
|---|---|---|---|
| C2（等价性脚本不可复跑） | `cmd-output/equivalence_check_260910-07.ps1` 已产出：带 `-Version` 参数、自动定位旧 jar（`runtime/<ver>/java/build/libs/*.jar`，注明**勿删**）/新 jar（`versions/<ver>/java/build/libs/*.jar`）/目标 dll（1.21.6 = `worldgen1216.dll`），可原位复跑 | 脚本前 30 行实读，路径来源与证据文件口径一致；**当前为未跟踪文件**（`??`，下个提交应入库） | ✅ **实质闭合**（脚本待入库） |
| 1.21.6 三元组（原 §9「未核」项） | 脚本揭示目标 dll 名 = `target/release/worldgen1216.dll` | 我实测 sha256 = **`abd7d8893d22e030a52568ec5e4741689761229a341157ff3e6e0e4bbf792131`** = 证据 `:12/:13` ⇒ **1.21.6 三元组也已独立复算通过** | ✅ 覆盖面上移到「已核」 |
| `.gitignore` 再改（未提交）：删除 `runtime/1.20.1/java/{build,.gradle,run,data}` 四行，改由 `/runtime/` 整树覆盖 | 我用 `git check-ignore -v --no-index` 复测该四类路径（含 `src/main/java/x.java`）→ 全部仍命中 `.gitignore:92:/runtime/` ⇒ **语义等价，无回归** | ✅ 无回归 |
| `RELEASE-CONTRACT.md` 补齐新 jar 路径说明（`versions/<mc>/java/build/libs/…`） | 与计划 D5/§7.4 的「发布契约路径字样」待办对齐 | ✅ 该待办闭合 |
| **新发现（C13）**：`43076b3` 归档了 `logs/oa-r{1,2}.log.err`、`results.txt`、`MANIFEST-sha256.txt`…，但 **`logs/oa-r1.log` 与 `logs/oa-r2.log` 两支决定性日志未被 git 收录** | `git ls-files '…/cmd-output'` **无** `.log` 两行；`git check-ignore -v` 显示二者命中 **`.gitignore:7:*.log`**（`git add` 静默跳过，`.log.err` 因不匹配 `*.log` 才入库）。对照：260910-06 块 `cmd-output` 已跟踪 **176 个文件、其中 10 个 `.log`** ⇒ 本项目既有惯例是「日志入库」 | ❌ **未闭合 → C13**（正是本块要修的「忽略规则静默吃掉内容」同族；MANIFEST 列了 8 条而 git 只落 6 条，可作校验钩子） |
| `NEXT_SESSION.md:26`「5 周」 | 仍未同步为 11.1 天 | ❌ C12 仍成立 |
| verdict V2/V4/§5/V5 四处（C5/C6/C7/C8） | `43076b3` 已把 verdict 入库，但四处文字未改（V2 仍「oj」；V4 仍「8 条全对」；§5:55 仍「见 §7 提交记录」；V5 仍「记录即算」） | ❌ C5/C6/C7/C8 仍成立 |

### 0.2 收尾核验（父会话通知「条件已全部应用」后，23:04–23:06，对 HEAD=`b8f91b4`）

主会话回报 C1-C12 全部应用；我按「只读 + 定位到行」复核，**逐条实测通过**（新增 2 项）：

| 条件 | 复核对象（实读） | 结论 |
|---|---|---|
| C5 | verdict L40「证据文件现含 **33 行**」；`ignore-checks-260910-07.txt` 实为 **33 行**，新增格 = `.gradle/`(两版本,:98)、1.21.6 `build/`(:97)、1.21.6 `run/`(:99)、`content-test/run`(:100)、假想新文件 `NewFile.java`、`data/**/biome/plains.json` | ✅ 与我独立 `check-ignore` 结果一致（.gitignore 改动后行号已整体上移） |
| C6 | verdict L38 已为「（**`oa`** 异步 + `-Pchunktime=1`）」 | ✅ |
| C7 | verdict L55 已列 `d5151a1`(2876 文件) / `43076b3`(16 文件) + §8 应用记录 | ✅ |
| C8 | verdict L41 已补四步回滚清单（反向 Move-Item → 还原 .gitignore/AGENTS → 删 runDir → 复建验 sha） | ✅ |
| C9 | verdict L70 已补「新旧**两套**缓存并存」并引 `.gitignore:97-98` | ✅ |
| C11 | `run_rust_client.ps1:94` 注释改为「260910-07 起 = versions/1.20.1/java；运行环境在 runtime/1.20.1/java/run」 | ✅ |
| C12 | `NEXT_SESSION.md:26` = 「**11.1 天**（`0fc44d3` 08-30 20:29 → `d5151a1` 09-10 22:56）」+ 同族更早一犯 | ✅ |
| 顺手项 | `RELEASE-CONTRACT.md:21` 模板注记新路径；`.gitignore` legacy 四条已删、`/runtime/`(:92) 整树覆盖（我复测 build/run/data/src 仍全部 IGNORED，**无回归**） | ✅ |
| **C2** | `equivalence_check_260910-07.ps1`（67 行）全读 + 主会话已在两版本原位复跑（1077/1797、0/0/0、整包 sha 同值、TRIPLE MATCH、exit 0） | ✅ 脚本可用；**但见下面 C14 的两处门禁缺陷** |
| **C13** | `git ls-files '…/cmd-output'`（HEAD=`b8f91b4`）仍**只有 `.log.err`，无 `oa-r{1,2}.log`**；`check-ignore` 仍命中 `.gitignore:7 *.log` | ❌ **仍未修复 → must-fix** |
| **C14（新）** | `equivalence_check_260910-07.ps1`：L39-40 依赖 `$env:JAVA_HOME\bin\jar.exe` 但**不校验**（若未设/失效 → 解包为空 → `entries old=0 new=0`，L52 打印 0/0/0，L67 判据全为假 ⇒ **exit 0 + `[OK] PASSED` 假绿**）；L65 `TRIPLE MATCH` **未纳入 L67 退出判据**（三元组不符仍 exit 0） | ❌ **will-fix（3 行）** |

**收尾结论**：C1-C12 全部闭合（含我此前未核的 1.21.6 三元组已补核通过）；剩 **C13（证据入库，must-fix，1 条命令）** 与 **C14（脚本假绿门禁，3 行）**。二者修完，本 judge 侧对「迁移成立 + 等价性门」**无阻塞**；是否 confirmed 由人类拍板（并受 verdict 自身的实机前提约束，不属本 judge 判定范围）。

**仍成立的条件**（收尾核验 §0.2 后）：**must-fix = C13**（两支决定性 `.log` 仍在 git 之外）、**will-fix = C14**（等价性脚本的两处假绿门禁，3 行）；C15-info（见下）。已闭合：C1、C2、C3、C4、C5、C6、C7、C8、C9、C11、C12（+ 1.21.6 三元组补核、`.gitignore` 再改无回归、RELEASE-CONTRACT 补齐、V5 回滚四步、§8 应用记录）。

| # | 级别 | 位置 | 问题 | 建议动作 |
|---|---|---|---|---|
| **C15** | info | `ignore-checks-260910-07.txt` 两代行号并存；假想新文件标 `TRACKED` | ① 原始 8 格引 `.gitignore:95/100-104`，而 `.gitignore` 删 legacy 四条后行号上移（现 `/runtime/`=:92、java 四条=:97-100），同一文件里两代行号并存易误读；② 不存在的假想文件被标 `TRACKED (not ignored)`，语义应为「not ignored（不存在）」 | 在文件头加一行「行号以 260910-07 收尾版 .gitignore 为准（:92/:97-100）；早期 8 格行号为改前版本」；假想格标签改 `NOT-IGNORED (hypothetical)` |
| **C16** | info（confirmed 前必改，非阻塞） | verdict L4 + `.artifacts/index.yaml:1277` | 本审查意见已落盘（`judge-verdict-260910-07.md`），但两处仍写「**judge 未做** / judge pending」⇒ 记录与事实不符（status 仍应保持 candidate，只改这句描述） | 改为「judge 已做：`judge-verdict-260910-07.md` = PASS-with-conditions（C1-C12 已闭合；C13/C14 待修）」 |

## 1. 三源核对（spec §4 / §15.4 Judge review baseline）

| 源 | 实况 | 一致性 |
|---|---|---|
| ① 交付快照（`.artifacts/perf-reg-260910-07/`） | 初审：仅 `verdict-260910-07.md`（64 行，`状态：candidate`）；index.yaml:1273-1299 已登记 verdict/errors/plan 三条（均 candidate）。复审：本文件 `judge-verdict-260910-07.md`（status: draft）加入 | 基本一致，但见 C7（提交指针悬空） |
| ② 工作区实际状态（git HEAD + worktree） | HEAD=`d5151a1`（2876 文件，含 `versions/<ver>/java/**` 全部源码）；`git status` 仅 ` M .artifacts/index.yaml` + 未跟踪 knowledge-draft；新旧位置目录、`.gitignore`（119 行）、`AGENTS.md:138`、`run_rust_client.ps1:74`、两个 `build.gradle` 的 runDir 均与 verdict/计划声称一致 | **一致** |
| ③ 验证/回归记录 | **初审快照**：`equivalence-260910-07.txt`(15 行)、`ignore-checks-260910-07.txt`(9 行) 在 `.investigations/perf-reg-260910-07/cmd-output/`；`results.txt` 与 `logs/` **不在该目录**（实际只在 `.tmp/perf-reg-260910-07/`，被 `.gitignore:89` 忽略）→ **不一致 → C1（blocking）**。**复审快照**：主会话已于 23:00–23:02 归档 `results.txt`/`logs/`/驱动/rcon + `MANIFEST-sha256.txt`，ignore-checks 扩到 24 行 → **已一致**（§0 实测 sha256 MATCH） | 初审不一致 → 复审一致（C1 闭合） |

- 三源中 ② 与 ③ 的差异**以工作区实况为准**：我按 ③ 的引用路径查无 `results.txt`/`logs/`，改从 `.tmp/perf-reg-260910-07/` 读到原始文件并完成逐格回核（数字全部对上，见 §2）。
- 未解决噪声卡：`.tmp/perf-reg-260910-07/` 与指标无关；本块无新增噪声卡（无未解决项）。

## 2. 数字逐格回一手（★ = judge 自行重算，未采信证据文件自述）

### 2.1 V1 等价性门（决定性）

★ 我独立用 `[System.IO.Compression.ZipFile]` 解两支 jar、逐条目算 sha256，并用 `Get-FileHash` 重算整文件 sha：

| 项 | 证据文件值 | ★ 我重算值 | 结论 |
|---|---|---|---|
| 1.20.1 条目数（非 META-INF 文件条目） | 1077/1077（`:2`） | **1077/1077**（zip 总条目 1147 = 1077 文件 + 68 目录 + 2 META-INF） | ✅ 一致 |
| 1.20.1 only-old / only-new / content-diff | 0/0/0（`:2`） | **0/0/0**（含目录条目口径亦为 0） | ✅ 一致 |
| 1.20.1 jarsha 前后 | `297680e796b64c50…`（`:3,:4`） | **`297680e796b64c50244bff80610f7b77ccbfbcd10104c7d26c9cc23ade9e35cb`（旧=新，全 64 位逐字符相同）** | ✅ 一致 |
| 1.21.6 条目数 | 1797/1797（`:9`） | **1797/1797**（总条目 1908 = 1797 + 109 目录 + 2 META-INF） | ✅ 一致 |
| 1.21.6 only-old / only-new / content-diff | 0/0/0（`:9`） | **0/0/0** | ✅ 一致 |
| 1.21.6 jarsha 前后 | `16d5e5e7adf0780c…`（`:10,:11`） | **`16d5e5e7adf0780c74d4be60ac85298ec9bfcd45d347f5fcf40b1f4fe77bfc8e`（旧=新）** | ✅ 一致 |
| 被测文件可定位性 | 证据文件**未写**路径 | 旧 = `runtime/1.20.1/java/build/libs/coreswap-1.20.1-1.0.28.jar`（ctime 21:45:41 / mtime 22:03:34）；新 = `versions/1.20.1/java/build/libs/coreswap-1.20.1-1.0.28.jar`（**ctime 22:45:46**）；1.21.6 = `coreswap1216-1.21.6-0.1.0.jar`（旧 mtime 19:58:39 / 新 ctime 22:54:12） | ✅ 新 jar 为**真重编**（ctime 在迁移后），非复制旧物 |

> 口径注记：证据文件的 1077/1797 等于**排除 META-INF 且排除目录条目**后的文件条目数（我按目录条目口径得 1145/1906，其差 68/109 恰为目录条目数）。verdict §3「逐条目 sha256 全等（排除 META-INF）」表述正确，但未点明「目录条目不计」，建议随 C2 一并注明。

### 2.2 V3 执行体三元组

| 项 | 证据值 | ★ 我重算值 | 结论 |
|---|---|---|---|
| 1.20.1 target dll | `597e12ed2275ada5…`（`:5`） | **`597e12ed2275ada51d1681071c552e6626fdf024fa1889df11b2b14208a9fc84`**（`target/release/worldgen.dll` 实测） | ✅ 一致 |
| 1.20.1 jar-inner dll | 同上（`:6`） | 同上（解包同值） | ✅ MATCH |
| 1.20.1 运行体 dll（第三重，运行日志实测） | 未在证据文件列出 | `oa-r2.log:120` `[CppBridge] dll=…sha256=597e12ed2275ada5…` | ✅ 三元组闭合（含运行侧） |
| 1.21.6 target/jar-inner | `abd7d8893d22e030…`（`:12,:13`） | **`abd7d8893d22e030a52568ec5e4741689761229a341157ff3e6e0e4bbf792131`**（`target/release/worldgen1216.dll` 实测，见 §0.1） | ✅ 一致（复审补核） |

### 2.3 V2 路径回归（原始运行证据）

| 判据 | verdict §3 V2 | ★ 我读到的原始行 | 结论 |
|---|---|---|---|
| 臂名 | 「跑 1 臂（**oj** 异步 + `-Pchunktime=1`）」 | 驱动 `.tmp/perf-reg-260910-07/run_arms_1201.ps1:20` 只有 `oa`（无 `oj`）；结果行 `oa-r1`/`oa-r2` | ❌ **臂名错字 → C6**（数值不受影响） |
| Chunky Processed | `4225` | `oa-r2.log:156` `[Chunky] Task finished for minecraft:overworld. Processed: 4225 chunks (100.00%), Total time: 0:00:38`；`oa-r1.log:165` … `Total time: 0:01:06` | ✅ 一致 |
| `[CHUNKTIME] inflight max` | `23` | `oa-r2.log:155` `[CHUNKTIME] n=4096 … mixin=104.51 gap=102.93 \| sumMixin=428.1s sumGap=421.6s \| inflight max=23 now=19`（末行；`results.txt:4` 的 8 个派生字段与之逐字对应）；`oa-r1.log:164` 同形 `inflight max=23 now=4`（`results.txt:3` 对应） | ✅ 一致 |
| bridge init | `stageMask=3` | `oa-r2.log:119` `[CppBridge] init seed=417950215108767439 worldgenDir=…\versions\1.20.1\data\worldgen enabled=true stageMask=3` | ✅ 一致 |
| 无 STALL | 「无 STALL」 | `results.txt` 4 行 = 2×`FAIL_no_done` + oa-r1 + oa-r2，**无 STALL 行** | ✅ 一致 |
| 66 s 离群 / 复跑 38 s 如实记录 | §3 V2 + §6 E4 双处记录 | `oa-r1.log:165`（0:01:06）+ `results.txt:3`（sec=66/wallgen=72.2/cores=6.58）、`oa-r2.log:156`（0:00:38）+ `results.txt:4`（sec=38/wallgen=41.3/cores=11.53） | ✅ **两值均如实记录且与原始日志逐格一致** |
| world 仍在 runtime/ 下 | §3 V2「world 仍在 runtime/」 | `versions/1.20.1/java/run` **不存在**、`versions/1.21.6/java/run` 不存在；`runtime/1.20.1/java/run/world` LastWrite=**22:52:40**（与 oa-r2 日志尾部 `Saving chunks…22:52:40` 同秒） | ✅ 成立（且是「runDir 生效」的强证据） |
| 2×`FAIL_no_done` 行 | verdict 正文未提，§6 E1/E2 各记一次失败 | `results.txt:1,:2` 两条 `oa-r1 FAIL_no_done`（对应 E1 起进程失败 + E2 驱动路径推导失效） | ✅ 可归因，非隐瞒（建议 §3 备注一句两行失败行的来源，属 info） |

### 2.4 V4 入库/忽略双向核

| 项 | 证据文件（`ignore-checks-260910-07.txt`） | ★ 我另行执行 `git check-ignore -v` 的结果 | 结论 |
|---|---|---|---|
| 源码未忽略 | `:2`(1.20.1)、`:8`(1.21.6) src/main/java | 同（NOT-IGN） | ✅ |
| worldgen-data 未忽略 | `:3`（根级 `biome_params.json`） | 同（NOT-IGN） | ✅ |
| 构建定义未忽略 | `:4` build.gradle | 同（NOT-IGN） | ✅ |
| `run/` 忽略 | `:5` `.gitignore:102` | 同（1.20.1 + 1.21.6 均 `:102`） | ✅ |
| `build/` 忽略 | `:6` `.gitignore:100` | 同（1.20.1 + 1.21.6） | ✅ |
| `resources/native/` 忽略 | `:7` `.gitignore:104` | 同 | ✅ |
| `runtime/` 忽略 | `:9` `.gitignore:95` | 同（1.20.1 + 1.21.6 均命中 `/runtime/`） | ✅ |
| `.gradle/` 忽略 | **证据文件无此行**，但 verdict §3 V4 声称 8 条含 `.gradle/` | 我实核 `.gitignore:101:versions/*/java/.gradle/` → IGNORED；另核 `versions/*/java/*/run/`（`:103`，content-test）→ IGNORED | ⚠️ 结论成立但**证据组成与 §3 表述不符 → C5** |
| **worldgen-data/data/** 白名单链**（本块最易踩的 #24 陷阱）** | **无任何一行覆盖** | 我实测 `git check-ignore -v --no-index` 假想新文件 `versions/1.20.1/java/src/main/resources/worldgen-data/data/worldgen/zz_new.json` → 命中 `!.gitignore:110:!/versions/*/java/src/main/resources/worldgen-data/data/**` ⇒ **未忽略（新文件会自动入库）**；对照根级 `biome_params.json` 无命中 ⇒ 未忽略 | ⚠️ 机制成立（我实证），但**证据未覆盖 → C4** |
| D4 入库完整性 | 计划 D4 | `git ls-files`：`versions/1.20.1/java`=1077、`1.21.6/java`=1793；src=1070/1790；worldgen-data=**1019/1739**；`worldgen-data/data`=**1015/1735 且与磁盘计数完全相等**；`git status --untracked-files=all` 在两树内**空** | ✅ 全量入库，无遗漏 |
| 旧位置残留 | 计划/verdict | `runtime/1.20.1/java`={.gradle,.gradle-home,build,run,scripts}、`1.21.6/java`={.gradle,.gradle-home,build,run}，**无 src/ 无 build.gradle**；`git ls-files runtime/**/java`=0；旧根 **无散件文件** | ✅ 仅环境/缓存残留 |
| 历史 jar 未误删 | §7.2 | `runtime/1.20.1/java/build/libs/` 含 **1.0.17→1.0.28 共 12 支**（1.0.27/1.0.28 在位） | ✅ 成立 |
| 散件隔离 | §2 | `.tmp/legacy-runtime-260910-07/1.20.1/` = 3 × `.java`（MaterialRules/NoiseChunkGenerator/VanillaSurfaceRules）+ **58 × `.log`** + 1 个无扩展名文件（`x`） | ✅ 与 §2「58 个 stray .log + x」一致 |

## 3. 等价性门的强度表述是否恰当

- 「迁移前后 jar **逐条目 sha256 全等且 jar 整体 sha 逐字节不变**」→ **可由证据支持，且经我独立重算复核**（§2.1）。这是本块最强可得的形态，表述**未过度**（不是「行为等价」的越界表述）。
- 「执行体三元组仍 MATCH」→ 成立（dll 三重：target / jar-inner / 运行日志实测 sha 三者同值，§2.2）。
- 边界表述：§7.5 已明确「等价性门证明『jar 不变』，**不证明** dev-run 行为完全一致（已用 1 臂回归覆盖主通路）」→ **正确且必要**，无「等价性 ⇒ 行为一致」的越界。
- §4 已按 §9.7 三要素声明（载体 = loom 1.10.5 / jdk-17.0.12 / `.gradle-home`；覆盖面 = 全条目无抽样 + 1 臂；可比性 = 与 260910-06 性能数字不可混读）→ **合规**。
- 唯一措辞瑕疵：「迁移**零语义影响**」为 §1 结论句的收束语，其证据面严格等于「jar 逐字节不变」；建议加半句「（jar 级；运行时行为见 §7.5）」以与 §7.5 自洽（info，可与 C6 同批处理）。

## 4. 诚实边界（§7）覆盖度

| 真实缺口 | §7 是否覆盖 | 我的核验 |
|---|---|---|
| content-test/run 随迁 | ✅ §7.1 | 实测 `versions/1.20.1/java/content-test/run` = **21 文件 / 0.08 MB**（≈「0.1 MB」成立）；旧位置已无 `content-test` 目录；`.gitignore:103` 已忽略 |
| runtime 残留 + 旧 jar 不得误删 | ✅ §7.2 | ✅（见 §2.4） |
| 多份 gradle home | ✅ §7.3 | 实测存在 `$root/.gradle-home`、`$root/.gradle`、`runtime/<ver>/java/.gradle`、`runtime/<ver>/java/.gradle-home` —— **但漏列本块新产生的 `versions/<ver>/java/.gradle` 与 `versions/<ver>/java/build`** → C9 |
| 发布契约/工单模板路径字样 | ✅ §7.4 | 实测 `RELEASE-CONTRACT.md` **无硬编码旧路径**（用 `<绝对路径>` 占位）；`RELEASE-1.0.26/27.md` 含旧路径但属已发热工单（按历史读不改）→ §7.4 表述准确 |
| 历史归档不改写 | ✅ §7.4 | ✅（`.investigations/**`、`versions/*/docs/**` 未动） |
| **1.21.6 `runServer` 未跑（D1 半开）** | ⚠️ **未显式声明**（仅 §4.2 覆盖面隐含 1 臂 = 1.20.1） | 1.21.6 `run/` 的 world mtime = 19:36:37（**早于** 22:4x 迁移）；本块 `results.txt` 只有 1.20.1 的 oa 臂 → D1「两版本 runServer 均可跑」实际只证了 1.20.1 一侧 → C3 |
| V5 回滚「步骤自足」 | ⚠️ 只写「记录即算」，无步骤清单 | 仓库内无回滚步骤（计划 §3/§5 只有片语）→ C8 |
| 「零数据搬迁」是否严格成立 | 部分（§7.1 已声明 content-test/run 随迁） | 我扫描 `runtime/1.20.1/java/run` 下 56 个小文本配置（.properties/.json/.txt/.yml/.cfg/.toml，<2MB，排除 world/）：**无一条活配置引用旧工程路径**；仅 10+ 条 `crash-reports/*.txt` 记录里残留更早的 `E:/python/MC/...` 历史路径（历史记录，非活配置）；`run/mods/` 只有 Chunky + content-test（**无 coreswap jar** ⇒ 运行不依赖 run/ 内的陈旧构建物）。故「主 runDir 零搬迁」严格成立 → 建议把「零数据搬迁」限定为「主 runDir」，content-test/run 已由 §7.1 声明（info，并入 C11） |

## 5. 引用面清扫的独立复核

我按「活配置 vs 历史归档」独立清扫 **187 个活文件**（AGENTS.md / NEXT_SESSION.md / README×2 / Cargo.toml / .gitignore / versions/*/rust / worldgen-core / knowledge / .artifacts（除已发工单）/ scripts / protocol / .dsh / 两 java 工程树；排除 .git、target、build、.gradle、world、.investigations、.tmp、versions/*/docs）：

| 活引用点 | 计划/verdict 声称 | ★ 实测 | 结论 |
|---|---|---|---|
| `.gitignore` | 保留 `/runtime/` + 新增 java 忽略 + 白名单链 | `:95 /runtime/`、`:100-104` 五条、`:106-110` **worldgen-data/data 逐级白名单链**（119 行实况） | ✅ 已改；但 verdict §2 未描述白名单链 → C4 |
| `AGENTS.md:138` | 现役工程/运行环境路径 | 实测该行 = 「现役 Java mod 工程（源码/构建定义，260910-07 起已入库）= `versions\<ver>\java`；运行环境 = `runtime\<ver>\java\run`（runDir 相对路径指回）」 | ✅ 已改（AGENTS.md 按项目规则不入库，故不在提交内） |
| `versions/1.20.1/java/run_rust_client.ps1` | `$runJava` 指向新路径 | `:74 $runJava = "…\versions\1.20.1\java"` ✅；`:28 $vsDir = …\runtime\1.20.1\vanilla-server`（另一个合法环境，与本次无关）；**`:94` 注释仍写「切到 mod 工程（runtime）」** | ⚠️ 功能已改，注释陈旧 → C11 |
| 驱动约定 | 两变量 `$run`/`$rd` | `.tmp/perf-reg-260910-07/run_arms_1201.ps1:34` `$run = "…\versions\1.20.1\java"; $rd = "…\runtime\1.20.1\java\run"` | ✅ |
| 发布契约 | 新工单用新路径 | `RELEASE-CONTRACT.md` 无硬编码旧路径；历史工单保留 | ✅ |
| NEXT_SESSION.md | 计划 D5 列为活引用面 | **22:58:24 已更新**：`:14` 交付 jar 已改为 `versions/1.20.1/java/build/libs/…`；`:43-46` 复测口径已按新路径；`:6` 仍写证据含 `results.txt,logs/`（同 C1） | ✅（本审查开始时读到的是更早快照，已按最新版复核） |
| **其它活文件是否仍有旧路径** | — | 187 个活文件内 `runtime[\\/]1\.(20\.1|21\.6)[\\/]java` **零命中**（除上述有意保留的 runDir/vmArg/vanilla-server/hs_err 记录） | ✅ 活引用面已闭合 |

> `build.gradle` 的 `vmArg "-XX:ErrorFile=…/runtime/<ver>/java/run/hs_err_%p.log"`（1.20.1:202,207；1.21.6:201,208）指向**未搬迁的运行环境**，属正确保留，非旧引用。

## 6. 置信度纪律

- verdict 头部与 index.yaml:1277 均为 `candidate`，并显式注明「judge 未做 / 用户未 confirmed（AI 不得自授 confirmed）」→ **未被自我升格**。
- 全文检索 `confirmed` **仅 1 处**（`:4` 的「用户未 confirmed」），**无任何「已 confirmed」措辞**。
- 本审查意见不改 status（verdict/计划/台账/index.yaml 的 status 字段一律未动）。

## 7. 子角色触发纪律

- **scout 不触发**：本块是「载体已知的确定性结构迁移」，非「机制未明」大排查；计划 §2 的 F1-F10 事实表以目录列举/文件计数/build.gradle 检视/git log 为证据闭合了载体差异（我抽验 F1/F4/F5/F7/F9 均与实况一致）→ **理由成立**。唯一提示：F 表为主会话自查（非 scout subagent），在「确定性 + 有界引用面」前提下可接受。
- **fan-out 不触发**：V1 条目级 **0 差异**（我独立复算），未出现「≥2 互斥机制候选」的分叉；E1-E4 是**四个彼此独立的确定性根因**（各自有唯一错误签名：`error=267` 的 `(in directory …)` / 报错路径 / file-watcher 异常 / 复跑复现），不是同一现象的多机制竞争 → **理由成立**。
  - 条件性保留（与计划 §7 一致）：若 V1 出现条目级差异且归因不明，或 E1/E2 类启动失败在无明确错误签名的情形下再次出现，则 MUST 转 fan-out（b1 路径/编译器字节差异、b2 loom 配置、b3 生成物混入）。
- **judge 预置**：计划 §6 已预置「candidate 前 SHOULD + 收尾 MUST」两点，本次审查即该预置项的落地 → **计划期预置合规**（非事后补跑）。
- **worker（§9 声称的 Phase 2.5 等价性独立复核）**：仓库内未见独立复核记录文件；实际由本 judge 独立重算承担了该面 → 建议把「证据独立复核」记为 judge 面或补留 worker 记录（info，可并入 C2）。

## 8. 结论与编号条件

**结论等级：PASS-with-conditions。**
实质结论（迁移成立、等价性门（jar 逐字节相同）、环境原地、源码入库、活引用面闭合）**经我独立重算全部成立**；初审唯一的 §15.4 blocking 项 **C1 已在审查期间由主会话修复并经我核验（§0）**，其余条件均为 should-fix / info，不阻塞提交、不阻塞 candidate 保持。**是否推进 confirmed 仍取决于人类拍板 + 实机反馈；C5/C6/C7/C8/C12 建议在收尾提交前一并修掉。**

| # | 级别 | 位置 | 问题 | 建议动作 |
|---|---|---|---|---|
| **C1** | ~~blocking~~ → ✅**已修复** | verdict 头部 L6 + `NEXT_SESSION.md:6` | （初审）两处都把 `results.txt`、`logs/` 写成 `.investigations/perf-reg-260910-07/cmd-output/` 下的证据，但该目录当时**只有** equivalence + ignore-checks；V2 原始运行证据只在被 `.gitignore:89` 忽略的 `.tmp/` | 主会话已归档（`.tmp` 原件与归档件 sha256 逐一 MATCH、MANIFEST 在位，§0 实测）→ **无需再动作**；建议收尾提交把 `cmd-output/**` 一并入库 |
| **C2** | ~~should-fix~~ → ✅**已闭合（脚本待入库）** | `cmd-output/equivalence_check_260910-07.ps1` | （初审）等价性门只有结果、无脚本，不可原位复跑 | 主会话已产出可复跑脚本（含旧/新 jar 与 target dll 的定位规则；1.21.6 = `worldgen1216.dll`）；**该脚本当前仍是未跟踪文件**，随下个提交一并入库即闭合 |
| **C3** | ~~should-fix~~ → ✅**已闭合** | verdict §3 V2 / §4.2 / §7，对照计划 §1 D1 | D1 要求**两版本** `:build` 与 `:runServer` 均可从新路径跑；实际只跑了 1.20.1 一臂（1.21.6 `run/` 的 world mtime 19:36 早于迁移） | 主会话已在 §7.5 显式登记「未覆盖：1.21.6 的运行回归」→ 无需再动作；若要闭合 D1 则补跑 1.21.6 一臂 |
| **C4** | ~~should-fix~~ → ✅**已闭合** | verdict §2「.gitignore」行 + `ignore-checks-260910-07.txt` | （初审）本块最易踩的 **#24 目录级 prune 陷阱**所对应的白名单链（`.gitignore:106-110`）无证据覆盖 | 主会话已补 data/** 两格 + `ls-files` 计数段（我复核两文件在库、`build\|run\|.gradle\|native`=0），§3 V4 亦已说明；仅 §2 的 .gitignore 行未列 `!` 两行（文字完整性，并入 C11） |
| **C5** | should-fix | verdict §3 V4 | 文中仍写「8 条全对」且列举含 `.gradle/`，但证据文件**现为 10 格**（原 8 行 = 4 正 + 4 负、无 `.gradle/` 行；复审已补 data/** 两格 + 计数段），`.gradle/` 断言仍只由我另行核到（`.gitignore:101`） | 按证据文件现状重述（把 `.gradle/`、`content-test/run`、`data/**` 两格计入并写明总数），或直接改「13 格全对（含补充段）」 |
| **C6** | should-fix | verdict §3 V2 | 「跑 1 臂（**oj** 异步 + `-Pchunktime=1`）」——驱动里不存在 `oj` 臂，实际臂名是 `oa`（`run_arms_1201.ps1:20`；结果行 oa-r1/oa-r2） | 改为 `oa`；顺带把 §1「零语义影响」补半句「（jar 级；运行时行为见 §7.5）」 |
| **C7** | should-fix | verdict §5；`.artifacts/index.yaml:1286` | §5 写「提交：本块改动见 **§7 提交记录**」，但 §7 是「老实边界/未核项」、**无提交记录**；唯一的提交指针在 index.yaml（`提交：d5151a1（2876 文件）`，与 HEAD 一致），而 index.yaml 目前仍是 **` M`（未提交）** 状态 | verdict §5 直接写 `d5151a1`（2876 文件）；index.yaml 的 260910-07 三条登记随下一个提交入库 |
| **C8** | should-fix | verdict §3 V5 | 判据是「记录即算（步骤自足）」，但仓库内**没有回滚步骤清单**（仅计划 §3/§5 有片语），真回滚时需现场重推 | 在 §3 V5 补 3 行步骤（反向 `Move-Item` 两版本 / 还原 `.gitignore` / 还原 `runDir`）并注明「依赖 `.tmp/legacy-runtime-260910-07/` 未删」 |
| **C9** | info | verdict §7.3；`NEXT_SESSION.md:36②` | 债清单列了 4 处 gradle home，**漏了本块迁移时新建的 `versions/<ver>/java/.gradle` 与 `versions/<ver>/java/build`**（旧位置残留 + 新位置缓存，实际是 6 处） | 补进后续清理项 |
| **C10** | info | 全文 pin | 本审查与父会话收尾并发：审查期间 HEAD 由 `bd23b9a`(22:56:59) 被 amend 为 `d5151a1`(22:57:13，.gitignore 回调白名单链 6 行)，其后 index.yaml(22:57:47)/NEXT_SESSION.md(22:58:24)/knowledge-draft(23:00) 才落盘 | 若下一个提交再动 `.gitignore`/verdict/证据文件，请按「重算两支 jar sha + `check-ignore` 三条 + `git ls-files` 两树计数」做一次廉价复核，再据以推进 confirmed |
| **C11** | info | `versions/1.20.1/java/run_rust_client.ps1:94`；verdict §2「环境」行；§1 目标句 | ① 注释仍写「切到 mod 工程（runtime）」（功能已改，仅注释陈旧）；② §2 把 `scripts/` 写成两版本共有（实测 1.21.6 无 `scripts/`）且未列 `.gitignore:109-110` 白名单链；③ 「零数据搬迁」宜限定为「主 runDir 零搬迁」（content-test/run 0.08MB 已随迁，§7.1 已声明）；④ `results.txt` 里两条 `FAIL_no_done` 行建议备注来源（E1/E2）（verdict 复审已补「臂名复用说明」✓） | 逐条改字/备注即可 |
| **C12** | should-fix | `NEXT_SESSION.md:26` | 该行仍写「`0fc44d3` 的 `/runtime/` 整树 untrack …（**5 周**无 git 史）」——verdict §1 已更正为 **11.1 天**（我用 `git log --date=iso` 复算 `0fc44d3`(08-30 20:29:53) → `d5151a1`(09-10 22:56:59) = 11 天 2h27m ≈ 11.10 天，**更正正确**），但活交接文档未同步 ⇒ 同一事实两处数字互斥 | 把 `NEXT_SESSION.md:26` 的「5 周」改为「11.1 天（08-30→09-10）」；全库其余「5 周」字样已 grep 确认清零 |
| **C13** | **must-fix**（§0.1/§0.2） | `.gitignore:7` + 归档提交；`cmd-output/MANIFEST-sha256.txt` | **至 HEAD=`b8f91b4` 仍未修复**：`logs/oa-r1.log`、`logs/oa-r2.log`（V2 决定性日志）未被 git 收录——全局 `*.log` 使 `git add` 静默跳过（`.log.err` 不匹配才入库）；对照 260910-06 同目录已跟踪 176 文件 / 10 个 `.log`（日志入库是既有惯例） | `git add -f .investigations/perf-reg-260910-07/cmd-output/logs/*.log`（或加 `!.investigations/**/cmd-output/logs/*.log` 白名单），并在 MANIFEST 尾部加一行「条目数 vs `git ls-files` 检出数」自检 |
| **C14** | will-fix（3 行，建议同批） | `equivalence_check_260910-07.ps1` L39-40 / L52 / L65-67 | ① 解包依赖 `$env:JAVA_HOME` 但不校验 ⇒ 失败时 `entries=0/0/0` 仍走 L67 判据全假 → **exit 0 `[OK] PASSED` 假绿**（本项目 #23/#25/#40 家族）；② `TRIPLE MATCH`（L65）未纳入 L67 退出判据 ⇒ 三元组不符仍 exit 0 | ① L52 后加 `if ($oh.Count -eq 0 -or $nh.Count -eq 0) { Write-Output '[FAIL] empty extraction (JAVA_HOME?)'; exit 2 }`；② L67 判据末尾追加 `-or ($tSha -ne $iSha)` 并对 `$iSha` 做存在性判断 |

## 9. 覆盖面声明（我实际核了什么 / 没核什么）

**全量核（未抽样）**
1. V1 等价性：**两支 jar 的全部 zip 条目**（1147 / 1908 条）逐条目 sha256 + 整文件 sha256 重算（1.20.1 与 1.21.6）——数据来源为**工作区实况 jar 本身**，不依赖证据文件自述。
2. V3 三元组：1.20.1 侧 target dll 实测 sha + jar-inner + 运行日志 dll 行三者比对。
3. V4/D4：`git check-ignore -v` 实跑 **14 个路径**（含证据未覆盖的 `.gradle/`、`content-test/run`、1.21.6 build/run、假想新文件）；`git ls-files` 对 `versions/{1.20.1,1.21.6}/java` 及 `src`、`worldgen-data`、`worldgen-data/data` 五组计数 + 磁盘计数比对；`git status --untracked-files=all` 两树内全量。
4. V2 关键行：`results.txt` 全 4 行、`oa-r1.log`/`oa-r2.log` 的 Processed/Total time/CHUNKTIME/bridge/dll 全部命中行（含日志尾部的 world 保存时间）。
5. 结构与残留：新旧两版本 java 目录顶层全列举、`src/main/java` 与 `worldgen-data` 文件计数（49/49、1019/1739）、旧位置散件清零、`runtime` 残留目录集、历史 jar 1.0.17–1.0.28 清点、散件隔离区（3 java + 58 log + 1）。
6. 引用面：187 个活文件全量正则清扫（定义域见 §5）+ 两个 `build.gradle` 的 runDir/vmArg 行 + AGENTS.md:138 + `run_rust_client.ps1` 变量/注释 + git 侧 `.gitignore` 与 HEAD 提交内容/reflog。
7. 置信度与子角色：verdict 全文 + index.yaml:1273-1299 的 status 与注释 + 计划 §6/§7/§9 的预置项。
8. 复审补档核验（§0）：新归档的 5 个文件 vs `.tmp/` 原件 **sha256 逐一比对**（全 MATCH）；`MANIFEST-sha256.txt` 8 行在场；`git ls-files` 对 `build/run/.gradle/native` 四类 = **0**；`worldgen-data/data/**/overworld.json` 两版本确在库；「11.1 天」独立算术复算；`git ls-files` 视角的 `run_arms_1201.ps1`/`rcon_one.py` 归档在位。
9. 第三轮（§0.1）：1.21.6 `target/release/worldgen1216.dll` sha 实测；`equivalence_check_260910-07.ps1` 全 67 行实读（含逻辑缺陷定位）；`.gitignore` 再改后四类 runtime 路径 `check-ignore` 复测（全部仍忽略）；**`git ls-files …/cmd-output` 与 `MANIFEST-sha256.txt` 条目逐一对照**（发现 2 个 `.log` 缺失 → C13）；`RELEASE-CONTRACT.md` 补丁实读；`43076b3` 的 16 个文件清单与 shortstat。
10. 收尾核验（§0.2，HEAD=`b8f91b4`）：verdict L38/L40/L41/L55/L70 + §8 应用记录逐行实读；`ignore-checks-260910-07.txt` 33 行全读；`NEXT_SESSION.md:26`、`run_rust_client.ps1:91-95`、`RELEASE-CONTRACT.md:19-22`、`.gitignore`（legacy 段 + `:92`）实读；两 jar 的 `native/` 条目实列（确认脚本 L62 硬编码 `native/worldgen.dll` 对两版本都成立）；`$env:JAVA_HOME` 实查（=jdk-24，脚本未校验该依赖 → C14）。

**抽样核**
1. 计划 §2 事实表 F1-F10：抽验 F1/F4/F5/F7/F9（文件计数、dll 复制步骤、runDir DSL、settings/gradle.properties）——**未逐条复验 F2/F3/F6/F8/F10**（F2 需解包对照 class↔源清单、F3 需体量分解、F6 需读 `x` magic、F10 需 `git log --all` 追三处 commit；本次等价性门已从更上游覆盖 F2 的实质关注）。
2. `runtime/1.20.1/java/run` 配置扫描：**56 个 <2MB 的文本配置**全扫；未扫 `run/world/**` 二进制 region 与 `logs/*.gz`（无路径引用语义）。
3. `.tmp/perf-reg-260910-07/run_arms_1201.ps1` 全读，但未执行；`rcon_one.py` 未读。

**未核（附原因）**
1. **未重跑 `gradle :build` / `:runServer`**：judge 会话无构建/运行授权（且重跑会污染 runDir/环境与既有读数）；替代强证据 = 旧/新 jar 整文件 sha 逐字节相同 + 新 jar ctime 证明真重编 + 目标 dll 与运行日志 dll 同值。
2. ~~未独立复算 1.21.6 侧 dll sha~~ → **已补核**（§0.1：`target/release/worldgen1216.dll` = `abd7d889…`，与证据逐字符一致）；仍**未**核的是 1.21.6 jar 内 dll 解包复算（整文件 sha 已复算 ⇒ 内含物随之确定，冗余）。
3. **未跑 ignore-checks 的生成脚本**（该脚本仓库内不存在；`ignore-checks-260910-07.txt` 的 `TRACKED (not ignored)` 是包装层自述标签，我已用原生 `git check-ignore -v` 独立复核其结论）。
4. **未核 content-test 子工程 `:runServer` 行为**（§7.1 的「本就不影响主流程」声明未独立复验）。
5. **未分析隔离件 `x`（4.2MB，magic `33 42 47 57`）的身份**（本块按计划只做「不删、移入 `.tmp`」，不属审查判据）。
6. **未访问任何 workspace 外路径**（`E:\PYTHON\MC` 按 AGENTS.md §四铁律禁读禁写）。
7. **未做 knowledge-draft-260910-07.md（23:00 新增）的审查**：该文件在本次审查窗口后出现，属知识库落盘流程（应由 knowledge + 主会话验证），不在本 judge 项的审查对象内。

## 10. 附：审查期间的状态漂移记录（供父会话留痕）

| 时刻 | 观测 | 证据 |
|---|---|---|
| 22:56:59 | `bd23b9a` 首次提交（此时 .gitignore 内**无** worldgen-data 白名单链，但提交信息已声称有） | `git reflog --date=iso` |
| ~22:57:0x | 我读到 HEAD=`bd23b9a`、`.gitignore` 无白名单链（113 行快照） | 本会话首次 `git log -3` + `.gitignore` 读 |
| 22:57:13 | amend → `d5151a1`（`.gitignore` 回调白名单链 6 行；diff `d5151a1^..d5151a1` 中它相对 `bd23b9a` = +6 行） | `git reflog --date=iso`、`git diff --stat d5151a1 bd23b9a` |
| 22:57:47 / 22:58:24 / 23:00 | `.artifacts/index.yaml`（未提交）/ `NEXT_SESSION.md` / `knowledge-draft-260910-07.md`（未跟踪） 依次落盘 | 文件 mtime + `git status --porcelain` |
| 23:00–23:02 | 主会话在我出意见前继续推进：verdict 补「复审补档/离库窗口更正/臂名复用说明/V4 第 9 格/§7.5 1.21.6 未覆盖」；`cmd-output/` 新增 `results.txt`、`logs/`、`run_arms_1201.ps1`、`rcon_one.py`、`MANIFEST-sha256.txt`；`ignore-checks` 增 15 行；knowledge 三文件 + 两版本时间线开始改动 | `git status`（此时 verdict 已 ` M`）+ 我的 sha256 比对 |
| 23:01:35 | 提交 `43076b3`（16 文件 +879/-3）：verdict 新版、index.yaml、`cmd-output/` 归档件（**不含两支 `.log`**）、knowledge-draft、knowledge 三文件、两版本时间线 —— **本 judge 文件亦在此提交内**（当时为初审版，其后我继续改 → 现为 ` M`） | `git show --name-only 43076b3` + `git ls-files '…/cmd-output'` |
| 23:02–23:03 | `equivalence_check_260910-07.ps1` 落盘（未跟踪）；`.gitignore` 删 `runtime/1.20.1/java/*` 四行改由 `/runtime/` 覆盖（我 `check-ignore` 复测无回归）；`RELEASE-CONTRACT.md` 补新 jar 路径说明 | 三者的 `git status`/`git diff` |

> 因此：**`.gitignore` 白名单链缺失一说是 bd23b9a 的瞬态，不适用于现 HEAD**；上文 C4 只保留「verdict/证据未覆盖该链」，不主张机制缺失。
> **本意见的定稿 pin = HEAD `43076b3`（23:01:35）+ 23:03 worktree**；`43076b3` 之后主会话仍有 3 处未提交改动（本 judge 文件、`.gitignore`、`RELEASE-CONTRACT.md`）与 1 个未跟踪脚本，收尾提交后 C10 的廉价复核仍然适用。
