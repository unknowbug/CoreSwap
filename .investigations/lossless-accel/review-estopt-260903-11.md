# judge 审查记录（260903-11 · est-opt 包，两次审查）

## P2 选型审查（SHOULD）：PASS（有条件）

- b2 主形态判死严密；收敛单线 b1 成立；G5 supersedes / K3（步长 8=8，size_vertical=2）/ K2（blend 常数化）独立复核通过。
- CONCERN：R3 K2 证据表述（`blending_active` 字段不存在 → 已补正引 density.rs:626-628）；R5 新增 D3 扫描域差异（est_at noise_height vs Aquifer height，nether 128<256）→ 已入 k3-k2-verdict。
- 结论：P2 选型建议 candidate。

## P5 交付审查（MUST，candidate 前）：PASS（4 CONCERN，无 BLOCK）→ 建议授予 candidate

- a) 三源一致 ✅（C1：快照「inserts≈2000」为外推无落盘——已修订为「64 chunks 实测 1914，e2e stats 未落盘」）
- b) 代码抽查 8 项全过：key 打包双射无碰撞、FIFO 淘汰同构、L2 读写优先级正确、i32::MAX 哨兵入 L2 合 Java computeIfAbsent、闭包借用无冲突、OnceLock 线程安全（epoch 挂 handle 为合法简化）、门控 chunk 级、默认路径零回归（C2：证据载体 = 64-chunk 聚合 hash + stash 基线，非 block_probe 全量 diff——已标注）
- c) e2e 收益超微测上界处理诚实（观察非定论）✅
- d) shared 臂处置合规（默认关 + 翻默认前置 Java 逐位验证）✅
- e) C3：L1-L3 分层用例与 shared 臂 e2e 计时未执行未声明——已补声明，列入翻默认前置
- f) 无 BLOCK

**授予范围**：本包「默认关」交付（b1-b EstL2 + b1-a 门控 + 探针）= candidate。翻 WG_EST_L2 默认开（前置：mt_fill Mutex 基线 + 大 region 淘汰 + e2e l2 stats 落盘）与翻 WG_EST_SHARED 默认开（前置：Java 逐位验证 D1 + 角列陡变用例）均不在授予范围。confirmed 留用户。
