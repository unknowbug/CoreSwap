# judge review — k2a-writeback-probe-260920-04

- 总判定：**PASS-with-conditions**（建议 candidate；N1/N2 两项 MUST 修正后落定）
- 审查角色：anchor-judge（隔离，§15.4）；三源核对已执行（交付快照 / git 侧工作区实读 / 验证记录）
- 逐条抽验 ≥10 处全部一致（Q1 sha log:5442、Q3 armed log:152、Q7 sum log:39992、wbq_total=11109=3×3703、E-存在 1518+520=2038、H1H2 ratio 0.642 mixed、ledger 引例、Q2 log:5393+5460、Q5 smokeoff grep=0、Q6 log:5441）
- 插桩在位性：Mixin 20 处调用实测在位（P1:1062/P2:1071/P4:572/P5:591/P6:583/P7:600/P8:756/808/P9:864/P10:875/legacy-fb 963/972/998/1052/P11:1014/P12:1096/legacy-enter 922）——packed/legacy 零行 = 未触发非未打点；P1 在 G0 门之后（info，与 idk-WBQ2 域定一致）

## MUST 条件（已应用到 verdict）
- **N1**：H1/H2 三处读法与 design §3.3 预登记漂移（zline 143 vs 119 / 期望平写 1/3 未按 dk 分布实算 / never-enter 未按最近批锚归属）→ 以声明退化形态补登记进 §1 H1/H2 行；judge 复算两口径均 mixed，结论不变。
- **N2**：verdict §1 Q2/Q5/Q6 原为「主会话读数」→ 替换为 file:line/json 锚（judge 代核实证）。

## SHOULD 建议
- **S1**：never-enter 归并方向 = 并入 #198 workload 驱动面候选，暂不新立 idk-K2c——FP-DRV 轨迹为确定性走廊（6 waypoint，cx 200→320 沿 cz=200），520 南带聚集很可能主要是驱动路径几何投影；廉价判别 = 全覆盖驱动一臂，残留缺席才立 idk-K2c。
- **S2**：比较器下轮加 FP-DRV 轨迹 ∩ face 覆盖率指标。
- **S3**：WbQualStats.counterOf 默认分支（:120）未知 ev 静默归 ENTER，建议改独立 UNKNOWN 计数器。

## 通过项
判据预登记-执行一致性（Q1 回填有预授权）、证据链闭合（含 d_self=2185=3703−1518 旁证自洽）、§9.7/§9.8 完备、无 confirmed 越权措辞、产物契约齐全。
