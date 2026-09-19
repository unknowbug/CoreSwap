# legacy-sweep 260919-06 — A 组低成本项 record（draft）

- 日期标签: 260919-06（Get-Date 2026-09-19 锚）
- 架构: `.investigations/000-架构设计/架构计划-260919-06-旧遗留清单验证.md`（已批准）
- 溯源: `.investigations/legacy-sweep-260919-06/backlog-map.md`（scout draft）
- status: candidate（judge review-260919-06 PASS-with-conditions N1-N7；N1-N4 已应用，N5 已预登记，N3/N6/N7 登记；confirmed 留用户）

## T1 R4 上报跟进 ✅

- 上游复核：`E:\PYTHON\RE-Framework\scripts\merge_index.py` sha256 前 16 位 = `299598f46ebe1269`（与提案基线一致），上游最后 commit `eeebff9`（2026-08-08）→ 缺陷仍活、提案未过时。
- 动作：`framework-proposals/RE-FRAMEWORK-merge-index-list-root-proposal.md` 状态 draft → **submitted（已转交维护侧）**，含复核证据与 §5 验收判据指引。
- 验证状态：根因=已验证（继承）；转交动作=本轮完成（流程性，无需判据）。

## T4 260914-02 open①③ ✅（open② 维持挂起）

- **open① 根因静态坐实（机制候选 → 已验证）**：驱动脚本 `.tmp/ns-ab-260914-02/run_nsab_1216_260914-02.ps1` 原含**编辑残留的第二次 `rcon stop` + `WaitForExit` 块**（原 :76-78）——第一处 stop（:57）已发起关停并等待退出，第二次 stop 必然 ConnectionRefused。judge C2 的「第二次 stop 时服务器已在退出」候选静态证实（证据 = 脚本原文结构；未做运行时复现，回归面 = 修复后 stop 唯一化 + 重试计数行）。
- **open③ 修复**：删除 pickHit 重复重置（原 :73-74 `$pickHit = 0` 残留块）；stop 重试可见化（新增 `[stop] refusedCount=N` 输出，瑕疵 3 教训落地）。
- **回归验证**：`Parser::ParseFile` PARSE OK；修复后 pickHit 逻辑对当日 log（nsab1216-bulk-r4.log + .err）复算 = **2**，与 260914-02 人工补验值一致。（judge N3 登记项：修复后脚本未实跑整臂回归——下次采集自然覆盖，属登记非阻塞。）
- **§9.8 副作用逆登记（judge N4）**：该驱动脚本直接改写 `runtime\1.21.6\java\run\server.properties` 的 `level-seed`（无备份步）——逆 = 常量 `SEED=417950215108767439` 重写为固定值，若需还原原值，原值以 git/历史记录或用户确认为准（本轮未还原，seed 与该台历史采集一致，环境未漂移）。
- stop 数量核对：FAIL 分支 stop（:43）保留（此时服务器在跑，合法）；正常路径唯一 stop。
- 修复方式：脚本工程修复（不计 evidence saturation）。
- open②（warmup 臂重采）：按批准范围续挂。

## T5 260915-01 形态审计后续 ✅

1. **零漂移核对**：`interpretation-draft.md:94-102` 排序表（CP-1 升首 / CP-4 降后 judge C-3 / CP-2 降后三否定 / CP-5 随批）与 `candidate-pool-260915-01.md:6` confirmed 范围行 → 与 backlog-map 转抄**零漂移**（逐行对照）。
2. **CP-5 静态参数表（现状直读，Degraded）**：三处同族缺省全部 = `max(1, logical/2 − 2)`：
   - `NoiseChunkGeneratorMixin.resolveMaxInflight()`（:105-116，`-Dcoreswap.maxinflight` 覆盖）
   - `NoiseChunkGeneratorMixin.resolveExecPoolSize()`（:151-164，`-Dcoreswap.execpool` 覆盖）
   - `worldgen-core/src/api.rs adaptive_threads()`（:24-37，`CORESWAP_THREADS` 覆盖；SMT2 假设注释在位；A1d clamp 修正已含）
   - 低核机缺省推演（同公式）：4 逻辑核 → 1；8 → 2；16 → 6；24 → 10。4 核机缺省 = 1 线程（exec 池宽 1 = 近串行）——CP-5 发行面价值的量化基础；是否立项仍属用户范围决策，本块不立项。
3. CP-2/CP-4：无新形态证据，遵守降后复议条件，不重开。

## T2 Scope B、C ✅（重算完成，用户拍板：B/C 均继续挂起）

- 重算产物：`.artifacts/legacy-sweep-260919-06/scope-bc-reassessment.md`（worker draft；diff 原始证据 `.tmp/scopebc/diffs.txt`）。
- 要点：探针现 19 个（原计划 16 过时）；8 逐字节相同 / 10 纯改名（缝面 4-5 个 WgCompat 方法）/ 1 实质差异（BlobProbe）；mixin 侧 NoiseChunkGeneratorMixin 463 vs 253 行重度分叉；C 体量远超原计划且 exec 形态现役演进中。
- **用户拍板（260919-06 HOOK-2）：B/C 均继续挂起**，等 exec 形态收口后再评估。

## T3 B3 余额 ✅（重算完成，用户拍板：继续挂起）

- 重算产物：`.artifacts/legacy-sweep-260919-06/b3-balance.md`（worker draft）。
- 余额：仍缺 2（exec 模式 / P1 maxinflight，均 1.20.1 独有）/ 顺带完成 4 / 作废 3。
- **用户拍板：继续挂起**（触发条件 = 1.21.6 上线/并发压测需求；不满足关账）。

## T6 260918-03 open①② ✅（用户拍板：open② 不动出货树）

- open① 静态定位：`.investigations/legacy-sweep-260919-06/knot-selftransform-static.md`（worker draft，Degraded）——root cause = Sponge MixinProcessor 对已注册 mixin 类直接 classload 的设计性拒绝（IllegalClassLoadError「cannot be referenced directly」，字节码偏移 ~415-430 实证），经 KnotClassDelegate.java:422-427 包装、BlobProbe.java:46 吞 cause；一手源 = gradle cache fabric-loader 0.15.11 sources jar。证据等级 = 机制方向（未运行时验证；定论需 getCause() 打点一次最小 run）。
- open② 现况复核仍成立（1.20.1 BlobProbe.java:39/42 反射坏路径）。**用户拍板：不动出货树，登记为已知限制**（影响面 = 探针 stats 读取失败，仅诊断用）。

## T7 global palette 回退采样 ✅（零回退，用户拍板：转关账 + idk）

- 采集：`.tmp/c1a-260919-05/run_phase.py legacy-t7-1 off`（复用 260919-05 现役台，SELFCERT 全绿：done/move5/lightInit/hook/dll=cc4e39fe，domain_task_ok=457）。
- 结果：`fallback vanilla` = 0、`packed-rc` = 0、`[LIGHT-PATH]` legacy-fallback = 0（log 全文 grep，`.investigations/c1a-260919-05/cmd-output/legacy-t7-1.log`）。
- 读法（预登记于 backlog-map §5 建议）：回退发生率 = 0/457 任务满载 → 计数器在位且未触发。
- 边界声明：单 seed 单臂 n=1，非穷尽证明（「未证不可能」维持）——关账形态 = 关闭遗留项 + @anchor.idk 90 天线（judge 确认后生效）。
- **用户拍板：转关账 + idk**。

## T8 域批值 vs vanilla 对拍 ✅（判据 FAIL：值面差异存在）

- 判据预登记：`.tmp/legacy-sweep-260919-06/criteria-t8.md`（先于采集，#112）；驱动 `run_t8.py` + 比较器 `cmp_t8.py` 与判据同批定稿。
- 采集（双臂 r1，均 SELFCERT 全绿）：
  - domain 臂：`lightInit ok=1`、`[LIGHT-DOMAIN] hook armed=1`、dll=cc4e39fe、settle 60s；snap = t8-domain-r1-light.json（9450 chunk）。
  - vanilla 臂：负自证通过（lightInit=0 / [LightRust]=0 / hook=0，#118 硬门）、dll=cc4e39fe；snap = t8-vanilla-r1-light.json（9450 chunk）。
- **判定（预登记读法）**：common = 9450/9450（new=0 gone=0，覆盖面一致）；**light-hash diff = 4223 = 44.6878% of common → VERDICT: FAIL（值面差异存在）**。
  - 量级对照（§9.7 等价档位声明，judge N2）：「≫ ~0.8%」中 0.8% 为 **E1（同构建态 run 间）** 噪声带（G17 旁证口径），本对拍为 **E2（跨执行体：Rust 域批 vs vanilla 光照）**——E1 带不作 E2 噪声上界，该句仅作「差异远超 run 级摆动」的量级示意；FAIL verdict 依据 = 预登记存在性判据（diff > 0），不依赖该量级句。差异为系统性（非 run 噪声）的读法成立。
  - 机制归因不在本判据层（判据为存在性判据）；课题化建议交 judge/用户（关联面：260917-01 verdict:56 显式遗留「未证伪到值层面」——本轮已证伪「值面一致」假设；此前 G3 收敛门证明的是域批臂自身 run 间收敛，与「对 vanilla 值对齐」是两个不同命题，#16 对照基线家族）。
- 比较器外观缺陷（诚实登记）：`cmp_t8.py` sample 行对字符串 key 做序列切片 → 抽样坐标显示失真（"1,3" ×10）；**计数与 VERDICT 基于完整字符串 key 集合计算，不受影响**；修复一行即可（显示层），已登记不静默。
- 覆盖面：单 seed × FP-DRV 驱动箱 × overworld × n=1 × 载具 snap_light.py；不外推。
- 处置：verdict = FAIL 如实上报；后续课题（值差剖析/归因）超本轮「廉价验证」范围，交用户决策是否立项。

## 边界声明

- open① 为**静态机制坐实**（脚本结构 + 日志现象吻合），未运行时复现回归；§9.7 口径：载体=脚本原文+260914-02 日志，覆盖面=该驱动脚本，跨脚本外推需各自核对。
- CP-5 参数表为静态直读（Degraded），无运行时低核机实测。
