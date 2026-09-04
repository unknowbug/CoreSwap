# judge 审查意见 — 260904-03 P2 回归收尾交付（core.judge，Anchorlaw §15/§16 参考实现）

- 审查对象：`.artifacts/lossless-accel/p2full-regression-verdict-260904-03.md`（candidate）+ 配套登记 + 知识库落盘 + 原始证据
- 审查方式：三源核对（.artifacts 快照 / git HEAD+工作区 diff / 验证记录）+ 逐项数字核对 + 脚本↔导出器格式核对
- 审查日期：260904-03（judge 会话，实际宿主时间）
- **总评：PASS-with-CONCERN**（1 项 MUST 清偿 + 2 项 SHOULD；核心结论证据链成立，可建议保持/授予 candidate，confirmed 留用户）

---

## 一、三源核对（基线 1）

| 源 | 结果 |
|---|---|
| .artifacts 快照 | verdict 存在、index.yaml 新条目 `estopt:p2full-regression-260904-03`（kind: analysis, status: candidate）✓；260903-13 末尾闭合指针为**追加**（diff 确认原正文未删改）✓ |
| git HEAD + 工作区 diff | 工作区改动 7 文件 +113 行（index.yaml / 260903-13 闭合指针 / errors LL12-LL13+速查表 / INDEX.md / workflow-patterns #32 / 07 篇小节 / 10 时间线条目）+ 5 个 untracked 证据/草稿文件——**快照与落盘一致，无滞后**（HEAD d6b04fe 不含本轮，全部为待提交工作区态，与交付声明相符）✓ |
| 验证记录 | cmp_full2 / arm-compare / run1b / run2 / run3 / cmp_full-misaligned 六份均在 `cmd-output/` 落盘，非 untracked 缺失 ✓ |

## 二、验证记录逐项核对（基线 2）

- **数字核对**：`cmp_full2-260904-03.txt` = 13328/1572864、99.1526%、16/16 chunk（450–1347 分布）、top 对 ref=1→9 n=4165 / 37→34 n=1687 / 1→34 n=1581 / 37→9 n=1359 / 32→0 n=520——verdict / 07 篇「存档口径残差模式化」/ 10 时间线条目**逐项一致** ✓。
- **seed/四要素**：run1b/run2/run3 日志均 `[BlockProbe] worldSeed=8576294172403134396`；cmp_full2.py 断言双侧 header（magic/seed/size/origin/min_y/h）一致 + trailing bytes=0（assert 通过才会输出）✓。per-chunk 键 (12..15,12..15) 与 origin=(200,200)→chunk 坐标换算一致 ✓。
- **脚本↔导出器格式**：`cmp_full2.py` 与 `runtime/1.20.1/java/src/main/java/wg/bench/BlockProbe.java:477-938` 逐项对上——header 32B（magic i32 `0x57474232` + seed i64 + size/ox/oz/min_y/h 5×i32）、per-chunk 8B（wx/wz i32）+ 98304×2B writeShort（DataOutputStream big-endian，与脚本 `>i/>H` 一致）+ 256×writeUTF（2B 长度 + UTF，脚本同构解析）✓。FULL 口径由 `-DblockProbe.full=true`→ChunkStatus.FULL 确认（L895-896），「存档口径 FULL」声明成立 ✓。biome 差异单独检测且输出中无 biome 差条目（脚本会追加 ` biome` 键），未混入 cell 计数 ✓。
- **AQF-J NPE**：run1b/run2/run3 日志均见每 chunk `[AQF-J] failed NPE`——「两臂同现、既有噪声」声明与日志一致 ✓。
- **生产 dll**：run2 日志 `[CppBridge] dll=...worldgen.dll size=1897984 sha256=ec4a9aedf4104788...`——与 verdict `EC4A9AED…C8FC3` 前缀一致（但**日志与 verdict 都是截断值**，全串两侧均不可引用，见 SHOULD-2）。
- ⚠️ **MUST-1（同 hash 证据引用断裂）**：verdict 结论 1 与 10 时间线均写「Run2 与 Run3 SHA256 完全一致 `2FF71249E9C5CF9745CAC95BC27DC011D53F91C5C454C042E2DF9C9B23F210BC`，`cmd-output/arm-compare-260904-03.txt`」——该文件实际内容仅一行 `identical: True`，**全仓库（含 .tmp）检索不到该 64 位 hash 值的任何落盘出处**。声称本身（identical: True）有落盘支撑且「无回归」核心结论不因此动摇，但「具体 hash 值 + 出处文件」是**不可引用的证据引用**（judge §落盘自证力：引用的每条核验事实须可在引用文件中复核）。清偿：① 在 `cmd-output/` 落盘实际 hash 计算输出（两臂各一行完整 SHA256），verdict 引用改为该文件；或 ② verdict/时间线删除具体 hash 串，改为「arm-compare-260904-03.txt: identical: True」。二选一，改动极小。

## 三、置信度 / 落盘契约（基线 3）

- **status=candidate 合法** ✓：有 Full 分层验证证据（逐位对比），无越权 confirmed；draft→candidate 由 judge SHOULD 审查覆盖（本意见即）。
- **§9.7 三要素齐备** ✓：载体（WGB2 FULL 存档口径 4×4 单区域）/ 覆盖面（16 chunk×98304 cell）/ 可比性（与 260903-13 角列口径不可比、与 260903-14 同族不可数值对齐）verdict 与 07 篇双落点一致。
- **局限如实声明** ✓：Run3 daemon env 死判别（LL13/#32，降级为旁证且声明核心结论不依赖）、Run1 覆盖瑕疵（LL12，run1b 重采闭环）、AQF-J NPE——三条均与日志/草稿核实一致。
- **环境遗留交代** ✓：残留 java 进程/world 状态未构成混淆源（本轮为文件级对比，参照四要素核对通过）；「日期以 git/日志时间戳锚」符合编号纪律。
- **知识落盘程序合规** ✓：4 份草稿在 `knowledge-drafts/260904-03/`，verdict 与 draft-p2full-verdict 逐段比对为忠实应用版；LL12/LL13 五段式齐备（现象含具体数据/根因机制层/定位含方法/修复/教训）+ 速查表两行已同步；#32 五段式 + 同族引用（#20/#14）合规；INDEX.md 追加行 ✓。
- **记录价值门**：#32（env 判别死同值——可复用判据，高价值）与 LL12/LL13（错误链条）过门 ✓；07 篇小节为既有 260903-14 记录的**模式补充**而非新结论灌水，措辞已自证 ✓。

## 四、与 260903-13/14 结论链自洽（基线 4）

- **无 §15.4 supersedes 需求** ✓：本轮闭合 260903-13 遗留第 1 条（「如翻默认后需存档口径 Full 证据，下轮补」），是遗留项闭合非结论推翻；verdict `supersedes: 无` 声明正确；07 篇小节明确「量级读数互相兼容、补充非取代 260903-14」✓。
- **闭合指针措辞** ✓：260903-13 末尾以 `> [闭合指针 260904-03]` 追加（diff 确认 +2 行，原 31-33 行未动）——符合「追加不覆盖」；指针指向 verdict 并复述核心数字（99.1526% / 同 SHA256），数字与 cmp_full2 输出一致。
- **逻辑自洽** ✓：「default==off 同输出 ⇒ 残差与 est 无关」推断成立（即便按局限①的保守读法，Run2 vs Run1b 的残差对比独立闭合核心结论；Run2==Run3 只是旁证——verdict 措辞已按此降级，无过度声称）。

## 五、逐项清单

| # | 项 | 级别 | 结论 |
|---|---|---|---|
| 1 | 证据落盘完整（6 份 cmd-output + 脚本 + 4 草稿） | — | PASS |
| 2 | 三源一致（快照/git diff/验证记录） | — | PASS |
| 3 | 数字逐项一致（verdict/07/10 vs cmp_full2） | — | PASS |
| 4 | cmp_full2.py ↔ BlockProbe.java 导出格式 | — | PASS |
| 5 | seed 三查 + 参照四要素 + trailing=0 | — | PASS |
| 6 | §9.7 三要素 + 局限如实 + candidate 合法 | — | PASS |
| 7 | 260903-13 闭合指针追加不覆盖 / 无 supersedes 需求 | — | PASS |
| 8 | LL12/LL13 五段式 + 速查表 / #32 / INDEX 追加 | — | PASS |
| 9 | **同 hash 值 `2FF71249…` 无落盘出处（引用文件只有 identical: True）** | **MUST** | 清偿后 PASS |
| 10 | 生产 dll sha256 两侧均截断（日志 `ec4a9aedf4104788...` / verdict `EC4A9AED…C8FC3`），全串不可引用 | SHOULD | 补落盘完整 hash 或统一引用口径 |
| 11 | run1b「blocks ->」目标路径未在已核日志行中直接确认（ref\ 下文件存在 + header 核对通过，间接成立） | SHOULD | 下轮采集时在对比脚本/输出中回显参照文件路径 |

## 六、推荐

- **verdict 建议授予 candidate**（当前即 candidate，维持）；**confirmed 留用户拍板**。
- MUST-1 清偿属引用修正级改动（≤2 文件各 1 行），不动摇结论；清偿后本意见可视为全项 PASS。
- judge 不改任何 status、不写结论性 docs——本文件仅为审查意见。
