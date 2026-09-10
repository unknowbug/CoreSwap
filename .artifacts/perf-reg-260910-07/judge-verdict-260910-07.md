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
审查时点锚: Get-Date 2026-09-10 23:00:11；git HEAD = d5151a1（22:57:13 amend）；worktree = ` M .artifacts/index.yaml` + `?? .investigations/perf-reg-260910-07/knowledge-draft-260910-07.md`
结论等级: PASS-with-conditions（C1 为 blocking 级；在 C1 修复前不建议进入 confirmed 授予流程）
推荐状态: 保持 candidate（本意见不改任何 status；confirmed 只能由人类授予）
---

# judge 审查意见 — 260910-07「Java mod 工程迁出 runtime/」

> 本角色只出审查意见，**未修改** verdict / index.yaml / errors 台账 / 计划 / 证据文件的任何 status 字段。
> 沙箱内**无构建/运行授权**：本审查以「读一手文件 + 自行重算」为限，凡未核项在 §9 逐条声明。
> **审查时点固定声明**：本次审查与父会话收尾**并发**进行 —— 审查期间观测到 HEAD 由 `bd23b9a`(22:56:59) 被 amend 为 `d5151a1`(22:57:13，.gitignore 回调白名单链 6 行)，其后 `.artifacts/index.yaml`(22:57:47)、`NEXT_SESSION.md`(22:58:24) 才落盘，`knowledge-draft-260910-07.md`(23:00) 为新增未跟踪文件。**本意见的全部 pin 以 HEAD=d5151a1 + 23:00:11 的 worktree 为准**；若其后 `.gitignore`/verdict/证据文件再变动，见 C10。

## 1. 三源核对（spec §4 / §15.4 Judge review baseline）

| 源 | 实况 | 一致性 |
|---|---|---|
| ① 交付快照（`.artifacts/perf-reg-260910-07/`） | 仅 `verdict-260910-07.md`（64 行，`状态：candidate`）；index.yaml:1273-1299 已登记 verdict/errors/plan 三条（均 candidate） | 基本一致，但见 C7（提交指针） |
| ② 工作区实际状态（git HEAD + worktree） | HEAD=`d5151a1`（2876 文件，含 `versions/<ver>/java/**` 全部源码）；`git status` 仅 ` M .artifacts/index.yaml` + 未跟踪 knowledge-draft；新旧位置目录、`.gitignore`（119 行）、`AGENTS.md:138`、`run_rust_client.ps1:74`、两个 `build.gradle` 的 runDir 均与 verdict/计划声称一致 | **一致** |
| ③ 验证/回归记录 | `equivalence-260910-07.txt`(15 行)、`ignore-checks-260910-07.txt`(9 行) 在 `.investigations/perf-reg-260910-07/cmd-output/`；**`results.txt` 与 `logs/` 不在该目录**（实际只在 `.tmp/perf-reg-260910-07/`，被 `.gitignore:89` 忽略） | **不一致 → C1（blocking）** |

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
| 1.21.6 target/jar-inner | `abd7d8893d22e030…`（`:12,:13`） | 我未重算（`versions/1.21.6/rust` 薄壳产物位置未在本次扫描范围内定位） | ⚠️ 未核（见 §9） |

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
实质结论（迁移成立、等价性门（jar 逐字节相同）、环境原地、源码入库、活引用面闭合）**经我独立重算全部成立**；下述条件中 **C1 属 §15.4 blocking 级**（声称的证据路径不存在 —— 协议声称与磁盘实况矛盾），其余为 should-fix / info，不阻塞提交。**在 C1 修复前，不建议进入 confirmed 授予流程。**

| # | 级别 | 位置 | 问题 | 建议动作 |
|---|---|---|---|---|
| **C1** | **blocking** | verdict 头部 L6 + `NEXT_SESSION.md:6` | 两处都把 `results.txt`、`logs/` 写成 `.investigations/perf-reg-260910-07/cmd-output/` 下的证据，但该目录**只有** equivalence + ignore-checks 两个文件；V2 的原始运行证据实际只在被 `.gitignore:89` 忽略的 `.tmp/perf-reg-260910-07/`（未入库） | 把 `results.txt` + `logs/oa-r1.log(.err)` + `logs/oa-r2.log(.err)` 复制进 `.investigations/perf-reg-260910-07/cmd-output/`（对齐 260910-06 的 cmd-output 归档惯例），并订正这两处路径字样 |
| **C2** | should-fix | `equivalence-260910-07.txt`（15 行） | 只有结果、无「命令 + 被测文件绝对路径」，产生它的比对脚本在仓库内不存在（`.tmp` 只有 `rcon_one.py`、`run_arms_1201.ps1`）⇒ 该门**不可原位复跑**，也无法从文件本身确认比对的是哪两支 jar | 把比对脚本落 `cmd-output/`（并记录新旧 jar 绝对路径 + `--rerun-tasks` 事实）；否则以「judge 独立复算记录」作为替代证据一并落盘 |
| **C3** | should-fix | verdict §3 V2 / §4.2 / §7，对照计划 §1 D1 | D1 要求**两版本** `:build` 与 `:runServer` 均可从新路径跑；实际只跑了 1.20.1 一臂（1.21.6 `run/` 的 world mtime 19:36 早于迁移），§7 未把这一半列为缺口 | 在 §7 显式登记「1.21.6 `runServer` 未跑（D1 半开）」；若要闭合则补跑一臂 1.21.6 |
| **C4** | should-fix | verdict §2「.gitignore」行 + `ignore-checks-260910-07.txt` | 本块最易踩的 **#24 目录级 prune 陷阱**（`data/`+`**/data/*` 吃掉 `worldgen-data/data/**`）所对应的逐级白名单链（`.gitignore:106-110`）**既未写进 verdict §2，也无任何一条 check-ignore 证据行**（8 条里没有一条落在 `…/worldgen-data/data/` 下） | 在 §2 补写白名单链，并在证据里补一条 `git check-ignore -v --no-index`（含一个假想新文件，证明「新 JSON 会自动入库」） |
| **C5** | should-fix | verdict §3 V4 | 文中声称「8 条全对」且列举含 `.gradle/`，但证据文件 8 行的实际组成是 4 正 + 4 负（1.20.1 src、worldgen-data 根、build.gradle、1.21.6 src；run/、build/、native/、runtime/），**没有 `.gradle/` 行**（我另行实核 `.gitignore:101` 与 `:103` 均 IGNORED） | 按证据文件实际行重述 V4，或补 `.gradle/` 与 `content-test/run` 两行后再写「10 条全对」 |
| **C6** | should-fix | verdict §3 V2 | 「跑 1 臂（**oj** 异步 + `-Pchunktime=1`）」——驱动里不存在 `oj` 臂，实际臂名是 `oa`（`run_arms_1201.ps1:20`；结果行 oa-r1/oa-r2） | 改为 `oa`；顺带把 §1「零语义影响」补半句「（jar 级；运行时行为见 §7.5）」 |
| **C7** | should-fix | verdict §5；`.artifacts/index.yaml:1286` | §5 写「提交：本块改动见 **§7 提交记录**」，但 §7 是「老实边界/未核项」、**无提交记录**；唯一的提交指针在 index.yaml（`提交：d5151a1（2876 文件）`，与 HEAD 一致），而 index.yaml 目前仍是 **` M`（未提交）** 状态 | verdict §5 直接写 `d5151a1`（2876 文件）；index.yaml 的 260910-07 三条登记随下一个提交入库 |
| **C8** | should-fix | verdict §3 V5 | 判据是「记录即算（步骤自足）」，但仓库内**没有回滚步骤清单**（仅计划 §3/§5 有片语），真回滚时需现场重推 | 在 §3 V5 补 3 行步骤（反向 `Move-Item` 两版本 / 还原 `.gitignore` / 还原 `runDir`）并注明「依赖 `.tmp/legacy-runtime-260910-07/` 未删」 |
| **C9** | info | verdict §7.3；`NEXT_SESSION.md:36②` | 债清单列了 4 处 gradle home，**漏了本块迁移时新建的 `versions/<ver>/java/.gradle` 与 `versions/<ver>/java/build`**（旧位置残留 + 新位置缓存，实际是 6 处） | 补进后续清理项 |
| **C10** | info | 全文 pin | 本审查与父会话收尾并发：审查期间 HEAD 由 `bd23b9a`(22:56:59) 被 amend 为 `d5151a1`(22:57:13，.gitignore 回调白名单链 6 行)，其后 index.yaml(22:57:47)/NEXT_SESSION.md(22:58:24)/knowledge-draft(23:00) 才落盘 | 若下一个提交再动 `.gitignore`/verdict/证据文件，请按「重算两支 jar sha + `check-ignore` 三条 + `git ls-files` 两树计数」做一次廉价复核，再据以推进 confirmed |
| **C11** | info | `versions/1.20.1/java/run_rust_client.ps1:94`；verdict §2「环境」行；§1 目标句 | ① 注释仍写「切到 mod 工程（runtime）」（功能已改，仅注释陈旧）；② §2 把 `scripts/` 写成两版本共有（实测 1.21.6 无 `scripts/`）；③ 「零数据搬迁」宜限定为「主 runDir 零搬迁」（content-test/run 0.08MB 已随迁，§7.1 已声明）；④ `results.txt` 里两条 `FAIL_no_done` 行建议备注来源（E1/E2） | 逐条改字/备注即可 |

## 9. 覆盖面声明（我实际核了什么 / 没核什么）

**全量核（未抽样）**
1. V1 等价性：**两支 jar 的全部 zip 条目**（1147 / 1908 条）逐条目 sha256 + 整文件 sha256 重算（1.20.1 与 1.21.6）——数据来源为**工作区实况 jar 本身**，不依赖证据文件自述。
2. V3 三元组：1.20.1 侧 target dll 实测 sha + jar-inner + 运行日志 dll 行三者比对。
3. V4/D4：`git check-ignore -v` 实跑 **14 个路径**（含证据未覆盖的 `.gradle/`、`content-test/run`、1.21.6 build/run、假想新文件）；`git ls-files` 对 `versions/{1.20.1,1.21.6}/java` 及 `src`、`worldgen-data`、`worldgen-data/data` 五组计数 + 磁盘计数比对；`git status --untracked-files=all` 两树内全量。
4. V2 关键行：`results.txt` 全 4 行、`oa-r1.log`/`oa-r2.log` 的 Processed/Total time/CHUNKTIME/bridge/dll 全部命中行（含日志尾部的 world 保存时间）。
5. 结构与残留：新旧两版本 java 目录顶层全列举、`src/main/java` 与 `worldgen-data` 文件计数（49/49、1019/1739）、旧位置散件清零、`runtime` 残留目录集、历史 jar 1.0.17–1.0.28 清点、散件隔离区（3 java + 58 log + 1）。
6. 引用面：187 个活文件全量正则清扫（定义域见 §5）+ 两个 `build.gradle` 的 runDir/vmArg 行 + AGENTS.md:138 + `run_rust_client.ps1` 变量/注释 + git 侧 `.gitignore` 与 HEAD 提交内容/reflog。
7. 置信度与子角色：verdict 全文 + index.yaml:1273-1299 的 status 与注释 + 计划 §6/§7/§9 的预置项。

**抽样核**
1. 计划 §2 事实表 F1-F10：抽验 F1/F4/F5/F7/F9（文件计数、dll 复制步骤、runDir DSL、settings/gradle.properties）——**未逐条复验 F2/F3/F6/F8/F10**（F2 需解包对照 class↔源清单、F3 需体量分解、F6 需读 `x` magic、F10 需 `git log --all` 追三处 commit；本次等价性门已从更上游覆盖 F2 的实质关注）。
2. `runtime/1.20.1/java/run` 配置扫描：**56 个 <2MB 的文本配置**全扫；未扫 `run/world/**` 二进制 region 与 `logs/*.gz`（无路径引用语义）。
3. `.tmp/perf-reg-260910-07/run_arms_1201.ps1` 全读，但未执行；`rcon_one.py` 未读。

**未核（附原因）**
1. **未重跑 `gradle :build` / `:runServer`**：judge 会话无构建/运行授权（且重跑会污染 runDir/环境与既有读数）；替代强证据 = 旧/新 jar 整文件 sha 逐字节相同 + 新 jar ctime 证明真重编 + 目标 dll 与运行日志 dll 同值。
2. **未独立复算 1.21.6 侧 dll sha（`abd7d889…`）**：未在本次扫描域内定位 1.21.6 薄壳产物路径，故该格仅采信证据 + 1.21.6 jar 整文件 sha 已复算（jar 内 dll 因而随文件 sha 一并确定）。
3. **未跑 ignore-checks 的生成脚本**（脚本不存在，见 C2）；`ignore-checks-260910-07.txt` 的 `TRACKED (not ignored)` 是包装层自述标签，我已用原生 `git check-ignore -v` 独立复核其结论。
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

> 因此：**`.gitignore` 白名单链缺失一说是 bd23b9a 的瞬态，不适用于现 HEAD**；上文 C4 只保留「verdict/证据未覆盖该链」，不主张机制缺失。
