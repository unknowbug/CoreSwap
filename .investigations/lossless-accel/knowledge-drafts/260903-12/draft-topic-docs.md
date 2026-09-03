# 草稿：主题篇追加小节（est 优化/shared 裁决/gpu-merge，subagent 产出，主会话应用）

> **落点核查结论**：已查 `versions/1.20.1/docs/`——260903-11 est 优化结论**未落任何主题篇**（git 367de35/1a2967f/0949402 显示其只进 `.artifacts/est-opt-result-260903-11.md` + 10 时间线 + `.investigations/est-opt/`；04 篇 est 节停留在 C++ 时代「两版一致」，06 篇 est 节无 L2/shared 内容）。
> **建议载体**：`versions/1.20.1/docs/07-block-pipeline.md`——该篇已承载「Rust worldgen 端到端性能定位」系列性能结论（含 aquifer 段 22%、pc_e2e、Q-PD1 归因），本节为其自然延续；若主会话判断 est 语义修正偏 04/06 篇，可将「裁决结论」小节拆至 04-aquifer.md（est 扫描/含 aquifer 域）——二选一，**推荐 07 篇**（性能课题归口一篇，避免 est 语义细节拆散）。
> **应用位置**：07 篇「Rust worldgen 端到端性能定位」系列小节之后（文件相应位置末尾追加）。追加不覆盖。所有结论标注 candidate，confirmed 留用户。

---

## 2026-09-03 est 优化收口（追加小节草稿，260903-12）：shared 臂裁决 candidate + 翻默认前置达成 + gpu-batch-merge 降级建议

> 承接本篇「Q-PD1/Q-AQ1」归因与 260903-11 est 查表化优化包（est_at 共享 + 跨 chunk est L2，commit 0949402，门控默认关）。本轮 Full 层运行时证据裁决 shared 臂语义 + 清偿翻默认前置。产物：`.artifacts/lossless-accel/{est-shared-verdict,est-l2-defaultflip-p2,gpu-merge-revisit}-260903-12.md`（均 candidate）；judge 两轮（review-est-shared / review-p2-p3-final，建议 confirmed 前清偿文档级修正）。

### ✅ shared 臂裁决结论（candidate）：shared = 修正，off = 系统性偏离

- **验证分层 Full**：Java `ChunkNoiseSampler.estimateSurfaceHeight` mixin RETURN dump vs Rust `WG_EST_DUMP` 角值 dump，同 seed 8576294172403134396 同 region (200,200) 64 chunks（§9.7 三要素见 verdict 头部）。
- **结论**：共同列（c0 原点角，Java dump 列 residue 无 residue-12，其余 3 角列无对应列——覆盖面精确表述 per judge A2）**shared 64/64 与 Java 逐值一致；off 0/64 全偏且 delta 恒 −1**；角列敏感性 63/64（唯一敏感 chunk (201,200)：java@+16=56 / shared@+12=48 / off=55）。Java 表 11877 条 conflicts=0（est 为量化列纯函数）。
- **⚠️ 待办（翻 shared 默认的前置）**：Rust 两臂 heights4 参数 `cx*16+15`（量化 +12）≠ Java SURFACE 四角 `(i+1)<<4`=+16——改 +16（两臂独立小包）后完全对齐；量化敏感 chunk 约 1.6%~4.7%。
- **⚠️ 附带生产 bug 线索（judge A1，另立验证）**：off 臂扫描 `(min_y..min_y+noise_height).rev().step_by(8)` 半开区间 rev 首采样点 = 319 vs Java 320——**off 是当前默认臂，−1 系统偏移独立于翻默认决策**。

### ✅ 翻默认前置达成情况（est-l2-defaultflip-p2，candidate）

四项门控（260903-11 judge 预置）逐项清偿：

| 前置 | 结果 |
|---|---|
| 默认路径零回归 | ✅ off 臂 hash == HEAD 基线（74f5dfc4，stash 重建复跑） |
| Mutex 争用 | ✅ 无退化：L2 加速比 2.55→3.12× 随线程不降反升（T=1/2/4/8 交错双跑，偏差 <3%） |
| 大 region 淘汰 | ✅ 64×64 sweep 命中 92±1%、evictions=0；触顶投影 ~7600+ chunk（inserts ~40k，judge B 修正），typical region 远未触顶 |
| e2e l2 stats 落盘 | ✅ T=1 l2 hits/misses/inserts 逐条可溯源（89.8%） |

- 剩余差归因（P2.4）：est_price_probe 同代码 hot ~60ns/iter vs cold 5.7µs/iter（形态差 ~95×，workflow-patterns #21 量化实锤）；跨 session 生产隐含单价 8.5/9.9µs 稳定 → 剩余 ~1.5×（生产侧 aquifer/缓存压力）为 Partial 解释，已声明。
- ⚠️ sweep 在 ~2304-2560 chunk 处 panic（`surface_rules.rs:505 missing noise sampler`）——数据截止于此，panic 另立课题，不影响 typical region 结论。

### 📉 gpu-batch-merge 降级建议（candidate，待用户拍板）

- 新基线（§9.7 同 region/size，256 chunks median；**Java 侧为跨 bench 近似比较**，judge D 标注）：Rust off 75.94 / **l2 27.69** / l2 8 线程 ~4.5 ms/chunk vs Java FULL ~32-33。
- est L2 无损优化已消除立项时目标差距（260903-08：Rust 72-77 vs Java 33，慢 2.2×）→ 单线程恢复快 ~1.2× 方向（260903-10 大样本同向），8 线程 ≈ 7× Java（量级判断）。GPU 路径 dispatch/readback 成本（369ms/chunk）在小批量下为负收益。
- **建议**：gpu-batch-merge 降级/搁置（从待办移除或标注「暂无需求」）；未来目标改为「大幅超越 Java 10×+」再重议。

> 过程（P0 复现 → scout → 探针搭建 → 裁决 → 三件套 → P2.4 → 重议 → 两轮 judge）→ 10 时间线 260903-12 条；新课题四项（off −1 偏移 / surface_rules.rs:505 panic / +16 角修正 / 翻默认拍板）→ 10 时间线「新课题登记」。
