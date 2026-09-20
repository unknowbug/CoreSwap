# judge-review-260920-06 — T5/T6 运行时单验 candidate 审查意见

---
角色: judge（Anchorlaw §15/§16 隔离审查；只出意见不改 status；confirmed 留人类）
审查对象: T5 修正后表述（level ≤33 ⇒ light() 被调；level 34 档 = INITIALIZE_LIGHT 停 light() 之前）candidate 建议 + design-260920-05 §1.1 T5 原表述的 §15.4 取代
依据: design-260920-06.md §3 判据预登记 / record-260920-06.md / formedge-fe2_result.json / formedge-fe2.log（原始抽查）/ git diff / fe1 留档
结论: **PASS-with-conditions**（条件见文末；无 blocking 项）
---

## 0. 三源交叉核对（MUST，§15.4 baseline）

| 源 | 核对结果 |
|---|---|
| 产物快照 | design/record/result.json 三者判据口径一致；判读严格按 design §3.2 预登记读法矩阵落格（双 PASS 格），无事后挑读。原始 log 抽查与 record 时间轴表逐行吻合（行号存在系统性 ±1 偏差，见 info-1） |
| git 工作区 diff | 改动面 = 3 文件：FormProbe.java（+153，EDGE 驱动 + BASE probe + R5 到位确认）、ServerWorldFormProbeMixin.java（放行门 `DRIVE` → `DRIVE \|\| GRID \|\| EDGE`，1 行，即 fe1 缺陷修复）、build.gradle（恰 3 条映射行，驼峰族对齐）。无其他夹带；dll 未动（Q1 cc4e39fe 实测在位，纯 Java 面承诺成立） |
| 验证记录一致性 | fe1 VOID 原名留档（formedge-fe1.log 639KB + formedge-fe1_result.json 均在位，未删未改名），fe2 换新 tag 重采——#144/#146（失败轮原名留档 + 禁自动回退）合规 |

附：judge 现场复跑 `check_switch_mapping.py`（design §2.3 MUST 项，record 未附运行证据）：DEAD=0、#47 作用域 OK、formedge 3 行驼峰族命名一致（1.20.1 驼峰 109），1.21.6 同名缺口按预登记理由成立——门禁通过。

## 1. 逐项核查

### N1 窗口化分界锚（probe phase=N t=60598.0）— 通过，附 should-fix

代码面核实（FormProbe.java edgeTick case 2）：probe N 打印与 remove(34)/add(33) 发生在**同一 serverTick 内、probe 之后数行**——故「N 窗 = t<60598.0」与「N 窗 = P 切票之前」之间存在一个**同 tick 缝隙**（probe 行到切票行之间）。record §2.1 仅以「P 切票 ≈ t 60598（probe N 后）」隐式登记该近似，**未显式登记缝隙方向**。

缝隙方向分析（judge 补做）：WBQ 行无 t 字段、按 log 位置归窗——若 light 行落在缝隙内会被误归 P 窗，理论上有掩盖 N FAIL 的方向。但本 run 该风险被三重闭合：① probe N `status=initialize_light` 运行时自证边界时刻 light 未跑；② 目标 chunk 唯一的 [FP-LIGHT] call 带 t=70860.6 ≫ 60598.0（切票后）；③ WBQ enter（#5537，无 t）之前夹有 P 相 ring 扩张的邻 chunk fill 行（#5489-5536，t=60617–60788，33 票 FULL 族扩环触发），enter 在其后——「enter 发生在切票后」有独立旁证。本 run 判读成立，但近似本身应显式登记（条件 C1）。

### N2 对照区充分性 — 通过

spawn 环境 18 条 light 行的作用是**仪器面存活对照**（证明 N 窗打点面未整体死），不是同驱动对照——这一区分 record 已如实表述（「判据面在 N 窗存活，非打点窗整体死」）。真正的同驱动对照由实验结构自身提供：**同 chunk 同 run 双相自对照**（34 票 vs 33 票，唯一变量 = 票档，E1），比任何外部对照区更强。加 #81 成对互证（enter/call 各 1 同窗出现，529+1 与 e3a_lines=530 自洽）。判定：对照设计充分，无需补同驱动对照区。

### N3 BASE 基线解读 — 通过

BASE probe（t=59388.5，arm 前）`lvl=-1 status=absent`。关键排雷：FormProbe edgeProbe 反射失败时同样输出 lvl=-1（且打 `holder-reflect-FAILED` 行）——judge 已核 fe2 log 全部 [FP-EDGE] 行，**无任何 FAILED 行**，故 -1 是真「holder 不存在」读数而非反射失败的缺省值。(160,96) 加票前无 ambient 票据污染成立，设计 §7.5 兜底项落实有效。

### N4 CRITERIA-HINT 缺陷处置 — 通过，附 should-fix

record §6 如实登记 hint 全窗计数缺陷（hint 判 N_FAIL vs design 口径 N PASS），并明确「判读权威 = design 预登记，hint 非判据载体」——分离正确，本 candidate 不受影响。**重跑 hint 非必要**（hint 是提示性输出，重跑不产生新数据层证据）；但修复必须先于下一次使用该脚本的采集，防下轮误触发 fan-out（条件 C3，已列 record 未决项 1）。

### N5 T5 取代表述措辞 vs 证据强度 — 通过，附 should-fix

运行时直证面 = **恰两点：level 34（停 light 前）与 level 33（进 light）**，单 chunk (160,96) 单 seed 单维 E1——33/34 边界两侧的贴界差分证据是硬的。但表述「**level ≤33** ⇒ light() 被调用」中「≤33」侧其余档位（≤32）并未逐档实测，靠静态链 A5（level<33 → FULL）+ A8（level 22 海量间接背书）支撑。§9.7 声明已禁外推 seed/点/档位组合，与「≤33」全域措辞之间存在措辞-证据张力。不构成 blocking（静态链一手源已核 + 22 档有实测旁证），但取代登记落盘时应限定措辞（条件 C2）。

### N6 T6 维持 I 处置 — 通过

本 run 未实现 ring=1 补充 probe（design §3.3 标注「可选实现」），record 将 T6 维持 I + 独立降权、不随 T5 升档绑定——与 design §6 R2 预案（「T6 判读独立降权，不绑定主判据」）完全一致。处置正确。

## 2. info 项（不阻塞）

- **info-1 行号 ±1**：record 时间轴所引 log 行号系统性偏 1（如 probe N 记 #5487 / 实测 #5488；WBQ enter 记 #5536 / 实测 #5537；FP-LIGHT 记 #5541 / 实测 #5542；FP-FILL 记 5458/5461/5480 / 实测 5459/5462/5481）——疑 0 基/1 基计数差。相对次序不变、不影响归窗结论，但 record 以「log 位置」作归窗依据，行号引用宜校正或注明计数基（随 C1 一并处理即可）。
- **info-2 设计改动面预测偏差**：design §2.3 称「改动文件 = 仅 2 个」，实际 3 个（mixin 放行门为 fe1 缺陷修复，design 时点未知）。该偏差已在 record §7 错误记录中如实登记（根因五段式完整），属合规事后登记，非夹带。
- **info-3 hint 字段残留**：formedge-fe2_result.json `criteria_hint` 字段值（N_FAIL）与判读结论相反且未在 json 内标注作废——判读权威在 record 已足够，但建议修复 hint 时让失效 hint 带自标识，防裸读 json 误判。

## 3. 其余检查项（anchor-judge 清单）

- 置信度状态：record/design 均 draft，candidate 为建议非自授，无违规 confirmed。✓
- 产物契约：judgment 落盘 .investigations/，log/json 落 .tmp 唯一临时区。✓
- 判据前置集（§15.1）：Q1-Q8 全绿，硬门 lvl 运行时直读非参数回显（probe 代码路径 reflection 实读 holder level 已核实）。✓
- §9.8 副作用与逆：design §4 登记齐全，fe1→fe2 换 tag 即逆的落地形态。✓
- §9.7 口径：E1 + S={(160,96)} 三面声明完整，实测一致。✓
- retry cap / PI-1：不适用（无 halt，fe1 VOID 走 SELFCERT 机械拦截 + 换 tag 重采，未静默继承污染产物）。✓
- fan-out 触发：双 PASS 落格，未触发读法矩阵 fan-out 分支，合规。✓

## 4. 结论

**PASS-with-conditions**。T5 修正后表述的双 PASS 判读经原始 log 抽查独立复核成立（N 窗目标 chunk 光照行确为 0、probe 自证 initialize_light、P 窗 enter/call 成对且 probe 自证 full），candidate 升档建议与 §15.4 取代登记动议**可交人类拍板**。条件（均为 should-fix，落盘取代登记时一并处置，不阻塞本 candidate）：

- **C1**：record 补登 N1 同 tick 缝隙近似（probe N → 切票之间无 t 缝隙、WBQ 归窗靠 log 位置、本 run 由 probe 自证 + call t 值 + ring fill 旁证三重闭合）——落盘取代登记时写入；顺带校正 ±1 行号。
- **C2**：T5 取代登记措辞限定证据强度：33/34 为运行时直证；「≤33 其余档」标注为静态链 A5 + A8（level 22）间接背书，不写成全域直证。
- **C3**：run_formedge.py CRITERIA-HINT 窗口化修复须先于下一次使用该脚本的采集（已列 record 未决项 1，本 verdict 不需重跑）。

confirmed 授予权留人类。
