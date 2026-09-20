# judge-review-260920-05 — 全覆盖驱动判别臂 verdict 草稿 MUST 级审查

status: 审查意见（只出意见，不改任何 status；confirmed 留人类）
审查对象（三源）：
1. verdict 草稿 `.artifacts/k2a-fullcov-260920-05/verdict-260920-05.md`
2. git 工作区 diff（FormProbe.java / WbQualStats.java / build.gradle，本审查逐行直读）
3. `.investigations/k2a-fullcov-260920-05/{design,record}-260920-05.md` + `.tmp/k2a-260920-05/` 原始产物（cmp2-fullcov02.json、d1_recompute.py + d1-recompute-out.log、verify_design.py + log、fullcov01/02 result.json；123MB log 未逐行重读，机械复核采信 d1-recompute-out.log + cmp2 json 双源互证）

**总体判定：PASS-with-conditions。** D1 读数、Q 门执行、VOID 处置、代码 diff 形态全部经得起核对；但存在 2 项 MUST 级条件（cov 门失配的裁决归属 + verdict §4 一处证据误引），须在 candidate 授予前由人类/修订解决。

---

## N1（MUST）— cov 门失配：门的字面违反不得由 worker「意图满足」论消解，须人类裁决门偏差

- 机械事实（独立复核 d1_recompute.py:129-138 逻辑 + d1-recompute-out.log:24-25）：cov(face_new)=0.8556 < 0.90（lag2 收紧门）；cov(face_hist)=1.0。
- design §3.3 门文本写死 `cov(face_new)`，且 `< 0.90 ⇒ 判据 suspended → exit 5`。**按预登记字面，本臂判据处于 suspended 形态。** §112/#195 禁事后挑读法：worker §8 的「门的本意——驱动铺满判读面——满足，登记不 VOID」是对预登记门的**事后重解释**，超出 worker 权限。
- 缓解面（如实登记，方向上支持从宽）：① 比较器 exit_code 实际 =5（机械信号在位，非冒充 PASS 的 exit 0），PI-1 形态满足；② verdict 是 draft 判读建议而非 PASS 授予；③ 判读面 N520∩face_new（=520 点）实测全部在 waypoint 覆盖集内（worst Chebyshev 在 N520 上 ≤10，S2 face_hist=1.0 与 N520⊂face_hist 互证），D1 ρ=1.0 的读数本身不受 face_new 外延区 13k+ 未覆盖 chunk 影响。
- **意见**：门失配的最终裁决（按字面 suspended 待 D-Prune 式修正复跑，还是修订门口径为 cov(N520∩face_new)/cov(face_hist) 并走取代链登记偏差）**必须交人类拍板**。在人类裁决前，C2 candidate 建议为**有条件建议**。本审查推荐采纳修订路线（门意图确为「判读面铺满」，face_new 实证尺寸远超设计假设是门覆盖面假设失效，不是驱动缺陷——worker §8 的事实分析本身成立），但程序上必须以「预登记门偏差 + 人类确认」而非「worker 判读不 VOID」的形式落地。

## N2（MUST）— §3.2 分母写死 face_new 未被执行：双口径并算对 D1 足够，但 face_new 口径 7 类分账恒等式从未计算

- design §3.2 字面：「新臂分账分母 = face_new，恒等式 |face_new| = Σ7 类终态」。实际比较器 ledger 分母 = face_hist（2038）；|face_new|=104025 的 7 类分账不存在；Q7 新臂恒等式只对 face_hist 口径成立。
- 双口径补救评估：**对 D1 充分**——N520 ⊂ face_hist ⊂ face_new，两口径交集同为 520、ρ 同为 1.000000（d1 log:6-9 双行读数），verdict §1「结论对口径选择不敏感」的声明就 D1 而言被证据支撑。
- 但**「新臂 never-enter = ∅ / 缺席被消灭」仅在 face_hist ledger 口径为真**：face_new 的独立 log 口径下有 13816 个 chunk 无任何 [WBQ] 行（d1 log:12）。verdict §4 已登记「sparse 采样面、登记不判」，登记诚实；本条要求：**结论措辞（§0 一句话、§7 环 2「new-arm never-enter=∅」）必须保留口径限定词**（现文大部分带「face_hist ledger 口径」，修订时不得脱落）。
- face_new∖face_hist 区零/低覆盖的外推：verdict 未对该区做存在性外推（§1 明示不外推），合规。13816 无 [WBQ] chunk 不能被引用为「新臂仍有大量缺席」的反证（该区本在网格设计域外），verdict 处理正确。

## N3（MUST）— verdict §4 子组表误引自家证据：机械「其余」= 180，非 179

- d1-recompute-out.log:16 与 :20 均为 `rest=180`（198+143+180=521，因 z188 带与框外**交集 1 点**，非不交划分）；verdict §4 表写「其余 179（相应差 1）」并声称 198+143+179=520 的平移关系——这与它自己引用的复算产物不符，属 #90 转抄家族新实例（verdict 自家的勘误项里再次发生转抄差）。
- 正确表述：机械划分 = 框外 198 ∪ z188 带 143（两者交 1），其余（纯）= 180；record 的 142/180 → 机械 143/180，差仅在 z188 带 ±1（142 = verify_design `zline188_in=142` 的框内口径被 record 误标「含框外」，比较器 h1_h2.zline188.members=143 互证全含口径）。
- 三组恢复率在任意口径下均为 1.000，D1 分母不受影响——**判定不变，但该误引必须在 candidate 前修正**（证据引用与落盘产物一致性是 verdict 交付质量底线）。

## N4（PASS）— lag2 混杂方向性论证成立

- 恢复证据 = 逐 chunk 实际存在的 `ev=wb` 行 + N520 全部出现在终态 lit snapshot（face_new），是观测事实非推断；卸载竞态的机制通道（摘票→卸载→light 前终止）只能消灭 wb 行/制造缺席，无「制造假恢复」通道。verdict §6 方向性分析成立，残余混杂面登记诚实（§6-①②）。
- INFO：record-260920-05 与 FormProbe 注释称 D-Lag 滞留「≈45s+2×GRACE」，机械为 L=2 × dwell 15s = **30s**（design §2.1 原文即 ≈30s）——record/代码注释的 45s 为笔误，建议顺手修正；GRACE 余量结论（30s > 10s+裕量）不受影响。

## N5（PASS）— fullcov01 VOID 轮处置合规

- 原名留档（wbq-fullcov-fullcov01.log 81MB + result.json 在盘，本审查已核：wbq_sum_present=0 / drv_move_total=168 / regions_archived=0）；不进判据声明明确（verdict §10.1），且「崩溃前 enter 65454 方向性读数」显式声明未作证据使用。§9.8 逆登记在位（record §9.8 增量表）。D-Lag 切换 = design §2.1/§6 R1 预登记变体 + ticket_release=lag2 / cov 收紧 / 混杂标注三项声明齐全（result02 `grid_lag2=1` + init 行 `lag=2` 自证）。

## N6（PASS）— Q7 恒等式/负自证/残缺率门在 30 万行量级执行证据在盘

- cmp2-fullcov02.json：q7 mismatches=[]、wb_lines = e3a_lines = 101126、[WBQ-SUM] 逐字行与 UNKNOWN=0、slot_violations=[]；malformed 0/303378（QG2）；[WBQ] 303378 行 / enter 101126（Q4）；SELFCERT 全绿（result02）。Q5 关臂基线「沿用上臂」见 N7-INFO。

## N7（PASS，附 INFO）— 代码 diff 与 #188 形态覆盖

- 静态核读当前工作区版：`GRID = ON && getProperty("coreswap.formdrivegrid")` —— 关臂（无 -P）下 GRID=false，`serverTick` 守卫 `!DRIVE && !GRID` 精确退化为原 `!DRIVE`，grid 分支静态不可达，走廊路逐行为不变 ✓。build.gradle 映射行驼峰命名、位置（formdrive 行后）、注释带 block 标签 ✓。
- INFO-1：`[FP-ON]` 行在关臂下也新增 ` grid=false` 字段——日志格式面变化（非 worldgen 行为），design §2.2/§7-5 已预声明为扩展点，合规但应知。
- INFO-2：WbQualStats 未知 ev 兜底 ENTER→UNKNOWN 是对既有臂语义的修改（仅在未知 ev 出现时可见，设计上不应发生），SUM 行格式随之加 UNKNOWN 字段——比较器同步消费，闭环 ✓。
- INFO-3：Q5 关臂基线沿用**改动前代码**的上臂 log，严格说未测新代码的关臂路径；#188 静态核读兜底，可接受；如需严格 Q5 可日后跑一次无 -P 的 smoke 臂，非本块义务。

## N8（MUST，措辞收窄）— C2「level≤32 ticket 覆盖的直接函数」超出证据

- D1/D2/D2' 直接证明的命题是：「驱动 workload/ticket 覆盖面 ⇒ light() 被调 ⇒ 缺席随覆盖消灭」——即 **workload/ticket 覆盖面直接函数**。「level ≤ 32」这一具体机制来自 T5/T6 静态推演（design §1.1 自标 I，未运行时单验；本臂没有任何「覆盖在位但 level 档不达」的对照臂去独立行使 T5）。verdict §7 环 3 也承认 T5/T6「仍标 I」。
- **意见：采纳收窄建议**——升级措辞改为「light() 覆盖 = workload/ticket 覆盖面的直接函数」；「level≤32 机制」保留 I 标注，待运行时单验（如 level 33 边界臂）后再升。当前措辞若进主题篇，会把静态推演混入有 D1 直接证据的结论，违反分级纪律。

## N9（SHOULD）— B6-1 门禁运行证据未落 record（本审查已代跑补证）

- design §2.2 要求合入后 MUST 跑 `check_switch_mapping.py`，record 无运行记录。本审查已执行：`formdrivegrid` 未出现在任何 DEAD/缺口/ORPHAN 段（映射生效、族内驼峰对称）；`--strict` rc=1 来自**既有已登记缺口**（1.20.1 缺口 13 / 1.21.6 缺口 11，与 AGENTS 在册数字同族），非本改动引入。建议 record 回填一行运行证据。

## N10（INFO）— exit 5 归因勿混淆

- verdict §5 把 exit_code=5 读作 #210 预登记「H1/H2 丧失类分母 0」退化读法（cmp2 h1_h2.overall="not-judgeable" 互证，成立）；但 §3.3 字面上 cov<0.90 的终点**也是** exit 5。本臂 exit 5 实际触发路径是 H1/H2（比较器 cov 按 face_hist 算=1.0，未内部触发 cov 门）；cov 门失配只在 d1 复算中被发现。两者终态巧合同号，归因应分开登记，避免后世误读「cov 门已由比较器执行」。

---

## 条件清单（candidate 授予前）

1. **N1**：cov(face_new)=0.8556 < 0.90 门失配——按预登记字面判据 suspended；「门意图满足、不 VOID」须以「预登记门偏差 + 人类确认 + 取代链登记」落地，不得以 worker 判读生效。
2. **N3**：verdict §4「其余 179」误引复算产物（机械=180，z188∩框外交 1），修正后随附 record 142 的口径误标勘误。
3. **N8**：C2 升级措辞收窄为「workload/ticket 覆盖面直接函数」，level≤32 保留 I。

推荐状态：verdict 保持 **draft**；上述条件解决后可建议 **candidate**（C2 主命题 + D1 ρ=1.0 读数），confirmed 留用户。
