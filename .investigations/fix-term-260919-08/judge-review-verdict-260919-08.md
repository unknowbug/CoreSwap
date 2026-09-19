# judge review verdict — 档② C1 判别实验（260919-08）

## Conditions 应用记录（主会话补证，2026-09-19 21:5x）

- **C-N1（已应用）**：`c1-criteria.md` mtime = **21:27:27** < `t8-domain-r4.log` CreationTime = **21:30:22** → 判据预登记先于采集，机械成立。备注：比较器最终版（A044031A）mtime 21:52 晚于采集，系两轮解析层 bug 修复（IndexError → 赋值评估序错位 + `_i8` 光标停滞），判据层零改动；修复链证据 = cmd-output/o1o3-r3r4-run.log 三轮历史原文（VOID 轮 #144/#146 留档），C-N1 主旨（预登记先于采集）满足。
- **C-N2（已应用）**：`Get-FileHash o1_o3_cmp.py -Algorithm SHA256` 前 8 位 = **A044031A**（366 行，与 judge 实数一致）。
- **C-N3/C-N4**：非阻塞，随比较器下次改动顺带处理，不单独返工。


- 审查对象：.artifacts/fix-term-260919-08/verdict-260919-08.md（draft，建议 candidate）
- 判据基线：.investigations/fix-term-260919-08/c1-criteria.md（#195 四件套预登记）
- 审查角色：core.judge / anchor-judge——只出意见，不改 status，confirmed 留用户
- 三源核对：产物快照 ✓ / 原始读数 ✓ / 采集载体 ✓（逐条见下）

## 结论：**PASS-with-conditions**

总评：数字链、降级声明、判别边界、§9.8 逆登记均经原始读数独立核对成立；仅两项机械核对（比较器 sha256、判据-采集 mtime 时序）因 judge 会话无 shell 无法独立复算，需主会话补一行证据，不影响判读内容本身。

## 逐审查点核对结果

- **N1 判据预登记完整性 — 内容层 PASS，时序待补证（C-N1）**
  - 14 项 preconditions（P01-P14）齐备且每项带 key/expected/check（c1-criteria §1）；O1 两分支读法（§2.3）+ NO-JITTER 分支（§2.1）+ O3 降级读法（§3.2）全部预登记写死，无灰区（#154 声明在案）。
  - 两臂 log SELFCERT 5 项已逐一回原始 log 核实：r3（t8-domain-r3.log L150-152/L2253-2273/L15208）与 r4（t8-domain-r4.log L150-151/L2257-2278/L15213）均命中 lightInit ok / hook armed / sha256=cc4e39fe / seed=8576294172403134396 / FP-DRV seq=5。P05 leveldat 缺失降级分支按预登记走 log 转引，[NOTE] 两行在 log 与 json notes 中一致携带 ✓。
  - ⚠️ C-N1：「判据定稿先于 r4 采集」的 mtime/git 时序证据本 judge 无 shell 不可独立复算——判据自称「mtime 序可核 #112」，请主会话补一条机械核对记录（c1-criteria.md + o1_o3_cmp.py 的 mtime 均早于 r4 log 首行时刻 21:31:03）。
- **N2 数字链 — PASS**：verdict 全部数字回原始读数逐一对上——9450/0 与 0.0000%（o1o3-r3r4-run.log L39-40）、gap=0/hash_mismatch=0（L42-43）、jitter=6453/stable=69448/one_arm=137（L44 = json L11-13）、diff-nibbles=3354675/band=3340941(99.59%)/interior=13734(0.41%)（L45 = json L6-8）。3340941+13734=3354675 自洽 ✓。
- **N3 降级声明 — PASS**：O3-DEGRADED 语义「缺失 ≠ 反对」在 log L48 与 verdict §3 均按判据 §3.2 原文携带，且 §3 显式声明「覆写方向性未证」；band 子句近恒真的诚实声明在判据 §2.2（预登记时即声明）与 verdict §1 ⚠️（复述「不得据 band% 单独宣布通过」）均保留 ✓。
- **N4 口径不可直比 — PASS**：jitter=6453（section 级）与 2185（chunk 级）不可直比声明在 verdict §6 明示；SCOPE 行（log L36）同步声明 per-section sha256 与 per-chunk 16hex 不同粒度 ✓。
- **N5 §9.8 逆登记表 — PASS**：verdict §7 五行覆盖 r3/r4 region 归档（regions\r3\ 与 \r4\ 各 18 文件已 glob 实数）、rmtree、.bak-formprobe、VOID 轮日志保留、比较器只读无逆可登记面；「无自动回退 #146」显式声明 ✓。VOID 轮处置合规：log 保留三轮历史（第 1 轮 traceback 崩溃 + 第 2/3 轮 [VOID] malformed 75.13%），verdict 引用的全部数字均出自最终 PASS 轮（L36-49），VOID 轮按 #160 断链未进判读、按 #144/#146 留作判错链（§7 行 3 登记 _i8 停滞/赋值评估序两 bug 过程证据）✓。
- **N6 判别结论边界 — PASS**：verdict 通篇严格执行「O1 过门只裁决 2185 抖动面」——§0「2038 恒定面本档不裁决」、§2 落位与「两者并存」分支重申、§2 对乙的削弱限定为「对 2185 面的解释必要性」且明示乙的恒定面主张完好、§2.3 写死「不构成升 confirmed 的充分条件」。未发现任何句子暗示恒定面已裁决 ✓。
- **N7 抽样/自洽 — PASS（json 内部一致性核对）**：① y_hist 15 项求和 = 6453 = jitter 计数（逐项相加核实）✓；② band+interior = diff-nibbles ✓；③ jitter+stable+one_arm = 76038 = 两臂候选节总数，与 9450 chunk × 平均 ~8 个 lit section 量级自洽 ✓；④ json rc=0/status=OK/o3=null 与 log 最终轮输出逐字段一致 ✓。原始字节级坐标回溯（region 槽位→(cx,cz,Y,channel) 抽点）无 shell 不可执行，json 内部一致性无矛盾。

## Conditions（candidate 授予前/后补证，不改判读）

- **C-N1**：主会话补一条机械核对：c1-criteria.md 与 o1_o3_cmp.py 的 mtime（或 git 时序）早于 r4 采集首行时刻，落实「预登记先于采集」。
- **C-N2**：verdict 声称比较器 sha256 前 8 位 = A044031A（366 行已实数核实 ✓）——本 judge 无 shell 不可复算 hash，请主会话补 `Get-FileHash` 一行留档。
- **C-N3（非阻塞）**：最终轮 log 无独立 [P13] 输出行（P13 通过以「无 [VOID] + exit 0」间接体现）；建议后续比较器版本为 P13 加显式输出行，便于审计。
- **C-N4（cosmetic）**：verdict §7 行 3 表格单元格「derived（…） |VOID 轮输出…」缺分隔空格，排版瑕疵不影响语义。

## 状态建议

同意 verdict 建议：**candidate 可授予**（O1 门 = 预登记判据的 candidate 门，判据自身预登记了该授予语义；本审查为 SHOULD 级 candidate 前审查，已通过）。confirmed 留用户；2038 恒定面与 R6 清零仍归档③ E-3a，与 verdict §8 待办链一致。
