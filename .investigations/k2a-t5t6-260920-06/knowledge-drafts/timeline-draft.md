# timeline-draft — 10-timewise-archive.md 追加「260920-06 块」全文（subagent 草稿，主会话应用）

> 应用方式：整段追加到 `versions/1.20.1/docs/10-timewise-archive.md` 末尾（260920-05 块之后）。格式对齐既有块。

---

## 260920-06（实际 2026-09-20 深夜跨 00:00，宿主 Get-Date 已核至 09-21 00:1x）T5/T6 运行时单验（formedge 双相臂）——off-by-one 口径修正（level 33 → 34 负臂）/ fe1 VOID（mixin DRIVE-only 放行门 + 上块 grid 寄生 formdrive 开关）/ fe2 双相 PASS / T5 candidate（judge PASS-with-conditions C1-C3）

> 承 260920-05（T5/T6 运行时单验 = C2 措辞升档前置）；design `.investigations/k2a-t5t6-260920-06/design-260920-06.md`（判据预登记 Q1-Q8 + 判据 N/P + 读法矩阵）；record 同目录 record-260920-06.md；judge-review-260920-06.md（PASS-with-conditions C1-C3）；采集 `.tmp/k2a-t5t6-260920-06/`（fe1 VOID 原名留档 + fe2 重采）。

### 过程链

- ✅ **目的与立项**：260920-05 判读 C2（「light() 覆盖 = workload/ticket 覆盖面的直接函数」）升档前置 = T5/T6 运行时单验——T5 当时为 I-静态推演（design-260920-05 §1.1「level 33 档目标 = INITIALIZE_LIGHT 停 light() 之前」）。设计 = 受控单 chunk (160,96) 双臂：负臂票据恰停 INITIALIZE_LIGHT 观测 light() 不被调，正臂升档观测 light() 被调；单 run 双相、同 chunk 成对日志（贴界最小差分，E1）。
- ✅ **34:33 口径修正（用户批准；design 头号发现）**：设计定稿时一手源核对发现原 T5 表述 off-by-one——`ChunkLevels.getStatus(level)`（ChunkLevels.java:11-13）：`level < 33 ? FULL : byDistanceFromFull(level - 33)` ⇒ **level 33 → byDistanceFromFull(0) = FULL**（ChunkStatus.java:303-308 + 203-216），LIGHT 在 FULL 之前必须完成 ⇒ light() 会被调；**恰停 INITIALIZE_LIGHT 的档 = level 34**（byDistanceFromFull(1)）。原表述把 33 当成第一个非 FULL 档，属推演链套用出错。负臂/正臂改为 **34:33**（用户拍板确认 34 口径；取代登记候选随实验结果 judge 后落）。
- ❌→✅ **fe1 臂 VOID（五段式，record §7）**：`[FP-ON] ... edge=true` 打出（开关链通）但 `[FP-EDGE]` 行 **0 条**——根因 = FormProbe serverTick 分发放行谓词 **DRIVE-only**（只认 formdrive 族开关，edge 分支 init 打印却不可达，「开关家族扩展漏改门卫」结构错；B6-1 映射门是必要非充分）。SELFCERT 硬门 `edge_init:0` 机械拦下整轮 VOID（防「开关打了 = 功能跑了」假阳性判读）；fe1 log/json 原名留档不自动回退（#144/#146）。修复 = 放行门 `DRIVE` → `DRIVE || GRID || EDGE`（ServerWorldFormProbeMixin 1 行）。**git 历史核清副产物**：260920-05 的 grid 臂实际是在 formdrive 开关**寄生开启**状态下运行的（run_fullcov.py:35,37 同时传了 formdrive=1，放行谓词恰被 formdrive 满足，grid 时序得以走通）——上块 grid 数据面的驱动通路实为 formdrive 门放行，非独立 grid 门（同根缺陷的历史已发面）。
- ✅ **fe2 双相 PASS（SELFCERT Q1-Q8 全绿，运行时直读非参数回显）**：BASE probe lvl=-1/status=absent（票位无 ambient 票据，干净基线）；probe N **lvl=34 status=minecraft:initialize_light**（N 窗目标 chunk [WBQ] enter = 0 且 [FP-LIGHT] call = 0，FP-FILL 三行齐全 = 生成链活着、light 未跑）；probe P **lvl=33 status=minecraft:full**（目标 chunk 首条 [FP-LIGHT] call **t=70860.6** 落 P 相 30s dwell 窗内，enter/call #81 成对）；N 窗对照区 spawn 环境 chunk **18 条 light 行**证明判据面在 N 窗存活（非打点窗整体死）。
- ✅ **判读：N PASS + P PASS ⇒ T5 candidate**（读法矩阵「双 PASS」格机械落格，无 fan-out 触发）：运行时直接证实 33/34 为边界两侧——level 34 档 = INITIALIZE_LIGHT 停 light() 之前；level 33 档 = FULL ⇒ light() 被调。T6（环档 Chebyshev 线性加距）本 run 未打 ring=1 补充 probe，维持 I 标注独立降权。
- ✅ **judge PASS-with-conditions（C1-C3，无 blocking）**：三源交叉核对（产物快照 + git diff 3 文件无夹带 + fe1 留档合规）；C1 = 补登 probe N 与 P 切票同 serverTick 的归窗缝隙近似（本 run 由 probe 自证 + call t 值 + ring fill 行旁证三重闭合）；C2 = T5 取代措辞限定证据强度（33/34 直证、「≤33 其余档」= 静态链 A5+A8 间接背书）；C3 = run_formedge.py CRITERIA-HINT 全窗计数缺陷须先于下次采集修复（hint 误报 N_FAIL，判读权威 = 预登记判据文本）。
- ✅ 知识库：workflow-patterns #215 + build-tooling #162 + record C1/C2 补丁 + design-260920-05 §1.1 T5 §15.4 取代登记 + INDEX + 本块（subagent 草稿 → 主会话应用）。

### 结论与状态

- **T5（修正后表述）**：level 34 档 = INITIALIZE_LIGHT 停 light() 之前、level 33 档 = FULL ⇒ light() 被调——33/34 贴界对获运行时直证（单 chunk 单 seed 单维 E1）；「level ≤33 其余档」= 静态链 A5 + A8（level 22 实测）间接背书（judge C2 措辞限定）。状态 = **candidate**（judge PASS-with-conditions C1-C3；**confirmed 留用户**）。
- fe1 错误台账教训：① SELFCERT 前置集机械拦 VOID 价值实证；② 新增开关分支 MUST 审查既有放行谓词（不止加 getProperty 消费点）；③ 历史正结果的「靠哪个门放行」值得核清（寄生接线防把巧合当公理，→ build-tooling #162）。

### 产物

`.investigations/k2a-t5t6-260920-06/{design,record,judge-review}-260920-06.md` + `.tmp/k2a-t5t6-260920-06/{formedge-fe1.log,formedge-fe1_result.json,formedge-fe2.log,formedge-fe2_result.json,run_formedge.py,analyze_fe2.py}`。
