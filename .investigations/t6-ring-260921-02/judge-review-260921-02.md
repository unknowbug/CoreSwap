# judge-review-260921-02 — T6 ring=1 candidate 审查意见（judge subagent 原文归档）

> 审查对象：260921-02 块 T6 ring=1 补充 probe 的 candidate 升档建议 + 相关落盘。
> 结论：**PASS-with-conditions**（N1/N2 条件已由主会话应用；N3/N4 提示登记）。judge 只出意见不改 status；confirmed 留用户。

## 逐项核对结果

**源 1 — 产物快照**
- record §1 全部 10 条 [FP-EDGE] 引文与 log **逐字一致**（行号 5460-5462/5492-5496/5553-5554、t 值 38064.6/63091.0/63091.2/64439.4-64439.7/74715.5、lvl 34/35/33/-1、status initialize_light/carvers/full/absent 全部吻合）✓
- result.json 与 record §1 SELFCERT 行一致：dll_sha8=cc4e39fe、lightInit_ok/domain_hook/wbq_armed=1、fp_on_edge 等=1、edge_lines_gate_closed=0、drv_lines=0、ring_count=4、stderr_malformed=0、seed+region_min_mtime_gt_rmtree ✓；ring 4 条坐标/档位/相位与 record §2 表完全一致 ✓；criteria_hint/t6_hint 与 record 引用一致 ✓

**源 2 — git HEAD + 工作区 diff**（4 文件均 M，新增 `.investigations/t6-ring-260921-02/` untracked）
1. FormProbe：`edgeRingProbe` 仅 N 相触发（`if ("N".equals(phase))`）、4 cardinal 邻 `{±1,0},{0,±1}`、经 `edgeProbe(..., true)` 打 `ring=1` 标——与 record 描述一致 ✓；BASE/P 行不受影响（ring 参数默认 false）✓
2. docs/12 新小节：C2 段与 confirmed 结论（supersede-draft §2 + verdict :76 指针）**逐要点一致**（level≤33 档位语义、33/34 直证 + ≤32 静态背书、取代双指针、E1 口径声明）；T6 段明确写「**candidate**（judge/confirmed 流转中）」，未把 candidate 写成 confirmed ✓
3. docs/10 260921-02 块：与 record 一致（含对角邻未采样声明、judge 待跑标注）✓
4. record-260920-06：diff 仅追加两行 §15.4 指针（块引用形式），原两 bullet 原文完好，「追加不覆盖」✓

**源 3 — 验证记录**
- design-260920-06 §3.3 预登记原文核对：失败分支「若实测环档 ≠ Chebyshev 预测 ⇒ I 不能摘」逐字在案；本块 4/4 carvers = 预测成立，未改判据、无事后改口径 ✓
- SELFCERT ring 门实现核对（run_formedge_t6.py :141-157/:227-239）：ring_count==4 硬门 + `expect_ring` 与票位 (CX,CZ) cardinal d=1 集合精确核对 + `probe_line` 排除 ` ring=1` 行防 HARDGATE 误抓 lvl=35——三项均落实 ✓
- dll sha：result.json `dll_sha8=cc4e39fe`，驱动 `dll=.*sha256=(\w{8})` 抓取并与常量比对（:199-200），声明与机制一致 ✓

## 重点项

- record §3 覆盖面诚实声明：充分——明确「对角邻 (±1,±1) 未采样、非全域环面直证、不外推」，且与 A7 表静态推演的旁证定位分清，无过度声称 ✓（docs/12、docs/10 同口径转录）
- #215 家族判据口径：驱动 t6_hint（RING_PASS→I removable）与 record §2 判读同口径，hint 为采集侧机械读法、record 未加码 ✓
- 日期/编号：Get-Date = 2026-09-21 17:18，log wall 17:11-17:12，当日既有 260921-01（commit 5b6d468 @15:00）→ **260921-02 = 同日第二块，自洽** ✓

## 发现清单

- **N1（条件 1）**：design §3.3 只预登记了**失败分支**（≠预测 ⇒ I 不能摘）；「4/4 ⇒ I 可摘」是其逆否命题，逻辑有效但**成功分支的升档动作（I 摘除 + candidate）未逐字预登记**。不构成判据漂移（判读方向、阈值、A7 表均未动），建议在 record 或时间线补一行「成功分支为 §3.3 失败分支之逆否 + design §6 R2 独立降权轨道」的显式说明。
- **N2（条件 2）**：run_formedge_t6.py :236-239 中 ring 坐标核对仅在 `ring_count==4` 分支内执行；count≠4 时只报 count 不报坐标差。当前 run 无影响（count=4），建议后续补 else 分支输出 `got_ring` 便于失败轮诊断。
- **N3（提示）**：ring 门验收 4 行 status 一致性靠 t6_hint 字符串而非硬门（RING_DEVIATION 不进 fail 列表）——本轮 RING_PASS 无影响，但若未来把 T6 从辅助读数升为独立判据，须把 deviation 纳入机械 fail。
- **N4（提示）**：record :10 与 diff 中 edgeRingProbe 行号标注「:410-416」，与 diff 实际新增位置（~:409-416）吻合，仅提示行号随后续编辑会漂移。

## 状态建议（不改任何 status）

- 建议 T6 record 保持/授予 **candidate**（judge SHOULD 级审查本轮通过，附 N1/N2 条件）；**confirmed 留用户**。
- docs/12 C2 段为 confirmed 结论的如实转录，无需改动；N1 建议以一行补注方式落盘（主会话应用）。

---
**条件应用记录（主会话，2026-09-21）**：N1 → record §2 已补「升档动作预登记说明」引用块；N2 → run_formedge_t6.py 已补 count≠4 分支坐标差输出；N3/N4 登记不实施（前者条件触发、后者提示面）。时间线 260921-02 块 judge 终态已回写。
