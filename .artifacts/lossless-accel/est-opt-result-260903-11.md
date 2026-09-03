# est-opt 结果（260903-11 · candidate，judge PASS 260903-11）

> 任务：est 查表化优化包（Q-AQ1 后继）。方案：b1-a est_at 共享（门控默认关）+ b1-b 跨 chunk est L2（门控默认关）。
> 状态：**candidate**（judge PASS 4 CONCERN，R 项已修订；confirmed 待用户拍板）。

## judge 修订（260903-11）

- [C1] L2 stats 口径修正：64 chunks 实测 inserts=1914（ab-arms 落盘）；**256-chunk e2e 的 [l2] 行未落盘**，原「≈2000」为外推表述作废。
- [C2] 零回归证据载体标注：off 臂 == HEAD 基线为 **64-chunk 聚合 FNV hash 相等 + git stash 重建基线**（同确定性管线下 hash 相等），非 block_probe 全量逐字节 diff 口径。
- [C3] 未执行清单显式声明：① shared/shared+l2 臂无 e2e 计时落盘（只 off vs l2）；② b1 设计 L1 海洋深列哨兵专项 / L2 角列陡变区 / L3 32×32 邻域 block_probe 分层用例未执行（被 64-chunk 聚合 hash A/B 替代，对零回归目的可接受）；以上列入翻默认前置。

## 交付

1. **b1-b 跨 chunk est L2**（aquifer.rs `EstL2`）：精确值缓存（key=量化列 bx,bz；FIFO 淘汰 131072 条硬上限；代际隔离=挂 WorldgenHandle；blend 闸门 `BLEND_ACTIVE=false` 常量 + 未来置真全量旁路）；注入走 `Aquifer::set_est_l2`（构造签名不变，30+ 调用点零改动）；env 门控 `WG_EST_L2`（chunk 级判一次）；统计 `est_l2_stats()` [hits,misses,inserts,evictions]。
2. **b1-a est_at 共享**：`WG_EST_SHARED` 门控（默认关），开启后 SURFACE est_at 走 `va.aq.estimate_surface_height`（对齐 Java ChunkNoiseSampler.java:222-226 语义）。
3. 探针：bin-diag/estopt_ab.rs（四臂 hash A/B + L2 统计）；.tmp/estopt_hash.rs（HEAD 基线）。

## 验证（§9.7 三要素：载体=WorldgenHandle fill_chunk_blocks 全管线；覆盖面=64 chunks A/B + 256 chunks e2e region(200,200) seed 8576294172403134396；与历史口径=同 pc_e2e 260903-08 口径可比）

| 验证项 | 结果 |
|---|---|
| 默认路径零回归 | off 臂 hash `74f5dfc4eede8ef4` == HEAD 基线（git stash a3c0154 重建）**64-chunk 聚合 hash 相等**（载体标注见 judge C2）✅ |
| L2 精确性 | l2 臂 hash == off 臂 **逐位一致**（同值证明：淘汰/重算同值）✅ |
| L2 性能（16 chunks） | est 迭代 7342→1715/chunk（−76.6%），median 70.54→27.22ms |
| L2 命中率 | 84.9%（64 chunks 实测，inserts=1914；256-chunk e2e stats 未落盘，见 C1） |
| e2e 大样本（256 chunks） | median **75.94→27.69ms（−63.5%）**，avg 79.77→29.50ms ✅ 显著超预期 |
| 共享臂语义变化 | hash 变化（`8bff4087…`）——D1 角列量化（旧 est_at +15 直采为 **Java 发散点**）+ D3 扫描域；默认关，翻默认前 MUST 单独过 Java 逐位验证 |

## 观察与未闭合

- **e2e 收益（−48ms/chunk）超 est 微测上界（15.5ms）**：生产冷路径 est 实际单价 ≈11µs/iter（vs 微测 2117ns）——微测形态未复刻生产 working set（pattern #21 同族），以生产实测为准；不反推机制定论（未逐项归因剩余差）。
- shared 臂（D1 修正）可能修正潜在 surface 错位 bug——需 Java 逐位对比裁决，独立小包。
- b1-b 翻默认前置（未做）：多线程 Mutex 争用基线（mt_fill，b1 文档 R6 硬前置）+ 大 region 淘汰行为。
- nether（noise_height≠height）路径未收敛 est_at（D3）：生产仅 overworld，显式声明。

## 产物索引

- 设计：est-opt/candidate-b1-cache-lifecycle.md / candidate-b2-coarse-table.md / java-est-cache-semantics.md / k3-k2-verdict-260903-11.md
- 数据：cmd-output/estopt-ab-arms-260903-11.txt / estopt-perf-260903-11.txt / p0-handover-verify-260903-11.md
- 改动：WorldgenRust/src/aquifer.rs（EstL2+闸门+L2 读写点）、worldgen_handle.rs（est_l2 字段+helper+两处门控）、bin-diag/estopt_ab.rs（新）
