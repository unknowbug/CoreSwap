# 旧遗留清单溯源地图（backlog-map）

- status: draft
- 日期标签: 260919-06（宿主 Get-Date 锚 2026-09-19 18:42 实测）
- 角色: scout（只读勘探；产物仅本目录；不做深度分析解读）
- 任务: 把 NEXT_SESSION.md（260919-05 版）「下轮开工点 1」的 8 项旧遗留逐项溯源到**首次取证原文**（禁止以 NEXT_SESSION/转抄文档为据——转抄漂移反模式 = knowledge 发现 #90）。
- 方法: glob 日期标签目录 + 中文关键词 grep（.investigations/ .artifacts/ versions/*/docs/ knowledge/ framework-proposals/）+ 少量源码/上游直读核对。本文件引用的 file:line 均为本 session 实读核对，非转抄。

> 验证状态三分标注：**已验证结论**（有 confirmed/实锤链背书）/ **机制方向（⚠️ 未验证）** / **未验证假设**。
> 成本：低 = 纯文件核对 ｜ 中 = 需跑一次构建或采集 ｜ 高 = 需新探针工程。

---

## 1. R4 上报跟进（merge_index 裸列表根崩溃 → 框架提案）

- **① 首次取证原文**：
  - `.investigations/shared-java-core-260912-02/errors-260912-02.md:323`（W13，首证）：「## W13 工具链缺陷：`merge_index.py` 在**裸列表根**片段上崩溃 ⇒ 本块 index 登记只能手工应用」；:386 速查表行「`ref_merge_index` 抛 `AttributeError: 'list' object has no attribute 'get'`、退出码 1、未写任何文件」。
  - `.investigations/shared-java-core-260912-02/judge-260912-02.md:404`（judge 确认残留 R4 原文）：「**R4（工具，非本块产物）**｜ `ref_merge_index` 不可用的根因**已核实**：…5 片段根为**裸 YAML 列表**…⇒ 该缺陷属 **RE-Framework 工具**…建议经 `ref-maintain` 上报框架仓库」。
- **② 当前状态**：`framework-proposals/RE-FRAMEWORK-merge-index-list-root-proposal.md:5`「状态：draft（待维护 agent 评估）」——提案已成文（含源码三点定位 `:36-38`：`scripts/merge_index.py:52` 无类型分支 / `:59-60` 无 per-file 容错 / `:37+:55+:129-139` 写回丢注释）但**未见已转交/上游已修的登记**。本勘探直读上游 `E:\PYTHON\RE-Framework\scripts\merge_index.py:52` 仍为 `for e in data.get('entries', []) or []:` —— **上游未修，缺陷仍活**（260919-05 块 merge_index 仍崩溃，NEXT_SESSION 记作框架缺陷 #48）。
- **③ 验证状态**：根因与提案 = **已验证结论**（实锤 + judge 核实）；「上报跟进」本身 = **未做**（提案停留 draft，无转交记录）。
- **④ 建议廉价验证**：无需新验证——剩余动作是流程动作：把 `framework-proposals/RE-FRAMEWORK-merge-index-list-root-proposal.md` 转交 ref-maintain/Maint 会话（或经用户批准直接在 RE-Framework 仓库开 issue/PR）；转交前重算一次提案内工具 sha（`299598f46ebe1269`）确认上游仍同版。
- **⑤ 成本**：低。

## 2. Scope B、C（共享 Java 核抽取的 B/C 波次）

- **① 首次取证原文**：`.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md:82-84`（定义原文）：
  - 「**范围 A（最小运行时核，推荐起步）**：`CppWorldgen` / `CppBridge` / `BulkWb` / … + 新版 `WgCompat` shim（分版）；mixin **全部留分版**。」
  - 「**范围 B（A + 探针全量）**：再加 16 个探针文件（差异几乎全是 `Identifier.of`/`getOrThrow` 改名）⇒ 消掉最大宗重复；成本 = 每文件双映射编译核对。」
  - 「**范围 C（A + mixin 体抽取）**：把 `NoiseChunkGeneratorMixin` 的主体逻辑抽到共享 `FillDispatch`…」
  - 执行边界 `:217`：「范围 A 之外的文件（探针 16 个、mixin 全部）**本计划不动**；任何超范围改动 MUST 回 HOOK-2。」
- **② 当前状态**：多块「不做」列表结转（260913-01 `:25`「❌ Scope B/C 波次（需新 HOOK-2）」→ 260919-06 计划 `:12`）。实况核对（本勘探）：`java-core/src/main/java/wg/` 现仅 8 文件 = **范围 A 成员（CppWorldgen + bench 7 件），无探针、无 mixin、无 FillDispatch** ⇒ B/C 从未实施。
- **③ 验证状态**：「B/C 未做」= **已验证**（目录清点，本勘探代做）；「B/C 是否值得做」= **未验证假设**（1.21.6 树已大量使用 java-core，探针分叉面可能已变化，需重算双版 diff 再上 HOOK-2）。
- **④ 建议廉价验证**：纯文件核对——diff `versions/1.20.1/java` 与 `versions/1.21.6/java` 的探针 16 文件 + `NoiseChunkGeneratorMixin.java` 现状，重估 B/C 两波次的真实体量（原计划估算已 7 个月旧），产出后交用户 HOOK-2 拍板做/不做/废弃。
- **⑤ 成本**：低。

## 3. B3（1.21.6 Java 分版项移植——用户挂起）

- **① 首次取证原文**：`.investigations/000-架构设计/架构计划-260911-05.md:152-159`（方向变更记录原文）：
  - `:154`「**变更内容**：放弃『先把 B3 的 6 项 Java 分版项移植到 1.21.6』，改为直接做**方案 C = Rust 侧 bulk section 写回**…」
  - `:156`（用户依据，2026-09-11 对话原文）：「反正 1.21.6 暂时不上线也没问题，做 C」。
  - `:159`「**B3（原『全部 Java 分版项移植』）→ 暂停**。其中 **A1a（写回跳空气）/ A1b（扫描门控）** 在 C 落地后**整体消失**…正式作废这两项的 1.21.6 移植。」
  - B3 原始定义 `:84-86`（「### B3 实施移植 + 验证：按 B2 清单逐项移植（Java 分版项）…」）。
- **② 当前状态**：挂起未动，多块「不做」结转：`架构计划-260913-02:12`「B3 恢复」、`架构计划-260913-05:20`「不动 B3（用户挂起中）」、`架构计划-260913-06:16`「B3 恢复」，直至 260919-06 计划。无任何恢复/取代记录。
- **③ 验证状态**：**机制方向（⚠️ 未验证）**——「挂起」是范围决策非结论；恢复前置（B2 清单 6 项在 bulk 写回/共享核落地后的存废）未重算：A1a/A1b 已作废、exec/P1/指纹门/StallWatch 已随 260912-01 范围 A 部分顺带处理，**清单余额未知**。
- **④ 建议廉价验证**：读 `架构计划-260911-05.md` §2 B2 清单及其产物文件，逐项 grep 1.21.6 树（`versions/1.21.6/java` + java-core）核对「已顺带完成 / 仍缺 / 已作废」三态，产出恢复清单余额表 → 交用户拍板（恢复/继续挂起/正式关账）。
- **⑤ 成本**：低（纯文件核对；若余额非空转实施则另立项）。

## 4. 260914-02 open 三项

- **① 首次取证原文**：`.investigations/ns-ab-260914-02/record-260914-02.md`：
  - open①：`:51`「### 瑕疵 3：rcon stop ConnectionRefused ×3（open，未查机制）」+ `:54`「**根因**：未查（open 登记）。来源为驱动侧控制台输出，不在 server log 内」；judge 补充线索 `:53`「脚本含两次 `rcon stop` 调用（FAIL 分支 + 正常分支各一…）——『第二次 stop 时服务器已在退出』为机制候选高度可疑」。
  - open②：`:108`「open ②：r1 首跑预热假设未做独立验证（如额外 warmup 臂）——不影响主判据（保守读法两路一致），留待下批可选。」
  - open③：`:109`「open ③：pickHit 脚本双重重置未修脚本本体（本块以手工补验覆盖）。」（bug 定义 `:37-42` 瑕疵 1）。
- **② 当前状态**：record 头 `:3`「status: confirmed（用户授予 2026-09-14；judge review-001 = PASS-with-conditions，C1/C2 已应用）」——**主结论已 confirmed，open①②③ 三项仍 open**；review-001.md 为同目录 judge 件。无后续闭合记录。
- **③ 验证状态**：open① = **机制方向（⚠️ 未验证）**（judge 已给高可疑候选）；open② = **未验证假设**；open③ = 已定位的脚本 bug（未修，事实已验证）。
- **④ 建议廉价验证**：① 读 `.tmp/ns-ab-260914-02/` 驱动脚本确认双 `rcon stop` 结构 + 跑任一现成采集臂观察是否复现/记录重试（`脚本修复+重试计数` 一次搞定①③）；③ 修 pickHit 双重重置一行并对照当日 log 手验值（pickHit=2/臂）回归；② 最贵（需加 warmup 臂重采），可继续挂起——三项均不影响已 confirmed 主判据。
- **⑤ 成本**：①③ 低；② 中。

## 5. global palette 回退计数

- **① 首次取证原文**：`.investigations/light-round3-260914-04/review-final-judge.md:64`（judge 遗留风险原文）：「1. **global palette（ID_LIST，bits≥15）整 chunk 回退路径**：1.20.1 直方图实测 0 例（r1/r2/r8 bitsHist 4-6 均无 ≥15），但**未证不可能**…回退正确性 = 走原 blocks9 ABI…静默正确性由 rc=-2→闩回退的结构保证，但若真发生是整 chunk 性能悬崖…且无生产计数器可见——建议：12 篇声明 + 后续轮加一行回退计数日志（或 @anchor.idk 声明 90 天线）」；同件 `:74`、`record-260914-04.md:49`、docs 落盘 `versions/1.20.1/docs/12-lighting.md:129`。
- **② 当前状态**：**计数器已在 1.20.1 生产路径落地**（本勘探源码直读）：`versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:91-93`「回退打点（chunk 级计数，非热路径逐点日志）」+ `:169-178` `wgLightFallback(reason)`（打印 `[LightRust] fallback vanilla xN reason=...`，首 8 次全打 + 每 256 次采样）+ **`:794` `wgLightFallback("packed-rc" + rc + "-then-collect" ...)` 恰好覆盖 packed 解码 rc=-2（global 节）整 chunk 回退分支**（解码 null 定义 `:324`/`:376`「bits<4||bits>14 → 整 chunk 回退」）。另有 `[LIGHT-PATH]` path=legacy-fallback 三态打点（`:114`，260917-05 起）。**1.21.6 树**：mixin 有 `wgLightFallback`（`:100`）但**无 packed-rc 分支**——该树尚无 packed ABI 面，原 judge 建议「1.21.6 packed 移植时补 counter」仍适用。
- **③ 验证状态**：「计数器存在」= **已验证**（源码直读）；「global palette 在实机实际发生率」= **仍未采样验证**（计数器落地后无已知的采样记录；260914-04 的 0 例为旧直方图口径）。
- **④ 建议廉价验证**：跑任一现成采集臂（如 C-1a 复测台 `run_phase.py` 一臂），grep 输出 log 中 `fallback vanilla x` 与 `bits` 直方图（`-Dcoreswap.light.paldump` 现成探针），统计 packed-rc 回退是否 >0——若长期 0，可把该项从遗留转 idk/关账。
- **⑤ 成本**：中（需一次 run 采集；探针与判读台全部现成）。

## 6. 域批值 vs vanilla 直接对拍

- **① 首次取证原文**：`.artifacts/g3-drift-basis-260917-01/verdict-260917-01.md:56`（verdict 边界声明原文）：「n=1（每臂单次复现，run4 未验）；**域批值 vs vanilla 直接对拍未做（未证伪到值层面）**；settle 60s 非渐近上限；C-B3 宽限期未执行…」；过程登记同块 `versions/1.20.1/docs/10-timewise-archive.md:3295` 遗留⑤「域批光照值 vs vanilla 直接对拍未做（E2 修复后前置）」。
- **② 当前状态**：仍 open——260919-05 NEXT_SESSION 开工点 1 结转；`架构计划-260917-04:16`「域批值对拍」列入不做；后续 260919-01（A1 判别补通道）/260919-03（A1 .b2 系）/260919-05（C-1a）均做的是域批内部计时/等价，未做对 vanilla 的值面对拍。
- **③ 验证状态**：**未验证**（显式登记的未做项；前置「E2 修复」已闭合——`g3-drift-errors.md:11` E2 对拍脚本缺陷已修，判读工具 snap_light 在位）。
- **④ 建议廉价验证**：同一 world/seed 双臂采集——domain 臂 vs vanilla light 臂（`-Dcoreswap.light.rust=0` 或 vanilla-init0 路径）各一次 snap_light 快照，用现有比较器做值面 diff（非只 changed 计数）；判据预登记先行（#112）。
- **⑤ 成本**：中（两次现成台采集 + 现成比较器；无新探针工程）。

## 7. 260918-03 open①②

- **① 首次取证原文**：`.investigations/b61x-blobprobe-260918-03/record-260918-03.md:13-18`（根因原文）：「**stats 统计读取的设计灰区…**：1. `BlobProbe.java:39,42` 经 `Class.forName("wg.bench.mixin.BlobProbeMixin").getDeclaredField("CALLS"/"WRITTEN")` 反射读取计数…3. 反射读取是全 run **唯一一次加载 mixin 类本体**的时点；Knot/Sponge transformer 对这次自载尝试变换失败…4. `BlobProbe.java:46` 只打印顶层消息 `+ t`，**底层 cause 被吞**（…更深层机制未开箱，登记 open）」；现象首证 `:8`「原始日志 `.investigations/b62-260918-02/cmd-output/sentinel-1216-all12.log:142` 实锤」。
- **② 当前状态**：record 与镜像 verdict 均「status: confirmed（用户授予 2026-09-18，H3；judge M-1 已闭合）」（`record:3` / `.artifacts/blobprobe-260918-03/verdict-260918-03.md:3`）。open 两条即 **open①②**（`verdict-260918-03.md:33-36`）：**open①** =「transformer 拒绝自变换的底层 cause（未开箱，不阻塞）」；**open②** =「1.20.1 树同名探针 stats 面仍未修（本块范围排除…低优先）」。本勘探代核 open② 现况：`versions/1.20.1/java/src/main/java/wg/bench/BlobProbe.java:39/:42` 仍是 `Class.forName(...BlobProbeMixin...)` 反射路径 ⇒ **open② 仍成立**。
- **③ 验证状态**：主结论（候选 A 成立）= **已验证结论**（confirmed）；open① = **机制方向（⚠️ 未验证）**（cause 被吞、未采集）；open② = 已验证的事实陈述（本勘探复核仍真）。
- **④ 建议廉价验证**：open② 若决定修 = 把 1.20.1 树 `BlobProbe.java` 反射读改直读 `BlobProbeStats` 同款 holder（1.21.6 已有现成实现可照抄，注意 1.20.1 侧属出货冻结面需 HOOK 确认是否值得动）；open① 静态面 = 读 fabric-loader Knot/Sponge transformer 拒绝自变换的源码定位（纯文件核对）；动态面 = 临时补一行 cause 打点跑最小 run（改诊断代码，超出「廉价」边界，可只做静态面）。
- **⑤ 成本**：open② 修复 = 低-中（照抄 + 一次编译验证；涉及出货树需拍板）；open① 静态 = 低。

## 8. 260915-01 形态审计后续

- **① 首次取证原文**：
  - 勘探层：`.investigations/form-audit-260915-01/p1a-coreswap-forms.md` / `p1b-vanilla-forms.md`；worker 行集 `p2-w1..w4`（W4 执行器层 R1-R5 见 `p2-w4-executor-rows.md:12-63`）。
  - 池与裁决：`.artifacts/form-audit-260915-01/candidate-pool-260915-01.md:23-76`（CP-1..CP-6 候选池 + 预验证清单 + 执行序「CP-3 → CP-6 源码核对 → 预验证 1/2/3/4 → …」）；judge `.artifacts/form-audit-260915-01/judge-review-260915-01.md:112`「**PASS-with-conditions（C-1..C-7，其中 C-3/C-6 为 binding）**」；池状态 `candidate-pool-260915-01.md:6`「**confirmed（2026-09-15 用户『授权确认』；范围 = 候选池 CP-1..6 排序 + 矩阵等价判定 + D-hm 结案…各 CP 修复实施后的行为级结论另走各自验收节点）**」。
- **② 当前状态**（执行链逐块核实）：
  - CP-3（scratch RefCell→Mutex）：260915-02 已落地，judge 建议 candidate（`form-audit-260915-02/review-001-cp3-cp6-judge.md:46`）；后被 260916-01 further per-call 化取代/演进（`cp1-light-form-260916-01/record-260916-01.md:8`）。
  - CP-6（needsSaving）：**结案不立项**（`form-audit-260915-02/cp6-needssaving-verdict.md:25`）。
  - 预验证探针 1-4：260915-03 完成，CP-1 升首 / CP-2 降后（.b1/.b2/.b3 三否定）/ CP-4 降后（`form-audit-260915-03/judge-review-260915-03.md:40-43,77`）。
  - CP-1 主线：260916-01 C-1 失败挂起 → 260917-01 G3 drift 回炉（「域批有害」方向撤销取代）→ 260917-04 C-1(R1) 重述 confirmed → 260919-01 A1 判别补通道 → 260919-03 A1 .b2 系（B-主未满足）→ 260919-05 **C-1a 形态 S1 收口 candidate（J4 性能判据未满足，用户拍板保留缺省生效）**。
  - **仍开放余额**：CP-2（降后，复议条件 = 新形态证据）、CP-4（降后，judge C-3 范围限定绑定）、CP-5（#122 池宽死参数，随批未做）、judge 绑定条件 C-1（T_fill 双口径统一回填）/ C-5（13.8× 池宽依赖声明）；NEXT_SESSION 260919-05 开工点 2 的「C-4 灰区二值化」亦属本链延伸。
- **③ 验证状态**：池级结论（排序/矩阵/D-hm）= **已验证结论**（confirmed 范围内）；各 CP 实施级 = 各自验收节点（CP-1 主线现役 candidate、J4 未满足）；CP-5 = **未验证假设**（本机 24 逻辑核不触发退化面，发行面价值未评估）。
- **④ 建议廉价验证**：① 读 `form-audit-260915-03/interpretation-draft.md` §2 排序表（`:98-102`）与 `candidate-pool:6` 状态行核对本节状态转抄零漂移（文件核对即可）；② CP-5 廉价面 = 静态核对 `resolveExecPoolSize`/`resolveMaxInflight`/Rust `adaptive_threads` 三处「logical/2-2」同族参数现状 + 低核机参数表（不跑机）；③ CP-2/CP-4 无新证据不重开（遵守降后复议条件）。
- **⑤ 成本**：① 低；② 低；CP 实施级 = 另立项（不在「廉价验证」范围）。

---

## 汇总表

| # | 项 | 溯源 | 首证 file:line | 现状态 | 验证状态 | 建议验证成本 | 处置方向 |
|---|---|---|---|---|---|---|---|
| 1 | R4 上报跟进 | ✅ | errors-260912-02.md:323 + judge-260912-02.md:404 | 提案 draft，上游未修 | 根因已验证；跟进未做 | 低 | 转交 ref-maintain/Maint，流程动作 |
| 2 | Scope B、C | ✅ | 架构计划-260912-01:82-84 | 未实施（java-core 仅范围 A，已核） | 未做=已验证；价值=未验证 | 低 | 重算双版 diff → HOOK-2 拍板做/废 |
| 3 | B3 | ✅ | 架构计划-260911-05:154-159 | 用户挂起中 | ⚠️ 未验证（清单余额未知） | 低 | 重算 B2 清单余额 → 用户拍板恢复/关账 |
| 4 | 260914-02 open 三项 | ✅ | record-260914-02.md:51-56,104-109 | 主结论 confirmed；open①②③ open | ①⚠️未验证 ②未验证 ③已定位未修 | ①③低 / ②中 | 修脚本+重试计数（①③）；②可续挂 |
| 5 | global palette 回退计数 | ✅ | review-final-judge (260914-04):64 | 1.20.1 计数器已在位（:794 packed-rc）；1.21.6 无该面 | 计数器已验证；发生率未采样 | 中 | 现成台跑一臂 grep 采样，0 则转关账 |
| 6 | 域批值 vs vanilla 对拍 | ✅ | verdict-260917-01.md:56 | open（E2 前置已闭合） | 未验证 | 中 | 双臂 snap 对拍一轮（判据先登记） |
| 7 | 260918-03 open①② | ✅ | record-260918-03.md:13-18 + verdict:33-36 | confirmed；open①② open | 主结论已验证；①⚠️ ②复核仍真 | ①静态低 / ②低-中 | ①Knot 源静态定位；②照抄 Stats 修复（出货树需拍板） |
| 8 | 260915-01 形态审计后续 | ✅ | candidate-pool:23-76 + judge-review:112 | 池 confirmed；CP-3/6/预验证已执行；CP-1 主线至 C-1a candidate | 池级已验证；CP-5 ⚠️ | 低（核对/静态） | 核对零漂移 + CP-5 静态评估；CP-2/4 不重开 |

## 边界与诚实声明

- 本文件为勘探地图（draft），不改变任何既有 status；所有「当前状态」判断以文内 file:line 现读为准。
- 交接线索核对结果：R4/Scope B/C 与形态审计猜测命中；B3 的交接线索（jungle-l / 260914 系列）**未命中**——实际溯源为 260911-05 方向变更记录（jungle-l B3=BB 坐标系已于 260907-03 结案、C1 清单 B3 已于 260908-14 取代关账、-288 B3 已撤销，均非挂起项，检索过程已排除）。
- 本勘探自身核对动作（可作下轮直接继承的一手证据）：java-core 目录清点、RE-Framework 上游 merge_index.py:52 直读、1.20.1/1.21.6 ServerLightingProviderMixin 直读、1.20.1 BlobProbe.java:39/42 直读。
