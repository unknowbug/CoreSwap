# Q-AQ1 数据层证据包（260903-10，fan-out 输入）

口径（§9.7）：seed=8576294172403134396 / region (200,200) / 64 或 16 chunks / Rust 生产 fill_chunk_blocks，OFF 配置。同 Q-PD1 62ms 口径族。

## 已验证事实（数据层实测，可继承）

- F1 段差分（两轮稳定，qpd1_stage_bench）：FULL=60.43；aquifer 段 = 48.84−13.77 = **35.07ms/chunk**（no-ore vs no-aquifer）；density 底座=13.77。
- F2 生产计数器（qaq1_counter_probe，16 chunks，全段开）：
  - get_block_pos ≈ **815,747/chunk**（bp miss 仅 ~281/chunk）
  - get_water_level_at ≈ **110,161/chunk**（miss 174/chunk）
  - barrier.sample 仅 ~10/chunk（calculate_density 的 barrier 分支罕见）
  - 两批线性稳定（#20 自检过）
- F3 apply 邻域循环 = 12 格/次（2×3×2），故 apply 总调用 ≈ 815k/12 ≈ **68k/chunk**（含 carver 的 aq.apply(x,y,z,0.0)——carver.rs L409 对每雕刻点全量 apply，与 classify 共享计数器）。
- F4 diag 微测分解（qaq1_apply_breakdown，独立构建 DensityBuilder 树，同 seed/region）：
  - 12 格 bp+距离：2.52ms/chunk（@1.18M 次）
  - get_water_level_at 1×/点：3.68ms/chunk（@98304）
  - calculate_density fluid logic：≈0
  - apply 完整（逐点树采样 final_density + d≤0 调 apply，applied=64,433/chunk）：13.51ms/chunk
  - **[R1 supersedes 前向指针 260903-10 judge]：本条 t_fl=4.44ms 为暖态值；摘要漏收该行曾误导缺口归因（R3：是误导源非缺口主体）。量化后被 qaq1-attribution-260903-10.md R2 调和口径取代。**
- F5 initial_density 单次树采样 = 0.0893µs；生产 surf est = 214 调用 × 34.35 迭代 = 7342 samples/chunk ≈ **0.66ms/chunk**（冷 surface_cache 假设已被否）。
  **[R1 supersedes 前向指针 260903-10 judge]：F5 的 0.0893µs 基线作废（样本模式命中缓存，与 est 扫描形态差 40×）；现行口径见 qaq1-attribution-260903-10.md 结论 1①（2117ns/iter 新鲜进程实测）。workflow-patterns #21。**
- F6 生产 classify 只对 d≤0 调 aq.apply（terrain.rs L222-233）；skip_aquifer 时 classify 直接返回 Air；后处理（ore/surface/carver/features）与 aquifer ON/OFF 无代码分支耦合，但 **carver 的 getState 直调 aq.apply（绕过 va.skip_aquifer 标志）**。
- F7 每点数：98,304（16×16×384）；d≤0 点 ≈ 64-68k。
- F8 基线带：FULL 60-77ms（08/09/10 三日）；本次 60.43 ✓。

## 未闭合缺口（核心谜题）

**G1**：生产 aquifer 段 35.07ms vs diag 可解释内部成本合计 ~4-6ms/bp+wl+surf（F4-F5 口径）→ **~29ms/chunk 未归因**。
- 每 apply 隐含成本 ≈ 35ms/68k ≈ 515ns（生产）vs diag 内部 ≈ 90ns/apply → 6× 差。
- 注意 F4 的 apply 完整 13.51ms 含逐点 final_density 树采样（生产用 macro cell-grid 插值，不同载体，不可直接相减）。

## 互斥候选（fan-out 分叉）

- b1（生产 apply 更贵）：生产 Aquifer/密度函数构建与 diag 独立构建存在实现差异（NoiseSet/共享状态/缓存互扰/macro 采样器交互），使生产 apply 单次 ~6× 贵。
- b2（测量级联/语义）：35ms 差分被非 classify 因素污染——carver 雕刻点的 aq.apply（绕 skip 标志）、macro cell-grid 在 aquifer ON/OFF 下行为差、WG_SKIP_* env 读取路径、或二阶级联（Q-PD1 caveat b）。
- b3（计数盲区）：存在未计数的 aquifer 相关热路径成本（计数器只盖 bp/wl/barrier/surf 四处）。

## 约束

- subagent 无 shell：探针命令由主会话执行，worker 只设计/解读。
- 诊断 env 门控默认关；诊断 bin 落 bin-diag；热路径不加每点探针。
- 现成工具：WG_AQUIFERCOUNT/WL/BP/SURF 计数器、qpd1_stage_bench、bin-diag 系列、FLAG_SKIP_* atomic flags（worldgen_handle）。
