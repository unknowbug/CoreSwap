# judge 审查意见 — 260909-05 ca_min memo/优化立项 fan-out 裁决链
judge: core.judge（只出意见，不改 status；confirmed 留人类）
三源核对：① .artifacts/camin-perf/memo-decision-26090905.b{1,2,3}.yaml ② git 工作区 diff（worldgen_handle.rs WG_CA_MEMODIAG，未提交）③ memo-review-260909-05.md + perf-record/scout-map-260909-04 + #100/#101

## 总裁定：PASS-with-conditions

## 逐项
a) 判别数据支持性 — PASS。桶合计 1343+46827+22593+3159=73,922=唯一列合计（内部自洽）；216,826,945/257≈843.6k 读/chunk，与 E4b 853k 吻合。257 含 warmup chunk(0,0) 双计：分母偏差 ~0.4%，对 98%/288/pending_writes=0 结论无实质影响（建议记录一行声明）。pending_writes=0 为单 region 单样本，B2 已附 §9.7 外推声明，合规。
b) B3 P1 算术修正 — 修正正确且必要：73,922 确为唯一列合计（桶和自洽可证），B3 以其为读数得「P1 仅 9%」不成立；216.8M×51ns≈43ms/chunk 才与残差吻合。主会话同时指出 51ns=44ms÷853k 为摊衔值可能含 miss 重生成摊入——诚实，且正确推出「P1/P2 占比未分解」→ 前置探针合理。
c) 收敛裁定 — 方向 PASS，但有条件（见下）。B3 变体 a 覆盖 B1 收益面（2-4ns vs 15-30ns）且多消 P2，采 B3 为主成立；B1 降补充件理由成立（仅当探针示 3×3 覆盖<100% 时兜底有值，不应双实现并行起步）。B2「收窄」+ §15.4 取代链适配正确（#101 正文不改、取代记录表达、subagent 草稿落盘），且 B2 对三证逐证核对质量高。
d) 探针/硬门 — 判据可证伪性成立（miss≈288 → P2 实锤 / miss≪288 → 回 fan-out）；硬门四件齐（hash 双臂、wall A/B 目标 on≈off±5%、workspace 全量绿、§9.7 声明）。diff 与记录一致；诊断门控合规：env::var 每 chunk 一次、关臂每读仅一个 bool 分支，不违反诊断污染铁律；hash 恒等 6908dbfc 有记录引用。
e) 降级声明 — 三候选均标 Degraded/静态+成本模型，收敛记录证据分层诚实，无升格。

## 条件（进实现轮前 MUST 落实）
1. **E2b 冲突未核对（最重要）**：B3 的 P2（CAP 256 clear-all 雪崩重生成 ≈30-43ms）与 260909-04 E2b 实测直接矛盾——WG_CA_CAP 256 vs 2048 差 <2%（178.6 vs 176.5ms），当时已「排除 C2」。若 P2 主导，CAP 2048 臂应大幅回落而未。收敛记录必须显式记录此张力；前置探针判据建议加一条廉价臂：CAP=2048 A/B（或 miss 计数 + 单次 fill_terrain_column 直接计时），防止复活一条已被既有数据削弱的假说。P1/P2 分解解读须防循环论证（51ns 本身由残差导出）。
2. warmup 双计与 pending_writes=0 的口径声明补一行进 memo-review（单 region 样本、含 warmup 分母）。
3. #101 §15.4 取代记录落盘前按知识库强制触发点走 subagent 草稿（收敛记录已列，执行时勿由主会话直写）。

## 建议
- 探针可再廉价化：miss 计数 + (tcx-cx,tcz-cz) 直方图 + fill_terrain_column 计时一次采齐，一轮 bench 内闭环。
- B1 的 risk-3（不变量注释钉死 IDK-cA1 耦合）在 B3 实现中同样适用，实现轮迁移该条。
- B2 的「单槽 last-value memo」作为探针后兜底备选保留引用即可，不另立候选。
