# est L2 翻默认前置三件套 + 剩余差归因（P2，260903-12）

- id: `re-code:lossless-accel:est-l2-defaultflip-p2-260903-12`
- session: 260903-12
- status: **candidate**
- 验证分层: Full（运行时 bench 采集，生产门控路径）
- §9.7 三要素: 载体 = estopt_mt_bench（16×16 region 200,200 完整管线整批 wall + 工作队列多线程）/ est_price_probe（自持 init DF 扫描循环）；覆盖面 = seed 8576294172403134396、off/l2 双臂交错双跑（#24）、T=1/2/4/8；历史口径 = 与 260903-11 pc_e2e/qaq1_surf_probe 同 region 同 seed（微测形态不同不可直接比，见 P2.4）。

## P2.1 Mutex 争用基线（estopt-mt-baseline-260903-12.txt）

| T | off 吞吐 (ms/chunk) | l2 吞吐 | L2 加速比 |
|---|---|---|---|
| 1 | 91.4 / 94.0 | 35.8 / 36.0 | 2.55× |
| 2 | 49.3 / 48.7 | 17.8 / 18.0 | 2.75× |
| 4 | 25.9 / 26.1 | 8.3 / 8.4 | 3.12× |
| 8 | 14.3 / 14.0 | 4.5 / 4.6 | 3.12× |

- **无 Mutex 争用退化**：L2 加速比随线程数不降反升（2.55→3.12×）；交错双跑偏差 <3%。
- 线程扩展：off 6.5×、l2 7.9×（8 线程）——L2 臂扩展性更好（est 串行扫描占比消失）。
- 注意：本 bench T=1 吞吐 91ms/chunk vs pc_e2e_bench 同 region 76ms——预热/调度形态差异，两 bench 各自内部 A/B 有效，跨 bench 绝对值不可比（§9.7 声明）。

## P2.2 大 region 淘汰行为（estopt-sweep-260903-12.txt）

- 64×64=4096 chunk 扫描至 2304 chunk（inserts 累计 ~39.7k ≈ FIFO 上限 131072 的 ~30%，实测 ~17 条/chunk）：**命中率稳定 92±1%、无淘汰、每 256-chunk 块 wall 平稳（9.0-9.6s）**。
- 上限按 ~17 entries/chunk 投影在 **~7600+ chunk** 处触发（judge B 复算修正，原 ~4370 估计偏低）；typical region（16×16=256）远低于。**淘汰风险不构成翻默认障碍**（64×64 极端 region 也未触顶）。
- ⚠️ sweep 在 ~2304-2560 chunk 处 panic（surface_rules.rs:505 missing noise sampler）——**数据截止于此；4096 chunk 全程未完成**。panic 另立课题（见 est-shared-verdict 附带发现），不影响本结论在 typical region 的有效性。

## P2.3 e2e l2 stats 落盘（judge C1）

- mt T=1 l2 臂（256 chunks）：hits=51627 misses=5872 inserts=5872 evictions=0（89.8%）；off/l2/吞吐逐条落盘（estopt-mt-baseline-260903-12.txt），每条量化声明可溯源。

## P2.4 剩余差归因（e2e 收益 vs 微测上界，#21）

- 生产隐含 est 单价跨 session 稳定：260903-11 48ms/(7342−1715 iter)≈**8.5µs/iter**；本 session mt 差 55.6ms/5627 iter≈**9.9µs/iter**。单价稳定 → 次级效应候选（b2）弱化，未构成并存互斥候选，fan-out 免触发（判定依据：单价稳定性核算）。
- 决定性微探针（est-price-p24-260903-12.txt）：同一段扫描代码，**hot 单列重复 ~60ns/iter vs cold 生产形态（顺序新列）~5.7µs/iter，形态差 ~95×**。
- **结论：微测上界（2117ns/iter 形态）外推生产无效（#21 量化实锤）；冷形态单价 5.7µs 与生产隐含 8.5-9.9µs 同量级，剩余 ~1.5×（生产侧 aquifer/缓存压力）为 Partial 解释，已声明。** 观察闭合。

## 翻默认建议（供用户拍板，本 session 不翻）

前置条件已满足度：
1. ✅ 默认路径零回归（P0 off 臂 hash = HEAD 基线）
2. ✅ Mutex 争用无退化（P2.1）
3. ✅ 大 region 淘汰无风险（P2.2，typical region 远未触顶）
4. ✅ e2e stats 落盘（P2.3）
5. ⚠️ 建议与 shared 臂裁决联动：先落「角参数 +15→+16」修正（est-shared-verdict 新发现，两臂共有）再一起翻默认，一次到位对齐 Java SURFACE。
