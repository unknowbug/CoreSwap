# core.judge 审查意见：convergence-main-260908-02（GPU 介入点重勘探收敛纪要）（260908-02）

> 审查角色：core.judge（subagent，只出意见不改 status；confirmed 留人类）。
> 审查对象：`.investigations/gpu-reentry-260908-02/convergence-main-260908-02.md`（draft）。
> 证据链：同目录 scout-a / scout-b / scout-c；`.artifacts/perf-rework/gpu-phase1-verdict-260908-01.md`（confirmed，负面）；`.investigations/perf-rework/gpu-accel-errors.md`（D24/D25/D27 及速查表）。
> **裁定：PASS-with-conditions**——方向性收敛（B2 aquifer 候选 + 分档 + 备选提醒）成立且合规，但门槛论述与一处数据声明存在过度承诺，须按 C1-C5 修订后才可进入 Phase 0 / 用户拍板。

---

## 审查点 1：范围合法性 —— **通过（附 C1 表述补强）**

- verdict-260908-01 的 confirmed 范围 = **「noise/density 管线 GPU 接入」现引擎形态，负面结论**（verdict L5）；其判定含义 L29 明示选项 ③ =「GPU 只服务特定子管线/离线预生成场景」——即**子管线/离线是显式留白**。
- 收敛纪要建议的 B2 aquifer、B3 biome、B5/B6 均为非 density 子管线，落在留白内；B1（density 专用 kernel）/B9（离线）纪要第 27 行维持「verdict 留白项 / 新立项级」定性，与 verdict L15「达门槛等价 C2ME 式专用 kernel 重设计（新立项级）」一致——**无 §15.4 取代冲突，无越界推翻 confirmed**。
- 纪要第 8 行「此前三次 GPU 失败全部集中在 density/interp 底座」与 gpu-accel-errors D24/D25/D27 主题一致（该文件 grep 确认无 aquifer 条目，scout-a §二同样确认）——引用准确。
- **C1（轻）**：纪要未逐字引用 verdict 的范围声明（L5「范围 = noise/density 管线 GPU 接入」）。建议在 §一或 §三补一行显式引用，把「为何 B2 不构成对 confirmed 结论的取代」说破，避免后续读者误判为重开已否课题。

## 审查点 2：数字口径（§9.7）—— **基本一致，两处瑕疵（C2a/C2b）**

逐一核对（scout-a 主表 A 为出处）：

| 纪要数字 | scout-a 出处 | 判定 |
|---|---|---|
| aquifer ~35-37ms/chunk，占 FULL ~60% | L15（qpd1_stage_bench 隔离 bench，两口径 37/35.07，FULL=62/60.43） | ✅ 一致 |
| density 14.4ms（~23%） | L16（同 bench） | ✅ 一致 |
| Rust FULL 62 | L15 口径注 | ✅ 一致 |
| est ~15.4 + 冷 miss 6-8 + 暖 apply ~5.5 | L20（Q-AQ1 归因，260903-10） | ⚠️ 见 C2a |
| biome 无实测 / feature 隐含 ~10ms 无独立实测 | L24/L25/L63 | ✅ 一致 |

- **C2a**：纪要 §二把 est 15.4 + 6-8 + 5.5 列为「aquifer CPU 侧 ~35-37ms」的构成，但三段合计 ≈27-29ms ≠ 35-37，残差未注明——scout-a 自己也未声明三段求和等于总量（两行口径不同：qpd1_stage_bench vs Q-AQ1 归因）。应补「构成分解来自另一口径（Q-AQ1），与总量差 ~7-9ms 未归因」一句，否则读者会误做加法。
- **C2b**：「回本门槛 ≈1×」的分母 = **单线程** qpd1 bench 的 35-37ms。这与 D27/verdict 的门槛核算方法不一致：D27 用 **8 线程吞吐 4.5ms** 做分母（两口径二分），且 verdict 口径注记明确生产 T=10、门槛随之抬高。多线程下 aquifer 每 chunk 吞吐分母**无数据**（C++ 侧警讯：aquifer+ore T=1 8ms → T=8 25-28ms 反扩张，scout-a 表 C）。门槛表述必须改写为「**单线程口径的上界估计**；生产多线程分母未知，需先实测」。
- 口径随行：§五有总声明，但 §二/§三表内关键数字（60%、~1×、35-37）未逐条带口径。§9.7 要求「同行声明」，建议至少在门槛句和 J8 满分处随行标注载体（qpd1_stage_bench、单线程）。

## 审查点 3：「回本门槛 ~1×」是否过度承诺 —— **是，两处（C3a/C3b），须修订**

- **C3a（最重要）：「每点输入是坐标派生量，非 8672 floats split」无 scout 材料支撑。**
  - scout-c B2 的 J1 明标 ⚠️：「aquifer 网格点数 × 每点数据量无实测（split 依赖未知）」；盲区 3 列明需「读 aquifer 实现算术 + 一轮探针」。
  - scout-b §四 AQUIFER_PREFILL 记录深度 = **浅（仅 kernel 名字）**，输入量/每点数据未记录（scout-b L43-46）；其待深入点 1 同样指向「需读 OpenCLCGen 源码才能回答量级问题」。
  - 即三份 scout 证据一致指向「aquifer 每点输入量 = 未知」，纪要 §二却把它写成既成事实（「J1/J4 形态好」并进 B2 分档依据）。**必须删除或降级为待验假设**，否则 B2 行的「J1/J4」判据分是凭空打分——这正是 J2/J1 判据自己禁止的形态（跨形态/无形态结论不可下）。
- **C3b：固定成本与互斥成本未纳入门槛式。**「GPU 只要算得比 CPU 快就正收益」隐含 dispatch/readback/上传/共存成本为零：
  - dispatch/readback：D27 证明其**非主项**（摊销封顶 13%）但**非零**——且该结论绑定角点形态 + 解释器 kernel（J2 形态限定），对 aquifer prefill 形态不可直接迁移；
  - 与 CPU 多线程共存：D24 P2-4 实锤——并发 fill 驱动崩溃（0xC0000005 @ nvtfi）→ mutex 串行化「正确性解决但性能更劣化」（gpu-accel-errors L598/L615-616/L627）；scout-c J10 对 B2 也是 ⚠️ 未设计。aquifer 在生产是 T=10 多线程管线内的一段，互斥串行化可能直接吃掉正收益。
  - 纪要 §二 ⚠️ 只诚实声明了「可达性无实测」，未覆盖以上两项。修订建议：门槛句改为「GPU 侧每 chunk 总成本（kernel + dispatch + readback + 上传 + 与 CPU 线程互斥开销）须低于 CPU 侧对应口径基线；单线程口径下粗门槛 ~1× 量级（上界），多线程口径分母未知」。

## 审查点 4：draft 状态与 HOOK 纪律 —— **通过**

- 纪要 L4「性质：勘探收敛 + 方向建议（draft），非立项决定——立项须另走 Phase 0 + judge + 用户拍板」+ §五「draft / SHOULD judge / 立项 = 用户 HOOK」——边界守住。
- B7/B8「维持否决」引用的是「判据 + 用户已有裁决」，是引用既有裁决而非自行 confirmed，合规。
- 「建议进 Phase 0」是建议动作而非立项决定，合规。无越权下结论。

## 审查点 5：scout 间矛盾的处理 —— **部分处理，需显式调和（C5）**

- 矛盾实体：scout-c 盲区 1 / B2-J8 称「Rust 侧 aquifer 阶段耗时占比**无 stage 级数字**（QPD1 只给了与 Java 的差值归因）」，而 scout-a 主表已给出 qpd1_stage_bench 的 stage 级隔离 bench 数字（35-37ms）。两者对「同一事实是否存在」直接冲突（可能 = Q-AQ1 归因与 qpd1_stage_bench 是两个不同 bench，scout-c 未见后者）。
- 纪要的处理方式 = **默默采用 scout-a 数字**并给 B2 标「J8 满分」，未声明：(a) 矛盾存在及裁决依据（以 scout-a 实测为准）；(b) 载体切换——J8 判据原文要求 WG_PHASETICK 口径，实际用的是 qpd1_stage_bench 隔离 bench 口径（§9.7 应随行声明）；(c) B3/B5/B6 行仍按 scout-c 盲区 1 处理（需补实测），与 B2 行的口径来源不一致。
- **C5**：纪要补一小节「scout-c ⚠️ → scout-a 实测的调和说明」：确认 qpd1_stage_bench 与 Q-AQ1 的关系/口径差异、声明 B2 的 J8 判定依据载体、并复核 scout-c 盲区 1 表中「aquifer 行可销、biome/feature 行仍开放」的修正。若 qpd1_stage_bench 覆盖 stage 不全（如 biome 无行），逐阶段标注哪些 J8 已有数、哪些仍缺。

---

## 结论与推荐状态

- **推荐状态：维持 draft**；本审查为 SHOULD 级（candidate 授予前）意见，修订 C1-C5 后可再走一轮轻量核对，然后交用户 HOOK 决定是否进 B2 Phase 0。
- 优先级：**C3a（无支撑数据声明，必改）> C3b（门槛式缺固定/互斥成本，必改）> C2b（门槛分母口径，必改）> C5（scout 矛盾显式调和，应改）> C2a（构成分解残差，应改）> C1（范围声明引用，可改）**。
- 方向面肯定：aquifer「从未被 GPU 课题评估 + CPU 最大头」是三源一致的新事实（scout-a §二 ❌ 行 + qpd1 表 + gpu-accel-errors 无条目），§四「CPU 攻关与 GPU 化并列评估、非 GPU 单选题」的提醒与 J6「门槛随 CPU 优化抬高」自洽——重议方向本身证据充分、值得进 Phase 0。

— core.judge（260908-02），只出意见，不改 status。
