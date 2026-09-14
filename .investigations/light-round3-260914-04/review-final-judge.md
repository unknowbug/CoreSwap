# review-final-judge — 光照 round3（260914-04）收尾审查意见

> 角色：core.judge（收尾 MUST，三源核对）。**只出意见，不改 status；confirmed 留人类。**
> 审查对象：① `.investigations/light-round3-260914-04/`（record 终稿 + 全套中间产物 + cmd-output 23 件）② git 工作区 diff（6 改 + 新增未跟踪）③ 验证记录（golden/probe-r1..r9/e2e 12 件）。
> 日期标签：260914-04。

## 1. 三源核对结果（逐项）

| # | record 引用 | 原始日志核对 | 结论 |
|---|---|---|---|
| 1 | r1 collect 4.527 / native 2.770（chunks=400） | probe-r1.log `chunks=400 collectAvgMs=4.527 nativeAvgMs=2.770` | **PASS** |
| 2 | r1 Done 17.465s 双锚 | probe-r1.log `Done (17.465s)` | **PASS** |
| 3 | r2 collect 4.527→1.257（native 2.86≈2.77） | probe-r2.log `chunks=400 collectAvgMs=1.257 nativeAvgMs=2.859` + DUAL 4 chunks diffTotal=0 | **PASS** |
| 4 | r8 native 3.33/3.24（免除法后） | probe-r8.log `chunks=200 3.329 / chunks=400 3.236` + DUAL/DUALP 双 ALL-MATCH | **PASS** |
| 5 | r9 collect 0.288 / native 2.651 / fallback=0 / paldump 0 节 | probe-r9.log `chunks=400 collectAvgMs=0.288 nativeAvgMs=2.651`；`LIGHTPROBE-PAL nonEmptySections=0 fail=0`（空直方图 = 旧收集路径零采样）；复编 `BUILD SUCCESSFUL` | **PASS**（措辞注：`fallback=0` 无显式计数器行，由 nonEmptySections=0 承载——成立但属间接直证，见 §3） |
| 6 | 六对中位 ON 18.58（16.47-19.68）/ OFF 17.32（15.69-18.31）/ 比 1.072 | e2e-on1..6 = 18.001/19.681/18.326/16.473/19.134/18.838 → 中位 18.582；off1..6 = 17.143/16.166/15.694/17.502/18.309/17.711 → 中位 17.3225；比 1.0727 | **PASS**（手算复现逐位吻合） |
| 7 | 配对差 [0.86, 3.52, 2.63, −1.03, 0.83, 1.13] 均值 1.32 / 中位 ~1.0 | 逐对重算：0.858/3.515/2.632/−1.029/0.825/1.127；均值 1.321；中位 0.99 | **PASS** |
| 8 | golden 4 用例与 round2 冻结件逐位一致 | `Compare-Object` golden-round3-260914-04.txt vs `.tmp/d3-opt-260905-04/golden_post_round2.txt` = **逐位相同（4/4）** | **PASS** |

**git diff 面核对**：6 个修改文件（worldgen-core/src/light/mod.rs = light_decode_packed + PackedDecode 错误码 + 2 个 doc-hidden 诊断访问器；ServerLightingProviderMixin.java；LightPalDump.java；build.gradle 新增 lightTiming/lightPaldump 两 -P 映射；CppWorldgen.java lightComputePacked native 声明；1.20.1 与 1.21.6 rust/jni_bridge.rs）与 record §3/§4a/§6 声明**一一对应，无未声明改动**。未跟踪 = 2 个 bin-diag 探针（light_probe_260914.rs、light_packed_test.rs）+ 调查目录 + 架构设计两份——符合临时区纪律。注：审查指令称「bin-diag 三个新探针」，实际 diff 面为 **2 个**（record 本身未声明 3，非 record 缺陷，指令侧笔误）。1.21.6 Java 侧零改动 ✓（git status 无 1.21.6 java 条目，与 §6 声明一致）。

**中间产物齐全**：scout-map / b1/b2/b3 / review-p2-ordering-judge / probe-verdict 均在，probe-verdict 带 §9.7 先行声明 + draft 标注 ✓。**.artifacts/index.yaml 尚无 light-round3 条目**——record §7 已列「知识库更新/提交」为未完成待办，属已知未闭项（round2 judge 的 standing 条件延续），收尾提交前须补。

## 2. 判据诚实性

- **FAIL 如实**：B 期 1.070-1.083 FAIL、n=2 批「不可判」（OFF 极差 2.14s 与缺口同阶，明示）、n=6 批 1.072 FAIL——三处均如实归档，无挑读法（六对原始 Done 值全存日志可复核，record 未省略不利对，−1.03 反向对在列）。
- **跨批漂移已声明**：§4 ⚠️ 整批相对四臂 #2 漂升 ~2s（21 时段负载），跨批绝对值不可比——声明到位。
- **「接受现状收尾」为用户裁决**：record §0/§4 记 HOOK-3 用户裁决 + C-gate 3 轮未满足（B 期 FAIL / n=2 不可判 / n=6 FAIL）触发链完整。judge 无法核验对话现场，以 record 落盘声明为准——**建议候选**：提交信息/12 篇小节复述裁决时保持「用户裁决收尾，e2e 判据 FAIL 未达标」措辞，防止后续被读成「判据通过」。
- 净收表述（7.3→2.94ms、1.25×→~1.07×）与 §9.7 串行 per-chunk 口径同行声明 ✓（探针串行 vs e2e wall 不可换算已明示）。

## 3. 行为门完整性

门链：golden 4/4 逐位（内核）→ DUAL ALL-MATCH（B 输入层，r2/r7/r8）→ DUALP ALL-MATCH（C packed 输出层，r7/r8）→ fallback=0 + paldump 0 节（r9，零回退直证）→ 诊断移除后复编 BUILD SUCCESSFUL（r9 尾）。

- 覆盖面判断：B/C 候选的等价面由「同内核同输入」结构性承载（packed 解码输出 = B 路 blocks9 同一数组）+ golden 逐位 + 双层对拍，门链成立。**两点如实标注**：① DUAL/DUALP 对拍样本 = 4 chunk/轮——r7 期 4 个回退 chunk 曾造成假绿（W1），修复后 4 chunk 覆盖了含位流节，但大样本正确性依赖 fallback=0（全 chunk 走 packed）+ 结构性论证而非逐 chunk 对拍，属「构造性等价 + 抽样直证」组合，可接受但不应表述为全量逐位验证；② r9 `fallback=0` 由 paldump 采样 0 节承载（间接但强：旧收集路径零采样 = 全 packed），无独立显式回退计数器——若后续 1.21.6 packed 移植，建议加显式回退计数（一行 counter，诊断面已有先例）。
- 开关语义：`coreswap.light.oldcollect`（B 层回退）与 `coreswap.light.blockabi`（C 层回退）双层开关 + 对拍失败闩——record §4a 声明保留。语义上 oldcollect 只影响收集路径、blockabi 决定 JNI ABI，两层正交，但**两开关组合态（都开/只开其一）的行为矩阵未在 record 或代码注释集中落盘**——建议在 12 篇 round3 小节给一行矩阵说明（见 §7 风险）。

## 4. §9.7 覆盖面

- 载体（1.20.1 dev loom runServer boot pregen、~400-600 chunk 接管、探针 env 门控默认关）：record §6 + probe-verdict §0 双处声明，贯穿各轮数字 ✓。
- 1.21.6 范围声明（仅 Rust 增量导出、Java 不动、#26 帧格式差故 packed 未做）：与 git diff 一致（1.21.6 仅 jni_bridge.rs，Java 零改动）✓；「1.21.6 光照行为与 round2 完全一致」为声明式（构建绿 + 语义零变化），record 已如实标注「声明式」而非运行时验证——诚实 ✓。
- 探针串行 vs e2e wall 不可换算（#128）+ nativeAvg 含回退全价前提（r7 前）——均声明 ✓。

## 5. §15.4 取代指示（给 P6 知识库 subagent 的具体输入）

被证伪结论定位：`versions/1.20.1/docs/12-lighting.md` **L94** 行尾「→ **Java 收集/JNI 侧开销已基本消除**」及 L94 前半「绝对开销 ≈3.5s ≈ 内核份额预测…吻合」（round2 减法口径把 3.2s 归因内核、Java 段记 ≈0.15ms/chunk）。证伪证据 = r1 探针实测 collect 4.5ms/chunk（30×）+ 对价模型重算（probe-verdict §1.2）。

落盘指示：
1. **12 篇 L94 原文不删不改**，在行尾追加取代标记（supersedes 双指针形态）：`〔260914-04 round3 supersedes：本行「Java/JNI 侧基本消除」归因被探针实测证伪——collect 串行 4.5ms/chunk（30×）；见本篇 round3 小节 + .investigations/light-round3-260914-04/probe-verdict-260914-04.md §1〕`。同理 L101 降级声明处补一行指向（该行自认「Java/JNI 收益未单独微基准」——round3 微基准已补，属补强非推翻，标注即可）。
2. **12 篇新增 round3 小节要点**：判据线 1.05× + HOOK 链（HOOK-2/2b/3）；三源探针（K/T/P）与对价模型（ON 7.3 = collect 4.5 + native 2.8 vs vanilla ~5.6）；B palette 展开 collect 4.527→1.257→0.288（DUAL ALL-MATCH）；C packed 直传 native 2.651（DUALP ALL-MATCH，fallback=0）；e2e FAIL 1.072（n=6，用户裁决接受现状收尾，C-gate 3 轮链）；净收 7.3→2.94ms/chunk、1.25×→~1.07×；golden 4/4 逐位；1.21.6 范围（Rust 增量/Java 不动/#26）；旧 collect/双开关保留面。
3. **10 时间线**追加 260914-04 条（过程链：探针→B→C→W1 假绿→修复→r9 收尾）。
4. **knowledge/discovered 候选**（按价值门筛）：W1 教训（「部分成功比全红更危险」——协议解析改动必须用含位流节用例覆盖，singular 免疫造成假绿）是高价值通用模式；W2/W3（初值-谓词成对核对 / PowerShell 输出流即返回值）判可复用性后简记；round2 减法口径证伪本身进 12 篇即可，不进 discovered。
5. `.artifacts/index.yaml` 补 light-round3 条目（round2 standing 闭环）。

## 6. W1-W4 五段式完整性（record §5）

- **W1**：现象/根因/定位/修复/教训五段齐全，定位链（合成帧单测→分层 rc→帧 dump）与签名（idx=15 v=8 + bufLeft=2 + 0x80 0x02）详实——**达标**，且为四条中知识价值最高。
- **W2**：现象/根因/修复/教训四段，无独立定位段（发现过程未记）——内容简单（初值 0 vs `>=0`），**基本达标**；建议落盘时补一句定位方式（如何发现对拍从未运行）。
- **W3**：现象/根因/修复/教训齐全，定位隐含（汇总行污染即现象即定位）——**达标**。
- **W4**：现象/根因（回退全价 + 批内方差）/教训齐全，无修复段（预测偏差类，无 bug 可修）——**达标**（N/A 合理），教训「回退计数=0 前提下读均值」可操作。

## 7. 遗留风险（不阻塞收尾，须随 12 篇落盘）

1. **global palette（ID_LIST，bits≥15）整 chunk 回退路径**：1.20.1 直方图实测 0 例（r1/r2/r8 bitsHist 4-6 均无 ≥15），但**未证不可能**（源 P 自declared「深土/矿道形态未充分采样」）。回退正确性 = 走原 blocks9 ABI（round1/2 已验证路径），**静默正确性由 rc=-2→闩回退的结构保证**，但若真发生是整 chunk 性能悬崖（W4 实测全价）且无生产计数器可见——建议：12 篇声明 + 后续轮加一行回退计数日志（或 @anchor.idk 声明 90 天线）。
2. **双层开关组合语义**：`oldcollect` × `blockabi` 四组合态（尤其双开 = 全旧路径）无集中矩阵文档——12 篇 round3 小节补一行即可。
3. e2e 载体噪声（boot Done ±1.5s 漂移）已由 record 列为下轮候选（Chunky/光照阶段口径）——保持为待办，不本轮堵。

## 8. 给用户的决策要点（≤5）

1. **三源核对 8/8 PASS**：record 所有关键数字与原始日志逐位吻合，diff 面与声明一致、无未声明改动——产物可信。
2. **e2e 判据 FAIL 是如实结论**（1.072 ≥ 1.05，n=6）；「接受现状收尾」为 record 落盘的用户裁决——judge 建议按此收尾，判据线留给下轮（载体更换后重测）。
3. **行为门全绿但性质如实**：golden 逐位 + 双层对拍（4 chunk/轮抽样）+ fallback=0（paldump 间接直证）+ 复编绿——建议授予 **candidate**（round3 净收数字与等价性证据充分；confirmed 由您拍板）。
4. **收尾前置两件事**：§15.4 取代落盘（12 篇 L94，指示见本意见 §5）+ `.artifacts/index.yaml` 登记（round2 standing 闭环）——均已在 record §7 待办，须在提交前完成。
5. **遗留风险非阻塞**：global palette 回退未证不可能（建议显式计数器/@anchor.idk）、双开关组合矩阵待一行文档——随知识库更新带走。

> 推荐状态：record 及 round3 结论 **draft → candidate（建议）**；confirmed 由人类拍板。
