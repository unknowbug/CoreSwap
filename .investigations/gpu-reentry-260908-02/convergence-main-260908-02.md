# GPU 介入点重勘探——主会话收敛纪要（260908-02）

> 输入：scout-a-cpu-cost-profile.md / scout-b-c2me-coverage.md / scout-c-criteria-candidates.md（本目录）
> 性质：勘探收敛 + 方向建议（draft），非立项决定——立项须另走 Phase 0 + judge + 用户拍板。

## 一、核心事实（三源交叉）

1. **此前三次 GPU 失败全部集中在 density/interp 底座（14.4ms/chunk，占 Rust FULL 62ms 的 ~23%）**——aquifer ~35-37ms/chunk（**~60%，最大头**）从未被 GPU 课题评估过（scout-a）。
2. **C2ME 有 AQUIFER_PREFILL / BIOME_MULTINOISE / ESTIMATE_SURFACE / FLAT_CACHE_PREFILL 四类 kernel 的存在性证据**（外部可行性形态参照），我们从未评估过（scout-b）。
3. **D25 的「角点分组结构不兼容」有外部已验证解**——C2ME INTERPOLATOR_PREFILL = 每工作项 1 角点 × 1 次 delegate 参数化坐标，即 D25 要求的「共享实例 + 每点坐标」形态（scout-b）。
4. D27 判死的条件是「解释器式 kernel + 现引擎形态」；判据框架（J1-J12）确认**专用 kernel 实测性能是唯一纸面不可判项**（J6），需最小原型 micro-bench（scout-c）。

## 二、关键核算：aquifer 的回本门槛与 density 完全不同（judge C2b/C3a/C3b 修订后）

- D27 的 38×/170× 门槛来自「density 底座 CPU 8 线程仅 4.5ms/chunk + 摊销预算 <1ms」——分母极小，门槛极高。
- **aquifer CPU 侧 ~35-37ms/chunk**（est ~15.4 + 冷 miss 6-8 + 暖 apply ~5.5，两行口径不同源，合计 ≈28 ≠ 35-37，残差未分解——judge C2a）：「GPU 只需快过 CPU 即正收益、门槛 ~1× 量级」**为单线程 bench 口径的上界估计**（judge C2b）；多线程 aquifer 分母无数据（C++ 侧 aquifer T=8 曾现反扩张，警讯）。且该估算**未含固定成本**（dispatch/readback 非零且 D27 结论绑定角点+解释器形态不可迁移；上传量未知；生产管线内 aquifer 与多线程并存的 mutex 串行化开销——D24 P2-4 实锤，J10 未设计——可能直接吃掉收益，judge C3b）。
- ⚠️ **「est 每点输入是坐标派生量、非 8672 floats split」是待验假设，非已证事实**（judge C3a）：scout-b 的 AQUIFER_PREFILL 记录深度仅名字、scout-c B2-J1 标 ⚠️ 无实测——J1 数据量判据须在 Phase 0 用 C2ME 源码 emitter 调研 + est 输入分解实测钉死后才能打分。

## 三、候选分档（判据对照 + 成本占比）

| 档 | 候选 | 依据 | 前置 |
|---|---|---|---|
| **建议进 Phase 0** | B2 aquifer GPU 化（est 冷扫描/prefill 形态） | CPU 占比 60%（J8 满分——注：占比数字来自 qpd1_stage_bench 隔离 bench 口径，非 WG_PHASETICK 载体，§9.7 随行声明，judge C5）+ C2ME AQUIFER_PREFILL 存在性 + 单线程口径门槛上界 ~1×（见 §二 修订）；J1 数据量/J10 并存开销 = 待验假设不打分 | C2ME 源码 emitter 调研 + aquifer est 每点输入分解 + 多线程分母实测 + 最小原型 micro-bench（J6 唯一硬未知） |
| 需先补实测再定 | B3 biome（无任何计时行）；feature/树花（Java 侧隐含 ~10ms，Rust 侧无独立实测） | 占比未知，J8 无法打 | 一轮 stage bench 补计时（便宜） |
| 顺延观察 | B5 surface（5.5-6.7ms）/ B6 carver（5-6.5ms，b4 机械成本反直觉未定位） | 占比 ~10% 偏小，且 carver 有未定位的 CPU 侧怪象（全 Air 列更慢）——**先查 CPU 侧再谈 GPU** | carver b4 根因定位（CPU 课题） |
| 维持否决 | B7 feature 放置（J3/J4 红灯）/ B8 光照（维持不立项简记） | 判据 + 用户已有裁决 | 无 |
| 附注 | B1 density 专用 kernel 重设计 / B9 离线预生成 | verdict 留白项，维持「新立项级」定性；若 B2 走通可复用其基建 | 不在本轮建议内 |

## 四、备选提醒（诚实面）

aquifer 60% 占比本身就值得**CPU 侧攻关**（est-L2 缓存优化先例：density 单线程 882→27.69 的路径）——CPU 优化与 GPU 化是两条竞争路线，Phase 0 应并列评估，不是 GPU 单选题。

## 五、纪律声明

- 本纪要为 draft；judge 已审（review-convergence-260908-02.md，PASS-with-conditions，条件 C3a/C3b/C2b/C5/C2a 已应用，C1 见本条）；立项决定 = 用户 HOOK。
- 范围声明（judge C1）：verdict-260908-01 的 confirmed 范围 = 「noise/density 管线现引擎形态接入」负面结论；本纪要所有候选均在 verdict 选项③留白（特定子管线/离线场景）内操作，**不构成对该结论的取代**。
- §9.7：scout-a 各口径（C++ PHASETICK 50 / Rust FULL 62 / 无树花 45.48 / Java 55）互不可比，引用数字须带口径；scout-c 盲区①「stage 占比无实测」与 scout-a 已有阶段耗时表的矛盾系两 scout 独立采样窗口差——以 scout-a 的 qpd1_stage_bench 实测为准（载体口径已随行标注）。
